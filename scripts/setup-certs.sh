#!/bin/bash
set -e

echo "🔐 Setting up SSL certificates for Boid Wars..."

# Change to project root
cd "$(dirname "$0")/.."

# Define cert directory
CERT_DIR="$HOME/.boid-wars/certs"

# Check if mkcert is installed
if ! command -v mkcert &> /dev/null; then
    echo "❌ mkcert not found! Please install it first:"
    echo "   brew install mkcert    # macOS"
    echo "   See https://github.com/FiloSottile/mkcert for other platforms"
    exit 1
fi

# Create cert directory
echo "📁 Creating certificate directory at $CERT_DIR..."
mkdir -p "$CERT_DIR"

# Install local CA if not already done
echo "🏛️  Installing local CA (if needed)..."
mkcert -install

# Generate certificates
echo "🔧 Generating certificates..."
cd "$CERT_DIR"
mkcert localhost 127.0.0.1 ::1

# Rename to match expected names
mv localhost+2.pem localhost.pem
mv localhost+2-key.pem localhost-key.pem

echo "✅ Certificates created successfully!"
echo ""
echo "📝 Update your .env file with these paths:"
echo "   GAME_SERVER_CERT=$CERT_DIR/localhost.pem"
echo "   GAME_SERVER_KEY=$CERT_DIR/localhost-key.pem"
echo ""
echo "🔒 These certificates are stored securely in your home directory"