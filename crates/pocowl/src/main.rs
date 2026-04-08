mod protocols;

use anyhow::Result;
use pocowl_backend::{Backend, BackendSender};
use pocowl_protocols::WaylandProtocol;
use pocowl_protocols::wayland::{WlDisplay, WlDisplayError};
use pocowl_wlclient::WaylandClient;
use pocowl_wlsocket::{WaylandClientState, WaylandSocket, WaylandState};
use std::collections::HashMap;
use std::rc::Rc;
use tokio::io::AsyncWriteExt as _;
use tokio::runtime::Handle;
use tokio::task::LocalSet;
use tokio_util::sync::CancellationToken;

use crate::protocols::wayland::PocoWlState;
use crate::protocols::xdg_shell::PocoXdgShellState;

pub const DISPLAY_OBJECT: WlDisplay = WlDisplay { object_id: 1 };

pub struct PocoWlClient {
    client: WaylandClient,
    backend_sender: Rc<BackendSender>,

    objects: HashMap<u32, Rc<dyn WaylandProtocol<Self>>>,

    wl_state: PocoWlState,
    xdg_shell_state: PocoXdgShellState,
}
impl PocoWlClient {
    fn new(client: WaylandClient, backend_sender: Rc<BackendSender>) -> Self {
        let mut objects: HashMap<u32, Rc<dyn WaylandProtocol<Self>>> = HashMap::new();
        objects.insert(1, Rc::new(DISPLAY_OBJECT));
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
    clients: HashMap<usize, PocoWlClient>,
    backend_sender: Rc<BackendSender>,
}
impl PocoWl {
    fn new(backend_sender: BackendSender) -> Self {
        Self {
            clients: HashMap::new(),
            backend_sender: Rc::new(backend_sender),
        }
    }
}

impl WaylandState for PocoWl {
    type ClientState = PocoWlClient;

    fn get_client_state_mut(&mut self, id: usize) -> Option<&mut Self::ClientState> {
        self.clients.get_mut(&id)
    }

    fn add_client(&mut self, client: WaylandClient) -> &mut Self::ClientState {
        self.clients
            .entry(client.id)
            .or_insert(PocoWlClient::new(client, self.backend_sender.clone()))
    }
}

impl WaylandClientState for PocoWlClient {
    fn get_client_mut(&mut self) -> &mut WaylandClient {
        &mut self.client
    }
    fn get_protocol_of_object(&self, id: u32) -> Option<Rc<dyn WaylandProtocol<Self>>> {
        self.objects.get(&id).cloned()
    }
    fn on_invalid_object(&mut self, id: u32) {
        tokio::task::block_in_place(move || {
            Handle::current().block_on(async move {
                let _ = self
                    .client
                    .stream
                    .write(
                        &WlDisplay::error(
                            DISPLAY_OBJECT,
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
    let local = LocalSet::new();
    let cancel = CancellationToken::new();

    let (mut backend, backend_sender) = pocowl_backend_glfw::GlfwBackend::new_pair();
    let state = PocoWl::new(backend_sender);
    let mut socket = WaylandSocket::create(state)?;
    println!("Listening on {}", socket.path().display());

    local.spawn_local(async move {
        println!("Starting backend");
        backend.run().await;
        println!("Backend stopped");
    });
    local.spawn_local(async move {
        socket.run().await;
    });
    local.spawn_local({
        let cancel = cancel.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            cancel.cancel();
        }
    });

    cancel.run_until_cancelled(local).await;
    Ok(())
}
