use super::Backend;
use crate::utils::Geometry;
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
                super::Message::WithBuffer { .. } => {}
                super::Message::GetBox { resp } => {
                    let _ = resp.send(Geometry {
                        x: 0,
                        y: 0,
                        w: 100,
                        h: 100,
                    });
                }
                super::Message::Quit => break,
            }
        }
    }
}
