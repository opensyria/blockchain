//! Orphan Transaction Pool
//!
//! PERF-P2-003: Add orphan transaction handling
//!
//! Manages transactions that reference parent transactions not yet in the blockchain.
//! Common scenarios:
//! - Transaction arrives before its dependencies (network propagation delay)
//! - User broadcasts transaction chain (tx1 → tx2 → tx3) but they arrive out of order
//!
//! Orphan pool temporarily holds these until parent transactions confirm, then
//! automatically moves them to main mempool for inclusion in blocks.
//!
//! Prevents:
//! - Legitimate transactions being rejected due to network timing
//! - Need for users to manually re-broadcast transactions
//! - Poor UX from transaction chains failing

use crate::{Mempool, MempoolError, Result};
use opensyria_core::Transaction;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};

/// Maximum number of orphan transactions to hold
const MAX_ORPHAN_TRANSACTIONS: usize = 1000;

/// Maximum time to keep orphan transaction (seconds)
const MAX_ORPHAN_AGE_SECS: u64 = 600; // 10 minutes

/// Orphan transaction pool for transactions with missing parents
pub struct OrphanPool {
    /// Orphan transactions by hash
    orphans: HashMap<[u8; 32], Transaction>,

    /// Orphan transactions by parent hash they're waiting for
    /// Key: parent tx hash, Value: set of orphan tx hashes waiting for this parent
    by_parent: HashMap<[u8; 32], HashSet<[u8; 32]>>,

    /// Orphan insertion timestamps
    timestamps: HashMap<[u8; 32], u64>,
}

impl OrphanPool {
    /// Create new orphan pool
    pub fn new() -> Self {
        Self {
            orphans: HashMap::new(),
            by_parent: HashMap::new(),
            timestamps: HashMap::new(),
        }
    }

    /// Add transaction to orphan pool
    ///
    /// # Arguments
    /// * `tx` - Transaction to add
    /// * `missing_parent` - Hash of the parent transaction this orphan depends on
    ///
    /// # Returns
    /// Ok if added, Err if pool is full
    pub fn add_orphan(&mut self, tx: Transaction, missing_parent: [u8; 32]) -> Result<()> {
        let tx_hash = tx.hash();

        // Check if already in orphan pool
        if self.orphans.contains_key(&tx_hash) {
            return Ok(());
        }

        // Enforce size limit
        if self.orphans.len() >= MAX_ORPHAN_TRANSACTIONS {
            // Evict oldest orphan
            self.evict_oldest();
        }

        info!(
            "Adding orphan transaction {} (waiting for parent {})",
            hex::encode(&tx_hash[..8]),
            hex::encode(&missing_parent[..8])
        );

        // Add to parent index
        self.by_parent
            .entry(missing_parent)
            .or_default()
            .insert(tx_hash);

        // Add timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.timestamps.insert(tx_hash, now);

        // Add orphan
        self.orphans.insert(tx_hash, tx);

        debug!("Orphan pool size: {}", self.orphans.len());

        Ok(())
    }

    /// Process confirmed transaction and promote any dependent orphans to main mempool
    ///
    /// When a transaction confirms (included in block), check if any orphans were
    /// waiting for it. If so, try to add them to main mempool.
    ///
    /// # Arguments
    /// * `tx_hash` - Hash of the confirmed parent transaction
    /// * `mempool` - Main mempool to promote orphans to
    ///
    /// # Returns
    /// Number of orphans promoted
    pub async fn process_parent_confirmation(
        &mut self,
        tx_hash: &[u8; 32],
        mempool: &mut Mempool,
    ) -> usize {
        let mut promoted_count = 0;

        // Get orphans waiting for this parent
        if let Some(waiting_orphans) = self.by_parent.remove(tx_hash) {
            for orphan_hash in waiting_orphans {
                if let Some(orphan_tx) = self.orphans.remove(&orphan_hash) {
                    // Try to add to main mempool
                    match mempool.add_transaction(orphan_tx).await {
                        Ok(_) => {
                            info!(
                                "Promoted orphan {} to mempool (parent {} confirmed)",
                                hex::encode(&orphan_hash[..8]),
                                hex::encode(&tx_hash[..8])
                            );
                            promoted_count += 1;

                            // Recursively check if this orphan's confirmation enables other orphans
                            promoted_count +=
                                self.process_parent_confirmation(&orphan_hash, mempool).await;
                        }
                        Err(e) => {
                            warn!(
                                "Failed to promote orphan {} to mempool: {}",
                                hex::encode(&orphan_hash[..8]),
                                e
                            );
                            // Don't re-add to orphan pool, let it expire
                        }
                    }

                    // Remove timestamp
                    self.timestamps.remove(&orphan_hash);
                }
            }
        }

        if promoted_count > 0 {
            info!("Promoted {} orphan transactions to mempool", promoted_count);
        }

        promoted_count
    }

    /// Remove expired orphan transactions
    ///
    /// Returns number of expired orphans removed
    pub fn remove_expired(&mut self) -> usize {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut expired = Vec::new();

        for (hash, timestamp) in &self.timestamps {
            if now - timestamp > MAX_ORPHAN_AGE_SECS {
                expired.push(*hash);
            }
        }

        for hash in &expired {
            self.remove_orphan(hash);
        }

        if !expired.is_empty() {
            warn!("Removed {} expired orphan transactions", expired.len());
        }

        expired.len()
    }

    /// Remove orphan transaction
    fn remove_orphan(&mut self, tx_hash: &[u8; 32]) {
        if let Some(tx) = self.orphans.remove(tx_hash) {
            // Remove from parent index - need to find which parent(s) reference this orphan
            for (_, orphan_set) in self.by_parent.iter_mut() {
                orphan_set.remove(tx_hash);
            }

            // Clean up empty parent entries
            self.by_parent.retain(|_, orphans| !orphans.is_empty());

            self.timestamps.remove(tx_hash);

            debug!("Removed orphan transaction: {}", hex::encode(tx_hash));
        }
    }

    /// Evict oldest orphan transaction
    fn evict_oldest(&mut self) {
        if let Some((&oldest_hash, _)) = self.timestamps.iter().min_by_key(|(_, &ts)| ts) {
            warn!(
                "Evicting oldest orphan transaction: {}",
                hex::encode(&oldest_hash[..8])
            );
            self.remove_orphan(&oldest_hash);
        }
    }

    /// Get orphan pool size
    pub fn size(&self) -> usize {
        self.orphans.len()
    }

    /// Check if orphan pool is empty
    pub fn is_empty(&self) -> bool {
        self.orphans.is_empty()
    }

    /// Get orphan transaction by hash
    pub fn get_orphan(&self, hash: &[u8; 32]) -> Option<&Transaction> {
        self.orphans.get(hash)
    }

    /// Clear all orphan transactions
    pub fn clear(&mut self) {
        info!("Clearing orphan pool ({} transactions)", self.orphans.len());
        self.orphans.clear();
        self.by_parent.clear();
        self.timestamps.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> OrphanPoolStats {
        OrphanPoolStats {
            total_orphans: self.orphans.len(),
            unique_parents: self.by_parent.len(),
        }
    }
}

impl Default for OrphanPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Orphan pool statistics
#[derive(Debug, Clone)]
pub struct OrphanPoolStats {
    /// Total number of orphan transactions
    pub total_orphans: usize,
    /// Number of unique parent transactions being waited for
    pub unique_parents: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::crypto::KeyPair;
    use opensyria_storage::StateStorage;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    fn create_test_tx(
        sender: &KeyPair,
        receiver: &KeyPair,
        amount: u64,
        fee: u64,
        nonce: u64,
    ) -> Transaction {
        let mut tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            amount,
            fee,
            nonce,
        );
        let msg = tx.signing_hash();
        tx.signature = sender.sign(&msg);
        tx
    }

    #[tokio::test]
    async fn test_add_orphan() {
        let mut orphan_pool = OrphanPool::new();

        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();
        let tx = create_test_tx(&sender, &receiver, 1000, 100, 0);
        let parent_hash = [1u8; 32];

        assert!(orphan_pool.add_orphan(tx, parent_hash).is_ok());
        assert_eq!(orphan_pool.size(), 1);
    }

    #[tokio::test]
    async fn test_orphan_promotion() {
        let temp_dir = std::env::temp_dir()
            .join(format!("orphan_promotion_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let state = StateStorage::open(temp_dir.clone()).unwrap();
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        // Set up sender balance
        state.set_balance(&sender.public_key(), 10_000_000).unwrap();
        state.set_nonce(&sender.public_key(), 0).unwrap();

        let state = Arc::new(RwLock::new(state));
        let config = crate::MempoolConfig::default();
        let mut mempool = Mempool::new(config, state);
        let mut orphan_pool = OrphanPool::new();

        // Create parent transaction
        let parent_tx = create_test_tx(&sender, &receiver, 1000, 100, 0);
        let parent_hash = parent_tx.hash();

        // Create orphan (depends on parent)
        let orphan_tx = create_test_tx(&sender, &receiver, 500, 100, 1);
        orphan_pool.add_orphan(orphan_tx, parent_hash).unwrap();

        assert_eq!(orphan_pool.size(), 1);
        assert_eq!(mempool.size(), 0);

        // Add parent to mempool (simulating confirmation)
        mempool.add_transaction(parent_tx).await.unwrap();

        // Process parent confirmation - should promote orphan
        let promoted = orphan_pool
            .process_parent_confirmation(&parent_hash, &mut mempool)
            .await;

        assert_eq!(promoted, 1);
        assert_eq!(orphan_pool.size(), 0);
        assert_eq!(mempool.size(), 2); // parent + promoted orphan

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_orphan_chain_promotion() {
        let temp_dir = std::env::temp_dir()
            .join(format!("orphan_chain_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);

        let state = StateStorage::open(temp_dir.clone()).unwrap();
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        state.set_balance(&sender.public_key(), 10_000_000).unwrap();
        state.set_nonce(&sender.public_key(), 0).unwrap();

        let state = Arc::new(RwLock::new(state));
        let config = crate::MempoolConfig::default();
        let mut mempool = Mempool::new(config, state);
        let mut orphan_pool = OrphanPool::new();

        // Create chain: tx0 → tx1 → tx2
        let tx0 = create_test_tx(&sender, &receiver, 1000, 100, 0);
        let tx1 = create_test_tx(&sender, &receiver, 800, 100, 1);
        let tx2 = create_test_tx(&sender, &receiver, 600, 100, 2);

        let hash0 = tx0.hash();
        let hash1 = tx1.hash();

        // Add tx1 and tx2 as orphans (waiting for tx0 and tx1 respectively)
        orphan_pool.add_orphan(tx1, hash0).unwrap();
        orphan_pool.add_orphan(tx2, hash1).unwrap();

        assert_eq!(orphan_pool.size(), 2);

        // Add tx0 to mempool
        mempool.add_transaction(tx0).await.unwrap();

        // Process confirmation - should promote both tx1 and tx2 recursively
        let promoted = orphan_pool.process_parent_confirmation(&hash0, &mut mempool).await;

        assert_eq!(promoted, 2, "Should promote tx1 and tx2");
        assert_eq!(orphan_pool.size(), 0);
        assert_eq!(mempool.size(), 3); // tx0 + tx1 + tx2

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_orphan_expiration() {
        let mut orphan_pool = OrphanPool::new();

        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();
        let tx = create_test_tx(&sender, &receiver, 1000, 100, 0);
        let parent_hash = [1u8; 32];
        let tx_hash = tx.hash();

        orphan_pool.add_orphan(tx, parent_hash).unwrap();

        // Manually set old timestamp
        let old_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - MAX_ORPHAN_AGE_SECS
            - 1;
        orphan_pool.timestamps.insert(tx_hash, old_timestamp);

        assert_eq!(orphan_pool.size(), 1);

        let expired = orphan_pool.remove_expired();
        assert_eq!(expired, 1);
        assert_eq!(orphan_pool.size(), 0);
    }

    #[tokio::test]
    async fn test_orphan_pool_size_limit() {
        let mut orphan_pool = OrphanPool::new();
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        // Add MAX_ORPHAN_TRANSACTIONS + 10 orphans
        for i in 0..(MAX_ORPHAN_TRANSACTIONS + 10) {
            let tx = create_test_tx(&sender, &receiver, 1000, 100, i as u64);
            let parent_hash = [i as u8; 32];
            orphan_pool.add_orphan(tx, parent_hash).unwrap();
        }

        // Should be capped at MAX_ORPHAN_TRANSACTIONS
        assert_eq!(orphan_pool.size(), MAX_ORPHAN_TRANSACTIONS);
    }
}
