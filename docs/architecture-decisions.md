# Architecture Decision Records

## ADR-001: Use Lightyear 0.21 with Bevy 0.16
**Date**: 2025-01-10  
**Status**: Superseded (Updated 2025-01-15)

### Context
Originally chose Lightyear 0.17 due to nightly Rust concerns, but encountered WASM compatibility issues due to bevy version conflicts.

### Decision
Upgrade to Lightyear 0.21 with Bevy 0.16 to resolve WASM build failures.

### Consequences
- ✅ WASM builds work correctly
- ✅ Consistent bevy versions across dependencies
- ✅ Access to Lightyear 0.21 improvements (entity-based networking)
- ❌ Major API changes require new implementation approach
- ❌ Some features need reimplementation (Resource→Entity model)

---

## ADR-002: WebTransport Only (No WebRTC)
**Date**: 2025-01-10
**Status**: Accepted

### Context
WebTransport is simpler than WebRTC but lacks Safari support (79% browser coverage).

### Decision
Start with WebTransport only for the validation phase.

### Consequences
- ✅ Simpler implementation
- ✅ No STUN/TURN servers needed
- ✅ Better performance potential
- ❌ No Safari/iOS support initially
- ❌ May need to add WebRTC later

---

## ADR-003: Thin WASM Bridge Pattern
**Date**: 2025-01-10
**Status**: Accepted

### Context
Choice between full Bevy WASM client (5-10MB) vs TypeScript + thin WASM bridge (500KB).

### Decision
Use TypeScript + Pixi.js for client with minimal WASM bridge for networking only.

### Consequences
- ✅ Small bundle size
- ✅ Fast client development
- ✅ Native web development experience
- ❌ Some code duplication between client/server
- ❌ Extra complexity in WASM bridge

---

## ADR-004: Monorepo Structure
**Date**: 2025-01-10
**Status**: Accepted

### Context
Need to manage multiple related projects (server, client, WASM, shared types).

### Decision
Use Cargo workspace for Rust code and monorepo structure.

### Consequences
- ✅ Shared dependencies and versioning
- ✅ Atomic commits across projects
- ✅ Easier refactoring
- ❌ Larger initial clone size
- ❌ More complex CI/CD

---

## ADR-005: Dual Coordinate System with Rapier2D Physics
**Date**: 2025-01-17
**Status**: Accepted

### Context
Game requires physics simulation for players, projectiles, and obstacles while maintaining network protocol compatibility with top-left origin coordinates.

### Decision
Implement dual coordinate system:
- **Network Layer**: Uses Position component with top-left origin (0,0 at top-left)
- **Physics Layer**: Uses Transform component with center origin (standard Bevy/Rapier)
- **Sync Layer**: PositionSyncPlugin handles bidirectional conversion

### Implementation
```rust
// Network coordinate (top-left origin)
Position(Vec2::new(x, y))

// Physics coordinate (center origin)
Transform::from_xyz(x - game_width/2, y - game_height/2, 0.0)
```

### Consequences
- ✅ Clean separation of concerns
- ✅ Network protocol remains unchanged
- ✅ Physics works with standard conventions
- ✅ Easy to reason about each system independently
- ❌ Additional sync overhead (negligible)
- ❌ Must remember which coordinate system when debugging

---

## ADR-006: Bounded Object Pooling for Projectiles
**Date**: 2025-01-17
**Status**: Accepted

### Context
Projectile spawning/despawning causes performance issues and memory fragmentation. Need efficient reuse of entities.

### Decision
Implement generation-based bounded object pool:
- Fixed maximum pool size with pre-spawning
- Generation tracking prevents use-after-free bugs
- Pooled entities positioned off-screen when inactive
- Fallback to regular spawning when pool exhausted

### Key Features
```rust
pub struct PooledEntity {
    entity: Entity,
    generation: u32,
}

pub struct BoundedPool<T> {
    available: VecDeque<PooledEntity>,
    active: HashMap<Entity, u32>,
    generations: HashMap<Entity, u32>,
    max_size: usize,
}
```

### Consequences
- ✅ Eliminates allocation overhead during gameplay
- ✅ Prevents memory leaks and use-after-free bugs
- ✅ Predictable memory usage
- ✅ Generation tracking catches stale references
- ❌ Initial memory overhead from pre-spawning
- ❌ Complexity in managing pooled state

---

## ADR-007: Explicit System Ordering with PhysicsSet
**Date**: 2025-01-17
**Status**: Accepted

### Context
Physics systems had race conditions due to undefined execution order. Bevy's default parallel execution caused non-deterministic behavior.

### Decision
Define explicit system sets with clear dependencies:
```rust
enum PhysicsSet {
    Input,          // Process player input
    AI,             // AI decision making
    Movement,       // Apply forces/velocity
    Combat,         // Shooting/damage
    Collision,      // Handle collisions
    ResourceManagement, // Pool/despawn
    NetworkSync,    // Sync to network layer
}
```

### Consequences
- ✅ Deterministic system execution
- ✅ Eliminates race conditions
- ✅ Clear dependencies between systems
- ✅ Easier to debug physics issues
- ❌ Less parallelism (acceptable tradeoff)
- ❌ Must maintain ordering constraints

---

## ADR-008: Configuration-Driven Physics Constants
**Date**: 2025-01-17
**Status**: Accepted

### Context
Magic numbers scattered throughout physics code made tuning difficult and hurt maintainability.

### Decision
Centralize all physics constants in configuration resources:
- `PhysicsConfig`: Movement, combat, collision parameters
- `MonitoringConfig`: Performance monitoring settings
- Resources injected at plugin initialization

### Consequences
- ✅ Single source of truth for tuning
- ✅ Easy A/B testing of parameters
- ✅ No magic numbers in code
- ✅ Can expose to designers/config files later
- ❌ Extra indirection when reading code
- ❌ Must ensure config loaded before use