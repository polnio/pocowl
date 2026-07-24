use crate::socket::{WaylandMessage, WaylandStream};
use crate::{AppHandle, ClientId};
use anyhow::{Context as _, Result};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;
use tracing::{debug, error};

#[derive(Clone)]
struct ServerState {
    app_handle: AppHandle,
}

pub struct Server {
    path: PathBuf,
    listener: UnixListener,
    state: ServerState,
}

impl Server {
    fn get_new_socket_path() -> Option<(PathBuf, String)> {
        let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")?;
        let runtime_dir = PathBuf::from(runtime_dir);
        for display in 1..10 {
            let path = runtime_dir.join(format!("wayland-{}", display));
            if !path.exists() {
                let env = format!("WAYLAND_DISPLAY=wayland-{}", display);
                return Some((path, env));
            }
        }
        None
    }

    pub fn create(app_handle: AppHandle) -> Result<(Self, String)> {
        let (path, env) = Self::get_new_socket_path().context("No socket found")?;
        let listener = UnixListener::bind(&path).context("Failed to bind socket")?;
        let state = ServerState { app_handle };
        let server = Self {
            listener,
            path,
            state,
        };
        Ok((server, env))
    }

    pub async fn run(self) -> Result<()> {
        loop {
            let (stream, _) = self.listener.accept().await?;
            let stream = stream.into_std()?;
            let stream = WaylandStream::new(stream)?;

            // Cloning is fine, since this is a ref counter
            let state = self.state.clone();
            tokio::spawn(async move {
                let result = Self::handle_connection(stream, state)
                    .await
                    .context("Failed to handle connection");
                if let Err(e) = result {
                    error!("{e:?}");
                }
            });
        }
    }

    async fn handle_connection(mut stream: WaylandStream, mut state: ServerState) -> Result<()> {
        let (id, mut rx) = state.app_handle.connection().await;
        loop {
            tokio::select! {
                resp = rx.recv() => {
                    let Some(resp) = resp else {
                        break;
                    };
                    Self::handle_response(&mut stream, resp).await;
                }
                msg = WaylandMessage::read(&mut stream) => {
                    let Some(msg) = msg? else {
                        break;
                    };
                    Self::handle_msg(&mut stream, &mut state, id, msg).await;
                }
            }
        }
        debug!("Connection closed");
        Ok(())
    }
    async fn handle_response(stream: &mut WaylandStream, resp: WaylandMessage) {
        let result = stream
            .write_all(&resp.into_raw())
            .await
            .context("Failed to send response");
        if let Err(e) = result {
            error!("{e:?}");
        };
    }
    async fn handle_msg(
        stream: &mut WaylandStream,
        state: &mut ServerState,
        id: ClientId,
        msg: WaylandMessage,
    ) {
        debug!("Got message: {msg:?}");
        stream
            .fds_mut()
            .drain(..)
            .for_each(|fd| state.app_handle.fd_attached(id, fd));

        state.app_handle.message(id, msg);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
