//! Generational State Machine & State Types for VPNBridge

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// Strong identifier for an Android Network handle (e.g. from android_getaddrinfofornetwork / network.getNetworkHandle())
pub type VpnNetworkHandle = u64;

/// Generational tracking for active VPN connections.
///
/// When an active VPN disconnects or changes, the generation increases monotonically,
/// and the prior `CancellationToken` is cancelled immediately. All sockets created under
/// previous generations are aborted.
#[derive(Clone, Debug)]
pub struct VpnGeneration {
    generation: Arc<AtomicU64>,
    network_handle: Arc<AtomicU64>,
    cancel_token: Arc<tokio::sync::RwLock<CancellationToken>>,
}

impl VpnGeneration {
    pub fn new() -> Self {
        Self {
            generation: Arc::new(AtomicU64::new(0)),
            network_handle: Arc::new(AtomicU64::new(0)),
            cancel_token: Arc::new(tokio::sync::RwLock::new(CancellationToken::new())),
        }
    }

    /// Current generation number.
    pub fn current_generation(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    /// Current network handle bound to this generation.
    pub fn current_network_handle(&self) -> u64 {
        self.network_handle.load(Ordering::SeqCst)
    }

    /// Retrieve the cancellation token for the active generation.
    pub async fn cancellation_token(&self) -> CancellationToken {
        self.cancel_token.read().await.clone()
    }

    /// Atomically activate a new VPN network generation.
    /// Cancels all existing generation flows immediately.
    pub async fn advance_generation(&self, new_network_handle: VpnNetworkHandle) -> u64 {
        let old_gen = self.generation.fetch_add(1, Ordering::SeqCst);
        let new_gen = old_gen + 1;
        self.network_handle.store(new_network_handle, Ordering::SeqCst);

        let mut token_guard = self.cancel_token.write().await;
        token_guard.cancel(); // Cancel all old generation tasks
        *token_guard = CancellationToken::new(); // Fresh token for new generation

        tracing::info!(
            old_generation = old_gen,
            new_generation = new_gen,
            new_network_handle = new_network_handle,
            "Advanced VPN generation; invalidated previous flows"
        );
        new_gen
    }

    /// Invalidate the current VPN generation without setting a new valid handle (fail-closed).
    pub async fn invalidate(&self) -> u64 {
        let old_gen = self.generation.fetch_add(1, Ordering::SeqCst);
        let new_gen = old_gen + 1;
        self.network_handle.store(0, Ordering::SeqCst);

        let mut token_guard = self.cancel_token.write().await;
        token_guard.cancel();
        *token_guard = CancellationToken::new();

        tracing::warn!(
            old_generation = old_gen,
            new_generation = new_gen,
            "VPN invalidated; triggered FAIL-CLOSED on all active flows"
        );
        new_gen
    }

    /// Verify if a given receipt belongs to the currently active generation.
    pub fn is_generation_valid(&self, receipt: &VpnBindingReceipt) -> bool {
        let current = self.current_generation();
        let handle = self.current_network_handle();
        current == receipt.generation && handle != 0 && handle == receipt.network_handle
    }
}

impl Default for VpnGeneration {
    fn default() -> Self {
        Self::new()
    }
}

/// Binding receipt issued when a socket is successfully bound to a VPN network.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VpnBindingReceipt {
    pub generation: u64,
    pub network_handle: VpnNetworkHandle,
    pub created_at_millis: u64,
}

/// Comprehensive Top-Level State Machine according to docs/01_SYSTEM_ARCHITECTURE.md
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GatewayState {
    Stopped,
    PreparingHotspot,
    WaitingForVpn,
    ValidatingVpn,
    ReadyLocal,
    SafeForwarding,
    VpnLostFailClosed,
    Revalidating,
}

impl GatewayState {
    pub fn is_forwarding_allowed(&self) -> bool {
        matches!(self, GatewayState::SafeForwarding)
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            GatewayState::Stopped => "Stopped",
            GatewayState::PreparingHotspot => "Preparing Local-Only Hotspot",
            GatewayState::WaitingForVpn => "Waiting for Android VPN",
            GatewayState::ValidatingVpn => "Validating VPN Capabilities",
            GatewayState::ReadyLocal => "Local Gateway Ready",
            GatewayState::SafeForwarding => "Safe Protected Forwarding",
            GatewayState::VpnLostFailClosed => "VPN Lost (Fail-Closed)",
            GatewayState::Revalidating => "Revalidating New VPN",
        }
    }
}

/// Snapshot of the active VPN state and DNS servers
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct VpnStateSnapshot {
    pub is_active: bool,
    pub generation: u64,
    pub network_handle: u64,
    pub interface_name: Option<String>,
    pub dns_servers: Vec<IpAddr>,
    pub is_vpn_transport: bool,
    pub validated_at_millis: u64,
}

/// Client session identifier for authenticated connections
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
