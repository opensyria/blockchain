# Frequently Asked Questions (FAQ)

**Common questions about the OpenSyria blockchain**

---

## Table of Contents

### General Questions
- [What is OpenSyria?](#what-is-opensyria)
- [Who is this blockchain for?](#who-is-this-blockchain-for)
- [Is this production-ready?](#is-this-production-ready)
- [What makes OpenSyria different?](#what-makes-opensyria-different)
- [Is OpenSyria a fork of Bitcoin/Ethereum?](#is-opensyria-a-fork-of-bitcoinethereum)

### Technical Questions
- [What consensus mechanism does it use?](#what-consensus-mechanism-does-it-use)
- [What programming language is it written in?](#what-programming-language-is-it-written-in)
- [What are the system requirements?](#what-are-the-system-requirements)
- [Can I mine on my laptop?](#can-i-mine-on-my-laptop)
- [What's the block time?](#whats-the-block-time)
- [What's the block reward?](#whats-the-block-reward)

### Getting Started
- [How do I install it?](#how-do-i-install-it)
- [Do I need to download the entire blockchain?](#do-i-need-to-download-the-entire-blockchain)
- [How do I create a wallet?](#how-do-i-create-a-wallet)
- [Where are my private keys stored?](#where-are-my-private-keys-stored)
- [How do I backup my wallet?](#how-do-i-backup-my-wallet)

### Mining
- [How do I start mining?](#how-do-i-start-mining)
- [What's a good difficulty setting?](#whats-a-good-difficulty-setting)
- [Can I join a mining pool?](#can-i-join-a-mining-pool)
- [What's the expected hash rate?](#whats-the-expected-hash-rate)

### Networking
- [How do I connect to other nodes?](#how-do-i-connect-to-other-nodes)
- [Is there a public network?](#is-there-a-public-network)
- [Can I run a private network?](#can-i-run-a-private-network)
- [What ports does it use?](#what-ports-does-it-use)

### Cultural Identity
- [What are heritage tokens?](#what-are-heritage-tokens)
- [How do I create a heritage token?](#how-do-i-create-a-heritage-token)
- [What can I upload to IPFS?](#what-can-i-upload-to-ipfs)
- [Do I need IPFS installed?](#do-i-need-ipfs-installed)

### Governance
- [How does governance work?](#how-does-governance-work)
- [Who can vote?](#who-can-vote)
- [What can be changed via governance?](#what-can-be-changed-via-governance)
- [How do I create a proposal?](#how-do-i-create-a-proposal)

### Development
- [How do I contribute?](#how-do-i-contribute)
- [Where is the API documentation?](#where-is-the-api-documentation)
- [Can I build applications on top?](#can-i-build-applications-on-top)
- [Are there smart contracts?](#are-there-smart-contracts)

### Troubleshooting
- [Build fails with errors](#build-fails-with-errors)
- [IPFS daemon won't start](#ipfs-daemon-wont-start)
- [Network node won't connect](#network-node-wont-connect)
- [Transaction not included in block](#transaction-not-included-in-block)

---

## General Questions

### What is OpenSyria?

OpenSyria is a **sovereign blockchain** designed specifically for Syria and the Syrian diaspora. It combines:
- A decentralized digital currency (Digital Lira / SYL)
- A cultural heritage preservation system
- On-chain governance
- P2P networking
- IPFS integration for multimedia storage

Built from scratch in Rust, it's optimized for Syrian needs rather than being a generic blockchain.

### Who is this blockchain for?

**Primary Users:**
- **Syrians worldwide**: Inside Syria and across the diaspora
- **Heritage contributors**: Museums, historians, cultural organizations
- **Developers**: Building Syrian-focused dApps
- **Miners**: Running nodes and securing the network
- **Governance participants**: Shaping the blockchain's future

**Use Cases:**
- Preserving Syrian cultural heritage
- Peer-to-peer digital payments
- Decentralized applications
- Community governance
- Educational blockchain projects

### Is this production-ready?

**Status: Development/Testing Phase**

The blockchain is:
- ✅ **Feature-complete**: All core features implemented
- ✅ **Well-tested**: 72 passing unit/integration tests
- ✅ **Documented**: Comprehensive documentation
- ⚠️ **Not audited**: No external security audit yet
- ⚠️ **No mainnet**: Currently test network only

**Recommendation:** Use for development, testing, and education. **Do not store real value** until after security audits and mainnet launch.

### What makes OpenSyria different?

**Unique Features:**
1. **Cultural Heritage Focus**: Built-in system for preserving Syrian culture
2. **Bilingual Interface**: Arabic and English throughout
3. **Syrian-Specific Design**: Optimized for Syrian needs
4. **IPFS Integration**: Decentralized multimedia storage
5. **Comprehensive Governance**: On-chain decision making
6. **Mining Pools**: Built-in pool support
7. **Multi-signature Accounts**: Enhanced security
8. **Heritage-Aware Explorer**: Specialized blockchain browser

### Is OpenSyria a fork of Bitcoin/Ethereum?

**No.** OpenSyria is built **from scratch** in Rust. While it shares concepts with Bitcoin (PoW consensus, UTXO-like model), the codebase is completely original.

**Similarities:**
- Proof-of-Work consensus
- Ed25519 signatures (different from Bitcoin's ECDSA)
- Block-based structure

**Differences:**
- Written in Rust (not C++)
- Cultural identity system
- Integrated governance
- IPFS support
- Modern networking (libp2p)
- Bilingual interface

---

## Technical Questions

### What consensus mechanism does it use?

**Currently:** Proof-of-Work (PoW) with SHA-256 hashing

**Mining Algorithm:**
```
hash = SHA256(SHA256(block_header))
valid if hash < target_difficulty
```

**Future:** Migration path to Proof-of-Stake (PoS) is planned.

### What programming language is it written in?

**Rust** (edition 2021)

**Why Rust?**
- Memory safety without garbage collection
- High performance (comparable to C/C++)
- Modern tooling and package management
- Growing blockchain ecosystem
- Prevents entire categories of bugs

### What are the system requirements?

**Minimum:**
- OS: Linux, macOS, or Windows (WSL2)
- CPU: 2 cores
- RAM: 2GB
- Storage: 10GB free space
- Rust: 1.75+

**Recommended:**
- CPU: 4+ cores
- RAM: 8GB
- Storage: 50GB SSD
- Network: Stable internet connection

**For Mining:**
- CPU: 8+ cores recommended
- Adjustable difficulty allows laptop mining

### Can I mine on my laptop?

**Yes!** Mining difficulty is adjustable.

**Recommended settings for laptop:**
```bash
./target/release/opensyria-node-cli mine --blocks 10 --difficulty 16
```

**Performance:**
- Difficulty 16: ~1-3 seconds per block (M1/M2 MacBook)
- Difficulty 20: ~10-30 seconds per block
- Difficulty 24: ~1-5 minutes per block

Lower difficulty = faster mining = good for testing.

### What's the block time?

**Target:** 60 seconds (configurable)

**Current:** Depends on difficulty setting
- Difficulty adjustment is manual (governance can change this)
- In production, difficulty auto-adjusts to maintain target time

### What's the block reward?

**Current:** 50 SYL per block

**Future:** Can be adjusted via governance proposals

**Supply Schedule:**
- No halving schedule currently
- Total supply unlimited (inflationary)
- Governance can vote to change this

---

## Getting Started

### How do I install it?

**Quick Start:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone repository
git clone https://github.com/OpenSyria/blockchain.git opensyria
cd opensyria

# Build
cargo build --release

# Initialize
./target/release/opensyria-node-cli init --difficulty 16
```

**See also:** [Getting Started Guide](docs/GETTING_STARTED.md)

### Do I need to download the entire blockchain?

**For full node:** Yes, but it's small initially (< 1MB for genesis)

**For light client:** Not implemented yet (planned future feature)

**Current:** The blockchain starts fresh when you initialize. There's no global mainnet to download yet.

### How do I create a wallet?

```bash
./target/release/wallet create --name mywalletname
```

**Output includes:**
- Public key (your address)
- Balance (initially 0)
- Nonce (transaction counter)

**Private key** is saved to `~/.opensyria/wallets/mywalletname.key`

### Where are my private keys stored?

**Location:** `~/.opensyria/wallets/`

**Format:** PEM-encoded Ed25519 private keys

**Files:**
```
~/.opensyria/
└── wallets/
    ├── alice.key
    ├── bob.key
    └── miner.key
```

**Security:** 
- File permissions: 0600 (readable only by you)
- **Never share these files**
- **Backup immediately**

### How do I backup my wallet?

**Method 1: Copy key files**
```bash
# Backup all wallets
cp -r ~/.opensyria/wallets ~/backup/opensyria-wallets-$(date +%Y%m%d)

# Backup specific wallet
cp ~/.opensyria/wallets/alice.key ~/backup/alice-key-backup.pem
```

**Method 2: Secure cloud storage**
```bash
# Encrypt first
gpg -c ~/.opensyria/wallets/alice.key
# Upload alice.key.gpg to cloud storage
```

**⚠️ Critical:** Loss of private key = permanent loss of funds. No recovery possible.

---

## Mining

### How do I start mining?

**Basic mining:**
```bash
./target/release/opensyria-node-cli mine --blocks 10 --difficulty 16
```

**With verbose output:**
```bash
./target/release/opensyria-node-cli mine --blocks 10 --difficulty 16 --verbose
```

**Background daemon with auto-mining:**
```bash
./target/release/opensyria-node-cli network daemon --mine --difficulty 16
```

### What's a good difficulty setting?

**By use case:**

| Use Case | Difficulty | Block Time |
|----------|-----------|------------|
| Testing/Development | 12-16 | 1-5 seconds |
| Laptop Mining | 16-20 | 5-30 seconds |
| Desktop Mining | 20-24 | 30 sec - 2 min |
| Production Network | 24-28 | 1-10 minutes |

**Recommendation:** Start with 16, adjust based on your hardware.

### Can I join a mining pool?

**Yes!** Mining pool support is built-in.

```bash
# Initialize a pool (operator)
./target/release/opensyria-node-cli pool init \
  --operator <your-pubkey> \
  --fee 2 \
  --method proportional

# Register as miner
./target/release/opensyria-node-cli pool register <miner-pubkey>

# View pool stats
./target/release/opensyria-node-cli pool stats
```

**Reward methods:**
- Proportional: Based on shares contributed
- PPS: Pay Per Share (fixed payment)
- PPLNS: Pay Per Last N Shares

### What's the expected hash rate?

**Benchmarks** (M1/M2 MacBook):
- ~1.6 million hashes/second (single-threaded)
- ~12-15 million H/s (8 cores)

**Intel/AMD:**
- Varies by CPU model
- Generally 500K - 3M H/s per core

**To benchmark:**
```bash
./target/release/opensyria-node-cli mine --blocks 1 --difficulty 20 --verbose
# Watch hash rate in output
```

---

## Networking

### How do I connect to other nodes?

**Auto-discovery (local network):**
```bash
./target/release/opensyria-node-cli network start
# Automatically finds peers via mDNS
```

**Manual bootstrap:**
```bash
./target/release/opensyria-node-cli network start \
  --bootstrap /ip4/192.168.1.100/tcp/9000/p2p/12D3KooW...
```

**Custom listen address:**
```bash
./target/release/opensyria-node-cli network start \
  --listen /ip4/0.0.0.0/tcp/9000
```

### Is there a public network?

**Not yet.** Currently:
- Test networks only
- Local multi-node testing
- Development phase

**Future:** Public mainnet after security audits and testing.

### Can I run a private network?

**Yes!** Perfect for:
- Development
- Testing
- Private organizational use

**Setup:**
1. Initialize multiple nodes with same genesis block
2. Run network on isolated network segment
3. Use custom bootstrap peers

**Example:**
```bash
# Node 1
./target/release/opensyria-node-cli -d /tmp/node1 init --difficulty 16
./target/release/opensyria-node-cli -d /tmp/node1 network start -l /ip4/127.0.0.1/tcp/9000

# Node 2
./target/release/opensyria-node-cli -d /tmp/node2 init --difficulty 16
./target/release/opensyria-node-cli -d /tmp/node2 network start \
  -l /ip4/127.0.0.1/tcp/9001 \
  -b /ip4/127.0.0.1/tcp/9000/p2p/<node1-peer-id>
```

### What ports does it use?

**Default ports:**
- **9000/tcp**: P2P networking (libp2p)
- **3000/tcp**: Block explorer web UI
- **8080/tcp**: Wallet REST API
- **5001/tcp**: IPFS API (if using IPFS)
- **8080/tcp**: IPFS Gateway (if using IPFS)

**Customize:**
```bash
# Custom P2P port
--listen /ip4/0.0.0.0/tcp/9999

# Custom API port
--port 8888
```

---

## Cultural Identity

### What are heritage tokens?

**Heritage tokens** are digital certificates representing elements of Syrian culture:
- Heritage sites (Palmyra, Damascus Old City, etc.)
- Traditional crafts (Damascus steel, textiles, etc.)
- Oral traditions (stories, poetry, music)
- Historical figures
- Artifacts and documents

**Properties:**
- Non-fungible (unique)
- Immutable metadata
- IPFS multimedia support
- Community verifiable
- Bilingual (Arabic/English)

### How do I create a heritage token?

```bash
./target/release/identity create \
  --id unique-token-id \
  --name "Token Name" \
  --name-ar "الاسم بالعربية" \
  --description "Description of the heritage" \
  --token-type <site|craft|tradition|figure|artifact|music|dance|cuisine|language|document> \
  --category <ancient|islamic|ottoman|modern|contemporary> \
  --city "City name"
```

**Example:**
```bash
./target/release/identity create \
  --id umayyad-mosque-001 \
  --name "Umayyad Mosque" \
  --name-ar "الجامع الأموي" \
  --description "Great Mosque of Damascus, built 705 AD" \
  --token-type site \
  --category islamic \
  --city Damascus
```

### What can I upload to IPFS?

**Supported file types:**
- **Images**: JPG, PNG, GIF, SVG
- **Videos**: MP4, WebM, MOV
- **Audio**: MP3, WAV, OGG
- **Documents**: PDF, JSON, TXT

**Size limits:** Determined by your IPFS node configuration (typically several GB)

**Best practices:**
- Compress large files
- Use appropriate formats
- Include metadata
- Pin important content

### Do I need IPFS installed?

**For heritage multimedia:** Yes

**Installation:**
```bash
# macOS
brew install ipfs

# Linux
wget https://dist.ipfs.io/go-ipfs/v0.18.0/go-ipfs_v0.18.0_linux-amd64.tar.gz
tar -xvzf go-ipfs_v0.18.0_linux-amd64.tar.gz
cd go-ipfs && sudo ./install.sh

# Initialize and start
ipfs init
ipfs daemon
```

**For basic token creation:** No (IPFS optional)

---

## Governance

### How does governance work?

**Process:**
1. **Proposal Creation**: Anyone can create a proposal
2. **Voting Period**: Stakeholders vote (weighted by balance)
3. **Quorum Check**: Minimum participation required
4. **Execution**: Passed proposals are automatically executed

**Proposal Types:**
- Text proposals (non-binding)
- Minimum fee adjustments
- Block size limits
- Block rewards
- Protocol upgrades
- Treasury spending

### Who can vote?

**Anyone with SYL balance** can vote.

**Voting power** = Account balance (stake-weighted voting)

**Example:**
- Alice has 100 SYL → 100 voting power
- Bob has 50 SYL → 50 voting power
- Charlie has 10 SYL → 10 voting power

### What can be changed via governance?

**Current proposal types:**

1. **Difficulty Adjustment**: Change mining difficulty
2. **Minimum Fee**: Set transaction fee floor
3. **Block Size Limit**: Maximum block size
4. **Block Reward**: Mining reward amount
5. **Protocol Upgrade**: Activate new features
6. **Treasury Spending**: Allocate funds
7. **Text Proposals**: Non-binding recommendations

**Future:** Smart contract parameters, staking requirements, etc.

### How do I create a proposal?

```bash
./target/release/opensyria-node-cli governance propose \
  --title "Proposal Title" \
  --description "Detailed description" \
  --type text \
  --proposer <your-pubkey>
```

**Vote on proposal:**
```bash
./target/release/opensyria-node-cli governance vote <proposal-id> \
  --choice yes \
  --voter <your-pubkey>
```

**See also:** [Governance Documentation](docs/governance/GOVERNANCE.md)

---

## Development

### How do I contribute?

**See:** [CONTRIBUTING.md](CONTRIBUTING.md)

**Quick start:**
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Write tests
5. Submit pull request

**Areas needing help:**
- Core blockchain optimizations
- P2P networking
- UI/UX improvements
- Documentation
- Testing
- Arabic translations

### Where is the API documentation?

**REST API:** [docs/api/WALLET_API.md](docs/api/WALLET_API.md)

**Rust API docs:**
```bash
cargo doc --all --no-deps --open
```

**Architecture:** [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

### Can I build applications on top?

**Yes!** Use the REST API:

```bash
# Start API server
./target/release/opensyria-wallet-api --port 8080

# Make requests
curl http://localhost:8080/api/v1/blockchain/info
curl http://localhost:8080/api/v1/account/{address}/balance
```

**Example applications:**
- Mobile wallets
- Web dashboards
- Heritage catalogues
- Trading platforms
- Analytics tools

### Are there smart contracts?

**Not yet.** Smart contracts are planned for future versions.

**Workarounds:**
- Use multi-signature accounts for escrow
- Build application logic off-chain
- Use governance for on-chain parameters

---

## Troubleshooting

### Build fails with errors

**Solution 1: Update Rust**
```bash
rustup update stable
cargo clean
cargo build --release
```

**Solution 2: Check dependencies**
```bash
# macOS
brew install openssl

# Ubuntu/Debian
sudo apt-get install libssl-dev pkg-config
```

**Solution 3: Check Rust version**
```bash
rustc --version
# Should be 1.75+
```

### IPFS daemon won't start

**Check if already running:**
```bash
ps aux | grep ipfs
curl http://127.0.0.1:5001/api/v0/version
```

**Kill and restart:**
```bash
pkill ipfs
ipfs daemon
```

**Reinitialize:**
```bash
rm -rf ~/.ipfs
ipfs init
ipfs daemon
```

### Network node won't connect

**Check firewall:**
```bash
# macOS
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add ./target/release/opensyria-node-cli

# Linux (ufw)
sudo ufw allow 9000/tcp
```

**Check listening address:**
```bash
# Use 0.0.0.0 instead of 127.0.0.1 for external connections
--listen /ip4/0.0.0.0/tcp/9000
```

**Verify peer ID:**
```bash
# Make sure bootstrap peer ID is correct
--bootstrap /ip4/<ip>/tcp/<port>/p2p/<correct-peer-id>
```

### Transaction not included in block

**Possible causes:**

1. **Transaction not in mempool**
   ```bash
   # Check mempool
   curl http://localhost:8080/api/v1/mempool/status
   ```

2. **No mining happening**
   ```bash
   # Mine a block manually
   ./target/release/opensyria-node-cli mine --blocks 1
   ```

3. **Fee too low**
   ```bash
   # Use higher fee
   --fee 0.01
   ```

4. **Invalid nonce**
   ```bash
   # Check current nonce
   ./target/release/wallet info <wallet-name>
   # Use correct nonce value
   ```

---

## Still Have Questions?

**Documentation:**
- [Getting Started Guide](docs/GETTING_STARTED.md)
- [Architecture Guide](docs/ARCHITECTURE.md)
- [Deployment Guide](docs/DEPLOYMENT.md)
- [Full Documentation Index](docs/README.md)

**Community:**
- GitHub Issues: Bug reports and questions
- GitHub Discussions: General conversation
- Discord: Coming soon
- Telegram: Coming soon

**Contact:**
- Email: opensyria.net@gmail.com
- Security: opensyria.net@gmail.com

---

**Last Updated:** November 18, 2025  
**Version:** 0.1.0
