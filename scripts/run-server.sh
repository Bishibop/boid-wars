#!/bin/bash
set -e

echo "🚀 Starting Boid Wars Server..."

# Change to project root
cd "$(dirname "$0")/.."

# Load environment variables if .env exists
if [ -f .env ]; then
    echo "Loading environment from .env"
    export $(grep -v '^#' .env | xargs)
else
    echo "Warning: .env file not found. Using defaults."
    echo "Create one with: cp .env.example .env"
    # Set defaults
    export RUST_LOG=debug,boid_wars_server=trace,lightyear=debug
    export RUST_BACKTRACE=1
    export GAME_SERVER_PORT=3000
    export GAME_SERVER_HOST=0.0.0.0
fi

# Change to server directory
cd server

# Run the server
echo "Server starting on https://localhost:$GAME_SERVER_PORT"
cargo run