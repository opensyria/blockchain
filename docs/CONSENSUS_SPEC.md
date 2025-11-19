# Consensus Specification | مواصفات الإجماع

**Open Syria Blockchain - Protocol Consensus Rules**

**Version:** 1.0 (CANONICAL SPECIFICATION)  
**Status:** 🚨 **DRAFT - Requires Implementation**  
**Last Updated:** November 18, 2025  
**Chain ID:** 5963 (0x1753 - "SY" in hex)

---

## Document Purpose

This specification defines the **canonical consensus rules** for the Open Syria blockchain. All node implementations MUST follow these rules to maintain network consensus. Deviations will result in chain splits.

**Scope:**
- Block structure and validation
- Proof-of-Work algorithm
- Difficulty adjustment mechanism
- Transaction validation rules
- Chain selection (fork choice)
- Protocol constants

**Out of Scope (See Other Docs):**
- Economic parameters (see `TOKENOMICS.md`)
- Network protocol (see `docs/network/P2P_PROTOCOL.md`)
- Governance (see `docs/governance/GOVERNANCE_SPEC.md`)

---

## Protocol Constants

### Network Identifiers

```rust
/// Mainnet chain ID (prevents replay attacks across networks)
pub const MAINNET_CHAIN_ID: u32 = 5963; // "SY" = 0x1753 in hex

/// Testnet chain ID
pub const TESTNET_CHAIN_ID: u32 = 5964;

/// Protocol version (for soft/hard fork coordination)
pub const PROTOCOL_VERSION: u32 = 1;
```

### Timing Parameters

```rust
/// Target time between blocks (seconds)
pub const TARGET_BLOCK_TIME_SECS: u64 = 120; // 2 minutes

/// Maximum allowed drift between block timestamp and system time
pub const MAX_FUTURE_DRIFT_SECS: u64 = 300; // 5 minutes

/// Minimum timestamp (must be after previous block)
pub const MIN_PAST_DRIFT_SECS: u64 = 0; // No backward time travel

/// Blocks per difficulty adjustment period
pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u32 = 10; // Adjust every 10 blocks
```

### Size Limits

```rust
/// Maximum size of a single transaction (bytes)
pub const MAX_TRANSACTION_SIZE: usize = 100_000; // 100 KB

/// Maximum size of a block (bytes)
pub const MAX_BLOCK_SIZE: usize = 1_000_000; // 1 MB

/// Maximum number of transactions per block
pub const MAX_TRANSACTIONS_PER_BLOCK: usize = 1000;

/// Maximum length of transaction data field (bytes)
pub const MAX_TRANSACTION_DATA_SIZE: usize = 80_000; // 80 KB
```

### Difficulty Parameters

```rust
/// Initial difficulty for genesis block (bits of leading zeros)
pub const GENESIS_DIFFICULTY: u32 = 16; // 2 leading zero bytes

/// Minimum allowed difficulty
pub const MIN_DIFFICULTY: u32 = 8; // 1 leading zero byte

/// Maximum allowed difficulty
pub const MAX_DIFFICULTY: u32 = 192; // 24 leading zero bytes

/// Maximum difficulty adjustment per interval (percentage)
pub const MAX_DIFFICULTY_CHANGE_PERCENT: u32 = 25; // ±25%
```

### Economic Parameters (Cross-Reference)

```rust
/// See TOKENOMICS.md for full economic specification
pub const INITIAL_BLOCK_REWARD: u64 = 50_000_000; // 50 Lira
pub const HALVING_INTERVAL: u64 = 210_000; // blocks
pub const MIN_TRANSACTION_FEE: u64 = 100; // 0.0001 Lira
pub const FEE_PER_BYTE: u64 = 10; // 0.00001 Lira
```

---

## Genesis Block Specification

### Canonical Genesis Block

**CRITICAL:** All nodes MUST use this exact genesis block. Any deviation will create an incompatible chain.

```rust
pub fn canonical_genesis() -> Block {
    Block {
        header: BlockHeader {
            version: 1,
            previous_hash: [0u8; 32],
            merkle_root: [0u8; 32], // Empty merkle root (no transactions)
            timestamp: 1700000000, // Nov 14, 2023, 22:13:20 UTC (symbolic)
            difficulty: 16,
            nonce: 0, // To be pre-mined before mainnet launch
        },
        transactions: vec![
            // Coinbase transaction (to be added)
        ],
    }
}
```

**Genesis Block Parameters:**
- **Timestamp:** 1700000000 (November 14, 2023, 22:13:20 UTC)
  - Symbolic: Represents Syrian resilience and hope for the future
- **Difficulty:** 16 (2 leading zero bytes)
- **Previous Hash:** All zeros (no parent)
- **Nonce:** TBD (will be pre-mined and published before mainnet launch)

**Genesis Block Hash (Mainnet):** TBD (calculate after finalizing genesis coinbase)

**Testnet Genesis:**
- Use different timestamp: 1700001000 (Nov 14, 2023, 22:30:00 UTC)
- Lower difficulty: 8 (easier for testing)

---

## Block Structure

### Block Header

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Protocol version (enables soft forks)
    pub version: u32,
    
    /// Hash of previous block header
    pub previous_hash: [u8; 32],
    
    /// Merkle root of all transactions in block
    pub merkle_root: [u8; 32],
    
    /// Block timestamp (Unix epoch seconds)
    pub timestamp: u64,
    
    /// Difficulty target (bits of leading zeros required)
    pub difficulty: u32,
    
    /// Proof-of-work nonce
    pub nonce: u64,
}
```

**Header Hash Calculation:**
```rust
impl BlockHeader {
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.version.to_le_bytes());
        hasher.update(self.previous_hash);
        hasher.update(self.merkle_root);
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.difficulty.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.finalize().into()
    }
}
```

**Serialization:** Little-endian byte order for all integer fields (cross-platform compatibility).

### Complete Block

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}
```

**Invariants:**
1. First transaction MUST be coinbase (creates new coins)
2. All other transactions MUST NOT be coinbase
3. Merkle root MUST match `calculate_merkle_root(transactions)`
4. Block size MUST NOT exceed `MAX_BLOCK_SIZE`
5. Transaction count MUST NOT exceed `MAX_TRANSACTIONS_PER_BLOCK`

---

## Proof-of-Work Algorithm

### Target Calculation

**Difficulty Encoding:** Number of leading zero bits required in block hash.

```
Difficulty = 16 → Requires 16 leading zero bits (2 full bytes)
Difficulty = 24 → Requires 24 leading zero bits (3 full bytes)
```

**Valid Hash Check:**
```rust
impl BlockHeader {
    pub fn meets_difficulty(&self) -> bool {
        let hash = self.hash();
        let leading_zeros = self.difficulty / 8;
        let remainder = self.difficulty % 8;
        
        // Check full zero bytes
        for &byte in hash.iter().take(leading_zeros as usize) {
            if byte != 0 {
                return false;
            }
        }
        
        // Check partial byte (if remainder > 0)
        if remainder > 0 {
            let byte = hash[leading_zeros as usize];
            let mask = 0xFF << (8 - remainder);
            if byte & mask != 0 {
                return false;
            }
        }
        
        true
    }
}
```

**Example:**
```
Difficulty 16:
Hash must start with: 0x0000XXXXXXXXXXXX... (2 zero bytes)

Difficulty 24:
Hash must start with: 0x000000XXXXXXXXXXXX... (3 zero bytes)
```

### Mining Algorithm

**Nonce Search:**
```rust
pub fn mine_block(mut block: Block, difficulty: u32) -> (Block, MiningStats) {
    block.header.difficulty = difficulty;
    let start = Instant::now();
    let mut hashes = 0u64;
    
    for nonce in 0..u64::MAX {
        block.header.nonce = nonce;
        hashes += 1;
        
        if block.header.meets_difficulty() {
            let duration = start.elapsed();
            let hash_rate = hashes as f64 / duration.as_secs_f64();
            
            return (block, MiningStats {
                hashes_computed: hashes,
                duration,
                hash_rate,
                nonce_found: nonce,
            });
        }
    }
    
    panic!("Nonce space exhausted (should never happen with proper difficulty)");
}
```

**Nonce Exhaustion Handling:**
- Probability of exhausting 2^64 nonce space: negligible at difficulty < 64
- If exhausted: Increment timestamp by 1 second and retry
- Miners SHOULD randomize initial nonce to avoid collision

---

## Difficulty Adjustment

### Adjustment Algorithm

**Goal:** Maintain `TARGET_BLOCK_TIME_SECS` (120 seconds) average block time.

**Adjustment Frequency:** Every `DIFFICULTY_ADJUSTMENT_INTERVAL` (10) blocks.

**Calculation (Integer Arithmetic Only):**
```rust
pub fn adjust_difficulty(
    current_difficulty: u32,
    actual_time: Duration,
    block_count: u32,
) -> u32 {
    // Only adjust at full intervals
    if block_count < DIFFICULTY_ADJUSTMENT_INTERVAL {
        return current_difficulty;
    }
    
    // Calculate expected vs actual time
    let target_secs = TARGET_BLOCK_TIME_SECS * block_count as u64;
    let actual_secs = actual_time.as_secs().max(1); // Prevent division by zero
    
    // Integer-only calculation (avoid floating point)
    // new_difficulty = current * target / actual
    let new_difficulty = (current_difficulty as u128 * target_secs as u128 
                         / actual_secs as u128) as u32;
    
    // Clamp to ±25% change per interval
    let min_diff = (current_difficulty as u128 * 75 / 100) as u32;
    let max_diff = (current_difficulty as u128 * 125 / 100) as u32;
    
    // Clamp to absolute bounds
    new_difficulty
        .clamp(min_diff.max(MIN_DIFFICULTY), max_diff.min(MAX_DIFFICULTY))
}
```

**Rationale:**
- **Integer arithmetic:** Eliminates floating-point precision errors
- **±25% limit:** Prevents wild swings from timestamp manipulation
- **Absolute bounds:** Ensures reasonable difficulty range
- **Zero-time protection:** `max(1)` prevents division by zero

### Adjustment Edge Cases

**Case 1: All blocks mined instantly (actual_time = 0)**
```
Result: actual_secs = max(0, 1) = 1
new_difficulty = current × (target / 1) = very high
Clamped to: current × 1.25 (25% increase)
```

**Case 2: Mining stalls (actual_time >> target)**
```
Example: 10 blocks take 1 hour instead of 20 minutes
new_difficulty = 16 × (1200 / 3600) = 5.33
Clamped to: max(5.33, 16 × 0.75) = 12 (25% decrease)
```

**Case 3: Difficulty at minimum (8)**
```
Cannot decrease below MIN_DIFFICULTY (8)
Even if blocks are very slow
```

---

## Transaction Validation

### Transaction Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Chain ID (prevents cross-chain replay)
    pub chain_id: u32,
    
    /// Sender public key
    pub from: PublicKey,
    
    /// Recipient public key
    pub to: PublicKey,
    
    /// Amount in smallest unit (1 Lira = 1_000_000 units)
    pub amount: u64,
    
    /// Transaction fee for miner
    pub fee: u64,
    
    /// Sender's account nonce (prevents replay)
    pub nonce: u64,
    
    /// Ed25519 signature
    pub signature: Vec<u8>,
    
    /// Optional data payload
    pub data: Option<Vec<u8>>,
}
```

### Signing Hash

**CRITICAL:** Signature covers all transaction fields EXCEPT signature itself.

```rust
impl Transaction {
    pub fn signing_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.chain_id.to_le_bytes()); // Prevents cross-chain replay
        hasher.update(self.from.0);
        hasher.update(self.to.0);
        hasher.update(self.amount.to_le_bytes());
        hasher.update(self.fee.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        
        // Include data if present
        if let Some(ref data) = self.data {
            hasher.update(data);
        }
        
        hasher.finalize().into()
    }
}
```

### Transaction Validity Rules

**MUST Pass All Checks:**

1. **Size Check:**
   ```rust
   serialized_size(tx) <= MAX_TRANSACTION_SIZE
   if let Some(data) = tx.data {
       data.len() <= MAX_TRANSACTION_DATA_SIZE
   }
   ```

2. **Chain ID Check:**
   ```rust
   tx.chain_id == MAINNET_CHAIN_ID // or TESTNET_CHAIN_ID
   ```

3. **Signature Verification:**
   ```rust
   tx.from.verify(&tx.signing_hash(), &tx.signature) == Ok(())
   ```

4. **Amount/Fee Non-Zero (for regular tx):**
   ```rust
   tx.amount > 0 || tx.fee > 0 // Must transfer something
   tx.amount.checked_add(tx.fee).is_some() // No overflow
   ```

5. **Minimum Fee Check:**
   ```rust
   let min_fee = MIN_TRANSACTION_FEE + (serialized_size(tx) as u64 * FEE_PER_BYTE);
   tx.fee >= min_fee
   ```

6. **State Validation (on execution):**
   ```rust
   // Check nonce matches expected
   state.get_nonce(tx.from) == tx.nonce
   
   // Check sufficient balance
   let total_cost = tx.amount.checked_add(tx.fee)?;
   state.get_balance(tx.from) >= total_cost
   ```

### Coinbase Transaction Exception

**Coinbase transactions have different rules:**
```rust
impl Transaction {
    pub fn is_coinbase(&self) -> bool {
        self.from == PublicKey::zero() && self.signature.is_empty()
    }
    
    pub fn validate_coinbase(&self, block_height: u64, block_fees: u64) -> Result<(), Error> {
        if !self.is_coinbase() {
            return Err(Error::NotCoinbase);
        }
        
        // Coinbase nonce must equal block height
        if self.nonce != block_height {
            return Err(Error::InvalidCoinbaseNonce);
        }
        
        // Coinbase amount must equal block reward + fees
        let expected_amount = calculate_block_reward(block_height)
            .checked_add(block_fees)
            .ok_or(Error::RewardOverflow)?;
        
        if self.amount != expected_amount {
            return Err(Error::InvalidCoinbaseAmount);
        }
        
        Ok(())
    }
}
```

---

## Block Validation

### Full Block Validation Rules

**MUST Pass All Checks (in order):**

1. **Header Validation:**
   ```rust
   // Version check
   block.header.version >= 1 && block.header.version <= PROTOCOL_VERSION
   
   // Difficulty bounds
   block.header.difficulty >= MIN_DIFFICULTY && block.header.difficulty <= MAX_DIFFICULTY
   
   // Proof-of-work check
   block.header.meets_difficulty() == true
   ```

2. **Timestamp Validation:**
   ```rust
   let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
   
   // Not too far in the future
   block.header.timestamp <= now + MAX_FUTURE_DRIFT_SECS
   
   // After previous block (if not genesis)
   if let Some(prev_block) = get_previous_block() {
       block.header.timestamp >= prev_block.header.timestamp
   }
   ```

3. **Previous Hash Validation:**
   ```rust
   if block_height > 1 {
       let chain_tip_hash = blockchain.get_chain_tip()?;
       block.header.previous_hash == chain_tip_hash
   } else {
       // Genesis block
       block.header.previous_hash == [0u8; 32]
   }
   ```

4. **Size Validation:**
   ```rust
   serialized_size(block) <= MAX_BLOCK_SIZE
   block.transactions.len() <= MAX_TRANSACTIONS_PER_BLOCK
   ```

5. **Coinbase Validation:**
   ```rust
   // Must have at least one transaction (coinbase)
   if block.transactions.is_empty() {
       return Err(Error::MissingCoinbase);
   }
   
   // First transaction must be coinbase
   let coinbase = &block.transactions[0];
   if !coinbase.is_coinbase() {
       return Err(Error::InvalidCoinbase);
   }
   
   // Calculate total fees from other transactions
   let total_fees: u64 = block.transactions[1..]
       .iter()
       .map(|tx| tx.fee)
       .sum();
   
   // Validate coinbase amount
   coinbase.validate_coinbase(block_height, total_fees)?;
   
   // No other coinbase transactions
   for tx in &block.transactions[1..] {
       if tx.is_coinbase() {
           return Err(Error::MultipleCoinbase);
       }
   }
   ```

6. **Transaction Validation:**
   ```rust
   for tx in &block.transactions[1..] { // Skip coinbase
       tx.validate()?; // Check signature, chain ID, size, fees
   }
   ```

7. **Merkle Root Validation:**
   ```rust
   let calculated_root = calculate_merkle_root(&block.transactions);
   if block.header.merkle_root != calculated_root {
       return Err(Error::InvalidMerkleRoot);
   }
   ```

8. **State Transition Validation:**
   ```rust
   // Apply block to state and check for errors
   let mut state = blockchain.get_state()?;
   
   // Execute coinbase (mint new coins)
   state.add_balance(coinbase.to, coinbase.amount)?;
   
   // Execute regular transactions
   for tx in &block.transactions[1..] {
       // Check nonce
       if state.get_nonce(tx.from)? != tx.nonce {
           return Err(Error::InvalidNonce);
       }
       
       // Check balance
       let total_cost = tx.amount.checked_add(tx.fee)
           .ok_or(Error::Overflow)?;
       if state.get_balance(tx.from)? < total_cost {
           return Err(Error::InsufficientBalance);
       }
       
       // Execute transfer
       state.sub_balance(tx.from, total_cost)?;
       state.add_balance(tx.to, tx.amount)?;
       state.increment_nonce(tx.from)?;
   }
   
   // Commit state changes
   blockchain.set_state(state)?;
   ```

---

## Chain Selection (Fork Choice Rule)

### Longest Chain Rule

**Canonical Chain:** The chain with the **most cumulative proof-of-work** (most blocks at valid difficulty).

**Not Used:** "Heaviest chain" (sum of difficulties) — too complex for initial implementation.

### Chain Reorganization

**When to Reorg:**
```rust
pub enum ChainAction {
    Append,         // Block extends current chain tip
    Reorganize,     // Fork has more work, switch chains
    Store,          // Orphan block, store for potential future reorg
    Reject,         // Invalid block
}

pub fn evaluate_block(block: &Block) -> Result<ChainAction, Error> {
    // Validate block first
    block.validate()?;
    
    let current_height = blockchain.get_chain_height()?;
    let current_tip = blockchain.get_chain_tip()?;
    
    // Case 1: Extends current chain
    if block.header.previous_hash == current_tip {
        return Ok(ChainAction::Append);
    }
    
    // Case 2: Builds on older block (potential fork)
    if let Some(fork_base) = blockchain.get_block(&block.header.previous_hash)? {
        let fork_height = blockchain.get_block_height(&fork_base.hash())?;
        
        // Recursively calculate fork length
        let mut fork_blocks = vec![block.clone()];
        let mut current = block.clone();
        
        while current.header.previous_hash != fork_base.hash() {
            if let Some(parent) = blockchain.get_block(&current.header.previous_hash)? {
                fork_blocks.push(parent.clone());
                current = parent;
            } else {
                break; // Orphan chain
            }
        }
        
        let fork_length = fork_height + fork_blocks.len() as u64;
        
        // Reorg if fork is longer
        if fork_length > current_height {
            return Ok(ChainAction::Reorganize);
        }
    }
    
    // Case 3: Orphan block (parent not known yet)
    if blockchain.get_block(&block.header.previous_hash)?.is_none() {
        return Ok(ChainAction::Store);
    }
    
    // Case 4: Valid but not longest chain
    Ok(ChainAction::Store)
}
```

### Reorg Depth Limit (Checkpointing)

**Current:** No limit (unlimited reorg possible — SECURITY RISK)

**Recommended (Future Implementation):**
```rust
pub const MAX_REORG_DEPTH: u64 = 100; // ~3.3 hours at 2min/block

pub fn evaluate_block(block: &Block) -> Result<ChainAction, Error> {
    // ... existing logic
    
    // Reject reorgs deeper than MAX_REORG_DEPTH
    let fork_depth = current_height - fork_height;
    if fork_depth > MAX_REORG_DEPTH {
        return Err(Error::ReorgTooDeep);
    }
    
    // ... rest of logic
}
```

**Rationale:** Prevents long-range attacks where attacker builds secret chain from genesis.

---

## Merkle Tree Specification

### Construction Algorithm

```rust
pub fn calculate_merkle_root(transactions: &[Transaction]) -> [u8; 32] {
    if transactions.is_empty() {
        return [0u8; 32]; // Empty merkle root
    }
    
    let mut hashes: Vec<[u8; 32]> = transactions
        .iter()
        .map(|tx| tx.hash())
        .collect();
    
    while hashes.len() > 1 {
        let mut new_hashes = Vec::new();
        
        for chunk in hashes.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(chunk[0]);
            
            if chunk.len() > 1 {
                hasher.update(chunk[1]);
            } else {
                hasher.update(chunk[0]); // Duplicate odd node
            }
            
            new_hashes.push(hasher.finalize().into());
        }
        
        hashes = new_hashes;
    }
    
    hashes[0]
}
```

**Known Issue:** Odd-node duplication enables CVE-2012-2459 style attack.

**Mitigation (Future):**
```rust
// Alternative: Include position in hash to prevent duplication attack
hasher.update(chunk[0]);
hasher.update(&[0u8]); // Left marker

if chunk.len() > 1 {
    hasher.update(chunk[1]);
    hasher.update(&[1u8]); // Right marker
} else {
    // No duplication - use special "odd" marker
    hasher.update(&[2u8]);
}
```

---

## Protocol Upgrade Mechanism

### Soft Forks (Backward Compatible)

**Process:**
1. New rule is **stricter** than old rule
2. Old nodes accept new blocks (think they're valid)
3. New nodes reject invalid old blocks
4. Requires miner majority (>50% hashrate)

**Example:** Reduce `MAX_BLOCK_SIZE` from 1MB to 500KB
- New nodes: Reject 600KB blocks
- Old nodes: Accept 600KB blocks (not enforcing new rule)
- Outcome: Old nodes follow new chain if miners upgrade

**Activation:**
```rust
pub fn check_soft_fork_active(height: u64) -> bool {
    // Soft fork activates at height X
    height >= SOFT_FORK_ACTIVATION_HEIGHT
}
```

### Hard Forks (Backward Incompatible)

**Process:**
1. New rule is **different** from old rule
2. Old nodes reject new blocks
3. New nodes reject old blocks
4. Requires community coordination

**Example:** Change block time from 120s to 60s
- Incompatible: Old and new chains diverge permanently

**Activation:**
```rust
pub const HARD_FORK_HEIGHT: u64 = 500_000; // Activate at specific height
pub const HARD_FORK_TIMESTAMP: u64 = 1750000000; // Or at specific time

pub fn protocol_version_at_height(height: u64) -> u32 {
    if height >= HARD_FORK_HEIGHT {
        2 // New protocol version
    } else {
        1 // Old protocol version
    }
}
```

---

## Security Considerations

### Attack Vectors Mitigated

1. **Replay Attacks (Cross-Chain):** Mitigated by `chain_id` in transaction signature
2. **Replay Attacks (In-Chain):** Mitigated by mandatory nonce enforcement
3. **Timestamp Manipulation:** Mitigated by `MAX_FUTURE_DRIFT_SECS` limit
4. **Integer Overflow:** Mitigated by `checked_add/sub` operations
5. **Signature Forgery:** Mitigated by Ed25519 verification
6. **Invalid Block Acceptance:** Mitigated by comprehensive validation

### Remaining Vulnerabilities

**High Priority:**
1. **Selfish Mining:** No defense (requires protocol change)
2. **Eclipse Attacks:** No peer reputation system
3. **51% Attack:** Inherent to PoW (mitigated by PoS migration)
4. **CVE-2012-2459 (Merkle Tree):** Odd-node duplication attack

**Medium Priority:**
5. **Time-Warp Attack:** Possible if >50% miners collude on timestamps
6. **Long-Range Attack:** No checkpointing (unlimited reorg depth)
7. **DOS via Large Blocks:** Size limits help but not sufficient

---

## Testnet vs Mainnet Differences

| Parameter | Mainnet | Testnet |
|-----------|---------|---------|
| **Chain ID** | 5963 | 5964 |
| **Genesis Timestamp** | 1700000000 | 1700001000 |
| **Genesis Difficulty** | 16 | 8 |
| **Block Reward** | 50 Lira | 50 Lira (same) |
| **Coins Have Value** | YES | NO |
| **Reset Frequency** | Never | As needed |

**Testnet Purpose:**
- Protocol testing
- Wallet integration
- Mining software development
- Chain reorg simulation

---

## Implementation Checklist

### Phase 1: Core Consensus (Week 1-2)
- [ ] Create `constants.rs` with all protocol constants
- [ ] Implement `chain_id` in transaction signature
- [ ] Add timestamp validation to block validation
- [ ] Implement checked arithmetic everywhere
- [ ] Add nonce enforcement in state transitions

### Phase 2: Chain Reorganization (Week 2-3)
- [ ] Implement fork detection logic
- [ ] Add cumulative work calculation
- [ ] Implement chain switching mechanism
- [ ] Test 1-block, 10-block, 100-block reorgs
- [ ] Add reorg depth limit (checkpointing)

### Phase 3: Economic Integration (Week 3-4)
- [ ] Implement coinbase transaction creation
- [ ] Add coinbase validation to block validation
- [ ] Integrate `calculate_block_reward()` from TOKENOMICS.md
- [ ] Test halving across 26 halvings

### Phase 4: Size Limits & DOS Protection (Week 4-5)
- [ ] Add transaction size validation
- [ ] Add block size validation
- [ ] Implement fee-per-byte minimum
- [ ] Test with maximum-size blocks

### Phase 5: Testing & Auditing (Week 6-8)
- [ ] Comprehensive unit tests (500+ assertions)
- [ ] Integration tests (multi-node scenarios)
- [ ] Fuzzing (random invalid blocks/transactions)
- [ ] External security audit
- [ ] Testnet deployment (public testing)

---

## Test Vectors

### Genesis Block (Mainnet)

```json
{
  "header": {
    "version": 1,
    "previous_hash": "0000000000000000000000000000000000000000000000000000000000000000",
    "merkle_root": "TBD (after coinbase finalized)",
    "timestamp": 1700000000,
    "difficulty": 16,
    "nonce": "TBD (pre-mined)"
  },
  "transactions": [
    {
      "comment": "Coinbase transaction (TBD)"
    }
  ]
}
```

### Valid Block Example

```json
{
  "header": {
    "version": 1,
    "previous_hash": "00001a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f",
    "merkle_root": "abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab",
    "timestamp": 1700000240,
    "difficulty": 16,
    "nonce": 123456789
  },
  "transactions": [
    {
      "comment": "Coinbase + 3 regular transactions"
    }
  ]
}
```

### Invalid Block Examples (Must Reject)

**Case 1: Invalid PoW**
```json
{
  "header": {
    "difficulty": 16,
    "nonce": 0
  },
  "comment": "Hash does not meet difficulty target"
}
```

**Case 2: Timestamp Too Far Future**
```json
{
  "header": {
    "timestamp": 9999999999
  },
  "comment": "Timestamp > now + MAX_FUTURE_DRIFT_SECS"
}
```

**Case 3: Missing Coinbase**
```json
{
  "transactions": []
}
```

**Case 4: Invalid Coinbase Amount**
```json
{
  "transactions": [
    {
      "from": "0000000000000000000000000000000000000000000000000000000000000000",
      "amount": 1000000000,
      "comment": "Amount > block_reward + fees"
    }
  ]
}
```

---

## References

**Bitcoin Consensus Rules:**
- https://en.bitcoin.it/wiki/Protocol_rules
- https://github.com/bitcoin/bitcoin/blob/master/doc/developer-notes.md

**Ethereum Yellow Paper:**
- https://ethereum.github.io/yellowpaper/paper.pdf

**Difficulty Adjustment Algorithms:**
- Bitcoin: https://en.bitcoin.it/wiki/Difficulty
- Digibyte (Multi-algo): https://github.com/digibyte/digibyte/wiki/DigiShield

---

## Document History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0 | Nov 18, 2025 | Initial specification from audit | Audit Team |

---

## Appendix A: Difficulty Adjustment Simulation

**Test Case 1: Blocks Mined Too Fast**
```
Initial Difficulty: 16
Target Time: 10 blocks × 120s = 1200s (20 minutes)
Actual Time: 600s (10 minutes) — 2× too fast
Expected New Difficulty: 16 × (1200/600) = 32
Clamped to: min(32, 16 × 1.25) = 20 (25% increase limit)
```

**Test Case 2: Blocks Mined Too Slow**
```
Initial Difficulty: 16
Target Time: 1200s
Actual Time: 2400s (40 minutes) — 2× too slow
Expected New Difficulty: 16 × (1200/2400) = 8
Clamped to: max(8, 16 × 0.75) = 12 (25% decrease limit)
```

**Test Case 3: Perfect Timing**
```
Initial Difficulty: 16
Target Time: 1200s
Actual Time: 1200s — exactly on target
New Difficulty: 16 × (1200/1200) = 16 (no change)
```

---

## Appendix B: State Transition Example

**Initial State:**
```
Alice Balance: 100 Lira
Alice Nonce: 5
Bob Balance: 0 Lira
Miner Balance: 0 Lira
```

**Block Contains:**
```
1. Coinbase: 50 Lira → Miner
2. Alice → Bob: 10 Lira, fee 0.001 Lira, nonce 5
```

**State After Block:**
```
Alice Balance: 100 - 10 - 0.001 = 89.999 Lira
Alice Nonce: 6
Bob Balance: 0 + 10 = 10 Lira
Miner Balance: 0 + 50 + 0.001 = 50.001 Lira
```

**Validation Steps:**
1. ✅ Alice nonce matches (5 == 5)
2. ✅ Alice balance sufficient (100 >= 10.001)
3. ✅ Coinbase amount correct (50 + 0.001 == 50.001)
4. ✅ Merkle root matches transaction hashes
5. ✅ Block hash meets difficulty 16

---

**Status:** 🚨 **This specification must be implemented before mainnet launch.**

**Next Steps:**
1. Community review and feedback (2 weeks)
2. Finalize constants (chain ID, genesis timestamp)
3. Implementation sprint (6-8 weeks)
4. Testnet deployment (6+ months)
5. Security audit (external)
6. Mainnet launch (Q2-Q3 2026)

**Contact:** opensyria.net@gmail.com
