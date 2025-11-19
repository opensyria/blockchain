# Digital Lira Tokenomics | اقتصاديات الليرة الرقمية

**OpenSyria Blockchain - Economic Specification**

**Version:** 1.0 (DRAFT - Requires Community Review)  
**Status:** 🚨 **CRITICAL - NOT IMPLEMENTED**  
**Last Updated:** November 18, 2025  
**Authors:** OpenSyria Core Team

---

## ⚠️ IMPLEMENTATION STATUS

**CRITICAL NOTICE:** This document defines the **intended** economic model for the Digital Lira. **NONE OF THESE PARAMETERS ARE CURRENTLY IMPLEMENTED IN CODE.** This is a blocking issue for production deployment.

**Required Actions:**
1. Community consensus on economic parameters
2. Code implementation of emission schedule
3. Coinbase transaction mechanism
4. Fee market design
5. Treasury allocation (if adopted)

---

## Executive Summary

The Digital Lira (الليرة الرقمية) is designed as a **deflationary, fixed-supply digital currency** with a Bitcoin-inspired emission schedule, adapted for Syrian cultural and economic context. The system balances long-term value preservation with sufficient early liquidity to bootstrap network security and ecosystem development.

### Key Parameters (Proposed)

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| **Maximum Supply** | 100,000,000 Lira | Symbolic: Syria's population (~21M) × 5 millennia of heritage |
| **Initial Block Reward** | 50 Lira | Sufficient for early miner incentives |
| **Block Time Target** | 120 seconds (2 minutes) | Faster than Bitcoin, more stable than Ethereum |
| **Halving Interval** | 210,000 blocks (~1 year) | Annual halvings for predictable scarcity |
| **Emission Duration** | ~26 years | After 2051, fees only (like Bitcoin after 2140) |
| **Minimum Transaction Fee** | 0.0001 Lira (100 units) | Low barrier for daily transactions |
| **Decimal Precision** | 6 decimals (1 Lira = 1,000,000 units) | Matches existing code |

---

## Supply Model

### Total Supply Calculation

```
Maximum Supply: 100,000,000 Lira (100 million)

Emission Schedule:
Year 1:  50 Lira/block × 262,800 blocks = 13,140,000 Lira (13.14%)
Year 2:  25 Lira/block × 262,800 blocks = 6,570,000 Lira
Year 3:  12.5 Lira/block × 262,800 blocks = 3,285,000 Lira
Year 4:  6.25 Lira/block × 262,800 blocks = 1,642,500 Lira
...
Year 26: ~0.0000019 Lira/block (minimum unit)

Total After 26 Years: ~99,999,999 Lira (99.99999%)
Remaining Supply: Issued via fees only (deflationary pressure)
```

### Emission Chart

| Halving # | Block Range | Reward (Lira) | Annual Issuance | % of Max Supply |
|-----------|-------------|---------------|-----------------|-----------------|
| 0 | 1 - 210,000 | 50.0 | 13,140,000 | 13.14% |
| 1 | 210,001 - 420,000 | 25.0 | 6,570,000 | 6.57% |
| 2 | 420,001 - 630,000 | 12.5 | 3,285,000 | 3.29% |
| 3 | 630,001 - 840,000 | 6.25 | 1,642,500 | 1.64% |
| 4 | 840,001 - 1,050,000 | 3.125 | 821,250 | 0.82% |
| 5 | 1,050,001 - 1,260,000 | 1.5625 | 410,625 | 0.41% |
| ... | ... | ... | ... | ... |
| 25 | 5,250,001 - 5,460,000 | 0.0000019 | 0.5 | 0.0000005% |

---

## Block Reward Implementation

### Code Specification

**File:** `crates/core/src/constants.rs` (TO BE CREATED)

```rust
/// Economic constants for Digital Lira
pub mod economics {
    /// Maximum total supply (1 Lira = 1_000_000 units)
    pub const MAX_SUPPLY: u64 = 100_000_000_000_000; // 100M Lira
    
    /// Initial block reward in smallest unit
    pub const INITIAL_BLOCK_REWARD: u64 = 50_000_000; // 50 Lira
    
    /// Block interval for reward halving
    pub const HALVING_INTERVAL: u64 = 210_000; // ~1 year at 2min/block
    
    /// Target block time in seconds
    pub const TARGET_BLOCK_TIME_SECS: u64 = 120; // 2 minutes
    
    /// Calculate block reward for given height
    pub fn calculate_block_reward(height: u64) -> u64 {
        if height == 0 {
            return 0; // Genesis has no reward
        }
        
        let halvings = (height - 1) / HALVING_INTERVAL;
        
        // After 64 halvings, reward is 0 (all bits shifted out)
        if halvings >= 64 {
            return 0;
        }
        
        // Right shift = divide by 2^halvings
        INITIAL_BLOCK_REWARD >> halvings
    }
    
    /// Calculate total supply issued up to given height
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
}

#[cfg(test)]
mod tests {
    use super::economics::*;
    
    #[test]
    fn test_initial_reward() {
        assert_eq!(calculate_block_reward(1), 50_000_000);
    }
    
    #[test]
    fn test_first_halving() {
        assert_eq!(calculate_block_reward(210_000), 50_000_000);
        assert_eq!(calculate_block_reward(210_001), 25_000_000);
    }
    
    #[test]
    fn test_max_supply_never_exceeded() {
        let supply_at_1m_blocks = total_supply_at_height(1_000_000);
        assert!(supply_at_1m_blocks <= MAX_SUPPLY);
    }
}
```

---

## Coinbase Transaction Design

### Structure

Every block must contain a **coinbase transaction** as its first transaction, creating new coins for the miner.

```rust
impl Transaction {
    /// Create coinbase transaction for miner reward
    pub fn coinbase(
        miner_address: PublicKey,
        block_height: u64,
        transaction_fees: u64,
    ) -> Result<Self, TransactionError> {
        let block_reward = economics::calculate_block_reward(block_height);
        let total_reward = block_reward.checked_add(transaction_fees)
            .ok_or(TransactionError::RewardOverflow)?;
        
        Ok(Self {
            from: PublicKey::zero(), // Special "coinbase" address
            to: miner_address,
            amount: total_reward,
            fee: 0, // Coinbase pays no fee
            nonce: block_height, // Use height as unique nonce
            signature: Vec::new(), // No signature required (network consensus validates)
            data: Some(Self::coinbase_data(block_height)),
        })
    }
    
    /// Coinbase metadata (block height + timestamp)
    fn coinbase_data(height: u64) -> Vec<u8> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut data = Vec::new();
        data.extend_from_slice(&height.to_le_bytes());
        data.extend_from_slice(&timestamp.to_le_bytes());
        data.extend_from_slice(b"Digital Lira | الليرة الرقمية");
        data
    }
    
    /// Check if transaction is coinbase
    pub fn is_coinbase(&self) -> bool {
        self.from == PublicKey::zero() && self.signature.is_empty()
    }
}
```

### Validation Rules

**Coinbase Transaction Must:**
1. Be the **first transaction** in every block
2. Have `from` = `PublicKey::zero()` (all zeros)
3. Have `amount` = `block_reward(height) + sum(tx.fee for tx in block)`
4. Have `nonce` = `block_height`
5. Have empty `signature` (not signed)
6. Be **exactly one** per block (no more, no less)

**Validation Code:**
```rust
impl Block {
    pub fn validate_coinbase(&self, height: u64) -> Result<(), BlockError> {
        // Must have at least one transaction (coinbase)
        if self.transactions.is_empty() {
            return Err(BlockError::MissingCoinbase);
        }
        
        let coinbase = &self.transactions[0];
        
        // First tx must be coinbase
        if !coinbase.is_coinbase() {
            return Err(BlockError::InvalidCoinbase);
        }
        
        // No other coinbase transactions
        for tx in &self.transactions[1..] {
            if tx.is_coinbase() {
                return Err(BlockError::MultipleCoinbase);
            }
        }
        
        // Validate coinbase amount
        let block_reward = economics::calculate_block_reward(height);
        let total_fees: u64 = self.transactions[1..]
            .iter()
            .map(|tx| tx.fee)
            .sum();
        
        let expected_amount = block_reward.checked_add(total_fees)
            .ok_or(BlockError::RewardOverflow)?;
        
        if coinbase.amount != expected_amount {
            return Err(BlockError::InvalidCoinbaseAmount);
        }
        
        Ok(())
    }
}
```

---

## Fee Market Design

### Fee Structure

**Minimum Fee Calculation:**
```
min_fee = BASE_FEE + (transaction_size_bytes × FEE_PER_BYTE)

BASE_FEE = 100 units (0.0001 Lira)
FEE_PER_BYTE = 10 units (0.00001 Lira)
```

**Example:**
- Simple transfer (250 bytes): 100 + (250 × 10) = 2,600 units (0.0026 Lira)
- With data (1KB): 100 + (1000 × 10) = 10,100 units (0.0101 Lira)

### Priority Sorting

Mempool sorts pending transactions by **fee density** (fee per byte):
```rust
fee_density = tx.fee / serialized_size(tx)
```

Higher fee density = higher priority for inclusion in next block.

### Fee Burning (Optional - Future Consideration)

**Option 1: No Burning** (Current Proposal)
- All fees go to miners
- Simple, predictable

**Option 2: Partial Burning** (EIP-1559 style)
- Base fee burned (deflationary)
- Tip goes to miner
- Requires more complex implementation

**Recommendation:** Start with Option 1, evaluate Option 2 after 1 year of mainnet operation.

---

## Treasury Allocation (Proposed)

### Background

To fund ongoing development, cultural preservation initiatives, and governance operations, a **community treasury** is proposed.

### Options for Community Decision

**Option A: No Treasury** (Pure Miner Rewards)
- Pros: Simplest, aligns with Bitcoin model
- Cons: No guaranteed funding for development

**Option B: 10% Treasury Tax** (Zcash-style)
- Miners receive 90% of block reward
- 10% goes to treasury address
- Requires governance for spending

**Option C: Premine Treasury** (One-time allocation)
- 5% of max supply (5M Lira) minted in genesis block
- Locked with 4-year vesting schedule
- Requires multi-signature control

### Proposed Implementation (Option B - Pending Approval)

```rust
pub const TREASURY_PERCENT: u64 = 10; // 10% of block reward

pub fn calculate_miner_reward(height: u64, fees: u64) -> (u64, u64) {
    let block_reward = calculate_block_reward(height);
    let treasury_amount = (block_reward * TREASURY_PERCENT) / 100;
    let miner_amount = block_reward - treasury_amount + fees; // Miner gets all fees
    
    (miner_amount, treasury_amount)
}

impl Transaction {
    pub fn coinbase_with_treasury(
        miner_address: PublicKey,
        treasury_address: PublicKey,
        block_height: u64,
        transaction_fees: u64,
    ) -> Result<Vec<Self>, TransactionError> {
        let (miner_amount, treasury_amount) = calculate_miner_reward(block_height, transaction_fees);
        
        let mut txs = Vec::new();
        
        // Miner coinbase
        txs.push(Self::coinbase(miner_address, block_height, miner_amount)?);
        
        // Treasury coinbase (if non-zero)
        if treasury_amount > 0 {
            txs.push(Self::coinbase(treasury_address, block_height, treasury_amount)?);
        }
        
        Ok(txs)
    }
}
```

**Treasury Address:** To be determined by governance vote (multi-signature account recommended).

---

## Inflation Analysis

### Year-by-Year Inflation Rate

| Year | Supply Start | New Issuance | Supply End | Inflation Rate |
|------|--------------|--------------|------------|----------------|
| 1 | 0 | 13,140,000 | 13,140,000 | ∞% (bootstrap) |
| 2 | 13,140,000 | 6,570,000 | 19,710,000 | 50.0% |
| 3 | 19,710,000 | 3,285,000 | 22,995,000 | 16.7% |
| 4 | 22,995,000 | 1,642,500 | 24,637,500 | 7.1% |
| 5 | 24,637,500 | 821,250 | 25,458,750 | 3.3% |
| 10 | 26,241,563 | 25,664 | 26,267,227 | 0.098% |
| 20 | 26,249,998 | 25 | 26,250,023 | 0.000095% |
| 26+ | ~26,250,000 | 0 | 26,250,000 | 0% (fee-only) |

**Note:** After year 5, inflation rate drops below typical central bank targets (2-3%), making Digital Lira deflationary in real terms.

---

## Comparison with Other Cryptocurrencies

| Feature | Digital Lira | Bitcoin | Ethereum | Syrian Pound (SYP) |
|---------|--------------|---------|----------|-------------------|
| **Max Supply** | 100M | 21M | Unlimited | Unlimited |
| **Block Time** | 2 min | 10 min | 12 sec | N/A |
| **Halving Interval** | 1 year | 4 years | N/A | N/A |
| **Emission Duration** | 26 years | 130+ years | Perpetual | Perpetual |
| **Current Inflation (Year 5)** | ~3.3% | ~1.8% | ~0.5% | ~200%+ (hyperinflation) |
| **Consensus** | PoW→PoS | PoW | PoS | Centralized |

**Rationale for Differences:**
- **Faster Halvings:** Accelerates scarcity timeline vs. Bitcoin
- **Larger Supply:** Lower per-coin price = easier psychological pricing ("1000 Lira" vs "0.001 BTC")
- **2-Minute Blocks:** Faster confirmations without Ethereum's complexity

---

## Economic Security Analysis

### Mining Security Budget

**Question:** Will fee revenue sustain network security after block rewards end?

**Year 26+ Scenario (Fee-Only Era):**
```
Assumptions:
- 1,000 transactions/day (conservative)
- Average fee: 0.01 Lira
- Block time: 2 minutes (720 blocks/day)

Daily Fee Revenue: 1,000 tx × 0.01 Lira = 10 Lira/day
Per-Block Fee Revenue: 10 Lira / 720 blocks = 0.0139 Lira/block

Compare to Year 1 Block Reward: 50 Lira/block
Ratio: 0.0278% of initial reward
```

**Risk:** Fee-only security may be insufficient.

**Mitigations:**
1. **Transaction Volume Growth:** 1M tx/day → 13.9 Lira/block (still only 27.8% of initial)
2. **Higher Fees:** Average fee of 0.1 Lira → 139 Lira/block (278% of initial) ✅
3. **PoS Migration:** Stake-based security less dependent on monetary rewards

**Recommendation:** Plan PoS migration before year 20 to ensure security sustainability.

---

## Comparison with Syrian Economy

### Context: Syrian Pound Hyperinflation

**Historical Data:**
- 2011: 1 USD = 47 SYP (pre-war)
- 2025: 1 USD = 13,000+ SYP (current, unofficial rate)
- Cumulative Inflation: 27,000%+

### Digital Lira Value Proposition

**Predictable Supply vs. Hyperinflation:**
```
Digital Lira Max Supply: 100,000,000 (hard cap in code)
Syrian Pound Supply: Unlimited (controlled by central bank)

Digital Lira Inflation (Year 10): 0.098%
Syrian Pound Inflation (2023): ~200%+ annual
```

**Use Case:** Store of value for diaspora communities, remittances, and long-term savings protected from central bank debasement.

---

## Distribution Fairness

### Launch Considerations

**Fair Launch Criteria:**
1. ✅ No premine (except optional treasury, pending governance)
2. ✅ Public genesis block with known timestamp
3. ✅ Open-source code (auditable)
4. ✅ No special founder allocation
5. ⚠️ Early miner advantage (Year 1 = 13.14% of supply)

**Mitigating Early Miner Advantage:**
- Public launch announcement (1 month notice)
- Mining pool support (democratizes hashrate)
- GUI miner for non-technical users
- Initial difficulty set to consumer hardware (no ASICs)

---

## Governance Economic Parameters

The following parameters should be adjustable via on-chain governance proposals:

**Immutable (Hard-Coded):**
- ✅ Maximum supply (100M)
- ✅ Halving interval (210,000 blocks)
- ✅ Initial block reward (50 Lira)

**Governable (Via Proposals):**
- ⚙️ Minimum transaction fee
- ⚙️ Fee-per-byte rate
- ⚙️ Block size limit
- ⚙️ Treasury allocation percentage (if adopted)
- ⚙️ Fee burning mechanism (if adopted)

---

## Implementation Roadmap

### Phase 1: Core Economic Logic (Week 1-2)
- [ ] Create `constants.rs` with all economic parameters
- [ ] Implement `calculate_block_reward()`
- [ ] Implement `total_supply_at_height()`
- [ ] Add comprehensive tests (100 years simulation)

### Phase 2: Coinbase Transactions (Week 2-3)
- [ ] Implement `Transaction::coinbase()`
- [ ] Update `Block` to require coinbase as first tx
- [ ] Add `Block::validate_coinbase()`
- [ ] Update mining logic to create coinbase
- [ ] Test reward halving across 10 halvings

### Phase 3: Fee Market (Week 3-4)
- [ ] Implement minimum fee calculation
- [ ] Update mempool to sort by fee density
- [ ] Add fee validation to transaction verification
- [ ] Test mempool prioritization

### Phase 4: Treasury (Week 4-5 - If Adopted)
- [ ] Community governance vote on treasury model
- [ ] Implement chosen treasury allocation
- [ ] Create multi-signature treasury account
- [ ] Test treasury coinbase generation

### Phase 5: Testing & Audit (Week 6-8)
- [ ] Economic simulation (10-year projection)
- [ ] Security audit of reward calculations
- [ ] Testnet deployment with real mining
- [ ] Monitor for edge cases

---

## Open Questions for Community Input

1. **Treasury Allocation:**
   - Should we implement a treasury? If yes, what percentage (0%, 5%, 10%, 15%)?
   - Who controls treasury spending (governance, foundation, DAO)?

2. **Fee Burning:**
   - Should we burn a portion of transaction fees (deflationary)?
   - If yes, what percentage (0%, 25%, 50%, 100%)?

3. **Block Time:**
   - Is 2 minutes optimal, or should we target 1 minute (faster) or 5 minutes (more stable)?

4. **Maximum Supply:**
   - Is 100M Lira the right cap, or should it be 21M (Bitcoin parity) or 1B (lower unit price)?

5. **PoS Transition Timeline:**
   - When should we migrate to Proof-of-Stake (Year 5, Year 10, Year 20)?
   - What should be the staking reward rate?

---

## Community Feedback Process

**How to Participate:**
1. **GitHub Discussions:** Open issues on `opensyria/blockchain` repository
2. **Governance Proposals:** Submit formal proposals via on-chain voting (once implemented)
3. **Community Calls:** Monthly tokenomics review meetings (details TBD)

**Deadline for Feedback:** Before mainnet genesis block is mined (target: Q2 2026)

---

## References

**Similar Economic Models:**
- Bitcoin: https://bitcoin.org/bitcoin.pdf (Section 6: Incentive)
- Zcash: https://z.cash/technology/paramgen/ (Dev fund model)
- Monero: https://www.getmonero.org/resources/moneropedia/tail-emission.html (Tail emission alternative)

**Economic Analysis Tools:**
- Supply simulation spreadsheet: `docs/economics/supply_model.xlsx` (TO BE CREATED)
- Inflation calculator: `scripts/calculate_inflation.py` (TO BE CREATED)

---

## Document History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0 | Nov 18, 2025 | Initial draft based on audit findings | Audit Team |

---

## Appendix A: Code Integration Checklist

**Files to Create:**
- [ ] `crates/core/src/constants.rs` - Economic constants
- [ ] `crates/core/src/economics.rs` - Reward calculation functions
- [ ] `docs/economics/supply_model.xlsx` - Supply projections
- [ ] `scripts/calculate_inflation.py` - Inflation calculator

**Files to Modify:**
- [ ] `crates/core/src/transaction.rs` - Add coinbase methods
- [ ] `crates/core/src/block.rs` - Add coinbase validation
- [ ] `crates/consensus/src/pow.rs` - Generate coinbase in mining
- [ ] `crates/storage/src/state.rs` - Track total supply
- [ ] `crates/mempool/src/lib.rs` - Fee-based priority sorting
- [ ] `crates/node-cli/src/main.rs` - Display economic stats

**Tests to Add:**
- [ ] `economics::test_reward_halving_schedule` (26 halvings)
- [ ] `economics::test_max_supply_convergence` (1M blocks)
- [ ] `coinbase::test_reward_plus_fees` (edge cases)
- [ ] `fees::test_priority_sorting` (mempool)

---

**Status:** 🚨 **This document requires implementation before any production deployment.**

**Contact:** opensyria.net@gmail.com
