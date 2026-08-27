//! Gateway Configuration & Limit Policies

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

/// Configuration for the VPNBridge SOCKS5 and control plane gateway.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Local address on the Local-Only Hotspot interface to bind the gateway listener.
    pub listen_addr: SocketAddr,

    /// Maximum concurrent client TCP streams.
    pub max_tcp_streams: usize,

    /// Maximum concurrent UDP associations.
    pub max_udp_mappings: usize,

    /// Maximum idle timeout for TCP streams before auto-reap.
    #[serde(with = "duration_millis")]
    pub tcp_idle_timeout: Duration,

    /// Maximum idle timeout for UDP NAT mappings.
    #[serde(with = "duration_millis")]
    pub udp_idle_timeout: Duration,

    /// Secret token used for SOCKS5 and control channel authentication.
    pub auth_token: String,

    /// Require authenticated client handshake before forwarding.
    pub require_auth: bool,

    /// Channel buffer capacity for backpressure protection.
    pub channel_buffer_size: usize,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:10808".parse().unwrap(),
            max_tcp_streams: 1024,
            max_udp_mappings: 4096,
            tcp_idle_timeout: Duration::from_secs(300),
            udp_idle_timeout: Duration::from_secs(60),
            auth_token: "vpnbridge-default-secret".to_string(),
            require_auth: true,
            channel_buffer_size: 128,
        }
    }
}

mod duration_millis {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_millis() as u64)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}
