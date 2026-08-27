//! Controllable Mock VPN Controller

use std::sync::Arc;
use vpnbridge_core::state::VpnNetworkHandle;
use vpnbridge_core::traits::MockSocketBinder;

pub struct MockVpnController {
    binder: Arc<MockSocketBinder>,
}

impl MockVpnController {
    pub fn new() -> Self {
        Self {
            binder: Arc::new(MockSocketBinder::new()),
        }
    }

    pub fn binder(&self) -> Arc<MockSocketBinder> {
        self.binder.clone()
    }

    pub async fn connect_vpn(&self, handle: VpnNetworkHandle) -> u64 {
        self.binder.activate_vpn(handle).await
    }

    pub async fn disconnect_vpn(&self) -> u64 {
        self.binder.drop_vpn().await
    }

    pub async fn replace_vpn(&self, new_handle: VpnNetworkHandle) -> u64 {
        self.binder.activate_vpn(new_handle).await
    }
}

impl Default for MockVpnController {
    fn default() -> Self {
        Self::new()
    }
}
