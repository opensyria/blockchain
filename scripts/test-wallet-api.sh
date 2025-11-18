#!/bin/bash

# Start API server
echo "Starting wallet API..."
./target/debug/opensyria-wallet-api -d data &
API_PID=$!

# Wait for server to start
sleep 3

echo "Testing endpoints..."
echo ""

echo "1. Health check:"
curl -s http://localhost:8080/health | jq .
echo ""

echo "2. Blockchain info:"
curl -s http://localhost:8080/api/v1/blockchain/info | jq .
echo ""

echo "3. Mempool status:"
curl -s http://localhost:8080/api/v1/mempool/status | jq .
echo ""

echo "4. Balance query (coinbase address from genesis):"
COINBASE="b1946ac92492d2347c6235b4d2611184ac13518cfea0f8a99e46fcf0a0b1946ac92492d2347c6235b4d2611184ac1"
curl -s "http://localhost:8080/api/v1/account/${COINBASE}/balance" | jq .
echo ""

# Cleanup
echo "Stopping API..."
kill $API_PID 2>/dev/null
wait $API_PID 2>/dev/null

echo "✅ All tests complete"
