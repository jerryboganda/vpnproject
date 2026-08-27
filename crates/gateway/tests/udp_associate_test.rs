//! SOCKS5 UDP ASSOCIATE Integration Test

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::sleep;
use vpnbridge_core::config::GatewayConfig;
use vpnbridge_core::traits::MockSocketBinder;
use vpnbridge_gateway::socks5::parser::{build_udp_header, parse_udp_header, TargetAddress};
use vpnbridge_gateway::GatewayServer;
use vpnbridge_metrics::MetricsTracker;
use vpnbridge_test_support::UdpEchoServer;

#[tokio::test]
async fn test_socks5_udp_associate_and_relay() {
    // 1. Start UDP Echo Server
    let (_echo_server, echo_addr) = UdpEchoServer::start().await.expect("UDP Echo server failed");

    // 2. Start Gateway Server with Mock Socket Binder
    let binder = Arc::new(MockSocketBinder::new());
    binder.activate_vpn(101).await;

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

    // 3. Connect TCP Control Stream
    let mut tcp_client = TcpStream::connect(gateway_addr).await.expect("Connect failed");

    // Greeting (NO_AUTH)
    tcp_client.write_all(&[0x05, 0x01, 0x00]).await.unwrap();
    let mut method_resp = [0u8; 2];
    tcp_client.read_exact(&mut method_resp).await.unwrap();
    assert_eq!(method_resp, [0x05, 0x00]);

    // Send UDP ASSOCIATE: VER=5, CMD=3 (UDP_ASSOCIATE), RSV=0, ATYP=1 (0.0.0.0:0)
    let req = [0x05, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    tcp_client.write_all(&req).await.unwrap();

    let mut resp = [0u8; 10];
    tcp_client.read_exact(&mut resp).await.unwrap();
    assert_eq!(resp[0], 0x05);
    assert_eq!(resp[1], 0x00, "UDP ASSOCIATE should return REP_SUCCESS (0x00)");
    assert_eq!(resp[3], 0x01, "Address type should be IPv4");

    let relay_port = u16::from_be_bytes([resp[8], resp[9]]);
    let relay_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, relay_port));

    // 4. Create Client UDP Socket and Send Encapsulated Packet
    let client_udp = UdpSocket::bind("127.0.0.1:0").await.unwrap();

    let payload = b"Hello VPNBridge UDP Protected Relay!";
    let mut packet = Vec::new();
    build_udp_header(&TargetAddress::Socket(echo_addr), &mut packet);
    packet.extend_from_slice(payload);

    client_udp.send_to(&packet, relay_addr).await.unwrap();

    // 5. Receive Echo Response from Relay
    let mut recv_buf = vec![0u8; 65536];
    let (len, _from) = tokio::time::timeout(
        Duration::from_secs(3),
        client_udp.recv_from(&mut recv_buf),
    )
    .await
    .expect("Timed out waiting for UDP echo")
    .unwrap();

    let (src_addr, header_offset) = parse_udp_header(&recv_buf[..len]).expect("Valid SOCKS5 UDP header");
    assert_eq!(src_addr, TargetAddress::Socket(echo_addr));
    assert_eq!(&recv_buf[header_offset..len], payload);

    let snap = metrics.snapshot();
    assert!(snap.total_udp_packets >= 1);

    server.shutdown_token().cancel();
}
