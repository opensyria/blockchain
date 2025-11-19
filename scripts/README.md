# Test Scripts

This directory contains integration test scripts for the Open Syria blockchain.

---

## Available Scripts

### Network Testing

**`test-network.sh`** - Multi-node P2P network testing
- Tests peer discovery via mDNS
- Verifies block synchronization
- Tests transaction propagation
- Checks network resilience

**Usage:**
```bash
./scripts/test-network.sh
```

**Requirements:**
- Compiled `opensyria-node-cli` binary
- Available ports: 9000-9002

---

### Multi-Signature Testing

**`test-multisig.sh`** - Multi-signature account testing
- Creates 2-of-3 multisig account
- Tests transaction signing workflow
- Verifies threshold requirements
- Tests submission and execution

**Usage:**
```bash
./scripts/test-multisig.sh
```

**Requirements:**
- Compiled `opensyria-node-cli` binary
- Initialized blockchain

---

### Mining Pool Testing

**`test-pool.sh`** - Mining pool operations
- Initializes mining pool
- Registers multiple miners
- Submits shares
- Tests payout distribution
- Verifies all reward methods (Proportional, PPS, PPLNS)

**Usage:**
```bash
./scripts/test-pool.sh
```

**Requirements:**
- Compiled `opensyria-node-cli` binary
- Initialized blockchain

---

### IPFS Integration Testing

**`test-ipfs.sh`** - Heritage content storage
- Checks IPFS daemon status
- Creates heritage token
- Uploads sample content (text, image, JSON)
- Links content to token
- Retrieves and verifies content integrity

**Usage:**
```bash
# Start IPFS daemon first
ipfs daemon &

# Run test
./scripts/test-ipfs.sh
```

**Requirements:**
- IPFS installed and daemon running
- Compiled `identity` binary
- Sample test files created automatically

---

### Wallet API Testing

**`test-wallet-api.sh`** - REST API endpoint testing
- Starts wallet API server
- Tests health check endpoint
- Tests blockchain info endpoint
- Tests balance queries
- Tests transaction submission
- Verifies response formats

**Usage:**
```bash
./scripts/test-wallet-api.sh
```

**Requirements:**
- Compiled `opensyria-wallet-api` binary
- `curl` or `httpie` installed
- Available port: 8080

---

### Network Daemon Testing

**`test-daemon.sh`** - Background node testing
- Starts daemon in background
- Verifies auto-mining
- Tests sync interval
- Checks graceful shutdown

**Usage:**
```bash
./scripts/test-daemon.sh
```

**Requirements:**
- Compiled `opensyria-node-cli` binary
- Clean data directory

---

### IPFS Summary

**`ipfs-summary.sh`** - Display IPFS integration status
- Shows token count
- Lists tokens with IPFS content
- Displays IPFS CIDs
- Shows content metadata

**Usage:**
```bash
./scripts/ipfs-summary.sh
```

**Requirements:**
- Compiled `identity` binary
- Heritage tokens created

---

## Running All Tests

Run all integration tests sequentially:

```bash
# Network tests
./scripts/test-network.sh

# Multi-signature tests
./scripts/test-multisig.sh

# Mining pool tests
./scripts/test-pool.sh

# IPFS tests (requires IPFS daemon)
ipfs daemon &
./scripts/test-ipfs.sh

# Wallet API tests
./scripts/test-wallet-api.sh

# Daemon tests
./scripts/test-daemon.sh
```

---

## Test Output

All scripts provide colored output:
- 🟢 **Green**: Success
- 🟡 **Yellow**: Warnings or info
- 🔴 **Red**: Errors or failures

**Example:**
```
================================
  Multi-Signature Test
================================

Step 1: Create wallets
✅ Created signer1
✅ Created signer2
✅ Created signer3

Step 2: Create multisig account
✅ Multisig account created (2-of-3)
   Address: 9a7b8c6d5e4f3a2b1c0d9e8f...

Step 3: Fund multisig account
✅ Account funded with 1000 SYL

...
```

---

## Troubleshooting

### Script Won't Execute

```bash
# Make script executable
chmod +x scripts/test-*.sh

# Run with bash explicitly
bash scripts/test-network.sh
```

### Binary Not Found

```bash
# Build all binaries first
cargo build --release

# Or build specific binary
cargo build --release -p opensyria-node-cli
```

### Port Already in Use

```bash
# Find and kill process using port
lsof -ti:9000 | xargs kill -9

# Or use different port in script
```

### IPFS Daemon Not Running

```bash
# Check IPFS status
curl http://127.0.0.1:5001/api/v0/version

# Start daemon
ipfs daemon &

# Wait a few seconds before running test
sleep 3
./scripts/test-ipfs.sh
```

### Permission Denied

```bash
# Fix permissions
chmod -R u+x scripts/

# Check ownership
ls -la scripts/
```

---

## Writing New Tests

### Template

```bash
#!/bin/bash
# Test description

echo "================================"
echo "  Test Name"
echo "================================"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check prerequisites
if [ ! -f "target/release/binary-name" ]; then
    echo -e "${RED}✗ Binary not found${NC}"
    exit 1
fi

# Test steps
echo "Step 1: Description"
if command_that_might_fail; then
    echo -e "${GREEN}✅ Success${NC}"
else
    echo -e "${RED}✗ Failed${NC}"
    exit 1
fi

# Cleanup
echo ""
echo -e "${GREEN}✅ All tests passed!${NC}"
```

### Best Practices

1. **Clear output**: Use colors and emoji for visibility
2. **Check prerequisites**: Verify binaries exist before running
3. **Cleanup**: Remove test data after completion
4. **Error handling**: Exit with non-zero code on failure
5. **Documentation**: Add comments explaining each step
6. **Idempotent**: Script should work when run multiple times

---

## Continuous Integration

These scripts can be used in CI/CD pipelines:

**GitHub Actions Example:**
```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Build
        run: cargo build --release
        
      - name: Run Network Tests
        run: ./scripts/test-network.sh
        
      - name: Run Multisig Tests
        run: ./scripts/test-multisig.sh
        
      - name: Run Pool Tests
        run: ./scripts/test-pool.sh
```

---

## System Service & Deployment | خدمة النظام والنشر

### Systemd Service Installation (Linux)

Install OpenSyria node as a system service:

```bash
# Interactive setup
./scripts/setup-systemd.sh

# Manual installation
sudo cp opensyria-node.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable opensyria-node
sudo systemctl start opensyria-node
```

**Service Management:**
```bash
# Start/Stop/Restart
sudo systemctl start opensyria-node
sudo systemctl stop opensyria-node
sudo systemctl restart opensyria-node

# Check status
sudo systemctl status opensyria-node

# View logs
sudo journalctl -u opensyria-node -f
```

### Log Rotation

Install automatic log rotation:

```bash
# Install logrotate config
sudo cp scripts/opensyria-node.logrotate /etc/logrotate.d/opensyria-node

# Test
sudo logrotate -d /etc/logrotate.d/opensyria-node
```

**Settings:**
- Daily rotation
- Keep 7 days
- Max 100MB per file
- Compress old logs

### Configuration File

Generate config file:

```bash
# Create example config
mkdir -p ~/.opensyria
cat > ~/.opensyria/config.toml << 'EOF'
data_dir = "~/.opensyria/node"

[network]
port = 9000
bootstrap_nodes = []
max_peers = 50

[mining]
difficulty = 16
threads = 4

[daemon]
auto_mine = false
log_file = "~/.opensyria/node/opensyria.log"
log_max_size_mb = 100
log_backups = 7
EOF
```

**Usage:**
```bash
# Run with config
opensyria-node daemon --config ~/.opensyria/config.toml
```

---

## Performance Benchmarks

Some scripts include performance measurements:

```bash
# test-network.sh outputs:
⚡ Block sync time: 1.2 seconds
⚡ Transaction propagation: 0.3 seconds

# test-pool.sh outputs:
⚡ Share validation: 15ms per share
⚡ Payout calculation: 45ms for 100 miners
```

---

## Contributing

To add a new test script:

1. Create script in `scripts/` directory
2. Follow naming convention: `test-<feature>.sh`
3. Make executable: `chmod +x scripts/test-<feature>.sh`
4. Add documentation to this README
5. Test on clean environment
6. Submit pull request

**See also:** [CONTRIBUTING.md](../CONTRIBUTING.md)

---

## Related Documentation

- [Integration Tests Guide](../docs/tests/INTEGRATION_TESTS.md)
- [Network CLI Documentation](../docs/network/NETWORK_CLI.md)
- [IPFS Integration Guide](../docs/identity/IPFS_INTEGRATION.md)
- [Governance Documentation](../docs/governance/GOVERNANCE.md)
- [Wallet API Reference](../docs/api/WALLET_API.md)

---

**Last Updated:** November 18, 2025  
**Scripts:** 7 integration test scripts  
**Coverage:** All major features tested
