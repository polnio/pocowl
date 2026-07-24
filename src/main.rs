mod app;
mod args;
mod backends;
mod protocols;
mod socket;
mod utils;

pub use app::AppHandle;

slotmap::new_key_type! {
    pub struct ClientId;
}

use crate::app::App;
use crate::args::Args;
use crate::backends::Backend;
use crate::socket::Server;
use anyhow::Result;
use clap::Parser as _;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

#[cfg(not(any(feature = "backend-dummy", feature = "backend-glfw")))]
compile_error!(
    "No backend selected. Select one with the `backend-dummy` or `backend-glfw` feature"
);

fn auto_backend() -> Option<crate::args::Backend> {
    #[cfg(feature = "backend-glfw")]
    if std::env::vars().any(|(k, _)| k == "WAYLAND_DISPLAY" || k == "DISPLAY") {
        return Some(crate::args::Backend::Glfw);
    }

    #[cfg(feature = "backend-dummy")]
    return Some(crate::args::Backend::Dummy);

    #[allow(unreachable_code)]
    None
}

#[tokio::main]
#[cfg_attr(
    not(any(feature = "backend-dummy", feature = "backend-glfw")),
    allow(unused)
)]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    //////////////////////////////////////////////////////////////////////////////

    let args = Args::parse();

    let backend = match args.backend {
        args::Backend::Auto => match auto_backend() {
            Some(b) => b,
            None => {
                tracing::error!("No backend available");
                std::process::exit(1);
            }
        },
        b => b,
    };

    let mut backend: Box<dyn Backend + Send> = match backend {
        #[cfg(feature = "backend-dummy")]
        args::Backend::Dummy => {
            info!("Using dummy backend");
            Box::new(backends::dummy::DummyBackend::new())
        }
        #[cfg(feature = "backend-glfw")]
        args::Backend::Glfw => {
            info!("Using glfw backend");
            Box::new(backends::glfw::GlfwBackend::new())
        }
        args::Backend::Auto => unreachable!(),
    };

    let (backend_tx, backend_rx) = crossbeam::channel::unbounded();
    let backend_sender = backends::BackendSender::new(backend_tx);

    //////////////////////////////////////////////////////////////////////////////

    let mut app = App::new(backend_sender);
    let (io_server, wenv) = Server::create(app.handle())?;
    info!("Listening on {}", wenv);

    //////////////////////////////////////////////////////////////////////////////

    let backend_task = tokio::task::spawn_blocking(move || {
        info!("Starting backend");
        backend.run(backend_rx);
        info!("Backend stopped");
    });
    let io_task = io_server.run();
    let cancel_task = tokio::signal::ctrl_c();
    let app_task = app.run();

    tokio::select! {
        _ = backend_task => (),
        _ = io_task => (),
        _ = app_task => (),
        _ = cancel_task => ()
    }

    Ok(())
}
