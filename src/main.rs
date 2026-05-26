mod args;
mod backends;
mod protocols;
mod socket;
mod utils;

use crate::args::Args;
use crate::backends::Backend;
use crate::socket::Server;
use anyhow::Result;
use backends::Backend as _;
use clap::Parser as _;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

fn auto_backend(backend: crate::args::Backend) -> Option<crate::args::Backend> {
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
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    let backend = match args.backend {
        args::Backend::Auto => match auto_backend(args.backend) {
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

    let (server, wenv) = Server::create(backend_sender)?;
    info!("Listening on {}", wenv);

    let backend_task = tokio::task::spawn_blocking(move || {
        info!("Starting backend");
        backend.run(backend_rx);
        info!("Backend stopped");
    });

    let server_task = server.run();

    let cancel_task = tokio::signal::ctrl_c();

    tokio::select! { _ = backend_task => (), _ = server_task => (), _ = cancel_task => () }

    Ok(())
}
