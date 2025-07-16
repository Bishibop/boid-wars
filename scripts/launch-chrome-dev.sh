#!/bin/bash

# Chrome launcher for Boid Wars development with WebTransport support

SECURITY_LEVEL="${1:-force-quic}"
URL="${2:-http://localhost:8080}"

show_help() {
    echo "Boid Wars Chrome Development Launcher"
    echo "===================================="
    echo ""
    echo "Usage: $0 [security-level] [url]"
    echo ""
    echo "Security levels:"
    echo "  secure      - Use SPKI certificate hash (most secure, requires cert digest)"
    echo "  force-quic  - Force QUIC/WebTransport on localhost (default)"
    echo "  insecure    - Ignore all certificate errors (least secure)"
    echo "  help        - Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                    # Default: force-quic mode"
    echo "  $0 secure             # Use certificate hash validation"
    echo "  $0 insecure           # Ignore all certificate errors"
    echo "  $0 force-quic http://localhost:3000"
    echo ""
    echo "Manual Chrome Setup:"
    echo "==================="
    echo "1. Open chrome://flags"
    echo "2. Search for 'WebTransport Developer Mode'"
    echo "3. Enable it and restart Chrome"
    echo ""
    echo "Note: For production use, WebSocket is recommended for local development"
    echo "to avoid certificate complexity. WebTransport is best for production."
}

# Detect Chrome executable based on OS
detect_chrome() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if [ -d "/Applications/Google Chrome.app" ]; then
            echo "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
        elif [ -d "/Applications/Google Chrome Canary.app" ]; then
            echo "/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary"
        else
            echo "google-chrome"
        fi
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        if command -v google-chrome &> /dev/null; then
            echo "google-chrome"
        elif command -v google-chrome-stable &> /dev/null; then
            echo "google-chrome-stable"
        elif command -v chromium &> /dev/null; then
            echo "chromium"
        elif command -v chromium-browser &> /dev/null; then
            echo "chromium-browser"
        else
            echo "google-chrome"
        fi
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
        # Windows
        echo "chrome"
    else
        echo "google-chrome"
    fi
}

if [ "$SECURITY_LEVEL" == "help" ] || [ "$SECURITY_LEVEL" == "-h" ] || [ "$SECURITY_LEVEL" == "--help" ]; then
    show_help
    exit 0
fi

CHROME_EXEC=$(detect_chrome)

# Check if Chrome is available
if ! command -v "$CHROME_EXEC" &> /dev/null && [ ! -f "$CHROME_EXEC" ]; then
    echo "Error: Chrome/Chromium not found!"
    echo "Please install Google Chrome or Chromium to use WebTransport."
    exit 1
fi

# Common flags for all modes
COMMON_FLAGS=(
    "--enable-features=WebTransport"
    "--enable-experimental-web-platform-features"
    "--disable-web-security"
    "--user-data-dir=/tmp/chrome-dev-$$"
)

case "$SECURITY_LEVEL" in
    "secure")
        echo "Launching Chrome with SPKI certificate validation..."
        echo "Note: You need to update the certificate hash in your client code"
        
        # Get certificate digest if available
        CERT_PATH="$HOME/.boid-wars/certs/localhost.pem"
        if [ -f "$CERT_PATH" ]; then
            DIGEST=$(openssl x509 -in "$CERT_PATH" -outform der | openssl dgst -sha256 | cut -d' ' -f2)
            echo "Current certificate digest: $DIGEST"
            echo ""
        fi
        
        "$CHROME_EXEC" "${COMMON_FLAGS[@]}" \
            --ignore-certificate-errors-spki-list="$DIGEST" \
            "$URL"
        ;;
        
    "force-quic")
        echo "Launching Chrome with forced QUIC/WebTransport..."
        echo "This forces WebTransport on localhost:5000"
        
        "$CHROME_EXEC" "${COMMON_FLAGS[@]}" \
            --origin-to-force-quic-on=localhost:5000 \
            --ignore-certificate-errors \
            "$URL"
        ;;
        
    "insecure")
        echo "Launching Chrome with all certificate checks disabled..."
        echo "⚠️  WARNING: This is insecure and should only be used for testing!"
        
        "$CHROME_EXEC" "${COMMON_FLAGS[@]}" \
            --ignore-certificate-errors \
            --allow-insecure-localhost \
            --disable-web-security \
            "$URL"
        ;;
        
    *)
        echo "Error: Unknown security level '$SECURITY_LEVEL'"
        echo ""
        show_help
        exit 1
        ;;
esac

echo ""
echo "Chrome launched. Check the browser console for any errors."
echo ""
echo "Troubleshooting:"
echo "- If WebTransport fails, try using WebSocket for local development"
echo "- Check chrome://flags for WebTransport Developer Mode"
echo "- Ensure certificates are generated with scripts/setup-certs.sh"