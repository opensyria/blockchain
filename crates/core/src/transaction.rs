use crate::crypto::PublicKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Transaction transferring Digital Lira
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
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
    /// Create new unsigned transaction
    pub fn new(from: PublicKey, to: PublicKey, amount: u64, fee: u64, nonce: u64) -> Self {
        Self {
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
    pub fn signing_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.from.0);
        hasher.update(&self.to.0);
        hasher.update(&self.amount.to_le_bytes());
        hasher.update(&self.fee.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        if let Some(data) = &self.data {
            hasher.update(data);
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

    /// Calculate transaction hash (unique ID)
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.signing_hash());
        hasher.update(&self.signature);
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionError {
    MissingSignature,
    InvalidSignature,
    InsufficientBalance,
    InvalidAmount,
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::MissingSignature => write!(f, "Transaction missing signature"),
            TransactionError::InvalidSignature => write!(f, "Invalid transaction signature"),
            TransactionError::InsufficientBalance => write!(f, "Insufficient balance"),
            TransactionError::InvalidAmount => write!(f, "Invalid transaction amount"),
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
        
        assert!(matches!(tx.verify(), Err(TransactionError::MissingSignature)));
    }
}
