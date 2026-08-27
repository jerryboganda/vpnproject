//! TCP Proxy Forwarding Engine with Generational Fail-Closed Cancellation

use crate::socks5::parser::{write_server_response, TargetAddress, REP_GENERAL_FAILURE, REP_SUCCESS};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;
use vpnbridge_core::config::GatewayConfig;
use vpnbridge_core::error::{Error, Result};
use vpnbridge_core::traits::ProtectedSocketBinder;
use vpnbridge_metrics::MetricsTracker;

pub struct TcpForwarder;

impl TcpForwarder {
    pub async fn forward_tcp(
        mut client_stream: TcpStream,
        target: TargetAddress,
        binder: Arc<dyn ProtectedSocketBinder>,
        config: Arc<GatewayConfig>,
        metrics: MetricsTracker,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        metrics.inc_tcp_stream();
        let _guard = ScopeGuard::new({
            let m = metrics.clone();
            move || m.dec_tcp_stream()
        });

        // 1. Resolve Target Address to SocketAddr
        let target_socket_addr = match target {
            TargetAddress::Socket(addr) => addr,
            TargetAddress::Domain(ref domain, port) => {
                let mut addrs = tokio::net::lookup_host((domain.as_str(), port)).await.map_err(|e| {
                    Error::NetworkError(format!("DNS lookup failed for {domain}:{port}: {e}"))
                })?;

                match addrs.next() {
                    Some(addr) => addr,
                    None => {
                        let _ = write_server_response(
                            &mut client_stream,
                            REP_GENERAL_FAILURE,
                            "0.0.0.0:0".parse().unwrap(),
                        )
                        .await;
                        return Err(Error::NetworkError(format!("No IP resolved for {domain}")));
                    }
                }
            }
        };

        // 2. Connect upstream via VPN-bound socket binder
        let (upstream_stream, receipt) = match binder.connect_tcp(target_socket_addr).await {
            Ok(res) => res,
            Err(e) => {
                tracing::warn!(target = %target_socket_addr, error = %e, "Failed to connect upstream VPN socket");
                let _ = write_server_response(
                    &mut client_stream,
                    REP_GENERAL_FAILURE,
                    "0.0.0.0:0".parse().unwrap(),
                )
                .await;
                return Err(e);
            }
        };

        // 3. Double-check receipt validity against active generation
        if !binder.generation().is_generation_valid(&receipt) {
            tracing::error!("VPN generation changed during connect; failing closed");
            let _ = write_server_response(
                &mut client_stream,
                REP_GENERAL_FAILURE,
                "0.0.0.0:0".parse().unwrap(),
            )
            .await;
            return Err(Error::VpnLostFailClosed);
        }

        // 4. Send SOCKS5 SUCCESS reply to client
        let local_bound = client_stream
            .local_addr()
            .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());
        write_server_response(&mut client_stream, REP_SUCCESS, local_bound).await?;

        // 5. Bidirectional copy loop with generational cancellation & idle timeout
        let (mut client_read, mut client_write) = client_stream.into_split();
        let (mut upstream_read, mut upstream_write) = upstream_stream.into_split();

        let client_to_upstream = async {
            let mut buf = vec![0u8; 16384];
            loop {
                let n = match client_read.read(&mut buf).await {
                    Ok(0) => break Ok::<(), Error>(()), // EOF
                    Ok(n) => n,
                    Err(e) => break Err(Error::Io(e.to_string())),
                };
                if let Err(e) = upstream_write.write_all(&buf[..n]).await {
                    break Err(Error::Io(e.to_string()));
                }
                metrics.record_tx(n as u64);
            }
        };

        let upstream_to_client = async {
            let mut buf = vec![0u8; 16384];
            loop {
                let n = match upstream_read.read(&mut buf).await {
                    Ok(0) => break Ok::<(), Error>(()), // EOF
                    Ok(n) => n,
                    Err(e) => break Err(Error::Io(e.to_string())),
                };
                if let Err(e) = client_write.write_all(&buf[..n]).await {
                    break Err(Error::Io(e.to_string()));
                }
                metrics.record_rx(n as u64);
            }
        };

        tokio::select! {
            res = tokio::time::timeout(config.tcp_idle_timeout, client_to_upstream) => {
                match res {
                    Ok(Ok(())) => tracing::debug!("Client closed connection"),
                    Ok(Err(e)) => tracing::debug!(error = %e, "Client-to-upstream transfer ended"),
                    Err(_) => tracing::debug!("TCP client idle timeout expired"),
                }
            }
            res = tokio::time::timeout(config.tcp_idle_timeout, upstream_to_client) => {
                match res {
                    Ok(Ok(())) => tracing::debug!("Upstream closed connection"),
                    Ok(Err(e)) => tracing::debug!(error = %e, "Upstream-to-client transfer ended"),
                    Err(_) => tracing::debug!("TCP upstream idle timeout expired"),
                }
            }
            _ = cancel_token.cancelled() => {
                tracing::warn!("Generational cancellation triggered! Aborting TCP stream immediately");
                return Err(Error::VpnLostFailClosed);
            }
        }

        Ok(())
    }
}

struct ScopeGuard<F: FnOnce()> {
    cleanup: Option<F>,
}

impl<F: FnOnce()> ScopeGuard<F> {
    fn new(cleanup: F) -> Self {
        Self {
            cleanup: Some(cleanup),
        }
    }
}

impl<F: FnOnce()> Drop for ScopeGuard<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}
