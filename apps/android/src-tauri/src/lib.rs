//! VPNBridge Android Tauri Backend

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;
use vpnbridge_core::config::GatewayConfig;
use vpnbridge_core::state::{GatewayState, VpnGeneration};
use vpnbridge_core::traits::MockSocketBinder;
use vpnbridge_gateway::GatewayServer;
use vpnbridge_metrics::{MetricsSnapshot, MetricsTracker};

pub struct AppState {
    pub generation: VpnGeneration,
    pub metrics: MetricsTracker,
    pub server: Arc<RwLock<Option<Arc<GatewayServer>>>>,
    pub current_status: Arc<RwLock<GatewayState>>,
}

#[derive(Serialize, Deserialize)]
pub struct SharingStatusResponse {
    pub state: GatewayState,
    pub state_display: String,
    pub is_forwarding: bool,
    pub generation: u64,
    pub network_handle: u64,
}

mod commands {
    use super::*;

    #[tauri::command]
    pub async fn get_status(state: State<'_, AppState>) -> Result<SharingStatusResponse, String> {
        let current = *state.current_status.read().await;
        let gen = state.generation.current_generation();
        let handle = state.generation.current_network_handle();

        Ok(SharingStatusResponse {
            state: current,
            state_display: current.display_name().to_string(),
            is_forwarding: current.is_forwarding_allowed(),
            generation: gen,
            network_handle: handle,
        })
    }

    #[tauri::command]
    pub async fn start_sharing(
        auth_token: Option<String>,
        state: State<'_, AppState>,
    ) -> Result<String, String> {
        let mut server_guard = state.server.write().await;
        if server_guard.is_some() {
            return Ok("Gateway already running".to_string());
        }

        let mut config = GatewayConfig::default();
        if let Some(token) = auth_token {
            if !token.is_empty() {
                config.auth_token = token;
            }
        }

        let binder = Arc::new(MockSocketBinder::new());
        // Seed binder with active handle if any
        let handle = state.generation.current_network_handle();
        if handle != 0 {
            binder.activate_vpn(handle).await;
        }

        let server = Arc::new(GatewayServer::new(
            config,
            binder,
            state.metrics.clone(),
        ));

        let server_run = server.clone();
        tokio::spawn(async move {
            let _ = server_run.run().await;
        });

        *server_guard = Some(server);
        *state.current_status.write().await = GatewayState::SafeForwarding;

        tracing::info!("Started VPNBridge sharing gateway from UI");
        Ok("Sharing active".to_string())
    }

    #[tauri::command]
    pub async fn stop_sharing(state: State<'_, AppState>) -> Result<String, String> {
        let mut server_guard = state.server.write().await;
        if let Some(server) = server_guard.take() {
            server.shutdown_token().cancel();
            *state.current_status.write().await = GatewayState::Stopped;
            tracing::info!("Stopped VPNBridge sharing gateway");
            Ok("Sharing stopped".to_string())
        } else {
            Ok("Sharing was not active".to_string())
        }
    }

    #[tauri::command]
    pub async fn get_metrics(state: State<'_, AppState>) -> Result<MetricsSnapshot, String> {
        Ok(state.metrics.snapshot())
    }

    #[tauri::command]
    pub async fn notify_vpn_change(
        active: bool,
        network_handle: u64,
        _dns_servers: Vec<String>,
        state: State<'_, AppState>,
    ) -> Result<(), String> {
        if active && network_handle != 0 {
            state.generation.advance_generation(network_handle).await;
            *state.current_status.write().await = GatewayState::SafeForwarding;
            tracing::info!(network_handle, "VPN validated and active");
        } else {
            state.generation.invalidate().await;
            *state.current_status.write().await = GatewayState::VpnLostFailClosed;
            tracing::warn!("VPN lost; entered Fail-Closed state");
        }
        Ok(())
    }
}

pub fn run() {
    let app_state = AppState {
        generation: VpnGeneration::new(),
        metrics: MetricsTracker::new(),
        server: Arc::new(RwLock::new(None)),
        current_status: Arc::new(RwLock::new(GatewayState::Stopped)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::get_status,
            commands::start_sharing,
            commands::stop_sharing,
            commands::get_metrics,
            commands::notify_vpn_change,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
