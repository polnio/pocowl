mod client;
mod handle;
mod layout;

pub use client::Client;
pub use handle::AppHandle;
use tracing::error;

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
    layout: layout::Layout,
}

impl App {
    pub fn new(backend_sender: BackendSender) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let handle = AppHandle::new(sender, backend_sender);
        let clients = SlotMap::with_key();
        let fds = SecondaryMap::new();
        let last_recalculate = std::time::Instant::now();
        let layout = layout::Layout::new();
        Self {
            handle,
            receiver,
            clients,
            fds,
            last_recalculate,
            layout,
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
                    let handle = self.handle();
                    let id = self
                        .clients
                        .insert_with_key(|id| Client::new(id, handle, sender));
                    resp.send(id).unwrap();
                }
                Event::Disconnection { id } => {
                    // FIXME: clear the screen region
                    // self.clients[id].handle_disconnection();
                    for xdg_toplevel in self.clients[id].xdg_toplevels() {
                        let window = self.layout.find_window(id, xdg_toplevel).unwrap();
                        self.layout.remove_window(window);
                    }
                    self.clients.remove(id);
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
                Event::Render { id, surface, buf } => {
                    println!("Render {id:?}");
                    let Some(geometry) = self.surface_geometry(id, surface) else {
                        error!("wl_surface#{}::render: No geometry", surface.object_id);
                        continue;
                    };
                    self.handle.backend().with_buffer(move |wbuffer, w, h| {
                        for (i, c) in buf.data.iter().enumerate() {
                            let w = w as usize;
                            let h = h as usize;
                            let x = geometry.x as usize + i % buf.stride;
                            let y = geometry.y as usize + i / buf.stride;
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
                    println!("--------------- Recalculate ---------------");
                    let geometry = self.handle.backend().get_box();
                    let configured = self.layout.recalculate(geometry);
                    for (id, xdg_toplevel, geometry) in configured {
                        println!("{geometry:?}");
                        let client = &mut self.clients[id];
                        client.configure(xdg_toplevel, geometry);
                    }
                    println!("--------------- End Recalculate ---------------");
                }
                Event::AddWindowAtFocused { id, xdg_toplevel } => {
                    self.layout.add_window_at_focused(id, xdg_toplevel);
                }
            }
        }
    }
}
