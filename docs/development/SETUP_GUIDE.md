# Setup Guide

This guide will help you get Boid Wars running on your local machine from scratch.

## System Requirements

- **OS**: macOS, Linux, or Windows (with WSL2 recommended)
- **RAM**: 8GB minimum, 16GB recommended
- **Browser**: Chrome, Edge, or Firefox (Safari not supported - no WebTransport)

## Step 1: Install Prerequisites

### Rust
```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts, then reload your shell
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

### Required Rust Tools
```bash
# Install wasm target
rustup target add wasm32-unknown-unknown

# Install wasm-pack for building WASM
cargo install wasm-pack

# Install trunk for serving WASM client
cargo install trunk

# Install cargo-watch for hot reload (optional but recommended)
cargo install cargo-watch
```

### System Tools
```bash
# macOS
brew install make

# Ubuntu/Debian
sudo apt-get update
sudo apt-get install build-essential

# Verify make is installed
make --version
```

## Step 2: Clone and Setup Project

```bash
# Clone the repository
git clone https://github.com/yourusername/boid_wars.git
cd boid_wars

# Run the setup script
make setup
```

The setup script will:
- Check all prerequisites
- Install Rust dependencies
- Build the WASM client
- Create necessary directories

## Step 3: Development Setup

### Option A: Quick Start (Recommended)
```bash
# Start both server and client with WebSocket (no certificates needed!)
make dev
```

This will:
- Start the game server on `ws://localhost:5001`
- Start the web client on `http://localhost:8080`
- Enable hot reload for both

Open http://localhost:8080 in your browser and you should see the game!

### Option B: Run Components Separately
```bash
# Terminal 1: Run the server
make dev-server

# Terminal 2: Run the client
make dev-client
```

### Option C: WebTransport Testing (Advanced)
If you need to test WebTransport specifically:

```bash
# 1. Generate certificates (one time)
./scripts/setup-certs.sh

# 2. Start server with WebTransport
cargo run --bin server --features webtransport

# 3. Launch Chrome with certificate bypass
./scripts/launch-chrome-dev.sh force-quic
```

See [WEBTRANSPORT_GUIDE.md](WEBTRANSPORT_GUIDE.md) for details on why this is complex.

## Step 4: Verify Setup

1. **Server is running**: You should see logs like:
   ```
   INFO server: Listening on 127.0.0.1:5001
   INFO server: Game loop running at 30 Hz
   ```

2. **Client loads**: Browser should show the game canvas
3. **No console errors**: Check browser DevTools (F12)

## Troubleshooting

### "cargo: command not found"
- Ensure you've reloaded your shell after installing Rust
- Try: `source $HOME/.cargo/env`

### "wasm-pack: command not found"
```bash
cargo install wasm-pack --force
```

### WASM build fails
```bash
# Clear WASM cache and rebuild
rm -rf target/wasm32-unknown-unknown
make build-wasm
```

### Port already in use
```bash
# Find what's using port 5001
lsof -i :5001  # macOS/Linux
netstat -ano | findstr :5001  # Windows

# Kill the process or use different ports
SERVER_PORT=5002 make dev-server
```

### Browser shows blank page
1. Check browser console for errors (F12)
2. Ensure you're using a supported browser (not Safari)
3. Try clearing browser cache
4. Check that WASM is enabled in your browser

### Certificate errors with WebTransport
- This is expected! Use WebSocket for local development
- Run `make dev` instead of trying to fix certificates
- See [WEBTRANSPORT_GUIDE.md](WEBTRANSPORT_GUIDE.md) for the full story

## IDE Setup (Optional)

### VS Code
1. Install extensions:
   - rust-analyzer
   - Even Better TOML
   - Error Lens

2. Create `.vscode/settings.json`:
```json
{
    "rust-analyzer.cargo.target": "wasm32-unknown-unknown",
    "rust-analyzer.checkOnSave.command": "clippy"
}
```

### RustRover / CLion
1. Open project root
2. Enable Rust plugin
3. Set WASM target in Cargo settings

## Next Steps

Now that you have the project running:

1. **Read the Architecture**: [SYSTEM_ARCHITECTURE.md](../../SYSTEM_ARCHITECTURE.md)
2. **Learn Development Workflow**: [DEVELOPMENT.md](DEVELOPMENT.md)
3. **Understand Code Standards**: [CODING_STANDARDS.md](CODING_STANDARDS.md)
4. **Make your first change**: Try modifying a constant in `shared/src/protocol.rs`

## Getting Help

- **Documentation**: Check the `docs/` directory
- **Issues**: Search existing GitHub issues
- **Debugging**: See [DEVELOPMENT.md#debugging](DEVELOPMENT.md#debugging)

## Production Build

When you're ready to build for production:

```bash
# Build optimized version
make build-release

# Test production build locally
./target/release/server
```

The production build:
- Enables optimizations
- Uses WebTransport (requires real certificates)
- Minimizes WASM size
- Removes debug features