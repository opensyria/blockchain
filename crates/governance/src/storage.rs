use crate::manager::GovernanceSnapshot;
use bincode;
use rocksdb::{Options, DB};
use std::path::Path;

/// Storage errors
#[derive(Debug)]
pub enum StorageError {
    DatabaseError(rocksdb::Error),
    SerializationError(String),
    NotFound,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e),
            Self::NotFound => write!(f, "Governance snapshot not found"),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<rocksdb::Error> for StorageError {
    fn from(err: rocksdb::Error) -> Self {
        Self::DatabaseError(err)
    }
}

impl From<bincode::error::EncodeError> for StorageError {
    fn from(err: bincode::error::EncodeError) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<bincode::error::DecodeError> for StorageError {
    fn from(err: bincode::error::DecodeError) -> Self {
        Self::SerializationError(err.to_string())
    }
}

/// Persistent storage for governance state
pub struct GovernanceStorage {
    db: DB,
}

impl GovernanceStorage {
    /// Open governance storage at specified path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db = DB::open(&opts, path)?;

        Ok(Self { db })
    }

    /// Save governance snapshot
    pub fn save_snapshot(&self, snapshot: &GovernanceSnapshot) -> Result<(), StorageError> {
        let config = bincode::config::standard();
        let encoded = bincode::encode_to_vec(snapshot, config)?;
        self.db.put(b"governance_snapshot", encoded)?;
        Ok(())
    }

    /// Load governance snapshot
    pub fn load_snapshot(&self) -> Result<GovernanceSnapshot, StorageError> {
        let data = self
            .db
            .get(b"governance_snapshot")?
            .ok_or(StorageError::NotFound)?;

        let config = bincode::config::standard();
        let (snapshot, _): (GovernanceSnapshot, _) = bincode::decode_from_slice(&data, config)?;
        Ok(snapshot)
    }

    /// Check if snapshot exists
    pub fn has_snapshot(&self) -> Result<bool, StorageError> {
        Ok(self.db.get(b"governance_snapshot")?.is_some())
    }

    /// Clear all governance data
    pub fn clear(&self) -> Result<(), StorageError> {
        self.db.delete(b"governance_snapshot")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{GovernanceConfig, Proposal, ProposalType};
    use opensyria_core::crypto::KeyPair;
    use std::env;

    #[test]
    fn test_save_and_load_snapshot() {
        let temp_dir = env::temp_dir().join("governance_storage_test");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let storage = GovernanceStorage::open(&temp_dir).unwrap();

        let proposer = KeyPair::generate();
        let proposal = Proposal::new(
            1,
            proposer.public_key(),
            ProposalType::TextProposal {
                description: "Test".to_string(),
            },
            "Test Proposal".to_string(),
            "Description".to_string(),
            100,
            1000,
            100,
            10_000_000,
        );

        let snapshot = GovernanceSnapshot {
            proposals: vec![proposal],
            votes: vec![],
            balance_snapshots: vec![],
            next_proposal_id: 2,
            config: GovernanceConfig::default(),
        };

        storage.save_snapshot(&snapshot).unwrap();

        let loaded = storage.load_snapshot().unwrap();
        assert_eq!(loaded.proposals.len(), 1);
        assert_eq!(loaded.next_proposal_id, 2);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_has_snapshot() {
        let temp_dir = env::temp_dir().join("governance_has_snapshot_test");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let storage = GovernanceStorage::open(&temp_dir).unwrap();

        assert!(!storage.has_snapshot().unwrap());

        let snapshot = GovernanceSnapshot {
            proposals: vec![],
            votes: vec![],
            balance_snapshots: vec![],
            next_proposal_id: 2,
            config: GovernanceConfig::default(),
        };

        storage.save_snapshot(&snapshot).unwrap();
        assert!(storage.has_snapshot().unwrap());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_clear_snapshot() {
        let temp_dir = env::temp_dir().join("governance_clear_test");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let storage = GovernanceStorage::open(&temp_dir).unwrap();

        let snapshot = GovernanceSnapshot {
            proposals: vec![],
            votes: vec![],
            balance_snapshots: vec![],
            next_proposal_id: 1,
            config: GovernanceConfig::default(),
        };

        storage.save_snapshot(&snapshot).unwrap();
        assert!(storage.has_snapshot().unwrap());

        storage.clear().unwrap();
        assert!(!storage.has_snapshot().unwrap());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
