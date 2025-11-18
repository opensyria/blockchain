use crate::StorageError;
use opensyria_core::Block;
use rocksdb::{DB, Options};
use std::path::PathBuf;

/// Blockchain storage using RocksDB
pub struct BlockchainStorage {
    db: DB,
}

impl BlockchainStorage {
    /// Open blockchain storage at path
    pub fn open(path: PathBuf) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        let db = DB::open(&opts, path)?;
        
        Ok(Self { db })
    }

    /// Save block to storage
    pub fn put_block(&self, block: &Block) -> Result<(), StorageError> {
        let hash = block.hash();
        let data = bincode::serialize(block)?;
        
        self.db.put(&hash, &data)?;
        
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
                let bytes: [u8; 8] = data.try_into()
                    .map_err(|_| StorageError::InvalidChain)?;
                Ok(u64::from_le_bytes(bytes))
            }
            None => Ok(0),
        }
    }

    /// Set blockchain height
    pub fn set_chain_height(&self, height: u64) -> Result<(), StorageError> {
        self.db.put(b"chain_height", &height.to_le_bytes())?;
        Ok(())
    }

    /// Append block to chain (validates and stores)
    pub fn append_block(&self, block: &Block) -> Result<(), StorageError> {
        // Get current tip
        let current_height = self.get_chain_height()?;
        let current_tip = self.get_chain_tip()?;

        // Validate previous hash matches
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_storage_genesis_block() {
        let dir = tempdir().unwrap();
        let storage = BlockchainStorage::open(dir.path().to_path_buf()).unwrap();

        let genesis = Block::genesis(16);
        storage.append_block(&genesis).unwrap();

        assert_eq!(storage.get_chain_height().unwrap(), 1);
        
        let retrieved = storage.get_block_by_height(1).unwrap().unwrap();
        assert_eq!(retrieved.hash(), genesis.hash());
    }

    #[test]
    fn test_storage_chain_validation() {
        let dir = tempdir().unwrap();
        let storage = BlockchainStorage::open(dir.path().to_path_buf()).unwrap();

        let genesis = Block::genesis(16);
        storage.append_block(&genesis).unwrap();

        // Valid next block
        let block2 = Block::new(genesis.hash(), vec![], 16);
        assert!(storage.append_block(&block2).is_ok());

        // Invalid block (wrong previous hash)
        let invalid_block = Block::new([1u8; 32], vec![], 16);
        assert!(storage.append_block(&invalid_block).is_err());
    }

    #[test]
    fn test_storage_block_retrieval() {
        let dir = tempdir().unwrap();
        let storage = BlockchainStorage::open(dir.path().to_path_buf()).unwrap();

        let genesis = Block::genesis(16);
        let genesis_hash = genesis.hash();
        storage.append_block(&genesis).unwrap();

        // Retrieve by hash
        let by_hash = storage.get_block(&genesis_hash).unwrap().unwrap();
        assert_eq!(by_hash.hash(), genesis_hash);

        // Retrieve by height
        let by_height = storage.get_block_by_height(1).unwrap().unwrap();
        assert_eq!(by_height.hash(), genesis_hash);
    }
}
