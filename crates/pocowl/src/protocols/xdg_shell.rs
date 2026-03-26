use crate::PocoWlClient;
use pocowl_protocols::xdg_shell::XdgWmBaseListener;
use pocowl_wlclient::WaylandClient;

impl XdgWmBaseListener for PocoWlClient {
    fn destroy(&mut self, object_id: u32, client: &mut WaylandClient) {
        _ = object_id;
        _ = client;
        todo!();
    }

    fn create_positioner(&mut self, object_id: u32, id: u32, client: &mut WaylandClient) {
        _ = object_id;
        _ = id;
        _ = client;
        todo!();
    }

    fn get_xdg_surface(
        &mut self,
        object_id: u32,
        id: u32,
        surface: u32,
        client: &mut WaylandClient,
    ) {
        _ = object_id;
        _ = id;
        _ = surface;
        _ = client;
        todo!();
    }

    fn pong(&mut self, object_id: u32, serial: u32, client: &mut WaylandClient) {
        _ = object_id;
        _ = serial;
        _ = client;
        todo!();
    }
}
