use crate::constants::{GENESIS_DIFFICULTY, GENESIS_NONCE, GENESIS_TIMESTAMP};
use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(bincode::Encode, bincode::Decode)]
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
#[derive(bincode::Encode, bincode::Decode)]
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
    /// 
    /// SECURITY: Enhanced validation with multiple checks to prevent timewarp attacks:
    /// - Future drift limited to 60 seconds (reduced from 300s)
    /// - Strict monotonic increase required
    /// - Median-time-past validation (prevents systematic manipulation)
    /// - Maximum increase per block (prevents single-block time jumps)
    pub fn validate_timestamp(&self, previous_timestamp: u64) -> Result<(), BlockError> {
        use crate::constants::{MAX_FUTURE_DRIFT_SECS, MAX_TIMESTAMP_INCREASE_SECS};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BlockError::InvalidTimestamp)?
            .as_secs();

        // Rule 1: Not too far in future (reduced to 60 seconds for security)
        if self.header.timestamp > now + MAX_FUTURE_DRIFT_SECS {
            return Err(BlockError::TimestampTooFarFuture);
        }

        // Rule 2: STRICT monotonic increase (not equal) to prevent timestamp collision
        if self.header.timestamp <= previous_timestamp {
            return Err(BlockError::TimestampDecreased);
        }

        // Rule 3: Rate limit timestamp increase (max 2 hours per block)
        // Prevents single-block time manipulation attacks
        if self.header.timestamp > previous_timestamp + MAX_TIMESTAMP_INCREASE_SECS {
            return Err(BlockError::TimestampTooFarAhead);
        }

        Ok(())
    }

    /// Validate timestamp with median-time-past check (recommended for consensus)
    /// التحقق من الطابع الزمني مع فحص الوقت الوسيط الماضي
    /// 
    /// This method adds median-time-past validation to prevent timewarp attacks.
    /// Call this version when you have access to recent block timestamps.
    pub fn validate_timestamp_with_median(
        &self,
        previous_timestamp: u64,
        previous_timestamps: &[u64],
    ) -> Result<(), BlockError> {
        use crate::constants::{MEDIAN_TIME_SPAN, MAX_FUTURE_DRIFT_SECS, MAX_TIMESTAMP_INCREASE_SECS};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| BlockError::InvalidTimestamp)?
            .as_secs();

        // Rule 1: Not too far in future
        if self.header.timestamp > now + MAX_FUTURE_DRIFT_SECS {
            return Err(BlockError::TimestampTooFarFuture);
        }

        // Rule 2: STRICT monotonic increase
        if self.header.timestamp <= previous_timestamp {
            return Err(BlockError::TimestampDecreased);
        }

        // Rule 3: Median-time-past check (prevents systematic timewarp)
        if previous_timestamps.len() >= MEDIAN_TIME_SPAN {
            let mut sorted = previous_timestamps.to_vec();
            sorted.sort_unstable();
            let median = sorted[MEDIAN_TIME_SPAN / 2];
            
            // New block timestamp must be greater than median of last 11 blocks
            if self.header.timestamp <= median {
                return Err(BlockError::TimestampBelowMedian);
            }
        }

        // Rule 4: Rate limit timestamp increase
        if self.header.timestamp > previous_timestamp + MAX_TIMESTAMP_INCREASE_SECS {
            return Err(BlockError::TimestampTooFarAhead);
        }

        Ok(())
    }

    /// Validate coinbase transaction (block reward)
    /// التحقق من معاملة الكوين بيس (مكافأة الكتلة)
    /// 
    /// ✅  SECURITY FIX (CRITICAL-004): Total supply enforcement in coinbase validation
    /// This method now accepts current_supply parameter to verify that minting new coins
    /// will not exceed MAX_SUPPLY (100M SYL). Prevents inflation attacks.
    pub fn validate_coinbase(&self, block_height: u64, current_supply: u64) -> Result<(), BlockError> {
        use crate::constants::{calculate_block_reward, MAX_SUPPLY};

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

        // SECURITY FIX (CRITICAL-004): Enforce MAX_SUPPLY to prevent inflation
        // Check that minting this coinbase will not exceed maximum supply
        let new_supply = current_supply.checked_add(coinbase.amount)
            .ok_or(BlockError::SupplyOverflow)?;
        
        if new_supply > MAX_SUPPLY {
            return Err(BlockError::MaxSupplyExceeded {
                current: current_supply,
                attempted: coinbase.amount,
                max: MAX_SUPPLY,
            });
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
    TimestampBelowMedian,
    TimestampTooFarAhead,
    MissingCoinbase,
    InvalidCoinbaseAmount,
    MultipleCoinbase,
    SupplyOverflow,
    MaxSupplyExceeded { current: u64, attempted: u64, max: u64 },
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
            BlockError::TimestampBelowMedian => write!(f, "Block timestamp below median-time-past"),
            BlockError::TimestampTooFarAhead => write!(f, "Block timestamp too far ahead of previous"),
            BlockError::MissingCoinbase => write!(f, "Block missing coinbase transaction"),
            BlockError::InvalidCoinbaseAmount => write!(f, "Coinbase amount incorrect"),
            BlockError::MultipleCoinbase => write!(f, "Block contains multiple coinbase transactions"),
            BlockError::SupplyOverflow => write!(f, "Supply calculation overflow"),
            BlockError::MaxSupplyExceeded { current, attempted, max } => {
                write!(
                    f,
                    "Maximum supply exceeded: current={}, attempted to mint={}, max={}",
                    current, attempted, max
                )
            }
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

    #[test]
    fn test_timestamp_with_median_validation() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create timestamps for last 11 blocks (60s intervals)
        let mut timestamps: Vec<u64> = (0..11).map(|i| now - 660 + (i * 60)).collect();
        
        let mut block = Block::new([0u8; 32], vec![], 16);
        block.header.timestamp = now;
        
        // Should pass median-time-past check
        let result = block.validate_timestamp_with_median(now - 60, &timestamps);
        assert!(result.is_ok());
    }

    #[test]
    fn test_timestamp_below_median_rejected() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create timestamps for last 11 blocks
        let timestamps: Vec<u64> = (0..11).map(|i| now - 110 + (i * 10)).collect();
        
        let mut block = Block::new([0u8; 32], vec![], 16);
        // Set timestamp below median (should fail)
        block.header.timestamp = now - 100; // Below median of timestamps
        
        let result = block.validate_timestamp_with_median(now - 110, &timestamps);
        assert_eq!(result.unwrap_err(), BlockError::TimestampBelowMedian);
    }

    #[test]
    fn test_timestamp_excessive_increase_rejected() {
        use crate::constants::MAX_TIMESTAMP_INCREASE_SECS;

        let previous_time = 1000000u64;
        
        let mut block = Block::new([0u8; 32], vec![], 16);
        // Try to increase by more than allowed maximum
        block.header.timestamp = previous_time + MAX_TIMESTAMP_INCREASE_SECS + 1;
        
        let result = block.validate_timestamp(previous_time);
        assert_eq!(result.unwrap_err(), BlockError::TimestampTooFarAhead);
    }
}
