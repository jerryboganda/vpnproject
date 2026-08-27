use vpnbridge_core::error::Error;
use vpnbridge_core::state::{GatewayState, VpnBindingReceipt, VpnGeneration};
use vpnbridge_core::traits::{MockSocketBinder, ProtectedSocketBinder};

#[tokio::test]
async fn test_vpn_generation_advancement_and_cancellation() {
    let gen = VpnGeneration::new();
    assert_eq!(gen.current_generation(), 0);
    assert_eq!(gen.current_network_handle(), 0);

    let token_0 = gen.cancellation_token().await;
    assert!(!token_0.is_cancelled());

    // Advance to generation 1
    let new_gen = gen.advance_generation(1001).await;
    assert_eq!(new_gen, 1);
    assert_eq!(gen.current_generation(), 1);
    assert_eq!(gen.current_network_handle(), 1001);

    // Old token must be cancelled
    assert!(token_0.is_cancelled(), "Prior generation cancellation token must be triggered");

    let token_1 = gen.cancellation_token().await;
    assert!(!token_1.is_cancelled());

    // Invalidate (fail-closed)
    let inv_gen = gen.invalidate().await;
    assert_eq!(inv_gen, 2);
    assert_eq!(gen.current_generation(), 2);
    assert_eq!(gen.current_network_handle(), 0);
    assert!(token_1.is_cancelled(), "Invalidation must trigger cancellation");
}

#[tokio::test]
async fn test_vpn_receipt_validation() {
    let gen = VpnGeneration::new();
    gen.advance_generation(500).await;

    let valid_receipt = VpnBindingReceipt {
        generation: 1,
        network_handle: 500,
        created_at_millis: 12345,
    };
    assert!(gen.is_generation_valid(&valid_receipt));

    let stale_receipt = VpnBindingReceipt {
        generation: 0,
        network_handle: 500,
        created_at_millis: 12345,
    };
    assert!(!gen.is_generation_valid(&stale_receipt));

    // After invalidation
    gen.invalidate().await;
    assert!(!gen.is_generation_valid(&valid_receipt));
}

#[tokio::test]
async fn test_mock_socket_binder() {
    let binder = MockSocketBinder::new();
    assert!(!binder.is_vpn_ready());

    // Egress should fail when no VPN is active
    let res = binder.create_bound_udp().await;
    assert!(matches!(res, Err(Error::VpnNotActive)));

    // Activate VPN
    binder.activate_vpn(888).await;
    assert!(binder.is_vpn_ready());

    let (_udp, receipt) = binder.create_bound_udp().await.expect("UDP bind should succeed");
    assert_eq!(receipt.network_handle, 888);
    assert!(binder.generation().is_generation_valid(&receipt));
}

#[test]
fn test_gateway_state_transitions() {
    assert!(!GatewayState::Stopped.is_forwarding_allowed());
    assert!(GatewayState::SafeForwarding.is_forwarding_allowed());
    assert!(!GatewayState::VpnLostFailClosed.is_forwarding_allowed());
}
