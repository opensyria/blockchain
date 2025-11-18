use crate::transaction::Transaction;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

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
            .unwrap()
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

    /// Create genesis block
    pub fn genesis(difficulty: u32) -> Self {
        Self::new([0u8; 32], Vec::new(), difficulty)
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockError {
    InvalidTransaction,
    InvalidMerkleRoot,
    InvalidProofOfWork,
    InvalidPreviousHash,
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockError::InvalidTransaction => write!(f, "Block contains invalid transaction"),
            BlockError::InvalidMerkleRoot => write!(f, "Invalid merkle root"),
            BlockError::InvalidProofOfWork => write!(f, "Invalid proof of work"),
            BlockError::InvalidPreviousHash => write!(f, "Invalid previous block hash"),
        }
    }
}

impl std::error::Error for BlockError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis(16);
        assert_eq!(genesis.header.previous_hash, [0u8; 32]);
        assert!(genesis.transactions.is_empty());
    }

    #[test]
    fn test_block_hash_deterministic() {
        let block = Block::genesis(16);
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
}
