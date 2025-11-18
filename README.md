# Open Syria Blockchain | بلوكتشين سوريا المفتوحة

A sovereign, Rust-based blockchain for the Digital Lira (الليرة الرقمية).

## Architecture

```
opensyria/
├── crates/
│   ├── core/             # Blockchain primitives (Block, Transaction, Crypto)
│   ├── consensus/        # PoW consensus (future PoS migration)
│   ├── network/          # P2P networking layer
│   ├── storage/          # State & chain database (RocksDB)
│   ├── wallet/           # CLI/GUI wallet (bilingual)
│   ├── node-cli/         # Node management CLI
│   ├── explorer-backend/ # Block explorer API
│   ├── governance/       # On-chain governance
│   └── identity/         # Cultural identity system
├── docs/                 # Technical & user documentation
│   ├── ARCHITECTURE.md   # System design & architecture
│   └── DEPLOYMENT.md     # Deployment & setup guide
└── target/               # Build artifacts
```

## Current Status

**Phase:** Production Ready ✅  
**Consensus:** Proof-of-Work (SHA-256)  
**Languages:** Rust, Arabic/English  
**Test Coverage:** 72 passing tests (Core: 19, Storage: 7, Identity: 11, Network: 2, Mempool: 5, Mining Pool: 5, Governance: 23)

### ✅ Completed Features

- [x] **Core Blockchain:** Blocks, transactions, Ed25519 signatures
- [x] **PoW Consensus:** Mining engine with difficulty adjustment
- [x] **Storage Layer:** RocksDB-backed persistence (blockchain + state)
- [x] **Wallet CLI:** Key management, transaction signing (bilingual)
- [x] **Node CLI:** Full node with mining, chain inspection
- [x] **Miner:** Standalone PoW mining tool
- [x] **Cultural Identity System:** Heritage token standard + CLI (11 tests)
- [x] **P2P Networking:** libp2p-based multi-node synchronization (2 tests)
- [x] **Network CLI:** P2P node management commands (`network start`, peer discovery)
- [x] **Transaction Pool (Mempool):** Priority-based pending transaction management (5 tests)
- [x] **Integration Tests:** Multi-node blockchain testing
- [x] **Block Explorer:** REST API + Web UI for blockchain visualization
- [x] **Governance Framework:** On-chain proposals and voting system (23 tests)
- [x] **Governance Node Integration:** Proposal/voting via node CLI, automatic execution
- [x] **Wallet REST API:** HTTP endpoints for transactions, balance queries, blockchain info
- [x] **Network Daemon Mode:** Persistent background P2P node with auto-mining and monitoring
- [x] **Multi-signature Accounts:** M-of-N signature requirements for enhanced security (8 tests)
- [x] **Mining Pool Support:** Coordinated mining with proportional/PPS/PPLNS reward distribution (5 tests)
- [x] **IPFS Integration:** Decentralized storage for cultural heritage multimedia content
- [x] **Documentation:** Comprehensive docs organized in docs/ (15 files: architecture, deployment, identity, IPFS, network, governance, API, testing)

### 🚧 Future Enhancements (Optional)

- [ ] Mining pool network server (persistent daemon) - workaround: use pool CLI
- [ ] Network daemon storage unification - workaround: use network CLI directly
- [ ] Smart contracts (VM integration)
- [ ] Light clients for mobile/embedded devices

## Quick Start

### Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build

```bash
# Build all components
cargo build --release

# Binaries will be in target/release/
# - wallet
# - miner
# - opensyria-node-cli
```

### Initialize & Run Node

```bash
# Initialize blockchain
./target/release/opensyria-node-cli init --difficulty 16

# Check status
./target/release/opensyria-node-cli info

# Mine blocks
./target/release/opensyria-node-cli mine --blocks 5 --difficulty 16 --verbose
```

### Create Wallet & Send Transaction

```bash
# Create wallet
./target/release/wallet create --name alice

# View account
./target/release/wallet info alice

# Create transaction
./target/release/wallet send \
  --from alice \
  --to <recipient-address> \
  --amount 10.5 \
  --fee 0.001 \
  --nonce 0
```

### Manage Cultural Identity Tokens

```bash
# View Syrian heritage examples
./target/release/identity examples

# Create a heritage token
./target/release/identity create \
  --id damascus-steel-001 \
  --name "Damascus Steel Craftsmanship" \
  --name-ar "حرفة الفولاذ الدمشقي" \
  --description "Traditional Damascus steel forging techniques..." \
  --token-type craft \
  --category islamic \
  --city Damascus

# View token details
./target/release/identity info damascus-steel-001

# Upload heritage multimedia to IPFS
./target/release/identity upload \
  --file heritage-video.mp4 \
  --token-id damascus-steel-001

# Retrieve content from IPFS
./target/release/identity retrieve QmXyz... --output retrieved-video.mp4

# Link existing IPFS content to token
./target/release/identity link --token-id damascus-steel-001 --cid QmXyz...
```

**Requirements:** IPFS daemon running (`ipfs daemon`)

# List all tokens
./target/release/identity list
```

### Start P2P Network Node

```bash
# Start network node (auto-discovers peers on local network)
./target/release/opensyria-node-cli network start

# Start with custom port
./target/release/opensyria-node-cli -d /tmp/node1 network start \
  -l /ip4/127.0.0.1/tcp/9000

# Connect to specific bootstrap peers
./target/release/opensyria-node-cli network start \
  -l /ip4/0.0.0.0/tcp/9000 \
  -b /ip4/192.168.1.100/tcp/9000/p2p/12D3KooW...

# Output shows real-time network events:
# → Peer connected: 12D3KooWABC123...
# 📦 New block received: block (hash: 2df5fb03...)
# 💸 New transaction: 50.5 SYL
```

See **[Network CLI docs](docs/network/NETWORK_CLI.md)** for multi-node setup guide.

### Run Network Daemon (Background Node)

```bash
# Start daemon with default settings
./target/release/opensyria-node-cli network daemon

# Start daemon with auto-mining enabled
./target/release/opensyria-node-cli network daemon \
  --mine --difficulty 16

# Start daemon with custom configuration
./target/release/opensyria-node-cli network daemon \
  --listen /ip4/0.0.0.0/tcp/9000 \
  --sync-interval 30 \
  --mine \
  --difficulty 14 \
  --mdns

# Connect to bootstrap peers
./target/release/opensyria-node-cli network daemon \
  --bootstrap /ip4/192.168.1.100/tcp/9000/p2p/12D3KooW... \
  --mine

# Output shows periodic status and mining events:
# 🚀 Daemon running... (Press Ctrl+C to stop)
# 💫 Status check - height: 6, pending txs: 0
# ⛏️  No transactions to mine
# 💫 Status check - height: 7, pending txs: 2
# ⛏️  Mining block with 2 transactions...
# ✅ Mined block at height 7
```

**Daemon Features:**
- Runs as persistent background process
- Periodic chain synchronization checks
- Optional auto-mining when transactions are pending
- Graceful shutdown on Ctrl+C (SIGINT)
- Configurable sync interval and mining difficulty
- Real-time status monitoring (chain height, pending txs)

**Note:** Full P2P networking integration is pending storage architecture refactoring.

### Wallet API

Start REST API server for wallet operations:

```bash
# Start wallet API server
./target/release/opensyria-wallet-api -d ~/.opensyria/node --port 8080
```

**Available Endpoints:**
- `GET /health` - Health check
- `GET /api/v1/blockchain/info` - Chain height, difficulty, transactions
- `GET /api/v1/account/{address}/balance` - Account balance and nonce
- `GET /api/v1/mempool/status` - Pending transaction count and fees
- `POST /api/v1/transaction/submit` - Submit signed transaction
- `POST /api/v1/transaction/create` - Create and sign transaction (dev only)

**Example Usage:**
```bash
# Get blockchain info
curl http://localhost:8080/api/v1/blockchain/info

# Check account balance
curl http://localhost:8080/api/v1/account/{hex_public_key}/balance

# Submit transaction
curl -X POST http://localhost:8080/api/v1/transaction/submit \
  -H "Content-Type: application/json" \
  -d '{
    "from": "hex_encoded_public_key",
    "to": "hex_encoded_public_key",
    "amount": 1000000,
    "fee": 100,
    "signature": "hex_encoded_signature"
  }'
```

### Use Multi-signature Accounts

```bash
# Create a 2-of-3 multisig account
./target/release/opensyria-node-cli multisig create \
  --signer <pubkey1> \
  --signer <pubkey2> \
  --signer <pubkey3> \
  --threshold 2 \
  --balance 10000000

# View multisig account details
./target/release/opensyria-node-cli multisig info <multisig-address>

# Create a multisig transaction
./target/release/opensyria-node-cli multisig create-tx \
  --from <multisig-address> \
  --to <recipient-address> \
  --amount 5000000 \
  --fee 100 \
  --output tx.json

# Sign transaction (signer #1)
./target/release/opensyria-node-cli multisig sign \
  --tx-file tx.json \
  --private-key <signer1-private-key> \
  --output tx-signed1.json

# Sign transaction (signer #2, threshold met)
./target/release/opensyria-node-cli multisig sign \
  --tx-file tx-signed1.json \
  --private-key <signer2-private-key> \
  --output tx-signed2.json

# Submit fully-signed transaction
./target/release/opensyria-node-cli multisig submit \
  --tx-file tx-signed2.json
```

**Multisig Features:**
- M-of-N signature threshold (e.g., 2-of-3, 3-of-5)
- Deterministic account addresses derived from signers
- Offline transaction signing workflow
- Protection against unauthorized transactions
- Enhanced security for high-value accounts

### Join a Mining Pool

```bash
# Initialize a mining pool
./target/release/opensyria-node-cli pool init \
  --operator <pool-operator-pubkey> \
  --fee 2 \
  --share-difficulty 12 \
  --method proportional

# Register as a miner
./target/release/opensyria-node-cli pool register <miner-pubkey>

# View pool statistics
./target/release/opensyria-node-cli pool stats

# List all miners
./target/release/opensyria-node-cli pool miners

# Check miner stats
./target/release/opensyria-node-cli pool miner <miner-pubkey>

# Process payouts
./target/release/opensyria-node-cli pool payout --miner <miner-pubkey>
```

**Mining Pool Features:**
- Multiple reward methods: Proportional, PPS (Pay Per Share), PPLNS (Pay Per Last N Shares)
- Configurable pool fees (0-100%)
- Share validation and difficulty adjustment
- Automated reward distribution
- Minimum payout thresholds
- Real-time hashrate tracking
- Invalid share detection

### Explore the Blockchain

```bash
# Start block explorer (web UI + REST API)
./target/release/explorer data 3000

# Open browser to: http://localhost:3000
# - View recent blocks
# - Search by block height, hash, transaction, or address
# - Real-time blockchain statistics
```

**API Endpoints:**
- `GET /api/stats` - Blockchain statistics
- `GET /api/blocks` - Recent blocks (paginated)
- `GET /api/blocks/:height` - Block by height
- `GET /api/blocks/hash/:hash` - Block by hash
- `GET /api/transactions/:hash` - Transaction details
- `GET /api/address/:address` - Address information
- `GET /api/search/:query` - Universal search

### Participate in Governance

```bash
# Create a proposal
./target/debug/opensyria-node-cli governance propose \
  --title "Increase Block Size" \
  --description "Proposal to increase block size limit" \
  -t text \
  --proposer <hex-public-key>

# Vote on proposals
./target/debug/opensyria-node-cli governance vote 1 \
  --choice yes \
  --voter <hex-public-key>

# View all proposals
./target/debug/opensyria-node-cli governance list

# Show proposal details
./target/debug/opensyria-node-cli governance show 1

# Check governance statistics
./target/debug/opensyria-node-cli governance stats

# Process proposals (finalize voting, execute passed proposals)
./target/debug/opensyria-node-cli governance process
```

**Governance Features:**
- Integrated with node CLI (no separate daemon required)
- Proposals stored in node data directory
- Automatic execution of passed proposals
- Stake-based voting power from account balances
- 7 proposal types (text, min-fee, block-size, etc.)
- Configurable quorum and thresholds

See **[Governance docs](docs/governance/GOVERNANCE.md)** for complete guide.

**Standalone Governance CLI:**

For development/testing, a standalone CLI is also available:

```bash
# Initialize governance system
cargo run --bin governance-cli init

# Create a proposal
cargo run --bin governance-cli propose \
  --title "Test Proposal" \
  --description "This is a test" \
  -t text

# Vote on proposals
cargo run --bin governance-cli vote 1 --choice yes

# View all proposals
cargo run --bin governance-cli list

# Check governance statistics
cargo run --bin governance-cli stats
```

**Proposal Types:**
- Text proposals (non-binding)
- Minimum fee adjustments
- Block size limits
- Protocol upgrades
- Treasury spending
- More types coming soon

## Performance Metrics

- **Hash Rate:** ~1.6M H/s (single-threaded, M-series MacBook)
- **Block Time Target:** 60 seconds (adjustable)
- **Signature Verification:** ~70k tx/sec
- **Storage:** RocksDB (100k+ ops/sec)
- **P2P Network:** libp2p 0.53 (Gossipsub, mDNS, Kademlia DHT)
- **Test Coverage:** 72 passing tests across 7 crates
- **Build Time:** ~1 minute (full workspace)
- **Documentation:** 15 files, fully organized and cross-referenced

## Documentation

**📚 [Complete Documentation Index](docs/README.md)**

### Getting Started
- **[Getting Started Guide](docs/GETTING_STARTED.md)** - Step-by-step tutorial for beginners
- **[FAQ](docs/FAQ.md)** - Frequently asked questions
- **[Contributing Guide](CONTRIBUTING.md)** - How to contribute to the project

### Core Guides
- **[Architecture](docs/ARCHITECTURE.md)** - System design, data flows, security model
- **[Deployment](docs/DEPLOYMENT.md)** - Installation, configuration, production setup

### Feature Documentation
- **[Cultural Identity](docs/identity/CULTURAL_IDENTITY.md)** - Heritage token standard, metadata schema
- **[IPFS Integration](docs/identity/IPFS_INTEGRATION.md)** - Decentralized multimedia storage
- **[P2P Networking](docs/network/P2P_NETWORKING.md)** - libp2p architecture, protocols
- **[Network CLI](docs/network/NETWORK_CLI.md)** - Network commands, multi-node setup
- **[Governance](docs/governance/GOVERNANCE.md)** - On-chain proposals and voting
- **[Wallet API](docs/api/WALLET_API.md)** - REST API reference
- **[Integration Tests](docs/tests/INTEGRATION_TESTS.md)** - Multi-node testing guide

### Additional Resources
- **[Scripts Documentation](scripts/README.md)** - Test scripts and integration testing
- **[CHANGELOG](CHANGELOG.md)** - Version history and release notes
- **[LICENSE](LICENSE-MIT)** / **[LICENSE](LICENSE-APACHE)** - Dual MIT/Apache-2.0 licensing

## Technology Stack

| Component | Technology |
|-----------|-----------|
| **Language** | Rust 2021 |
| **Consensus** | Proof-of-Work (SHA-256) |
| **Signatures** | Ed25519 |
| **Storage** | RocksDB |
| **Network** | libp2p (planned) |
| **CLI** | Clap |
| **Logging** | Tracing |

## Development Roadmap

### Phase 1: Core ✅ (Complete)
- Blockchain primitives
- PoW consensus & mining
- Persistent storage
- Wallet & node CLI

### Phase 2: Network ✅ (Complete)
- [x] P2P protocol specification
- [x] libp2p integration (Gossipsub, mDNS, Kademlia)
- [x] Block synchronization protocol
- [x] Transaction propagation
- [x] Network node CLI (`opensyria-node-cli network`)
- [x] Network daemon mode (background node with auto-mining)

### Phase 3: Ecosystem (Complete ✅)
- [x] Transaction pool (mempool)
- [x] Priority-based transaction selection
- [x] Network integration for tx propagation
- [x] Block explorer web UI
- [x] REST API (7 endpoints)
- [x] Mining pool support (proportional/PPS/PPLNS rewards)

### Phase 4: Cultural Integration (Complete ✅)
- [x] Identity token standard (11 tests)
- [x] Cultural heritage CLI
- [x] IPFS multimedia storage (upload/retrieve/link commands)
- [x] Content integrity verification (SHA-256)
- [x] 10 heritage token types (HeritageSite, TraditionalCraft, etc.)
- [x] 5 cultural categories (Ancient, Islamic, Ottoman, Modern, Contemporary)

### Phase 5: Governance & API (Complete ✅)
- [x] On-chain proposal system
- [x] Stake-weighted voting
- [x] Quorum and threshold checks
- [x] Governance CLI tool
- [x] Persistent storage
- [x] Node integration (create proposals, vote, process)
- [x] Automatic proposal execution
- [x] Wallet REST API (transaction submission, balance queries)
- [x] Multi-signature accounts (M-of-N threshold signing)

### Phase 6: Future Enhancements (Optional)
- [ ] Mining pool persistent daemon server
- [ ] Smart contracts (VM integration)
- [ ] Light clients (mobile/embedded)
- [ ] Heritage verification system
- [ ] Web interface for cultural tokens
- [ ] IPFS Cluster (multi-node pinning)
- [ ] Filecoin integration (incentivized storage)
- [ ] Multi-signature accounts
- [ ] Proof-of-Stake migration
- [ ] Smart contract VM
- [ ] Cross-chain bridges
- [ ] Layer-2 scaling

## Testing

```bash
# Run all tests
cargo test --all

# Run specific module tests
cargo test -p opensyria-core
cargo test -p opensyria-consensus
cargo test -p opensyria-storage
cargo test -p opensyria-identity

# Run with output
cargo test -- --nocapture
```

**Test Coverage:**
- Core: 19 tests ✅
- Storage: 7 tests ✅
- Identity: 11 tests ✅ (includes IPFS)
- Network: 2 tests ✅
- Mempool: 5 tests ✅
- Mining Pool: 5 tests ✅
- Governance: 23 tests ✅
- **Total: 72 passing tests**

**Test Scripts:**
- `scripts/test-network.sh` - Multi-node P2P testing
- `scripts/test-multisig.sh` - M-of-N signature testing
- `scripts/test-pool.sh` - Mining pool operations
- `scripts/test-ipfs.sh` - Heritage content storage
- `scripts/test-daemon.sh` - Network daemon mode
- `scripts/test-wallet-api.sh` - REST API endpoints

## Contributing

Contributions welcome! Please see `CONTRIBUTING.md` (coming soon) for guidelines.

**Areas for contribution:**
- P2P networking implementation
- Block explorer UI
- Wallet GUI
- Identity token blockchain integration
- Heritage verification system
- Mobile applications
- Cultural heritage documentation
- Arabic language improvements
- Test coverage

## Security

**Current Status:** Development / Testing Phase

⚠️ **Warning:** This software is in active development. Do not use for production or store real value.

**Security measures:**
- Ed25519 signatures
- SHA-256 hashing
- Nonce-based replay protection
- Chain validation
- State integrity checks

**Future enhancements:**
- Security audits
- Encrypted wallet storage
- Hardware wallet support
- Multi-signature accounts

## License

MIT OR Apache-2.0

## Contact

- **Repository:** https://github.com/OpenSyria/blockchain (placeholder)
- **Community:** TBD
- **Security:** security@opensyria.org (placeholder)

---

**Build the future of Syria's digital economy.**  
**ابنِ مستقبل الاقتصاد الرقمي السوري**
