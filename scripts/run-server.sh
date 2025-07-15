#!/bin/bash
set -e

echo "🚀 Starting Boid Wars Server..."

# Set environment variables
export RUST_LOG=debug,boid_wars_server=trace,lightyear=debug
export RUST_BACKTRACE=1
export GAME_SERVER_PORT=3000
export GAME_SERVER_HOST=0.0.0.0

# Change to server directory
cd "$(dirname "$0")/../server"

# Run the server
echo "Server starting on https://localhost:$GAME_SERVER_PORT"
cargo run