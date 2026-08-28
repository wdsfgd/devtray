pub mod bridge;
pub mod tray;

pub use bridge::{BridgeError, SlintAppController};
pub use tray::{format_tray_tooltip, load_tray_icon, DevTraySysTray};
