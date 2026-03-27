pub use pocowl_protocols_base::WaylandProtocol;

use wayland::*;
pocowl_scanner::scan_protocol!("vendor/wayland/protocol/wayland.xml");
pocowl_scanner::scan_protocol!("vendor/wayland-protocols/stable/xdg-shell/xdg-shell.xml");
