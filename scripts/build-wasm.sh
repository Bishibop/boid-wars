#!/bin/bash
set -e

echo "🏗️  Building WASM module..."

# Change to project root
cd "$(dirname "$0")/.."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "❌ wasm-pack not found! Please install it:"
    echo "   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    exit 1
fi

# Check if wasm32-unknown-unknown target is installed
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "❌ wasm32-unknown-unknown target not found!"
    echo "📦 Installing it now..."
    rustup target add wasm32-unknown-unknown
fi

# Change to lightyear-wasm directory
cd lightyear-wasm

# Clean previous build
rm -rf pkg

# Build the WASM module
echo "🔨 Building with wasm-pack..."
# Use specific features that work with WASM
wasm-pack build --target web --out-dir ../client/src/wasm

echo "✅ WASM module built successfully"
echo "📁 Output: client/src/wasm/"