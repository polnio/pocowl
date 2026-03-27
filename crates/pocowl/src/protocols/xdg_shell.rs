use crate::PocoWlClient;
use pocowl_protocols::wayland::*;
use pocowl_protocols::xdg_shell::*;
use pocowl_wlclient::WaylandClient;

#[allow(unused_variables)]
impl XdgWmBaseListener for PocoWlClient {
    fn destroy(&mut self, object: XdgWmBase, client: &mut WaylandClient) {
        todo!();
    }

    fn create_positioner(
        &mut self,
        object: XdgWmBase,
        id: XdgPositioner,
        client: &mut WaylandClient,
    ) {
        todo!();
    }

    fn get_xdg_surface(
        &mut self,
        object: XdgWmBase,
        id: XdgSurface,
        surface: WlSurface,
        client: &mut WaylandClient,
    ) {
        todo!();
    }

    fn pong(&mut self, object: XdgWmBase, serial: u32, client: &mut WaylandClient) {
        todo!();
    }
}

#[allow(unused_variables)]
impl XdgPositionerListener for PocoWlClient {
    fn destroy(&mut self, object: XdgPositioner, client: &mut WaylandClient) {
        todo!()
    }

    fn set_size(
        &mut self,
        object: XdgPositioner,
        width: i32,
        height: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_anchor_rect(
        &mut self,
        object: XdgPositioner,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_anchor(&mut self, object: XdgPositioner, anchor: u32, client: &mut WaylandClient) {
        todo!()
    }

    fn set_gravity(&mut self, object: XdgPositioner, gravity: u32, client: &mut WaylandClient) {
        todo!()
    }

    fn set_constraint_adjustment(
        &mut self,
        object: XdgPositioner,
        constraint_adjustment: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_offset(&mut self, object: XdgPositioner, x: i32, y: i32, client: &mut WaylandClient) {
        todo!()
    }

    fn set_reactive(&mut self, object: XdgPositioner, client: &mut WaylandClient) {
        todo!()
    }

    fn set_parent_size(
        &mut self,
        object: XdgPositioner,
        parent_width: i32,
        parent_height: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_parent_configure(
        &mut self,
        object: XdgPositioner,
        serial: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }
}

#[allow(unused_variables)]
impl XdgSurfaceListener for PocoWlClient {
    fn destroy(&mut self, object: XdgSurface, client: &mut WaylandClient) {
        todo!()
    }

    fn get_toplevel(&mut self, object: XdgSurface, id: XdgToplevel, client: &mut WaylandClient) {
        todo!()
    }

    fn get_popup(
        &mut self,
        object: XdgSurface,
        id: XdgPopup,
        parent: Option<XdgSurface>,
        positioner: XdgPositioner,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_window_geometry(
        &mut self,
        object: XdgSurface,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn ack_configure(&mut self, object: XdgSurface, serial: u32, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl XdgToplevelListener for PocoWlClient {
    fn destroy(&mut self, object: XdgToplevel, client: &mut WaylandClient) {
        todo!()
    }

    fn set_parent(
        &mut self,
        object: XdgToplevel,
        parent: Option<XdgToplevel>,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_title(&mut self, object: XdgToplevel, title: String, client: &mut WaylandClient) {
        todo!()
    }

    fn set_app_id(&mut self, object: XdgToplevel, app_id: String, client: &mut WaylandClient) {
        todo!()
    }

    fn show_window_menu(
        &mut self,
        object: XdgToplevel,
        seat: WlSeat,
        serial: u32,
        x: i32,
        y: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn r#move(
        &mut self,
        object: XdgToplevel,
        seat: WlSeat,
        serial: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn resize(
        &mut self,
        object: XdgToplevel,
        seat: WlSeat,
        serial: u32,
        edges: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_max_size(
        &mut self,
        object: XdgToplevel,
        width: i32,
        height: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_min_size(
        &mut self,
        object: XdgToplevel,
        width: i32,
        height: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_maximized(&mut self, object: XdgToplevel, client: &mut WaylandClient) {
        todo!()
    }

    fn unset_maximized(&mut self, object: XdgToplevel, client: &mut WaylandClient) {
        todo!()
    }

    fn set_fullscreen(
        &mut self,
        object: XdgToplevel,
        output: Option<WlOutput>,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn unset_fullscreen(&mut self, object: XdgToplevel, client: &mut WaylandClient) {
        todo!()
    }

    fn set_minimized(&mut self, object: XdgToplevel, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl XdgPopupListener for PocoWlClient {
    fn destroy(&mut self, object: XdgPopup, client: &mut WaylandClient) {
        todo!()
    }

    fn grab(&mut self, object: XdgPopup, seat: WlSeat, serial: u32, client: &mut WaylandClient) {
        todo!()
    }

    fn reposition(
        &mut self,
        object: XdgPopup,
        positioner: XdgPositioner,
        token: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }
}
