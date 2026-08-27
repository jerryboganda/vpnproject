//! VPNBridge Core Library
//!
//! Foundational models, state machines, generational VPN tracking,
//! traits, error handling, and configuration for VPNBridge.

pub mod config;
pub mod error;
pub mod state;
pub mod traits;

pub use config::GatewayConfig;
pub use error::{Error, Result};
pub use state::{
    GatewayState, SessionId, VpnBindingReceipt, VpnGeneration, VpnNetworkHandle, VpnStateSnapshot,
};
pub use traits::{MockSocketBinder, ProtectedSocketBinder};
