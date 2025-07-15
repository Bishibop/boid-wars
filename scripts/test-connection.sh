#!/bin/bash
set -e

echo "🔌 Testing WebTransport connection..."

# Test if server is running
SERVER_URL="https://localhost:3000"

# Simple curl test (will fail due to WebTransport, but shows if server is up)
if curl -k -s --head --connect-timeout 2 "$SERVER_URL" > /dev/null 2>&1; then
    echo "✅ Server is responding on $SERVER_URL"
else
    echo "❌ Server is not responding on $SERVER_URL"
    echo "   Make sure to run: ./scripts/run-server.sh"
    exit 1
fi

echo ""
echo "To test WebTransport connection:"
echo "1. Run the server: ./scripts/run-server.sh"
echo "2. Run the client: cd client && npm run dev"
echo "3. Open https://localhost:5173 in Chrome/Edge/Firefox"