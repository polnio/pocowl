use crate::backends::BackendSender;
use crate::utils::{Geometry, WaylandBuffer};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

pub struct SharedState {
    is_recalculate_needed: AtomicBool,
    server_sender: tokio::sync::mpsc::UnboundedSender<Event>,
    backend_sender: BackendSender,
}
impl SharedState {
    pub fn new(backend_sender: BackendSender, server_sender: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            is_recalculate_needed: AtomicBool::new(false),
            server_sender,
            backend_sender,
        }
    }
    pub fn render(&self, buffer: WaylandBuffer) {
        let _ = self.server_sender.send(Event::Render(buffer));
    }
    pub fn recalculate(&self) {
        self.is_recalculate_needed.store(true, Ordering::Relaxed);
        let _ = self.server_sender.send(Event::Recalculate);
    }
    pub fn is_recalculate_needed(&self) -> bool {
        self.is_recalculate_needed.load(Ordering::Relaxed)
    }
    pub fn unset_recalculate_needed(&self) {
        self.is_recalculate_needed.store(false, Ordering::Relaxed);
    }
    pub fn get_box(&self) -> Geometry {
        self.backend_sender.get_box()
    }
}

pub enum Event {
    Render(WaylandBuffer),
    Recalculate,
}
