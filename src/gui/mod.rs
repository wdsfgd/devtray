pub mod bridge;
pub mod tray;

pub use bridge::{BridgeError, SlintAppController, TaskItemSnapshot, TaskSnapshot};
pub use tray::{format_tray_tooltip, load_tray_icon, DevTraySysTray};
