#!/usr/bin/env bash
# Frontend development server with hot reload

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FRONTEND_DIR="$SCRIPT_DIR/../crates/explorer-backend/frontend"

echo "🔥 Starting frontend dev server with HMR..."
echo "📝 Edit files in: $FRONTEND_DIR/src/"
echo "🌐 Frontend: http://localhost:3000"
echo "🔌 API Proxy: http://localhost:8080"
echo ""
echo "⚠️  Make sure the backend is running on port 8080!"
echo ""

cd "$FRONTEND_DIR"

if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
    echo ""
fi

npm run dev
