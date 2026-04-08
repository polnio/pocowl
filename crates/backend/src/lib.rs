use pocowl_wlbuffer::WaylandBuffer;
use tokio::sync::mpsc::Sender;

pub trait Backend {
    fn new_pair() -> (Self, BackendSender)
    where
        Self: Sized;
    fn run(&mut self) -> impl std::future::Future<Output = ()> + Send;
}

pub struct BackendSender {
    tx: Sender<Message>,
}
impl BackendSender {
    pub fn new(tx: Sender<Message>) -> Self {
        Self { tx }
    }
    pub async fn draw(&self, x: u32, y: u32, buffer: WaylandBuffer) {
        let _ = self.tx.send(Message::Draw { x, y, buffer }).await;
    }
    pub async fn get_box(&self) -> (u32, u32, u32, u32) {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(Message::GetBox { resp: tx }).await;
        rx.await.unwrap()
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
}

pub type Responder<T> = tokio::sync::oneshot::Sender<T>;
