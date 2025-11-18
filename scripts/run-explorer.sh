#!/usr/bin/env bash
# Development server launcher for Open Syria Explorer

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FRONTEND_DIR="$SCRIPT_DIR/../crates/explorer-backend/frontend"
BACKEND_DIR="$SCRIPT_DIR/../crates/explorer-backend"

echo "🚀 Open Syria Block Explorer - Development Mode"
echo ""

# Check if frontend dependencies are installed
if [ ! -d "$FRONTEND_DIR/node_modules" ]; then
    echo "📦 Installing frontend dependencies..."
    cd "$FRONTEND_DIR"
    npm install
    echo ""
fi

# Build frontend
echo "🔨 Building frontend..."
cd "$FRONTEND_DIR"
npm run build
echo ""

# Start backend server
echo "🌐 Starting backend server..."
cd "$SCRIPT_DIR/.."
cargo run --bin explorer -- data 8080

echo ""
echo "✅ Explorer running at http://localhost:8080"
