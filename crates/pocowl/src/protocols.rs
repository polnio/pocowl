use crate::value::WaylandValue;
pub trait WaylandProtocol<T> {
    fn call(&self, state: &mut T, opcode: u16, buf: &mut &[u8]) -> u32;
}
pocowl_scanner::scan_protocol!("vendor/wayland/protocol/wayland.xml");
pocowl_scanner::scan_protocol!("vendor/wayland-protocols/stable/xdg-shell/xdg-shell.xml");
