# Client Architecture Evolution

This document chronicles the evolution of Boid Wars' client architecture, documenting the technical journey and lessons learned.

## Timeline Overview

1. **Phase 1**: TypeScript + Pixi.js Client (Original Design)
2. **Phase 2**: Thin WASM Bridge Attempt 
3. **Phase 3**: Full Bevy WASM Client (Current)

## Phase 1: TypeScript + Pixi.js (December 2024)

### Initial Design Goals
- **Small bundle size**: Target ~500KB for fast loading
- **Familiar tech stack**: TypeScript for rapid development
- **Proven rendering**: Pixi.js v8 for optimized 2D WebGL
- **Clean separation**: Game logic on server, rendering on client

### Architecture
```
Browser → TypeScript Client → WebSocket → Rust Server
           ↓
         Pixi.js
           ↓
         WebGL Canvas
```

### Why This Made Sense
- TypeScript ecosystem maturity
- Excellent debugging tools
- Fast hot reload development
- Pixi.js proven for 2D games
- Small bundle size for web delivery

### What Worked
- Rapid prototyping
- Smooth 60 FPS rendering
- Great developer experience
- Easy to onboard contributors

## Phase 2: Thin WASM Bridge (January 2025)

### Motivation for Change
- Needed Lightyear's entity-based networking
- Wanted to share protocol code between client/server
- Binary protocol efficiency over JSON

### Architecture Attempt
```
Browser → TypeScript → WASM Bridge → Lightyear Protocol
           ↓             ↓
         Pixi.js    (thin layer)
           ↓
         WebGL
```

### Implementation Approach
The WASM bridge would:
- Handle network protocol only
- Parse binary messages
- Emit events to TypeScript
- Minimal memory footprint

### Technical Blockers Encountered

#### 1. Resource Initialization Issues
```rust
// This internal Lightyear resource wouldn't initialize in WASM
Parameter `ResMut<Events<ReceiveMessage<AuthorityChange, ClientMarker>>>` 
failed validation: Resource does not exist
```

The `AuthorityChange` type was internal (`pub(crate)`) to Lightyear, making it impossible to manually register.

#### 2. WASM Borrow Checker Violations
```
recursive use of an object detected which would lead to unsafe aliasing in rust
```

JavaScript calling into WASM while Bevy's `app.update()` was running created borrow checker violations. Attempts to fix included:
- Mutex protection (too slow)
- State caching (complex synchronization)
- Update flags (race conditions)

#### 3. Complex State Synchronization
Keeping entity state synchronized between:
- Rust ECS (source of truth)
- JavaScript (for rendering)
- Without copying entire world each frame

### Why It Failed
- Lightyear 0.20/0.21 wasn't designed for thin WASM bridges
- JavaScript/Rust boundary created fundamental ownership issues
- Entity synchronization overhead negated performance benefits
- Debugging across language boundary was painful

## Phase 3: Full Bevy WASM Client (Current)

### The Pivot Decision
After spending weeks on WASM bridge issues, we recognized:
- Fighting the framework wasn't productive
- Bevy WASM had improved significantly
- Bundle size increase (500KB → 3.5MB) acceptable for games
- Unity of technology stack had major benefits

### Current Architecture
```
Browser → Bevy WASM Client → WebTransport → Rust Server
           ↓
         Bevy Renderer
           ↓
         WebGL Canvas
```

### Benefits Realized
1. **No JavaScript/Rust boundary** - Eliminates borrow checker issues
2. **Native Lightyear integration** - Use as designed
3. **Shared systems** - Reuse logic between client/server
4. **Single language** - Rust throughout
5. **Better performance** - 10k+ entities in pure ECS

### Tradeoffs Accepted
- **Larger bundle**: 3.5MB vs 500KB target
- **Slower iteration**: Rust compilation vs TypeScript
- **Steeper learning curve**: Rust vs TypeScript for contributors
- **Limited browser APIs**: Some web features harder to access

### Current Status
- ✅ Lightyear integration working
- ✅ WebSocket for development (no certificates)
- ✅ WebTransport for production
- ✅ Meeting performance targets
- ✅ Clean architecture

## Lessons Learned

### 1. Don't Fight the Framework
Lightyear was designed for full Bevy clients. Trying to use it differently created unnecessary complexity.

### 2. Bundle Size Isn't Everything
For games, 3.5MB is acceptable if it provides better architecture and performance. Initial load time is less critical than runtime performance.

### 3. Language Boundaries Are Expensive
The JavaScript/Rust boundary introduced:
- Ownership complexities
- Performance overhead
- Debugging challenges
- State synchronization issues

### 4. WASM Has Matured
2025 WASM tooling is much better than initial assessments:
- Better optimization
- Improved debugging
- Smaller output sizes
- Better browser support

### 5. Unified Stack Benefits
Having Rust/Bevy throughout provides:
- Code sharing
- Consistent patterns
- Single mental model
- Better maintainability

## Future Considerations

### If Starting Today
We would likely start with full Bevy WASM, knowing:
- The tooling has matured
- Bundle size is acceptable for games
- Unity of stack provides major benefits
- The ecosystem has proven patterns

### Potential Future Optimizations
- Module splitting for faster initial load
- Streaming WASM compilation
- Progressive enhancement
- Hybrid rendering approaches

## Conclusion

The journey from TypeScript → WASM Bridge → Full Bevy WASM taught valuable lessons about:
- When to pivot vs persist
- Real vs perceived constraints
- Framework design intentions
- Modern web game architecture

The current full Bevy WASM approach provides the best foundation for achieving our ambitious goal of 10,000+ entities in a multiplayer browser game.