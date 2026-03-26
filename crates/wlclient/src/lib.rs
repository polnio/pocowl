use tokio::net::UnixStream;

pub struct WaylandClient {
    pub id: usize,
    pub stream: UnixStream,
}
