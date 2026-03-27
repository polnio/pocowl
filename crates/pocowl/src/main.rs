mod protocols;

use anyhow::Result;
use pocowl_protocols::WaylandProtocol;
use pocowl_protocols::wayland::WlDisplay;
use pocowl_wlsocket::{WaylandClientState, WaylandSocket, WaylandState};
use std::collections::HashMap;
use std::rc::Rc;
use tokio::task::LocalSet;
use tokio_util::sync::CancellationToken;

pub const DISPLAY_OBJECT: WlDisplay = WlDisplay { object_id: 1 };

pub struct PocoWlClient {
    objects: HashMap<u32, Rc<dyn WaylandProtocol<Self>>>,
}
impl PocoWlClient {
    fn new() -> Self {
        let mut objects: HashMap<u32, Rc<dyn WaylandProtocol<Self>>> = HashMap::new();
        objects.insert(1, Rc::new(DISPLAY_OBJECT));
        Self { objects }
    }
}

struct PocoWl {
    clients: HashMap<usize, PocoWlClient>,
}
impl PocoWl {
    fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
}

impl WaylandState for PocoWl {
    type ClientState = PocoWlClient;

    fn get_client_state_mut(&mut self, id: usize) -> Option<&mut Self::ClientState> {
        self.clients.get_mut(&id)
    }

    fn add_client(&mut self, id: usize) {
        self.clients.insert(id, PocoWlClient::new());
    }
}

impl WaylandClientState for PocoWlClient {
    fn get_protocol_of_object(&self, id: u32) -> Option<Rc<dyn WaylandProtocol<Self>>> {
        self.objects.get(&id).cloned()
    }
}

// fn get_transport() -> Option<PathBuf> {
//     let display = std::env::var_os("WAYLAND_DISPLAY");
//     let display = display.as_deref().unwrap_or("wayland-0".as_ref());
//     let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")?;
//     let path = PathBuf::from(runtime_dir).join(display);
//     Some(path)
// }

async fn run_with_cancellation<F>(f: F)
where
    F: Future<Output = ()> + 'static,
{
    let local = LocalSet::new();

    let cancel = CancellationToken::new();
    local.spawn_local({
        let cancel = cancel.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            cancel.cancel();
        }
    });

    local.spawn_local(async move { cancel.run_until_cancelled(f).await });
    local.await;
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = PocoWl::new();
    let mut socket = WaylandSocket::create(state)?;
    println!("Listening on {}", socket.path().display());
    run_with_cancellation(async move { socket.run().await }).await;
    Ok(())
}
