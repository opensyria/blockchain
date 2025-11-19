use crate::StorageError;
use opensyria_core::{Block, block::BlockError, Transaction};
use rocksdb::{ColumnFamilyDescriptor, Options, WriteBatch, DB};
use std::path::PathBuf;

/// Column family names for secondary indexes
const CF_TX_INDEX: &str = "tx_index";       // tx_hash → (block_height, tx_index)
const CF_ADDRESS_INDEX: &str = "address_index"; // address → Vec<tx_hash>
const CF_BLOCK_HASH_INDEX: &str = "block_hash_index"; // block_hash → height

/// Blockchain storage using RocksDB with secondary indexes
/// التخزين المستمر لسلسلة الكتل باستخدام RocksDB مع الفهارس الثانوية
pub struct BlockchainStorage {
    db: DB,
}

impl BlockchainStorage {
    /// Open blockchain storage at path with secondary indexes
    /// فتح تخزين سلسلة الكتل مع الفهارس الثانوية
    pub fn open(path: PathBuf) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);

        // Define column families for secondary indexes
        let cf_descriptors = vec![
            ColumnFamilyDescriptor::new("default", Options::default()),
            ColumnFamilyDescriptor::new(CF_TX_INDEX, Options::default()),
            ColumnFamilyDescriptor::new(CF_ADDRESS_INDEX, Options::default()),
            ColumnFamilyDescriptor::new(CF_BLOCK_HASH_INDEX, Options::default()),
        ];

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)?;

        Ok(Self { db })
    }

    /// Save block to storage
    pub fn put_block(&self, block: &Block) -> Result<(), StorageError> {
        let hash = block.hash();
        let data = bincode::serialize(block)?;

        self.db.put(hash, &data)?;

        // Also store by height if we know it
        // For now, just store by hash

        Ok(())
    }

    /// Get block by hash
    pub fn get_block(&self, hash: &[u8; 32]) -> Result<Option<Block>, StorageError> {
        match self.db.get(hash)? {
            Some(data) => {
                let block: Block = bincode::deserialize(&data)?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// Store the current chain tip (latest block hash)
    pub fn set_chain_tip(&self, hash: &[u8; 32]) -> Result<(), StorageError> {
        self.db.put(b"chain_tip", hash)?;
        Ok(())
    }

    /// Get the current chain tip
    pub fn get_chain_tip(&self) -> Result<Option<[u8; 32]>, StorageError> {
        match self.db.get(b"chain_tip")? {
            Some(data) => {
                if data.len() != 32 {
                    return Ok(None);
                }
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&data);
                Ok(Some(hash))
            }
            None => Ok(None),
        }
    }

    /// Store block height mapping (height -> hash)
    pub fn set_block_height(&self, height: u64, hash: &[u8; 32]) -> Result<(), StorageError> {
        let key = format!("height_{}", height);
        self.db.put(key.as_bytes(), hash)?;
        Ok(())
    }

    /// Get block hash by height
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Block>, StorageError> {
        let key = format!("height_{}", height);
        match self.db.get(key.as_bytes())? {
            Some(hash_data) => {
                if hash_data.len() != 32 {
                    return Ok(None);
                }
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&hash_data);
                self.get_block(&hash)
            }
            None => Ok(None),
        }
    }

    /// Get current blockchain height
    pub fn get_chain_height(&self) -> Result<u64, StorageError> {
        match self.db.get(b"chain_height")? {
            Some(data) => {
                let bytes: [u8; 8] = data.try_into().map_err(|_| StorageError::InvalidChain)?;
                Ok(u64::from_le_bytes(bytes))
            }
            None => Ok(0),
        }
    }

    /// Set blockchain height
    fn set_chain_height(&self, height: u64) -> Result<(), StorageError> {
        self.db.put(b"chain_height", height.to_le_bytes())?;
        Ok(())
    }

    /// Index transaction in secondary index (tx_hash → (block_height, tx_index))
    /// فهرسة المعاملة في الفهرس الثانوي
    fn index_transaction(&self, tx: &Transaction, block_height: u64, tx_index: usize) -> Result<(), StorageError> {
        let tx_hash = tx.hash();
        let tx_cf = self.db.cf_handle(CF_TX_INDEX)
            .ok_or(StorageError::ColumnFamilyNotFound)?;
        
        // Store: tx_hash → (block_height, tx_index)
        let location = bincode::serialize(&(block_height, tx_index))?;
        self.db.put_cf(tx_cf, tx_hash, location)?;
        
        Ok(())
    }

    /// Index address transactions (address → Vec<tx_hash>)
    /// فهرسة معاملات العنوان
    fn index_address(&self, address: &[u8; 32], tx_hash: &[u8; 32]) -> Result<(), StorageError> {
        let addr_cf = self.db.cf_handle(CF_ADDRESS_INDEX)
            .ok_or(StorageError::ColumnFamilyNotFound)?;
        
        let addr_key = format!("addr_{}", hex::encode(address));
        
        // Get existing transaction hashes for this address
        let mut tx_hashes: Vec<[u8; 32]> = self.db
            .get_cf(addr_cf, addr_key.as_bytes())?
            .map(|data| bincode::deserialize(&data).unwrap_or_default())
            .unwrap_or_default();
        
        // Append new transaction hash
        tx_hashes.push(*tx_hash);
        
        // Store updated list
        self.db.put_cf(addr_cf, addr_key.as_bytes(), bincode::serialize(&tx_hashes)?)?;
        
        Ok(())
    }

    /// Index block hash (block_hash → block_height)
    /// فهرسة تجزئة الكتلة
    fn index_block_hash(&self, block_hash: &[u8; 32], height: u64) -> Result<(), StorageError> {
        let block_cf = self.db.cf_handle(CF_BLOCK_HASH_INDEX)
            .ok_or(StorageError::ColumnFamilyNotFound)?;
        
        self.db.put_cf(block_cf, block_hash, height.to_le_bytes())?;
        Ok(())
    }

    /// Get transaction by hash (O(1) lookup using index)
    /// الحصول على المعاملة بواسطة التجزئة (بحث O(1) باستخدام الفهرس)
    pub fn get_transaction_by_hash(&self, tx_hash: &[u8; 32]) -> Result<Option<(Transaction, u64)>, StorageError> {
        let tx_cf = self.db.cf_handle(CF_TX_INDEX)
            .ok_or(StorageError::ColumnFamilyNotFound)?;
        
        // O(1) index lookup
        if let Some(location_data) = self.db.get_cf(tx_cf, tx_hash)? {
            let (block_height, tx_index): (u64, usize) = bincode::deserialize(&location_data)?;
            
            // Fetch block and extract transaction
            if let Some(block) = self.get_block_by_height(block_height)? {
                if let Some(tx) = block.transactions.get(tx_index) {
                    return Ok(Some((tx.clone(), block_height)));
                }
            }
        }
        
        Ok(None)
    }

    /// Get all transaction hashes for an address (O(1) lookup using index)
    /// الحصول على جميع تجزئات المعاملات لعنوان (بحث O(1) باستخدام الفهرس)
    pub fn get_address_transactions(&self, address: &[u8; 32]) -> Result<Vec<[u8; 32]>, StorageError> {
        let addr_cf = self.db.cf_handle(CF_ADDRESS_INDEX)
            .ok_or(StorageError::ColumnFamilyNotFound)?;
        
        let addr_key = format!("addr_{}", hex::encode(address));
        
        let tx_hashes: Vec<[u8; 32]> = self.db
            .get_cf(addr_cf, addr_key.as_bytes())?
            .map(|data| bincode::deserialize(&data).unwrap_or_default())
            .unwrap_or_default();
        
        Ok(tx_hashes)
    }

    /// Get address balance by scanning indexed transactions (optimized)
    /// الحصول على رصيد العنوان بمسح المعاملات المفهرسة (محسّن)
    pub fn get_address_balance(&self, address: &[u8; 32]) -> Result<u64, StorageError> {
        let tx_hashes = self.get_address_transactions(address)?;
        
        let mut balance: i64 = 0;
        
        // Only scan transactions involving this address (much smaller set!)
        for tx_hash in tx_hashes {
            if let Some((tx, _)) = self.get_transaction_by_hash(&tx_hash)? {
                // Skip coinbase transactions in balance calculation
                if !tx.is_coinbase() {
                    if tx.from.0 == *address {
                        balance -= tx.amount as i64 + tx.fee as i64;
                    }
                }
                if tx.to.0 == *address {
                    balance += tx.amount as i64;
                }
            }
        }
        
        Ok(balance.max(0) as u64)
    }

    /// Get block height by block hash (O(1) lookup using index)
    /// الحصول على ارتفاع الكتلة بواسطة تجزئة الكتلة (بحث O(1) باستخدام الفهرس)
    pub fn get_block_height_by_hash(&self, block_hash: &[u8; 32]) -> Result<Option<u64>, StorageError> {
        let block_cf = self.db.cf_handle(CF_BLOCK_HASH_INDEX)
            .ok_or(StorageError::ColumnFamilyNotFound)?;
        
        if let Some(height_data) = self.db.get_cf(block_cf, block_hash)? {
            let bytes: [u8; 8] = height_data.try_into()
                .map_err(|_| StorageError::InvalidChain)?;
            Ok(Some(u64::from_le_bytes(bytes)))
        } else {
            Ok(None)
        }
    }

    /// Append block to chain (validates and stores)
    pub fn append_block(&self, block: &Block) -> Result<(), StorageError> {
        // Get current tip
        let current_height = self.get_chain_height()?;
        let current_tip = self.get_chain_tip()?;

        // 1. Verify proof of work (skip for genesis block)
        let is_genesis = current_height == 0 && block.header.previous_hash == [0u8; 32];
        if !is_genesis && !block.header.meets_difficulty() {
            return Err(StorageError::InvalidProofOfWork);
        }

        // 2. Verify transaction signatures
        block.verify_transactions()
            .map_err(|_| StorageError::InvalidTransaction)?;

        // 3. Verify merkle root
        if !block.verify_merkle_root() {
            return Err(StorageError::InvalidMerkleRoot);
        }

        // 4. Validate timestamp against previous block (skip for genesis)
        if !is_genesis {
            if let Some(tip_hash) = current_tip {
                if let Some(prev_block) = self.get_block(&tip_hash)? {
                    block.validate_timestamp(prev_block.header.timestamp)
                        .map_err(|e| match e {
                            BlockError::TimestampTooFarFuture => StorageError::TimestampTooFarFuture,
                            BlockError::TimestampDecreased => StorageError::TimestampDecreased,
                            _ => StorageError::InvalidChain,
                        })?;
                }
            }
        }

        // 5. Validate previous hash matches
        if let Some(tip_hash) = current_tip {
            if block.header.previous_hash != tip_hash {
                return Err(StorageError::InvalidChain);
            }
        } else if current_height == 0 {
            // Genesis block should have zero previous hash
            if block.header.previous_hash != [0u8; 32] {
                return Err(StorageError::InvalidChain);
            }
        }

        // Store block
        let block_hash = block.hash();
        self.put_block(block)?;

        // Update height mapping
        let new_height = current_height + 1;
        self.set_block_height(new_height, &block_hash)?;
        self.set_chain_height(new_height)?;

        // Update chain tip
        self.set_chain_tip(&block_hash)?;

        // ✅ INDEX BLOCK HASH
        self.index_block_hash(&block_hash, new_height)?;

        // ✅ INDEX TRANSACTIONS
        for (tx_idx, tx) in block.transactions.iter().enumerate() {
            let tx_hash = tx.hash();
            
            // Index: tx_hash → (block_height, tx_index)
            self.index_transaction(tx, new_height, tx_idx)?;
            
            // Index: from_address → append tx_hash
            if !tx.is_coinbase() {
                self.index_address(&tx.from.0, &tx_hash)?;
            }
            
            // Index: to_address → append tx_hash
            self.index_address(&tx.to.0, &tx_hash)?;
        }

        Ok(())
    }

    /// Append block with checkpoint verification (for syncing from network)
    /// إضافة كتلة مع التحقق من نقطة الفحص (للمزامنة من الشبكة)
    pub fn append_block_with_checkpoint(
        &self,
        block: &Block,
        use_testnet: bool,
    ) -> Result<(), StorageError> {
        // First, do standard validation
        self.append_block(block)?;

        // Then verify checkpoint if this height is a checkpoint
        let new_height = self.get_chain_height()?;
        let block_hash = block.hash();

        if let Err(e) = opensyria_consensus::verify_checkpoint(new_height, &block_hash, use_testnet) {
            return Err(StorageError::CheckpointMismatch {
                height: new_height,
                expected: format!("{}", e),
                got: format!("{:x?}", &block_hash[..4]),
            });
        }

        Ok(())
    }

    /// Get blocks in range [start_height, end_height]
    pub fn get_block_range(&self, start: u64, end: u64) -> Result<Vec<Block>, StorageError> {
        let mut blocks = Vec::new();

        for height in start..=end {
            if let Some(block) = self.get_block_by_height(height)? {
                blocks.push(block);
            } else {
                break;
            }
        }

        Ok(blocks)
    }

    /// Revert blockchain to specified height (for chain reorganizations)
    /// إعادة سلسلة الكتل إلى ارتفاع محدد (لإعادة تنظيم السلسلة)
    pub fn revert_to_height(&self, target_height: u64) -> Result<Vec<Block>, StorageError> {
        let current_height = self.get_chain_height()?;

        if target_height >= current_height {
            return Ok(Vec::new()); // Nothing to revert
        }

        let mut reverted_blocks = Vec::new();

        // Collect blocks that will be reverted (for state rollback)
        for height in (target_height + 1)..=current_height {
            if let Some(block) = self.get_block_by_height(height)? {
                reverted_blocks.push(block);
            }
        }

        // Use atomic batch to remove all blocks at once
        let mut batch = WriteBatch::default();

        for height in (target_height + 1)..=current_height {
            if let Some(block) = self.get_block_by_height(height)? {
                let block_hash = block.hash();

                // Delete block data
                batch.delete(&block_hash);

                // Delete height index
                let height_key = format!("height_{}", height);
                batch.delete(height_key.as_bytes());
            }
        }

        // Update chain state
        batch.put(b"chain_height", target_height.to_le_bytes());

        // Update chain tip to target height's block
        if target_height > 0 {
            if let Some(block) = self.get_block_by_height(target_height)? {
                batch.put(b"chain_tip", &block.hash());
            }
        } else {
            // Reverted to genesis
            batch.put(b"chain_tip", &[0u8; 32]);
        }

        // Commit all changes atomically
        self.db.write(batch)?;

        Ok(reverted_blocks)
    }

    /// Handle chain reorganization - revert to fork point and apply new blocks
    /// معالجة إعادة تنظيم السلسلة - العودة إلى نقطة التفرع وتطبيق الكتل الجديدة
    pub fn reorganize(
        &self,
        fork_height: u64,
        new_blocks: Vec<Block>,
    ) -> Result<Vec<Block>, StorageError> {
        // Step 1: Revert to fork point
        let reverted_blocks = self.revert_to_height(fork_height)?;

        // Step 2: Apply new blocks
        for block in new_blocks {
            self.append_block(&block)?;
        }

        // Return reverted blocks so state can be rolled back
        Ok(reverted_blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// Helper function to mine a block for testing
    fn mine_block(mut block: Block) -> Block {
        // Simple mining: find nonce that meets difficulty
        for nonce in 0..1_000_000 {
            block.header.nonce = nonce;
            if block.header.meets_difficulty() {
                return block;
            }
        }
        panic!("Failed to mine block with difficulty {}", block.header.difficulty);
    }

    #[test]
    fn test_storage_genesis_block() {
        let dir = tempdir().unwrap();
        let storage = BlockchainStorage::open(dir.path().to_path_buf()).unwrap();

        let genesis = Block::genesis();
        storage.append_block(&genesis).unwrap();

        assert_eq!(storage.get_chain_height().unwrap(), 1);

        let retrieved = storage.get_block_by_height(1).unwrap().unwrap();
        assert_eq!(retrieved.hash(), genesis.hash());
    }

    #[test]
    fn test_storage_chain_validation() {
        let dir = tempdir().unwrap();
        let storage = BlockchainStorage::open(dir.path().to_path_buf()).unwrap();

        let genesis = Block::genesis();
        storage.append_block(&genesis).unwrap();

        // Valid next block
        let block2 = mine_block(Block::new(genesis.hash(), vec![], 16));
        assert!(storage.append_block(&block2).is_ok());

        // Invalid block (wrong previous hash) - mine it so only previous_hash is wrong
        let invalid_block = mine_block(Block::new([1u8; 32], vec![], 16));
        assert!(storage.append_block(&invalid_block).is_err());
    }

    #[test]
    fn test_storage_block_retrieval() {
        let dir = tempdir().unwrap();
        let storage = BlockchainStorage::open(dir.path().to_path_buf()).unwrap();

        let genesis = Block::genesis();
        let genesis_hash = genesis.hash();
        storage.append_block(&genesis).unwrap();

        // Retrieve by hash
        let by_hash = storage.get_block(&genesis_hash).unwrap().unwrap();
        assert_eq!(by_hash.hash(), genesis_hash);

        // Retrieve by height
        let by_height = storage.get_block_by_height(1).unwrap().unwrap();
        assert_eq!(by_height.hash(), genesis_hash);
    }

    #[test]
    fn test_revert_to_height() {
        let dir = tempdir().unwrap();
        let storage = BlockchainStorage::open(dir.path().to_path_buf()).unwrap();

        // Build chain: genesis -> block2 -> block3 -> block4
        let genesis = Block::genesis();
        storage.append_block(&genesis).unwrap();

        let block2 = mine_block(Block::new(genesis.hash(), vec![], 16));
        storage.append_block(&block2).unwrap();

        let block3 = mine_block(Block::new(block2.hash(), vec![], 16));
        storage.append_block(&block3).unwrap();

        let block4 = mine_block(Block::new(block3.hash(), vec![], 16));
        storage.append_block(&block4).unwrap();

        assert_eq!(storage.get_chain_height().unwrap(), 4);

        // Revert to height 2
        let reverted = storage.revert_to_height(2).unwrap();
        assert_eq!(reverted.len(), 2); // Reverted blocks 3 and 4
        assert_eq!(storage.get_chain_height().unwrap(), 2);

        // Verify chain tip is block2
        let tip = storage.get_chain_tip().unwrap().unwrap();
        assert_eq!(tip, block2.hash());

        // Verify block3 and block4 are removed
        assert!(storage.get_block_by_height(3).unwrap().is_none());
        assert!(storage.get_block_by_height(4).unwrap().is_none());
    }

    #[test]
    fn test_chain_reorganization() {
        let dir = tempdir().unwrap();
        let storage = BlockchainStorage::open(dir.path().to_path_buf()).unwrap();

        // Original chain: genesis -> block2 -> block3
        let genesis = Block::genesis();
        storage.append_block(&genesis).unwrap();

        let block2 = mine_block(Block::new(genesis.hash(), vec![], 16));
        storage.append_block(&block2).unwrap();

        let block3 = mine_block(Block::new(block2.hash(), vec![], 16));
        storage.append_block(&block3).unwrap();

        assert_eq!(storage.get_chain_height().unwrap(), 3);

        // Fork chain: genesis -> block2 -> block3' -> block4'
        let block3_fork = mine_block(Block::new(block2.hash(), vec![], 16));
        let block4_fork = mine_block(Block::new(block3_fork.hash(), vec![], 16));

        // Reorganize to fork at height 2
        let reverted = storage
            .reorganize(2, vec![block3_fork.clone(), block4_fork.clone()])
            .unwrap();

        assert_eq!(reverted.len(), 1); // Only block3 was reverted
        assert_eq!(storage.get_chain_height().unwrap(), 4);

        // Verify new chain tip is block4_fork
        let tip = storage.get_chain_tip().unwrap().unwrap();
        assert_eq!(tip, block4_fork.hash());

        // Verify block3 is replaced by block3_fork
        let b3 = storage.get_block_by_height(3).unwrap().unwrap();
        assert_eq!(b3.hash(), block3_fork.hash());
    }

    #[test]
    fn test_indexed_transaction_lookup() {
        use opensyria_core::{Transaction, crypto::KeyPair};
        
        let dir = tempdir().unwrap();
        let storage = BlockchainStorage::open(dir.path().to_path_buf()).unwrap();

        // Create genesis
        let genesis = Block::genesis();
        storage.append_block(&genesis).unwrap();

        // Create transactions
        let sender_key = KeyPair::generate();
        let sender_pub = sender_key.public_key();
        let recipient_key = KeyPair::generate();
        let recipient_pub = recipient_key.public_key();

        let mut tx1 = Transaction::new(sender_pub.clone(), recipient_pub.clone(), 1000, 10, 0);
        let sig1 = sender_key.sign(&tx1.signing_hash());
        tx1 = tx1.with_signature(sig1);
        let tx1_hash = tx1.hash();

        let mut tx2 = Transaction::new(sender_pub.clone(), recipient_pub.clone(), 2000, 20, 1);
        let sig2 = sender_key.sign(&tx2.signing_hash());
        tx2 = tx2.with_signature(sig2);
        let tx2_hash = tx2.hash();

        // Create block with transactions
        let block2 = mine_block(Block::new(genesis.hash(), vec![tx1.clone(), tx2.clone()], 16));
        storage.append_block(&block2).unwrap();

        // ✅ Test O(1) transaction lookup by hash
        let result = storage.get_transaction_by_hash(&tx1_hash).unwrap();
        assert!(result.is_some());
        let (retrieved_tx, block_height) = result.unwrap();
        assert_eq!(retrieved_tx.hash(), tx1_hash);
        assert_eq!(block_height, 2);

        // ✅ Test second transaction
        let result = storage.get_transaction_by_hash(&tx2_hash).unwrap();
        assert!(result.is_some());
        let (retrieved_tx, block_height) = result.unwrap();
        assert_eq!(retrieved_tx.hash(), tx2_hash);
        assert_eq!(block_height, 2);

        // ✅ Test non-existent transaction
        let fake_hash = [99u8; 32];
        let result = storage.get_transaction_by_hash(&fake_hash).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_indexed_address_lookup() {
        use opensyria_core::{Transaction, crypto::KeyPair};
        
        let dir = tempdir().unwrap();
        let storage = BlockchainStorage::open(dir.path().to_path_buf()).unwrap();

        // Create genesis
        let genesis = Block::genesis();
        storage.append_block(&genesis).unwrap();

        // Create sender and recipient
        let sender_key = KeyPair::generate();
        let sender_pub = sender_key.public_key();
        let recipient_key = KeyPair::generate();
        let recipient_pub = recipient_key.public_key();

        // Create transaction
        let mut tx1 = Transaction::new(sender_pub.clone(), recipient_pub.clone(), 1000, 10, 0);
        let sig1 = sender_key.sign(&tx1.signing_hash());
        tx1 = tx1.with_signature(sig1);
        let tx1_hash = tx1.hash();

        // Add block with transaction
        let block2 = mine_block(Block::new(genesis.hash(), vec![tx1.clone()], 16));
        storage.append_block(&block2).unwrap();

        // ✅ Test address transaction lookup (sender)
        let sender_txs = storage.get_address_transactions(&sender_pub.0).unwrap();
        assert_eq!(sender_txs.len(), 1);
        assert_eq!(sender_txs[0], tx1_hash);

        // ✅ Test address transaction lookup (recipient)
        let recipient_txs = storage.get_address_transactions(&recipient_pub.0).unwrap();
        assert_eq!(recipient_txs.len(), 1);
        assert_eq!(recipient_txs[0], tx1_hash);

        // ✅ Test non-existent address
        let fake_address = [99u8; 32];
        let result = storage.get_address_transactions(&fake_address).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_indexed_block_hash_lookup() {
        let dir = tempdir().unwrap();
        let storage = BlockchainStorage::open(dir.path().to_path_buf()).unwrap();

        let genesis = Block::genesis();
        let genesis_hash = genesis.hash();
        storage.append_block(&genesis).unwrap();

        let block2 = mine_block(Block::new(genesis_hash, vec![], 16));
        let block2_hash = block2.hash();
        storage.append_block(&block2).unwrap();

        // ✅ Test O(1) block height lookup by hash
        let height = storage.get_block_height_by_hash(&genesis_hash).unwrap();
        assert_eq!(height, Some(1));

        let height = storage.get_block_height_by_hash(&block2_hash).unwrap();
        assert_eq!(height, Some(2));

        // ✅ Test non-existent block hash
        let fake_hash = [99u8; 32];
        let height = storage.get_block_height_by_hash(&fake_hash).unwrap();
        assert_eq!(height, None);
    }
}
