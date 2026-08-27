//! Atomic Performance & Connection Counters

use crate::snapshot::MetricsSnapshot;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Central metrics collector with atomic thread-safe updates.
#[derive(Clone, Debug)]
pub struct MetricsTracker {
    inner: Arc<TrackerInner>,
}

#[derive(Debug)]
struct TrackerInner {
    start_time: Instant,
    bytes_rx: AtomicU64,
    bytes_tx: AtomicU64,
    active_tcp_streams: AtomicU64,
    total_tcp_connections: AtomicU64,
    active_udp_mappings: AtomicU64,
    total_udp_packets: AtomicU64,
    vpn_drops_count: AtomicU64,
    auth_failures_count: AtomicU64,
}

impl MetricsTracker {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(TrackerInner {
                start_time: Instant::now(),
                bytes_rx: AtomicU64::new(0),
                bytes_tx: AtomicU64::new(0),
                active_tcp_streams: AtomicU64::new(0),
                total_tcp_connections: AtomicU64::new(0),
                active_udp_mappings: AtomicU64::new(0),
                total_udp_packets: AtomicU64::new(0),
                vpn_drops_count: AtomicU64::new(0),
                auth_failures_count: AtomicU64::new(0),
            }),
        }
    }

    #[inline]
    pub fn record_rx(&self, bytes: u64) {
        self.inner.bytes_rx.fetch_add(bytes, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_tx(&self, bytes: u64) {
        self.inner.bytes_tx.fetch_add(bytes, Ordering::Relaxed);
    }

    #[inline]
    pub fn inc_tcp_stream(&self) {
        self.inner.active_tcp_streams.fetch_add(1, Ordering::Relaxed);
        self.inner.total_tcp_connections.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn dec_tcp_stream(&self) {
        self.inner.active_tcp_streams.fetch_sub(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn inc_udp_mapping(&self) {
        self.inner.active_udp_mappings.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn dec_udp_mapping(&self) {
        self.inner.active_udp_mappings.fetch_sub(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_udp_packet(&self) {
        self.inner.total_udp_packets.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub fn record_vpn_drop(&self) {
        self.inner.vpn_drops_count.fetch_add(1, Ordering::SeqCst);
    }

    #[inline]
    pub fn record_auth_failure(&self) {
        self.inner.auth_failures_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let uptime_secs = self.inner.start_time.elapsed().as_secs();
        MetricsSnapshot {
            uptime_seconds: uptime_secs,
            bytes_rx: self.inner.bytes_rx.load(Ordering::Relaxed),
            bytes_tx: self.inner.bytes_tx.load(Ordering::Relaxed),
            active_tcp_streams: self.inner.active_tcp_streams.load(Ordering::Relaxed),
            total_tcp_connections: self.inner.total_tcp_connections.load(Ordering::Relaxed),
            active_udp_mappings: self.inner.active_udp_mappings.load(Ordering::Relaxed),
            total_udp_packets: self.inner.total_udp_packets.load(Ordering::Relaxed),
            vpn_drops_count: self.inner.vpn_drops_count.load(Ordering::SeqCst),
            auth_failures_count: self.inner.auth_failures_count.load(Ordering::Relaxed),
        }
    }
}

impl Default for MetricsTracker {
    fn default() -> Self {
        Self::new()
    }
}
