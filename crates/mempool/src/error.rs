use thiserror::Error;

pub type Result<T> = std::result::Result<T, MempoolError>;

#[derive(Error, Debug, Clone)]
pub enum MempoolError {
    #[error("Transaction already in mempool: {0}")]
    DuplicateTransaction(String),

    #[error("Transaction validation failed: {0}")]
    ValidationFailed(String),

    #[error("Mempool is full (max: {max}, current: {current})")]
    MempoolFull { max: usize, current: usize },

    #[error("Insufficient balance: required {required}, available {available}")]
    InsufficientBalance { required: u64, available: u64 },

    #[error("Invalid nonce: expected {expected}, got {got}")]
    InvalidNonce { expected: u64, got: u64 },

    #[error("Nonce too far ahead: current {current}, got {got}, max gap {max_gap}")]
    NonceTooFar {
        current: u64,
        got: u64,
        max_gap: u64,
    },

    #[error("Transaction fee too low: minimum {min}, got {got}")]
    FeeTooLow { min: u64, got: u64 },

    #[error("Transaction expired")]
    Expired,

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Transaction not found in mempool")]
    NotFound,

    #[error("Invalid transaction")]
    InvalidTransaction,
}
