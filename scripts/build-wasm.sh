#!/bin/bash
set -e

echo "🏗️  Building WASM module..."

# Change to lightyear-wasm directory
cd "$(dirname "$0")/../lightyear-wasm"

# Build the WASM module
wasm-pack build --target web --out-dir ../client/src/wasm

echo "✅ WASM module built successfully"
echo "📁 Output: client/src/wasm/"