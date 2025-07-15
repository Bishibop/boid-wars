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

## Iteration 1: Prove the Core

**Goal**: Validate that shooting at swarms is fun

### Features
- 2-4 players
- 100-500 boids
- Basic "hunt nearest player" AI
- Simple projectile combat
- Health and death for players/boids
- Shrinking zone (basic battle royale)

### Technical Goals
- Basic spatial partitioning
- Simple collision detection
- Performance baseline metrics

### Success Criteria
- 60 FPS with 500 boids
- Players report "this is fun"
- No major netcode issues

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

🎯 **We are here**: Starting Iteration 0

Next steps:
1. Complete tech stack validation
2. Deploy minimal server
3. Connect with one player and one boid
4. Validate entire pipeline works