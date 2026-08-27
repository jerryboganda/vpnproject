//! Control & Handshake Messages

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use vpnbridge_core::state::GatewayState;

pub const CURRENT_PROTOCOL_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ControlMessage {
    HandshakeReq(HandshakeRequest),
    HandshakeResp(HandshakeResponse),
    PairingAuthReq(PairingAuth),
    PairingAuthResp(PairingResult),
    HeartbeatReq(Heartbeat),
    HeartbeatResp(HeartbeatAck),
    StateUpdate(VpnStateNotification),
    Disconnect(DisconnectNotice),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandshakeRequest {
    pub protocol_version: u32,
    pub client_id: String,
    pub device_name: String,
    pub nonce: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandshakeResponse {
    pub server_version: u32,
    pub session_id: String,
    pub challenge: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingAuth {
    pub session_id: String,
    pub hmac_proof: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub gateway_state: GatewayState,
    pub active_generation: u64,
    pub socks5_port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Heartbeat {
    pub timestamp_millis: u64,
    pub session_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeartbeatAck {
    pub timestamp_millis: u64,
    pub is_vpn_protected: bool,
    pub generation: u64,
    pub state: GatewayState,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VpnStateNotification {
    pub state: GatewayState,
    pub generation: u64,
    pub is_protected: bool,
    pub dns_servers: Vec<IpAddr>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisconnectNotice {
    pub reason: String,
}
