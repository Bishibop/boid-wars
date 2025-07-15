# Recommended Development Tools for Boid Wars

## Essential Tools

### 1. **mkcert** - Local HTTPS Certificates
WebTransport requires HTTPS with valid certificates. mkcert makes this painless for local dev.
```bash
brew install mkcert
mkcert -install
mkcert localhost 127.0.0.1 ::1
```

### 2. **bacon** - Better Rust Error Output
Alternative to cargo-watch with better error formatting and terminal UI.
```bash
cargo install bacon
# Use: bacon instead of cargo watch
```

### 3. **cargo-machete** - Find Unused Dependencies
Helps keep Cargo.toml clean as we experiment with libraries.
```bash
cargo install cargo-machete
# Use: cargo machete
```

## Performance & Debugging Tools

### 4. **tokio-console** - Async Runtime Debugging
Since Lightyear uses Tokio, this helps debug async issues.
```bash
cargo install --locked tokio-console
# Requires adding console-subscriber to your app
```

### 5. **wasm-bindgen-cli** - WASM Debugging
Additional tools for WASM development beyond wasm-pack.
```bash
cargo install wasm-bindgen-cli
```

### 6. **Chrome DevTools** - Built-in Browser Tools
- Network tab for WebTransport inspection
- Performance profiler for client-side bottlenecks
- WASM debugging support

## Code Quality Tools

### 7. **cargo-fmt** and **cargo-clippy**
Already included in rust-toolchain.toml, but worth setting up pre-commit hooks.

### 8. **prettier** - TypeScript/JavaScript Formatting
```bash
npm install -D prettier
# Add .prettierrc
```

## Monitoring Tools

### 9. **htop/btop** - System Resource Monitoring
Watch CPU/memory usage during development.
```bash
brew install btop  # Better than htop
```

### 10. **websocat** - WebSocket/WebTransport CLI Testing
Test connections without writing client code.
```bash
brew install websocat
# Use: websocat wss://localhost:3000
```

## Optional but Helpful

### 11. **cargo-expand** - Macro Debugging
See what Bevy's macros expand to.
```bash
cargo install cargo-expand
```

### 12. **trunk** - WASM Web App Bundler
Alternative to our manual wasm-pack setup, but less flexible.
```bash
cargo install trunk
```

### 13. **just** - Command Runner
Alternative to npm scripts for complex commands.
```bash
brew install just
```

## Browser Extensions

### 14. **React Developer Tools**
Even though we're not using React, it has good performance profiling.

### 15. **WASM Debugging Extension**
Chrome has experimental WASM debugging support - enable in DevTools settings.

## Recommended Setup Commands

```bash
# Install the most useful ones
brew install mkcert btop websocat
cargo install bacon cargo-machete
mkcert -install && mkcert localhost 127.0.0.1 ::1

# Optional: Set up git hooks
echo '#!/bin/sh
cargo fmt --check
cargo clippy -- -D warnings' > .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```