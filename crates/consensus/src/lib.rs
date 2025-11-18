// Consensus engine - PoW implementation (future PoS)

pub mod pow;

pub use pow::{DifficultyAdjuster, MiningStats, ProofOfWork};
