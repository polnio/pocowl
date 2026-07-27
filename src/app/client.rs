use crate::protocols::imp::ImpProtoStates;
use crate::protocols::{DISPLAY_OBJECT, WaylandProtocol};
use crate::socket::WaylandMessage;
use crate::{AppHandle, ClientId};
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct Client {
    pub id: ClientId,
    pub app_handle: AppHandle,
    pub imp_proto_states: ImpProtoStates,
    sender: mpsc::UnboundedSender<WaylandMessage>,
    objects: HashMap<u32, Box<dyn WaylandProtocol<Self> + Send>>,
}
impl Client {
    pub fn new(
        id: ClientId,
        app_handle: AppHandle,
        sender: mpsc::UnboundedSender<WaylandMessage>,
    ) -> Self {
        let objects: HashMap<u32, Box<dyn WaylandProtocol<Self> + Send>> = HashMap::new();
        let imp_proto_states = ImpProtoStates::default();
        let mut this = Self {
            id,
            app_handle,
            imp_proto_states,
            sender,
            objects,
        };
        this.add_object(DISPLAY_OBJECT);
        this
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

    pub fn send(&self, resp: WaylandMessage) {
        self.sender.send(resp).unwrap();
    }

    pub fn error(&self, id: u32, code: u32, message: String) {
        self.send(DISPLAY_OBJECT.error(id, code, message));
    }
}
