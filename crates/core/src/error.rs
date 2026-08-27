//! VPNBridge Error Model

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("VPN not available or not active")]
    VpnNotActive,

    #[error("VPN generation mismatch: expected {expected}, actual {actual}")]
    GenerationMismatch { expected: u64, actual: u64 },

    #[error("VPN connection lost; fail-closed triggered")]
    VpnLostFailClosed,

    #[error("Socket binding to VPN network {network_handle} failed: {reason}")]
    SocketBindFailed {
        network_handle: u64,
        reason: String,
    },

    #[error("Client authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Unauthorized client: session {0} not paired")]
    Unauthorized(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("SOCKS5 parser error: {0}")]
    Socks5Error(String),

    #[error("Capacity limit exceeded: {0}")]
    CapacityExceeded(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("I/O error: {0}")]
    Io(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Windows route error: {0}")]
    WindowsRouteError(String),

    #[error("Windows WFP firewall error: {0}")]
    WindowsWfpError(String),

    #[error("Wintun error: {0}")]
    WintunError(String),

    #[error("Internal service error: {0}")]
    Internal(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err.to_string())
    }
}
