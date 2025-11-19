pub mod blockchain;
pub mod indexer;
pub mod state;

pub use blockchain::BlockchainStorage;
pub use indexer::BlockchainIndexer;
pub use state::StateStorage;

use std::path::PathBuf;

/// Combined storage manager for blockchain and state
pub struct Storage {
    pub blockchain: BlockchainStorage,
    pub state: StateStorage,
}

impl Storage {
    /// Open storage at specified path
    pub fn open(path: PathBuf) -> Result<Self, StorageError> {
        let blockchain = BlockchainStorage::open(path.join("blocks"))?;
        let state = StateStorage::open(path.join("state"))?;

        Ok(Self { blockchain, state })
    }

    /// Validate and apply block with full state validation (defense-in-depth)
    /// 
    /// SECURITY: This method provides an additional layer of validation beyond
    /// what BlockchainStorage.append_block() does. It verifies that all transactions
    /// are economically valid (sufficient balances, correct nonces) before applying
    /// state changes. This prevents invalid states from corrupted storage.
    pub fn validate_and_apply_block(&self, block: &opensyria_core::Block) -> Result<(), StorageError> {
        // First, validate block structure (PoW, merkle root, etc.)
        self.blockchain.append_block(block)?;

        // Then, validate and apply state transitions atomically
        // This catches any inconsistencies if blockchain storage was corrupted
        self.state.apply_block_atomic(&block.transactions)?;

        Ok(())
    }

    /// Validate block without applying (for testing/validation)
    pub fn validate_block_state(&self, block: &opensyria_core::Block) -> Result<(), StorageError> {
        // Verify all non-coinbase transactions have sufficient balance
        for tx in &block.transactions {
            if tx.is_coinbase() {
                continue;
            }

            // Check sender balance
            let balance = self.state.get_balance(&tx.from)?;
            let required = tx.amount
                .checked_add(tx.fee)
                .ok_or(StorageError::BalanceOverflow)?;

            if balance < required {
                return Err(StorageError::InsufficientBalance);
            }

            // Check nonce matches expected
            let current_nonce = self.state.get_nonce(&tx.from)?;
            if tx.nonce != current_nonce {
                return Err(StorageError::InvalidTransaction);
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum StorageError {
    DatabaseError(rocksdb::Error),
    SerializationError(bincode::Error),
    BlockNotFound,
    InvalidChain,
    InsufficientBalance,
    BalanceOverflow,
    InvalidProofOfWork,
    InvalidTransaction,
    InvalidMerkleRoot,
    TimestampTooFarFuture,
    TimestampDecreased,
    MissingCoinbase,
    InvalidCoinbaseAmount,
    MultipleCoinbase,
    CheckpointMismatch { height: u64, expected: String, got: String },
    ReorgTooDeep { depth: u64, max: u64 },
    ColumnFamilyNotFound,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::DatabaseError(e) => write!(f, "Database error: {}", e),
            StorageError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            StorageError::BlockNotFound => write!(f, "Block not found"),
            StorageError::InvalidChain => write!(f, "Invalid blockchain"),
            StorageError::InsufficientBalance => write!(f, "Insufficient balance"),
            StorageError::BalanceOverflow => write!(f, "Balance overflow"),
            StorageError::InvalidProofOfWork => write!(f, "Invalid proof of work"),
            StorageError::InvalidTransaction => write!(f, "Invalid transaction in block"),
            StorageError::InvalidMerkleRoot => write!(f, "Invalid merkle root"),
            StorageError::TimestampTooFarFuture => write!(f, "Block timestamp too far in future"),
            StorageError::TimestampDecreased => write!(f, "Block timestamp decreased from previous"),
            StorageError::MissingCoinbase => write!(f, "Block missing coinbase transaction"),
            StorageError::InvalidCoinbaseAmount => write!(f, "Coinbase amount incorrect"),
            StorageError::MultipleCoinbase => write!(f, "Block contains multiple coinbase transactions"),
            StorageError::CheckpointMismatch { height, expected, got } => {
                write!(f, "Checkpoint mismatch at height {}: expected {}, got {}", height, expected, got)
            }
            StorageError::ReorgTooDeep { depth, max } => {
                write!(f, "Reorganization too deep: {} blocks (max {})", depth, max)
            }
            StorageError::ColumnFamilyNotFound => write!(f, "RocksDB column family not found"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<rocksdb::Error> for StorageError {
    fn from(e: rocksdb::Error) -> Self {
        StorageError::DatabaseError(e)
    }
}

impl From<bincode::Error> for StorageError {
    fn from(e: bincode::Error) -> Self {
        StorageError::SerializationError(e)
    }
}
