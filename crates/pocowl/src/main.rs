mod protocols;

use anyhow::Result;
use pocowl_backend::{Backend, BackendSender};
use pocowl_protocols::WaylandProtocol;
use pocowl_protocols::wayland::{WlDisplay, WlDisplayError};
use pocowl_wlclient::WaylandClient;
use pocowl_wlsocket::{WaylandClientState, WaylandSocket, WaylandState};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncWriteExt as _;
use tokio::runtime::Handle;
use tokio_util::sync::CancellationToken;

use crate::protocols::wayland::PocoWlState;
use crate::protocols::xdg_shell::PocoXdgShellState;

pub const DISPLAY_OBJECT: WlDisplay = WlDisplay { object_id: 1 };

pub struct PocoWlClient {
    client: WaylandClient,
    backend_sender: Arc<BackendSender>,

    objects: HashMap<u32, Box<dyn WaylandProtocol<Self> + Send>>,

    wl_state: PocoWlState,
    xdg_shell_state: PocoXdgShellState,
}
impl PocoWlClient {
    fn new(client: WaylandClient, backend_sender: Arc<BackendSender>) -> Self {
        let mut objects: HashMap<u32, Box<dyn WaylandProtocol<Self> + Send>> = HashMap::new();
        objects.insert(1, Box::new(DISPLAY_OBJECT));
        Self {
            client,
            backend_sender,
            objects,
            wl_state: PocoWlState::new(),
            xdg_shell_state: PocoXdgShellState::new(),
        }
    }
}

struct PocoWl {
    backend_sender: Arc<BackendSender>,
}
impl PocoWl {
    fn new(backend_sender: BackendSender) -> Self {
        Self {
            backend_sender: Arc::new(backend_sender),
        }
    }
}

impl WaylandState for PocoWl {
    type ClientState = PocoWlClient;

    fn create_client(&self, client: WaylandClient) -> Self::ClientState {
        PocoWlClient::new(client, self.backend_sender.clone())
    }
}

impl WaylandClientState for PocoWlClient {
    fn get_client_mut(&mut self) -> &mut WaylandClient {
        &mut self.client
    }
    fn get_protocol_of_object(&self, id: u32) -> Option<Box<dyn WaylandProtocol<Self> + Send>> {
        self.objects.get(&id).map(|p| p.copy())
    }
    fn on_invalid_object(&mut self, id: u32) {
        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let _ = self
                    .client
                    .stream
                    .write(
                        &DISPLAY_OBJECT
                            .error(
                                id,
                                WlDisplayError::InvalidObject as u32,
                                format!("Invalid object id: {}", id),
                            )
                            .to_raw(),
                    )
                    .await;
            });
        });
    }
}

// fn get_transport() -> Option<PathBuf> {
//     let display = std::env::var_os("WAYLAND_DISPLAY");
//     let display = display.as_deref().unwrap_or("wayland-0".as_ref());
//     let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")?;
//     let path = PathBuf::from(runtime_dir).join(display);
//     Some(path)
// }

#[tokio::main]
async fn main() -> Result<()> {
    let cancel = CancellationToken::new();

    let (backend_tx, backend_rx) = crossbeam::channel::unbounded();
    let mut backend = pocowl_backend_glfw::GlfwBackend {};
    let backend_sender = pocowl_backend::BackendSender::new(backend_tx);
    let state = PocoWl::new(backend_sender);
    let (socket, wenv) = WaylandSocket::create(state)?;
    println!("Listening on {}", wenv);

    let backend_task = tokio::task::spawn_blocking(move || {
        println!("Starting backend");
        backend.run(backend_rx);
        println!("Backend stopped");
    });

    let socket_task = socket.run();

    let cancel_task = tokio::task::spawn({
        let cancel = cancel.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            cancel.cancel();
        }
    });

    cancel
        .run_until_cancelled(async move {
            let _ = tokio::join!(backend_task, socket_task, cancel_task);
        })
        .await;
    Ok(())
}
