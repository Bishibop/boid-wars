#!/bin/bash

set -e

echo "⚡ Quick WASM build (no clean, debug mode)..."

# Build with wasm-pack (debug mode, no clean for speed)
echo "📦 Building with wasm-pack (quick debug mode)..."
wasm-pack build \
    --target web \
    --out-dir pkg \
    --dev \
    --no-typescript

echo "📊 Bundle size:"
ls -lh pkg/*.wasm | awk '{print "WASM: " $9 " - " $5}'

echo "⚡ Quick build complete!"