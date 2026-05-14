use async_trait::async_trait;
use pocowl_wlmessage::WaylandMessage;
use std::collections::VecDeque;
use std::os::fd::OwnedFd;

pub trait CanFetchFd {
    fn fetch_fd(&mut self) -> impl Future<Output = Option<OwnedFd>>;
}

#[async_trait]
pub trait WaylandProtocol<T> {
    async fn call(&self, state: &mut T, message: WaylandMessage, fds: &mut VecDeque<OwnedFd>);
    fn name(&self) -> &'static str;
    fn version(&self) -> u32;
    fn object_id(&self) -> u32;
    fn copy(&self) -> Box<dyn WaylandProtocol<T> + Send>;
}
