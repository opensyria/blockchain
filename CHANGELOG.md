# Changelog

All notable changes to the OpenSyria blockchain project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Future Enhancements
- Mining pool persistent daemon server
- Smart contracts (VM integration)
- Light clients for mobile/embedded devices
- Heritage verification system
- Web interface for cultural tokens
- IPFS Cluster for multi-node pinning
- Filecoin integration for incentivized storage

## [0.1.0] - 2025-11-18

### Added - IPFS Integration
- IPFS client module for decentralized multimedia storage (270 lines)
- Upload methods: `upload_file()`, `upload_bytes()`, `upload_text()`, `upload_json()`
- Retrieve methods: `retrieve()`, `retrieve_text()`, `retrieve_json()`
- Pin/unpin for content persistence
- MIME type detection (jpg, png, mp4, pdf, json, etc.)
- SHA-256 content hashing for integrity verification
- ContentMetadata struct with CID tracking
- Identity token `ipfs_cid` field for linking heritage multimedia
- CLI commands: `identity upload`, `identity retrieve`, `identity link`
- Dependencies: tokio (async runtime), reqwest (HTTP client), sha2 (hashing)
- Documentation: IPFS_INTEGRATION.md (500+ lines), IPFS_ARCHITECTURE.md
- Test script: `test-ipfs.sh` for integration testing

### Added - Documentation Reorganization
- Comprehensive documentation index (`docs/README.md`)
- Organized docs into subdirectories:
  - `identity/` - Cultural heritage system (5 files)
  - `network/` - P2P networking (3 files)
  - `governance/` - On-chain governance (1 file)
  - `api/` - REST APIs (1 file)
  - `tests/` - Testing guides (2 files)
- Cross-references added between related documentation
- Role-based navigation (Developer, Node Operator, Heritage User, etc.)
- Topic-based organization (Blockchain Core, Cultural Heritage, Networking, etc.)

### Fixed
- Removed duplicate symlinks from docs root
- Updated all internal documentation paths to subdirectory structure
- Updated main README with new documentation paths

## [0.0.9] - 2025-11-17

### Added - Mining Pool Support
- Mining pool coordinator (`MiningPool` struct)
- Share validation and tracking
- Three reward distribution methods:
  - Proportional (shares proportional to contribution)
  - PPS (Pay Per Share - fixed payment)
  - PPLNS (Pay Per Last N Shares - with window)
- Configurable pool fees (0-100%)
- Minimum payout thresholds (default: 1 Lira)
- Pool statistics (total shares, miners, blocks found)
- Miner statistics (shares submitted, rewards earned)
- Pool CLI commands:
  - `pool init` - Initialize new pool
  - `pool stats` - View pool statistics
  - `pool miners` - List all miners
  - `pool miner <address>` - View miner details
  - `pool register <address>` - Register miner
  - `pool payout <address>` - Process payout
- 5 unit tests for pool operations
- Test script: `test-pool.sh`
- Documentation in node CLI integration

### Tests
- Core: 19 tests ✅
- Storage: 7 tests ✅
- Identity: 11 tests ✅
- Mempool: 5 tests ✅
- Mining Pool: 5 tests ✅
- Governance: 23 tests ✅
- Total: 70+ passing tests

## [0.0.8] - 2025-11-16

### Added - Multi-signature Accounts
- M-of-N threshold signature support
- `MultisigAccount` struct with required signers and threshold
- `MultisigTransaction` for aggregating signatures
- Deterministic address generation (SHA-256 of sorted signers + threshold)
- Offline signing workflow using JSON transaction files
- Storage integration for multisig accounts (RocksDB)
- CLI commands:
  - `multisig create` - Create M-of-N account
  - `multisig info` - View account details
  - `multisig create-tx` - Create unsigned transaction
  - `multisig sign` - Sign transaction
  - `multisig submit` - Submit signed transaction
- 8 unit tests for multisig functionality
- Test script: `test-multisig.sh`
- Example: 2-of-3 multisig for treasury accounts

### Documentation
- Added multisig usage examples to README
- Updated architecture documentation

## [0.0.7] - 2025-11-15

### Added - Network Daemon Mode
- Persistent background P2P node
- Auto-mining capability
- Network monitoring and statistics
- Daemon CLI commands:
  - `network daemon start` - Start daemon
  - `network daemon stop` - Stop daemon
  - `network daemon status` - Check status
- Background process management
- Test script: `test-daemon.sh`
- Documentation: Network daemon guide

## [0.0.6] - 2025-11-14

### Added - Wallet REST API
- HTTP REST API for wallet operations
- Endpoints:
  - `POST /api/transaction` - Submit transaction
  - `GET /api/balance/:address` - Query balance
  - `GET /api/blockchain/info` - Chain info
  - `GET /api/blockchain/height` - Current height
  - `GET /api/block/:hash` - Get block by hash
  - `GET /api/transaction/:hash` - Get transaction
- JSON request/response format
- CORS support for web clients
- Error handling and validation
- Test script: `test-wallet-api.sh`
- Documentation: WALLET_API.md

## [0.0.5] - 2025-11-13

### Added - Governance System
- On-chain proposal system
- 5 proposal types:
  - TextProposal (community discussions)
  - ParameterChange (adjust consensus parameters)
  - BlockSizeChange (modify block size limits)
  - TreasurySpend (allocate funds)
  - ProtocolUpgrade (network upgrades)
- Stake-weighted voting mechanism
- Quorum requirements (default: 30%)
- Approval thresholds (default: 60%)
- Automatic proposal execution
- Proposal lifecycle: Pending → Active → Succeeded/Failed → Executed
- Node CLI integration:
  - `governance create` - Create proposal
  - `governance vote` - Vote on proposal
  - `governance list` - List all proposals
  - `governance info` - View proposal details
  - `governance process` - Execute approved proposals
- Persistent storage (RocksDB)
- 23 unit tests for governance operations
- Documentation: GOVERNANCE.md

### Tests
- Governance: 23 passing tests ✅
- Proposal creation and validation
- Voting mechanisms
- Quorum and threshold checks
- Automatic execution

## [0.0.4] - 2025-11-12

### Added - Block Explorer
- REST API backend (7 endpoints)
- React-based web UI
- Real-time blockchain visualization
- Features:
  - Recent blocks view
  - Block details (transactions, hash, timestamp)
  - Transaction details (sender, receiver, amount)
  - Chain statistics (height, difficulty, total supply)
  - Search by block height or hash
- 3 integration tests
- Documentation: Explorer setup guide

### Added - Transaction Pool (Mempool)
- Priority-based transaction selection
- Fee-based ordering
- Transaction validation
- Network propagation integration
- 5 unit tests for mempool operations
- Documentation: Mempool architecture

## [0.0.3] - 2025-11-11

### Added - P2P Networking
- libp2p 0.53 integration
- Gossipsub protocol for pub/sub messaging
- mDNS peer discovery (local network)
- Kademlia DHT for peer routing
- Block synchronization protocol
- Transaction propagation
- Multi-node testing support
- Network CLI commands:
  - `network start` - Start P2P node
  - `network peers` - List connected peers
  - `network dial <multiaddr>` - Connect to peer
  - `network status` - Show network status
- 2 unit tests for network operations
- Test script: `test-network.sh`
- Documentation: P2P_NETWORKING.md, NETWORK_CLI.md, NETWORK_IMPLEMENTATION.md

### Added - Integration Tests
- Multi-node blockchain testing
- 5 integration tests covering:
  - Multi-node network setup
  - Transaction propagation
  - Block synchronization
  - Chain consensus
- Documentation: INTEGRATION_TESTS.md

## [0.0.2] - 2025-11-10

### Added - Cultural Identity System
- Heritage token standard
- 10 token types:
  - HeritageSite (monuments, archaeological sites)
  - TraditionalCraft (Damascus steel, mosaic art)
  - HistoricalDocument (manuscripts, archives)
  - PerformingArts (music, dance, theater)
  - CulinaryHeritage (traditional cuisine)
  - OralTradition (folk tales, storytelling)
  - LanguageHeritage (dialects, scripts)
  - CommunityIdentity (regional traditions)
  - PersonalContribution (individual achievements)
  - DigitalCulture (digital preservation)
- 5 cultural categories:
  - Ancient (pre-Islamic civilizations)
  - Islamic (medieval Islamic era)
  - Ottoman (Ottoman period)
  - Modern (20th century)
  - Contemporary (21st century)
- Metadata schema:
  - Location (city, coordinates)
  - Historical period
  - UNESCO status
  - Tags and descriptions
- Identity CLI:
  - `identity create` - Create heritage token
  - `identity list` - List all tokens
  - `identity info` - View token details
  - `identity examples` - Show Syrian heritage examples
- 9 unit tests for identity operations
- Bilingual support (Arabic/English)
- Documentation: CULTURAL_IDENTITY.md, CULTURAL_IDENTITY_SUMMARY.md, SHOWCASE.md

## [0.0.1] - 2025-11-09

### Added - Core Blockchain
- Block structure with header and transactions
- Merkle tree implementation
- SHA-256 hashing for block IDs
- Ed25519 signature scheme
- Transaction structure (sender, receiver, amount, signature)
- Genesis block creation
- Chain validation and reorganization
- 11 unit tests for core functionality

### Added - Consensus
- Proof-of-Work (PoW) consensus mechanism
- SHA-256 mining algorithm
- Dynamic difficulty adjustment
- Target block time: 60 seconds
- Mining rewards
- Block validation
- 5 unit tests for consensus

### Added - Storage Layer
- RocksDB integration
- Blockchain persistence (blocks by height/hash)
- State database (account balances)
- Transaction history
- Efficient key-value storage
- 7 unit tests for storage operations

### Added - Wallet CLI
- Ed25519 key generation
- Address derivation (SHA-256 of public key)
- Transaction signing
- Balance queries
- Bilingual CLI (Arabic/English)
- Commands:
  - `wallet create` - Generate new wallet
  - `wallet balance` - Check balance
  - `wallet send` - Send transaction

### Added - Node CLI
- Full node implementation
- Mining commands
- Chain inspection tools
- Block/transaction queries
- Commands:
  - `node init` - Initialize blockchain
  - `node mine` - Start mining
  - `node info` - Show chain info
  - `node blocks` - List recent blocks
  - `node tx` - View transaction

### Documentation
- README.md with quick start guide
- ARCHITECTURE.md - System design
- DEPLOYMENT.md - Installation guide
- Examples and usage instructions

### Tests
- Core: 11 tests ✅
- Consensus: 5 tests ✅
- Storage: 7 tests ✅
- Total: 23 passing tests

## [0.0.0] - 2025-11-08

### Project Initialization
- Cargo workspace setup
- Project structure defined
- Crate organization:
  - `core` - Blockchain primitives
  - `consensus` - PoW implementation
  - `storage` - RocksDB persistence
  - `wallet` - Wallet CLI
  - `node-cli` - Node management
  - `identity` - Cultural identity system
  - `network` - P2P networking
  - `mempool` - Transaction pool
  - `governance` - On-chain governance
  - `explorer-backend` - Block explorer API
  - `mining-pool` - Mining pool support
- MIT License
- Repository initialized

---

## Version History Summary

- **0.1.0** (2025-11-18): IPFS Integration + Documentation Reorganization ✅
- **0.0.9** (2025-11-17): Mining Pool Support ✅
- **0.0.8** (2025-11-16): Multi-signature Accounts ✅
- **0.0.7** (2025-11-15): Network Daemon Mode ✅
- **0.0.6** (2025-11-14): Wallet REST API ✅
- **0.0.5** (2025-11-13): Governance System ✅
- **0.0.4** (2025-11-12): Block Explorer + Mempool ✅
- **0.0.3** (2025-11-11): P2P Networking + Integration Tests ✅
- **0.0.2** (2025-11-10): Cultural Identity System ✅
- **0.0.1** (2025-11-09): Core Blockchain Implementation ✅
- **0.0.0** (2025-11-08): Project Initialization ✅

---

**Current Status:** Production Ready ✅  
**Next Milestone:** v1.0.0 - Public Launch
