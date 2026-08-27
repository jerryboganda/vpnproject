//! UDP ASSOCIATE Forwarding Engine with Generational Fail-Closed Protection & Flow GC

use crate::socks5::parser::{
    build_udp_header, parse_udp_header, write_server_response, TargetAddress, REP_GENERAL_FAILURE,
    REP_SUCCESS,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpStream, UdpSocket};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use vpnbridge_core::config::GatewayConfig;
use vpnbridge_core::error::{Error, Result};
use vpnbridge_core::traits::ProtectedSocketBinder;
use vpnbridge_metrics::MetricsTracker;

pub struct UdpForwarder;

impl UdpForwarder {
    pub async fn handle_udp_associate(
        mut client_control_stream: TcpStream,
        _client_target: TargetAddress,
        binder: Arc<dyn ProtectedSocketBinder>,
        _config: Arc<GatewayConfig>,
        metrics: MetricsTracker,
        cancel_token: CancellationToken,
    ) -> Result<()> {
        metrics.inc_udp_mapping();
        let _guard = ScopeGuard::new({
            let m = metrics.clone();
            move || m.dec_udp_mapping()
        });

        // 1. Create a local UDP relay socket for the client to send datagrams
        let local_relay_socket = Arc::new(
            UdpSocket::bind("0.0.0.0:0")
                .await
                .map_err(|e| Error::Io(format!("Failed to bind local UDP relay socket: {e}")))?,
        );

        let local_relay_addr = local_relay_socket
            .local_addr()
            .map_err(|e| Error::Io(e.to_string()))?;

        // 2. Inform client of the bound relay address
        write_server_response(&mut client_control_stream, REP_SUCCESS, local_relay_addr).await?;

        // 3. Create outbound VPN-bound UDP socket
        let (vpn_socket, receipt) = binder.create_bound_udp().await?;
        if !binder.generation().is_generation_valid(&receipt) {
            let _ = write_server_response(
                &mut client_control_stream,
                REP_GENERAL_FAILURE,
                "0.0.0.0:0".parse().unwrap(),
            )
            .await;
            return Err(Error::VpnLostFailClosed);
        }
        let vpn_socket = Arc::new(vpn_socket);

        // 4. Track flows with (ClientAddress, LastSeenInstant) for GC sweep
        let flows = Arc::new(RwLock::new(HashMap::<SocketAddr, (SocketAddr, Instant)>::new()));

        // Periodic GC sweep for stale UDP NAT flows (60s idle timeout)
        let gc_flows = {
            let flows_map = flows.clone();
            async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
                loop {
                    interval.tick().await;
                    let mut map = flows_map.write().await;
                    let now = Instant::now();
                    map.retain(|_, (_, last_seen)| now.duration_since(*last_seen).as_secs() < 60);
                }
            }
        };

        // Client -> VPN upstream relay task
        let relay_inbound = {
            let relay_sock = local_relay_socket.clone();
            let vpn_sock = vpn_socket.clone();
            let flows_map = flows.clone();
            let metrics_tracker = metrics.clone();
            async move {
                let mut buf = vec![0u8; 65536];
                loop {
                    let (len, client_peer) = match relay_sock.recv_from(&mut buf).await {
                        Ok(res) => res,
                        Err(e) => break Err::<(), Error>(Error::Io(e.to_string())),
                    };

                    let (target, header_offset) = match parse_udp_header(&buf[..len]) {
                        Ok(res) => res,
                        Err(e) => {
                            tracing::debug!(error = %e, "Dropped malformed SOCKS5 UDP datagram");
                            continue;
                        }
                    };

                    let dest_addr = match target {
                        TargetAddress::Socket(addr) => addr,
                        TargetAddress::Domain(domain, port) => {
                            match tokio::net::lookup_host((domain.as_str(), port)).await {
                                Ok(mut addrs) => {
                                    if let Some(addr) = addrs.next() {
                                        addr
                                    } else {
                                        continue;
                                    }
                                }
                                Err(_) => continue,
                            }
                        }
                    };

                    let payload = &buf[header_offset..len];
                    if let Err(e) = vpn_sock.send_to(payload, dest_addr).await {
                        tracing::debug!(error = %e, dest = %dest_addr, "VPN UDP send failed");
                    } else {
                        metrics_tracker.record_tx(payload.len() as u64);
                        metrics_tracker.record_udp_packet();
                        flows_map.write().await.insert(dest_addr, (client_peer, Instant::now()));
                    }
                }
            }
        };

        // VPN upstream -> Client relay task
        let relay_outbound = {
            let relay_sock = local_relay_socket.clone();
            let vpn_sock = vpn_socket.clone();
            let flows_map = flows.clone();
            let metrics_tracker = metrics.clone();
            async move {
                let mut buf = vec![0u8; 65536];
                let mut out_pkt = Vec::with_capacity(65536);
                loop {
                    let (len, remote_peer) = match vpn_sock.recv_from(&mut buf).await {
                        Ok(res) => res,
                        Err(e) => break Err::<(), Error>(Error::Io(e.to_string())),
                    };

                    let client_peer = {
                        let map = flows_map.read().await;
                        map.get(&remote_peer).map(|(addr, _)| *addr)
                    };

                    if let Some(client_addr) = client_peer {
                        out_pkt.clear();
                        build_udp_header(&TargetAddress::Socket(remote_peer), &mut out_pkt);
                        out_pkt.extend_from_slice(&buf[..len]);

                        if let Err(e) = relay_sock.send_to(&out_pkt, client_addr).await {
                            tracing::debug!(error = %e, "Failed to relay UDP back to client");
                        } else {
                            metrics_tracker.record_rx(len as u64);
                        }
                    }
                }
            }
        };

        // Watch client TCP control connection (if closed, terminate UDP association)
        let watch_tcp = async {
            let mut buf = [0u8; 1];
            let _ = client_control_stream.peek(&mut buf).await;
            // Wait for EOF / disconnection
            loop {
                let mut discard = [0u8; 128];
                match client_control_stream.read(&mut discard).await {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {}
                }
            }
        };

        tokio::select! {
            _ = relay_inbound => tracing::debug!("Inbound UDP relay terminated"),
            _ = relay_outbound => tracing::debug!("Outbound UDP relay terminated"),
            _ = watch_tcp => tracing::debug!("Client TCP control stream closed; closing UDP association"),
            _ = gc_flows => {},
            _ = cancel_token.cancelled() => {
                tracing::warn!("Generational cancellation triggered; aborting UDP association");
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
