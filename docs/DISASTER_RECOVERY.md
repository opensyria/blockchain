# OpenSyria Blockchain Disaster Recovery Guide
# دليل استعادة الكوارث لسلسلة كتل OpenSyria

## Table of Contents
- [Overview](#overview)
- [Backup Procedures](#backup-procedures)
- [Restore Procedures](#restore-procedures)
- [Emergency Procedures](#emergency-procedures)
- [Testing & Validation](#testing--validation)
- [Contact Information](#contact-information)

## Overview

This document provides comprehensive disaster recovery procedures for the OpenSyria blockchain infrastructure. Follow these procedures to ensure business continuity and data integrity in case of:

- Hardware failures
- Data corruption
- Security incidents
- Natural disasters
- Accidental data loss

**Recovery Time Objective (RTO):** < 4 hours  
**Recovery Point Objective (RPO):** < 1 hour

## Backup Procedures

### 1. Blockchain Data Backup

The blockchain data directory contains critical state information:

```bash
# Default locations
~/.opensyria/data/blockchain/   # Block headers and metadata
~/.opensyria/data/blocks/        # Full block data
~/.opensyria/data/state/         # Account balances and nonces
~/.opensyria/data/governance/    # Governance proposals and votes
~/.opensyria/data/identity/      # Identity tokens (NFTs)
```

#### Daily Automated Backup Script

```bash
#!/bin/bash
# /usr/local/bin/opensyria-backup.sh

BACKUP_DIR="/backup/opensyria"
DATE=$(date +%Y%m%d_%H%M%S)
DATA_DIR="$HOME/.opensyria/data"

# Create backup directory
mkdir -p "$BACKUP_DIR/$DATE"

# Stop the node for consistent backup (optional - can use RocksDB checkpoints instead)
# systemctl stop opensyria-node

# Create RocksDB checkpoints (hot backup without stopping node)
for db in blockchain blocks state governance identity; do
    echo "Backing up $db..."
    rsync -a --link-dest="$BACKUP_DIR/latest" \
        "$DATA_DIR/$db/" \
        "$BACKUP_DIR/$DATE/$db/"
done

# Update latest symlink
ln -snf "$BACKUP_DIR/$DATE" "$BACKUP_DIR/latest"

# Restart node if stopped
# systemctl start opensyria-node

# Create compressed archive
tar -czf "$BACKUP_DIR/opensyria-$DATE.tar.gz" -C "$BACKUP_DIR" "$DATE"

# Upload to offsite storage (S3, Google Cloud, etc.)
aws s3 cp "$BACKUP_DIR/opensyria-$DATE.tar.gz" \
    s3://opensyria-backups/blockchain/

# Cleanup old backups (keep last 30 days)
find "$BACKUP_DIR" -name "opensyria-*.tar.gz" -mtime +30 -delete
find "$BACKUP_DIR" -maxdepth 1 -type d -mtime +30 ! -name latest -exec rm -rf {} \;

echo "Backup completed: $DATE"
```

#### Setup Automated Backups

```bash
# Install script
sudo cp opensyria-backup.sh /usr/local/bin/
sudo chmod +x /usr/local/bin/opensyria-backup.sh

# Add to crontab (daily at 2 AM)
crontab -e
# Add this line:
0 2 * * * /usr/local/bin/opensyria-backup.sh >> /var/log/opensyria-backup.log 2>&1
```

### 2. Configuration Backup

Backup node configuration files:

```bash
# Configuration files
~/.opensyria/config.toml
~/.opensyria/network/keys/      # P2P identity keys
/etc/systemd/system/opensyria-node.service

# Backup script
tar -czf config-backup-$(date +%Y%m%d).tar.gz \
    ~/.opensyria/config.toml \
    ~/.opensyria/network/keys/ \
    /etc/systemd/system/opensyria-node.service
```

### 3. Wallet Backup

**CRITICAL:** Wallet backups contain private keys. Encrypt and secure properly.

```bash
# Encrypted wallet location
~/.opensyria/wallets/*.wallet

# Backup with encryption
tar -czf wallets-backup.tar.gz ~/.opensyria/wallets/
gpg --symmetric --cipher-algo AES256 wallets-backup.tar.gz
# Securely store wallets-backup.tar.gz.gpg offsite
shred -u wallets-backup.tar.gz  # Securely delete unencrypted archive
```

**Mnemonic Phrase Backup:**
- Write down BIP-39 mnemonic phrases on paper
- Store in multiple secure locations (safe, safety deposit box)
- Never store electronically without encryption
- Consider steel backup solutions for fire/water resistance

### 4. Verification Checksums

```bash
# Generate checksums for verification
cd /backup/opensyria/latest
find . -type f -exec sha256sum {} \; > checksums.txt
```

## Restore Procedures

### 1. Full Node Restore from Backup

```bash
# Stop existing node
sudo systemctl stop opensyria-node

# Download latest backup from S3
aws s3 cp s3://opensyria-backups/blockchain/opensyria-YYYYMMDD_HHMMSS.tar.gz .

# Extract backup
tar -xzf opensyria-YYYYMMDD_HHMMSS.tar.gz

# Verify checksums
cd YYYYMMDD_HHMMSS
sha256sum -c checksums.txt

# Restore data
rm -rf ~/.opensyria/data/
cp -r blockchain blocks state governance identity ~/.opensyria/data/

# Set correct permissions
chmod 700 ~/.opensyria/data
chmod 600 ~/.opensyria/data/**/

# Restart node
sudo systemctl start opensyria-node

# Monitor sync
journalctl -u opensyria-node -f
```

### 2. Wallet Restore

```bash
# Decrypt wallet backup
gpg --decrypt wallets-backup.tar.gz.gpg > wallets-backup.tar.gz

# Extract wallets
tar -xzf wallets-backup.tar.gz -C ~/

# Verify wallet integrity
opensyria-wallet list

# Securely delete decrypted archive
shred -u wallets-backup.tar.gz
```

### 3. Restore from Mnemonic Phrase

```bash
# Launch wallet CLI
opensyria-wallet create-hd my-wallet

# Enter your 12 or 24-word mnemonic phrase when prompted
# Wallet will deterministically regenerate all accounts
```

### 4. Configuration Restore

```bash
# Extract configuration backup
tar -xzf config-backup-YYYYMMDD.tar.gz -C ~/

# Restore systemd service
sudo cp opensyria-node.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable opensyria-node
```

## Emergency Procedures

### Scenario 1: Data Corruption Detected

```bash
# 1. Immediately stop the node
sudo systemctl stop opensyria-node

# 2. Backup corrupted data for forensic analysis
mv ~/.opensyria/data ~/.opensyria/data.corrupted

# 3. Restore from latest known-good backup
# Follow "Full Node Restore" procedure above

# 4. Verify integrity
opensyria-node verify-chain

# 5. Resume operations
sudo systemctl start opensyria-node
```

### Scenario 2: Security Breach

```bash
# 1. Isolate the node
sudo ufw deny from any to any
sudo systemctl stop opensyria-node

# 2. Preserve evidence
tar -czf incident-evidence-$(date +%Y%m%d).tar.gz \
    ~/.opensyria/data/ \
    /var/log/opensyria* \
    /var/log/syslog

# 3. Rotate all keys
opensyria-wallet rotate-keys

# 4. Restore from backup before breach
# Use backup from before suspected compromise time

# 5. Notify security team
# See Contact Information section

# 6. Resume operations after investigation
sudo ufw reset
sudo systemctl start opensyria-node
```

### Scenario 3: Hardware Failure

```bash
# 1. Provision new hardware with identical specs

# 2. Install OS and dependencies
sudo apt update && sudo apt install -y build-essential

# 3. Install OpenSyria from source or package
cargo install --path /path/to/opensyria

# 4. Restore configuration
# Follow "Configuration Restore" procedure

# 5. Restore blockchain data
# Follow "Full Node Restore" procedure

# 6. Start node
sudo systemctl enable opensyria-node
sudo systemctl start opensyria-node

# 7. Monitor for 24h to ensure stability
watch -n 5 'systemctl status opensyria-node'
```

### Scenario 4: Network Partition / Fork

```bash
# 1. Identify fork point
opensyria-node status | grep chain_height

# 2. Check peer consensus
curl http://explorer.opensyria.org/api/chain-tip

# 3. If on minority fork, resync from majority
sudo systemctl stop opensyria-node

# Reorg to canonical chain
opensyria-node reorg --target-height <fork_height>

# Or full resync
rm -rf ~/.opensyria/data/blockchain ~/.opensyria/data/blocks
opensyria-node sync --bootstrap

sudo systemctl start opensyria-node
```

## Testing & Validation

### Monthly Restore Test

```bash
#!/bin/bash
# Test restore procedure monthly

# 1. Provision test environment
TEST_DIR="/tmp/opensyria-restore-test"
mkdir -p "$TEST_DIR"

# 2. Download latest backup
aws s3 cp s3://opensyria-backups/blockchain/latest.tar.gz "$TEST_DIR/"

# 3. Extract and restore
cd "$TEST_DIR"
tar -xzf latest.tar.gz

# 4. Verify checksums
sha256sum -c checksums.txt

# 5. Start test node
OPENSYRIA_DATA_DIR="$TEST_DIR/data" opensyria-node --testnet

# 6. Verify chain tip matches production
TEST_HEIGHT=$(curl -s localhost:8332/api/chain-tip | jq .height)
PROD_HEIGHT=$(curl -s https://explorer.opensyria.org/api/chain-tip | jq .height)

if [ "$TEST_HEIGHT" -eq "$PROD_HEIGHT" ]; then
    echo "✅ Restore test PASSED"
else
    echo "❌ Restore test FAILED: Heights don't match ($TEST_HEIGHT vs $PROD_HEIGHT)"
    exit 1
fi

# 7. Cleanup
rm -rf "$TEST_DIR"
```

### Restore Validation Checklist

- [ ] Blockchain data restored to correct height
- [ ] All databases accessible (blocks, state, governance, identity)
- [ ] Checksums match backup
- [ ] Node syncs new blocks
- [ ] Peer connections established
- [ ] RPC endpoints responsive
- [ ] Wallet operations functional
- [ ] Metrics endpoint active
- [ ] No database errors in logs

## Backup Retention Policy

| Backup Type | Frequency | Retention | Storage |
|-------------|-----------|-----------|---------|
| Full blockchain data | Daily | 30 days | S3 Standard |
| Configuration | Weekly | 90 days | S3 Standard-IA |
| Wallets | On change | Indefinite | S3 Glacier Deep Archive |
| Weekly snapshots | Weekly | 1 year | S3 Standard-IA |
| Monthly archives | Monthly | 7 years | S3 Glacier Deep Archive |

## Contact Information

### Emergency Contacts

**Blockchain Operations Team:**
- 24/7 Hotline: +963-XXX-XXXX
- Email: ops@opensyria.org
- Telegram: @opensyria_ops

**Security Incident Response:**
- Email: security@opensyria.org
- PGP Key: [fingerprint]
- Signal: +963-XXX-XXXX

**Infrastructure Team:**
- Email: infra@opensyria.org
- On-call: PagerDuty rotation

### Escalation Matrix

| Severity | Response Time | Escalation |
|----------|---------------|------------|
| P0 - Critical (chain halt) | < 15 minutes | CTO immediately |
| P1 - High (data loss risk) | < 1 hour | Lead Engineer |
| P2 - Medium (degraded performance) | < 4 hours | On-call engineer |
| P3 - Low (monitoring alerts) | < 24 hours | Next business day |

## Compliance & Audit

- All backup/restore activities are logged to `/var/log/opensyria-dr.log`
- Monthly DR tests documented in `docs/audits/DR_TEST_YYYY_MM.md`
- Annual DR plan review required
- Compliance with ISO 27001 backup requirements

## Related Documentation

- [Deployment Guide](DEPLOYMENT.md)
- [Monitoring Setup](monitoring/PROMETHEUS.md)
- [Security Procedures](SECURITY.md)
- [Network Architecture](ARCHITECTURE.md)

---

**Last Updated:** November 19, 2025  
**Document Owner:** Infrastructure Team  
**Review Cycle:** Quarterly
