#!/bin/bash

set -e

echo "🚀 Building optimized Bevy WASM client for release..."

# Clean previous build
echo "🧹 Cleaning previous build..."
rm -rf pkg

# Build with wasm-pack in release mode
echo "📦 Building with wasm-pack (release mode)..."
wasm-pack build \
    --target web \
    --out-dir pkg \
    --release

echo "🔧 Running wasm-opt for additional optimization..."
if command -v wasm-opt &> /dev/null; then
    wasm-opt -Os --enable-mutable-globals pkg/*_bg.wasm -o pkg/*_bg.wasm
    echo "✅ wasm-opt optimization complete"
else
    echo "⚠️  wasm-opt not found. Install with: apt install binaryen or brew install binaryen"
fi

echo "📊 Final bundle size analysis:"
echo "=================================="
ls -lh pkg/*.wasm | awk '{print "WASM file: " $9 " - " $5}'
ls -lh pkg/*.js | awk '{print "JS file:   " $9 " - " $5}'

# Calculate total size
TOTAL_SIZE=$(du -ch pkg/*.wasm pkg/*.js 2>/dev/null | tail -1 | cut -f1)
echo "Total size: $TOTAL_SIZE"
echo "=================================="

# Compare with target
echo ""
echo "📏 Size comparison:"
echo "Target: <5MB"
echo "Actual: $TOTAL_SIZE"

if [ -f pkg/*_bg.wasm ]; then
    WASM_SIZE_BYTES=$(stat -f%z pkg/*_bg.wasm 2>/dev/null || stat -c%s pkg/*_bg.wasm 2>/dev/null || echo "unknown")
    if [ "$WASM_SIZE_BYTES" != "unknown" ]; then
        WASM_SIZE_MB=$((WASM_SIZE_BYTES / 1024 / 1024))
        if [ $WASM_SIZE_MB -lt 5 ]; then
            echo "✅ Bundle size target met!"
        else
            echo "❌ Bundle size exceeds 5MB target"
        fi
    fi
fi

echo ""
echo "🎯 Build complete! Test with: python3 -m http.server 8080"