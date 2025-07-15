# Boid Wars Development Notes

Living document for ideas, optimizations, issues, and decisions during development. Updated frequently, split into separate docs as needed.

## Ideas & Features

### Gameplay Mechanics
- [ ] Boid "moods" - aggressive vs cautious based on swarm size
- [ ] Environmental hazards that affect both players and boids
- [ ] Temporary power-ups that affect boid behavior (repel, attract, confuse)
- [ ] Boid breeding - destroyed boids occasionally split into smaller ones
- [ ] "King boid" - larger, tougher boid that buffs nearby swarm

### Alliance Mechanics  
- [ ] Alliance chat channel
- [ ] Shared vision when allied
- [ ] Betrayal mechanic - bonus for breaking alliance at right moment?
- [ ] Maximum alliance size to prevent dominant teams

### Visual/Audio
- [ ] Boid swarm sounds that intensify with size
- [ ] Screen shake when large swarms approach
- [ ] Particle effects for boid deaths based on swarm density

## Optimizations

### Performance Tooling Needed
- [ ] Server-side frame time profiler (track time per system)
- [ ] Network latency measurement (client->server->client round trip)
- [ ] Bandwidth monitor (bytes/second per player)
- [ ] Entity count vs frame time graph
- [ ] Memory allocation tracker
- [ ] Client FPS counter with percentiles (not just average)
- [ ] Boid AI update time breakdown
- [ ] Chrome DevTools integration for WASM profiling
- [ ] Grafana dashboard for production metrics?

### To Investigate
- [ ] SIMD for boid position calculations
- [ ] GPU compute shaders for boid behaviors (WebGPU when available)
- [ ] Hierarchical boid updates - distant swarms update less frequently
- [ ] Predictive boid spawning based on player positions

### Memory
- [ ] Pre-allocate all boid components at start
- [ ] Use object pools for projectiles
- [ ] Investigate arena allocator for per-frame data

### Networking
- [ ] Delta compression for position updates
- [ ] Quantize positions to reduce precision (do we need f32?)
- [ ] Bundle multiple small messages into single packet
- [ ] Investigate unreliable ordered channel for positions
- [ ] Add network simulation tool (artificial latency/packet loss for testing)

## Known Issues

### Current Workarounds
- Using Lightyear 0.21 with Bevy 0.16 (upgraded from 0.17 to resolve WASM compatibility issues)
- No Safari support (WebTransport not available)
- WASM bundle size not optimized yet

### Tech Debt
- [ ] Move SSL certs to secure location (currently in deploy/)
- [ ] Add proper error handling to client connection
- [ ] Implement graceful server shutdown
- [ ] Add reconnection logic for dropped connections

## Decisions & Rationale

### Why WebTransport over WebRTC?
- **Decision**: Start with WebTransport, add WebRTC later
- **Rationale**: Simpler implementation, better performance, growing support
- **Revisit**: When we need Safari support

### Why TypeScript + Pixi instead of full Rust/WASM?
- **Decision**: Thin WASM bridge + TypeScript client
- **Rationale**: 10x smaller bundle, better debugging, faster iteration
- **Trade-off**: Some complexity in bridge layer

### Why 60Hz server tick?
- **Decision**: Match common display refresh rates
- **Rationale**: Smooth gameplay, predictable timing
- **Revisit**: If server CPU becomes bottleneck

## Metrics & Benchmarks

### Current Performance (Iteration 0)
- Server: TBD
- Client: TBD
- Network: TBD

### Targets
- 10,000 boids @ 60 Hz server tick
- <150ms latency tolerance
- <500MB client memory
- <2GB server memory

### Discoveries
- (To be filled as we profile)

## Nice to Haves

### Quick Wins for Early Development
- [ ] `/metrics` endpoint on server (even if minimal)
- [ ] Version field in protocol messages from day one
- [ ] `scripts/dev.sh` for one-command development
- [ ] Basic debug overlay in client (connection status, entity count)
- [ ] Simple feature flag system (`FEATURES` const)
- [ ] Shared crate for common types (Vector2, EntityId, etc.)
- [ ] Debug hotkeys (spawn 100 boids, teleport player, etc.)
- [ ] Performance baseline snapshot after each iteration

### Developer Experience
- [ ] Hot reload for game constants without restart
- [ ] Visual debugging for boid behaviors (show force vectors)
- [ ] Time controls for debugging (pause, slow-mo, fast-forward)
- [ ] Replay system for debugging netcode issues

## Random Thoughts

- What if boids could merge into larger "mega-boids" temporarily?
- Could we use boid swarms for environmental storytelling?
- Competitive mode where players control boid swarms vs each other?
- Boid behaviors affected by time of day in game?

---

*Last updated: Start of project*