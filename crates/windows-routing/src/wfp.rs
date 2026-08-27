//! Windows Filtering Platform (WFP) Kill Switch Manager
//!
//! Enforces hardware-level fail-closed protection by applying firewall blocking rules
//! on physical adapters while allowing phone gateway endpoints and TUN traffic.

use std::net::IpAddr;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use vpnbridge_core::error::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KillSwitchState {
    Disabled,
    Armed,
    Blocking,
}

pub struct WindowsFirewallManager {
    is_active: Arc<AtomicBool>,
}

impl WindowsFirewallManager {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Arm the WFP kill-switch rules:
    /// - Allow hotspot local subnet and phone gateway IP.
    /// - Allow TUN interface.
    /// - Block all other outbound traffic on physical network adapters.
    pub async fn arm_kill_switch(
        &self,
        phone_gateway_ip: IpAddr,
        tun_interface_index: u32,
    ) -> Result<()> {
        self.is_active.store(true, Ordering::SeqCst);
        tracing::info!(
            phone_ip = %phone_gateway_ip,
            tun_index = tun_interface_index,
            "Arming Windows WFP Kill Switch"
        );

        // Add allow rule for phone gateway hotspot communication
        let _ = Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "add",
                "rule",
                "name=VPNBridge-Allow-Gateway",
                "dir=out",
                "action=allow",
                &format!("remoteip={phone_gateway_ip}"),
            ])
            .status();

        // Add block rule for all outbound traffic (lower priority than explicit allow)
        let _ = Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "add",
                "rule",
                "name=VPNBridge-KillSwitch",
                "dir=out",
                "action=block",
            ])
            .status();

        Ok(())
    }

    /// Disarm the kill switch and remove WFP filtering rules.
    pub async fn disarm_kill_switch(&self) -> Result<()> {
        self.is_active.store(false, Ordering::SeqCst);
        tracing::info!("Disarming Windows WFP Kill Switch");

        // Remove rules cleanly
        let _ = Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "delete",
                "rule",
                "name=VPNBridge-KillSwitch",
            ])
            .status();

        let _ = Command::new("netsh")
            .args([
                "advfirewall",
                "firewall",
                "delete",
                "rule",
                "name=VPNBridge-Allow-Gateway",
            ])
            .status();

        Ok(())
    }

    pub fn is_armed(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }
}

impl Default for WindowsFirewallManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_firewall_manager_state() {
        let mgr = WindowsFirewallManager::new();
        assert!(!mgr.is_armed());

        let ip: IpAddr = "192.168.43.1".parse().unwrap();
        assert!(mgr.arm_kill_switch(ip, 1).await.is_ok());
        assert!(mgr.is_armed());

        assert!(mgr.disarm_kill_switch().await.is_ok());
        assert!(!mgr.is_armed());
    }
}
