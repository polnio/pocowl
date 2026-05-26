pub mod imp;

use crate::socket::WaylandMessage;
use async_trait::async_trait;
use std::collections::VecDeque;
use std::os::fd::OwnedFd;

pub const DISPLAY_OBJECT: WlDisplay = WlDisplay { object_id: 1 };

#[async_trait]
pub trait WaylandProtocol<T> {
    async fn call(&self, state: &mut T, message: WaylandMessage, fds: &mut VecDeque<OwnedFd>);
    fn name(&self) -> &'static str;
    fn version(&self) -> u32;
    fn object_id(&self) -> u32;
    fn copy(&self) -> Box<dyn WaylandProtocol<T> + Send>;
}

use wayland::*;
pocowl::scan_protocol!("vendor/wayland/protocol/wayland.xml");
pocowl::scan_protocol!("vendor/wayland-protocols/stable/xdg-shell/xdg-shell.xml");
