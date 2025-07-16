#!/bin/bash
echo "📥 Downloading Firefox 132.0.2 for macOS..."
echo ""

# Download URL for macOS
URL="https://ftp.mozilla.org/pub/firefox/releases/132.0.2/mac/en-US/Firefox%20132.0.2.dmg"

# Download to Downloads folder
cd ~/Downloads
curl -L -O "$URL"

echo ""
echo "✅ Downloaded to ~/Downloads/Firefox 132.0.2.dmg"
echo ""
echo "📝 Installation steps:"
echo "1. Open the DMG file"
echo "2. Drag Firefox to a different location (not Applications)"
echo "3. Rename it to 'Firefox 132' to keep it separate"
echo "4. Run it and it should work with WebTransport!"
echo ""
echo "⚠️  Turn off auto-updates in this version:"
echo "   Settings → General → Firefox Updates → Check for updates but let you choose"