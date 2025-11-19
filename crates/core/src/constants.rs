// Protocol Constants for OpenSyria Blockchain
// الثوابت البروتوكولية لبلوك تشين سوريا المفتوحة

/// Chain identifier for mainnet
pub const CHAIN_ID_MAINNET: u32 = 963; // Syria country code +963

/// Chain identifier for testnet
pub const CHAIN_ID_TESTNET: u32 = 963_000; // Testnet variant

/// Target block time in seconds (2 minutes)
/// وقت الكتلة المستهدف بالثواني (دقيقتان)
pub const TARGET_BLOCK_TIME_SECS: u64 = 120;

/// Number of blocks between difficulty adjustments
/// عدد الكتل بين تعديلات الصعوبة
/// Increased from 10 to 100 blocks to reduce difficulty oscillation (audit recommendation A1)
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u32 = 100;

/// Maximum reorganization depth (prevents deep chain rewrites)
/// الحد الأقصى لعمق إعادة التنظيم (يمنع إعادة كتابة السلسلة العميقة)
/// Prevents 51% attacks from rewriting entire blockchain history
pub const MAX_REORG_DEPTH: u64 = 100;

/// Genesis block timestamp (fixed for deterministic genesis)
/// November 18, 2025 00:00:00 UTC - Official Launch Date
pub const GENESIS_TIMESTAMP: u64 = 1763452800;

/// Genesis block difficulty (16 bits = 2 leading zero bytes)
pub const GENESIS_DIFFICULTY: u32 = 16;

/// Genesis block nonce (pre-mined)
pub const GENESIS_NONCE: u64 = 0xDEADBEEF;

/// Maximum future time drift allowed for block timestamps (5 minutes)
/// الانحراف الزمني المستقبلي المسموح للطوابع الزمنية (5 دقائق)
pub const MAX_FUTURE_DRIFT_SECS: u64 = 300;

/// Minimum difficulty (8 bits = 1 leading zero byte)
pub const MIN_DIFFICULTY: u32 = 8;

/// Maximum difficulty (192 bits = 24 leading zero bytes)
pub const MAX_DIFFICULTY: u32 = 192;

/// Maximum difficulty adjustment factor (25%)
pub const MAX_DIFFICULTY_ADJUSTMENT: f64 = 0.25;

/// Protocol version
pub const PROTOCOL_VERSION: u32 = 1;

// ============================================================================
// Economic Constants | الثوابت الاقتصادية
// ============================================================================

/// Maximum total supply in base units (1 Lira = 1,000,000 units)
/// الإمداد الأقصى الإجمالي بالوحدات الأساسية
/// 100 million Lira - symbolic of Syria's population × heritage
pub const MAX_SUPPLY: u64 = 100_000_000_000_000; // 100M Lira

/// Initial block reward in base units (50 Lira)
/// مكافأة الكتلة الأولية بالوحدات الأساسية
pub const INITIAL_BLOCK_REWARD: u64 = 50_000_000;

/// Block interval for reward halving (~1 year at 2min/block)
/// فترة الكتل لتنصيف المكافأة
pub const HALVING_INTERVAL: u64 = 210_000;

/// Units per Lira (6 decimal places)
pub const UNITS_PER_LIRA: u64 = 1_000_000;

/// Minimum transaction fee (0.0001 Lira = 100 units)
/// رسوم المعاملة الدنيا
pub const MIN_TRANSACTION_FEE: u64 = 100;

/// Maximum transaction size in bytes (100 KB)
pub const MAX_TRANSACTION_SIZE: usize = 100_000;

/// Maximum block size in bytes (1 MB)
pub const MAX_BLOCK_SIZE: usize = 1_000_000;

/// Maximum transactions per block
pub const MAX_TRANSACTIONS_PER_BLOCK: usize = 1000;

/// Calculate block reward for given height
/// Uses right-shift for efficient halving (divide by 2^halvings)
/// حساب مكافأة الكتلة للارتفاع المحدد
pub fn calculate_block_reward(height: u64) -> u64 {
    if height == 0 {
        return 0; // Genesis has no reward
    }

    let halvings = (height - 1) / HALVING_INTERVAL;

    // After 64 halvings, reward is 0 (all bits shifted out)
    if halvings >= 64 {
        return 0;
    }

    // Right shift = divide by 2^halvings (efficient integer division)
    INITIAL_BLOCK_REWARD >> halvings
}

/// Calculate total supply issued up to given height
/// حساب الإمداد الإجمالي المصدر حتى الارتفاع المحدد
pub fn total_supply_at_height(height: u64) -> u64 {
    let mut total = 0u64;
    let mut current_height = 1u64;

    while current_height <= height {
        let reward = calculate_block_reward(current_height);
        if reward == 0 {
            break; // No more rewards
        }

        let remaining_in_era = HALVING_INTERVAL - ((current_height - 1) % HALVING_INTERVAL);
        let blocks_to_count = remaining_in_era.min(height - current_height + 1);

        total = total.saturating_add(reward.saturating_mul(blocks_to_count));
        current_height += blocks_to_count;
    }

    total.min(MAX_SUPPLY)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_block_reward() {
        assert_eq!(calculate_block_reward(1), 50_000_000); // 50 Lira
        assert_eq!(calculate_block_reward(100), 50_000_000);
        assert_eq!(calculate_block_reward(210_000), 50_000_000);
    }

    #[test]
    fn test_first_halving() {
        // Last block of era 0
        assert_eq!(calculate_block_reward(210_000), 50_000_000);
        // First block of era 1 (halved)
        assert_eq!(calculate_block_reward(210_001), 25_000_000);
    }

    #[test]
    fn test_second_halving() {
        assert_eq!(calculate_block_reward(420_001), 12_500_000);
    }

    #[test]
    fn test_reward_eventually_zero() {
        // After many halvings, reward becomes 0
        let far_future = 64 * HALVING_INTERVAL + 1;
        assert_eq!(calculate_block_reward(far_future), 0);
    }

    #[test]
    fn test_total_supply_year_one() {
        // Year 1: blocks at 50 Lira each
        // 120 seconds per block = 2 minutes
        // 365.25 days * 24 hours * 60 minutes / 2 = 262,800 blocks
        // But halving happens at 210,000 blocks (~0.8 years)
        // So first year has 210,000 blocks at 50 Lira + 52,800 blocks at 25 Lira
        let year_one_blocks = 262_800;
        let supply = total_supply_at_height(year_one_blocks);
        
        // First 210,000 blocks: 210,000 * 50 = 10,500,000 Lira
        // Next 52,800 blocks: 52,800 * 25 = 1,320,000 Lira
        // Total: 11,820,000 Lira
        assert_eq!(supply, 11_820_000_000_000);
    }

    #[test]
    fn test_max_supply_never_exceeded() {
        let supply_at_1m_blocks = total_supply_at_height(1_000_000);
        assert!(supply_at_1m_blocks <= MAX_SUPPLY);
        
        // Even at absurdly high heights
        let supply_at_100m_blocks = total_supply_at_height(100_000_000);
        assert!(supply_at_100m_blocks <= MAX_SUPPLY);
    }

    #[test]
    fn test_genesis_has_no_reward() {
        assert_eq!(calculate_block_reward(0), 0);
    }
}

