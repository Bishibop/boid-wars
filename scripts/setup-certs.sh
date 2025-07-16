#!/bin/bash
set -e

CERT_DIR="$HOME/.boid-wars/certs"
CERT_DAYS=13

echo "Boid Wars Certificate Setup"
echo "=========================="

# Create certificate directory if it doesn't exist
mkdir -p "$CERT_DIR"

# Check if certificates exist and are valid
if [ -f "$CERT_DIR/localhost.pem" ] && [ -f "$CERT_DIR/localhost-key.pem" ]; then
    # Check if certificate is still valid
    if openssl x509 -checkend 86400 -noout -in "$CERT_DIR/localhost.pem" 2>/dev/null; then
        echo "✅ Valid certificates already exist at: $CERT_DIR"
        echo ""
        echo "Certificate digest (SHA-256):"
        openssl x509 -in "$CERT_DIR/localhost.pem" -outform der | openssl dgst -sha256 | cut -d' ' -f2
        exit 0
    else
        echo "⚠️  Existing certificate has expired or expires within 24 hours"
        echo "Generating new certificate..."
    fi
fi

# Try using mkcert first (preferred method)
if command -v mkcert &> /dev/null; then
    echo "Using mkcert to generate certificates..."
    
    # Ensure mkcert is initialized
    mkcert -install 2>/dev/null || true
    
    # Generate certificates with 13-day validity for WebTransport
    cd "$CERT_DIR"
    CAROOT="$HOME/.boid-wars/mkcert-ca" mkcert -cert-file localhost.pem -key-file localhost-key.pem localhost 127.0.0.1 ::1
    
    echo "✅ Certificates generated successfully with mkcert"
    
# Fallback to OpenSSL
else
    echo "mkcert not found, using OpenSSL..."
    echo "Note: For better browser compatibility, consider installing mkcert: https://github.com/FiloSottile/mkcert"
    echo ""
    
    # Generate certificate with OpenSSL
    openssl req -x509 -newkey rsa:4096 -sha256 -days $CERT_DAYS \
        -nodes -keyout "$CERT_DIR/localhost-key.pem" -out "$CERT_DIR/localhost.pem" \
        -subj "/CN=localhost" \
        -addext "subjectAltName=DNS:localhost,DNS:*.localhost,IP:127.0.0.1,IP:::1" \
        -addext "keyUsage=digitalSignature,keyEncipherment" \
        -addext "extendedKeyUsage=serverAuth" \
        2>/dev/null
    
    echo "✅ Certificates generated successfully with OpenSSL"
    echo "⚠️  Note: These are self-signed certificates with $CERT_DAYS-day validity"
fi

echo ""
echo "Certificate Information:"
echo "======================="
echo "Location: $CERT_DIR"
echo "Files:"
echo "  - Certificate: localhost.pem"
echo "  - Private Key: localhost-key.pem"
echo ""
echo "Certificate digest (SHA-256):"
DIGEST=$(openssl x509 -in "$CERT_DIR/localhost.pem" -outform der | openssl dgst -sha256 | cut -d' ' -f2)
echo "$DIGEST"
echo ""
echo "This digest can be used in the WASM client for certificate validation."
echo ""
echo "To use these certificates:"
echo "1. Update server/src/main.rs with the certificate paths"
echo "2. Update bevy-client with the certificate digest"
echo "3. Use 'scripts/launch-chrome-dev.sh' to open Chrome with proper flags"