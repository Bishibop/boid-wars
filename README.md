# Boid Wars

A multiplayer twin-stick bullet-hell space shooter featuring massive swarms of AI-controlled enemies.

## Tech Stack
- **Server**: Rust + Bevy + Lightyear (WebTransport)
- **Client**: TypeScript + Pixi.js
- **Bridge**: WASM (Lightyear client)

## Project Structure
```
boid-wars/
├── shared/           # Shared Rust types
├── server/           # Game server (Rust)
├── lightyear-wasm/   # WASM networking bridge
├── client/           # Browser client (TypeScript)
├── deploy/           # Deployment configurations
└── scripts/          # Development scripts
```

## Development Setup
See [Tech Stack Integration Validation](tech_stack_integration_validation.md) for setup steps.

## Current Status
🚧 Tech stack validation phase - building minimal integration test