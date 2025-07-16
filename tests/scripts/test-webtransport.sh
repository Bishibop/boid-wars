#!/bin/bash
set -e

echo "🧪 Testing WebTransport setup..."
echo ""

# 1. Check certificates exist
echo "1️⃣ Checking certificates..."
CERT_PATH="$HOME/.boid-wars/certs/localhost.pem"
KEY_PATH="$HOME/.boid-wars/certs/localhost-key.pem"

if [ ! -f "$CERT_PATH" ] || [ ! -f "$KEY_PATH" ]; then
    echo "❌ Certificates not found!"
    echo "   Run: ./scripts/setup-certs.sh"
    exit 1
fi
echo "✅ Certificates found"

# 2. Check certificate validity
echo ""
echo "2️⃣ Checking certificate validity..."
VALIDITY=$(openssl x509 -in "$CERT_PATH" -noout -enddate | cut -d= -f2)
echo "   Certificate valid until: $VALIDITY"

# Check if certificate was created with mkcert
ISSUER=$(openssl x509 -in "$CERT_PATH" -noout -issuer)
if [[ $ISSUER == *"mkcert"* ]]; then
    echo "✅ Certificate created with mkcert (good!)"
else
    echo "⚠️  Certificate not from mkcert"
fi

# 3. Get certificate digest
echo ""
echo "3️⃣ Certificate digest for client config:"
DIGEST=$(openssl x509 -in "$CERT_PATH" -noout -fingerprint -sha256 | cut -d= -f2 | tr -d ':' | tr '[:upper:]' '[:lower:]')
echo "   $DIGEST"

# 4. Check current digest in client code
echo ""
echo "4️⃣ Checking client configuration..."
CLIENT_FILE="/Users/nicholasmullen/Code/gauntlet/boid_wars/bevy-client/src/lib.rs"
CURRENT_DIGEST=$(grep -o 'certificate_digest = "[^"]*"' "$CLIENT_FILE" | cut -d'"' -f2)
echo "   Current digest in client: $CURRENT_DIGEST"

if [ "$DIGEST" = "$CURRENT_DIGEST" ]; then
    echo "✅ Client has correct certificate digest"
else
    echo "❌ Client has wrong certificate digest!"
    echo "   Update line 65 in $CLIENT_FILE with:"
    echo "   let certificate_digest = \"$DIGEST\".to_string();"
fi

# 5. Test server certificate loading
echo ""
echo "5️⃣ Testing server certificate loading..."
# Quick test to see if certificates can be loaded
if command -v openssl &> /dev/null; then
    openssl x509 -in "$CERT_PATH" -text -noout > /dev/null 2>&1
    if [ $? -eq 0 ]; then
        echo "✅ Certificate is valid PEM format"
    else
        echo "❌ Certificate format error"
    fi
fi

echo ""
echo "📋 Summary:"
echo "   - Certificates: $CERT_PATH"
echo "   - Digest: $DIGEST"
echo "   - Server: Should listen on 127.0.0.1:5000"
echo "   - Client: Should connect to https://127.0.0.1:5000"
echo ""
echo "🚀 To run:"
echo "   1. Terminal 1: cargo run --bin boid-wars-server"
echo "   2. Terminal 2: cd bevy-client && wasm-bindgen-cli serve --features client"
echo "   3. Open browser to http://localhost:8080"