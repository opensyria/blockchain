use crate::{types::*, error::*};
use opensyria_core::crypto::PublicKey;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Mining pool coordinator
pub struct MiningPool {
    /// Pool configuration
    config: PoolConfig,
    /// Active miners and their stats
    miners: HashMap<PublicKey, MinerStats>,
    /// Current mining round shares
    current_round: Vec<Share>,
    /// Total blocks mined
    blocks_mined: u64,
    /// Current work assignment
    current_work: Option<WorkAssignment>,
}

impl MiningPool {
    /// Create a new mining pool
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            miners: HashMap::new(),
            current_round: Vec::new(),
            blocks_mined: 0,
            current_work: None,
        }
    }

    /// Register a new miner
    pub fn register_miner(&mut self, miner: PublicKey) {
        self.miners.entry(miner).or_insert_with(|| MinerStats {
            miner,
            total_shares: 0,
            valid_shares: 0,
            invalid_shares: 0,
            hashrate: 0.0,
            total_rewards: 0,
            pending_rewards: 0,
            last_share_time: 0,
        });
    }

    /// Create new work assignment
    pub fn create_work(
        &mut self,
        height: u64,
        prev_hash: [u8; 32],
        merkle_root: [u8; 32],
        block_difficulty: u32,
    ) -> WorkAssignment {
        let work = WorkAssignment {
            height,
            prev_hash,
            merkle_root,
            block_difficulty,
            share_difficulty: self.config.share_difficulty,
            nonce_start: 0,
            nonce_end: u64::MAX,
        };

        self.current_work = Some(work.clone());
        work
    }

    /// Submit a share from a miner
    pub fn submit_share(&mut self, share: Share) -> Result<bool> {
        // Verify miner is registered
        if !self.miners.contains_key(&share.miner) {
            return Err(PoolError::MinerNotFound(hex::encode(share.miner.0)));
        }

        // Validate share difficulty
        if !self.validate_share_difficulty(&share) {
            if let Some(stats) = self.miners.get_mut(&share.miner) {
                stats.invalid_shares += 1;
            }
            return Err(PoolError::ShareDifficultyTooLow {
                actual: self.calculate_difficulty(&share.hash),
                required: self.config.share_difficulty,
            });
        }

        // Check for duplicate share
        if self.current_round.iter().any(|s| s.nonce == share.nonce && s.miner == share.miner) {
            if let Some(stats) = self.miners.get_mut(&share.miner) {
                stats.invalid_shares += 1;
            }
            return Err(PoolError::DuplicateShare);
        }

        // Update miner stats
        if let Some(stats) = self.miners.get_mut(&share.miner) {
            stats.total_shares += 1;
            stats.valid_shares += 1;
            stats.last_share_time = share.timestamp;
        }

        // Add to current round
        self.current_round.push(share.clone());

        // Check if this share meets block difficulty
        let is_block = self.validate_block_difficulty(&share);

        Ok(is_block)
    }

    /// Validate share meets minimum difficulty
    fn validate_share_difficulty(&self, share: &Share) -> bool {
        let diff = self.calculate_difficulty(&share.hash);
        diff >= self.config.share_difficulty
    }

    /// Check if share meets block difficulty
    fn validate_block_difficulty(&self, share: &Share) -> bool {
        if let Some(work) = &self.current_work {
            let diff = self.calculate_difficulty(&share.hash);
            diff >= work.block_difficulty
        } else {
            false
        }
    }

    /// Calculate difficulty from hash (count leading zeros)
    fn calculate_difficulty(&self, hash: &[u8; 32]) -> u32 {
        let mut zeros = 0;
        for byte in hash.iter() {
            if *byte == 0 {
                zeros += 8;
            } else {
                zeros += byte.leading_zeros();
                break;
            }
        }
        zeros
    }

    /// Distribute rewards for a found block
    pub fn distribute_rewards(&mut self, block_reward: u64) -> HashMap<PublicKey, u64> {
        let mut rewards = HashMap::new();

        if self.current_round.is_empty() {
            return rewards;
        }

        // Calculate pool fee
        let pool_fee = (block_reward * self.config.fee_percent as u64) / 100;
        let miner_reward = block_reward - pool_fee;

        // Add pool fee to operator
        rewards.insert(self.config.operator, pool_fee);

        match self.config.reward_method {
            RewardMethod::Proportional => {
                // Distribute proportionally by share count
                let total_shares = self.current_round.len() as u64;
                let mut share_counts: HashMap<PublicKey, u64> = HashMap::new();

                for share in &self.current_round {
                    *share_counts.entry(share.miner).or_insert(0) += 1;
                }

                for (miner, count) in share_counts {
                    let reward = (miner_reward * count) / total_shares;
                    *rewards.entry(miner).or_insert(0) += reward;

                    // Update pending rewards
                    if let Some(stats) = self.miners.get_mut(&miner) {
                        stats.pending_rewards += reward;
                        stats.total_rewards += reward;
                    }
                }
            }

            RewardMethod::PPS => {
                // Pay Per Share - fixed amount per share
                let per_share = miner_reward / self.current_round.len() as u64;
                
                for share in &self.current_round {
                    *rewards.entry(share.miner).or_insert(0) += per_share;
                    
                    if let Some(stats) = self.miners.get_mut(&share.miner) {
                        stats.pending_rewards += per_share;
                        stats.total_rewards += per_share;
                    }
                }
            }

            RewardMethod::PPLNS { window } => {
                // Pay Per Last N Shares
                let recent_shares: Vec<_> = self.current_round.iter()
                    .rev()
                    .take(window as usize)
                    .collect();

                let mut share_counts: HashMap<PublicKey, u64> = HashMap::new();
                for share in recent_shares {
                    *share_counts.entry(share.miner).or_insert(0) += 1;
                }

                let total = share_counts.values().sum::<u64>();
                for (miner, count) in share_counts {
                    let reward = (miner_reward * count) / total;
                    *rewards.entry(miner).or_insert(0) += reward;
                    
                    if let Some(stats) = self.miners.get_mut(&miner) {
                        stats.pending_rewards += reward;
                        stats.total_rewards += reward;
                    }
                }
            }
        }

        // Clear current round
        self.current_round.clear();
        self.blocks_mined += 1;

        rewards
    }

    /// Process payout for a miner
    pub fn process_payout(&mut self, miner: &PublicKey) -> Result<u64> {
        let stats = self.miners.get_mut(miner)
            .ok_or_else(|| PoolError::MinerNotFound(hex::encode(miner.0)))?;

        if stats.pending_rewards < self.config.min_payout {
            return Err(PoolError::InsufficientBalance);
        }

        let payout = stats.pending_rewards;
        stats.pending_rewards = 0;

        Ok(payout)
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Count active miners (submitted share in last 10 minutes)
        let active_miners = self.miners.values()
            .filter(|m| now - m.last_share_time < 600)
            .count();

        // Calculate total hashrate (simplified estimation)
        let pool_hashrate: f64 = self.miners.values()
            .filter(|m| now - m.last_share_time < 600)
            .map(|m| m.hashrate)
            .sum();

        PoolStats {
            active_miners,
            pool_hashrate,
            blocks_mined: self.blocks_mined,
            current_difficulty: self.config.share_difficulty,
            current_round_shares: self.current_round.len() as u64,
            pool_fee: self.config.fee_percent,
        }
    }

    /// Get miner statistics
    pub fn get_miner_stats(&self, miner: &PublicKey) -> Option<&MinerStats> {
        self.miners.get(miner)
    }

    /// Get all miners
    pub fn get_all_miners(&self) -> Vec<&MinerStats> {
        self.miners.values().collect()
    }

    /// Update miner hashrate estimation
    pub fn update_hashrate(&mut self, miner: &PublicKey, hashrate: f64) {
        if let Some(stats) = self.miners.get_mut(miner) {
            stats.hashrate = hashrate;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::crypto::KeyPair;

    #[test]
    fn test_pool_creation() {
        let config = PoolConfig::default();
        let pool = MiningPool::new(config);
        
        assert_eq!(pool.blocks_mined, 0);
        assert_eq!(pool.miners.len(), 0);
    }

    #[test]
    fn test_miner_registration() {
        let config = PoolConfig::default();
        let mut pool = MiningPool::new(config);
        
        let miner = KeyPair::generate().public_key();
        pool.register_miner(miner);
        
        assert_eq!(pool.miners.len(), 1);
        assert!(pool.miners.contains_key(&miner));
    }

    #[test]
    fn test_work_creation() {
        let config = PoolConfig::default();
        let mut pool = MiningPool::new(config);
        
        let work = pool.create_work(1, [0u8; 32], [0u8; 32], 16);
        
        assert_eq!(work.height, 1);
        assert_eq!(work.block_difficulty, 16);
        assert_eq!(work.share_difficulty, 12);
    }

    #[test]
    fn test_proportional_rewards() {
        let mut config = PoolConfig::default();
        config.reward_method = RewardMethod::Proportional;
        config.fee_percent = 2;
        
        let mut pool = MiningPool::new(config);
        
        let miner1 = KeyPair::generate().public_key();
        let miner2 = KeyPair::generate().public_key();
        
        pool.register_miner(miner1);
        pool.register_miner(miner2);
        
        // Miner1 submits 3 shares, Miner2 submits 1 share
        for _ in 0..3 {
            pool.current_round.push(Share {
                miner: miner1,
                height: 1,
                nonce: pool.current_round.len() as u64,
                hash: [0u8; 32],
                difficulty: 12,
                timestamp: 1234567890,
            });
        }
        
        pool.current_round.push(Share {
            miner: miner2,
            height: 1,
            nonce: 999,
            hash: [0u8; 32],
            difficulty: 12,
            timestamp: 1234567890,
        });
        
        let rewards = pool.distribute_rewards(1_000_000);
        
        // Pool fee: 2% = 20,000
        // Miner reward: 980,000
        // Miner1 (3/4): 735,000
        // Miner2 (1/4): 245,000
        
        assert_eq!(rewards.len(), 3); // operator + 2 miners
        assert_eq!(rewards.get(&miner1), Some(&735_000));
        assert_eq!(rewards.get(&miner2), Some(&245_000));
    }

    #[test]
    fn test_payout_threshold() {
        let mut config = PoolConfig::default();
        config.min_payout = 1_000_000; // 1 Lira minimum
        
        let mut pool = MiningPool::new(config);
        let miner = KeyPair::generate().public_key();
        
        pool.register_miner(miner);
        
        // Set pending rewards below threshold
        pool.miners.get_mut(&miner).unwrap().pending_rewards = 500_000;
        
        assert!(pool.process_payout(&miner).is_err());
        
        // Set pending rewards above threshold
        pool.miners.get_mut(&miner).unwrap().pending_rewards = 1_500_000;
        
        let payout = pool.process_payout(&miner).unwrap();
        assert_eq!(payout, 1_500_000);
        assert_eq!(pool.miners.get(&miner).unwrap().pending_rewards, 0);
    }
}
