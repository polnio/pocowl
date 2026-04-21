use crossbeam::channel::{Receiver, Sender};
use pocowl_wlbuffer::WaylandBuffer;

pub trait Backend {
    fn run(&mut self, rx: Receiver<Message>);
}

pub struct BackendSender {
    tx: Sender<Message>,
}
impl BackendSender {
    pub fn new(tx: Sender<Message>) -> Self {
        Self { tx }
    }
    pub fn draw(&self, x: u32, y: u32, buffer: WaylandBuffer) {
        let _ = self.tx.send(Message::Draw { x, y, buffer });
    }
    pub fn get_box(&self) -> (u32, u32, u32, u32) {
        let (tx, rx) = crossbeam::channel::bounded(1);
        let _ = self.tx.send(Message::GetBox { resp: tx });
        rx.recv().unwrap()
    }
}

#[derive(Debug)]
pub enum Message {
    Draw {
        x: u32,
        y: u32,
        buffer: WaylandBuffer,
    },
    GetBox {
        resp: Responder<(u32, u32, u32, u32)>,
    },
    Quit,
}

pub type Responder<T> = crossbeam::channel::Sender<T>;
