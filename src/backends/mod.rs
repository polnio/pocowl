#[cfg(feature = "backend-dummy")]
pub mod dummy;
#[cfg(feature = "backend-glfw")]
pub mod glfw;

use crossbeam::channel::{Receiver, Sender};

pub trait Backend {
    fn run(&mut self, rx: Receiver<Message>);
}

#[derive(Clone)]
pub struct BackendSender {
    tx: Sender<Message>,
}
impl BackendSender {
    pub fn new(tx: Sender<Message>) -> Self {
        Self { tx }
    }
    pub fn with_buffer(&self, f: impl FnOnce(&mut [u32], usize, usize) + Send + 'static) {
        let _ = self.tx.send(Message::WithBuffer { f: Box::new(f) });
    }
    pub fn get_box(&self) -> (u32, u32, u32, u32) {
        let (tx, rx) = crossbeam::channel::bounded(1);
        let _ = self.tx.send(Message::GetBox { resp: tx });
        rx.recv().unwrap()
    }
}

// #[derive(Debug)]
pub enum Message {
    WithBuffer {
        f: Box<dyn FnOnce(&mut [u32], usize, usize) + Send>,
    },
    GetBox {
        resp: Responder<(u32, u32, u32, u32)>,
    },
    Quit,
}

pub type Responder<T> = crossbeam::channel::Sender<T>;
