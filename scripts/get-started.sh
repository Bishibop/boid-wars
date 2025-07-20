#!/bin/bash

set -e

echo "🎮 Boid Wars - Getting Started"
echo "=============================="
echo ""
echo "This script will set up everything you need to run Boid Wars on your machine."
echo ""

# Function to install prerequisites on macOS
install_macos_prereqs() {
    echo "📦 Installing prerequisites for macOS..."
    
    # Check if Homebrew is installed
    if ! command -v brew &> /dev/null; then
        echo "Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
    
    # Install Rust if not present
    if ! command -v rustc &> /dev/null; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    # Install Node.js if not present
    if ! command -v node &> /dev/null; then
        echo "Installing Node.js..."
        brew install node
    fi
    
    # Install mkcert if not present
    if ! command -v mkcert &> /dev/null; then
        echo "Installing mkcert..."
        brew install mkcert
        mkcert -install
    fi
    
    # Install wasm-pack if not present
    if ! command -v wasm-pack &> /dev/null; then
        echo "Installing wasm-pack..."
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    fi
    
    # Add WASM target
    rustup target add wasm32-unknown-unknown
}

# Function to install prerequisites on Linux
install_linux_prereqs() {
    echo "📦 Installing prerequisites for Linux..."
    
    # Install Rust if not present
    if ! command -v rustc &> /dev/null; then
        echo "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi
    
    # Install Node.js if not present (assumes Ubuntu/Debian)
    if ! command -v node &> /dev/null; then
        echo "Installing Node.js..."
        curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
        sudo apt-get install -y nodejs
    fi
    
    # Install wasm-pack if not present
    if ! command -v wasm-pack &> /dev/null; then
        echo "Installing wasm-pack..."
        curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
    fi
    
    # Install mkcert
    if ! command -v mkcert &> /dev/null; then
        echo "Installing mkcert..."
        curl -JLO "https://dl.filippo.io/mkcert/latest?for=linux/amd64"
        chmod +x mkcert-v*-linux-amd64
        sudo mv mkcert-v*-linux-amd64 /usr/local/bin/mkcert
        mkcert -install
    fi
    
    # Add WASM target
    rustup target add wasm32-unknown-unknown
}

# Detect OS and install prerequisites
echo "🔍 Detecting operating system..."
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "macOS detected"
    install_macos_prereqs
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Linux detected"
    install_linux_prereqs
else
    echo "❌ Unsupported operating system: $OSTYPE"
    echo "Please install prerequisites manually:"
    echo "  - Rust: https://rustup.rs/"
    echo "  - Node.js: https://nodejs.org/"
    echo "  - wasm-pack: https://rustwasm.github.io/wasm-pack/"
    echo "  - mkcert: https://github.com/FiloSottile/mkcert"
    exit 1
fi

echo ""
echo "✅ Prerequisites installed!"
echo ""

# Run the existing setup process
echo "🚀 Setting up Boid Wars project..."
echo ""

# Check prerequisites
echo "🔍 Verifying installation..."
./scripts/check-prereqs.sh

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file..."
    cp .env.example .env
    echo "✅ Created .env file"
fi

# Setup certificates
echo "🔒 Setting up SSL certificates..."
./scripts/setup-certs.sh

# Build the project
echo "🔨 Building project..."
cargo build --all
./scripts/build-wasm.sh

echo ""
echo "🎉 Setup complete!"
echo ""
echo "To start developing:"
echo "  make dev     # Start both server and client"
echo ""
echo "Or run components separately:"
echo "  make server  # Start just the server"
echo "  make client  # Start just the client"
echo ""
echo "Open http://localhost:8080 in your browser to play!"
echo ""
echo "For more commands, run: make help"