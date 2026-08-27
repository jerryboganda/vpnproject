//! VPNBridge Pairing & Control Protocol
//!
//! Handles control-plane framing, handshakes, authenticated pairing,
//! heartbeats, and constant-time token verification.

pub mod auth;
pub mod codec;
pub mod messages;
pub mod pairing;

pub use auth::{compute_challenge_proof, generate_secure_token, verify_auth_token};
pub use codec::ProtocolCodec;
pub use messages::{ControlMessage, HandshakeRequest, HandshakeResponse, HeartbeatAck, PairingAuth};
pub use pairing::PairingPayload;
