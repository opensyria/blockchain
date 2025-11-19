use crate::constants::{calculate_block_reward, CHAIN_ID_MAINNET, MAX_TRANSACTION_SIZE, MIN_TRANSACTION_FEE};
use crate::crypto::PublicKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Transaction transferring Digital Lira
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Chain identifier for replay protection (963 = mainnet, 963000 = testnet)
    pub chain_id: u32,
    /// Sender's public key
    pub from: PublicKey,
    /// Recipient's public key
    pub to: PublicKey,
    /// Amount in smallest unit (1 Lira = 1_000_000 units)
    pub amount: u64,
    /// Transaction fee for miners
    pub fee: u64,
    /// Incrementing counter to prevent replay attacks
    pub nonce: u64,
    /// Signature over transaction data
    pub signature: Vec<u8>,
    /// Optional transaction metadata
    pub data: Option<Vec<u8>>,
}

impl Transaction {
    /// Create new unsigned transaction with default mainnet chain ID
    pub fn new(from: PublicKey, to: PublicKey, amount: u64, fee: u64, nonce: u64) -> Self {
        Self::new_with_chain_id(CHAIN_ID_MAINNET, from, to, amount, fee, nonce)
    }

    /// Create new unsigned transaction with specific chain ID
    pub fn new_with_chain_id(
        chain_id: u32,
        from: PublicKey,
        to: PublicKey,
        amount: u64,
        fee: u64,
        nonce: u64,
    ) -> Self {
        Self {
            chain_id,
            from,
            to,
            amount,
            fee,
            nonce,
            signature: Vec::new(),
            data: None,
        }
    }

    /// Add optional data payload
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Set signature (typically called by wallet after signing)
    pub fn with_signature(mut self, signature: Vec<u8>) -> Self {
        self.signature = signature;
        self
    }

    /// Get signing hash (what gets signed by sender)
    /// Includes chain_id for replay protection
    pub fn signing_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.chain_id.to_le_bytes()); // Prevents cross-chain replay
        hasher.update(self.from.0);
        hasher.update(self.to.0);
        hasher.update(self.amount.to_le_bytes());
        hasher.update(self.fee.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        // SECURITY: Always include data field state to prevent post-signature tampering
        match &self.data {
            Some(data) => {
                hasher.update(&[1u8]); // Marker for Some
                hasher.update((data.len() as u64).to_le_bytes());
                hasher.update(data);
            }
            None => {
                hasher.update(&[0u8]); // Marker for None
            }
        }
        hasher.finalize().into()
    }

    /// Verify transaction signature
    pub fn verify(&self) -> Result<(), TransactionError> {
        if self.signature.is_empty() {
            return Err(TransactionError::MissingSignature);
        }

        let message = self.signing_hash();
        self.from
            .verify(&message, &self.signature)
            .map_err(|_| TransactionError::InvalidSignature)?;

        Ok(())
    }

    /// Calculate transaction hash (includes signature for uniqueness)
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.signing_hash());
        hasher.update(&self.signature);
        hasher.finalize().into()
    }

    /// Check if this is a coinbase transaction (creates new coins)
    pub fn is_coinbase(&self) -> bool {
        self.from.is_zero() && self.signature.is_empty()
    }

    /// Create coinbase transaction for block reward
    /// مكافأة المُعدِّن - إنشاء معاملة كوين بيس
    pub fn coinbase(
        chain_id: u32,
        miner_address: PublicKey,
        block_height: u64,
        transaction_fees: u64,
    ) -> Result<Self, TransactionError> {
        let block_reward = calculate_block_reward(block_height);
        let total_reward = block_reward
            .checked_add(transaction_fees)
            .ok_or(TransactionError::RewardOverflow)?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();

        let mut coinbase_data = Vec::new();
        coinbase_data.extend_from_slice(&block_height.to_le_bytes());
        coinbase_data.extend_from_slice(&timestamp.to_le_bytes());

        Ok(Self {
            chain_id,
            from: PublicKey::zero(), // Special zero address for coinbase
            to: miner_address,
            amount: total_reward,
            fee: 0, // Coinbase pays no fee
            nonce: block_height, // Use height as unique identifier
            signature: Vec::new(), // No signature (validated by consensus)
            data: Some(coinbase_data),
        })
    }

    /// Validate transaction size to prevent DoS attacks
    pub fn validate_size(&self) -> Result<(), TransactionError> {
        let size = bincode::serialized_size(self).map_err(|_| TransactionError::InvalidSize)?;
        if size > MAX_TRANSACTION_SIZE as u64 {
            return Err(TransactionError::TooLarge);
        }
        Ok(())
    }

    /// Validate minimum fee requirement (skip for coinbase)
    pub fn validate_fee(&self) -> Result<(), TransactionError> {
        if self.is_coinbase() {
            return Ok(()); // Coinbase has no fee requirement
        }
        if self.fee < MIN_TRANSACTION_FEE {
            return Err(TransactionError::FeeTooLow);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionError {
    MissingSignature,
    InvalidSignature,
    InsufficientBalance,
    InvalidAmount,
    RewardOverflow,
    InvalidSize,
    TooLarge,
    FeeTooLow,
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::MissingSignature => write!(f, "Transaction missing signature"),
            TransactionError::InvalidSignature => write!(f, "Invalid transaction signature"),
            TransactionError::InsufficientBalance => write!(f, "Insufficient balance"),
            TransactionError::InvalidAmount => write!(f, "Invalid transaction amount"),
            TransactionError::RewardOverflow => write!(f, "Block reward calculation overflow"),
            TransactionError::InvalidSize => write!(f, "Cannot calculate transaction size"),
            TransactionError::TooLarge => write!(f, "Transaction exceeds maximum size"),
            TransactionError::FeeTooLow => write!(f, "Transaction fee below minimum"),
        }
    }
}

impl std::error::Error for TransactionError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;

    #[test]
    fn test_transaction_creation_and_signing() {
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let mut tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            1_000_000, // 1 Lira
            100,
            0,
        );

        let sig_hash = tx.signing_hash();
        tx = tx.with_signature(sender.sign(&sig_hash));

        assert!(tx.verify().is_ok());
    }

    #[test]
    fn test_transaction_hash_deterministic() {
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

        let hash1 = tx.hash();
        let hash2 = tx.hash();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_unsigned_transaction_fails_verify() {
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            1_000_000,
            100,
            0,
        );

        assert!(matches!(
            tx.verify(),
            Err(TransactionError::MissingSignature)
        ));
    }

    #[test]
    fn test_chain_id_in_signing_hash() {
        use crate::constants::{CHAIN_ID_MAINNET, CHAIN_ID_TESTNET};

        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let tx_mainnet = Transaction::new_with_chain_id(
            CHAIN_ID_MAINNET,
            sender.public_key(),
            receiver.public_key(),
            1_000_000,
            100,
            0,
        );

        let tx_testnet = Transaction::new_with_chain_id(
            CHAIN_ID_TESTNET,
            sender.public_key(),
            receiver.public_key(),
            1_000_000,
            100,
            0,
        );

        // Different chain IDs should produce different hashes
        assert_ne!(tx_mainnet.signing_hash(), tx_testnet.signing_hash());
    }

    #[test]
    fn test_coinbase_transaction() {
        use crate::constants::CHAIN_ID_MAINNET;

        let miner = KeyPair::generate();
        let coinbase = Transaction::coinbase(
            CHAIN_ID_MAINNET,
            miner.public_key(),
            1, // Block height 1
            1000, // Transaction fees
        )
        .unwrap();

        assert!(coinbase.is_coinbase());
        assert_eq!(coinbase.fee, 0);
        assert_eq!(coinbase.from, PublicKey::zero());
        // Block reward at height 1 is 50 Lira + fees
        assert_eq!(coinbase.amount, 50_000_000 + 1000);
    }

    #[test]
    fn test_coinbase_reward_halving() {
        use crate::constants::{CHAIN_ID_MAINNET, HALVING_INTERVAL};

        let miner = KeyPair::generate();

        // First era
        let cb1 = Transaction::coinbase(CHAIN_ID_MAINNET, miner.public_key(), 1, 0).unwrap();
        assert_eq!(cb1.amount, 50_000_000);

        // After first halving
        let cb2 = Transaction::coinbase(CHAIN_ID_MAINNET, miner.public_key(), HALVING_INTERVAL + 1, 0).unwrap();
        assert_eq!(cb2.amount, 25_000_000);
    }

    #[test]
    fn test_transaction_size_validation() {
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            1_000_000,
            100,
            0,
        );

        // Normal transaction should pass
        assert!(tx.validate_size().is_ok());
    }

    #[test]
    fn test_oversized_transaction_rejected() {
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let mut tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            1_000_000,
            100,
            0,
        );

        // Add massive data payload
        tx.data = Some(vec![0u8; MAX_TRANSACTION_SIZE + 1]);

        // Should fail size validation
        assert!(matches!(
            tx.validate_size(),
            Err(TransactionError::TooLarge)
        ));
    }

    #[test]
    fn test_data_field_authentication_prevents_tampering() {
        // SECURITY TEST: Verify FIX-001 prevents post-signature data tampering
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        // Create transaction without data
        let mut tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            1_000_000,
            100,
            0,
        );

        // Sign with data = None
        let sig_hash = tx.signing_hash();
        tx = tx.with_signature(sender.sign(&sig_hash));

        // Verify signature is valid
        assert!(tx.verify().is_ok());

        // ATTACK: Try to add data after signing
        tx.data = Some(vec![0xDE, 0xAD, 0xBE, 0xEF]);

        // Verification MUST fail (data field now part of signature)
        assert!(tx.verify().is_err(), "Post-signature data tampering should be detected!");
    }

    #[test]
    fn test_data_field_none_vs_empty_vector() {
        // SECURITY TEST: Ensure None and Some(vec![]) produce different signing hashes
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        let tx_none = Transaction::new_with_chain_id(
            963,
            sender.public_key(),
            receiver.public_key(),
            1_000_000,
            100,
            0,
        );

        let tx_empty = Transaction::new_with_chain_id(
            963,
            sender.public_key(),
            receiver.public_key(),
            1_000_000,
            100,
            0,
        ).with_data(vec![]);

        // Different data states must produce different hashes
        assert_ne!(
            tx_none.signing_hash(),
            tx_empty.signing_hash(),
            "None and Some(vec![]) must have different signing hashes"
        );
    }
}
