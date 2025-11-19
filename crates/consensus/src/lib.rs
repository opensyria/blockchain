// Consensus engine - PoW implementation (future PoS)

pub mod pow;
pub mod checkpoints;

pub use pow::{DifficultyAdjuster, MiningStats, ProofOfWork};
pub use checkpoints::{verify_checkpoint, Checkpoint, CheckpointError, MAINNET_CHECKPOINTS, TESTNET_CHECKPOINTS};

