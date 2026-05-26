use super::Backend;
use crossbeam::channel::Receiver;

pub struct DummyBackend;
impl DummyBackend {
    pub fn new() -> Self {
        Self
    }
}
impl Backend for DummyBackend {
    fn run(&mut self, rx: Receiver<super::Message>) {
        while let Ok(message) = rx.recv() {
            match message {
                super::Message::Draw { .. } => {}
                super::Message::GetBox { resp } => {
                    let _ = resp.send((0, 0, 100, 100));
                }
                super::Message::Quit => break,
            }
        }
    }
}
