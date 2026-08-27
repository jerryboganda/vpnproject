//! VPNBridge Windows Companion Tauri Backend

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use vpnbridge_metrics::{MetricsSnapshot, MetricsTracker};
use vpnbridge_windows_routing::{WindowsFirewallManager, WindowsRouteManager};

pub struct WindowsAppState {
    pub metrics: MetricsTracker,
    pub firewall: WindowsFirewallManager,
    pub is_connected: Arc<RwLock<bool>>,
    pub connection_mode: Arc<RwLock<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct WindowsStatusResponse {
    pub is_connected: bool,
    pub mode: String,
    pub is_kill_switch_armed: bool,
    pub status_text: String,
}

mod commands {
    use super::*;

    #[tauri::command]
    pub async fn get_status(state: State<'_, WindowsAppState>) -> Result<WindowsStatusResponse, String> {
        let connected = *state.is_connected.read().await;
        let mode = state.connection_mode.read().await.clone();
        let kill_switch = state.firewall.is_armed();

        Ok(WindowsStatusResponse {
            is_connected: connected,
            mode: mode.clone(),
            is_kill_switch_armed: kill_switch,
            status_text: if connected {
                format!("Protected via Phone VPN ({mode})")
            } else {
                "Disconnected".to_string()
            },
        })
    }

    #[tauri::command]
    pub async fn connect_tunnel(
        phone_ip: String,
        port: u16,
        _auth_token: String,
        mode: String,
        state: State<'_, WindowsAppState>,
    ) -> Result<String, String> {
        let mut connected_guard = state.is_connected.write().await;
        if *connected_guard {
            return Ok("Already connected".to_string());
        }

        let parsed_ip: std::net::IpAddr = phone_ip
            .parse()
            .map_err(|e| format!("Invalid phone IP {phone_ip}: {e}"))?;

        // 1. Configure system-wide proxy for instant Windows web & socket tunneling
        WindowsRouteManager::set_system_proxy(parsed_ip, port)
            .await
            .map_err(|e| e.to_string())?;

        // 2. In Full Tunnel mode, also arm kill-switch and install routes
        if mode == "full_tunnel" {
            let _ = state.firewall.arm_kill_switch(parsed_ip, 1).await;
            let _ = WindowsRouteManager::add_phone_bypass_route(parsed_ip, 1).await;
            let _ = WindowsRouteManager::install_tun_default_route(2).await;
        }

        *connected_guard = true;
        *state.connection_mode.write().await = mode.clone();

        tracing::info!(phone_ip = %phone_ip, port = port, mode = %mode, "Windows companion connected");
        Ok(format!("Connected in {mode} mode"))
    }

    #[tauri::command]
    pub async fn disconnect_tunnel(state: State<'_, WindowsAppState>) -> Result<String, String> {
        let mut connected_guard = state.is_connected.write().await;
        if !*connected_guard {
            return Ok("Already disconnected".to_string());
        }

        let _ = state.firewall.disarm_kill_switch().await;
        let _ = WindowsRouteManager::restore_routes().await;
        let _ = WindowsRouteManager::clear_system_proxy().await;

        *connected_guard = false;
        tracing::info!("Windows companion disconnected and restored network state");
        Ok("Disconnected".to_string())
    }

    #[tauri::command]
    pub async fn get_metrics(state: State<'_, WindowsAppState>) -> Result<MetricsSnapshot, String> {
        Ok(state.metrics.snapshot())
    }
}

pub fn run() {
    let app_state = WindowsAppState {
        metrics: MetricsTracker::new(),
        firewall: WindowsFirewallManager::new(),
        is_connected: Arc::new(RwLock::new(false)),
        connection_mode: Arc::new(RwLock::new("proxy".to_string())),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::connect_tunnel,
            commands::disconnect_tunnel,
            commands::get_metrics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
