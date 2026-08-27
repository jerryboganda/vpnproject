use std::net::IpAddr;
use std::process::Command;
use vpnbridge_core::error::Result;

pub struct WindowsRouteManager;

impl WindowsRouteManager {
    /// Configure Windows system-wide SOCKS5 proxy via registry.
    pub async fn set_system_proxy(phone_ip: IpAddr, port: u16) -> Result<()> {
        let proxy_str = format!("socks={phone_ip}:{port}");
        tracing::info!(proxy = %proxy_str, "Configuring Windows System SOCKS5 Proxy");

        let _ = Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
                "/v",
                "ProxyServer",
                "/t",
                "REG_SZ",
                "/d",
                &proxy_str,
                "/f",
            ])
            .status();

        let _ = Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
                "/v",
                "ProxyEnable",
                "/t",
                "REG_DWORD",
                "/d",
                "1",
                "/f",
            ])
            .status();

        Ok(())
    }

    /// Disable Windows system-wide proxy.
    pub async fn clear_system_proxy() -> Result<()> {
        tracing::info!("Disabling Windows System Proxy");
        let _ = Command::new("reg")
            .args([
                "add",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings",
                "/v",
                "ProxyEnable",
                "/t",
                "REG_DWORD",
                "/d",
                "0",
                "/f",
            ])
            .status();

        Ok(())
    }

    /// Add explicit host route for the Phone Hotspot Gateway endpoint so SOCKS/control bypasses TUN.
    pub async fn add_phone_bypass_route(
        phone_ip: IpAddr,
        physical_interface_index: u32,
    ) -> Result<()> {
        tracing::info!(
            phone_ip = %phone_ip,
            if_index = physical_interface_index,
            "Adding phone gateway bypass route to Windows routing table"
        );

        // Execute Windows route command
        let status = Command::new("route")
            .args([
                "add",
                &phone_ip.to_string(),
                "mask",
                "255.255.255.255",
                &phone_ip.to_string(),
                "metric",
                "1",
                "if",
                &physical_interface_index.to_string(),
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                tracing::info!("Successfully added phone bypass route");
            }
            Ok(s) => {
                tracing::warn!(code = ?s.code(), "route add bypass returned non-zero (may require elevation)");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed executing route.exe");
            }
        }

        Ok(())
    }

    /// Install low-metric default route (0.0.0.0/0) directed to the TUN interface.
    pub async fn install_tun_default_route(tun_interface_index: u32) -> Result<()> {
        tracing::info!(
            tun_index = tun_interface_index,
            "Installing TUN full-tunnel default route (0.0.0.0/0)"
        );

        let status = Command::new("route")
            .args([
                "add",
                "0.0.0.0",
                "mask",
                "0.0.0.0",
                "10.0.0.1",
                "metric",
                "1",
                "if",
                &tun_interface_index.to_string(),
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                tracing::info!("Successfully installed TUN default route");
            }
            Ok(s) => {
                tracing::warn!(code = ?s.code(), "route add default returned non-zero (may require elevation)");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed executing route.exe");
            }
        }

        Ok(())
    }

    /// Restore original default route and remove bypass routes.
    pub async fn restore_routes() -> Result<()> {
        tracing::info!("Restoring original Windows routing tables");

        let _ = Command::new("route")
            .args(["delete", "0.0.0.0", "mask", "0.0.0.0", "10.0.0.1"])
            .status();

        let _ = Self::clear_system_proxy().await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_route_manager_interface() {
        let ip: IpAddr = "192.168.43.1".parse().unwrap();
        assert!(WindowsRouteManager::add_phone_bypass_route(ip, 1).await.is_ok());
        assert!(WindowsRouteManager::install_tun_default_route(2).await.is_ok());
        assert!(WindowsRouteManager::restore_routes().await.is_ok());
    }
}
