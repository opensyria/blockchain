use crate::{MempoolError, Result};
use opensyria_core::Transaction;
use opensyria_storage::StateStorage;
use std::sync::Arc;
use tokio::sync::RwLock;

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

        // Accept current nonce or next nonce (for pending transactions)
        if tx.nonce < current_nonce {
            return Err(MempoolError::InvalidNonce {
                expected: current_nonce,
                got: tx.nonce,
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
}
