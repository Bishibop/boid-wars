# Boid Wars

A 2-player competitive space shooter featuring AI-controlled enemy swarms using flocking behavior.

## Overview

Boid Wars is a browser-based PvP space shooter where two players battle against coordinated AI enemies that move in realistic flocks. The game focuses on performance optimizations to handle 100+ entities at 60 FPS.

### Key Features
- **Flocking AI**: Enemy boids use realistic flocking algorithms for emergent group behavior
- **Performance Optimized**: Spatial grid system and object pooling for smooth gameplay
- **Three Boid Archetypes**: Assault, Defensive, and Recon groups with distinct behaviors
- **Physics-Based**: Thrust-based movement and projectile physics
- **Browser-Based**: Runs in modern web browsers without downloads

## Tech Stack

- **Server**: Rust + Bevy ECS + Lightyear 0.21
- **Client**: Rust + Bevy ECS (WASM build)
- **Physics**: Rapier 2D for movement and collisions
- **Networking**: WebSocket protocol

## Quick Start

**One-command setup:**
```bash
./scripts/get-started.sh
```

This script will automatically:
- Install all prerequisites (Rust, Node.js, wasm-pack, mkcert)
- Set up SSL certificates
- Build the project
- Configure the development environment

After setup completes:
```bash
make dev  # Start both server and client
```

Then open http://localhost:8080 in your browser.

### Manual Setup

If you prefer to install prerequisites manually:

1. **Install prerequisites**
   - [Rust](https://rustup.rs/) (stable toolchain)
   - [Node.js](https://nodejs.org/)
   - [wasm-pack](https://rustwasm.github.io/wasm-pack/)
   - [mkcert](https://github.com/FiloSottile/mkcert)

2. **Set up the project**
   ```bash
   make setup
   make dev
   ```

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

## Performance Optimizations

The game includes several key optimizations for handling 100+ entities at 60 FPS:

- **Spatial Grid System**: O(1) neighbor queries for flocking calculations instead of O(n²)
- **Object Pooling**: Pre-allocated projectile pools with generation tracking
- **Boid Archetypes**: Three distinct group behaviors (Assault, Defensive, Recon) with optimized update patterns
- **Memory Optimization**: Cache-friendly component design and pre-allocated buffers

## Browser Support

| Browser | Support | Notes |
|---------|---------|-------|
| Chrome | ✅ Full | Recommended |
| Edge | ✅ Full | Chromium-based |
| Firefox | ✅ Full | Good performance |
| Safari | ✅ Basic | WebSocket support only |

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

🚧 **In Development** - Core gameplay features implemented. Currently optimizing performance and adding game polish.

### Recent Changes
- Added three distinct boid group archetypes with unique behaviors  
- Implemented spatial grid optimization for flocking calculations
- Added object pooling for projectiles and entities
- Optimized rendering and networking for 100+ entity gameplay
- Enhanced boid AI with formation systems and combat behaviors