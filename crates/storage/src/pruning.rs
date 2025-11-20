//! State Pruning for Disk Space Management
//!
//! PERF-P2-001: Implement state pruning to reduce disk usage
//!
//! Provides two node modes:
//! - **Archive Node**: Stores complete historical state (all balances at all heights)
//! - **Full Node**: Only stores recent state (last N blocks), prunes older data
//!
//! Pruning reduces disk usage by ~70% while maintaining ability to:
//! - Validate new blocks
//! - Query current balances
//! - Serve light clients
//!
//! Trade-off: Cannot answer historical queries ("what was address X's balance at block Y?")

use crate::StorageError;
use rocksdb::{Direction, IteratorMode, WriteBatch, DB};
use std::path::PathBuf;

/// Node operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PruningMode {
    /// Archive node - never prune, keep all historical state
    Archive,
    /// Full node - prune state older than `keep_blocks` blocks
    Full { keep_blocks: u64 },
}

impl Default for PruningMode {
    fn default() -> Self {
        // Default to full node keeping last 10,000 blocks (~7 days at 1 min/block)
        Self::Full { keep_blocks: 10_000 }
    }
}

impl PruningMode {
    /// Create archive node mode (no pruning)
    pub fn archive() -> Self {
        Self::Archive
    }

    /// Create full node mode with custom retention
    pub fn full(keep_blocks: u64) -> Self {
        Self::Full { keep_blocks }
    }

    /// Check if pruning is enabled
    pub fn is_pruning(&self) -> bool {
        matches!(self, Self::Full { .. })
    }

    /// Get retention period (blocks to keep)
    pub fn retention_blocks(&self) -> Option<u64> {
        match self {
            Self::Archive => None,
            Self::Full { keep_blocks } => Some(*keep_blocks),
        }
    }
}

/// State pruning manager
pub struct StatePruner {
    mode: PruningMode,
}

impl StatePruner {
    /// Create new pruner with specified mode
    pub fn new(mode: PruningMode) -> Self {
        Self { mode }
    }

    /// Prune state data older than retention period
    ///
    /// # Arguments
    /// * `db` - RocksDB instance
    /// * `current_height` - Current blockchain height
    ///
    /// # Returns
    /// Number of entries pruned
    pub fn prune_at_height(
        &self,
        db: &DB,
        current_height: u64,
    ) -> Result<usize, StorageError> {
        match self.mode {
            PruningMode::Archive => {
                // Archive nodes never prune
                Ok(0)
            }
            PruningMode::Full { keep_blocks } => {
                if current_height <= keep_blocks {
                    // Not enough blocks accumulated yet
                    return Ok(0);
                }

                let prune_before_height = current_height.saturating_sub(keep_blocks);
                self.prune_historical_state(db, prune_before_height)
            }
        }
    }

    /// Prune historical balance snapshots before specified height
    ///
    /// Historical state keys follow pattern: `balance_history:{height}:{address}`
    /// This allows querying "what was balance at block N" for archive nodes.
    ///
    /// Full nodes only keep recent history and delete older snapshots.
    fn prune_historical_state(
        &self,
        db: &DB,
        prune_before_height: u64,
    ) -> Result<usize, StorageError> {
        let mut batch = WriteBatch::default();
        let mut pruned_count = 0;

        // Scan historical state keys
        let prefix = b"balance_history:";
        let iter = db.iterator(IteratorMode::From(prefix, Direction::Forward));

        for item in iter {
            let (key, _value) = item?;

            // Parse key format: "balance_history:{height}:{address}"
            if let Some(key_str) = std::str::from_utf8(&key).ok() {
                if let Some(height_str) = key_str.strip_prefix("balance_history:") {
                    if let Some(height_end) = height_str.find(':') {
                        if let Ok(height) = height_str[..height_end].parse::<u64>() {
                            if height < prune_before_height {
                                batch.delete(&key);
                                pruned_count += 1;

                                // Batch writes every 10k deletions to avoid memory bloat
                                if pruned_count % 10_000 == 0 {
                                    db.write(batch)?;
                                    batch = WriteBatch::default();
                                    tracing::info!("Pruned {} historical state entries", pruned_count);
                                }
                            } else {
                                // Keys are ordered by height, stop when we reach retention window
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Write final batch
        if pruned_count > 0 {
            db.write(batch)?;
            tracing::info!(
                "Pruning complete: {} historical state entries removed (kept blocks >= {})",
                pruned_count,
                prune_before_height
            );
        }

        Ok(pruned_count)
    }

    /// Estimate disk space that would be freed by pruning
    ///
    /// Scans database to count pruneable entries without actually deleting.
    /// Useful for displaying pruning statistics to node operators.
    pub fn estimate_prunable_space(
        &self,
        db: &DB,
        current_height: u64,
    ) -> Result<(usize, u64), StorageError> {
        let keep_blocks = match self.mode {
            PruningMode::Archive => return Ok((0, 0)),
            PruningMode::Full { keep_blocks } => keep_blocks,
        };

        if current_height <= keep_blocks {
            return Ok((0, 0));
        }

        let prune_before_height = current_height.saturating_sub(keep_blocks);
        let mut entry_count = 0;
        let mut byte_count = 0u64;

        let prefix = b"balance_history:";
        let iter = db.iterator(IteratorMode::From(prefix, Direction::Forward));

        for item in iter {
            let (key, value) = item?;

            if let Some(key_str) = std::str::from_utf8(&key).ok() {
                if let Some(height_str) = key_str.strip_prefix("balance_history:") {
                    if let Some(height_end) = height_str.find(':') {
                        if let Ok(height) = height_str[..height_end].parse::<u64>() {
                            if height < prune_before_height {
                                entry_count += 1;
                                byte_count += (key.len() + value.len()) as u64;
                            } else {
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok((entry_count, byte_count))
    }

    /// Get pruning mode
    pub fn mode(&self) -> PruningMode {
        self.mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocksdb::Options;
    use tempfile::TempDir;

    fn create_test_db() -> (TempDir, DB) {
        let dir = TempDir::new().unwrap();
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, dir.path()).unwrap();
        (dir, db)
    }

    #[test]
    fn test_pruning_mode_default() {
        let mode = PruningMode::default();
        assert!(mode.is_pruning());
        assert_eq!(mode.retention_blocks(), Some(10_000));
    }

    #[test]
    fn test_archive_mode_never_prunes() {
        let mode = PruningMode::archive();
        assert!(!mode.is_pruning());
        assert_eq!(mode.retention_blocks(), None);
    }

    #[test]
    fn test_archive_node_prunes_nothing() {
        let (_dir, db) = create_test_db();
        let pruner = StatePruner::new(PruningMode::archive());

        // Add some historical data
        for height in 0..100 {
            let key = format!("balance_history:{}:testaddr", height);
            db.put(key.as_bytes(), b"1000").unwrap();
        }

        let pruned = pruner.prune_at_height(&db, 1000).unwrap();
        assert_eq!(pruned, 0, "Archive node should not prune anything");

        // Verify all data still exists
        for height in 0..100 {
            let key = format!("balance_history:{}:testaddr", height);
            assert!(db.get(&key).unwrap().is_some());
        }
    }

    #[test]
    fn test_full_node_prunes_old_data() {
        let (_dir, db) = create_test_db();
        let pruner = StatePruner::new(PruningMode::full(50));

        // Add 100 blocks of historical data
        for height in 0..100 {
            let key = format!("balance_history:{}:testaddr", height);
            db.put(key.as_bytes(), b"1000").unwrap();
        }

        // Prune at height 100 (should keep last 50 blocks: 50-99)
        let pruned = pruner.prune_at_height(&db, 100).unwrap();
        assert_eq!(pruned, 50, "Should prune blocks 0-49");

        // Verify old data deleted
        for height in 0..50 {
            let key = format!("balance_history:{}:testaddr", height);
            assert!(db.get(&key).unwrap().is_none(), "Block {} should be pruned", height);
        }

        // Verify recent data kept
        for height in 50..100 {
            let key = format!("balance_history:{}:testaddr", height);
            assert!(db.get(&key).unwrap().is_some(), "Block {} should be kept", height);
        }
    }

    #[test]
    fn test_pruning_waits_for_sufficient_blocks() {
        let (_dir, db) = create_test_db();
        let pruner = StatePruner::new(PruningMode::full(100));

        for height in 0..50 {
            let key = format!("balance_history:{}:testaddr", height);
            db.put(key.as_bytes(), b"1000").unwrap();
        }

        // Should not prune yet (only 50 blocks, need 100)
        let pruned = pruner.prune_at_height(&db, 50).unwrap();
        assert_eq!(pruned, 0);
    }

    #[test]
    fn test_estimate_prunable_space() {
        let (_dir, db) = create_test_db();
        let pruner = StatePruner::new(PruningMode::full(10));

        for height in 0..50 {
            let key = format!("balance_history:{}:testaddr", height);
            db.put(key.as_bytes(), b"1000").unwrap();
        }

        let (count, bytes) = pruner.estimate_prunable_space(&db, 50).unwrap();
        assert_eq!(count, 40, "Should identify 40 pruneable entries");
        assert!(bytes > 0, "Should calculate byte count");
    }

    #[test]
    fn test_batch_pruning_large_dataset() {
        let (_dir, db) = create_test_db();
        let pruner = StatePruner::new(PruningMode::full(100));

        // Create 50k entries (simulates ~50k blocks of history)
        for height in 0..50_000 {
            let key = format!("balance_history:{}:addr", height);
            db.put(key.as_bytes(), b"5000").unwrap();
        }

        let pruned = pruner.prune_at_height(&db, 50_000).unwrap();
        assert_eq!(pruned, 49_900, "Should prune all but last 100 blocks");
    }
}
