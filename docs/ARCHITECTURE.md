# Open Syria Blockchain - Technical Architecture
**Digital Lira (الليرة الرقمية) | Sovereign Blockchain for Syria**

Version: 0.1.0  
Date: November 2025  
Status: Core Implementation Complete

---

## Executive Summary

Open Syria is a sovereign, Rust-based blockchain designed to power the Digital Lira, Syria's decentralized digital currency. The system implements a Proof-of-Work consensus mechanism with a clear migration path to Proof-of-Stake, featuring bilingual tooling (Arabic/English), comprehensive state management, and a modular architecture designed for long-term evolution.

**Key Metrics:**
- **Language:** Rust (safe, concurrent, performant)
- **Consensus:** Proof-of-Work (SHA-256, future PoS)
- **Crypto:** Ed25519 signatures
- **Storage:** RocksDB (persistent key-value store)
- **Network:** P2P (planned - libp2p)
- **Hash Rate:** ~1.6M hashes/second (single-threaded)

---

## System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     User Applications                        │
├─────────────┬──────────────┬──────────────┬─────────────────┤
│   Wallet    │   Explorer   │  Governance  │    Identity     │
│    CLI      │   Web UI     │   Platform   │    System       │
└─────────────┴──────────────┴──────────────┴─────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Node CLI (Full Node)                      │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Mining Engine  │  TX Pool  │  Chain Validation     │  │
│  └──────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
            ┌─────────────────┼─────────────────┐
            ▼                 ▼                 ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│   Consensus     │ │      Core       │ │    Storage      │
│  (PoW/PoS)      │ │  (Primitives)   │ │   (RocksDB)     │
├─────────────────┤ ├─────────────────┤ ├─────────────────┤
│ • PoW Mining    │ │ • Block         │ │ • Blockchain    │
│ • Difficulty    │ │ • Transaction   │ │ • State         │
│ • Validation    │ │ • Crypto        │ │ • Indexing      │
└─────────────────┘ └─────────────────┘ └─────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              Network Layer (Future - libp2p)                 │
│     Peer Discovery  │  Block Sync  │  TX Propagation        │
└─────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. Core Module (`opensyria-core`)

**Purpose:** Fundamental blockchain primitives and cryptographic operations.

**Components:**

#### Block Structure
```rust
Block {
    header: BlockHeader {
        version: u32,
        previous_hash: [u8; 32],
        merkle_root: [u8; 32],
        timestamp: u64,
        difficulty: u32,
        nonce: u64,
    },
    transactions: Vec<Transaction>,
}
```

**Design Decisions:**
- **SHA-256 hashing:** Industry standard, battle-tested
- **Merkle trees:** Efficient transaction verification
- **Version field:** Forward compatibility for protocol upgrades
- **64-bit nonce:** Sufficient for PoW search space

#### Transaction Model
```rust
Transaction {
    from: PublicKey,
    to: PublicKey,
    amount: u64,        // 1 Lira = 1,000,000 units
    fee: u64,
    nonce: u64,         // Replay protection
    signature: Vec<u8>,
    data: Option<Vec<u8>>, // Future extensibility
}
```

**Features:**
- Ed25519 signatures (fast, secure, 32-byte keys)
- Nonce-based replay attack prevention
- Micro-unit precision (6 decimal places)
- Optional data payload for future smart contracts

#### Cryptography
- **Signing:** Ed25519-Dalek
- **Hashing:** SHA-256
- **Key derivation:** Standard Ed25519 key generation
- **Address format:** 64-character hex (32 bytes)

---

### 2. Consensus Module (`opensyria-consensus`)

**Purpose:** Proof-of-Work implementation with mining capabilities.

#### Mining Algorithm

**Difficulty Representation:**
- Measured in leading zero bits
- Range: 8-192 bits (configurable)
- Validated via `BlockHeader::meets_difficulty()`

**Mining Process:**
```
1. Construct block with transactions
2. Set difficulty target
3. Iterate nonce from 0 to u64::MAX
4. Hash block header (SHA-256)
5. Check if hash meets difficulty
6. Return mined block + statistics
```

**Performance:**
- Single-threaded: ~1.6M H/s
- Difficulty 12: ~5,000 hashes avg
- Difficulty 16: ~72,000 hashes avg
- Scalable to multi-core (future)

#### Difficulty Adjustment

**Algorithm:**
```rust
Target: 60 seconds per block
Adjustment interval: Every 10 blocks
Max adjustment: ±25% per interval
```

**Formula:**
```
ratio = actual_time / target_time
adjustment = current_difficulty × (1 - ratio) × 0.25
new_difficulty = clamp(current + adjustment, 8, 192)
```

**Rationale:**
- Prevents rapid difficulty swings
- Adapts to changing hash power
- Maintains predictable block times
- Protects against time manipulation attacks

---

### 3. Storage Module (`opensyria-storage`)

**Purpose:** Persistent blockchain and state management using RocksDB.

#### Blockchain Storage

**Schema:**
```
Key                  → Value
─────────────────────────────────────
[block_hash]         → Block (bincode)
height_N             → block_hash
chain_tip            → latest_hash
chain_height         → current_height
```

**Operations:**
- `append_block()`: Validates previous_hash, stores block, updates indexes
- `get_block()`: Retrieve by hash
- `get_block_by_height()`: Retrieve by height (via index)
- `get_block_range()`: Batch retrieval for sync

**Validation:**
- Previous hash must match chain tip
- Genesis block must have zero previous hash
- Atomic updates (height + tip + block)

#### State Storage

**Schema:**
```
Key                  → Value
─────────────────────────────────────
balance_[pubkey]     → u64 (LE bytes)
nonce_[pubkey]       → u64 (LE bytes)
```

**Operations:**
- `get_balance()`, `set_balance()`: Direct access
- `add_balance()`, `sub_balance()`: Safe arithmetic
- `transfer()`: Atomic balance updates
- `get_nonce()`, `increment_nonce()`: Replay protection

**Features:**
- Prefix scanning for bulk queries
- Insufficient balance detection
- Nonce validation
- Account creation on first use (implicit)

---

### 4. Wallet (`opensyria-wallet`)

**Purpose:** Secure key management and transaction creation.

#### Features

**Commands:**
- `create`: Generate Ed25519 keypair
- `list`: Display all accounts
- `info`: Show account details
- `send`: Sign transaction
- `delete`: Remove account (with confirmation)

**Storage:**
```json
{
  "name": "account-name",
  "address": "hex-encoded-public-key",
  "private_key": "hex-encoded-secret",
  "created_at": unix_timestamp
}
```

**Location:** `~/.opensyria/wallet/*.json`

**Security Considerations:**
- Private keys stored as hex in JSON (unencrypted)
- File permissions: User-only read/write
- **Future:** Password encryption, hardware wallet support

#### Transaction Signing

**Process:**
```
1. Create unsigned transaction
2. Compute signing hash (SHA-256)
3. Sign with Ed25519 private key
4. Attach signature to transaction
5. Verify signature before broadcast
6. Output signed JSON
```

**Verification:**
- Immediate signature check
- Prevents unsigned transactions
- Deterministic hash computation

---

### 5. Node CLI (`opensyria-node-cli`)

**Purpose:** Full blockchain node with mining, validation, and state management.

#### Commands

**Initialization:**
```bash
opensyria-node-cli init --difficulty 16
```
- Creates data directory
- Generates genesis block
- Initializes storage

**Mining:**
```bash
opensyria-node-cli mine --blocks 10 --difficulty 16 --verbose
```
- Mines new blocks
- Updates chain state
- Displays real-time stats
- Validates before appending

**Inspection:**
```bash
opensyria-node-cli info                # Chain stats
opensyria-node-cli block latest        # Latest block
opensyria-node-cli block 5             # Specific height
opensyria-node-cli balance <address>   # Account balance
```

**Transaction Processing:**
```bash
opensyria-node-cli process-tx --file tx.json
```
- Verifies signature
- Validates nonce
- Updates balances
- Increments sender nonce

**Export:**
```bash
opensyria-node-cli export --output chain.json --start 1 --end 100
```
- Exports block range
- JSON format
- For analysis/backup

---

## Data Flow Diagrams

### Block Mining Flow

```
┌──────────────┐
│ Start Mining │
└──────┬───────┘
       │
       ▼
┌─────────────────────┐
│ Get Current Tip     │
│ previous_hash = tip │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│ Create New Block    │
│ • transactions      │
│ • merkle_root       │
│ • timestamp         │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│ Mine (find nonce)   │
│ Loop until valid    │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│ Validate Block      │
│ • difficulty check  │
│ • merkle verify     │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│ Append to Chain     │
│ • store block       │
│ • update height     │
│ • update tip        │
└──────┬──────────────┘
       │
       ▼
┌──────────────┐
│   Success    │
└──────────────┘
```

### Transaction Processing Flow

```
┌──────────────────┐
│ Receive TX JSON  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Deserialize TX   │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Verify Signature │
│ pk.verify(sig)   │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Check Nonce      │
│ expected == tx   │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Validate Balance │
│ bal >= amt + fee │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Update State     │
│ • sub from       │
│ • add to         │
│ • inc nonce      │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│    Success       │
└──────────────────┘
```

---

## Security Model

### Threat Analysis

| Threat | Mitigation |
|--------|-----------|
| **Double-spend** | Nonce-based ordering, blockchain immutability |
| **Replay attacks** | Per-account nonce increments |
| **Invalid signatures** | Ed25519 verification before processing |
| **Balance manipulation** | State validation, atomic transfers |
| **Chain reorg** | Longest chain rule (future: finality) |
| **51% attack** | PoW difficulty, future PoS migration |
| **Sybil attack** | Cost of PoW, future stake requirements |

### Cryptographic Security

**Ed25519 Properties:**
- 128-bit security level
- Deterministic signatures
- Fast verification (~70k/sec)
- No known practical attacks

**SHA-256 Properties:**
- 256-bit output
- Pre-image resistance
- Collision resistance
- Mining difficulty tunable

---

## Performance Characteristics

### Mining Performance

| Difficulty | Avg Hashes | Time (est) |
|------------|-----------|------------|
| 8          | ~256      | <0.01s     |
| 12         | ~4,096    | ~0.003s    |
| 16         | ~65,536   | ~0.04s     |
| 20         | ~1M       | ~0.6s      |
| 24         | ~16M      | ~10s       |

**Hardware:** M-series MacBook Pro (single-threaded)

### Storage Performance

**RocksDB Benchmarks:**
- Writes: ~100k ops/sec
- Reads: ~200k ops/sec
- Disk usage: ~1KB per block (empty)
- Compaction: Automatic LSM-tree

### Transaction Throughput

**Current (single-threaded):**
- Signature verification: ~70k/sec
- State updates: ~100k/sec
- Bottleneck: Network I/O (future)

**Theoretical Maximum:**
- With parallel processing: ~1M tx/sec
- Limited by storage bandwidth
- Future: Sharding, layer-2 solutions

---

## Deployment Architecture

### Node Types

**1. Full Node**
- Stores entire blockchain
- Validates all blocks
- Can mine blocks
- Serves peers (future)

**2. Mining Node**
- Full node + mining
- Constructs new blocks
- Competes for rewards
- High CPU usage

**3. Light Client (Future)**
- Stores headers only
- SPV verification
- Low resource usage
- Relies on full nodes

### System Requirements

**Minimum:**
- CPU: 2 cores, 2GHz
- RAM: 2GB
- Disk: 10GB SSD
- Network: 1Mbps

**Recommended:**
- CPU: 4+ cores, 3GHz+
- RAM: 8GB
- Disk: 50GB NVMe SSD
- Network: 10Mbps+

---

## Roadmap

### Phase 1: Core (✅ Complete)
- [x] Block & transaction primitives
- [x] PoW consensus
- [x] Persistent storage
- [x] Wallet CLI
- [x] Node CLI with mining
- [x] Comprehensive testing

### Phase 2: Network (In Progress)
- [ ] P2P networking (libp2p)
- [ ] Peer discovery
- [ ] Block synchronization
- [ ] Transaction propagation
- [ ] Network protocol v1

### Phase 3: Ecosystem
- [ ] Block explorer web UI
- [ ] REST API for queries
- [ ] Transaction pool (mempool)
- [ ] Mining pool support
- [ ] Hardware wallet integration

### Phase 4: Governance
- [ ] On-chain voting
- [ ] Proposal system
- [ ] Multi-signature accounts
- [ ] Protocol upgrade mechanism
- [ ] Community treasury

### Phase 5: Advanced Features
- [ ] Proof-of-Stake migration
- [ ] Smart contract VM
- [ ] Cross-chain bridges
- [ ] Privacy features (zk-SNARKs)
- [ ] Layer-2 scaling

### Phase 6: Cultural Integration
- [ ] Arabic language full support
- [ ] Cultural identity NFTs
- [ ] Heritage preservation system
- [ ] Community governance
- [ ] Educational resources

---

## Testing Strategy

### Current Coverage

**Unit Tests:** 23 passing
- Core: 11 tests
- Consensus: 5 tests
- Storage: 7 tests

**Integration Tests:** Manual
- Wallet → Transaction signing
- Node → Mining workflow
- Storage → Chain persistence

### Future Testing

- [ ] Property-based testing (proptest)
- [ ] Fuzzing (cargo-fuzz)
- [ ] Load testing (>10k blocks)
- [ ] Network simulation
- [ ] Security audits

---

## Monitoring & Observability

### Logging

**Framework:** `tracing` crate
- Structured logging
- Multiple severity levels
- JSON export capability

**Current Events:**
- Genesis block creation
- Block mining progress
- Transaction processing
- Chain height updates

### Metrics (Future)

- Block time distribution
- Hash rate trends
- Transaction volume
- Network peer count
- Storage usage growth

---

## Contributing Guidelines

### Code Style

- **Language:** Rust 2021 edition
- **Formatting:** `cargo fmt`
- **Linting:** `cargo clippy`
- **Documentation:** Required for public APIs

### Pull Request Process

1. Fork repository
2. Create feature branch
3. Write tests
4. Update documentation
5. Submit PR with description
6. Pass CI checks
7. Code review approval

### Branching Strategy

- `main`: Stable releases
- `develop`: Integration branch
- `feature/*`: New features
- `bugfix/*`: Bug fixes
- `release/*`: Release candidates

---

## Governance Model

### Decision Making

**Technical Decisions:**
- Core team approval required
- Community input via proposals
- Security-critical: audited

**Economic Parameters:**
- Block reward: TBD
- Fee structure: Market-based
- Supply cap: To be determined

**Protocol Upgrades:**
- BIP-style improvement proposals
- Testing on devnet
- Staged rollout
- Community consensus

---

## License

**License:** MIT OR Apache-2.0 (dual-licensed)

**Rationale:**
- Maximum compatibility
- Business-friendly
- Open-source ecosystem alignment

---

## Appendix

### A. Glossary

- **PoW:** Proof-of-Work consensus mechanism
- **PoS:** Proof-of-Stake consensus mechanism
- **UTXO:** Unspent Transaction Output model (not used)
- **Account-based:** Balance model used in Open Syria
- **Ed25519:** Elliptic curve signature scheme
- **Nonce:** Number used once (replay protection)
- **Merkle Tree:** Hash tree for efficient verification

### B. References

- [Rust Lang](https://www.rust-lang.org/)
- [RocksDB](https://rocksdb.org/)
- [Ed25519](https://ed25519.cr.yp.to/)
- [libp2p](https://libp2p.io/)
- [Bitcoin Whitepaper](https://bitcoin.org/bitcoin.pdf)
- [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf)

### C. Contact

- **Repository:** https://github.com/OpenSyria/blockchain (placeholder)
- **Community:** TBD
- **Security:** security@opensyria.org (placeholder)

---

**Document Version:** 1.0.0  
**Last Updated:** November 2025  
**Status:** Living Document
