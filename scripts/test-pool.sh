#!/bin/bash

# Test script for mining pool functionality

set -e

CLI="./target/debug/opensyria-node-cli"
TEST_DIR="/tmp/opensyria-pool-test"

echo "════════════════════════════════════════════════════════════"
echo "  Mining Pool Test"
echo "════════════════════════════════════════════════════════════"
echo

# Clean up previous test data
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"

# Initialize node
echo "1. Initializing blockchain..."
$CLI -d "$TEST_DIR" init --difficulty 12 > /dev/null
echo "✓ Blockchain initialized"
echo

# Generate operator key
OPERATOR_KEY=$(openssl rand -hex 32)
echo "2. Generated operator key: ${OPERATOR_KEY:0:16}..."
echo

# Initialize mining pool
echo "3. Initializing mining pool..."
$CLI -d "$TEST_DIR" pool init \
  --operator "$OPERATOR_KEY" \
  --fee 2 \
  --share-difficulty 12 \
  --method proportional
echo

# Show pool stats
echo "4. Pool statistics:"
$CLI -d "$TEST_DIR" pool stats
echo

# Register miners
echo "5. Registering miners..."
MINER1=$(openssl rand -hex 32)
MINER2=$(openssl rand -hex 32)
MINER3=$(openssl rand -hex 32)

$CLI -d "$TEST_DIR" pool register "$MINER1" > /dev/null
echo "✓ Miner 1 registered: ${MINER1:0:16}..."

$CLI -d "$TEST_DIR" pool register "$MINER2" > /dev/null
echo "✓ Miner 2 registered: ${MINER2:0:16}..."

$CLI -d "$TEST_DIR" pool register "$MINER3" > /dev/null
echo "✓ Miner 3 registered: ${MINER3:0:16}..."
echo

# List all miners
echo "6. Listing all registered miners:"
$CLI -d "$TEST_DIR" pool miners
echo

# Show specific miner stats
echo "7. Miner 1 statistics:"
$CLI -d "$TEST_DIR" pool miner "$MINER1"
echo

echo "════════════════════════════════════════════════════════════"
echo "  ✓ Mining Pool Test Passed"
echo "════════════════════════════════════════════════════════════"
echo
echo "Note: Share submission and reward distribution require"
echo "      a full pool server implementation with network protocol."
