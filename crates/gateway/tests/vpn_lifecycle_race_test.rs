//! VPN Lifecycle, Generational Fail-Closed & Race Conditions Test

use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::sleep;
use vpnbridge_core::config::GatewayConfig;
use vpnbridge_core::traits::MockSocketBinder;
use vpnbridge_gateway::GatewayServer;
use vpnbridge_metrics::MetricsTracker;
use vpnbridge_test_support::{LeakDetector, TcpEchoServer};

#[tokio::test]
async fn test_vpn_disconnect_during_active_transfer_triggers_instant_fail_closed() {
    let (_echo_server, echo_addr) = TcpEchoServer::start().await.expect("Echo server start");
    let leak_detector = LeakDetector::new();

    let binder = Arc::new(MockSocketBinder::new());
    binder.activate_vpn(100).await; // Generation 1, Handle 100

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let gateway_addr = listener.local_addr().unwrap();
    drop(listener);

    let config = GatewayConfig {
        listen_addr: gateway_addr,
        require_auth: false,
        ..Default::default()
    };

    let metrics = MetricsTracker::new();
    let server = Arc::new(GatewayServer::new(config, binder.clone(), metrics.clone()));

    let s = server.clone();
    tokio::spawn(async move {
        let _ = s.run().await;
    });

    sleep(Duration::from_millis(100)).await;

    // Connect Client #1
    let mut client1 = TcpStream::connect(gateway_addr).await.unwrap();
    client1.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
    let mut greeting_resp = [0u8; 2];
    client1.read_exact(&mut greeting_resp).await.unwrap();

    let mut conn_req = vec![0x05, 0x01, 0x00, 0x01];
    if let std::net::SocketAddr::V4(v4) = echo_addr {
        conn_req.extend_from_slice(&v4.ip().octets());
        conn_req.extend_from_slice(&v4.port().to_be_bytes());
    }
    client1.write_all(&conn_req).await.unwrap();

    let mut conn_resp = [0u8; 10];
    client1.read_exact(&mut conn_resp).await.unwrap();
    assert_eq!(conn_resp[1], 0x00, "CONNECT must succeed initially");

    // Send first chunk
    client1.write_all(b"Chunk 1 before VPN drop").await.unwrap();
    let mut recv_buf = [0u8; 23];
    client1.read_exact(&mut recv_buf).await.unwrap();
    assert_eq!(&recv_buf, b"Chunk 1 before VPN drop");

    // NOW TRIGGER SUDDEN VPN DISCONNECT!
    tracing::info!(">>> INJECTING SUDDEN VPN DISCONNECT <<<");
    binder.drop_vpn().await; // Invalidate generation

    // Attempt to read from client1: must receive EOF (stream was aborted)
    let mut eof_buf = [0u8; 64];
    let read_res = tokio::time::timeout(Duration::from_secs(2), client1.read(&mut eof_buf)).await;
    match read_res {
        Ok(Ok(0)) => tracing::info!("Client stream correctly received EOF on VPN drop"),
        Ok(Err(e)) => tracing::info!("Client stream correctly aborted with error: {e}"),
        Ok(Ok(n)) => panic!("Received unexpected {n} bytes after VPN dropped!"),
        Err(_) => panic!("Timed out waiting for socket close on VPN drop"),
    }

    // Attempt new connection while VPN is down: MUST FAIL
    let mut client2 = TcpStream::connect(gateway_addr).await.unwrap();
    client2.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
    client2.read_exact(&mut greeting_resp).await.unwrap();
    client2.write_all(&conn_req).await.unwrap();

    let mut conn2_resp = [0u8; 10];
    client2.read_exact(&mut conn2_resp).await.unwrap();
    assert_ne!(
        conn2_resp[1], 0x00,
        "New connections while VPN is disconnected MUST fail closed"
    );

    // NOW RECOVER: Activate new VPN generation (e.g. Generation 3, Handle 200)
    tracing::info!(">>> RECOVERING VPN WITH NEW GENERATION <<<");
    binder.activate_vpn(200).await;

    // New connection after recovery: MUST SUCCEED
    let mut client3 = TcpStream::connect(gateway_addr).await.unwrap();
    client3.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
    client3.read_exact(&mut greeting_resp).await.unwrap();
    client3.write_all(&conn_req).await.unwrap();

    let mut conn3_resp = [0u8; 10];
    client3.read_exact(&mut conn3_resp).await.unwrap();
    assert_eq!(conn3_resp[1], 0x00, "CONNECT must succeed after VPN recovery");

    client3.write_all(b"Recovered message").await.unwrap();
    let mut rec_buf = [0u8; 17];
    client3.read_exact(&mut rec_buf).await.unwrap();
    assert_eq!(&rec_buf, b"Recovered message");

    // Zero leaks
    leak_detector.assert_no_leaks();

    server.shutdown_token().cancel();
}
