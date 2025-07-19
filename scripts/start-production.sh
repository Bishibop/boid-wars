#!/bin/sh
set -e

echo "🚀 Starting Boid Wars production servers..."

# Start the game server on internal port 8081
echo "🎮 Starting game server on internal port 8081..."
BOID_WARS_SERVER_BIND_ADDR="127.0.0.1:8081" /app/server &
GAME_SERVER_PID=$!

# Give the game server time to start
sleep 3

# Copy the proxy script
cp /app/scripts/simple-http-ws-proxy.py /tmp/proxy.py

# Start the HTTP/WebSocket proxy server
echo "🌐 Starting HTTP/WebSocket proxy on port 8080..."
exec python3 /tmp/proxy.py