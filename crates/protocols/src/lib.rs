pub use pocowl_protocols_base::WaylandProtocol;
use pocowl_wlmessage::WaylandMessage;
use pocowl_wlvalue::WaylandValue;

pocowl_scanner::scan_protocol!("vendor/wayland/protocol/wayland.xml");
pocowl_scanner::scan_protocol!("vendor/wayland-protocols/stable/xdg-shell/xdg-shell.xml");
