use anyhow::Result;
use pocowl_protocols::WaylandProtocol;
use pocowl_protocols::wayland::{WlDisplay, WlDisplayListener, WlRegistry, WlRegistryListener};
use pocowl_wlsocket::{WaylandSocket, WaylandState};
use std::collections::HashMap;
use std::rc::Rc;
use tokio::net::UnixStream;
use tokio::task::LocalSet;
use tokio_util::sync::CancellationToken;

struct PocoWl {
    objects: HashMap<u32, Rc<dyn WaylandProtocol<Self>>>,
}
impl PocoWl {
    fn new() -> Self {
        let mut objects: HashMap<u32, Rc<dyn WaylandProtocol<Self>>> = HashMap::new();
        objects.insert(1, Rc::new(WlDisplay));
        Self { objects }
    }
}

impl WaylandState for PocoWl {
    fn get_protocol_of_object(&self, id: u32) -> Option<Rc<dyn WaylandProtocol<Self>>> {
        self.objects.get(&id).cloned()
    }
}

impl WlRegistryListener for PocoWl {
    fn bind(&mut self, name: u32, id: u32, stream: &mut UnixStream) {
        _ = name;
        _ = id;
        _ = stream;
        todo!()
    }
}

impl WlDisplayListener for PocoWl {
    fn sync(&mut self, callback: u32, stream: &mut UnixStream) {
        _ = callback;
        _ = stream;
        // todo!()
        println!("Syncing");
    }

    fn get_registry(&mut self, registry: u32, stream: &mut UnixStream) {
        self.objects.insert(registry, Rc::new(WlRegistry));
        println!("Added registry {}", registry);
        println!("TEST: {}", WlRegistry::NAME);
        let mut data = Vec::new();
        data.extend(
            WlRegistry::global(registry, 1, WlDisplay::NAME.to_owned(), WlDisplay::VERSION)
                .to_raw(),
        );
        stream.try_write(&data).unwrap();
        // TODO: Pass the UnixSocket to the methods so the compositor can respond
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
