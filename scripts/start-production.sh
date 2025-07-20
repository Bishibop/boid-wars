#!/bin/sh
set -e

echo "🚀 Starting Boid Wars production servers..."

# Create log directory
mkdir -p /tmp/logs

# Start the game server on internal port 8081 with logging
echo "🎮 Starting game server on internal port 8081..."
echo "📝 Server logs will be written to /tmp/logs/server.log"

BOID_WARS_SERVER_BIND_ADDR="127.0.0.1:8081" /app/server > /tmp/logs/server.log 2>&1 &
GAME_SERVER_PID=$!

echo "🔍 Game server PID: $GAME_SERVER_PID"

# Give the game server time to start and check if it's still running
sleep 3

if ! kill -0 $GAME_SERVER_PID 2>/dev/null; then
    echo "❌ Game server process died! Checking logs..."
    echo "📜 Last 20 lines of server output:"
    tail -n 20 /tmp/logs/server.log || echo "No log file found"
    exit 1
fi

echo "✅ Game server appears to be running"
echo "📜 Recent server logs:"
tail -n 10 /tmp/logs/server.log || echo "No log content yet"

# Copy the proxy script
cp /app/scripts/simple-http-ws-proxy.py /tmp/proxy.py

# Start the HTTP/WebSocket proxy server
echo "🌐 Starting HTTP/WebSocket proxy on port 8080..."
exec python3 /tmp/proxy.py