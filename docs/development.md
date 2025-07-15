# Boid Wars Development Guide

## Quick Start

### Prerequisites
- Rust (stable) with `wasm32-unknown-unknown` target
- Node.js 20+
- wasm-pack
- mkcert (for local HTTPS)

### First Time Setup
```bash
# Install dependencies
npm install
cd client && npm install && cd ..

# Generate certificates (if not done)
cd deploy && mkcert localhost 127.0.0.1 ::1 && cd ..

# Build everything
cargo build --all
./scripts/build-wasm.sh
```

### Running the Project

1. **Start the server:**
   ```bash
   ./scripts/run-server.sh
   ```

2. **In another terminal, start the client:**
   ```bash
   cd client && npm run dev
   ```

3. **Open browser:**
   - Navigate to https://localhost:5173
   - You should see a green circle moving up and down

## Development Workflow

### Hot Reloading
- **Server**: Use `cargo watch` or `bacon` in the server directory
- **Client**: Vite provides automatic hot module replacement
- **WASM**: Run `./scripts/build-wasm.sh` after changes

### Useful Commands
```bash
# Run all tests
cargo test --all

# Check code without building
cargo check --all

# Format code
cargo fmt --all

# Run clippy
cargo clippy --all

# Type check TypeScript
cd client && npm run type-check
```

## Project Structure
```
boid-wars/
├── server/          # Game server (Rust + Bevy + Lightyear)
├── lightyear-wasm/  # WASM networking bridge
├── shared/          # Shared types between server and WASM
├── client/          # Browser client (TypeScript + Pixi.js)
├── scripts/         # Development helper scripts
└── docs/            # Documentation
```

## Environment Variables
Copy `.env.example` to `.env` to get started:
```bash
cp .env.example .env
```

Key variables:
- `RUST_LOG`: Control logging verbosity
- `GAME_SERVER_PORT`: Server port (default: 3000)
- `GAME_SERVER_CERT/KEY`: SSL certificate paths
- `VITE_SERVER_URL`: Server URL for client
- `VITE_LOG_LEVEL`: Client-side logging level

## Troubleshooting
See [troubleshooting.md](troubleshooting.md) for common issues.