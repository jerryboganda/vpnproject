//! Hotspot-Local SOCKS5 Gateway Server

use crate::forwarder::{TcpForwarder, UdpForwarder};
use crate::socks5::parser::{
    read_client_greeting, read_client_request, read_user_pass_auth, write_server_method,
    write_server_response, write_user_pass_auth_response, Socks5Command, AUTH_METHOD_NONE,
    AUTH_METHOD_NO_ACCEPTABLE, AUTH_METHOD_USER_PASS, REP_CMD_NOT_SUPPORTED,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use vpnbridge_core::config::GatewayConfig;
use vpnbridge_core::error::{Error, Result};
use vpnbridge_core::traits::ProtectedSocketBinder;
use vpnbridge_metrics::MetricsTracker;
use vpnbridge_protocol::verify_auth_token;

/// SOCKS5 Proxy Gateway Server running on the Android Local-Only Hotspot interface.
pub struct GatewayServer {
    config: Arc<GatewayConfig>,
    binder: Arc<dyn ProtectedSocketBinder>,
    metrics: MetricsTracker,
    shutdown_token: CancellationToken,
}

impl GatewayServer {
    pub fn new(
        config: GatewayConfig,
        binder: Arc<dyn ProtectedSocketBinder>,
        metrics: MetricsTracker,
    ) -> Self {
        Self {
            config: Arc::new(config),
            binder,
            metrics,
            shutdown_token: CancellationToken::new(),
        }
    }

    pub fn shutdown_token(&self) -> CancellationToken {
        self.shutdown_token.clone()
    }

    pub fn metrics(&self) -> MetricsTracker {
        self.metrics.clone()
    }

    /// Start the gateway server listener loop.
    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(self.config.listen_addr).await.map_err(|e| {
            Error::Io(format!(
                "Failed to bind gateway listener to {}: {e}",
                self.config.listen_addr
            ))
        })?;

        tracing::info!(
            listen_addr = %self.config.listen_addr,
            require_auth = self.config.require_auth,
            "VPNBridge Gateway Server listening"
        );

        let shutdown_token = self.shutdown_token.clone();

        loop {
            tokio::select! {
                _ = shutdown_token.cancelled() => {
                    tracing::info!("Gateway server received shutdown signal; stopping accept loop");
                    break;
                }
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok((stream, peer_addr)) => {
                            let config = self.config.clone();
                            let binder = self.binder.clone();
                            let metrics = self.metrics.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_client(stream, peer_addr, binder, config, metrics).await {
                                    tracing::debug!(peer = %peer_addr, error = %e, "Client session closed");
                                }
                            });
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, "Accept error in gateway listener");
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_client(
        mut stream: TcpStream,
        _peer_addr: SocketAddr,
        binder: Arc<dyn ProtectedSocketBinder>,
        config: Arc<GatewayConfig>,
        metrics: MetricsTracker,
    ) -> Result<()> {
        // 1. Read SOCKS5 greeting
        let client_methods = read_client_greeting(&mut stream).await?;

        // 2. Negotiate Authentication
        if config.require_auth {
            if !client_methods.contains(&AUTH_METHOD_USER_PASS) {
                write_server_method(&mut stream, AUTH_METHOD_NO_ACCEPTABLE).await?;
                metrics.record_auth_failure();
                return Err(Error::AuthenticationFailed(
                    "Client does not support USER_PASS authentication".to_string(),
                ));
            }

            write_server_method(&mut stream, AUTH_METHOD_USER_PASS).await?;

            let (_username, password) = read_user_pass_auth(&mut stream).await?;
            if !verify_auth_token(&config.auth_token, &password) {
                write_user_pass_auth_response(&mut stream, false).await?;
                metrics.record_auth_failure();
                return Err(Error::AuthenticationFailed(
                    "Invalid client authentication token".to_string(),
                ));
            }

            write_user_pass_auth_response(&mut stream, true).await?;
        } else {
            if !client_methods.contains(&AUTH_METHOD_NONE) {
                write_server_method(&mut stream, AUTH_METHOD_NO_ACCEPTABLE).await?;
                return Err(Error::AuthenticationFailed(
                    "Client does not offer NO_AUTH".to_string(),
                ));
            }
            write_server_method(&mut stream, AUTH_METHOD_NONE).await?;
        }

        // 3. Read Client SOCKS5 Request
        let req = read_client_request(&mut stream).await?;
        let cancel_token = binder.generation().cancellation_token().await;

        match req.command {
            Socks5Command::Connect => {
                TcpForwarder::forward_tcp(stream, req.target, binder, config, metrics, cancel_token)
                    .await
            }
            Socks5Command::UdpAssociate => {
                UdpForwarder::handle_udp_associate(
                    stream,
                    req.target,
                    binder,
                    config,
                    metrics,
                    cancel_token,
                )
                .await
            }
            Socks5Command::Bind => {
                let _ = write_server_response(
                    &mut stream,
                    REP_CMD_NOT_SUPPORTED,
                    "0.0.0.0:0".parse().unwrap(),
                )
                .await;
                Err(Error::Socks5Error("SOCKS5 BIND is not supported".to_string()))
            }
        }
    }
}
