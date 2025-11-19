use crate::{MempoolError, Result};
use opensyria_core::Transaction;
use opensyria_storage::StateStorage;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Maximum allowed nonce gap for pending transactions
/// Prevents DoS attacks via unbounded future-nonce transactions
const MAX_NONCE_GAP: u64 = 5;

/// Validates transactions before adding to mempool
pub struct TransactionValidator {
    state: Arc<RwLock<StateStorage>>,
    min_fee: u64,
}

impl TransactionValidator {
    /// Create a new transaction validator
    pub fn new(state: Arc<RwLock<StateStorage>>, min_fee: u64) -> Self {
        Self { state, min_fee }
    }

    /// Validate a transaction
    pub async fn validate(&self, tx: &Transaction) -> Result<()> {
        // 1. Verify signature
        if let Err(e) = tx.verify() {
            return Err(MempoolError::ValidationFailed(format!(
                "Invalid signature: {}",
                e
            )));
        }

        // 2. Check minimum fee
        if tx.fee < self.min_fee {
            return Err(MempoolError::FeeTooLow {
                min: self.min_fee,
                got: tx.fee,
            });
        }

        // 3. Check sender balance and nonce
        let state = self.state.read().await;

        let balance = state
            .get_balance(&tx.from)
            .map_err(|e| MempoolError::Storage(e.to_string()))?;

        let required = tx.amount + tx.fee;
        if balance < required {
            return Err(MempoolError::InsufficientBalance {
                required,
                available: balance,
            });
        }

        let current_nonce = state
            .get_nonce(&tx.from)
            .map_err(|e| MempoolError::Storage(e.to_string()))?;

        // SECURITY FIX: Strict nonce validation to prevent DoS
        // Only accept current nonce or a small number of future nonces
        if tx.nonce < current_nonce {
            return Err(MempoolError::InvalidNonce {
                expected: current_nonce,
                got: tx.nonce,
            });
        }

        // NEW: Reject excessive future nonces (DoS prevention)
        // Allows max 5 pending transactions per account
        if tx.nonce > current_nonce + MAX_NONCE_GAP {
            return Err(MempoolError::NonceTooFar {
                current: current_nonce,
                got: tx.nonce,
                max_gap: MAX_NONCE_GAP,
            });
        }

        Ok(())
    }

    /// Validate multiple transactions
    pub async fn validate_batch(&self, transactions: &[Transaction]) -> Vec<Result<()>> {
        let mut results = Vec::with_capacity(transactions.len());
        for tx in transactions {
            results.push(self.validate(tx).await);
        }
        results
    }

    /// Get current nonce for an address (public for mempool)
    pub async fn get_current_nonce(&self, address: &opensyria_core::crypto::PublicKey) -> Result<u64> {
        let state = self.state.read().await;
        state
            .get_nonce(address)
            .map_err(|e| MempoolError::Storage(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::crypto::KeyPair;

    #[tokio::test]
    async fn test_validate_valid_transaction() {
        let temp_dir = std::env::temp_dir().join("mempool_validator_test");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let state = StateStorage::open(temp_dir.clone()).unwrap();

        // Create sender and receiver
        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        // Give sender some balance
        state.set_balance(&sender.public_key(), 1_000_000).unwrap();
        state.set_nonce(&sender.public_key(), 0).unwrap();

        let state = Arc::new(RwLock::new(state));
        let validator = TransactionValidator::new(state, 100);

        // Create valid transaction
        let mut tx = Transaction::new(sender.public_key(), receiver.public_key(), 500_000, 100, 0);
        let msg = tx.signing_hash();
        let sig = sender.sign(&msg);
        tx.signature = sig;

        assert!(validator.validate(&tx).await.is_ok());

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_reject_insufficient_balance() {
        let temp_dir = std::env::temp_dir().join("mempool_validator_insufficient");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let state = StateStorage::open(temp_dir.clone()).unwrap();

        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        // Give sender insufficient balance
        state.set_balance(&sender.public_key(), 100).unwrap();
        state.set_nonce(&sender.public_key(), 0).unwrap();

        let state = Arc::new(RwLock::new(state));
        let validator = TransactionValidator::new(state, 100);

        let mut tx = Transaction::new(sender.public_key(), receiver.public_key(), 500_000, 100, 0);
        let msg = tx.signing_hash();
        let sig = sender.sign(&msg);
        tx.signature = sig;

        match validator.validate(&tx).await {
            Err(MempoolError::InsufficientBalance { .. }) => {}
            _ => panic!("Expected InsufficientBalance error"),
        }

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_reject_excessive_nonce_gap() {
        let temp_dir = std::env::temp_dir().join("mempool_validator_nonce_gap");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let state = StateStorage::open(temp_dir.clone()).unwrap();

        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        // Give sender balance
        state.set_balance(&sender.public_key(), 10_000_000).unwrap();
        state.set_nonce(&sender.public_key(), 0).unwrap();

        let state = Arc::new(RwLock::new(state));
        let validator = TransactionValidator::new(state, 100);

        // Create transaction with excessive future nonce (current=0, gap=5, so max=5)
        let mut tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            500_000,
            100,
            6, // Nonce too far ahead (0 + 5 + 1)
        );
        let msg = tx.signing_hash();
        let sig = sender.sign(&msg);
        tx.signature = sig;

        match validator.validate(&tx).await {
            Err(MempoolError::NonceTooFar { current, got, max_gap }) => {
                assert_eq!(current, 0);
                assert_eq!(got, 6);
                assert_eq!(max_gap, MAX_NONCE_GAP);
            }
            _ => panic!("Expected NonceTooFar error"),
        }

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[tokio::test]
    async fn test_accept_nonce_within_gap() {
        let temp_dir = std::env::temp_dir().join("mempool_validator_nonce_valid");
        let _ = std::fs::remove_dir_all(&temp_dir);

        let state = StateStorage::open(temp_dir.clone()).unwrap();

        let sender = KeyPair::generate();
        let receiver = KeyPair::generate();

        state.set_balance(&sender.public_key(), 10_000_000).unwrap();
        state.set_nonce(&sender.public_key(), 0).unwrap();

        let state = Arc::new(RwLock::new(state));
        let validator = TransactionValidator::new(state, 100);

        // Create transaction with nonce within gap (current=0, max=5, so nonce=5 is OK)
        let mut tx = Transaction::new(
            sender.public_key(),
            receiver.public_key(),
            500_000,
            100,
            5, // Within gap limit
        );
        let msg = tx.signing_hash();
        let sig = sender.sign(&msg);
        tx.signature = sig;

        assert!(validator.validate(&tx).await.is_ok());

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
