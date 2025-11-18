#!/bin/bash
# Test script for P2P network with two nodes
# Usage: ./scripts/test-network.sh

set -e

echo "════════════════════════════════════════════════════════════"
echo "  Open Syria P2P Network Test - Two Node Setup"
echo "════════════════════════════════════════════════════════════"
echo ""

# Build the node CLI
echo "Building opensyria-node-cli..."
cargo build -p opensyria-node-cli --release --quiet
echo "✓ Build complete"
echo ""

# Clean up old data
rm -rf /tmp/opensyria-node1 /tmp/opensyria-node2 2>/dev/null || true

# Initialize node 1
echo "Initializing Node 1..."
./target/release/opensyria-node-cli -d /tmp/opensyria-node1 init --difficulty 16
echo ""

# Initialize node 2
echo "Initializing Node 2..."
./target/release/opensyria-node-cli -d /tmp/opensyria-node2 init --difficulty 16
echo ""

echo "════════════════════════════════════════════════════════════"
echo "  Starting Network Nodes"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Node 1 will listen on: /ip4/127.0.0.1/tcp/9000"
echo "Node 2 will listen on: /ip4/127.0.0.1/tcp/9001"
echo ""
echo "To start the nodes manually:"
echo ""
echo "Terminal 1:"
echo "  cargo run -p opensyria-node-cli -- -d /tmp/opensyria-node1 network start -l /ip4/127.0.0.1/tcp/9000"
echo ""
echo "Terminal 2 (after Node 1 shows its Peer ID):"
echo "  cargo run -p opensyria-node-cli -- -d /tmp/opensyria-node2 network start -l /ip4/127.0.0.1/tcp/9001 -b /ip4/127.0.0.1/tcp/9000/p2p/<PEER_ID_FROM_NODE_1>"
echo ""
echo "════════════════════════════════════════════════════════════"
echo ""
echo "Note: Since both nodes need to run interactively, you'll need"
echo "to start them in separate terminal windows."
echo ""
echo "Once connected, you should see peer connection messages."
echo "Press Ctrl+C in each terminal to stop the nodes."
echo ""
