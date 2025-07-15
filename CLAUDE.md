# Claude Code Project Memory

## Project Overview
Boid Wars is a multiplayer browser-based twin-stick bullet-hell space shooter featuring massive swarms of AI-controlled boids. This is a real-time multiplayer game optimized for performance.

## Before Starting Development

**ALWAYS read the coding standards first:**
- Read `/Users/nicholasmullen/Code/gauntlet/boid_wars/docs/development/CODING_STANDARDS.md` before writing any code
- Follow the core principles: Performance First, Type Safety, Clarity Over Cleverness, Consistent Style
- Use the performance patterns and memory management guidelines
- Remember: we're optimizing for 10,000+ entities at 60 FPS

## Key Architecture Decisions
1. **Server**: Rust + Bevy ECS + Lightyear 0.21 (entity-based networking)
2. **Client**: TypeScript + Pixi.js + thin WASM bridge
3. **Networking**: WebTransport (server) ↔ WebSocket (WASM client)
4. **Deployment**: Fly.io for global edge deployment

## Development Commands
```bash
make check      # Run all formatting, linting, and tests
make dev        # Run both server and client with hot reload
make build      # Build all components for production
./scripts/build-wasm.sh  # Build WASM module
```

## Performance Targets
- 60 FPS client rendering
- <150ms network latency tolerance
- 10,000+ entities without degradation
- Optimize for 90th percentile, not average

## Memory Management Priorities
- Keep components small and POD-like
- Pre-allocate collections where size is known
- Use fixed-size arrays for bounded data
- Pool entities at system level
- Start simple, profile, then optimize

## Code Quality Standards
- **TypeScript**: No `any` types, explicit return types, strict boolean expressions
- **Rust**: Use `cargo fmt` and `cargo clippy`, prefer iteration over recursion
- **Testing**: Test behavior not implementation, performance tests for critical paths
- **Commits**: Use conventional commits format

## Git Commit Guidelines
- Do NOT add Claude as a co-contributor in commit messages
- Focus on technical changes and their impact
- Use clear, descriptive commit messages without AI attribution
