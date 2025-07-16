#!/bin/bash
set -e

echo "🏗️  Building WASM module (debug mode)..."

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

# Only clean output directory, preserve Rust target cache for incremental builds
rm -rf pkg

# Build the WASM module in debug mode (faster compilation)
echo "🔨 Building with wasm-pack (debug mode)..."

# Check if we can use incremental compilation cache
if [ -d "target/wasm32-unknown-unknown" ]; then
    echo "📦 Using incremental compilation cache"
else
    echo "🆕 First build - creating incremental cache"
fi

# Use debug profile for faster builds during development
wasm-pack build --target web --out-dir ../client/src/wasm --dev

echo "✅ WASM module built successfully (debug)"
echo "📁 Output: client/src/wasm/"