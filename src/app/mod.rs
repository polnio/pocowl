mod client;
mod handle;

pub use client::Client;
pub use handle::AppHandle;

use crate::ClientId;
use crate::backends::BackendSender;
use crate::protocols::wayland::WlDisplayError;
use handle::Event;
use slotmap::{SecondaryMap, SlotMap};
use std::collections::VecDeque;
use std::os::fd::OwnedFd;
use tokio::sync::mpsc;

pub struct App {
    handle: AppHandle,
    receiver: mpsc::UnboundedReceiver<Event>,
    clients: SlotMap<ClientId, Client>,
    fds: SecondaryMap<ClientId, VecDeque<OwnedFd>>,
    last_recalculate: std::time::Instant,
}

impl App {
    pub fn new(backend_sender: BackendSender) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let handle = AppHandle::new(sender, backend_sender);
        let clients = SlotMap::with_key();
        let fds = SecondaryMap::new();
        let last_recalculate = std::time::Instant::now();
        Self {
            handle,
            receiver,
            clients,
            fds,
            last_recalculate,
        }
    }

    pub fn handle(&self) -> AppHandle {
        // Cloning is fine, since this is a ref counter
        self.handle.clone()
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.receiver.recv().await {
            match event {
                Event::Connection { resp, sender } => {
                    let id = self.clients.insert(Client::new(self.handle(), sender));
                    resp.send(id).unwrap();
                }
                Event::FdAttached { id, fd } => {
                    self.fds[id].push_back(fd);
                }
                Event::Message { id, msg } => {
                    let client = &mut self.clients[id];
                    let fds = &mut self.fds.entry(id).unwrap().or_default();
                    let Some(p) = client.get_object(msg.object_id) else {
                        client.error(
                            msg.object_id,
                            WlDisplayError::InvalidObject as u32,
                            format!("Invalid object id: {}", msg.object_id),
                        );
                        continue;
                    };
                    p.call(client, msg, fds).await;
                }
                Event::Render { buf } => {
                    self.handle.backend().with_buffer(move |wbuffer, w, h| {
                        for (i, c) in buf.data.iter().enumerate() {
                            let w = w as usize;
                            let h = h as usize;
                            let x = i % buf.stride;
                            let y = i / buf.stride;
                            if x > w || y > h {
                                continue;
                            }
                            let j = y * w + x;
                            wbuffer[j] = *c;
                        }
                    });
                }
                Event::Recalculate { time } => {
                    if time < self.last_recalculate {
                        continue;
                    }
                    // FIXME: Make proper recalculation (eg: layout), and send configure
                    // requests to clients
                }
            }
        }
    }
}
