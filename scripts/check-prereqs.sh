#!/bin/bash

echo "🔍 Checking prerequisites for Boid Wars..."
echo ""

# Track if all checks pass
ALL_GOOD=true

# Check Rust
echo -n "Rust... "
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    echo "✅ $RUST_VERSION"
else
    echo "❌ Not found!"
    echo "   Install from: https://rustup.rs/"
    ALL_GOOD=false
fi

# Check wasm target
echo -n "WASM target... "
if rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown; then
    echo "✅ wasm32-unknown-unknown"
else
    echo "❌ Not installed!"
    echo "   Run: rustup target add wasm32-unknown-unknown"
    ALL_GOOD=false
fi

# Check Node.js
echo -n "Node.js... "
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    echo "✅ $NODE_VERSION"
else
    echo "❌ Not found!"
    echo "   Install from: https://nodejs.org/"
    ALL_GOOD=false
fi

# Check wasm-pack
echo -n "wasm-pack... "
if command -v wasm-pack &> /dev/null; then
    WASM_PACK_VERSION=$(wasm-pack --version | cut -d' ' -f2)
    echo "✅ $WASM_PACK_VERSION"
else
    echo "❌ Not found!"
    echo "   Install: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    ALL_GOOD=false
fi

# Check mkcert
echo -n "mkcert... "
if command -v mkcert &> /dev/null; then
    echo "✅ Installed"
else
    echo "❌ Not found!"
    echo "   Install: brew install mkcert (macOS)"
    echo "   See: https://github.com/FiloSottile/mkcert"
    ALL_GOOD=false
fi

# Check certificates
echo -n "SSL certificates... "
CERT_DIR="$HOME/.boid-wars/certs"
if [ -f "$CERT_DIR/localhost.pem" ] && [ -f "$CERT_DIR/localhost-key.pem" ]; then
    echo "✅ Found in $CERT_DIR"
elif [ -f "deploy/localhost+2.pem" ] && [ -f "deploy/localhost+2-key.pem" ]; then
    echo "⚠️  Found in deploy/ (should run ./scripts/setup-certs.sh)"
else
    echo "❌ Not found!"
    echo "   Run: ./scripts/setup-certs.sh"
    ALL_GOOD=false
fi

# Check .env file
echo -n ".env file... "
if [ -f ".env" ]; then
    echo "✅ Found"
else
    echo "❌ Not found!"
    echo "   Run: cp .env.example .env"
    echo "   Then update certificate paths if needed"
    ALL_GOOD=false
fi

echo ""
if [ "$ALL_GOOD" = true ]; then
    echo "✅ All prerequisites satisfied!"
else
    echo "❌ Some prerequisites are missing. Please install them before continuing."
    exit 1
fi