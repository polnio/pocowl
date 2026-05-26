use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

use crate::utils::WaylandBuffer;

pub struct SharedState {
    is_recalculate_needed: AtomicBool,
    server_sender: tokio::sync::mpsc::UnboundedSender<Event>,
}
impl SharedState {
    pub fn new(server_sender: mpsc::UnboundedSender<Event>) -> Self {
        Self {
            is_recalculate_needed: AtomicBool::new(false),
            server_sender,
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
}

pub enum Event {
    Render(WaylandBuffer),
    Recalculate,
}
