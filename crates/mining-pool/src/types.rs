use serde::{Deserialize, Serialize};
use opensyria_core::crypto::PublicKey;

/// Mining pool share submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Share {
    /// Miner's public key
    pub miner: PublicKey,
    /// Block height being mined
    pub height: u64,
    /// Nonce that produced the share
    pub nonce: u64,
    /// Hash result
    pub hash: [u8; 32],
    /// Share difficulty (lower than block difficulty)
    pub difficulty: u32,
    /// Timestamp when share was found
    pub timestamp: u64,
}

/// Mining work assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAssignment {
    /// Block height to mine
    pub height: u64,
    /// Previous block hash
    pub prev_hash: [u8; 32],
    /// Transactions merkle root
    pub merkle_root: [u8; 32],
    /// Target difficulty for the block
    pub block_difficulty: u32,
    /// Share difficulty (easier than block)
    pub share_difficulty: u32,
    /// Nonce range start
    pub nonce_start: u64,
    /// Nonce range end
    pub nonce_end: u64,
}

/// Miner statistics in the pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerStats {
    /// Miner's public key
    pub miner: PublicKey,
    /// Total shares submitted
    pub total_shares: u64,
    /// Valid shares
    pub valid_shares: u64,
    /// Invalid shares
    pub invalid_shares: u64,
    /// Total hash power contributed (estimated)
    pub hashrate: f64,
    /// Total rewards earned (in smallest units)
    pub total_rewards: u64,
    /// Pending rewards not yet paid
    pub pending_rewards: u64,
    /// Last share submission time
    pub last_share_time: u64,
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Total active miners
    pub active_miners: usize,
    /// Total pool hashrate
    pub pool_hashrate: f64,
    /// Blocks mined by pool
    pub blocks_mined: u64,
    /// Current pool difficulty
    pub current_difficulty: u32,
    /// Total shares in current round
    pub current_round_shares: u64,
    /// Pool fee percentage (0-100)
    pub pool_fee: u8,
}

/// Reward distribution method
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RewardMethod {
    /// Proportional: Rewards split by share count
    Proportional,
    /// Pay Per Share: Fixed payment per share
    PPS,
    /// Pay Per Last N Shares
    PPLNS { window: u64 },
}

/// Pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Pool operator's public key (receives fees)
    pub operator: PublicKey,
    /// Pool fee percentage (0-100)
    pub fee_percent: u8,
    /// Minimum payout threshold
    pub min_payout: u64,
    /// Share difficulty
    pub share_difficulty: u32,
    /// Reward distribution method
    pub reward_method: RewardMethod,
    /// Pool server address
    pub server_address: String,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            operator: PublicKey([0u8; 32]),
            fee_percent: 2, // 2% pool fee
            min_payout: 1_000_000, // 1 Lira minimum
            share_difficulty: 12, // Easier than typical block difficulty
            reward_method: RewardMethod::Proportional,
            server_address: "0.0.0.0:3333".to_string(),
        }
    }
}
