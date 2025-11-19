use thiserror::Error;

pub type Result<T> = std::result::Result<T, PoolError>;

#[derive(Error, Debug)]
pub enum PoolError {
    #[error("Invalid share: {0}")]
    InvalidShare(String),

    #[error("Share difficulty too low: {actual} < {required}")]
    ShareDifficultyTooLow { actual: u32, required: u32 },

    #[error("Duplicate share")]
    DuplicateShare,

    #[error("Rate limit exceeded - too many shares submitted")]
    RateLimitExceeded,

    #[error("Miner not found: {0}")]
    MinerNotFound(String),

    #[error("Invalid work assignment")]
    InvalidWorkAssignment,

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Insufficient balance for payout")]
    InsufficientBalance,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
