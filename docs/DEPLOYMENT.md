# Open Syria Deployment Guide
**Getting Started with the Digital Lira Blockchain**

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Quick Start](#quick-start)
4. [Running a Full Node](#running-a-full-node)
5. [Mining Guide](#mining-guide)
6. [Wallet Setup](#wallet-setup)
7. [Troubleshooting](#troubleshooting)
8. [Production Deployment](#production-deployment)

---

## Prerequisites

### System Requirements

**Minimum:**
- Operating System: Linux, macOS, or Windows (WSL2)
- CPU: 2 cores
- RAM: 2GB
- Storage: 10GB free space
- Rust: 1.75+

**Recommended:**
- CPU: 4+ cores
- RAM: 8GB
- Storage: 50GB SSD
- Rust: Latest stable

### Installing Rust

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

---

## Installation

### From Source

```bash
# Clone repository (when available)
git clone https://github.com/OpenSyria/blockchain.git
cd blockchain

# Build all components
cargo build --release

# Binaries will be in target/release/
ls -la target/release/{wallet,miner,opensyria-node-cli}
```

### Install System-Wide

```bash
# Install binaries to ~/.cargo/bin
cargo install --path crates/wallet
cargo install --path crates/consensus --bin miner
cargo install --path crates/node-cli

# Verify installation
wallet --version
miner --version
opensyria-node-cli --version
```

---

## Quick Start

### 1. Initialize Your Node

```bash
# Initialize with default settings
opensyria-node-cli init

# Custom data directory and difficulty
opensyria-node-cli --data-dir ~/my-node init --difficulty 16
```

**Output:**
```
════════════════════════════════════════════════════════════
  Initializing Open Syria Node  
════════════════════════════════════════════════════════════

✓ Node initialized successfully

Data directory: /home/user/.opensyria/node
Genesis difficulty: 16
Chain height: 1
```

### 2. Check Node Status

```bash
opensyria-node-cli info
```

**Output:**
```
════════════════════════════════════════════════════════════
  Blockchain Information  
════════════════════════════════════════════════════════════

Chain Height: 1
Latest Block: 1f848c01a7662002...
Timestamp: 1763435917
Difficulty: 16
Transactions: 0
```

### 3. Create a Wallet

```bash
wallet create --name my-account
```

**Output:**
```
✓ Account created successfully | تم إنشاء الحساب بنجاح

Name | الاسم: my-account
Address | العنوان: af28c8c5419f772913b5194aceef6e91...

⚠ Keep your wallet files secure | احفظ ملفات المحفظة بأمان
```

### 4. Mine Your First Block

```bash
opensyria-node-cli mine --blocks 1 --difficulty 16 --verbose
```

**Output:**
```
════════════════════════════════════════════════════════════
  Open Syria Mining Node  
════════════════════════════════════════════════════════════

Starting height: 1
Difficulty: 16
Target blocks: 1

▶ Mining Block 2
  Hashes: 72186 | Rate: 1650525.31 H/s

  ✓ Block Mined #2
    Hash: 000078f8b92af1ea8b52a2b28cadc8c4...
    Nonce: 72185
    Hashes: 72186 (0.04s)
    Hash Rate: 1650525.31 H/s
```

---

## Running a Full Node

### Basic Configuration

```bash
# Start node with default settings
opensyria-node-cli init

# Custom configuration
opensyria-node-cli \
  --data-dir /var/lib/opensyria \
  init --difficulty 20
```

### Node Directory Structure

```
~/.opensyria/node/
├── blocks/           # Blockchain storage (RocksDB)
│   ├── CURRENT
│   ├── LOCK
│   ├── LOG
│   ├── MANIFEST-*
│   └── *.sst
└── state/            # Account state (RocksDB)
    ├── CURRENT
    ├── LOCK
    └── *.sst
```

### Inspecting the Blockchain

```bash
# View latest block
opensyria-node-cli block latest

# View specific block by height
opensyria-node-cli block 5

# Check account balance
opensyria-node-cli balance <hex-address>

# Export blockchain data
opensyria-node-cli export \
  --output blockchain.json \
  --start 1 \
  --end 100
```

---

## Mining Guide

### Using Node CLI

```bash
# Mine 10 blocks
opensyria-node-cli mine --blocks 10 --difficulty 16

# Continuous mining (Ctrl+C to stop)
opensyria-node-cli mine --blocks 0 --difficulty 16

# Verbose output with progress
opensyria-node-cli mine --blocks 5 --difficulty 16 --verbose
```

### Using Standalone Miner

```bash
# Mine 3 blocks at difficulty 12
miner --difficulty 12 --blocks 3 --verbose
```

**Output:**
```
═══════════════════════════════════════════════════════════
  Open Syria Proof-of-Work Miner | منقّب البرهان على العمل  
═══════════════════════════════════════════════════════════

Difficulty | الصعوبة: 12
Target Blocks | عدد الكتل: 3

▶ Mining Block | تعدين الكتلة 1
  ✓ Block Mined | تم تعدين الكتلة #1
    Hash | الهاش: 000ed84a6230d383ac8db8e0...
    Nonce | النونس: 5017
    Hashes | الهاشات: 5018
    Time | الوقت: 0.00s
    Hash Rate | معدل الهاش: 1584027.15 H/s
```

### Mining Performance Tuning

**Difficulty Selection:**
- Difficulty 12: ~5k hashes (~0.003s per block)
- Difficulty 16: ~65k hashes (~0.04s per block)
- Difficulty 20: ~1M hashes (~0.6s per block)
- Difficulty 24: ~16M hashes (~10s per block)

**Recommendations:**
- **Development:** Difficulty 12-16
- **Testnet:** Difficulty 18-20
- **Production:** Difficulty 22-24 (adjust based on network hash rate)

---

## Wallet Setup

### Creating Accounts

```bash
# Create new account
wallet create --name alice

# Create multiple accounts
wallet create --name bob
wallet create --name charlie
```

### Managing Accounts

```bash
# List all accounts
wallet list

# View account details
wallet info alice

# Delete account (with confirmation)
wallet delete alice
```

### Creating Transactions

```bash
# Get recipient address
RECIPIENT=$(wallet info bob | grep Address | awk '{print $4}')

# Create and sign transaction
wallet send \
  --from alice \
  --to $RECIPIENT \
  --amount 10.5 \
  --fee 0.001 \
  --nonce 0
```

**Output:**
```
✓ Transaction created and signed | تم إنشاء المعاملة وتوقيعها

Transaction Details | تفاصيل المعاملة
──────────────────────────────────────────────────

From | من: alice
To | إلى: b0196ae64da90525...
Amount | المبلغ: 10.5 SYL
Fee | الرسوم: 0.001 SYL
Nonce | الرقم: 0

Signed Transaction (JSON):
{
  "from": [...],
  "to": [...],
  "amount": 10500000,
  "fee": 1000,
  "nonce": 0,
  "signature": [...],
  "data": null
}
```

### Processing Transactions

```bash
# Save transaction to file
wallet send --from alice --to $RECIPIENT --amount 10 --fee 0.001 --nonce 0 > tx.json

# Process transaction on-chain
opensyria-node-cli process-tx --file tx.json
```

---

## Troubleshooting

### Common Issues

#### "Node not initialized"

```bash
# Error: Node not initialized. Run 'init' first.

# Solution: Initialize the node
opensyria-node-cli init
```

#### "Invalid nonce"

```bash
# Error: Invalid nonce: expected 5, got 0

# Solution: Check current nonce
opensyria-node-cli balance <address>  # Shows current state
# Use correct nonce in transaction
wallet send --from alice --to $RECIPIENT --amount 10 --fee 0.001 --nonce 5
```

#### "Insufficient balance"

```bash
# Error: Transfer failed

# Solution: Check account balance
wallet info alice
opensyria-node-cli balance <alice-address>
```

#### Database Lock Errors

```bash
# Error: Database error: IO error: lock hold by current process

# Solution: Ensure no other node instance is running
pkill opensyria-node-cli
# Then retry command
```

### Performance Issues

**Slow Mining:**
- Check CPU usage (`top` or `htop`)
- Reduce difficulty for testing
- Consider multi-threaded mining (future feature)

**High Disk Usage:**
- Monitor disk space: `du -sh ~/.opensyria`
- Prune old data (future feature)
- Use SSD for better performance

### Getting Help

```bash
# Command help
opensyria-node-cli --help
wallet --help
miner --help

# Subcommand help
opensyria-node-cli mine --help
wallet send --help
```

---

## Production Deployment

### Security Hardening

**1. Secure Wallet Storage:**
```bash
# Set restrictive permissions
chmod 700 ~/.opensyria/wallet
chmod 600 ~/.opensyria/wallet/*.json

# Consider encrypted filesystem
# Future: Password-protected wallets
```

**2. Firewall Configuration:**
```bash
# Allow only necessary ports (future P2P)
sudo ufw allow 8333/tcp  # Example port
sudo ufw enable
```

**3. System Service (systemd):**

Create `/etc/systemd/system/opensyria-node.service`:
```ini
[Unit]
Description=Open Syria Blockchain Node
After=network.target

[Service]
Type=simple
User=opensyria
Group=opensyria
WorkingDirectory=/var/lib/opensyria
ExecStart=/usr/local/bin/opensyria-node-cli --data-dir /var/lib/opensyria mine --blocks 0 --difficulty 20
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable opensyria-node
sudo systemctl start opensyria-node
sudo systemctl status opensyria-node
```

**4. Monitoring:**
```bash
# View logs
sudo journalctl -u opensyria-node -f

# Check node status
opensyria-node-cli --data-dir /var/lib/opensyria info
```

### Backup Strategy

**Blockchain Data:**
```bash
# Backup entire node
tar -czf opensyria-backup-$(date +%Y%m%d).tar.gz ~/.opensyria/node

# Backup wallets only
tar -czf wallets-backup-$(date +%Y%m%d).tar.gz ~/.opensyria/wallet
```

**Automated Backups (cron):**
```bash
# Edit crontab
crontab -e

# Add daily backup at 2 AM
0 2 * * * tar -czf /backup/opensyria-$(date +\%Y\%m\%d).tar.gz ~/.opensyria/node
```

### Resource Monitoring

```bash
# Disk usage
du -sh ~/.opensyria/node/*

# Node metrics (manual)
watch -n 5 'opensyria-node-cli info'

# System resources
htop
iotop  # I/O monitoring
```

---

## Environment Variables

```bash
# Custom data directory
export OPENSYRIA_DATA_DIR=~/custom-path
opensyria-node-cli init  # Uses custom path

# Logging level (future)
export RUST_LOG=info,opensyria=debug
opensyria-node-cli mine --blocks 1
```

---

## Advanced Topics

### Custom Genesis Block

Currently automated. Future support for:
- Custom genesis timestamp
- Pre-allocated balances
- Custom difficulty

### Network Configuration (Future)

```bash
# Example (not yet implemented)
opensyria-node-cli start \
  --p2p-port 8333 \
  --rpc-port 8332 \
  --bootstrap-peers peer1.opensyria.org,peer2.opensyria.org
```

### Chain Analysis

```bash
# Export full chain
opensyria-node-cli export --output chain.json --start 1 --end 0

# Analyze with jq
cat chain.json | jq 'length'  # Block count
cat chain.json | jq '.[].header.difficulty' | sort | uniq -c  # Difficulty distribution
```

---

## Next Steps

- **Join the Community:** (Links TBD)
- **Read Architecture:** See `docs/ARCHITECTURE.md`
- **Contribute:** See `CONTRIBUTING.md` (future)
- **Report Issues:** GitHub Issues (future)

---

**Last Updated:** November 2025  
**Version:** 1.0.0
