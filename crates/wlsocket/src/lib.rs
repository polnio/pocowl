pub use pocowl_wlvalue::WaylandValue;

use anyhow::{Context as _, Result};
use pocowl_protocols_base::WaylandProtocol;
use pocowl_wlclient::WaylandClient;
use pocowl_wlmessage::WaylandMessage;
use pocowl_wlstream::WaylandStream;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use tokio::net::UnixListener;

pub trait WaylandState {
    type ClientState: WaylandClientState;
    fn create_client(&self, client: WaylandClient) -> Self::ClientState;
}

pub trait WaylandClientState: Send {
    fn get_client_mut(&mut self) -> &mut WaylandClient;
    fn get_protocol_of_object(&self, id: u32) -> Option<Box<dyn WaylandProtocol<Self> + Send>>;
    fn on_invalid_object(&mut self, id: u32);
}

pub struct WaylandSocket<State: WaylandState> {
    path: PathBuf,
    listener: UnixListener,
    state: State,

    last_client_id: AtomicUsize,
}
impl<State: WaylandState> WaylandSocket<State> {
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

    pub fn create(state: State) -> Result<(Self, String)> {
        let (path, env) = Self::get_new_socket_path().context("No socket found")?;
        let listener = UnixListener::bind(&path).context("Failed to bind socket")?;
        let wayland_socket = Self {
            listener,
            path,
            state,

            last_client_id: AtomicUsize::new(0),
        };
        Ok((wayland_socket, env))
    }

    pub async fn run(&self)
    where
        <State as WaylandState>::ClientState: 'static,
    {
        loop {
            let socket = self.listener.accept().await;
            let (stream, _) = socket.unwrap();

            let id = self.last_client_id.fetch_add(1, Ordering::Relaxed);
            let stream = stream.into_std().unwrap();
            let stream = match WaylandStream::new(stream).context("Failed to create wayland stream")
            {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("{e:?}");
                    continue;
                }
            };
            let client = WaylandClient::new(id, stream);
            let client = self.state.create_client(client);

            tokio::spawn(async move {
                let result = Self::handle_connection(client)
                    .await
                    .context("Failed to handle connection");
                if let Err(e) = result {
                    eprintln!("{e:?}");
                }
            });
        }
    }

    async fn handle_connection(mut client: State::ClientState) -> Result<()> {
        let mut fds = VecDeque::new();
        loop {
            let Some(msg) = WaylandMessage::read(&mut client.get_client_mut().stream).await? else {
                break;
            };
            println!("Got message: {msg:?}");
            let sub_fds = client.get_client_mut().stream.fds();
            fds.extend(sub_fds.drain(..));
            let p = match client.get_protocol_of_object(msg.object_id) {
                Some(p) => p,
                None => {
                    client.on_invalid_object(msg.object_id);
                    continue;
                }
            };
            p.call(&mut client, msg, &mut fds).await;
        }
        println!("Connection closed");
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl<State: WaylandState> Drop for WaylandSocket<State> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
