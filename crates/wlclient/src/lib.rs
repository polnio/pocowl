use pocowl_wlstream::WaylandStream;

pub struct WaylandClient {
    pub id: usize,
    pub stream: WaylandStream,
}

impl WaylandClient {
    pub fn new(id: usize, stream: WaylandStream) -> Self {
        Self { id, stream }
    }
}
