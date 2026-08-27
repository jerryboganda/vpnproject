//! Metrics Snapshot Representation

use serde::{Deserialize, Serialize};

/// Serializable telemetry and throughput snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub uptime_seconds: u64,
    pub bytes_rx: u64,
    pub bytes_tx: u64,
    pub active_tcp_streams: u64,
    pub total_tcp_connections: u64,
    pub active_udp_mappings: u64,
    pub total_udp_packets: u64,
    pub vpn_drops_count: u64,
    pub auth_failures_count: u64,
}
