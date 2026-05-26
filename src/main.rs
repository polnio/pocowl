mod backends;
mod protocols;
mod socket;
mod utils;

use crate::socket::Server;
use anyhow::Result;
use backends::Backend as _;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (backend_tx, backend_rx) = crossbeam::channel::unbounded();
    let backend_sender = backends::BackendSender::new(backend_tx);
    let mut backend = backends::glfw::GlfwBackend::new();

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
