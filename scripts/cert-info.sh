#!/bin/bash

# Certificate information utility for Boid Wars

CERT_DIR="$HOME/.boid-wars/certs"
CERT_PATH="$CERT_DIR/localhost.pem"

show_help() {
    echo "Boid Wars Certificate Information"
    echo "================================"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  digest    - Show SHA-256 digest (default)"
    echo "  spki      - Show SPKI hash for Chrome"
    echo "  info      - Show certificate details"
    echo "  verify    - Verify certificate validity"
    echo "  help      - Show this help"
    echo ""
    echo "Examples:"
    echo "  $0              # Show digest"
    echo "  $0 info         # Show all certificate info"
    echo "  $0 verify       # Check if certificate is valid"
}

if [ ! -f "$CERT_PATH" ]; then
    echo "Error: No certificate found at $CERT_PATH"
    echo "Run 'scripts/setup-certs.sh' to generate certificates first."
    exit 1
fi

COMMAND="${1:-digest}"

case "$COMMAND" in
    "digest")
        echo "Certificate SHA-256 digest:"
        openssl x509 -in "$CERT_PATH" -outform der | openssl dgst -sha256 | cut -d' ' -f2
        ;;
        
    "spki")
        echo "Certificate SPKI hash (for Chrome --ignore-certificate-errors-spki-list):"
        openssl x509 -in "$CERT_PATH" -pubkey -noout | \
            openssl pkey -pubin -outform der | \
            openssl dgst -sha256 -binary | \
            base64
        ;;
        
    "info")
        echo "Certificate Information"
        echo "====================="
        echo ""
        echo "Location: $CERT_PATH"
        echo ""
        echo "Details:"
        openssl x509 -in "$CERT_PATH" -text -noout | grep -E "(Subject:|Issuer:|Not Before|Not After|Subject Alternative Name)" -A 1
        echo ""
        echo "SHA-256 Digest:"
        openssl x509 -in "$CERT_PATH" -outform der | openssl dgst -sha256 | cut -d' ' -f2
        echo ""
        echo "SPKI Hash:"
        openssl x509 -in "$CERT_PATH" -pubkey -noout | \
            openssl pkey -pubin -outform der | \
            openssl dgst -sha256 -binary | \
            base64
        ;;
        
    "verify")
        echo "Verifying certificate..."
        echo ""
        
        # Check if certificate exists
        if [ ! -f "$CERT_PATH" ]; then
            echo "❌ Certificate not found at $CERT_PATH"
            exit 1
        fi
        
        # Check validity period
        if openssl x509 -checkend 0 -noout -in "$CERT_PATH" 2>/dev/null; then
            echo "✅ Certificate is currently valid"
            
            # Show expiration
            EXPIRY=$(openssl x509 -enddate -noout -in "$CERT_PATH" | cut -d= -f2)
            echo "   Expires: $EXPIRY"
            
            # Check WebTransport compatibility (< 14 days)
            NOT_AFTER=$(openssl x509 -noout -enddate -in "$CERT_PATH" | cut -d= -f2)
            NOT_BEFORE=$(openssl x509 -noout -startdate -in "$CERT_PATH" | cut -d= -f2)
            
            # Convert to timestamps
            if [[ "$OSTYPE" == "darwin"* ]]; then
                # macOS
                NOT_AFTER_TS=$(date -j -f "%b %d %T %Y %Z" "$NOT_AFTER" +%s 2>/dev/null || date -j -f "%b %d %T %Y GMT" "$NOT_AFTER" +%s)
                NOT_BEFORE_TS=$(date -j -f "%b %d %T %Y %Z" "$NOT_BEFORE" +%s 2>/dev/null || date -j -f "%b %d %T %Y GMT" "$NOT_BEFORE" +%s)
            else
                # Linux
                NOT_AFTER_TS=$(date -d "$NOT_AFTER" +%s)
                NOT_BEFORE_TS=$(date -d "$NOT_BEFORE" +%s)
            fi
            
            VALIDITY_DAYS=$(( ($NOT_AFTER_TS - $NOT_BEFORE_TS) / 86400 ))
            
            if [ $VALIDITY_DAYS -le 14 ]; then
                echo "✅ WebTransport compatible (validity: $VALIDITY_DAYS days)"
            else
                echo "⚠️  Not WebTransport compatible (validity: $VALIDITY_DAYS days, max 14)"
            fi
            
            # Check if expiring soon
            if openssl x509 -checkend 86400 -noout -in "$CERT_PATH" 2>/dev/null; then
                echo "✅ Certificate valid for more than 24 hours"
            else
                echo "⚠️  Certificate expires within 24 hours!"
            fi
        else
            echo "❌ Certificate has expired!"
            EXPIRY=$(openssl x509 -enddate -noout -in "$CERT_PATH" | cut -d= -f2)
            echo "   Expired: $EXPIRY"
            echo ""
            echo "Run 'scripts/setup-certs.sh' to generate a new certificate."
            exit 1
        fi
        ;;
        
    "help"|"-h"|"--help")
        show_help
        ;;
        
    *)
        echo "Error: Unknown command '$COMMAND'"
        echo ""
        show_help
        exit 1
        ;;
esac