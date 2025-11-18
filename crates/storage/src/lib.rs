pub mod blockchain;
pub mod state;

pub use blockchain::BlockchainStorage;
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
}

#[derive(Debug)]
pub enum StorageError {
    DatabaseError(rocksdb::Error),
    SerializationError(bincode::Error),
    BlockNotFound,
    InvalidChain,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::DatabaseError(e) => write!(f, "Database error: {}", e),
            StorageError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            StorageError::BlockNotFound => write!(f, "Block not found"),
            StorageError::InvalidChain => write!(f, "Invalid blockchain"),
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
