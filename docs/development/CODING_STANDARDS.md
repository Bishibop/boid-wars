# Boid Wars Coding Standards

High-level style guide for consistent, performant code across the project.

## Core Principles

1. **Performance First**: This is a real-time multiplayer game. Profile before optimizing, but always consider performance implications.
2. **Type Safety**: Strong typing in both Rust and TypeScript. No shortcuts.
3. **Clarity Over Cleverness**: Readable code > clever code. Future you will thank present you.
4. **Consistent Style**: Use automated formatters. Don't waste time on style debates.

## Rust Guidelines

### General Style
- Use `cargo fmt` and `cargo clippy` before every commit
- Prefer iteration over recursion
- Avoid allocations in hot paths (game loop, networking)
- Use `Result<T, E>` for recoverable errors, `panic!` only for bugs

### Bevy Patterns
- **Components**: Data only, derive common traits
- **Systems**: Verb-first naming (`move_players`, `spawn_boids`)
- **Resources**: Shared state across systems
- **Events**: For one-off communications

### Memory Management
- Keep components small and POD-like (Plain Old Data)
- Pre-allocate collections where size is known
- Use fixed-size arrays for bounded data (e.g., `[f32; 3]` not `Vec<f32>`)
- Pool entities at the system level, not individual allocations
- Use `Arc` only for assets and long-lived configuration
- Start simple, profile, then optimize - Bevy's ECS is already efficient

## TypeScript Guidelines

### General Style
- ESLint and Prettier are law - configure once, follow always
- No `any` types. Ever. Use `unknown` if needed
- Explicit return types on all functions
- `const` by default, `let` when needed, never `var`

### Performance Patterns
- Pool objects (sprites, particles, UI elements)
- Batch rendering operations
- Avoid creating functions/objects in hot loops
- Use `requestAnimationFrame` for all animations

### Code Organization
- One class/interface per file
- Group by feature, not by type
- Explicit exports (no `export *`)

## Shared Patterns

### Networking
- Keep messages small - every byte counts
- Separate reliable vs unreliable data
- Client sends inputs, server sends state
- Never trust the client

### Testing
- Test behavior, not implementation
- Performance tests for critical paths
- Integration tests for network protocols

### Documentation
- Document the "why", not the "what"
- Keep docs next to code
- Update docs with code changes

## Git Workflow

### Commits
Use conventional commits:
- `feat:` New features
- `fix:` Bug fixes  
- `perf:` Performance improvements
- `refactor:` Code restructuring
- `docs:` Documentation only
- `test:` Test changes
- `chore:` Build/tool changes

### Pull Requests
- One feature per PR
- Tests must pass
- Include performance impact for gameplay changes

## Performance Targets

We optimize for the 90th percentile, not the average:
- 60 FPS client rendering
- <150ms network latency tolerance  
- 10,000+ entities without degradation

## Quick Checklist

Before committing:
```bash
make check  # Runs all formatting, linting, and tests
```

## Remember

We're building a game that's fun to play AND fun to develop. Keep it simple, keep it fast, keep it maintainable.