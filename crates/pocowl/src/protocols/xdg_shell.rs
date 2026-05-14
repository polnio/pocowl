use crate::PocoWlClient;
use async_trait::async_trait;
use pocowl_protocols::wayland::*;
use pocowl_protocols::xdg_shell::*;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt as _;

pub struct PocoXdgShellState {
    surface_map: HashMap<XdgSurface, WlSurface>,
    toplevel_map: HashMap<XdgToplevel, XdgSurface>,
    next_serial: u32,
}
impl PocoXdgShellState {
    pub fn new() -> Self {
        Self {
            surface_map: HashMap::new(),
            toplevel_map: HashMap::new(),
            next_serial: 1,
        }
    }
}

#[allow(unused_variables)]
#[async_trait]
impl XdgWmBaseListener for PocoWlClient {
    async fn destroy(&mut self, object: XdgWmBase) {
        todo!();
    }

    async fn create_positioner(&mut self, object: XdgWmBase, id: XdgPositioner) {
        todo!();
    }

    async fn get_xdg_surface(
        &mut self,
        object: XdgWmBase,
        xdg_surface: XdgSurface,
        surface: WlSurface,
    ) {
        self.xdg_shell_state
            .surface_map
            .insert(xdg_surface, surface);
        self.objects
            .insert(xdg_surface.object_id, Box::new(xdg_surface));
        let _ = self
            .client
            .stream
            .write(
                &xdg_surface
                    .configure(self.xdg_shell_state.next_serial)
                    .to_raw(),
            )
            .await;
        self.xdg_shell_state.next_serial += 1;
    }

    async fn pong(&mut self, object: XdgWmBase, serial: u32) {
        todo!();
    }
}

#[allow(unused_variables)]
#[async_trait]
impl XdgPositionerListener for PocoWlClient {
    async fn destroy(&mut self, object: XdgPositioner) {
        todo!()
    }

    async fn set_size(&mut self, object: XdgPositioner, width: i32, height: i32) {
        todo!()
    }

    async fn set_anchor_rect(
        &mut self,
        object: XdgPositioner,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        todo!()
    }

    async fn set_anchor(&mut self, object: XdgPositioner, anchor: XdgPositionerAnchor) {
        todo!()
    }

    async fn set_gravity(&mut self, object: XdgPositioner, gravity: XdgPositionerGravity) {
        todo!()
    }

    async fn set_constraint_adjustment(
        &mut self,
        object: XdgPositioner,
        constraint_adjustment: XdgPositionerConstraintAdjustment,
    ) {
        todo!()
    }

    async fn set_offset(&mut self, object: XdgPositioner, x: i32, y: i32) {
        todo!()
    }

    async fn set_reactive(&mut self, object: XdgPositioner) {
        todo!()
    }

    async fn set_parent_size(
        &mut self,
        object: XdgPositioner,
        parent_width: i32,
        parent_height: i32,
    ) {
        todo!()
    }

    async fn set_parent_configure(&mut self, object: XdgPositioner, serial: u32) {
        todo!()
    }
}

#[allow(unused_variables)]
#[async_trait]
impl XdgSurfaceListener for PocoWlClient {
    async fn destroy(&mut self, object: XdgSurface) {
        todo!()
    }

    async fn get_toplevel(&mut self, object: XdgSurface, id: XdgToplevel) {
        self.xdg_shell_state.toplevel_map.insert(id, object);
        self.objects.insert(id.object_id, Box::new(id));
    }

    async fn get_popup(
        &mut self,
        object: XdgSurface,
        id: XdgPopup,
        parent: Option<XdgSurface>,
        positioner: XdgPositioner,
    ) {
        todo!()
    }

    async fn set_window_geometry(
        &mut self,
        object: XdgSurface,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        todo!()
    }

    async fn ack_configure(&mut self, object: XdgSurface, serial: u32) {
        // TODO: check if serial < self.xdg_shell_state.next_serial
    }
}

#[allow(unused_variables)]
#[async_trait]
impl XdgToplevelListener for PocoWlClient {
    async fn destroy(&mut self, object: XdgToplevel) {
        todo!()
    }

    async fn set_parent(&mut self, object: XdgToplevel, parent: Option<XdgToplevel>) {
        todo!()
    }

    async fn set_title(&mut self, object: XdgToplevel, title: String) {
        todo!()
    }

    async fn set_app_id(&mut self, object: XdgToplevel, app_id: String) {
        todo!()
    }

    async fn show_window_menu(
        &mut self,
        object: XdgToplevel,
        seat: WlSeat,
        serial: u32,
        x: i32,
        y: i32,
    ) {
        todo!()
    }

    async fn r#move(&mut self, object: XdgToplevel, seat: WlSeat, serial: u32) {
        todo!()
    }

    async fn resize(
        &mut self,
        object: XdgToplevel,
        seat: WlSeat,
        serial: u32,
        edges: XdgToplevelResizeEdge,
    ) {
        todo!()
    }

    async fn set_max_size(&mut self, object: XdgToplevel, width: i32, height: i32) {
        todo!()
    }

    async fn set_min_size(&mut self, object: XdgToplevel, width: i32, height: i32) {
        todo!()
    }

    async fn set_maximized(&mut self, object: XdgToplevel) {
        todo!()
    }

    async fn unset_maximized(&mut self, object: XdgToplevel) {
        todo!()
    }

    async fn set_fullscreen(&mut self, object: XdgToplevel, output: Option<WlOutput>) {
        todo!()
    }

    async fn unset_fullscreen(&mut self, object: XdgToplevel) {
        todo!()
    }

    async fn set_minimized(&mut self, object: XdgToplevel) {
        todo!()
    }
}

#[allow(unused_variables)]
#[async_trait]
impl XdgPopupListener for PocoWlClient {
    async fn destroy(&mut self, object: XdgPopup) {
        todo!()
    }

    async fn grab(&mut self, object: XdgPopup, seat: WlSeat, serial: u32) {
        todo!()
    }

    async fn reposition(&mut self, object: XdgPopup, positioner: XdgPositioner, token: u32) {
        todo!()
    }
}
