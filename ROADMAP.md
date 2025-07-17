# Boid Wars Development Roadmap

This document outlines our iterative development approach. Each iteration builds on the previous one, allowing us to validate assumptions and adjust course as needed.

## Iteration 0: Tech Stack Validation ✅

**Goal**: Prove the entire technical pipeline works end-to-end

### Features
- 1-2 players maximum
- 1 boid (yes, just one!)
- Basic player movement (WASD)
- Basic shooting (click to shoot)
- Boid follows nearest player
- Server authoritative positions

### Technical Validation
- Rust/Bevy server runs
- WASM bridge builds and loads
- WebTransport connection established
- Entity replication functional
- Input → Server → State → Client flow working
- Pixi.js renders sprites
- Deploy to Fly.io (single region)

### Success Criteria
- Can connect to deployed server
- Can see other player moving
- Can see boid following players
- No critical errors in 5-minute play session

---

## Iteration 1: Prove the Core 🟡 (In Progress)

**Goal**: Validate that shooting at swarms is fun

### Features
- ❌ 2-4 players (single player only - no multiplayer yet)
- 🟡 100-500 boids (only 8 peaceful boids tested)
- ✅ Basic "hunt nearest player" AI (peaceful flocking implemented)
- ✅ Simple projectile combat (physics-based shooting)
- ✅ Health and death for players/boids
- ❌ Shrinking zone (basic battle royale) - not implemented

### Technical Goals
- ✅ Basic spatial partitioning (Rapier 2D broad phase)
- ✅ Simple collision detection (Rapier 2D integration)
- ✅ Performance baseline metrics (benchmarks added)

### Physics Implementation ✅
- ✅ Rapier 2D physics engine integrated
- ✅ Thrust-based player movement with rotation
- ✅ Projectile physics with proper trajectories
- ✅ Collision detection for all entities
- ✅ Bounded object pooling for projectiles
- ✅ Dual coordinate system (Transform/Position)
- ✅ Deterministic system ordering

### Success Criteria
- ❓ 60 FPS with 500 boids (not tested at scale)
- ✅ Physics feels responsive and fun
- ❓ No major netcode issues (multiplayer not working)

---

## Iteration 2: Scale the Swarms

**Goal**: Find our performance limits

### Features
- Same player count (2-4)
- Push boid count: 1k → 2k → 5k → 10k
- Add swarming behavior (flocking)
- Basic performance optimizations

### Technical Goals
- Implement spatial grid
- Optimize boid AI updates
- Add performance monitoring
- Interest management (basic)

### Success Criteria
- Identify maximum viable boid count
- Maintain playable performance
- Understand optimization needs

---

## Iteration 3: Scale the Players

**Goal**: Validate multiplayer at target scale

### Features
- 8-16 players
- Boid count from Iteration 2
- Basic matchmaking
- Proper interest management
- Deploy to production

### Technical Goals
- Optimize network traffic
- Implement proper culling
- Add interpolation/extrapolation
- Basic deployment automation

### Success Criteria
- Stable with 8+ players
- <150ms latency tolerance
- Successful play sessions

---

## Iteration 4: Enhance Gameplay

**Goal**: Add depth and polish

### Features
- Multiple boid behaviors (flee, patrol, swarm)
- Temporary alliance system
- Single-player mode
- UI/UX polish
- Sound effects

### Technical Goals
- Behavior tree for AI
- Alliance state management
- Game mode abstraction
- Asset pipeline

### Success Criteria
- Players engage with alliances
- Varied gameplay emerges
- "One more game" feeling

---

## Future Iterations

### Iteration 5: Platform Expansion
- WebRTC fallback for Safari
- Mobile touch controls
- Multiple deployment regions

### Iteration 6: Content & Features
- Co-op mode
- Special boid types
- Power-ups
- Progression system

### Iteration 7: Scale to 64
- Push player count
- Advanced optimization
- Multiple server regions
- Tournament mode

---

## Development Principles

1. **Ship each iteration** - Each phase should be playable
2. **Measure everything** - Data drives decisions
3. **Fun first** - If it's not fun at small scale, scale won't help
4. **Technical honesty** - If we hit limits, we adapt

## Current Status

🎯 **We are here**: Iteration 1 - Physics Complete, Multiplayer Pending

### Completed ✅
- **Iteration 0**: Tech Stack Validation ✅
  - Basic server game loop
  - Full Bevy WASM client implementation
  - WebSocket networking for local dev
  - Basic entity spawning
  
- **Physics Implementation** ✅
  - Full physics system with Rapier 2D
  - Thrust-based player movement
  - Projectile combat system
  - Collision detection for all entities
  - Peaceful boid flocking behavior (8 boids)
  - Object pooling for performance
  - Transform/Position synchronization
  - Performance benchmarks

### Not Yet Implemented ❌
- **Multiplayer**: No client-server connection working
- **Entity Replication**: Lightyear networking not functional
- **Scale Testing**: Only tested with 8 boids, not 500+
- **WebTransport**: Using WebSocket fallback only
- **Deployment**: Not deployed to Fly.io

### Physics Features Implemented ✅
- **Movement**: Thrust-based with rotation, momentum, and damping
- **Combat**: Physics-based projectiles with proper trajectories
- **Collisions**: Player-player, player-boid, projectile-all
- **Optimization**: Bounded pooling with generation tracking
- **Sync**: Dual coordinate system (Transform/Position)
- **Testing**: Integration tests for physics/network sync

### Recent Improvements ✅
- **Physics System**: Complete Rapier 2D integration
- **Performance**: Bounded object pools prevent memory fragmentation
- **Stability**: Explicit system ordering eliminates race conditions
- **Configuration**: All physics constants centralized
- **Code Quality**: Removed magic numbers and unnecessary logging

### Next Steps (Complete Iteration 1):
1. **Fix Multiplayer**: Get Lightyear 0.20 networking functional
2. **Entity Replication**: Sync positions between client/server
3. **Test at Scale**: Spawn 100-500 boids as planned
4. **Connect Bevy Client**: Get WASM client talking to server
5. **Deploy to Fly.io**: Test in production environment

### Future (Iteration 2):
1. **Scale boid count**: Test with 1k → 2k → 5k → 10k boids
2. **Optimize flocking**: Implement efficient spatial queries
3. **Zone shrinking**: Implement battle royale mechanics