//! Leak Detection Verification Harness

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Leak detector that tracks and asserts that zero packets escape to unmanaged / non-VPN interfaces.
#[derive(Clone, Debug)]
pub struct LeakDetector {
    leaked_tcp_bytes: Arc<AtomicU64>,
    leaked_udp_packets: Arc<AtomicU64>,
}

impl LeakDetector {
    pub fn new() -> Self {
        Self {
            leaked_tcp_bytes: Arc::new(AtomicU64::new(0)),
            leaked_udp_packets: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_leak_tcp(&self, bytes: u64) {
        self.leaked_tcp_bytes.fetch_add(bytes, Ordering::SeqCst);
    }

    pub fn record_leak_udp(&self) {
        self.leaked_udp_packets.fetch_add(1, Ordering::SeqCst);
    }

    pub fn assert_no_leaks(&self) {
        let tcp = self.leaked_tcp_bytes.load(Ordering::SeqCst);
        let udp = self.leaked_udp_packets.load(Ordering::SeqCst);
        assert_eq!(
            tcp, 0,
            "FAIL-CLOSED INVARIANT VIOLATION: {tcp} raw TCP bytes leaked!"
        );
        assert_eq!(
            udp, 0,
            "FAIL-CLOSED INVARIANT VIOLATION: {udp} raw UDP packets leaked!"
        );
    }
}

impl Default for LeakDetector {
    fn default() -> Self {
        Self::new()
    }
}
