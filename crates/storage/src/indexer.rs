/// Secondary indexes for fast blockchain queries
/// فهارس ثانوية لاستعلامات سريعة على سلسلة الكتل

use opensyria_core::{block::Block, crypto::PublicKey};
use rocksdb::{BoundColumnFamily, ColumnFamilyDescriptor, Options, WriteBatch, DB};
use std::path::Path;
use std::sync::Arc;

use crate::StorageError;

/// Transaction location in blockchain
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TxLocation {
    pub block_height: u64,
    pub tx_index: usize,
}

/// Blockchain indexer for O(1) lookups
/// مفهرس سلسلة الكتل للبحث السريع
pub struct BlockchainIndexer {
    db: Arc<DB>,
}

impl BlockchainIndexer {
    /// Column family names
    const CF_TX_INDEX: &'static str = "tx_index";
    const CF_ADDRESS_INDEX: &'static str = "address_index";
    const CF_BLOCK_HASH_INDEX: &'static str = "block_hash_index";
    const CF_STATS_CACHE: &'static str = "stats_cache";

    /// Open or create indexer at path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        // Define column families
        let tx_index_cf = ColumnFamilyDescriptor::new(Self::CF_TX_INDEX, Options::default());
        let address_index_cf =
            ColumnFamilyDescriptor::new(Self::CF_ADDRESS_INDEX, Options::default());
        let block_hash_cf =
            ColumnFamilyDescriptor::new(Self::CF_BLOCK_HASH_INDEX, Options::default());
        let stats_cache_cf = ColumnFamilyDescriptor::new(Self::CF_STATS_CACHE, Options::default());

        let db = DB::open_cf_descriptors(
            &db_opts,
            path,
            vec![tx_index_cf, address_index_cf, block_hash_cf, stats_cache_cf],
        )?;

        Ok(Self { db: Arc::new(db) })
    }

    /// Index a new block
    /// فهرسة كتلة جديدة
    pub fn index_block(&self, block: &Block, height: u64) -> Result<(), StorageError> {
        let mut batch = WriteBatch::default();

        let tx_cf = self
            .db
            .cf_handle(Self::CF_TX_INDEX)
            .ok_or_else(|| {
                StorageError::InvalidChain // Repurpose existing error
            })?;
        let addr_cf = self
            .db
            .cf_handle(Self::CF_ADDRESS_INDEX)
            .ok_or_else(|| StorageError::InvalidChain)?;
        let block_hash_cf = self
            .db
            .cf_handle(Self::CF_BLOCK_HASH_INDEX)
            .ok_or_else(|| StorageError::InvalidChain)?;

        // Index block hash -> height
        let block_hash = block.hash();
        batch.put_cf(&block_hash_cf, block_hash, height.to_le_bytes());

        // Index each transaction
        for (tx_idx, tx) in block.transactions.iter().enumerate() {
            let tx_hash = tx.hash();
            let location = TxLocation {
                block_height: height,
                tx_index: tx_idx,
            };

            // tx_hash -> location
            batch.put_cf(&tx_cf, tx_hash, bincode::serialize(&location)?);

            // Add to address indexes
            self.add_tx_to_address_index_internal(&mut batch, &self.db, &tx.from, &tx_hash)?;
            self.add_tx_to_address_index_internal(&mut batch, &self.db, &tx.to, &tx_hash)?;
        }

        self.db.write(batch)?;
        Ok(())
    }

    /// Add transaction hash to address index
    fn add_tx_to_address_index_internal(
        &self,
        batch: &mut WriteBatch,
        db: &Arc<DB>,
        address: &PublicKey,
        tx_hash: &[u8; 32],
    ) -> Result<(), StorageError> {
        let address_key = address.0;

        let cf = db
            .cf_handle(Self::CF_ADDRESS_INDEX)
            .ok_or_else(|| StorageError::InvalidChain)?;

        // Get existing transaction hashes for this address
        let mut tx_hashes: Vec<[u8; 32]> = if let Some(data) = db.get_cf(&cf, address_key)? {
            bincode::deserialize(&data)?
        } else {
            Vec::new()
        };

        // Add new transaction hash if not already present
        if !tx_hashes.contains(tx_hash) {
            tx_hashes.push(*tx_hash);
            batch.put_cf(&cf, address_key, bincode::serialize(&tx_hashes)?);
        }

        Ok(())
    }

    /// Get transaction location by hash - O(1)
    /// الحصول على موقع المعاملة بالتجزئة - O(1)
    pub fn get_tx_location(&self, tx_hash: &[u8; 32]) -> Result<Option<TxLocation>, StorageError> {
        let cf = self
            .db
            .cf_handle(Self::CF_TX_INDEX)
            .ok_or_else(|| StorageError::InvalidChain)?;

        if let Some(data) = self.db.get_cf(&cf, tx_hash)? {
            Ok(Some(bincode::deserialize(&data)?))
        } else {
            Ok(None)
        }
    }

    /// Get block height by hash - O(1)
    /// الحصول على ارتفاع الكتلة بالتجزئة - O(1)
    pub fn get_block_height(&self, block_hash: &[u8; 32]) -> Result<Option<u64>, StorageError> {
        let cf = self
            .db
            .cf_handle(Self::CF_BLOCK_HASH_INDEX)
            .ok_or_else(|| StorageError::InvalidChain)?;

        if let Some(data) = self.db.get_cf(&cf, block_hash)? {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data);
            Ok(Some(u64::from_le_bytes(bytes)))
        } else {
            Ok(None)
        }
    }

    /// Get all transaction hashes for an address - O(k) where k = tx count
    /// الحصول على جميع تجزئات المعاملات لعنوان - O(k)
    pub fn get_address_tx_hashes(
        &self,
        address: &PublicKey,
    ) -> Result<Vec<[u8; 32]>, StorageError> {
        let cf = self
            .db
            .cf_handle(Self::CF_ADDRESS_INDEX)
            .ok_or_else(|| StorageError::InvalidChain)?;

        if let Some(data) = self.db.get_cf(&cf, address.0)? {
            Ok(bincode::deserialize(&data)?)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get paginated transaction hashes for an address (prevents DoS)
    /// الحصول على تجزئات المعاملات مع الترقيم
    pub fn get_address_tx_hashes_paginated(
        &self,
        address: &PublicKey,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<[u8; 32]>, usize), StorageError> {
        let all_hashes = self.get_address_tx_hashes(address)?;
        let total = all_hashes.len();
        
        let paginated: Vec<[u8; 32]> = all_hashes
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect();
        
        Ok((paginated, total))
    }

    /// Get count of transactions for an address
    /// الحصول على عدد المعاملات لعنوان
    pub fn get_address_tx_count(&self, address: &PublicKey) -> Result<usize, StorageError> {
        let cf = self
            .db
            .cf_handle(Self::CF_ADDRESS_INDEX)
            .ok_or_else(|| StorageError::InvalidChain)?;

        if let Some(data) = self.db.get_cf(&cf, address.0)? {
            let tx_hashes: Vec<[u8; 32]> = bincode::deserialize(&data)?;
            Ok(tx_hashes.len())
        } else {
            Ok(0)
        }
    }

    /// Cache blockchain stats
    pub fn cache_stats(&self, stats: &str) -> Result<(), StorageError> {
        let cf = self
            .db
            .cf_handle(Self::CF_STATS_CACHE)
            .ok_or_else(|| StorageError::InvalidChain)?;

        self.db.put_cf(&cf, b"chain_stats", stats)?;
        Ok(())
    }

    /// Get cached blockchain stats
    pub fn get_cached_stats(&self) -> Result<Option<String>, StorageError> {
        let cf = self
            .db
            .cf_handle(Self::CF_STATS_CACHE)
            .ok_or_else(|| StorageError::InvalidChain)?;

        if let Some(data) = self.db.get_cf(&cf, b"chain_stats")? {
            Ok(Some(String::from_utf8_lossy(&data).to_string()))
        } else {
            Ok(None)
        }
    }

    /// Remove block from indexes (for reorganization)
    /// إزالة الكتلة من الفهارس (لإعادة التنظيم)
    pub fn remove_block_from_index(&self, block: &Block, height: u64) -> Result<(), StorageError> {
        let mut batch = WriteBatch::default();

        let tx_cf = self
            .db
            .cf_handle(Self::CF_TX_INDEX)
            .ok_or_else(|| StorageError::InvalidChain)?;
        let addr_cf = self
            .db
            .cf_handle(Self::CF_ADDRESS_INDEX)
            .ok_or_else(|| StorageError::InvalidChain)?;
        let block_hash_cf = self
            .db
            .cf_handle(Self::CF_BLOCK_HASH_INDEX)
            .ok_or_else(|| StorageError::InvalidChain)?;

        // Remove block hash index
        let block_hash = block.hash();
        batch.delete_cf(&block_hash_cf, block_hash);

        // Remove each transaction from indexes
        for tx in &block.transactions {
            let tx_hash = tx.hash();

            // Remove from tx index
            batch.delete_cf(&tx_cf, tx_hash);

            // Remove from address indexes
            self.remove_tx_from_address_index_internal(&mut batch, &self.db, &tx.from, &tx_hash)?;
            self.remove_tx_from_address_index_internal(&mut batch, &self.db, &tx.to, &tx_hash)?;
        }

        self.db.write(batch)?;
        Ok(())
    }

    /// Remove transaction hash from address index
    fn remove_tx_from_address_index_internal(
        &self,
        batch: &mut WriteBatch,
        db: &Arc<DB>,
        address: &PublicKey,
        tx_hash: &[u8; 32],
    ) -> Result<(), StorageError> {
        let address_key = address.0;

        let cf = db
            .cf_handle(Self::CF_ADDRESS_INDEX)
            .ok_or_else(|| StorageError::InvalidChain)?;

        // Get existing transaction hashes for this address
        if let Some(data) = db.get_cf(&cf, address_key)? {
            let mut tx_hashes: Vec<[u8; 32]> = bincode::deserialize(&data)?;

            // Remove the transaction hash
            tx_hashes.retain(|hash| hash != tx_hash);

            if tx_hashes.is_empty() {
                // Remove the address entry if no transactions left
                batch.delete_cf(&cf, address_key);
            } else {
                // Update with remaining transactions
                batch.put_cf(&cf, address_key, bincode::serialize(&tx_hashes)?);
            }
        }

        Ok(())
    }

    /// Rebuild indexes from scratch (for existing blockchains)
    /// إعادة بناء الفهارس من الصفر
    pub fn rebuild_indexes<F>(&self, get_block: F, chain_height: u64) -> Result<(), StorageError>
    where
        F: Fn(u64) -> Result<Option<Block>, StorageError>,
    {
        tracing::info!("Rebuilding blockchain indexes...");

        for height in 0..=chain_height {
            if let Some(block) = get_block(height)? {
                self.index_block(&block, height)?;

                if height % 1000 == 0 {
                    tracing::info!("Indexed {} blocks", height);
                }
            }
        }

        tracing::info!("Index rebuild complete: {} blocks", chain_height + 1);
        Ok(())
    }

    /// Compact all index column families
    /// ضغط جميع الفهارس
    pub fn compact_indexes(&self) -> Result<(), StorageError> {
        if let Some(cf) = self.db.cf_handle(Self::CF_TX_INDEX) {
            self.db.compact_range_cf(&cf, None::<&[u8]>, None::<&[u8]>);
        }
        if let Some(cf) = self.db.cf_handle(Self::CF_ADDRESS_INDEX) {
            self.db.compact_range_cf(&cf, None::<&[u8]>, None::<&[u8]>);
        }
        if let Some(cf) = self.db.cf_handle(Self::CF_BLOCK_HASH_INDEX) {
            self.db.compact_range_cf(&cf, None::<&[u8]>, None::<&[u8]>);
        }
        if let Some(cf) = self.db.cf_handle(Self::CF_STATS_CACHE) {
            self.db.compact_range_cf(&cf, None::<&[u8]>, None::<&[u8]>);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::crypto::KeyPair;
    use opensyria_core::transaction::Transaction;
    use tempfile::TempDir;

    fn create_test_block(height: u64, num_txs: usize) -> Block {
        let mut transactions = Vec::new();

        for i in 0..num_txs {
            let from_keypair = KeyPair::generate();
            let to_keypair = KeyPair::generate();

            let mut tx = Transaction::new(
                from_keypair.public_key().clone(),
                to_keypair.public_key().clone(),
                1000 + i as u64,
                10,
                0,
            );

            let sig = from_keypair.sign(&tx.signing_hash());
            tx = tx.with_signature(sig);
            transactions.push(tx);
        }

        Block::new([height as u8; 32], transactions, 1)
    }

    #[test]
    fn test_index_and_lookup_transaction() {
        let temp_dir = TempDir::new().unwrap();
        let indexer = BlockchainIndexer::open(temp_dir.path().join("index")).unwrap();

        let block = create_test_block(0, 3);
        let tx_hash = block.transactions[1].hash();

        indexer.index_block(&block, 0).unwrap();

        let location = indexer.get_tx_location(&tx_hash).unwrap().unwrap();
        assert_eq!(location.block_height, 0);
        assert_eq!(location.tx_index, 1);
    }

    #[test]
    fn test_index_block_hash() {
        let temp_dir = TempDir::new().unwrap();
        let indexer = BlockchainIndexer::open(temp_dir.path().join("index")).unwrap();

        let block = create_test_block(42, 2);
        let block_hash = block.hash();

        indexer.index_block(&block, 42).unwrap();

        let height = indexer.get_block_height(&block_hash).unwrap().unwrap();
        assert_eq!(height, 42);
    }

    #[test]
    fn test_index_address_transactions() {
        let temp_dir = TempDir::new().unwrap();
        let indexer = BlockchainIndexer::open(temp_dir.path().join("index")).unwrap();

        let block = create_test_block(0, 5);
        let target_address = block.transactions[2].from;

        indexer.index_block(&block, 0).unwrap();

        let tx_hashes = indexer.get_address_tx_hashes(&target_address).unwrap();
        assert!(!tx_hashes.is_empty());
        assert!(tx_hashes.contains(&block.transactions[2].hash()));
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let indexer = BlockchainIndexer::open(temp_dir.path().join("index")).unwrap();

        let stats = r#"{"height": 100, "difficulty": 5}"#;
        indexer.cache_stats(stats).unwrap();

        let cached = indexer.get_cached_stats().unwrap().unwrap();
        assert_eq!(cached, stats);
    }

    #[test]
    fn test_no_duplicate_address_transactions() {
        let temp_dir = TempDir::new().unwrap();
        let indexer = BlockchainIndexer::open(temp_dir.path().join("index")).unwrap();

        let block1 = create_test_block(0, 2);
        let block2 = create_test_block(1, 2);

        let address = block1.transactions[0].from;

        indexer.index_block(&block1, 0).unwrap();
        indexer.index_block(&block2, 1).unwrap();

        let tx_hashes = indexer.get_address_tx_hashes(&address).unwrap();
        // Should only have unique hashes
        let unique_hashes: std::collections::HashSet<_> = tx_hashes.iter().collect();
        assert_eq!(tx_hashes.len(), unique_hashes.len());
    }
}
