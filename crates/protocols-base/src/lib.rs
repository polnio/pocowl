use pocowl_wlclient::WaylandClient;
use pocowl_wlmessage::WaylandMessage;

pub trait WaylandProtocol<T> {
    fn call(&self, state: &mut T, message: WaylandMessage, client: &mut WaylandClient);
    fn name(&self) -> &'static str;
    fn version(&self) -> u32;
}
