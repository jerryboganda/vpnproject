use std::sync::Arc;
use tokio::net::UdpSocket;
use vpnbridge_core::traits::MockSocketBinder;
use vpnbridge_gateway::dns::VpnDnsResolver;

#[tokio::test]
async fn test_dns_resolver_routes_through_vpn() {
    // 1. Start mock DNS UDP responder
    let mock_dns = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let dns_addr = mock_dns.local_addr().unwrap();

    tokio::spawn(async move {
        let mut buf = vec![0u8; 512];
        loop {
            if let Ok((len, peer)) = mock_dns.recv_from(&mut buf).await {
                if len >= 12 {
                    let mut resp = buf[..len].to_vec();
                    resp[2] |= 0x80; // QR = response
                    let _ = mock_dns.send_to(&resp, peer).await;
                }
            }
        }
    });

    // 2. Setup Resolver with active Mock VPN
    let binder = Arc::new(MockSocketBinder::new());
    binder.activate_vpn(777).await;

    let resolver = VpnDnsResolver::new(binder.clone(), vec![dns_addr]);

    // Construct mock DNS query header (12 bytes)
    let query_packet = vec![0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    // Query should succeed over VPN-bound socket
    let response = resolver.resolve_packet(&query_packet).await.expect("DNS query should succeed");
    assert_eq!(response[0..2], [0x12, 0x34], "Transaction ID must match");
    assert_ne!(response[2] & 0x80, 0, "Response flag must be set");

    // 3. Test Fail-Closed on VPN Loss
    binder.drop_vpn().await; // Invalidate VPN!

    let fail_res = resolver.resolve_packet(&query_packet).await;
    assert!(
        fail_res.is_err(),
        "DNS resolution MUST fail closed when VPN is lost"
    );
}
