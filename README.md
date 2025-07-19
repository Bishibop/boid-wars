# Boid Wars

A high-performance multiplayer browser game featuring massive swarms of AI-controlled enemies (boids) in intense twin-stick shooter combat.

## Overview

Boid Wars is a competitive space shooter where players battle against thousands of coordinated enemy swarms while competing with other players. Built with cutting-edge web technologies for minimal latency and maximum performance.

### Key Features
- **Massive Scale**: 10,000+ AI enemies using optimized boid flocking algorithms
- **Realistic Physics**: Thrust-based movement, projectile physics, and collision detection
- **Low Latency**: WebTransport protocol for near-instant response times
- **Browser-Based**: No downloads required - runs in modern web browsers
- **Server Authoritative**: Secure, cheat-resistant architecture

## Tech Stack

- **Server**: Rust + Bevy ECS + Lightyear 0.20 (WebTransport/WebSocket)
- **Client**: Rust + Bevy ECS (WASM build)
- **Physics**: Rapier 2D for realistic movement and collisions
- **Protocol**: WebTransport (production) / WebSocket (development)
- **Architecture**: Unified Rust codebase for maximum performance

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- [trunk](https://trunkrs.dev/) (for serving WASM client)
- Modern browser with WebAssembly support

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
   - Install Rust dependencies
   - Build the WASM client
   - Set up development environment

4. **Start development**
   ```bash
   make dev
   ```
   
   This runs both server and client with hot reloading.

5. **Open the game**
   Navigate to http://localhost:8080 in Chrome, Edge, or Firefox

## Development

### Project Structure
```
boid_wars/
├── server/          # Rust game server
├── bevy-client/     # Bevy WASM client
├── shared/          # Shared protocol types
├── scripts/         # Development scripts
└── docs/            # Documentation

# Legacy (archived):
├── client/          # Original TypeScript client
└── lightyear-wasm/  # Attempted WASM bridge
```

### Useful Commands

```bash
# Development
make dev              # Run server and client concurrently
make dev-server       # Just the server
make dev-client       # Just the client

# Testing & Quality
make check            # Run all formatting, linting, and tests
cargo test --all      # Rust tests
cargo fmt --all       # Format Rust code
cargo clippy --all    # Lint Rust code

# Building
make build            # Build everything for production
make build-wasm       # Build WASM client only
cargo build --release # Build server only
```

### Performance Monitoring

The Bevy client includes built-in diagnostics:
- FPS counter overlay
- Entity count display
- Network statistics
- Frame time graphs

Enable diagnostics with the `--features debug` flag during development.

## Architecture

- **Server Authoritative**: All game logic runs on the server
- **Entity Replication**: Automatic synchronization via Lightyear
- **Physics Integration**: Rapier 2D with dual coordinate system (Transform/Position)
- **Object Pooling**: Efficient projectile management with generation tracking
- **Interest Management**: Only relevant entities sent to each client
- **Delta Compression**: Minimal bandwidth usage

See [SYSTEM_ARCHITECTURE.md](SYSTEM_ARCHITECTURE.md) for system design and [docs/](docs/) for additional documentation.

## Browser Support

| Browser | Support | Notes |
|---------|---------|-------|
| Chrome | ✅ Full | Recommended |
| Edge | ✅ Full | Chromium-based |
| Firefox | ✅ Full | Good performance |
| Safari | ❌ None | No WebTransport support |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup and guidelines.

## Deployment

### Local Docker
```bash
# Build and run locally
docker build . -t boid-wars
docker run -p 8080:8080 boid-wars

# Or use docker-compose
docker-compose up
```

### Fly.io Deployment
```bash
# Deploy to Fly.io (uses Dockerfile)
fly deploy

# Or use the deployment script
./scripts/deploy.sh
```

The Docker build uses optimized settings to work within memory constraints:
- `lto = "thin"` for reduced memory usage
- `codegen-units = 16` for better parallelization
- `opt-level = 2` for balanced optimization

## Troubleshooting

Common issues and solutions are documented in [docs/troubleshooting.md](docs/troubleshooting.md).

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Status

🚧 **Architecture Migration** - Currently migrating from TypeScript/Pixi.js to full Bevy WASM client. Core gameplay in active development.

### Recent Changes
- Implemented full physics system with Rapier 2D integration
- Added thrust-based player movement and projectile combat
- Optimized entity spawning with bounded object pooling
- Switched to full Bevy WASM client for unified architecture
- Using Lightyear 0.20 for networking (stable API)
- Implemented WebSocket fallback for local development