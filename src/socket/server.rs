use super::Client;
use super::WaylandStream;
use super::shared::Event;
use crate::backends::BackendSender;
use crate::protocols::wayland::WlDisplayError;
use crate::socket::WaylandMessage;
use crate::socket::shared::SharedState;
use anyhow::{Context as _, Result};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::UnixListener;
use tokio::sync::Mutex;
use tracing::debug;
use tracing::error;

pub struct Server {
    path: PathBuf,
    listener: UnixListener,

    // TODO: use a proper id
    clients: Mutex<HashMap<usize, Client>>,
    last_client_id: AtomicUsize,

    backend_sender: BackendSender,

    // The mutex is needed to satisfy the borrow checker, but it will be used in only one place, so
    // no blocking will occur.
    client_receiver: Mutex<tokio::sync::mpsc::UnboundedReceiver<Event>>,
    shared_state: Arc<SharedState>,
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

    pub fn create(backend_sender: BackendSender) -> Result<(Self, String)> {
        let (path, env) = Self::get_new_socket_path().context("No socket found")?;
        let listener = UnixListener::bind(&path).context("Failed to bind socket")?;

        let (server_sender, client_receiver) = tokio::sync::mpsc::unbounded_channel();
        let shared_state = Arc::new(SharedState::new(server_sender));

        let wayland_socket = Self {
            listener,
            path,
            backend_sender,
            last_client_id: AtomicUsize::new(0),
            shared_state,
            clients: Default::default(),
            client_receiver: client_receiver.into(),
        };
        Ok((wayland_socket, env))
    }

    pub async fn run(self) -> Result<()> {
        let this = Arc::new(self);
        let tthis = Arc::clone(&this);
        tokio::spawn(async move {
            while let Some(event) = tthis.client_receiver.lock().await.recv().await {
                match event {
                    Event::Render(buffer) => {
                        tthis.backend_sender.draw(0, 0, buffer);
                    }
                    Event::Recalculate => {
                        if tthis.shared_state.is_recalculate_needed() {
                            tthis.shared_state.unset_recalculate_needed();
                            // FIXME: Make proper recalculation (eg: layout), and send configure
                            // requests to clients
                        }
                    }
                }
            }
        });
        // this.run_socket().await
        Self::run_socket(this).await
    }

    async fn run_socket(this: Arc<Self>) -> Result<()> {
        loop {
            let (stream, _) = this.listener.accept().await?;
            let id = this.last_client_id.fetch_add(1, Ordering::Relaxed);
            let stream = stream.into_std()?;
            let stream = WaylandStream::new(stream)?;
            let client = Client::new(id, stream, this.shared_state.clone());
            let this = Arc::clone(&this);
            tokio::spawn(async move {
                let mut clients = this.clients.lock().await;
                let client = clients.entry(id).or_insert(client);
                let result = Self::handle_connection(client)
                    .await
                    .context("Failed to handle connection");
                if let Err(e) = result {
                    error!("{e:?}");
                }
            });
        }
    }

    async fn handle_connection(client: &mut Client) -> Result<()> {
        let mut fds = VecDeque::new();
        loop {
            let Some(msg) = WaylandMessage::read(&mut client.stream).await? else {
                break;
            };
            debug!("Got message: {msg:?}");
            let sub_fds = client.stream.fds_mut();
            fds.extend(sub_fds.drain(..));
            let p = match client.get_object(msg.object_id) {
                Some(p) => p,
                None => {
                    client
                        .error(
                            msg.object_id,
                            WlDisplayError::InvalidObject as u32,
                            format!("Invalid object id: {}", msg.object_id),
                        )
                        .await;
                    continue;
                }
            };
            p.call(client, msg, &mut fds).await;
        }
        debug!("Connection closed");
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
