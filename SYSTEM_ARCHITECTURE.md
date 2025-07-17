# Boid Wars - System Architecture

## Overview

Boid Wars is a multiplayer twin-stick bullet-hell space shooter featuring massive swarms of AI-controlled enemies (boids) in a battle royale format. This document outlines the complete system architecture and technical decisions.

**Current Architecture**: Full Bevy WASM client with Rust server (as of January 2025)

## Core Requirements

- **Players**: 
  - 8-16 players (medium-term target)
  - 64 players (long-term ambition)
  - Single player and 2-player co-op modes
- **Enemies**: 10,000+ AI-controlled boids (enemy swarms that attack players)
- **Game Modes**:
  - **Battle Royale** (primary): Last player standing with shrinking play area
  - **Single Player**: Survival against boid swarms
  - **Co-op** (2 players): Team survival mode
- **Platform**: Browser-based (WebGL/Canvas)
- **Latency**: Ultra-low latency required for bullet-hell gameplay

## Gameplay Mechanics

- **PvPvE**: Players fight both each other and AI boid swarms
- **Twin-Stick Controls**: Independent movement and aiming
- **Temporary Alliances**: Players can form temporary teams with friendly fire toggle
- **Projectile Combat**: Players shoot projectiles at boids and other players
- **Boid Behaviors**: AI enemies with varied behaviors (hunt, swarm, patrol, flee)

## Technology Stack

### Backend
- **Language**: Rust
- **Game Framework**: Bevy ECS 0.16
- **Networking**: Lightyear 0.20 (WebTransport/WebSocket)
- **Physics**: Rapier 2D (integrated)
- **Spatial Indexing**: R-tree (rstar crate)
- **Entity Pooling**: Custom bounded pool with generation tracking

### Frontend
- **Language**: Rust
- **Game Framework**: Bevy ECS 0.16 (WASM)
- **Rendering**: Bevy 2D sprites
- **Networking**: Lightyear 0.20 ClientPlugins
- **Build Tool**: wasm-pack + wasm-opt
- **Audio**: Bevy Audio

### Infrastructure
- **Hosting**: Fly.io (global edge deployment)
- **CDN**: Cloudflare (static assets)
- **Monitoring**: Fly.io metrics + Sentry

## Architecture Decisions & Rationale

### 1. Rust + Bevy for Game Server

**Decision**: Use Rust with Bevy ECS for the authoritative game server.

**Rationale**:
- **Performance**: Rust provides zero-cost abstractions and no garbage collection, critical for simulating 10k+ entities at 30Hz
- **Bevy ECS**: Entity Component System architecture is perfect for managing thousands of boids efficiently
- **Concurrency**: Rust's fearless concurrency enables parallel boid processing
- **Memory Safety**: Prevents common game server vulnerabilities

### 2. WebTransport over WebRTC

**Decision**: Use WebTransport as the primary protocol, with WebRTC fallback for Safari.

**Rationale**:
- **Latency**: QUIC-based transport provides UDP-like performance with reliability options
- **Simplicity**: No complex peer negotiation like WebRTC
- **Browser Support**: Growing support (Chrome, Firefox, Edge)
- **Fallback**: WebRTC DataChannels for Safari users

### 3. Bevy WASM Client

**Decision**: Build the browser client as a full Bevy WASM application.

**Rationale**:
- **Technical Feasibility**: Eliminates WASM bridge issues with Lightyear integration
- **Unified Architecture**: Single technology stack (Rust/Bevy) for both client and server
- **Native ECS Performance**: Handle 10k+ entities without JavaScript boundary overhead
- **Direct Integration**: Use Lightyear ClientPlugins as designed
- **2025 WASM Maturity**: Improved tooling and smaller bundles (~3.5MB optimized)

### 4. Server-Authoritative Architecture

**Decision**: All game logic runs on the server; clients only send input and render state.

**Rationale**:
- **Cheating Prevention**: Critical for competitive battle royale
- **Consistency**: Ensures all players see the same boid behaviors
- **Simplicity**: No complex client prediction for boids
- **Scalability**: Server controls computational load

### 5. Fly.io for Global Deployment

**Decision**: Use Fly.io for hosting game servers globally.

**Rationale**:
- **Edge Deployment**: Servers in 30+ regions worldwide
- **Simplicity**: Single `fly deploy` command
- **Cost Efficiency**: Pay per minute, scales to zero
- **WebTransport Support**: Built-in SSL/TLS handling

## System Components

### 1. Game Server (Rust)
```
Responsibilities:
- Authoritative game state
- Boid AI simulation (10k+ enemies with flocking behaviors)
- Physics simulation (Rapier2D with dual coordinate system)
- Player input processing (thrust-based movement)
- Hit detection (projectiles vs players/boids)
- Zone shrinking logic (battle royale mode) - planned
- Temporary alliance management - planned
- Game mode logic (battle royale, single player, co-op) - planned
- Entity replication via Lightyear 0.20 - not yet functional
- Projectile pooling for performance
- Transform/Position synchronization
```

### 2. Browser Client (Bevy WASM)
```
Responsibilities:
- Rendering via Bevy 2D sprite systems
- Input capture via Bevy input systems
- Sound playback via Bevy Audio
- UI/HUD rendering via Bevy UI
- Entity replication via Lightyear ClientPlugins
- Client-side prediction and interpolation
- Particle effects via Bevy systems
```

### 4. Matchmaking Service
```
Responsibilities:
- Player queuing
- Region selection
- Game server spawning (Fly Machines API)
- Match lifecycle management
```

## Data Flow

```
1. Player selects game mode (battle royale/single/co-op)
2. Matchmaker assigns to server (or spawns new instance)
3. Player establishes WebTransport connection to game server
4. Game loop:
   a. Client sends input (60Hz) - movement, aiming, shooting, alliance requests
   b. Server processes:
      - Player physics and projectiles (60Hz)
      - Boid AI decisions (30Hz)
      - Collision detection (60Hz)
      - Alliance state changes
   c. Server sends state deltas (30Hz) - positions, health, alliances
   d. Client interpolates and renders (60+ FPS)
```

## Performance Optimizations

### Server-Side
- **Spatial Partitioning**: R-tree for O(log n) nearest-neighbor queries
- **Interest Management**: Only replicate entities within player viewport
- **Delta Compression**: Send only changed component values
- **Fixed Timestep**: Deterministic simulation for consistency
- **Object Pooling**: Pre-allocated projectile pool with generation tracking
- **Dual Coordinate Systems**: Transform (physics) and Position (network) with automatic sync
- **Explicit System Ordering**: PhysicsSet enum for deterministic execution order

### Client-Side
- **Bevy Sprite Batching**: Automatic instanced rendering for boid swarms
- **ECS-based LOD**: Component-driven detail levels based on distance
- **Rust Memory Pooling**: Vec allocation strategies for entities
- **Bevy Frustum Culling**: Built-in viewport culling systems

## Scaling Strategy

### Initial Deployment
- Start with single server/region
- Focus on core gameplay with 8-16 players
- 10,000+ boids per match
- Iterate and optimize based on performance metrics

### Vertical Scaling
- Start with 2 CPU cores, 4GB RAM
- Scale up as needed when hitting performance limits
- Monitor: CPU usage, memory, network bandwidth, frame timing

### Future Horizontal Scaling
- Add regions based on player geography
- Edge deployment for ultra-low latency
- Matchmaker to distribute players to nearest server
- No inter-server communication needed

### Optimization Approach
1. Implement features with reasonable performance
2. Profile to identify bottlenecks
3. Optimize hot paths
4. Scale hardware if needed
5. Repeat until target metrics achieved

## Physics Implementation

### Coordinate System Architecture

The game uses a dual coordinate system to maintain compatibility between physics and networking:

- **Network Layer (Position)**: Top-left origin (0,0), matching web conventions
- **Physics Layer (Transform)**: Center origin, standard Bevy/Rapier conventions
- **Synchronization**: Automatic bidirectional sync via PositionSyncPlugin

### Entity Management

#### Projectile Pooling
- **Bounded Pool**: Fixed-size pool with generation tracking
- **Pre-spawning**: 100 initial projectiles, max 500
- **Generation Tracking**: Prevents use-after-free bugs
- **Fallback**: Dynamic spawning when pool exhausted

#### System Execution Order
```rust
PhysicsSet {
    Input -> AI -> Movement -> Combat -> Collision -> ResourceManagement -> NetworkSync
}
```

### Physics Configuration

All physics constants are centralized in configuration resources:
- `PhysicsConfig`: Movement speeds, forces, collision sizes
- `MonitoringConfig`: Performance tracking intervals
- No magic numbers in gameplay code

## Security Considerations

- Server validates all player inputs
- Rate limiting on client messages
- No client-side game logic
- HTTPS/WSS for all connections
- Input sanitization for chat/usernames
- Generation-based entity validation in pools

## Development Workflow

### Repository Structure
```
boid-wars/
├── shared/           # Rust shared types & protocol
├── server/           # Game server
│   ├── src/
│   │   ├── main.rs          # Server entry & connection handling
│   │   ├── physics.rs       # Physics systems & combat
│   │   ├── position_sync.rs # Transform/Position synchronization
│   │   ├── pool.rs          # Bounded object pooling
│   │   ├── config.rs        # Physics configuration
│   │   └── despawn_utils.rs # Safe entity cleanup
│   ├── benches/      # Performance benchmarks
│   └── tests/        # Integration tests
├── bevy-client/      # Bevy WASM client
└── deploy/           # Deployment configs

# Legacy (for reference):
├── client/           # Original TypeScript client
└── lightyear-wasm/   # Attempted WASM bridge
```

### Local Development
- Docker compose for local server
- Hot reload for client (Vite)
- Cargo watch for server

### Deployment Pipeline
1. GitHub Actions build all components
2. Docker images pushed to registry
3. `fly deploy` updates servers
4. CDN cache invalidation for client

## Future Considerations

### Safari Support
- Implement WebRTC DataChannel fallback
- Additional complexity but expands player base

### Mobile Support
- Touch controls for twin-stick input
- Performance profiling on mobile browsers
- Possible native app wrapper

### Monetization Ready
- Server-side validation for purchases
- Cosmetic items don't affect gameplay
- Account system integration points

## Architecture Evolution

### Original Design (2024)

The initial architecture used TypeScript + Pixi.js for the client with a thin WASM bridge for Lightyear networking. This design prioritized:
- Small bundle size (~500KB)
- Fast development iteration
- Mature TypeScript tooling

### Architectural Pivot (January 2025)

During implementation of the Lightyear 0.20 migration, critical technical blockers emerged with the thin WASM bridge approach:

1. **AuthorityChange Resource Conflicts**: Internal Lightyear types (`pub(crate)`) causing WASM-specific initialization failures
2. **WASM Borrow Checker Violations**: JavaScript/Rust boundary creating unsafe aliasing during `app.update()`
3. **Complex State Synchronization**: Managing entity state between Rust bridge and TypeScript proving error-prone
4. **Limited Debugging Capability**: Unable to inspect Lightyear's internal state from JavaScript side

### Revised Decision: Full Bevy WASM Client

**Decision**: Migrate from TypeScript + Pixi.js client to full Bevy WASM client.

**Rationale for Change**:
- **Unblock Development**: Eliminates all current WASM bridge technical issues
- **Simplified Architecture**: Single technology stack (Rust) vs dual-language complexity
- **Native Lightyear Integration**: Use ClientPlugins as designed, no custom bridges
- **Better Entity Performance**: Handle 10k+ entities entirely in ECS without JS boundary overhead
- **2025 WASM Improvements**: Better tooling, optimization, smaller bundles than original assessment

**Acknowledged Tradeoffs**:
- **Bundle Size**: Increases from ~500KB target to ~3.5MB (7x larger, but acceptable for game)
- **Development Experience**: Rust compile times vs TypeScript hot reload
- **Mobile Considerations**: Need validation that WASM performance acceptable on mobile browsers

### Updated Technology Stack

#### Frontend (Revised)
- **Language**: Rust (changed from TypeScript)
- **Rendering**: Bevy 2D (changed from Pixi.js v8)
- **Networking**: Lightyear 0.20 ClientPlugins (direct, no bridge)
- **Build Tool**: wasm-pack + wasm-opt (changed from Vite)
- **Audio**: Bevy Audio (changed from Howler.js)

### Updated System Components

#### 2. Browser Client (Bevy WASM) - Revised
```
Responsibilities:
- Rendering via Bevy 2D sprite systems
- Input capture via Bevy input systems
- Sound playback via Bevy Audio
- UI/HUD rendering via Bevy UI
- Entity replication via Lightyear ClientPlugins
- Client-side prediction and interpolation
```

**Removed Component**: WASM Networking Bridge (no longer needed)

### Updated Repository Structure
```
boid-wars/
├── shared/           # Rust shared types
├── server/           # Game server
├── bevy-client/      # Bevy WASM client (NEW)
└── deploy/           # Deployment configs
```

### Updated Performance Optimizations

#### Client-Side (Revised)
- **Bevy Instanced Rendering**: Use Bevy's sprite batching for boid swarms
- **ECS-based LOD**: Distance-based component systems for detail levels
- **Bevy Viewport Culling**: Built-in frustum culling systems
- **WASM Memory Management**: Rust Vec pooling instead of JS object pooling

### Updated Development Workflow

#### Local Development (Revised)
- **Single Language**: Rust for both server and client
- **Unified Debugging**: Bevy dev tools for networking and rendering
- **Shared Systems**: Common components and logic between server/client
- **Build Pipeline**: `wasm-pack build` + `wasm-opt` for client optimization

### Performance Validation Requirements

Before full migration, the following targets must be met:

1. **Bundle Size**: <5MB optimized WASM bundle (vs original 500KB target)
2. **Entity Performance**: 10k+ entities rendering at 60fps (same as original target)
3. **Mobile Compatibility**: Playable performance on mobile browsers
4. **Memory Usage**: Comparable to TypeScript client memory footprint
5. **Load Time**: Acceptable cold start performance for web game

### Migration Strategy

1. **Parallel Implementation**: Build Bevy client alongside existing TypeScript client
2. **Performance Validation**: Prove all targets before removing old client
3. **Rollback Plan**: Maintain ability to return to WASM bridge debugging if needed
4. **Documentation**: Update all docs to reflect new architecture decisions

### Original Architecture Preservation

The original TypeScript + Pixi.js decision (lines 72-80) was sound based on 2024 context:
- Bundle size concerns were valid for general web applications
- Bevy WASM tooling was less mature
- Development experience favored TypeScript for rapid iteration

This revision acknowledges those tradeoffs while adapting to:
- Current implementation blockers requiring architectural change
- 2025 improvements in WASM tooling and browser performance
- Game-specific requirements (10k+ entities) favoring unified ECS approach

### Performance Validation

The Bevy WASM client must meet these targets:
- **Bundle Size**: <5MB optimized
- **Entity Performance**: 10k+ entities at 60fps
- **Mobile Compatibility**: Playable on mobile browsers
- **Memory Usage**: Comparable to TypeScript implementation
- **Load Time**: Acceptable for web game standards

## Development Workflow

### Local Development
- **WebSocket**: Used for certificate-free local development
- **WebTransport**: Used in production with real certificates
- **Unified Debugging**: Bevy dev tools for both client and server
- **Hot Reload**: Trunk serve for client, cargo watch for server

### Build Commands
```bash
make dev        # Start both client and server
make check      # Run formatting, linting, tests
make build      # Production builds
```

## Conclusion

The evolution to a full Bevy WASM client represents a pragmatic response to technical challenges while maintaining ambitious performance goals. By unifying the technology stack, we've simplified the architecture and eliminated complex JavaScript/Rust boundary issues.

The system targets remain unchanged: 8-16 players initially, 64 players long-term, with 10,000+ AI-controlled boids. The Bevy ECS architecture on both client and server provides the performance foundation needed for this scale.

**Key Success Metrics**:
- Eliminate technical blockers
- Meet performance targets (10k entities @ 60fps)
- Enable rapid development iteration
- Maintain sub-150ms latency gameplay