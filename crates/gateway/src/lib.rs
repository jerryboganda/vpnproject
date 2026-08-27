//! VPNBridge Gateway Engine
//!
//! SOCKS5 server, TCP/UDP forwarder, fail-closed VPN generational binding,
//! and DNS-through-VPN routing.

pub mod dns;
pub mod forwarder;
pub mod socks5;

pub use forwarder::{TcpForwarder, UdpForwarder};
pub use socks5::server::GatewayServer;
