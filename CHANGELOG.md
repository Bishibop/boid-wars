# Changelog

All notable changes to Boid Wars will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- **Physics System**: Full Rapier 2D integration with realistic movement and collisions
- **Player Movement**: Thrust-based physics movement with rotation and momentum
- **Combat System**: Projectile shooting with physics-based trajectories
- **Object Pooling**: Bounded projectile pool with generation tracking for performance
- **Coordinate Systems**: Dual coordinate system (Transform for physics, Position for networking)
- **Position Sync**: Automatic bidirectional sync between physics and network layers
- **System Ordering**: Explicit PhysicsSet for deterministic system execution
- **Performance Benchmarks**: Entity scaling and physics update benchmarks
- **Integration Tests**: Comprehensive physics/network synchronization tests
- Comprehensive WebTransport development guide documenting certificate challenges and solutions
- WebSocket fallback for local development to avoid certificate issues
- Bevy WASM client implementation as the primary architecture
- Performance diagnostics and monitoring in Bevy client
- Unified Rust codebase for both client and server

### Changed
- **BREAKING**: Migrated from TypeScript/Pixi.js client to full Bevy WASM client
- Updated architecture to use single technology stack (Rust/Bevy)
- Switched local development to WebSocket to avoid WebTransport certificate issues
- Reorganized documentation structure for clarity
- All physics constants moved to centralized configuration
- Projectile ownership made optional (no dummy entities)

### Fixed
- Entity lifetime bugs with generation-based pooling
- System ordering race conditions with explicit dependencies
- Memory leaks in projectile spawning/despawning
- Position drift between physics and network layers
- WASM borrow checker violations in Lightyear integration
- AuthorityChange resource initialization issues
- WebTransport certificate validation blocking local development
- Build times reduced from 5+ minutes to 30 seconds with incremental compilation

### Removed
- TypeScript client implementation (moved to legacy)
- Thin WASM bridge approach (moved to legacy)
- Dependency on Node.js for client development
- Magic numbers throughout physics code
- Unnecessary debug logging

## [0.2.0] - 2025-01-16

### Architecture Migration
This release represents a major architectural shift from a TypeScript/Pixi.js client with WASM bridge to a full Bevy WASM client. This change was necessitated by technical blockers in the WASM bridge approach with Lightyear 0.20.

### Technical Details
- **Client Architecture**: TypeScript + Pixi.js → Rust + Bevy WASM
- **Bundle Size**: Increased from ~500KB target to ~3.5MB (acceptable for game)
- **Performance Target**: Maintained 10,000+ entities at 60 FPS
- **Networking**: WebTransport (production) with WebSocket fallback (development)

## [0.1.0] - 2024-12-15

### Initial Release
- Basic multiplayer infrastructure with Lightyear 0.21
- TypeScript client with Pixi.js rendering
- Thin WASM bridge for networking
- WebTransport protocol implementation
- Basic boid simulation (proof of concept)
- Fly.io deployment configuration

### Known Issues
- WebTransport requires complex certificate setup for local development
- WASM bridge has borrow checker issues with Lightyear
- Limited to ~1000 entities due to bridge overhead