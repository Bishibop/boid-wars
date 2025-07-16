#!/bin/bash

set -e

echo "🔨 Building Bevy WASM client..."

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf pkg

# Build with wasm-pack (debug mode for speed)
echo "📦 Building with wasm-pack (debug mode for fast iteration)..."
wasm-pack build \
    --target web \
    --out-dir pkg \
    --dev \
    --no-typescript

echo "📊 Bundle size analysis:"
ls -lh pkg/*.wasm | awk '{print "WASM file: " $9 " - " $5}'

echo "🎯 Starting development server..."
echo "Open http://localhost:8080 in your browser"

# Start a simple HTTP server
if command -v python3 &> /dev/null; then
    python3 -m http.server 8080
elif command -v python &> /dev/null; then
    python -m SimpleHTTPServer 8080
else
    echo "⚠️  No Python found. Install a local server to test the WASM client."
    echo "Suggested: npm install -g http-server && http-server -p 8080"
fi