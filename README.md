# Boid Wars

A high-performance multiplayer browser game featuring massive swarms of AI-controlled enemies (boids) in intense twin-stick shooter combat.

## Overview

Boid Wars is a competitive space shooter where players battle against thousands of coordinated enemy swarms while competing with other players. Built with cutting-edge web technologies for minimal latency and maximum performance.

### Key Features
- **Massive Scale**: 10,000+ AI enemies using optimized boid flocking algorithms
- **Low Latency**: WebTransport protocol for near-instant response times
- **Browser-Based**: No downloads required - runs in modern web browsers
- **Server Authoritative**: Secure, cheat-resistant architecture

## Tech Stack

- **Server**: Rust + Bevy ECS + Lightyear (WebTransport networking)
- **Client**: TypeScript + Pixi.js (WebGL rendering)
- **Protocol**: WebTransport for low-latency communication
- **Bridge**: Minimal WASM module for networking

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) v20+
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- [mkcert](https://github.com/FiloSottile/mkcert) (for local HTTPS)

## Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/boid_wars.git
   cd boid_wars
   ```

2. **Check prerequisites**
   ```bash
   make prereqs
   ```
   
   If anything is missing, follow the installation instructions shown.

3. **Set up the project**
   ```bash
   make setup
   ```
   
   This will:
   - Create your `.env` file
   - Generate SSL certificates in a secure location
   - Install all dependencies
   - Build the WASM bridge

4. **Start development**
   ```bash
   make dev
   ```
   
   This runs both server and client with hot reloading.

5. **Open the game**
   Navigate to https://localhost:5173 in Chrome, Edge, or Firefox

## Development

### Project Structure
```
boid_wars/
├── server/          # Rust game server
├── client/          # TypeScript web client
├── lightyear-wasm/  # WASM networking bridge
├── shared/          # Shared protocol types
├── scripts/         # Development scripts
└── docs/            # Documentation
```

### Useful Commands

```bash
# Development (hot reload)
make dev              # Run everything concurrently
./scripts/run-server.sh   # Just the server
cd client && npm run dev  # Just the client

# Testing
cargo test --all      # Rust tests
cd client && npm test # Client tests

# Code Quality
cargo fmt --all       # Format Rust code
cargo clippy --all    # Lint Rust code
cd client && npm run lint     # Lint TypeScript
cd client && npm run format   # Format TypeScript

# Building
cargo build --release # Production server
cd client && npm run build    # Production client
```

### Performance Monitoring

The client includes built-in performance monitoring in development mode:
- FPS counter (top-right)
- Entity count
- Network latency

Access debug tools in the browser console:
- `window.logger` - Logging utilities
- `window.perfMonitor` - Performance stats
- `window.pixiApp` - Pixi.js application

## Architecture

- **Server Authoritative**: All game logic runs on the server
- **Entity Replication**: Automatic synchronization via Lightyear
- **Interest Management**: Only relevant entities sent to each client
- **Delta Compression**: Minimal bandwidth usage

See [ARCHITECTURE.md](ARCHITECTURE.md) for system design and [docs/](docs/) for additional documentation.

## Browser Support

| Browser | Support | Notes |
|---------|---------|-------|
| Chrome | ✅ Full | Recommended |
| Edge | ✅ Full | Chromium-based |
| Firefox | ✅ Full | Good performance |
| Safari | ❌ None | No WebTransport support |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## Troubleshooting

Common issues and solutions are documented in [docs/troubleshooting.md](docs/troubleshooting.md).

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Status

🚧 **Early Development** - This project is in active development. Expect breaking changes.