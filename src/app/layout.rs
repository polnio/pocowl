use crate::ClientId;
use crate::app::App;
use crate::protocols::xdg_shell::XdgToplevel;
use crate::{protocols::wayland::WlSurface, utils::Geometry};
use slotmap::{SlotMap, new_key_type};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Window {
    parent: SublayoutId,
    client: ClientId,
    xdg_toplevel: XdgToplevel,
    cached_geometry: Option<Geometry>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Sublayout {
    parent: Option<SublayoutId>,
    children: Vec<NodeId>,
    last_focused: Option<usize>,
}

new_key_type! {
    pub struct WindowId;
    pub struct SublayoutId;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeId {
    Window(WindowId),
    Sublayout(SublayoutId),
}

#[derive(Debug, Clone)]
pub struct Layout {
    windows: SlotMap<WindowId, Window>,
    sublayouts: SlotMap<SublayoutId, Sublayout>,
    root: SublayoutId,
}

impl Layout {
    pub fn new() -> Self {
        let windows = SlotMap::with_key();
        let mut sublayouts = SlotMap::with_key();
        let root = sublayouts.insert(Sublayout::default());
        Self {
            windows,
            sublayouts,
            root,
        }
    }

    pub fn find_window(&self, client: ClientId, xdg_toplevel: XdgToplevel) -> Option<WindowId> {
        self.windows.iter().find_map(|(id, window)| {
            (window.client == client && window.xdg_toplevel == xdg_toplevel).then_some(id)
        })
    }

    pub fn find_focused(&self) -> NodeId {
        let mut current_id = self.root;
        let mut current = &self.sublayouts[current_id];
        while let Some(index) = current.last_focused {
            let node = current.children[index];
            match node {
                NodeId::Window(_) => return node,
                NodeId::Sublayout(sl) => {
                    current_id = sl;
                    current = &self.sublayouts[current_id];
                }
            }
        }
        return NodeId::Sublayout(current_id);
    }

    pub fn add_window_at_focused(&mut self, client: ClientId, xdg_toplevel: XdgToplevel) {
        let focused = self.find_focused();
        let (parent, is_window) = match focused {
            NodeId::Window(w) => (self.windows[w].parent, true),
            NodeId::Sublayout(sl) => (sl, false),
        };
        let sublayout = &mut self.sublayouts[parent];

        let window = Window {
            parent,
            xdg_toplevel,
            client,
            cached_geometry: None,
        };
        let window_id = self.windows.insert(window);

        if is_window {
            let index = sublayout.last_focused.unwrap() + 1;
            sublayout.children.insert(index, NodeId::Window(window_id));
        } else {
            sublayout.children.push(NodeId::Window(window_id));
        };
    }

    pub fn remove_window(&mut self, window: WindowId) {
        {
            let w = &self.windows[window];
            let sl = &mut self.sublayouts[w.parent];
            sl.children.retain(|c| c != &NodeId::Window(window));
        }
        self.windows.remove(window);
    }

    // TODO: Find a way to partially borrow self
    fn recalculate_sl(
        windows: &mut SlotMap<WindowId, Window>,
        sublayouts: &SlotMap<SublayoutId, Sublayout>,
        sl: SublayoutId,
        geometry: Geometry,
        configured: &mut Vec<(ClientId, XdgToplevel, Geometry)>,
    ) {
        let children = &sublayouts[sl].children;
        let cw = geometry.w / children.len() as u32;
        println!("{} {} {}", geometry.w, children.len(), cw);
        for (i, child) in children.iter().enumerate() {
            let geometry = Geometry {
                x: geometry.x + cw * i as u32,
                y: geometry.y,
                w: cw,
                h: geometry.h,
            };
            match *child {
                NodeId::Window(w) => {
                    let window = &mut windows[w];
                    if window.cached_geometry != Some(geometry) {
                        window.cached_geometry = Some(geometry);
                        configured.push((window.client, window.xdg_toplevel, geometry));
                    }
                }
                NodeId::Sublayout(sl) => {
                    Self::recalculate_sl(windows, sublayouts, sl, geometry, configured);
                }
            }
        }
    }

    pub fn recalculate(&mut self, geometry: Geometry) -> Vec<(ClientId, XdgToplevel, Geometry)> {
        let mut configured = Vec::new();
        Self::recalculate_sl(
            &mut self.windows,
            &self.sublayouts,
            self.root,
            geometry,
            &mut configured,
        );
        configured
    }
}

impl App {
    pub fn surface_geometry(&self, id: ClientId, surface: WlSurface) -> Option<Geometry> {
        let client = &self.clients[id];
        let toplevel = client.toplevel_from_surface(surface)?;
        let window = self
            .layout
            .windows
            .values()
            .find(|w| w.xdg_toplevel == toplevel)?;
        window.cached_geometry
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self::new()
    }
}
