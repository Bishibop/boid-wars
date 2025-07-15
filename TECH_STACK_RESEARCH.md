# Boid Wars - Tech Stack Research & Decision History

## Project Genesis

**Initial Requirements:**
- Twin-stick bullet-hell multiplayer space shooter
- Browser-based
- Massive enemy swarms using boid simulation
- Battle royale format
- Target: 64 players, 10k+ boids (100k stretch goal)

**Core Challenge**: Latency is paramount in bullet-hell gameplay where milliseconds matter.

## Research Journey

### 1. Networking Protocol Investigation

**Started with WebRTC vs WebTransport debate:**

Initially leaned toward WebTransport for its simplicity and modern design. However, discovered Safari doesn't support it (as of 2025). This led to considering:
- WebRTC DataChannels as primary (universal browser support)
- WebTransport as future migration path
- Hybrid approach with fallback

**Deep dive findings:**
- WebTransport has ~79% browser support (missing Safari/iOS completely)
- WebRTC DataChannels have ~98% support (including Safari)
- Latency difference: ~10-20ms improvement with WebTransport (not game-breaking)
- Development complexity: WebTransport is 10x simpler than raw WebRTC
- QUIC (WebTransport's foundation) offers 0-RTT reconnection - huge for mobile

**Infrastructure considerations:**
- WebRTC requires STUN/TURN servers ($50-500/month)
- WebTransport just needs HTTPS certificate
- LiveKit abstracts WebRTC complexity but adds SFU overhead (~5-10ms)

**Decision Point**: Start with WebRTC for compatibility, architect for WebTransport migration.

### 2. Backend Language Evaluation

**Performance requirements drove language choice:**

Investigated multiple languages for 10k+ boid simulation:

**Go:**
- Pros: Excellent WebRTC library (Pion), simple concurrency, fast development
- Cons: Poor memory layout control, no SIMD, GC pauses (1-10ms)
- Performance: ~100k boids @ 60fps
- WebTransport: Main library (quic-go/webtransport-go) ceased maintenance June 2024

**Rust:**
- Pros: Zero-cost abstractions, SIMD support, no GC, fearless concurrency
- Cons: Steeper learning curve
- Performance: ~500k boids @ 60fps
- WebTransport: Active ecosystem (wtransport, webrtc-rs)

**C++:**
- Pros: Maximum performance, mature ecosystem
- Cons: Memory safety concerns, complex for networking
- Performance: ~600k boids @ 60fps
- WebTransport: libwebtransport available

**C#/.NET:**
- Pros: Good performance, familiar to game devs, Unity integration
- Cons: GC overhead, experimental WebTransport
- Performance: ~200k boids @ 60fps
- WebTransport: ASP.NET Core experimental support via MsQuic

**Key Insight**: Rust offers 5x Go performance with memory safety. Critical for competitive game.

### 3. Game Framework Deep Dive

**Server-side frameworks evaluated:**

**Initially explored general solutions:**
- Colyseus (Node.js) - Good but performance concerns
- Custom UDP servers - Too much work
- Unity/Unreal headless - Overkill and licensing issues

**WebRTC abstraction libraries investigated:**
- LiveKit - Full-featured but video conferencing focused (adds ~$0.11/hour for 100 players)
- Geckos.io - UDP-like behavior via WebRTC, game-specific
- NetplayJS - P2P with rollback netcode, serverless option
- Nakama - Full game backend with WebRTC support
- Raw Pion (Go) - Maximum control but months of development

**Then discovered Rust game ecosystems:**
- Bevy ECS - Perfect fit for entity management
- Lightyear - Purpose-built for multiplayer games, supports both WebRTC and WebTransport
- Renet - QUIC-based, lower level alternative
- Naia - Cross-platform focus but less active
- matchbox - WebRTC specifically for games

**Key insight**: Building production WebRTC with raw Pion means reimplementing 70% of LiveKit

**Revelation**: Lightyear + Bevy provides exactly what we need - entity replication, interest management, and WebTransport support.

### 4. Client Architecture Exploration

**Major decision: Native Bevy WASM vs TypeScript renderer**

**Bevy WASM approach:**
- Pros: Shared code, perfect integration
- Cons: 5-10MB bundle, slower iteration, browser integration pain

**TypeScript + WebGL renderer:**
- Pros: 500KB bundle, fast development, native web
- Cons: Duplicate some logic, need integration layer

**Breakthrough**: Thin WASM bridge pattern - use Lightyear WASM just for networking, TypeScript for everything else.

### 5. Renderer Technology Comparison

**Evaluated rendering libraries:**

**Pixi.js:**
- Mature, optimized for 2D sprites
- ParticleContainer handles 10k+ sprites
- 380KB minified
- Clean API

**Phaser 3:**
- Full game framework
- More features but heavier (950KB)
- Opinionated structure conflicts with server authority

**Three.js/Babylon.js:**
- Overkill for 2D but incredible performance
- Could handle 100k+ boids via instancing

**Native Canvas:**
- Maximum control but requires implementing everything

**Decision**: Pixi.js strikes perfect balance - powerful enough for our needs, light enough for competitive play.

### 6. Deployment Strategy Evolution

**Started with traditional thinking:**
- Kubernetes on AWS/GCP
- Complex but "standard"

**Explored managed game hosting:**
- Agones - Still requires K8s knowledge
- Hathora/Edgegap - Good but another vendor

**Discovered Fly.io:**
- Global edge deployment
- Simple `fly deploy`
- Built-in SSL for WebTransport
- Pay-per-minute pricing
- Perfect for game servers

### 7. Authority Model Analysis

**Evaluated networking architectures:**

**Server Authoritative (chosen):**
- Cheat-proof
- Consistent game state
- Required for 10k+ boid coordination
- Standard for competitive games

**Client Authoritative:**
- Instant response but massive cheat risk
- Ruled out immediately

**P2P:**
- No server costs but NAT traversal hell
- Host migration complexity

**Hybrid (client prediction):**
- Best of both worlds for player movement
- Added to roadmap but not initial scope

## Key Technical Decisions

1. **Rust + Bevy + Lightyear** for server (performance critical)
2. **TypeScript + Pixi.js** for client (developer experience)
3. **Thin WASM bridge** pattern (best of both worlds)
4. **WebRTC initially**, architect for WebTransport
5. **Fly.io deployment** (simplicity + global reach)
6. **Server authoritative** (competitive integrity)

## Networking Library Decision Matrix

**For client-server architecture (no P2P needed):**

| Library | Complexity | Safari Support | Latency | Infrastructure Cost |
|---------|------------|----------------|---------|-------------------|
| WebTransport | Simple | ❌ No | Best | Just HTTPS |
| LiveKit | Medium | ✅ Yes | +5-10ms | $0.006/GB + servers |
| Geckos.io | Medium | ✅ Yes | Good | STUN/TURN needed |
| Raw WebRTC | Complex | ✅ Yes | Good | STUN/TURN needed |
| WebSockets | Simple | ✅ Yes | +20-30ms | None |

**Final networking choice**: Start with Geckos.io or LiveKit for Safari support, prepare WebTransport migration path for when Safari catches up.

## Lessons Learned

1. **Performance math matters**: 10k entities at 30Hz = 300k updates/sec. Language choice is critical.

2. **Bundle size affects player acquisition**: 5MB vs 500KB can mean losing players before they try.

3. **Safari WebTransport gap**: Can't ignore 19.62% of users (all iOS + Safari desktop). Need fallback strategy.

4. **Managed services worth it**: Fly.io vs raw K8s saves weeks of DevOps work.

5. **Start simple, optimize later**: Pure server authority first, add prediction if needed.

## What We Almost Chose (And Why We Didn't)

- **Go + Pion**: Would work but 5x slower than Rust for boids
- **Full Bevy WASM**: Bundle size and browser integration concerns
- **Phaser 3**: Too opinionated for server-authoritative design
- **Raw Kubernetes**: Complexity overkill for indie game
- **Client-side boid simulation**: Synchronization nightmare

## Future Considerations Identified

1. WebRTC → WebTransport migration path
2. Client prediction for player movement
3. Mobile browser support
4. Native app wrappers if needed
5. Monetization hooks (server-side validation ready)

## Conclusion

The research journey led us from "just use WebSockets" to a sophisticated architecture leveraging cutting-edge tech (Rust, ECS, WebTransport) while maintaining pragmatism (TypeScript client, Fly.io deployment). The stack balances performance, developer experience, and player experience - critical for a competitive multiplayer game with massive entity counts.