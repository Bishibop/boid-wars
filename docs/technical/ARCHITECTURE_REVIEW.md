# Critical Architecture Review: Boid Wars

## Executive Summary

The proposed architecture combines cutting-edge technologies (Rust/Bevy, WebTransport, Pixi.js) with ambitious performance targets (10,000+ boids, 64 players). While the technology choices are individually sound, several critical risks threaten project viability.

## Strengths

1. **Backend Performance Stack**: Rust + Bevy ECS + Rapier is excellent for high-performance game servers. Bevy's parallel ECS and Rapier's benchmarks (handling 12,500+ entities) validate this choice.

2. **Client Rendering**: Pixi.js v8 with 3-4x sprite performance improvements and reactive rendering is ideal for 2D browser games.

3. **Server-Authoritative Design**: Critical for competitive integrity in a battle royale format.

4. **Interest Management**: Only replicating entities within viewport is essential for scalability.

## Critical Issues

### 1. WebTransport Infrastructure Gap
**Risk Level: CRITICAL**
- Safari lacks WebTransport support (no timeline)
- Fly.io doesn't appear to support HTTP/3/QUIC required for WebTransport
- This breaks the core networking strategy

**Impact**: You'll need WebSocket fallback immediately, not just for Safari but potentially for all connections if Fly.io doesn't support HTTP/3.

### 2. Networking Bandwidth at Scale
**Risk Level: HIGH**
- 10,000 entiti
- Even with delta compression and interest management, this is massive
- 64 players receiving partial updates still creates significant bandwidth

**Calculation**: Assuming 50 bytes per entity update, 500 visible entities per player:
- 50 bytes × 500 entities × 20Hz × 64 players = 32 MB/s per match

### 3. Server CPU Requirements
**Risk Level: HIGH**
- Physics at 60Hz for 10,000+ entities
- Boid AI simulation at 30Hz
- Network serialization at 20Hz
- Spatial indexing overhead

The architecture suggests 2-4 CPU cores, but this seems optimistic for the target scale.

### 4. Development Complexity
**Risk Level: MEDIUM**
- Three codebases: Rust server, WASM bridge, TypeScript client
- WASM bridge adds complexity without clear performance benefit
- Debugging across language boundaries is challenging

## Architectural Recommendations

### 1. Immediate: Fix Networking Strategy
- Implement WebSocket transport in Lightyear as primary, not fallback
- Consider dedicated hosting with HTTP/3 support if WebTransport is critical
- Or embrace WebSockets fully - they're proven for games like Agar.io

### 2. Reduce Initial Scope
- Start with 1,000 boids and 16 players
- Implement progressive scaling once base game works
- This allows proving the architecture before hitting limits

### 3. Simplify Client Architecture
- Consider full WASM client with Bevy instead of hybrid approach
- OR drop WASM bridge entirely and use pure WebSocket client
- Current hybrid adds complexity without clear benefits

### 4. Infrastructure Alternative
If Fly.io doesn't support HTTP/3, consider:
- Cloudflare Workers for edge compute (supports WebTransport)
- AWS GameLift for proven game server hosting
- Self-managed Kubernetes with HTTP/3 ingress

### 5. Performance Optimizations
- Implement level-of-detail for boid simulation (distant boids use simpler AI)
- Consider client-side prediction for player movement
- Batch entity updates into larger, less frequent packets

## Final Assessment

The architecture is **technically ambitious but practically risky**. The combination of:
- Unproven WebTransport support in production infrastructure
- Aggressive performance targets (10k+ entities)
- Complex multi-language architecture

Creates a high probability of hitting fundamental blockers.

**Recommendation**: Simplify to proven technologies (WebSockets) and modest initial targets (1k boids, 16 players), then scale up. The current architecture risks months of development before discovering it won't work at the target scale.

## Supporting Evidence

### Technology Research Findings

**Bevy ECS Performance:**
- Described as "Fast: Massively Parallel and Cache-Friendly. The fastest ECS according to some benchmarks"
- Compile times: 0.8-3.0 seconds with 'fast compiles' configuration
- Architecture designed for cache-friendly iteration patterns and parallel system execution

**Lightyear Networking:**
- Supports WebTransport on native and WASM builds
- Built-in prediction & rollback capabilities
- Entity replication at 10Hz with client-side interpolation
- Bandwidth management with priority-based message sending

**Rapier Physics:**
- Runs 5-8x faster than nphysics
- Performance comparable to CPU version of NVidia PhysX
- Tested with scenarios including 12,500+ entities
- Multithreaded with SIMD optimizations available

**Infrastructure Limitations:**
- WebTransport requires HTTP/3/QUIC support
- Fly.io currently supports HTTP/1.1 and HTTP/2, but HTTP/3 support not documented
- Safari still lacks WebTransport support as of 2025
- Game server CPU throttling reported on Fly.io even with 4x shared CPU allocation
