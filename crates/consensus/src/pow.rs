use opensyria_core::{Block, DIFFICULTY_ADJUSTMENT_INTERVAL, MAX_DIFFICULTY, MAX_DIFFICULTY_ADJUSTMENT, MIN_DIFFICULTY, TARGET_BLOCK_TIME_SECS};
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
            if hashes.is_multiple_of(100_000) {
                let elapsed = start.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    let rate = hashes as f64 / elapsed;
                    tracing::debug!("Mining progress: {} hashes, {:.2} H/s", hashes, rate);
                }
            }
        }

        // Nonce space exhausted - caller should increment timestamp and retry
        // This is extremely rare (difficulty impossibly high or hash function broken)
        tracing::warn!(
            "Exhausted nonce space (2^64 attempts) without finding valid block at difficulty {}",
            self.difficulty
        );
        
        // Return the block with max nonce - caller should detect failure via meets_difficulty()
        // and increment timestamp to get new hash space
        block.header.nonce = u64::MAX;
        let duration = start.elapsed();
        let hash_rate = hashes as f64 / duration.as_secs_f64();
        
        let stats = MiningStats {
            hashes_computed: hashes,
            duration,
            hash_rate,
            nonce_found: u64::MAX, // Indicates exhaustion
        };
        
        (block, stats)
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

            // Progress update every 50k hashes
            if hashes.is_multiple_of(50_000) {
                let elapsed = start.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    let rate = hashes as f64 / elapsed;
                    callback(hashes, rate);
                }
            }
        }

        // Nonce space exhausted
        tracing::warn!(
            "Exhausted nonce space (2^64 attempts) in mine_with_callback at difficulty {}",
            self.difficulty
        );
        
        block.header.nonce = u64::MAX;
        let duration = start.elapsed();
        let hash_rate = hashes as f64 / duration.as_secs_f64();
        
        let stats = MiningStats {
            hashes_computed: hashes,
            duration,
            hash_rate,
            nonce_found: u64::MAX,
        };
        
        (block, stats)
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

    /// Create adjuster with default protocol parameters
    pub fn default() -> Self {
        Self::new(TARGET_BLOCK_TIME_SECS, DIFFICULTY_ADJUSTMENT_INTERVAL)
    }

    /// Calculate new difficulty based on actual mining times
    /// Uses integer arithmetic to prevent floating-point accumulation errors
    /// حساب الصعوبة الجديدة بناءً على أوقات التعدين الفعلية
    pub fn adjust(&self, current_difficulty: u32, actual_time: Duration, block_count: u32) -> u32 {
        // Wait for full adjustment interval before adjusting
        if block_count < self.adjustment_interval {
            return current_difficulty;
        }

        let target_total = self.target_block_time.as_secs() * block_count as u64;
        let actual_total = actual_time.as_secs();
        
        // SECURITY: Detect anomalous time values that could indicate attack
        if actual_total == 0 {
            tracing::warn!("Difficulty adjustment received zero actual_time - possible timewarp attack");
            return current_difficulty; // No adjustment on anomaly
        }

        // Integer-only calculation (Bitcoin-style) to avoid floating-point errors
        // Use u128 to prevent overflow during multiplication
        let new_difficulty = (current_difficulty as u128 * target_total as u128 
                             / actual_total as u128) as u32;

        // SECURITY: Use integer-only clamping to avoid float precision issues
        // Clamp adjustment to ±25% (MAX_DIFFICULTY_ADJUSTMENT)
        let adjustment_factor_num = (MAX_DIFFICULTY_ADJUSTMENT * 1000.0) as u32; // 250 for 25%
        let adjustment_factor_den = 1000u32;
        
        let decrease_amount = (current_difficulty as u64 * adjustment_factor_num as u64) / adjustment_factor_den as u64;
        let increase_amount = (current_difficulty as u64 * adjustment_factor_num as u64) / adjustment_factor_den as u64;
        
        let min_diff = (current_difficulty as u64).saturating_sub(decrease_amount).max(MIN_DIFFICULTY as u64) as u32;
        let max_diff = (current_difficulty as u64).saturating_add(increase_amount).min(MAX_DIFFICULTY as u64) as u32;

        new_difficulty.clamp(min_diff, max_diff)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opensyria_core::{crypto::KeyPair, transaction::Transaction};

    #[test]
    fn test_mine_genesis_block() {
        let pow = ProofOfWork::new(8); // Easy difficulty for testing
        let genesis = Block::genesis();
        let mut test_block = genesis.clone();
        test_block.header.difficulty = 8; // Override for test

        let (mined, stats) = pow.mine(test_block);

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
        let genesis = Block::genesis();
        let mut test_block = genesis.clone();
        test_block.header.difficulty = 8;
        let (mined, _) = easy_pow.mine(test_block);

        // Should fail validation with harder difficulty requirement
        assert!(!pow.validate(&mined));
    }

    #[test]
    fn test_difficulty_adjustment_increase() {
        let adjuster = DifficultyAdjuster::new(60, 10); // 60s target, adjust every 10 blocks

        // Blocks mined too fast (5 minutes instead of 10)
        let actual_time = Duration::from_secs(300);
        let new_difficulty = adjuster.adjust(16, actual_time, 10);

        // Should increase difficulty
        assert!(new_difficulty > 16);
    }

    #[test]
    fn test_difficulty_adjustment_decrease() {
        let adjuster = DifficultyAdjuster::new(60, 10);

        // Blocks mined too slow (20 minutes instead of 10)
        let actual_time = Duration::from_secs(1200);
        let new_difficulty = adjuster.adjust(16, actual_time, 10);

        // Should decrease difficulty
        assert!(new_difficulty < 16);
    }

    #[test]
    fn test_difficulty_adjustment_waits_for_interval() {
        let adjuster = DifficultyAdjuster::new(60, 10);

        // Only 5 blocks mined, should not adjust yet
        let actual_time = Duration::from_secs(300);
        let new_difficulty = adjuster.adjust(16, actual_time, 5);

        // Should keep same difficulty
        assert_eq!(new_difficulty, 16);
    }

    #[test]
    fn test_difficulty_adjustment_clamped() {
        let adjuster = DifficultyAdjuster::new(60, 10);

        // Extremely fast blocks (should clamp to max 25% increase)
        let actual_time = Duration::from_secs(10); // 10x faster
        let new_difficulty = adjuster.adjust(16, actual_time, 10);

        // Should not increase more than 25%
        assert!(new_difficulty <= 20); // 16 * 1.25 = 20
    }
}
