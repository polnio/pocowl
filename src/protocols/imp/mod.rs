mod wayland;
mod xdg_shell;

#[derive(Default)]
pub struct ImpProtoStates {
    pub wayland: wayland::ImpWaylandState,
    pub xdg_shell: xdg_shell::ImpXdgShellState,
}

// trait WaylandWrapper {
//     type Inner;
//     fn inner(&self) -> Self::Inner;
// }
