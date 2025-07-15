# Tech Stack Integration Validation Plan

## Goal
Verify that all chosen technologies work together by building the absolute minimal implementation that exercises every part of the stack.

## What We're Testing
- Rust + Bevy can compile and run a game server
- Lightyear can establish WebTransport connections
- WASM bridge can be built from Lightyear and loaded in browser
- TypeScript client can import and use the WASM module
- Pixi.js can render entities received via the bridge
- The full data flow works: Input → Server → State Update → Client → Render

## Minimal Implementation

### 1. Server (Rust + Bevy + Lightyear)
```rust
// Just enough to:
- Accept WebTransport connections
- Spawn a single entity that moves in a circle
- Replicate that entity's position to clients
```

### 2. WASM Bridge (Lightyear Client)
```rust
// Just enough to:
- Connect to server via WebTransport
- Receive entity updates
- Expose positions to JavaScript via a simple API
```

### 3. Client (TypeScript + Pixi.js)
```typescript
// Just enough to:
- Load the WASM module
- Create a Pixi.js canvas
- Draw a circle at the position from WASM
- Show "Connected" or "Disconnected" status
```

## Step-by-Step Validation

### Step 1: Rust Server Compilation
- Create minimal Bevy app
- Add Lightyear with WebTransport feature
- Verify it compiles and runs
- **Pass Criteria**: Server starts and logs "Listening on port 3000"

### Step 2: WASM Bridge Build
- Create minimal Lightyear client
- Build with wasm-pack
- Verify .wasm and .js files are generated
- **Pass Criteria**: `wasm-pack build` succeeds, outputs valid modules

### Step 3: Client Bundle
- Set up Vite project
- Import WASM module
- Add Pixi.js, create empty canvas
- **Pass Criteria**: Vite builds without errors, blank canvas appears

### Step 4: Local HTTPS
- Generate self-signed certificate
- Configure server for HTTPS/HTTP3
- Test WebTransport connection from browser console
- **Pass Criteria**: `new WebTransport('https://localhost:3000')` connects

### Step 5: End-to-End Test
- Server spawns moving entity
- Client connects via WASM bridge
- Entity position updates received
- Pixi.js renders moving circle
- **Pass Criteria**: Circle moves smoothly on screen

## What This Validates

✓ **Build Process**: All tools (cargo, wasm-pack, vite) work together
✓ **Dependencies**: No version conflicts between libraries
✓ **WebTransport**: Browser can connect to Rust server
✓ **Entity Replication**: Lightyear's core feature works through WASM
✓ **Rendering Pipeline**: Data flows from server to screen

## What This Doesn't Validate
- Performance at scale
- Multiple players
- Complex game logic
- Safari compatibility

## Time Estimate
- 1 day to set up build tools and project structure
- 1 day to implement minimal versions
- 1 day to debug integration issues
- **Total: 3 days**

## Success Criteria
A browser window showing a circle moving in a predictable pattern, with "Connected to server" text, proves the entire stack is integrated correctly.