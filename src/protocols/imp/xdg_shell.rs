use crate::protocols::wayland::{WlOutput, WlSeat, WlSurface};
use crate::protocols::xdg_shell::*;
use crate::socket::Client;
use async_trait::async_trait;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt as _;

struct ImpXdgSurface {
    inner: XdgSurface,
    wl_surface: WlSurface,
}
struct ImpXdgToplevel {
    inner: XdgToplevel,
    xdg_surface: XdgSurface,
}
#[derive(Default)]
pub struct ImpXdgShellState {
    surfaces: HashMap<XdgSurface, ImpXdgSurface>,
    toplevels: HashMap<XdgToplevel, ImpXdgToplevel>,
    next_serial: u32,
}
impl Client {
    fn xdg_shell_state(&self) -> &ImpXdgShellState {
        &self.imp_proto_states.xdg_shell
    }
    fn xdg_shell_state_mut(&mut self) -> &mut ImpXdgShellState {
        &mut self.imp_proto_states.xdg_shell
    }
}

#[allow(unused_variables)]
#[async_trait]
impl XdgWmBaseListener for Client {
    async fn destroy(&mut self, xdg_wm_base: XdgWmBase) {
        todo!();
    }

    async fn create_positioner(&mut self, xdg_wm_base: XdgWmBase, xdg_positioner: XdgPositioner) {
        todo!();
    }

    async fn get_xdg_surface(
        &mut self,
        xdg_wm_base: XdgWmBase,
        xdg_surface: XdgSurface,
        wl_surface: WlSurface,
    ) {
        let xdg_surface = ImpXdgSurface {
            inner: xdg_surface,
            wl_surface,
        };
        self.add_object(xdg_surface.inner);
        let _ = self
            .stream
            .write(
                &xdg_surface
                    .inner
                    .configure(self.xdg_shell_state().next_serial)
                    .to_raw(),
            )
            .await;
        self.xdg_shell_state_mut()
            .surfaces
            .insert(xdg_surface.inner, xdg_surface);
        self.xdg_shell_state_mut().next_serial += 1;
    }

    async fn pong(&mut self, xdg_wm_base: XdgWmBase, serial: u32) {
        todo!();
    }
}

#[allow(unused_variables)]
#[async_trait]
impl XdgSurfaceListener for Client {
    async fn destroy(&mut self, xd_surface: XdgSurface) {
        todo!()
    }

    async fn get_toplevel(&mut self, xdg_surface: XdgSurface, xdg_toplevel: XdgToplevel) {
        let xdg_toplevel = ImpXdgToplevel {
            inner: xdg_toplevel,
            xdg_surface,
        };
        self.add_object(xdg_toplevel.inner);
        self.xdg_shell_state_mut()
            .toplevels
            .insert(xdg_toplevel.inner, xdg_toplevel);
    }

    async fn get_popup(
        &mut self,
        xd_surface: XdgSurface,
        id: XdgPopup,
        parent: Option<XdgSurface>,
        positioner: XdgPositioner,
    ) {
        todo!()
    }

    async fn set_window_geometry(
        &mut self,
        xd_surface: XdgSurface,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        todo!()
    }

    async fn ack_configure(&mut self, xd_surface: XdgSurface, serial: u32) {
        // TODO: check if serial < self.xdg_shell_state.next_serial
    }
}

#[allow(unused_variables)]
#[async_trait]
impl XdgToplevelListener for Client {
    async fn destroy(&mut self, xdg_toplevel: XdgToplevel) {
        todo!()
    }

    async fn set_parent(&mut self, xdg_toplevel: XdgToplevel, parent: Option<XdgToplevel>) {
        todo!()
    }

    async fn set_title(&mut self, xdg_toplevel: XdgToplevel, title: String) {
        todo!()
    }

    async fn set_app_id(&mut self, xdg_toplevel: XdgToplevel, app_id: String) {
        todo!()
    }

    async fn show_window_menu(
        &mut self,
        xdg_toplevel: XdgToplevel,
        seat: WlSeat,
        serial: u32,
        x: i32,
        y: i32,
    ) {
        todo!()
    }

    async fn r#move(&mut self, xdg_toplevel: XdgToplevel, seat: WlSeat, serial: u32) {
        todo!()
    }

    async fn resize(
        &mut self,
        xdg_toplevel: XdgToplevel,
        seat: WlSeat,
        serial: u32,
        edges: XdgToplevelResizeEdge,
    ) {
        todo!()
    }

    async fn set_max_size(&mut self, xdg_toplevel: XdgToplevel, width: i32, height: i32) {
        todo!()
    }

    async fn set_min_size(&mut self, xdg_toplevel: XdgToplevel, width: i32, height: i32) {
        todo!()
    }

    async fn set_maximized(&mut self, xdg_toplevel: XdgToplevel) {
        todo!()
    }

    async fn unset_maximized(&mut self, xdg_toplevel: XdgToplevel) {
        todo!()
    }

    async fn set_fullscreen(&mut self, xdg_toplevel: XdgToplevel, output: Option<WlOutput>) {
        todo!()
    }

    async fn unset_fullscreen(&mut self, xdg_toplevel: XdgToplevel) {
        todo!()
    }

    async fn set_minimized(&mut self, xdg_toplevel: XdgToplevel) {
        todo!()
    }
}
