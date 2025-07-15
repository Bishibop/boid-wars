# Boid Wars - System Architecture

## Overview

Boid Wars is a multiplayer twin-stick bullet-hell space shooter featuring massive swarms of AI-controlled enemies (boids) in a battle royale format. This document outlines the complete system architecture and technical decisions.

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
- **Networking**: Lightyear 0.21 (WebTransport/QUIC)
- **Physics**: Rapier 2D
- **Spatial Indexing**: R-tree (rstar crate)

### Frontend
- **Language**: TypeScript
- **Rendering**: Pixi.js v8 (WebGL)
- **Networking**: Lightyear 0.21 WASM client (thin bridge)
- **Build Tool**: Vite
- **Audio**: Howler.js

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

### 3. TypeScript + Pixi.js Client

**Decision**: Build the browser client in TypeScript with Pixi.js, using a thin WASM bridge for networking.

**Rationale**:
- **Bundle Size**: ~500KB vs 5-10MB for full Bevy WASM
- **Performance**: Pixi.js optimized for 2D sprite rendering, handles 1000+ entities easily
- **Developer Experience**: Fast iteration, excellent debugging tools
- **WASM Bridge**: Get Lightyear 0.21's entity-based networking benefits without full WASM overhead

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
- Physics simulation (Rapier)
- Player input processing
- Hit detection (projectiles vs players/boids)
- Zone shrinking logic (battle royale mode)
- Temporary alliance management
- Game mode logic (battle royale, single player, co-op)
- Entity replication via Lightyear 0.21
```

### 2. WASM Networking Bridge
```
Responsibilities:
- Lightyear 0.21 client protocol handling (entity-based networking)
- Entity replication/interpolation
- Input buffering and sending
- Binary protocol parsing
- JavaScript event emission
```

### 3. Browser Client (TypeScript)
```
Responsibilities:
- Rendering via Pixi.js
- Input capture (twin-stick controls)
- Sound playback
- UI/HUD rendering
- Client-side interpolation
- Particle effects
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

### Client-Side
- **Instanced Rendering**: Single draw call for all boids
- **Level of Detail**: Distant boids rendered as simple dots
- **Object Pooling**: Reuse bullet/particle objects
- **Viewport Culling**: Don't render off-screen entities

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

## Security Considerations

- Server validates all player inputs
- Rate limiting on client messages
- No client-side game logic
- HTTPS/WSS for all connections
- Input sanitization for chat/usernames

## Development Workflow

### Repository Structure
```
boid-wars/
├── shared/           # Rust shared types
├── server/           # Game server
├── lightyear-wasm/   # WASM bridge
├── client/           # TypeScript client
└── deploy/           # Deployment configs
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

## Conclusion

This architecture balances performance, developer experience, and operational simplicity. The combination of Rust/Bevy for compute-intensive server work and TypeScript/Pixi.js for responsive client rendering provides the best of both worlds. Fly.io's global infrastructure ensures low latency without operational overhead.

The system is designed to start simple (8-16 players, 1k boids) and scale up (64 players, 100k boids) as optimization work continues.