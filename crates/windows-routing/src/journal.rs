//! Transactional Network Recovery Journal

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use vpnbridge_core::error::{Error, Result};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RouteSnapshot {
    pub original_gateway: Option<IpAddr>,
    pub original_dns_servers: Vec<IpAddr>,
    pub original_interface_index: u32,
    pub original_interface_metric: u32,
    pub hotspot_gateway_ip: IpAddr,
    pub tun_interface_index: u32,
    pub created_at_timestamp: u64,
}

pub struct RecoveryJournal {
    journal_path: PathBuf,
}

impl RecoveryJournal {
    pub fn new(journal_dir: impl AsRef<Path>) -> Self {
        Self {
            journal_path: journal_dir.as_ref().join("vpnbridge_recovery_journal.json"),
        }
    }

    /// Write active network state to disk before executing any route or firewall modifications.
    pub async fn write_snapshot(&self, snapshot: &RouteSnapshot) -> Result<()> {
        let data = serde_json::to_vec_pretty(snapshot)
            .map_err(|e| Error::Internal(format!("Failed to serialize recovery journal: {e}")))?;

        if let Some(parent) = self.journal_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                Error::Io(format!("Failed to create journal directory: {e}"))
            })?;
        }

        tokio::fs::write(&self.journal_path, data)
            .await
            .map_err(|e| Error::Io(format!("Failed to write recovery journal to disk: {e}")))?;

        tracing::info!(path = ?self.journal_path, "Recovery journal safely written to disk");
        Ok(())
    }

    /// Read journal from disk to recover from unexpected process crash or reboot.
    pub async fn read_snapshot(&self) -> Result<Option<RouteSnapshot>> {
        if !tokio::fs::try_exists(&self.journal_path).await.unwrap_or(false) {
            return Ok(None);
        }

        let data = tokio::fs::read(&self.journal_path)
            .await
            .map_err(|e| Error::Io(format!("Failed to read recovery journal: {e}")))?;

        let snapshot: RouteSnapshot = serde_json::from_slice(&data)
            .map_err(|e| Error::Internal(format!("Failed to deserialize recovery journal: {e}")))?;

        Ok(Some(snapshot))
    }

    /// Remove journal after clean disconnect and full route restoration.
    pub async fn clear_journal(&self) -> Result<()> {
        if tokio::fs::try_exists(&self.journal_path).await.unwrap_or(false) {
            tokio::fs::remove_file(&self.journal_path)
                .await
                .map_err(|e| Error::Io(format!("Failed to delete recovery journal: {e}")))?;
            tracing::info!("Recovery journal cleared");
        }
        Ok(())
    }
}
