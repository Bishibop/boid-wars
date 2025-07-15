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