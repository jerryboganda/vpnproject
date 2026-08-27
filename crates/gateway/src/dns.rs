//! DNS Resolution over VPN-Bound Sockets

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use vpnbridge_core::error::{Error, Result};
use vpnbridge_core::traits::ProtectedSocketBinder;

/// DNS resolver that explicitly routes queries through the Android VPN network.
pub struct VpnDnsResolver {
    binder: Arc<dyn ProtectedSocketBinder>,
    dns_servers: Vec<SocketAddr>,
    timeout: Duration,
}

impl VpnDnsResolver {
    pub fn new(binder: Arc<dyn ProtectedSocketBinder>, dns_servers: Vec<SocketAddr>) -> Self {
        Self {
            binder,
            dns_servers,
            timeout: Duration::from_secs(5),
        }
    }

    pub fn from_ips(binder: Arc<dyn ProtectedSocketBinder>, ips: Vec<IpAddr>) -> Self {
        let dns_servers = ips.into_iter().map(|ip| SocketAddr::new(ip, 53)).collect();
        Self::new(binder, dns_servers)
    }

    pub fn set_dns_servers(&mut self, servers: Vec<SocketAddr>) {
        self.dns_servers = servers;
    }

    /// Resolve a DNS query packet by forwarding it over a VPN-bound UDP socket to the configured VPN DNS servers.
    pub async fn resolve_packet(&self, query_packet: &[u8]) -> Result<Vec<u8>> {
        if self.dns_servers.is_empty() {
            return Err(Error::NetworkError("No VPN DNS servers configured".to_string()));
        }

        // Try configured VPN DNS servers in priority order
        for &target_addr in &self.dns_servers {
            let (vpn_socket, receipt) = self.binder.create_bound_udp().await?;

            if !self.binder.generation().is_generation_valid(&receipt) {
                return Err(Error::VpnLostFailClosed);
            }

            match self.send_query(&vpn_socket, target_addr, query_packet).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    tracing::warn!(server = %target_addr, error = %e, "DNS query to server failed; trying next");
                }
            }
        }

        Err(Error::NetworkError("All VPN DNS servers failed to respond".to_string()))
    }

    async fn send_query(
        &self,
        socket: &UdpSocket,
        target_addr: SocketAddr,
        query: &[u8],
    ) -> Result<Vec<u8>> {
        socket.send_to(query, target_addr).await.map_err(|e| {
            Error::Io(format!("Failed to send DNS query to {target_addr}: {e}"))
        })?;

        let mut buf = vec![0u8; 4096];
        let recv_future = socket.recv_from(&mut buf);

        let (len, from_addr) = tokio::time::timeout(self.timeout, recv_future)
            .await
            .map_err(|_| Error::NetworkError(format!("DNS query to {target_addr} timed out")))?
            .map_err(|e| Error::Io(format!("DNS receive failed: {e}")))?;

        if from_addr != target_addr {
            tracing::warn!(expected = %target_addr, got = %from_addr, "Spurious DNS response sender");
        }

        buf.truncate(len);
        Ok(buf)
    }
}
