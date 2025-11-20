use crate::token::IdentityToken;
use opensyria_core::crypto::PublicKey;
use rocksdb::{DB, Options, BlockBasedOptions};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

/// Persistent storage for identity tokens using RocksDB
pub struct IdentityStorage {
    db: Arc<DB>,
}

#[derive(Debug)]
pub enum StorageError {
    DatabaseError(String),
    SerializationError(String),
    TokenNotFound(String),
    DuplicateToken(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Self::TokenNotFound(id) => write!(f, "Token not found: {}", id),
            Self::DuplicateToken(id) => write!(f, "Token already exists: {}", id),
        }
    }
}

impl std::error::Error for StorageError {}

impl IdentityStorage {
    /// Open or create identity token storage
    /// 
    /// ✅  PERFORMANCE FIX (P1-002): Bloom filters enabled
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        // PERFORMANCE FIX: Enable bloom filters for faster lookups
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_bloom_filter(10.0, false);
        opts.set_block_based_table_factory(&block_opts);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);

        let db = DB::open(&opts, path)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(Self { db: Arc::new(db) })
    }

    /// Store a new identity token
    pub fn store_token(&self, token: &IdentityToken) -> Result<(), StorageError> {
        // Check if token already exists
        if self.get_token(&token.id)?.is_some() {
            return Err(StorageError::DuplicateToken(token.id.clone()));
        }

        let key = Self::token_key(&token.id);
        let config = bincode::config::standard();
        let value = bincode::encode_to_vec(token, config)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        self.db
            .put(&key, &value)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Index by owner
        self.index_by_owner(&token.owner, &token.id)?;

        // Index by creator
        self.index_by_creator(&token.creator, &token.id)?;

        Ok(())
    }

    /// Update an existing token (for transfers)
    pub fn update_token(&self, token: &IdentityToken) -> Result<(), StorageError> {
        let key = Self::token_key(&token.id);
        let config = bincode::config::standard();
        let value = bincode::encode_to_vec(token, config)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        self.db
            .put(&key, &value)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Update owner index
        self.index_by_owner(&token.owner, &token.id)?;

        Ok(())
    }

    /// Get a token by ID
    pub fn get_token(&self, token_id: &str) -> Result<Option<IdentityToken>, StorageError> {
        let key = Self::token_key(token_id);
        
        match self.db.get(&key) {
            Ok(Some(value)) => {
                let config = bincode::config::standard();
                let (token, _): (IdentityToken, _) = bincode::decode_from_slice(&value, config)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some(token))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(StorageError::DatabaseError(e.to_string())),
        }
    }

    /// Get all tokens owned by a specific address
    pub fn get_tokens_by_owner(&self, owner: &PublicKey) -> Result<Vec<IdentityToken>, StorageError> {
        let prefix = Self::owner_index_prefix(owner);
        let mut tokens = Vec::new();

        let iter = self.db.prefix_iterator(&prefix);
        for item in iter {
            let (key, _) = item.map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
            if !key.starts_with(&prefix) {
                break;
            }

            // Extract token ID from key
            if let Some(token_id) = Self::extract_token_id_from_owner_index(&key) {
                if let Some(token) = self.get_token(&token_id)? {
                    tokens.push(token);
                }
            }
        }

        Ok(tokens)
    }

    /// Get all tokens created by a specific address
    pub fn get_tokens_by_creator(&self, creator: &PublicKey) -> Result<Vec<IdentityToken>, StorageError> {
        let prefix = Self::creator_index_prefix(creator);
        let mut tokens = Vec::new();

        let iter = self.db.prefix_iterator(&prefix);
        for item in iter {
            let (key, _) = item.map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
            if !key.starts_with(&prefix) {
                break;
            }

            // Extract token ID from key
            if let Some(token_id) = Self::extract_token_id_from_creator_index(&key) {
                if let Some(token) = self.get_token(&token_id)? {
                    tokens.push(token);
                }
            }
        }

        Ok(tokens)
    }

    /// Delete a token (for burning)
    pub fn delete_token(&self, token_id: &str) -> Result<(), StorageError> {
        // Get token first to clean up indexes
        let token = self.get_token(token_id)?
            .ok_or_else(|| StorageError::TokenNotFound(token_id.to_string()))?;

        // Remove from owner index
        let owner_key = Self::owner_index_key(&token.owner, token_id);
        self.db.delete(&owner_key)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Remove from creator index
        let creator_key = Self::creator_index_key(&token.creator, token_id);
        self.db.delete(&creator_key)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        // Remove token itself
        let key = Self::token_key(token_id);
        self.db.delete(&key)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Get total number of tokens
    pub fn total_tokens(&self) -> Result<usize, StorageError> {
        let prefix = b"token:";
        let mut count = 0;

        let iter = self.db.prefix_iterator(prefix);
        for item in iter {
            let (key, _) = item.map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            if !key.starts_with(prefix) {
                break;
            }
            count += 1;
        }

        Ok(count)
    }

    // Helper methods for key generation
    fn token_key(token_id: &str) -> Vec<u8> {
        format!("token:{}", token_id).into_bytes()
    }

    fn owner_index_key(owner: &PublicKey, token_id: &str) -> Vec<u8> {
        format!("owner:{}:{}", hex::encode(owner.0), token_id).into_bytes()
    }

    fn owner_index_prefix(owner: &PublicKey) -> Vec<u8> {
        format!("owner:{}:", hex::encode(owner.0)).into_bytes()
    }

    fn creator_index_key(creator: &PublicKey, token_id: &str) -> Vec<u8> {
        format!("creator:{}:{}", hex::encode(creator.0), token_id).into_bytes()
    }

    fn creator_index_prefix(creator: &PublicKey) -> Vec<u8> {
        format!("creator:{}:", hex::encode(creator.0)).into_bytes()
    }

    fn extract_token_id_from_owner_index(key: &[u8]) -> Option<String> {
        let key_str = String::from_utf8_lossy(key);
        key_str.split(':').nth(2).map(|s| s.to_string())
    }

    fn extract_token_id_from_creator_index(key: &[u8]) -> Option<String> {
        let key_str = String::from_utf8_lossy(key);
        key_str.split(':').nth(2).map(|s| s.to_string())
    }

    fn index_by_owner(&self, owner: &PublicKey, token_id: &str) -> Result<(), StorageError> {
        let key = Self::owner_index_key(owner, token_id);
        self.db.put(&key, b"")
            .map_err(|e| StorageError::DatabaseError(e.to_string()))
    }

    fn index_by_creator(&self, creator: &PublicKey, token_id: &str) -> Result<(), StorageError> {
        let key = Self::creator_index_key(creator, token_id);
        self.db.put(&key, b"")
            .map_err(|e| StorageError::DatabaseError(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::HeritageMetadata;
    use crate::token::{TokenType, CulturalCategory};
    use tempfile::TempDir;

    #[test]
    fn test_store_and_retrieve_token() {
        let temp_dir = TempDir::new().unwrap();
        let storage = IdentityStorage::open(temp_dir.path()).unwrap();

        let owner = PublicKey([1u8; 32]);
        let metadata = HeritageMetadata::builder()
            .name_en("Test Site".to_string())
            .name_ar("موقع تجريبي".to_string())
            .build();

        let token = IdentityToken::new(
            "test-token-1".to_string(),
            owner,
            TokenType::HeritageSite,
            CulturalCategory::Ancient,
            metadata,
            5, // 5% royalty
            1000,
        ).unwrap();

        storage.store_token(&token).unwrap();

        let retrieved = storage.get_token("test-token-1").unwrap().unwrap();
        assert_eq!(retrieved.id, token.id);
        assert_eq!(retrieved.owner, token.owner);
        assert_eq!(retrieved.royalty_percentage, 5);
    }

    #[test]
    fn test_get_tokens_by_owner() {
        let temp_dir = TempDir::new().unwrap();
        let storage = IdentityStorage::open(temp_dir.path()).unwrap();

        let owner = PublicKey([1u8; 32]);
        let metadata = HeritageMetadata::builder()
            .name_en("Test".to_string())
            .name_ar("تجريبي".to_string())
            .build();

        for i in 0..3 {
            let token = IdentityToken::new(
                format!("token-{}", i),
                owner,
                TokenType::HeritageSite,
                CulturalCategory::Ancient,
                metadata.clone(),
                0,
                1000,
            ).unwrap();
            storage.store_token(&token).unwrap();
        }

        let tokens = storage.get_tokens_by_owner(&owner).unwrap();
        assert_eq!(tokens.len(), 3);
    }

    #[test]
    fn test_delete_token() {
        let temp_dir = TempDir::new().unwrap();
        let storage = IdentityStorage::open(temp_dir.path()).unwrap();

        let owner = PublicKey([1u8; 32]);
        let metadata = HeritageMetadata::builder()
            .name_en("Test".to_string())
            .name_ar("تجريبي".to_string())
            .build();

        let token = IdentityToken::new(
            "delete-me".to_string(),
            owner,
            TokenType::HeritageSite,
            CulturalCategory::Ancient,
            metadata,
            0,
            1000,
        ).unwrap();

        storage.store_token(&token).unwrap();
        assert!(storage.get_token("delete-me").unwrap().is_some());

        storage.delete_token("delete-me").unwrap();
        assert!(storage.get_token("delete-me").unwrap().is_none());
    }
}
