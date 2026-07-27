use crate::ClientId;
use crate::backends::BackendSender;
use crate::protocols::wayland::WlSurface;
use crate::protocols::xdg_shell::XdgToplevel;
use crate::socket::WaylandMessage;
use crate::utils::WaylandBuffer;
use std::os::fd::OwnedFd;
use tokio::sync::{mpsc, oneshot};

pub enum Event {
    Connection {
        resp: oneshot::Sender<ClientId>,
        sender: mpsc::UnboundedSender<WaylandMessage>,
    },
    Disconnection {
        id: ClientId,
    },
    FdAttached {
        id: ClientId,
        fd: OwnedFd,
    },
    Message {
        id: ClientId,
        msg: WaylandMessage,
    },
    Render {
        id: ClientId,
        surface: WlSurface,
        buf: WaylandBuffer,
    },
    Recalculate {
        time: std::time::Instant,
    },
    AddWindowAtFocused {
        id: ClientId,
        xdg_toplevel: XdgToplevel,
    },
}

#[derive(Clone)]
pub struct AppHandle {
    sender: mpsc::UnboundedSender<Event>,
    backend_sender: BackendSender,
}

impl AppHandle {
    pub(super) fn new(sender: mpsc::UnboundedSender<Event>, backend_sender: BackendSender) -> Self {
        Self {
            sender,
            backend_sender,
        }
    }

    pub fn backend(&self) -> &BackendSender {
        &self.backend_sender
    }

    pub async fn connection(&self) -> (ClientId, mpsc::UnboundedReceiver<WaylandMessage>) {
        let (tx, rx) = oneshot::channel();
        let (sender, receiver) = mpsc::unbounded_channel();
        self.sender
            .send(Event::Connection { resp: tx, sender })
            .unwrap();
        let id = rx.await.unwrap();
        (id, receiver)
    }

    pub fn disconnection(&self, id: ClientId) {
        self.sender.send(Event::Disconnection { id }).unwrap();
    }

    pub fn fd_attached(&self, id: ClientId, fd: OwnedFd) {
        self.sender.send(Event::FdAttached { id, fd }).unwrap();
    }

    pub fn message(&self, id: ClientId, msg: WaylandMessage) {
        self.sender.send(Event::Message { id, msg }).unwrap();
    }

    pub fn render(&self, id: ClientId, surface: WlSurface, buf: WaylandBuffer) {
        self.sender
            .send(Event::Render { id, surface, buf })
            .unwrap();
    }

    pub fn recalculate(&self) {
        let time = std::time::Instant::now();
        self.sender.send(Event::Recalculate { time }).unwrap();
    }

    pub fn add_window_at_focused(&self, id: ClientId, xdg_toplevel: XdgToplevel) {
        self.sender
            .send(Event::AddWindowAtFocused { id, xdg_toplevel })
            .unwrap();
    }
}
