# Networking Alternatives to Lightyear Proposal

**Status**: Draft  
**Date**: 2025-01-16  
**Author**: Development Team  
**Target**: Unblock Development & Future-Proof Architecture

## Executive Summary

Given the ongoing WebTransport integration challenges with Lightyear and the architectural pivot difficulties, this proposal evaluates alternatives to Lightyear for Boid Wars' networking layer. We analyze four viable alternatives, their implementation complexity, and provide a migration path that could unblock development while maintaining the game's ambitious performance targets of 10,000+ entities at 60 FPS.

## Current Situation

### Lightyear Integration Challenges
- **WebTransport Connection Issues**: Certificate validation failures, browser incompatibility
- **WASM Bridge Failures**: `AuthorityChange` resource conflicts, borrow checker violations
- **Architectural Pivot Blocked**: Full Bevy WASM client also facing connection timeout issues
- **Development Velocity**: Near zero progress on core networking for weeks

### Requirements Recap
- **10,000+ entities** with efficient replication
- **Browser-based** multiplayer (WASM support essential)
- **Low latency** for bullet-hell gameplay (<150ms tolerance)
- **Server-authoritative** for cheat prevention
- **WebTransport preferred** (fallback to WebSocket acceptable)

---

## Alternative 1: bevy_replicon + bevy_replicon_renet2

### Overview
`bevy_replicon` is a server-authoritative networking solution with automatic world replication. Combined with `bevy_replicon_renet2`, it provides a complete WebTransport solution that might solve your local development issues.

### Architecture
```rust
// Server setup
app.add_plugins(RepliconPlugins)
   .add_plugins(RepliconRenet2ServerPlugin);

// Automatic replication with simple markers
commands.spawn((
    PlayerBundle::new(),
    Replication, // That's it!
));
```

### Key Advantages Over Current Lightyear Setup

#### 1. **Better Local Development Experience**
```rust
// renet2 - much simpler WebTransport setup
let (wt_socket, cert_hash) = {
    let (config, cert_hash) = WebTransportServerConfig::new_selfsigned(
        wildcard_addr, 
        max_clients
    ).unwrap();
    (WebTransportServer::new(config, runtime.handle().clone()).unwrap(), cert_hash)
};

// Client can use cert hash instead of PKI validation
let socket_config = WebTransportClientConfig {
    server_dest: wt_server_dest.into(),
    server_cert_hashes: Vec::from([wt_server_cert_hash]) // Bypasses PKI!
};
```

#### 2. **Built-in WebTransport → WebSocket Fallback**
- Automatic graceful degradation
- No manual transport switching needed
- Works in all browsers (WebSocket fallback for Safari)

#### 3. **Multi-Platform Server**
- Same server handles both native UDP and web WebTransport clients
- No separate configurations needed

### Implementation Plan

#### Phase 1: Basic Integration (1 week)
```rust
// Server
app.add_plugins(RepliconPlugins)
   .add_plugins(RepliconRenet2ServerPlugin);

// Client (WASM)
app.add_plugins(RepliconPlugins)
   .add_plugins(RepliconRenet2ClientPlugin);
```

#### Phase 2: Feature Parity (2-3 weeks)
- Entity spawn/despawn replication ✅ (built-in)
- Component synchronization ✅ (built-in)
- WebTransport with better cert handling ✅ (built-in)
- Interest management ❌ (needs implementation)
- Client prediction ❌ (needs implementation)

#### Phase 3: Advanced Features (2-3 weeks)
```rust
// Custom interest management
fn update_client_visibility(
    mut replicated_clients: ResMut<ReplicatedClients>,
    players: Query<(&ClientId, &Position, &ViewDistance)>,
    entities: Query<(Entity, &Position), With<Replication>>,
) {
    for (client_id, player_pos, view_dist) in players.iter() {
        let visible_entities = entities
            .iter()
            .filter(|(_, pos)| player_pos.0.distance(pos.0) < view_dist.0)
            .map(|(entity, _)| entity);
            
        replicated_clients.set_visibility(client_id, visible_entities);
    }
}
```

### WebTransport Local Development Comparison

| Issue | Lightyear | bevy_replicon_renet2 |
|-------|-----------|----------------------|
| Certificate Generation | Manual (mkcert) | ✅ Built-in self-signed |
| Certificate Hash Verification | ❌ Complex setup | ✅ Built-in support |
| Local Development | ❌ Certificate issues | ✅ Better tooling |
| Automatic Fallback | ❌ Manual | ✅ WebTransport → WebSocket |
| Working Examples | ❌ Limited | ✅ Cross-platform examples |

### Important Limitations

**WebTransport constraints apply to both solutions:**
- ❌ 14-day certificate validity (WebTransport protocol requirement)
- ❌ Browser security restrictions (Chrome/Edge only)
- ❌ Still need certificate rotation for long-term development

**renet2 improves developer experience but doesn't eliminate fundamental WebTransport limitations.**

### Pros & Cons

**Pros:**
- ✅ Mature and actively maintained
- ✅ Excellent test coverage
- ✅ Better WebTransport local development experience
- ✅ Built-in certificate hash verification
- ✅ Automatic transport fallback
- ✅ Simpler API than Lightyear
- ✅ Multi-platform server support

**Cons:**
- ❌ No client prediction out-of-box
- ❌ Manual interpolation implementation
- ❌ Less feature-complete than Lightyear
- ❌ Still faces WebTransport certificate constraints
- ❌ Chrome/Edge only for WebTransport

### Effort Estimate: 4-6 weeks to feature parity

---

## Alternative 2: Custom WebTransport Solution

### Overview
Build a minimal, game-specific networking layer using `wtransport` directly, optimized for Boid Wars' specific needs.

### Architecture
```rust
// Minimal entity replication protocol
#[derive(Serialize, Deserialize)]
enum ServerMessage {
    EntitySpawn { id: u32, components: ComponentData },
    EntityUpdate { id: u32, components: ComponentData },
    EntityDespawn { id: u32 },
}

#[derive(Serialize, Deserialize)]
enum ClientMessage {
    Input(PlayerInput),
    Ping,
}
```

### Implementation Plan

#### Phase 1: Transport Layer (1 week)
```rust
// Server implementation
pub struct GameServer {
    endpoint: wtransport::Endpoint,
    clients: HashMap<ClientId, Connection>,
}

impl GameServer {
    async fn accept_connections(&mut self) {
        while let Some(conn) = self.endpoint.accept().await {
            let client_id = self.next_client_id();
            self.clients.insert(client_id, conn);
            self.spawn_client_task(client_id).await;
        }
    }
}
```

#### Phase 2: Entity Replication (2-3 weeks)
```rust
// Simplified replication system
fn replicate_entities(
    mut server: ResMut<GameServer>,
    changed: Query<(Entity, &Position, &Health), Changed<Position>>,
) {
    let updates: Vec<_> = changed
        .iter()
        .map(|(entity, pos, health)| EntityUpdate {
            id: entity.to_bits(),
            position: *pos,
            health: *health,
        })
        .collect();
        
    if !updates.is_empty() {
        let message = ServerMessage::BatchUpdate(updates);
        server.broadcast_unreliable(&message);
    }
}
```

#### Phase 3: Optimization (2-3 weeks)
- Delta compression
- Interest management
- Variable update rates
- Connection quality adaptation

### Pros & Cons

**Pros:**
- ✅ Full control over implementation
- ✅ Minimal overhead (no abstraction layers)
- ✅ Optimized for specific use case
- ✅ Direct WebTransport access
- ✅ Easier debugging

**Cons:**
- ❌ Significant development effort
- ❌ No built-in features (prediction, interpolation)
- ❌ Higher maintenance burden
- ❌ Risk of subtle bugs
- ❌ No community support

### Effort Estimate: 6-8 weeks for basic functionality

---

## Alternative 3: bevy_matchbox (WebRTC)

### Overview
Use WebRTC instead of WebTransport for browser compatibility, with `bevy_matchbox` providing the integration.

### Architecture
```rust
// Simple setup
app.add_plugins(MatchboxPlugin::new("wss://matchbox.example.com"));

// P2P or client-server topology
let socket = MatchboxSocket::new_reliable("room_id");
```

### Implementation Challenges
- WebRTC adds 20-50ms latency vs WebTransport
- P2P doesn't scale well for 10,000+ entities
- Would need relay server for client-server model
- Safari support is a plus

### Verdict: Not recommended for Boid Wars scale requirements

---

## Alternative 4: Hybrid Approach

### Overview
Use `bevy_replicon` for core replication with simplified networking requirements, focusing on getting a working game first.

### Strategy
1. **Start Simple**: WebSocket-only for all development
2. **Use bevy_replicon**: Automatic replication without transport complexity
3. **Defer Advanced Features**: No prediction/interpolation initially
4. **Iterate**: Add features based on actual needs

### Implementation
```rust
// Simple WebSocket transport
pub struct SimpleWebSocketTransport {
    server: TungsteniteServer,
}

// Focus on game logic, not networking complexity
app.add_plugins((
    RepliconPlugins,
    SimpleWebSocketTransport::new("ws://localhost:5000"),
));

// That's it - start building the game!
```

---

## Recommendation: Phased Migration Strategy

### Phase 1: Unblock Development (1 week)
1. **Keep Lightyear** but switch to WebSocket-only (already implemented)
2. **Complete core game mechanics** with working transport
3. **Validate gameplay** with current setup
4. **Establish performance baseline**

### Phase 2: Evaluate bevy_replicon_renet2 (2 weeks)
1. **Prototype bevy_replicon_renet2** integration
2. **Test WebTransport certificate handling** - does it really solve local dev issues?
3. **Benchmark performance** with target entity counts
4. **Evaluate migration complexity** and missing features

### Phase 3: Production Network Layer (4-6 weeks)
Based on Phase 2 results, either:
- **Option A**: Migrate to bevy_replicon_renet2 (if WebTransport works better)
- **Option B**: Stay with Lightyear + WebSocket (if bevy_replicon doesn't offer enough benefit)
- **Option C**: Deploy to production with proper certificates for WebTransport testing

## Risk Analysis

### Staying with Lightyear
- **Risk**: Continued blocking on WebTransport issues
- **Mitigation**: Use WebSocket for all development
- **Long-term**: May need to contribute fixes upstream

### Migrating to bevy_replicon
- **Risk**: Missing features need implementation
- **Mitigation**: Start simple, add features incrementally
- **Long-term**: More control, simpler architecture

### Custom Solution
- **Risk**: Significant effort, potential bugs
- **Mitigation**: Extensive testing, simple initial design
- **Long-term**: Maximum performance and control

## Performance Comparison

| Solution | Setup Time | 10k Entity Performance | Maintenance Burden | Risk Level |
|----------|------------|------------------------|-------------------|------------|
| Lightyear (current) | Blocked | Excellent (if working) | Low | High (blocked) |
| bevy_replicon | 1 week | Good (needs optimization) | Medium | Low |
| Custom WebTransport | 6-8 weeks | Excellent (optimized) | High | Medium |
| Hybrid Approach | 2 days | Good | Low | Very Low |

## Conclusion

The current Lightyear integration challenges have significantly impacted development velocity. However, the WebSocket fallback is now working, allowing development to continue. The question is whether bevy_replicon_renet2 offers enough advantages to justify a migration.

### Key Findings

**bevy_replicon_renet2 advantages:**
- ✅ Better WebTransport local development experience (built-in certificate handling)
- ✅ Automatic transport fallback (WebTransport → WebSocket)
- ✅ Simpler API for basic replication
- ✅ Multi-platform server support

**bevy_replicon_renet2 limitations:**
- ❌ Still faces fundamental WebTransport certificate constraints
- ❌ Missing client prediction and interpolation (crucial for bullet-hell gameplay)
- ❌ 4-6 weeks migration effort
- ❌ Less mature ecosystem than Lightyear

### Immediate Recommendation
1. **Continue with Lightyear + WebSocket** for core game development
2. **Focus on gameplay mechanics** rather than transport layer
3. **Establish performance baseline** with current setup
4. **Prototype bevy_replicon_renet2** as a 2-week evaluation

### Long-term Recommendation
**Stay with Lightyear** unless bevy_replicon_renet2 proves to dramatically improve the WebTransport development experience. The migration effort (4-6 weeks) plus re-implementing prediction/interpolation may not be worth it when:
- Current WebSocket setup is working
- Lightyear has more complete features for bullet-hell gameplay
- Performance targets can likely be achieved with either solution

### The Real Priority
The key insight remains: **working gameplay beats perfect transport**. Since WebSocket is now functional, focus on:
1. Implementing core game mechanics
2. Validating the 10,000+ entity performance targets
3. Building the actual game

Transport optimization can wait until the game is fun to play. Players care about smooth, responsive gameplay—not whether it's delivered via WebSocket or WebTransport.