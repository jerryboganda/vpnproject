//! Windows Wintun Adapter Abstraction & TUN-to-SOCKS Bridge Interface

pub mod adapter;
pub mod bridge;
pub mod wintun_ffi;

pub use adapter::{MockTunAdapter, TunAdapter, TunSession};
pub use bridge::{TunBridgeConfig, TunToSocksBridge};
pub use wintun_ffi::{WintunAdapter, WintunApi, WintunSession};
