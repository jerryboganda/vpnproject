//! Wintun Adapter & Session Traits

use async_trait::async_trait;
use bytes::Bytes;
use tokio::sync::mpsc;
use vpnbridge_core::error::{Error, Result};

#[async_trait]
pub trait TunAdapter: Send + Sync {
    fn name(&self) -> &str;
    async fn start_session(&self, ring_capacity: u32) -> Result<Box<dyn TunSession>>;
}

#[async_trait]
pub trait TunSession: Send + Sync {
    async fn read_packet(&mut self) -> Result<Bytes>;
    async fn write_packet(&mut self, packet: &[u8]) -> Result<()>;
    fn shutdown(&mut self);
}

/// Mock TUN Adapter for testing without physical Wintun driver loaded.
pub struct MockTunAdapter {
    name: String,
}

impl MockTunAdapter {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

#[async_trait]
impl TunAdapter for MockTunAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    async fn start_session(&self, _ring_capacity: u32) -> Result<Box<dyn TunSession>> {
        let (tx_in, rx_in) = mpsc::channel(128);
        let (tx_out, rx_out) = mpsc::channel(128);

        Ok(Box::new(MockTunSession {
            _tx_in: tx_in,
            rx_in,
            tx_out,
            _rx_out: rx_out,
        }))
    }
}

pub struct MockTunSession {
    _tx_in: mpsc::Sender<Bytes>,
    rx_in: mpsc::Receiver<Bytes>,
    tx_out: mpsc::Sender<Bytes>,
    _rx_out: mpsc::Receiver<Bytes>,
}

#[async_trait]
impl TunSession for MockTunSession {
    async fn read_packet(&mut self) -> Result<Bytes> {
        self.rx_in
            .recv()
            .await
            .ok_or_else(|| Error::WintunError("Mock session closed".to_string()))
    }

    async fn write_packet(&mut self, packet: &[u8]) -> Result<()> {
        self.tx_out
            .send(Bytes::copy_from_slice(packet))
            .await
            .map_err(|e| Error::WintunError(format!("Send failed: {e}")))
    }

    fn shutdown(&mut self) {
        // Drop channels
    }
}
