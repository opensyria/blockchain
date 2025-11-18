# Getting Started with Open Syria Blockchain

**مرحباً! Welcome to the Digital Lira blockchain.**

This tutorial will guide you through your first steps with the Open Syria blockchain, from installation to creating your first heritage token.

---

## Table of Contents

1. [What is Open Syria?](#what-is-open-syria)
2. [Installation](#installation)
3. [First Steps](#first-steps)
4. [Creating Your First Wallet](#creating-your-first-wallet)
5. [Mining Your First Blocks](#mining-your-first-blocks)
6. [Sending Transactions](#sending-transactions)
7. [Creating Heritage Tokens](#creating-heritage-tokens)
8. [Joining the Network](#joining-the-network)
9. [Next Steps](#next-steps)
10. [Getting Help](#getting-help)

---

## What is Open Syria?

Open Syria is a **sovereign blockchain** built specifically for Syria and the Syrian diaspora. It's designed to:

- 🏛️ **Preserve Syrian Culture**: Tokenize and protect 5,000+ years of heritage
- 💰 **Power the Digital Lira**: A decentralized Syrian digital currency
- 🌍 **Connect Syrians Worldwide**: Unite communities across borders
- ✊ **Ensure Digital Sovereignty**: Syrian technology, owned by Syrians

**Key Features:**
- Proof-of-Work consensus (like Bitcoin)
- Ed25519 cryptography
- Cultural identity token system
- P2P networking
- On-chain governance
- IPFS integration for heritage multimedia
- Bilingual interface (Arabic/English)

---

## Installation

### Prerequisites

**You'll need:**
- A computer running macOS, Linux, or Windows (WSL2)
- 10GB free disk space
- Internet connection
- 30 minutes of time

### Step 1: Install Rust

Open your terminal and run:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then restart your terminal

# Verify installation
rustc --version
cargo --version
```

You should see output like:
```
rustc 1.75.0 (82e1608df 2023-12-21)
cargo 1.75.0 (1d8b05cdd 2023-11-20)
```

### Step 2: Download Open Syria

```bash
# Clone the repository
git clone https://github.com/OpenSyria/blockchain.git opensyria
cd opensyria

# Or download and extract the ZIP from GitHub
```

### Step 3: Build the Software

```bash
# Build all components (takes ~1 minute)
cargo build --release

# Verify build succeeded
ls target/release/
```

You should see these binaries:
- `opensyria-node-cli` - Full node and blockchain manager
- `wallet` - Wallet for transactions
- `identity` - Cultural heritage token manager
- `explorer` - Block explorer web interface
- `governance-cli` - Governance system
- `opensyria-wallet-api` - REST API server

**Note:** If the build fails, see [Troubleshooting](#troubleshooting) below.

---

## First Steps

### Initialize Your Node

```bash
# Initialize the blockchain with difficulty 16
./target/release/opensyria-node-cli init --difficulty 16
```

Output:
```
✅ Blockchain initialized
📁 Data directory: /Users/you/.opensyria/node
⛏️  Difficulty: 16
🔗 Genesis block created
```

### Check Node Status

```bash
./target/release/opensyria-node-cli info
```

Output:
```
📊 Blockchain Info
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Height:           0
Difficulty:       16
Total Supply:     0 SYL
Pending TXs:      0
Genesis Hash:     000015a3b7c8d9e2f4a1b5c8...
```

**Congratulations!** 🎉 You now have a working blockchain node.

---

## Creating Your First Wallet

### Generate a New Wallet

```bash
./target/release/wallet create --name alice
```

Output:
```
✅ Wallet created successfully

👤 Name: alice
🔑 Public Key:  8f3e9d7c2a1b5f4e3d2c1a9b8f7e6d5c4b3a2f1e
💰 Balance:     0.000000 SYL
🔢 Nonce:       0

⚠️  Private key saved to: ~/.opensyria/wallets/alice.key
    KEEP THIS FILE SAFE! Anyone with this key controls the funds.
```

### View Your Wallet

```bash
./target/release/wallet info alice
```

### Generate More Wallets

```bash
# Create wallets for testing
./target/release/wallet create --name bob
./target/release/wallet create --name miner
```

---

## Mining Your First Blocks

Mining creates new blocks and rewards you with Digital Lira (SYL).

### Mine 5 Blocks

```bash
./target/release/opensyria-node-cli mine --blocks 5 --difficulty 16 --verbose
```

Output:
```
⛏️  Mining 5 blocks at difficulty 16...

🔨 Mining block 1...
   Nonce: 0 | Hash: a3f7b9...
   Nonce: 10000 | Hash: e8d2c5...
   Nonce: 23451 | Hash: 0000f8a3b7...
✅ Mined block 1 (nonce: 23451, hash: 0000f8a3b7...)

🔨 Mining block 2...
   ...
✅ Mined block 2 (nonce: 67890, hash: 00003d9e2f...)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ Mining complete!
📦 Blocks mined: 5
⏱️  Total time: 12.3 seconds
⚡ Avg hash rate: 1,621,374 H/s
```

### Check Updated Chain

```bash
./target/release/opensyria-node-cli info
```

Output:
```
Height:           5
Total Supply:     250 SYL  (5 blocks × 50 SYL reward)
```

**You now have a blockchain with 5 blocks!** ⛏️

---

## Sending Transactions

### Check Your Balance

First, let's check miner wallet balance (if you mined to a specific address):

```bash
./target/release/wallet info miner
```

### Create a Transaction

```bash
# Get recipient's public key
RECIPIENT=$(./target/release/wallet info bob | grep "Public Key" | awk '{print $3}')

# Send 10 SYL from alice to bob
./target/release/wallet send \
  --from alice \
  --to $RECIPIENT \
  --amount 10.0 \
  --fee 0.01 \
  --nonce 0
```

Output:
```
✅ Transaction created

📤 From:     8f3e9d7c2a1b5f4e3d2c1a9b8f7e6d5c4b3a2f1e
📥 To:       7a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b
💰 Amount:   10.000000 SYL
💸 Fee:      0.010000 SYL
🔢 Nonce:    0
✍️  Signature: 3f9e8d7c...

Transaction hash: 9a8b7c6d5e4f3a2b1c0d9e8f7a6b5c4d3e2f1a0b
```

### Mine the Transaction into a Block

```bash
# Mine 1 more block to include the transaction
./target/release/opensyria-node-cli mine --blocks 1 --difficulty 16
```

### Verify Balance Changed

```bash
./target/release/wallet info bob
```

Output:
```
💰 Balance: 10.000000 SYL
```

**Success!** You've sent your first transaction. 💸

---

## Creating Heritage Tokens

Heritage tokens preserve Syrian culture on the blockchain.

### Explore Heritage Examples

```bash
./target/release/identity examples
```

Output:
```
🏛️  Syrian Heritage Token Examples

═══════════════════════════════════════════════════════════
📍 Heritage Site: Palmyra
───────────────────────────────────────────────────────────
🆔 ID:       palmyra-ruins
📛 Name:     Ruins of Palmyra | آثار تدمر
🏛️  Type:     Heritage Site
📅 Category: Ancient
🌍 Location: Homs Governorate
📝 Description:
   Ancient Semitic city, UNESCO World Heritage Site.
   Contains monumental ruins of a great city...
═══════════════════════════════════════════════════════════

... (10 examples shown)
```

### Create Your First Heritage Token

```bash
./target/release/identity create \
  --id my-damascus-steel-001 \
  --name "Damascus Steel Blade" \
  --name-ar "نصل الفولاذ الدمشقي" \
  --description "Traditional forged steel blade with characteristic watered pattern" \
  --token-type craft \
  --category islamic \
  --city Damascus
```

Output:
```
✅ Heritage token created successfully

🆔 ID:          my-damascus-steel-001
👤 Owner:       8f3e9d7c2a1b5f4e3d2c1a9b8f7e6d5c4b3a2f1e
🏛️  Type:        Traditional Craft
📅 Category:    Islamic Era
📍 Location:    Damascus
📛 Name:        Damascus Steel Blade
📛 Name (AR):   نصل الفولاذ الدمشقي
⏰ Created:     2025-11-18 15:30:42 UTC
```

### List All Tokens

```bash
./target/release/identity list
```

### View Token Details

```bash
./target/release/identity info my-damascus-steel-001
```

### Upload Heritage Media to IPFS

**Note:** Requires IPFS installed and running.

```bash
# Install IPFS (macOS)
brew install ipfs

# Initialize and start IPFS daemon
ipfs init
ipfs daemon &

# Upload an image to IPFS
./target/release/identity upload \
  --file damascus-steel.jpg \
  --token-id my-damascus-steel-001
```

Output:
```
📤 Uploading to IPFS...

✅ Upload successful!

📦 CID:         QmXyz123abc...
📁 Filename:    damascus-steel.jpg
📏 Size:        2.4 MB
🎨 MIME Type:   image/jpeg
🔒 Content Hash: a3f9e8d7c6b5a4f3e2d1c0b9a8f7e6d5...
🔗 Gateway URL:  http://127.0.0.1:8080/ipfs/QmXyz123abc...

Token my-damascus-steel-001 linked to IPFS content
```

**You've just preserved Syrian heritage on a decentralized network!** 🏛️

---

## Joining the Network

### Start a Network Node

```bash
# Start P2P network node
./target/release/opensyria-node-cli network start
```

Output:
```
🚀 Starting P2P network node...

🔑 Peer ID: 12D3KooWABC123XYZ...
🎧 Listening on: /ip4/127.0.0.1/tcp/9000

🔍 Discovering peers via mDNS (local network)...
→ Peer connected: 12D3KooWDEF456...
📦 New block received: height 6 (hash: 0000a3f9...)
💸 New transaction: 5.5 SYL
```

### Run as Background Daemon

```bash
# Start daemon with auto-mining enabled
./target/release/opensyria-node-cli network daemon \
  --mine \
  --difficulty 16
```

Output:
```
🚀 Daemon running... (Press Ctrl+C to stop)

💫 Status check - height: 5, pending txs: 0
⛏️  No transactions to mine

💫 Status check - height: 5, pending txs: 2
⛏️  Mining block with 2 transactions...
✅ Mined block at height 6
```

**Your node is now part of the network!** 🌍

---

## Next Steps

Congratulations! You've completed the basics. Here's what to explore next:

### 🎯 Beginner Projects

1. **Set up a mining operation**
   - Join a mining pool
   - Optimize mining settings
   - See: `./target/release/opensyria-node-cli pool --help`

2. **Build a heritage collection**
   - Create 10+ heritage tokens
   - Upload media to IPFS
   - Document Syrian culture

3. **Run a block explorer**
   ```bash
   ./target/release/explorer ~/.opensyria/node 3000
   # Open: http://localhost:3000
   ```

### 📚 Learn More

- **[Architecture Guide](docs/ARCHITECTURE.md)** - Deep dive into system design
- **[Cultural Identity System](docs/identity/CULTURAL_IDENTITY.md)** - Heritage tokens in detail
- **[IPFS Integration](docs/identity/IPFS_INTEGRATION.md)** - Multimedia storage guide
- **[P2P Networking](docs/network/P2P_NETWORKING.md)** - Network architecture
- **[Governance System](docs/governance/GOVERNANCE.md)** - On-chain voting

### 🛠️ Developer Resources

- **API Development**: See `docs/api/WALLET_API.md`
- **Integration Testing**: Run `./scripts/test-network.sh`
- **Contributing**: Read `CONTRIBUTING.md`

### 🌟 Advanced Features

1. **Multi-signature Accounts**
   ```bash
   ./target/release/opensyria-node-cli multisig --help
   ```

2. **Governance Participation**
   ```bash
   ./target/release/opensyria-node-cli governance --help
   ```

3. **REST API Integration**
   ```bash
   ./target/release/opensyria-wallet-api --port 8080
   ```

---

## Getting Help

### Documentation

- **Full Documentation**: See `docs/README.md`
- **Quick Reference**: Check `README.md`
- **CLI Help**: Add `--help` to any command

### Troubleshooting

#### Build Fails

```bash
# Update Rust
rustup update stable

# Clean and rebuild
cargo clean
cargo build --release
```

#### IPFS Issues

```bash
# Check if daemon is running
curl http://127.0.0.1:5001/api/v0/version

# Restart daemon
pkill ipfs
ipfs daemon &
```

#### Node Won't Start

```bash
# Check data directory permissions
ls -la ~/.opensyria/

# Remove and reinitialize
rm -rf ~/.opensyria/node
./target/release/opensyria-node-cli init --difficulty 16
```

### Community

- **GitHub Issues**: Report bugs or ask questions
- **Discussions**: Join conversations at GitHub Discussions
- **Contributing**: See `CONTRIBUTING.md`

---

## Frequently Asked Questions

**Q: Is this ready for production?**  
A: The blockchain is feature-complete and tested, but hasn't undergone external security audits. Use for development and testing only.

**Q: Can I mine on my laptop?**  
A: Yes! Mining difficulty is adjustable. Start with `--difficulty 16` for laptop mining.

**Q: How do I connect to other nodes?**  
A: Use the network CLI with bootstrap peers:
```bash
./target/release/opensyria-node-cli network start \
  --bootstrap /ip4/192.168.1.100/tcp/9000/p2p/12D3KooW...
```

**Q: Where is my data stored?**  
A: Default location is `~/.opensyria/`. Change with `-d` flag:
```bash
./target/release/opensyria-node-cli -d /custom/path init
```

**Q: How do I backup my wallet?**  
A: Copy `~/.opensyria/wallets/*.key` files to safe storage. **Keep them secure!**

**Q: What's the block reward?**  
A: Currently 50 SYL per block (subject to change via governance).

---

## Summary

You've learned how to:

- ✅ Install and build Open Syria blockchain
- ✅ Initialize a node and check status
- ✅ Create wallets
- ✅ Mine blocks
- ✅ Send transactions
- ✅ Create heritage tokens
- ✅ Upload media to IPFS
- ✅ Join the P2P network

**You're now ready to participate in Syria's decentralized digital future!**

**أنت الآن جاهز للمشاركة في المستقبل الرقمي اللامركزي لسوريا!**

---

## What's Next?

Choose your path:

- **👨‍💻 Developer**: Read `docs/ARCHITECTURE.md` and start contributing
- **⛏️ Miner**: Optimize your mining setup and join pools
- **🏛️ Heritage Contributor**: Document Syrian culture with identity tokens
- **🗳️ Governance Participant**: Create proposals and vote on changes
- **🌐 Node Operator**: Run a public node and help decentralize the network

**Welcome to Open Syria. Let's build the future together.**

**مرحباً بك في سوريا المفتوحة. لنبني المستقبل معاً.**
