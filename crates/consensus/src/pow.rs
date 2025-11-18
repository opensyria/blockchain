use opensyria_core::Block;
use std::time::{Duration, Instant};

/// Mining statistics
#[derive(Debug, Clone)]
pub struct MiningStats {
    pub hashes_computed: u64,
    pub duration: Duration,
    pub hash_rate: f64, // hashes per second
    pub nonce_found: u64,
}

/// Proof-of-Work consensus implementation
pub struct ProofOfWork {
    difficulty: u32,
}

impl ProofOfWork {
    pub fn new(difficulty: u32) -> Self {
        Self { difficulty }
    }

    /// Get current difficulty
    pub fn difficulty(&self) -> u32 {
        self.difficulty
    }

    /// Set new difficulty
    pub fn set_difficulty(&mut self, difficulty: u32) {
        self.difficulty = difficulty;
    }

    /// Mine a block by finding valid nonce, returns block and stats
    pub fn mine(&self, mut block: Block) -> (Block, MiningStats) {
        block.header.difficulty = self.difficulty;
        let start = Instant::now();
        let mut hashes = 0u64;
        
        for nonce in 0..u64::MAX {
            block.header.nonce = nonce;
            hashes += 1;
            
            if block.header.meets_difficulty() {
                let duration = start.elapsed();
                let hash_rate = hashes as f64 / duration.as_secs_f64();
                
                let stats = MiningStats {
                    hashes_computed: hashes,
                    duration,
                    hash_rate,
                    nonce_found: nonce,
                };
                
                return (block, stats);
            }
            
            // Progress reporting every 100k hashes
            if hashes % 100_000 == 0 {
                let elapsed = start.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    let rate = hashes as f64 / elapsed;
                    tracing::debug!(
                        "Mining progress: {} hashes, {:.2} H/s",
                        hashes,
                        rate
                    );
                }
            }
        }
        
        // Should never reach here unless difficulty is impossibly high
        panic!("Exhausted nonce space without finding valid block");
    }

    /// Mine with callback for progress updates
    pub fn mine_with_callback<F>(&self, mut block: Block, mut callback: F) -> (Block, MiningStats)
    where
        F: FnMut(u64, f64), // (hashes, hash_rate)
    {
        block.header.difficulty = self.difficulty;
        let start = Instant::now();
        let mut hashes = 0u64;
        
        for nonce in 0..u64::MAX {
            block.header.nonce = nonce;
            hashes += 1;
            
            if block.header.meets_difficulty() {
                let duration = start.elapsed();
                let hash_rate = hashes as f64 / duration.as_secs_f64();
                
                let stats = MiningStats {
                    hashes_computed: hashes,
                    duration,
                    hash_rate,
                    nonce_found: nonce,
                };
                
                return (block, stats);
            }
            
            // Progress callback every 50k hashes
            if hashes % 50_000 == 0 {
                let elapsed = start.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    let rate = hashes as f64 / elapsed;
                    callback(hashes, rate);
                }
            }
        }
        
        panic!("Exhausted nonce space without finding valid block");
    }

    /// Validate block meets difficulty requirement
    pub fn validate(&self, block: &Block) -> bool {
        block.header.difficulty == self.difficulty && block.header.meets_difficulty()
    }

    /// Calculate expected time to mine at given hash rate
    pub fn expected_time_seconds(&self, hash_rate: f64) -> f64 {
        let target_space = 2u128.pow(256);
        let difficulty_bits = self.difficulty;
        
        // Simplified: each bit of difficulty halves the target space
        let reduced_space = target_space >> difficulty_bits;
        let expected_hashes = reduced_space as f64;
        
        expected_hashes / hash_rate
    }
}

/// Difficulty adjustment based on actual vs target block time
pub struct DifficultyAdjuster {
    target_block_time: Duration,
    adjustment_interval: u32, // blocks between adjustments
}

impl DifficultyAdjuster {
    pub fn new(target_block_time_secs: u64, adjustment_interval: u32) -> Self {
        Self {
            target_block_time: Duration::from_secs(target_block_time_secs),
            adjustment_interval,
        }
    }

    /// Calculate new difficulty based on actual mining times
    pub fn adjust(&self, current_difficulty: u32, actual_time: Duration) -> u32 {
        let target_total = self.target_block_time.as_secs_f64() * self.adjustment_interval as f64;
        let actual_total = actual_time.as_secs_f64();
        
        if actual_total == 0.0 {
            return current_difficulty;
        }
        
        let ratio = actual_total / target_total;
        
        // Adjust difficulty based on time ratio
        // If ratio > 1, blocks took longer → decrease difficulty
        // If ratio < 1, blocks were faster → increase difficulty
        let adjustment_factor = if ratio > 1.0 {
            // Too slow, decrease difficulty (subtract)
            -((current_difficulty as f64 * (ratio - 1.0).min(0.25)) as i32)
        } else {
            // Too fast, increase difficulty (add)
            ((current_difficulty as f64 * (1.0 - ratio).min(0.25)) as i32)
        };
        
        let new_difficulty = (current_difficulty as i32 + adjustment_factor).max(8) as u32;
        
        // Clamp to reasonable range (8-bit to 24-bit difficulty)
        new_difficulty.clamp(8, 192)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::{crypto::KeyPair, transaction::Transaction};

    #[test]
    fn test_mine_genesis_block() {
        let pow = ProofOfWork::new(8); // Easy difficulty for testing
        let genesis = Block::genesis(8);
        
        let (mined, stats) = pow.mine(genesis);
        
        assert!(pow.validate(&mined));
        assert!(stats.hashes_computed > 0);
        assert!(stats.hash_rate > 0.0);
    }

    #[test]
    fn test_mine_block_with_transactions() {
        let pow = ProofOfWork::new(8);
        
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
        
        let block = Block::new([0u8; 32], vec![tx], 8);
        let (mined, _stats) = pow.mine(block);
        
        assert!(pow.validate(&mined));
        assert_eq!(mined.transactions.len(), 1);
        assert!(mined.verify_merkle_root());
    }

    #[test]
    fn test_validation_rejects_insufficient_difficulty() {
        let pow = ProofOfWork::new(16);
        
        // Mine with easier difficulty
        let easy_pow = ProofOfWork::new(8);
        let genesis = Block::genesis(8);
        let (mined, _) = easy_pow.mine(genesis);
        
        // Should fail validation with harder difficulty requirement
        assert!(!pow.validate(&mined));
    }

    #[test]
    fn test_difficulty_adjustment_increase() {
        let adjuster = DifficultyAdjuster::new(60, 10); // 60s target, adjust every 10 blocks
        
        // Blocks mined too fast (5 minutes instead of 10)
        let actual_time = Duration::from_secs(300);
        let new_difficulty = adjuster.adjust(16, actual_time);
        
        // Should increase difficulty
        assert!(new_difficulty > 16);
    }

    #[test]
    fn test_difficulty_adjustment_decrease() {
        let adjuster = DifficultyAdjuster::new(60, 10);
        
        // Blocks mined too slow (20 minutes instead of 10)
        let actual_time = Duration::from_secs(1200);
        let new_difficulty = adjuster.adjust(16, actual_time);
        
        // Should decrease difficulty
        assert!(new_difficulty < 16);
    }
}
