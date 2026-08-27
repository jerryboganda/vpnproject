//! Abstract Contract for TUN-to-SOCKS Bridge

use async_trait::async_trait;
use std::net::SocketAddr;
use vpnbridge_core::error::Result;

#[derive(Clone, Debug)]
pub struct TunBridgeConfig {
    pub socks5_endpoint: SocketAddr,
    pub auth_token: Option<String>,
    pub tun_name: String,
    pub tun_ipv4: std::net::Ipv4Addr,
    pub tun_netmask: std::net::Ipv4Addr,
}

#[async_trait]
pub trait TunToSocksBridge: Send + Sync {
    async fn start(&self, config: TunBridgeConfig) -> Result<()>;
    async fn stop(&self) -> Result<()>;
    fn is_running(&self) -> bool;
}
