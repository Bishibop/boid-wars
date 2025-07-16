# Development Guide

This guide covers the day-to-day development workflow for Boid Wars.

## Table of Contents
1. [Development Environment Setup](#development-environment-setup)
2. [Development Workflow](#development-workflow)
3. [Code Organization](#code-organization)
4. [Common Development Tasks](#common-development-tasks)
5. [Build and Run Commands](#build-and-run-commands)
6. [Testing](#testing)
7. [Debugging](#debugging)
8. [Performance Considerations](#performance-considerations)
9. [Deployment](#deployment)

## Development Environment Setup

### Prerequisites
- Rust (stable) - Install via [rustup](https://rustup.rs/)
- wasm-pack - `cargo install wasm-pack`
- trunk - `cargo install trunk`
- make - Usually pre-installed on Unix systems

### Recommended IDE Setup
- **VS Code** with extensions:
  - rust-analyzer
  - Even Better TOML
  - crates
  - Error Lens
- **RustRover** or **CLion** with Rust plugin

### Initial Setup
```bash
# Clone the repository
git clone https://github.com/yourusername/boid_wars.git
cd boid_wars

# Install dependencies and build
make setup
```

## Development Workflow

### Local Development (Recommended)

We use **WebSocket** for local development to avoid WebTransport certificate complexity:

```bash
# Start both server and client with hot reload
make dev

# Or run separately:
make dev-server  # Server on ws://localhost:5001
make dev-client  # Client on http://localhost:8080
```

**Why WebSocket locally?**
- No certificate setup required
- Instant startup
- Same networking behavior
- WebTransport is used automatically in production

### WebTransport Testing (Optional)

If you need to test WebTransport specifically:

```bash
# 1. Generate certificates (once)
./scripts/setup-certs.sh

# 2. Start server with WebTransport
cargo run --bin server --features webtransport

# 3. Launch Chrome with proper flags
./scripts/launch-chrome-dev.sh force-quic
```

See [WEBTRANSPORT_GUIDE.md](WEBTRANSPORT_GUIDE.md) for details on certificate challenges.

## Code Organization

```
boid_wars/
├── shared/           # Shared game logic and protocol
│   └── src/
│       ├── protocol.rs    # Network protocol definition
│       ├── components.rs  # ECS components
│       └── messages.rs    # Network messages
├── server/           # Rust game server
│   └── src/
│       ├── main.rs       # Server entry point
│       ├── systems/      # Bevy systems
│       └── resources/    # Server resources
├── bevy-client/      # Bevy WASM client
│   └── src/
│       ├── lib.rs        # Client entry point
│       ├── systems/      # Client systems
│       └── rendering/    # Rendering logic
└── scripts/          # Development scripts
```

### Key Patterns

1. **Shared Protocol**: All network messages and components are defined in `shared/`
2. **System Organization**: Group related systems in subdirectories
3. **Component Naming**: Use descriptive names ending with component type (e.g., `PlayerInput`, `BoidState`)

## Common Development Tasks

### Adding a New Component

1. Define in `shared/src/components.rs`:
```rust
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}
```

2. Register in protocol (`shared/src/protocol.rs`):
```rust
app.register_component::<Health>(ChannelDirection::ServerToClient)
    .add_prediction::<Health>(ComponentSyncMode::Full);
```

### Adding a New Network Message

1. Define in `shared/src/messages.rs`:
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct FireProjectile {
    pub origin: Vec2,
    pub direction: Vec2,
}
```

2. Register in protocol:
```rust
app.register_message::<FireProjectile>(ChannelDirection::ClientToServer);
```

### Adding a New System

Server system example:
```rust
// server/src/systems/combat.rs
pub fn handle_projectile_firing(
    mut commands: Commands,
    mut events: EventReader<FireProjectile>,
) {
    for event in events.read() {
        // Spawn projectile entity
    }
}

// Register in server/src/main.rs
app.add_systems(Update, handle_projectile_firing);
```

## Build and Run Commands

### Development
```bash
make dev              # Run everything with hot reload
make check            # Format, lint, and test
make build-wasm       # Build WASM client only
```

### Production
```bash
make build            # Build everything for production
make build-release    # Optimized release build
```

### Useful Cargo Commands
```bash
# Run with specific features
cargo run --bin server --features debug

# Check for issues without building
cargo check --all

# Run specific tests
cargo test test_name

# Build with verbose output
cargo build -vv
```

## Testing

### Running Tests
```bash
# All tests
make test

# Rust tests only
cargo test --all

# Specific test
cargo test test_boid_movement

# With output
cargo test -- --nocapture
```

### Writing Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_boid_separation() {
        // Arrange
        let boid_pos = Vec2::ZERO;
        let neighbor_positions = vec![Vec2::new(1.0, 0.0)];
        
        // Act
        let force = calculate_separation(boid_pos, &neighbor_positions);
        
        // Assert
        assert!(force.x < 0.0); // Should move away
    }
}
```

### Performance Tests
For critical paths, add benchmarks:
```rust
#[bench]
fn bench_spatial_query(b: &mut Bencher) {
    // Setup spatial index with 10k entities
    b.iter(|| {
        // Query neighbors
    });
}
```

## Debugging

### Client Debugging

1. **Bevy Inspector**: Enable with `--features debug`
2. **Browser DevTools**: F12 for console and network inspection
3. **WASM Logging**:
```rust
web_sys::console::log_1(&format!("Debug: {:?}", value).into());
```

### Server Debugging

1. **Structured Logging**:
```rust
info!("Player connected"; player_id = ?id, addr = %addr);
```

2. **Bevy Diagnostics**:
```rust
app.add_plugins(FrameTimeDiagnosticsPlugin)
   .add_plugins(EntityCountDiagnosticsPlugin);
```

### Network Debugging

1. **Packet Inspection**: Use browser DevTools Network tab
2. **Message Logging**: Enable with `RUST_LOG=lightyear=debug`
3. **Entity Replication**: Monitor with custom diagnostics

## Performance Considerations

### Entity Limits
- Target: 10,000+ entities at 60 FPS
- Use spatial indexing for queries
- Batch similar operations

### Optimization Checklist
- [ ] Profile before optimizing
- [ ] Use Bevy's built-in batching
- [ ] Minimize component size
- [ ] Use change detection
- [ ] Pool temporary allocations

### Profiling Tools
```bash
# CPU profiling
cargo flamegraph --bin server

# WASM size analysis
wasm-opt -Os -g dist/client_bg.wasm -o optimized.wasm
twiggy top optimized.wasm
```

## Deployment

### Local Testing
```bash
# Build and test production mode locally
make build-release
./target/release/server
```

### Production Deployment
```bash
# Deploy to Fly.io
fly deploy

# Check deployment
fly status
fly logs
```

### Environment Variables
- `RUST_LOG` - Logging level (info, debug, trace)
- `SERVER_ADDR` - Server bind address
- `CERT_PATH` - Certificate path (production only)

## Troubleshooting

### Common Issues

**WASM build fails**
- Clear target directory: `rm -rf target/wasm32-unknown-unknown`
- Update wasm-pack: `cargo install wasm-pack --force`

**Certificate errors in browser**
- Use WebSocket for local development: `make dev`
- For WebTransport testing, see [WEBTRANSPORT_GUIDE.md](WEBTRANSPORT_GUIDE.md)

**High memory usage**
- Check for entity leaks in Bevy
- Profile with browser memory tools
- Verify spatial index cleanup

**Network lag**
- Check server tick rate (should be 30Hz)
- Monitor message size and frequency
- Enable compression for large updates

## Next Steps

- Read [SYSTEM_ARCHITECTURE.md](../../SYSTEM_ARCHITECTURE.md) for system design
- Check [CODING_STANDARDS.md](CODING_STANDARDS.md) for style guide
- See [docs/technical/](../technical/) for deep dives
- Join discussions in GitHub Issues