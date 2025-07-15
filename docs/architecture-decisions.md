# Architecture Decision Records

## ADR-001: Use Lightyear 0.17 instead of 0.21
**Date**: 2025-01-10
**Status**: Accepted

### Context
Lightyear 0.21 requires nightly Rust due to use of unstable "let chains" feature. This adds complexity and potential instability to our development process.

### Decision
Use Lightyear 0.17, which works with stable Rust.

### Consequences
- ✅ Stable Rust toolchain
- ✅ Faster development setup
- ✅ More reliable builds
- ❌ Missing some newer features from 0.21
- ❌ May need to migrate later if 0.21 features become essential

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