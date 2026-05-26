use tokio::io::AsyncWriteExt as _;

use super::WaylandStream;
use crate::protocols::{DISPLAY_OBJECT, WaylandProtocol, imp::ImpProtoStates};
use crate::socket::shared::SharedState;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Client {
    pub id: usize,
    pub stream: WaylandStream,
    objects: HashMap<u32, Box<dyn WaylandProtocol<Self> + Send>>,
    pub imp_proto_states: ImpProtoStates,

    pub shared_state: Arc<SharedState>,
}
impl Client {
    pub fn new(id: usize, stream: WaylandStream, shared_state: Arc<SharedState>) -> Self {
        let objects: HashMap<u32, Box<dyn WaylandProtocol<Self> + Send>> = HashMap::new();
        let imp_proto_states = ImpProtoStates::default();
        let mut this = Self {
            id,
            stream,
            objects,
            imp_proto_states,
            shared_state,
        };
        this.add_object(DISPLAY_OBJECT);
        this
    }

    pub async fn error(&mut self, id: u32, code: u32, message: String) {
        let _ = self
            .stream
            .write(&DISPLAY_OBJECT.error(id, code, message).to_raw())
            .await;
    }

    pub fn add_object(&mut self, object: impl WaylandProtocol<Self> + Send + 'static) {
        self.add_boxed_object(Box::new(object));
    }
    pub fn add_boxed_object(&mut self, object: Box<dyn WaylandProtocol<Self> + Send>) {
        self.objects.insert(object.object_id(), object);
    }
    pub fn get_object(&self, id: u32) -> Option<Box<dyn WaylandProtocol<Self> + Send>> {
        self.objects.get(&id).map(|p| p.copy())
    }
    pub fn remove_object(&mut self, id: u32) {
        self.objects.remove(&id);
    }
}
