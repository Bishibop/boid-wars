# Boid Wars Quick Reference

## 🚀 Common Commands

```bash
make prereqs   # Check prerequisites
make setup     # Initial setup
make dev       # Run everything (hot reload)
make test      # Run all tests
make lint      # Check code style
make format    # Auto-format code
make check     # Full validation
```

## 🔧 Development Workflow

1. **Before starting work**: `git pull && make prereqs`
2. **While developing**: `make dev` (in one terminal)
3. **Before committing**: `make check`
4. **Commit format**: `feat: add boid flocking behavior`

## 📁 Project Structure

```
server/          → Rust game server (Bevy + Lightyear)
client/          → TypeScript client (Pixi.js)
shared/          → Protocol definitions
lightyear-wasm/  → WASM networking bridge
scripts/         → Dev tools
```

## 🎮 Key Concepts

### Rust/Bevy
- **Components**: Data only (`Velocity`, `Position`)
- **Systems**: Logic (`move_players`, `update_boids`)
- **Resources**: Shared state (`GameConfig`, `NetworkStats`)

### TypeScript/Pixi
- **Sprites**: Use object pools
- **Updates**: In `requestAnimationFrame`
- **Network**: Via WASM bridge

### Performance Rules
- 🚫 No allocations in hot paths
- ✅ Pool frequently created objects
- ✅ Batch operations
- ✅ Profile before optimizing

## 🐛 Common Issues

**WASM build fails**
```bash
rustup target add wasm32-unknown-unknown
```

**Certificate errors**
```bash
./scripts/setup-certs.sh
```

**Type errors after protocol change**
```bash
make wasm  # Rebuild bridge
```

## 📊 Performance Targets

- Server: 60 Hz tick rate
- Client: 60 FPS
- Network: 30 Hz updates
- Latency: <150ms
- Boids: 10,000+

## 🔍 Debugging

### Rust
```rust
// Temporary debug logging
dbg!(&variable);
tracing::debug!("Event: {:?}", data);
```

### TypeScript
```typescript
// Performance monitoring
console.time('update');
// ... code ...
console.timeEnd('update');

// Check in browser console
window.perfMonitor.stats()
```

## 🎯 Code Style TL;DR

### Rust
- Use `cargo fmt` before committing
- Prefer `&str` over `String` for parameters
- Use `#[derive]` liberally
- Document public APIs

### TypeScript
- No `any` types
- Explicit return types
- `const` by default
- Pool game objects

## 🚢 Deployment

```bash
make build     # Production build
make docker    # Docker image
```

## 📚 Resources

- [Architecture](../ARCHITECTURE.md)
- [Coding Standards](../CODING_STANDARDS.md)
- [Contributing](../CONTRIBUTING.md)