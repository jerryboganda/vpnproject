//! Socket Binding Traits for Platform Abstraction

use crate::error::{Error, Result};
use crate::state::{VpnBindingReceipt, VpnGeneration, VpnNetworkHandle};
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::net::{TcpStream, UdpSocket};

/// Trait defining the platform interface for binding upstream sockets to the Android VPN network.
///
/// On Android, this calls `android_setsocknetwork(handle, fd)` via NDK or `Network.bindSocket()`.
/// On test harnesses, `MockSocketBinder` simulates active VPN states, drops, and replacements.
#[async_trait]
pub trait ProtectedSocketBinder: Send + Sync {
    /// Retrieve the shared VpnGeneration tracker.
    fn generation(&self) -> &VpnGeneration;

    /// Connect a TCP socket to `target_addr`, ensuring the underlying socket is explicitly bound
    /// to the current validated VPN network BEFORE initiating the connection.
    async fn connect_tcp(&self, target_addr: SocketAddr) -> Result<(TcpStream, VpnBindingReceipt)>;

    /// Create a UDP socket explicitly bound to the current validated VPN network.
    async fn create_bound_udp(&self) -> Result<(UdpSocket, VpnBindingReceipt)>;

    /// Check if the binder currently has a valid VPN network ready for egress.
    fn is_vpn_ready(&self) -> bool {
        self.generation().current_network_handle() != 0
    }
}

/// Mock socket binder for unit tests, leak testing, and simulation on non-Android platforms.
pub struct MockSocketBinder {
    generation: VpnGeneration,
}

impl MockSocketBinder {
    pub fn new() -> Self {
        Self {
            generation: VpnGeneration::new(),
        }
    }

    pub fn with_active_handle(handle: VpnNetworkHandle) -> Self {
        let binder = Self::new();
        // Seed initial handle
        let gen = binder.generation.clone();
        tokio::spawn(async move {
            gen.advance_generation(handle).await;
        });
        binder
    }

    pub async fn activate_vpn(&self, handle: VpnNetworkHandle) -> u64 {
        self.generation.advance_generation(handle).await
    }

    pub async fn drop_vpn(&self) -> u64 {
        self.generation.invalidate().await
    }
}

impl Default for MockSocketBinder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProtectedSocketBinder for MockSocketBinder {
    fn generation(&self) -> &VpnGeneration {
        &self.generation
    }

    async fn connect_tcp(&self, target_addr: SocketAddr) -> Result<(TcpStream, VpnBindingReceipt)> {
        let handle = self.generation.current_network_handle();
        let gen = self.generation.current_generation();

        if handle == 0 {
            return Err(Error::VpnNotActive);
        }

        // Connect standard TCP stream
        let stream = TcpStream::connect(target_addr).await.map_err(|e| {
            Error::SocketBindFailed {
                network_handle: handle,
                reason: format!("Mock TCP connect failed: {e}"),
            }
        })?;

        let receipt = VpnBindingReceipt {
            generation: gen,
            network_handle: handle,
            created_at_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        Ok((stream, receipt))
    }

    async fn create_bound_udp(&self) -> Result<(UdpSocket, VpnBindingReceipt)> {
        let handle = self.generation.current_network_handle();
        let gen = self.generation.current_generation();

        if handle == 0 {
            return Err(Error::VpnNotActive);
        }

        let socket = UdpSocket::bind("0.0.0.0:0").await.map_err(|e| {
            Error::SocketBindFailed {
                network_handle: handle,
                reason: format!("Mock UDP bind failed: {e}"),
            }
        })?;

        let receipt = VpnBindingReceipt {
            generation: gen,
            network_handle: handle,
            created_at_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        };

        Ok((socket, receipt))
    }
}
