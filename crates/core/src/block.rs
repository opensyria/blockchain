use crate::constants::{GENESIS_DIFFICULTY, GENESIS_NONCE, GENESIS_TIMESTAMP, MAX_FUTURE_DRIFT_SECS};
use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block version for protocol upgrades
    pub version: u32,
    /// Previous block hash (genesis = all zeros)
    pub previous_hash: [u8; 32],
    /// Merkle root of transactions
    pub merkle_root: [u8; 32],
    /// Block timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Mining difficulty target
    pub difficulty: u32,
    /// Proof-of-work nonce
    pub nonce: u64,
}

impl BlockHeader {
    /// Calculate block header hash
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.version.to_le_bytes());
        hasher.update(self.previous_hash);
        hasher.update(self.merkle_root);
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.difficulty.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.finalize().into()
    }

    /// Check if hash meets difficulty target (leading zeros)
    pub fn meets_difficulty(&self) -> bool {
        let hash = self.hash();
        let leading_zeros = self.difficulty / 8;
        let remainder = self.difficulty % 8;

        // Check full zero bytes
        for &byte in hash.iter().take(leading_zeros as usize) {
            if byte != 0 {
                return false;
            }
        }

        // Check remaining bits
        if remainder > 0 {
            let byte = hash[leading_zeros as usize];
            let mask = 0xFF << (8 - remainder);
            if byte & mask != 0 {
                return false;
            }
        }

        true
    }
}

/// Complete block with header and transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Create new block
    pub fn new(previous_hash: [u8; 32], transactions: Vec<Transaction>, difficulty: u32) -> Self {
        let merkle_root = Self::calculate_merkle_root(&transactions);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();

        let header = BlockHeader {
            version: 1,
            previous_hash,
            merkle_root,
            timestamp,
            difficulty,
            nonce: 0,
        };

        Self {
            header,
            transactions,
        }
    }

    /// Create genesis block (canonical, deterministic)
    /// إنشاء الكتلة الأولى (معيارية، حتمية)
    pub fn genesis() -> Self {
        let header = BlockHeader {
            version: 1,
            previous_hash: [0u8; 32],
            merkle_root: [0u8; 32], // Empty transaction list
            timestamp: GENESIS_TIMESTAMP,
            difficulty: GENESIS_DIFFICULTY,
            nonce: GENESIS_NONCE,
        };

        Self {
            header,
            transactions: Vec::new(),
        }
    }

    /// Calculate merkle root from transactions
    fn calculate_merkle_root(transactions: &[Transaction]) -> [u8; 32] {
        if transactions.is_empty() {
            return [0u8; 32];
        }

        let mut hashes: Vec<[u8; 32]> = transactions.iter().map(|tx| tx.hash()).collect();

        while hashes.len() > 1 {
            let mut new_hashes = Vec::new();
            for chunk in hashes.chunks(2) {
                let mut hasher = Sha256::new();
                hasher.update(chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(chunk[1]);
                } else {
                    hasher.update(chunk[0]); // Duplicate if odd
                }
                new_hashes.push(hasher.finalize().into());
            }
            hashes = new_hashes;
        }

        hashes[0]
    }

    /// Get block hash
    pub fn hash(&self) -> [u8; 32] {
        self.header.hash()
    }

    /// Verify all transactions in block
    pub fn verify_transactions(&self) -> Result<(), BlockError> {
        for tx in &self.transactions {
            tx.verify().map_err(|_| BlockError::InvalidTransaction)?;
        }
        Ok(())
    }

    /// Verify merkle root matches transactions
    pub fn verify_merkle_root(&self) -> bool {
        self.header.merkle_root == Self::calculate_merkle_root(&self.transactions)
    }

    /// Validate block timestamp against previous block and system time
    /// التحقق من الطابع الزمني للكتلة مقابل الكتلة السابقة ووقت النظام
    pub fn validate_timestamp(&self, previous_timestamp: u64) -> Result<(), BlockError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BlockError::InvalidTimestamp)?
            .as_secs();

        // Check timestamp is not too far in the future (5 minutes tolerance)
        // NOTE: Consider reducing MAX_FUTURE_DRIFT_SECS to 60s for better security
        if self.header.timestamp > now + MAX_FUTURE_DRIFT_SECS {
            return Err(BlockError::TimestampTooFarFuture);
        }

        // SECURITY: Require STRICT monotonic increase (not equal) to prevent timestamp collision
        if self.header.timestamp <= previous_timestamp {
            return Err(BlockError::TimestampDecreased);
        }

        Ok(())
    }

    /// Validate coinbase transaction (block reward)
    /// التحقق من معاملة الكوين بيس (مكافأة الكتلة)
    pub fn validate_coinbase(&self, block_height: u64) -> Result<(), BlockError> {
        use crate::constants::calculate_block_reward;

        // Genesis block has no coinbase
        if block_height == 0 {
            return Ok(());
        }

        // Block must have at least one transaction (coinbase)
        if self.transactions.is_empty() {
            return Err(BlockError::MissingCoinbase);
        }

        // First transaction must be coinbase
        let coinbase = &self.transactions[0];
        if !coinbase.is_coinbase() {
            return Err(BlockError::MissingCoinbase);
        }

        // Calculate expected reward
        let block_reward = calculate_block_reward(block_height);
        
        // SECURITY: Use checked_add to prevent overflow in fee summation
        let total_fees = self.transactions.iter()
            .skip(1) // Skip coinbase itself
            .try_fold(0u64, |acc, tx| acc.checked_add(tx.fee))
            .ok_or(BlockError::InvalidCoinbaseAmount)?;

        let expected_reward = block_reward.checked_add(total_fees)
            .ok_or(BlockError::InvalidCoinbaseAmount)?;

        // Validate coinbase amount
        if coinbase.amount != expected_reward {
            return Err(BlockError::InvalidCoinbaseAmount);
        }

        // Ensure no other coinbase transactions
        for tx in self.transactions.iter().skip(1) {
            if tx.is_coinbase() {
                return Err(BlockError::MultipleCoinbase);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockError {
    InvalidTransaction,
    InvalidMerkleRoot,
    InvalidProofOfWork,
    InvalidPreviousHash,
    InvalidTimestamp,
    TimestampTooFarFuture,
    TimestampDecreased,
    MissingCoinbase,
    InvalidCoinbaseAmount,
    MultipleCoinbase,
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockError::InvalidTransaction => write!(f, "Block contains invalid transaction"),
            BlockError::InvalidMerkleRoot => write!(f, "Invalid merkle root"),
            BlockError::InvalidProofOfWork => write!(f, "Invalid proof of work"),
            BlockError::InvalidPreviousHash => write!(f, "Invalid previous block hash"),
            BlockError::InvalidTimestamp => write!(f, "Invalid timestamp"),
            BlockError::TimestampTooFarFuture => write!(f, "Block timestamp too far in future"),
            BlockError::TimestampDecreased => write!(f, "Block timestamp decreased from previous block"),
            BlockError::MissingCoinbase => write!(f, "Block missing coinbase transaction"),
            BlockError::InvalidCoinbaseAmount => write!(f, "Coinbase amount incorrect"),
            BlockError::MultipleCoinbase => write!(f, "Block contains multiple coinbase transactions"),
        }
    }
}

impl std::error::Error for BlockError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();
        assert_eq!(genesis.header.previous_hash, [0u8; 32]);
        assert!(genesis.transactions.is_empty());
        assert_eq!(genesis.header.difficulty, crate::constants::GENESIS_DIFFICULTY);
        assert_eq!(genesis.header.timestamp, crate::constants::GENESIS_TIMESTAMP);
    }

    #[test]
    fn test_block_hash_deterministic() {
        let block = Block::genesis();
        let hash1 = block.hash();
        let hash2 = block.hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_difficulty_check() {
        let mut header = BlockHeader {
            version: 1,
            previous_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: 0,
            difficulty: 8, // 1 leading zero byte
            nonce: 0,
        };

        // Hash won't meet difficulty initially
        assert!(!header.meets_difficulty());

        // Simulate finding valid nonce
        for nonce in 0..100000 {
            header.nonce = nonce;
            if header.meets_difficulty() {
                break;
            }
        }
    }

    #[test]
    fn test_merkle_root_changes_with_transactions() {
        use crate::{crypto::KeyPair, transaction::Transaction};

        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let mut tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            1_000_000,
            100,
            0,
        );
        let sig_hash = tx.signing_hash();
        tx = tx.with_signature(sender.sign(&sig_hash));

        let block1 = Block::new([0u8; 32], vec![], 16);
        let block2 = Block::new([0u8; 32], vec![tx], 16);

        assert_ne!(block1.header.merkle_root, block2.header.merkle_root);
    }

    #[test]
    fn test_merkle_root_verification() {
        use crate::{crypto::KeyPair, transaction::Transaction};

        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let mut tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            1_000_000,
            100,
            0,
        );
        let sig_hash = tx.signing_hash();
        tx = tx.with_signature(sender.sign(&sig_hash));

        let block = Block::new([0u8; 32], vec![tx], 16);
        assert!(block.verify_merkle_root());
    }

    #[test]
    fn test_timestamp_validation_future_block() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Block with timestamp 10 minutes in future (should fail)
        let future_block = Block::new([0u8; 32], vec![], 16);
        let mut future_header = future_block.header.clone();
        future_header.timestamp = now + 600; // 10 minutes ahead

        let future_block = Block {
            header: future_header,
            transactions: vec![],
        };

        let result = future_block.validate_timestamp(now - 120);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BlockError::TimestampTooFarFuture);
    }

    #[test]
    fn test_timestamp_validation_monotonic() {
        // Block with timestamp before previous block (should fail)
        let block = Block::new([0u8; 32], vec![], 16);
        let mut header = block.header.clone();
        header.timestamp = 1000;

        let block = Block {
            header,
            transactions: vec![],
        };

        let result = block.validate_timestamp(2000); // Previous was 2000
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), BlockError::TimestampDecreased);
    }

    #[test]
    fn test_timestamp_validation_valid() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let block = Block::new([0u8; 32], vec![], 16);
        let result = block.validate_timestamp(now - 120);
        assert!(result.is_ok());
    }
}
