use opensyria_core::Transaction;
use opensyria_storage::StateStorage;
use crate::{MempoolError, Result, TransactionValidator};
use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

/// Transaction status in mempool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Transaction is pending
    Pending,
    /// Transaction is included in a block
    Confirmed,
    /// Transaction was rejected
    Rejected,
}

/// Mempool configuration
#[derive(Debug, Clone)]
pub struct MempoolConfig {
    /// Maximum number of transactions in mempool
    pub max_size: usize,
    
    /// Minimum transaction fee (in smallest units)
    pub min_fee: u64,
    
    /// Maximum transaction age in seconds
    pub max_age_secs: u64,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10_000,
            min_fee: 1_000, // 0.001 SYL
            max_age_secs: 3600, // 1 hour
        }
    }
}

/// Transaction memory pool
pub struct Mempool {
    /// Configuration
    config: MempoolConfig,
    
    /// Pending transactions by hash
    transactions: HashMap<[u8; 32], Transaction>,
    
    /// Transactions ordered by fee (descending) for priority selection
    /// Key: (fee, hash), Value: ()
    priority_queue: BTreeMap<(u64, [u8; 32]), ()>,
    
    /// Transactions by sender for nonce tracking
    /// Key: sender public key, Value: list of (nonce, tx_hash)
    by_sender: HashMap<[u8; 32], Vec<(u64, [u8; 32])>>,
    
    /// Transaction validator
    validator: Arc<TransactionValidator>,
    
    /// Transaction insertion timestamps
    timestamps: HashMap<[u8; 32], u64>,
}

impl Mempool {
    /// Create a new mempool
    pub fn new(config: MempoolConfig, state: Arc<RwLock<StateStorage>>) -> Self {
        let validator = Arc::new(TransactionValidator::new(state, config.min_fee));
        
        Self {
            config,
            transactions: HashMap::new(),
            priority_queue: BTreeMap::new(),
            by_sender: HashMap::new(),
            validator,
            timestamps: HashMap::new(),
        }
    }

    /// Add a transaction to the mempool
    pub async fn add_transaction(&mut self, tx: Transaction) -> Result<()> {
        let tx_hash = tx.hash();
        
        // Check if already in mempool
        if self.transactions.contains_key(&tx_hash) {
            return Err(MempoolError::DuplicateTransaction(hex::encode(tx_hash)));
        }
        
        // Check mempool size
        if self.transactions.len() >= self.config.max_size {
            return Err(MempoolError::MempoolFull {
                max: self.config.max_size,
                current: self.transactions.len(),
            });
        }
        
        // Validate transaction
        self.validator.validate(&tx).await?;
        
        info!(
            "Adding transaction to mempool: {} SYL from {}... to {}...",
            tx.amount as f64 / 1_000_000.0,
            hex::encode(&tx.from.0[..8]),
            hex::encode(&tx.to.0[..8])
        );
        
        // Add to priority queue (higher fee = higher priority)
        // Use negative fee for descending order
        let priority_key = (u64::MAX - tx.fee, tx_hash);
        self.priority_queue.insert(priority_key, ());
        
        // Add to sender index
        let sender_key = tx.from.0;
        self.by_sender
            .entry(sender_key)
            .or_insert_with(Vec::new)
            .push((tx.nonce, tx_hash));
        
        // Add timestamp
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.timestamps.insert(tx_hash, now);
        
        // Add transaction
        self.transactions.insert(tx_hash, tx);
        
        debug!("Mempool size: {}", self.transactions.len());
        
        Ok(())
    }

    /// Remove a transaction from the mempool
    pub fn remove_transaction(&mut self, tx_hash: &[u8; 32]) -> Option<Transaction> {
        if let Some(tx) = self.transactions.remove(tx_hash) {
            // Remove from priority queue
            let priority_key = (u64::MAX - tx.fee, *tx_hash);
            self.priority_queue.remove(&priority_key);
            
            // Remove from sender index
            if let Some(txs) = self.by_sender.get_mut(&tx.from.0) {
                txs.retain(|(_, hash)| hash != tx_hash);
                if txs.is_empty() {
                    self.by_sender.remove(&tx.from.0);
                }
            }
            
            // Remove timestamp
            self.timestamps.remove(tx_hash);
            
            info!("Removed transaction from mempool: {}", hex::encode(tx_hash));
            
            Some(tx)
        } else {
            None
        }
    }

    /// Get transactions ordered by priority (highest fee first)
    pub fn get_priority_transactions(&self, max_count: usize) -> Vec<Transaction> {
        self.priority_queue
            .keys()
            .take(max_count)
            .filter_map(|(_, hash)| self.transactions.get(hash).cloned())
            .collect()
    }

    /// Get all pending transactions
    pub fn get_all_transactions(&self) -> Vec<Transaction> {
        self.transactions.values().cloned().collect()
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, hash: &[u8; 32]) -> Option<&Transaction> {
        self.transactions.get(hash)
    }

    /// Get pending transactions for a sender
    pub fn get_sender_transactions(&self, sender: &[u8; 32]) -> Vec<Transaction> {
        if let Some(txs) = self.by_sender.get(sender) {
            txs.iter()
                .filter_map(|(_, hash)| self.transactions.get(hash).cloned())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get mempool size
    pub fn size(&self) -> usize {
        self.transactions.len()
    }

    /// Check if mempool is empty
    pub fn is_empty(&self) -> bool {
        self.transactions.is_empty()
    }

    /// Remove expired transactions
    pub fn remove_expired(&mut self) -> Vec<[u8; 32]> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut expired = Vec::new();
        
        for (hash, timestamp) in &self.timestamps {
            if now - timestamp > self.config.max_age_secs {
                expired.push(*hash);
            }
        }
        
        for hash in &expired {
            self.remove_transaction(hash);
        }
        
        if !expired.is_empty() {
            warn!("Removed {} expired transactions", expired.len());
        }
        
        expired
    }

    /// Clear all transactions (useful after block confirmation)
    pub fn clear(&mut self) {
        info!("Clearing mempool ({} transactions)", self.transactions.len());
        self.transactions.clear();
        self.priority_queue.clear();
        self.by_sender.clear();
        self.timestamps.clear();
    }

    /// Remove transactions that are included in a block
    pub fn remove_confirmed_transactions(&mut self, transactions: &[Transaction]) {
        for tx in transactions {
            let hash = tx.hash();
            self.remove_transaction(&hash);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::crypto::KeyPair;

    async fn create_test_mempool() -> Mempool {
        let temp_dir = std::env::temp_dir().join("mempool_test");
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        let mut state = StateStorage::open(temp_dir.clone()).unwrap();
        
        // Create test accounts with balances
        for _ in 0..5 {
            let keypair = KeyPair::generate();
            state.set_balance(&keypair.public_key(), 10_000_000).unwrap();
            state.set_nonce(&keypair.public_key(), 0).unwrap();
        }
        
        let state = Arc::new(RwLock::new(state));
        let config = MempoolConfig::default();
        
        Mempool::new(config, state)
    }

    #[tokio::test]
    async fn test_add_transaction() {
        let temp_dir = std::env::temp_dir().join(format!("mempool_add_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        let mut state = StateStorage::open(temp_dir.clone()).unwrap();
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();
        
        // Set up sender balance
        state.set_balance(&sender.public_key(), 1_000_000).unwrap();
        state.set_nonce(&sender.public_key(), 0).unwrap();
        
        let state = Arc::new(RwLock::new(state));
        let config = MempoolConfig::default();
        let mut mempool = Mempool::new(config, state);
        
        let mut tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            100_000,
            1_000,
            0,
        );
        let msg = tx.signing_hash();
        let sig = sender.sign(&msg);
        tx.signature = sig;
        
        assert!(mempool.add_transaction(tx).await.is_ok());
        assert_eq!(mempool.size(), 1);
        
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_priority_queue() {
        let temp_dir = std::env::temp_dir().join(format!("mempool_priority_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        let mut state = StateStorage::open(temp_dir.clone()).unwrap();
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();
        
        state.set_balance(&sender.public_key(), 10_000_000).unwrap();
        state.set_nonce(&sender.public_key(), 0).unwrap();
        
        let state = Arc::new(RwLock::new(state));
        let config = MempoolConfig::default();
        let mut mempool = Mempool::new(config, state);
        
        // Add transactions with different fees (all above minimum of 1000)
        let mut tx1 = Transaction::new(sender.public_key(), receiver.public_key(), 1000, 1000, 0);
        let msg1 = tx1.signing_hash();
        tx1.signature = sender.sign(&msg1);
        
        let mut tx2 = Transaction::new(sender.public_key(), receiver.public_key(), 1000, 5000, 1);
        let msg2 = tx2.signing_hash();
        tx2.signature = sender.sign(&msg2);
        
        let mut tx3 = Transaction::new(sender.public_key(), receiver.public_key(), 1000, 2000, 2);
        let msg3 = tx3.signing_hash();
        tx3.signature = sender.sign(&msg3);
        
        mempool.add_transaction(tx1).await.expect("tx1 failed");
        mempool.add_transaction(tx2.clone()).await.expect("tx2 failed");
        mempool.add_transaction(tx3).await.expect("tx3 failed");
        
        // Get priority transactions - highest fee first
        let priority_txs = mempool.get_priority_transactions(3);
        assert_eq!(priority_txs.len(), 3, "Expected 3 transactions, got {}", priority_txs.len());
        assert_eq!(priority_txs[0].fee, 5000); // tx2 has highest fee
        
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_remove_transaction() {
        let temp_dir = std::env::temp_dir().join(format!("mempool_remove_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        let mut state = StateStorage::open(temp_dir.clone()).unwrap();
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();
        
        state.set_balance(&sender.public_key(), 1_000_000).unwrap();
        state.set_nonce(&sender.public_key(), 0).unwrap();
        
        let state = Arc::new(RwLock::new(state));
        let config = MempoolConfig::default();
        let mut mempool = Mempool::new(config, state);
        
        let mut tx = Transaction::new(sender.public_key(), receiver.public_key(), 100_000, 1_000, 0);
        let msg = tx.signing_hash();
        let sig = sender.sign(&msg);
        tx.signature = sig;
        let tx_hash = tx.hash();
        
        mempool.add_transaction(tx).await.ok();
        assert_eq!(mempool.size(), 1);
        
        mempool.remove_transaction(&tx_hash);
        assert_eq!(mempool.size(), 0);
        
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
