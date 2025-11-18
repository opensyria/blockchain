#!/bin/bash

echo "Testing Network Daemon Mode..."
echo ""

# Start daemon in background
./target/debug/opensyria-node-cli -d data network daemon --sync-interval 10 &> /tmp/daemon-test.log &
DAEMON_PID=$!

echo "Daemon started (PID: $DAEMON_PID)"
sleep 3

echo ""
echo "=== Daemon Output ==="
head -30 /tmp/daemon-test.log

echo ""
echo "=== Stopping Daemon ==="
kill -INT $DAEMON_PID 2>/dev/null
wait $DAEMON_PID 2>/dev/null

sleep 1
echo ""
echo "=== Final Output ==="
tail -10 /tmp/daemon-test.log

echo ""
echo "✅ Daemon test complete"
