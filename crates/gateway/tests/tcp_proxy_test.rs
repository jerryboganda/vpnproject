//! End-to-End SOCKS5 TCP Proxy Integration Test

use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;
use vpnbridge_core::config::GatewayConfig;
use vpnbridge_core::traits::MockSocketBinder;
use vpnbridge_gateway::GatewayServer;
use vpnbridge_metrics::MetricsTracker;
use vpnbridge_test_support::TcpEchoServer;

#[tokio::test]
async fn test_socks5_tcp_proxy_with_user_pass_auth() {
    // 1. Start Echo Server
    let (_echo_server, echo_addr) = TcpEchoServer::start().await.expect("Echo server failed");

    // 2. Start Gateway Server with Mock Socket Binder
    let binder = Arc::new(MockSocketBinder::new());
    binder.activate_vpn(999).await; // Activate mock VPN

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let gateway_addr = listener.local_addr().unwrap();
    drop(listener);

    let config = GatewayConfig {
        listen_addr: gateway_addr,
        auth_token: "secret-token-12345".to_string(),
        require_auth: true,
        ..Default::default()
    };
    let metrics = MetricsTracker::new();
    let server = Arc::new(GatewayServer::new(config.clone(), binder.clone(), metrics.clone()));

    let server_clone = server.clone();
    tokio::spawn(async move {
        let _ = server_clone.run().await;
    });

    sleep(Duration::from_millis(100)).await;

    // 3. Connect SOCKS5 Client to Gateway
    let mut client = TcpStream::connect(gateway_addr).await.expect("Client connect failed");

    // Send greeting offering USER_PASS (0x02)
    client.write_all(&[0x05, 0x01, 0x02]).await.unwrap();

    let mut method_resp = [0u8; 2];
    client.read_exact(&mut method_resp).await.unwrap();
    assert_eq!(method_resp, [0x05, 0x02], "Server should accept USER_PASS auth");

    // Send auth request: ver=1, ulen=6, "client", plen=18, "secret-token-12345"
    let mut auth_req = vec![0x01, 0x06];
    auth_req.extend_from_slice(b"client");
    auth_req.push(config.auth_token.len() as u8);
    auth_req.extend_from_slice(config.auth_token.as_bytes());
    client.write_all(&auth_req).await.unwrap();

    let mut auth_resp = [0u8; 2];
    client.read_exact(&mut auth_resp).await.unwrap();
    assert_eq!(auth_resp, [0x01, 0x00], "Auth should succeed");

    // Send CONNECT request to Echo Server IP & port
    let mut conn_req = vec![0x05, 0x01, 0x00, 0x01]; // VER=5, CMD=1 (CONNECT), RSV=0, ATYP=1 (IPv4)
    if let std::net::SocketAddr::V4(v4) = echo_addr {
        conn_req.extend_from_slice(&v4.ip().octets());
        conn_req.extend_from_slice(&v4.port().to_be_bytes());
    }
    client.write_all(&conn_req).await.unwrap();

    let mut conn_resp = [0u8; 10];
    client.read_exact(&mut conn_resp).await.unwrap();
    assert_eq!(conn_resp[0], 0x05);
    assert_eq!(conn_resp[1], 0x00, "CONNECT should return REP_SUCCESS (0x00)");

    // 4. Send payload and verify echo
    let payload = b"Hello VPNBridge Protected TCP!";
    client.write_all(payload).await.unwrap();

    let mut recv_buf = vec![0u8; payload.len()];
    client.read_exact(&mut recv_buf).await.unwrap();
    assert_eq!(&recv_buf[..], payload);

    // 5. Verify metrics
    let snap = metrics.snapshot();
    assert!(snap.bytes_tx >= payload.len() as u64);
    assert!(snap.bytes_rx >= payload.len() as u64);

    // Cleanup
    server.shutdown_token().cancel();
}

#[tokio::test]
async fn test_socks5_rejects_invalid_auth() {
    let binder = Arc::new(MockSocketBinder::new());
    binder.activate_vpn(999).await;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let gateway_addr = listener.local_addr().unwrap();
    drop(listener);

    let config = GatewayConfig {
        listen_addr: gateway_addr,
        auth_token: "correct-secret".to_string(),
        require_auth: true,
        ..Default::default()
    };

    let metrics = MetricsTracker::new();
    let server = Arc::new(GatewayServer::new(config, binder, metrics.clone()));

    let s = server.clone();
    tokio::spawn(async move {
        let _ = s.run().await;
    });

    sleep(Duration::from_millis(100)).await;

    let mut client = TcpStream::connect(gateway_addr).await.expect("Client connect failed");
    client.write_all(&[0x05, 0x01, 0x02]).await.unwrap();

    let mut method_resp = [0u8; 2];
    client.read_exact(&mut method_resp).await.unwrap();

    // Send WRONG password
    let mut auth_req = vec![0x01, 0x04];
    auth_req.extend_from_slice(b"user");
    auth_req.push(12);
    auth_req.extend_from_slice(b"wrong-secret");
    client.write_all(&auth_req).await.unwrap();

    let mut auth_resp = [0u8; 2];
    client.read_exact(&mut auth_resp).await.unwrap();
    assert_eq!(auth_resp, [0x01, 0x01], "Auth should be rejected with 0x01 status");

    let snap = metrics.snapshot();
    assert_eq!(snap.auth_failures_count, 1);

    server.shutdown_token().cancel();
}
