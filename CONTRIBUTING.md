# Contributing to Boid Wars

## Development Setup

1. Install prerequisites:
   ```bash
   # Rust toolchain (via rustup.rs)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install wasm-pack
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   
   # Install development tools
   cargo install cargo-watch bacon
   npm install
   ```

2. Generate local certificates:
   ```bash
   brew install mkcert
   mkcert -install
   cd deploy && mkcert localhost 127.0.0.1 ::1
   ```

3. Run development servers:
   ```bash
   npm run dev
   ```

## Code Style

- Rust: Follow standard Rust conventions, enforced by rustfmt
- TypeScript: Prettier configuration in `.prettierrc`
- Commit messages: Use conventional commits format

## Testing

Run all tests:
```bash
cargo test
cd client && npm test
```

## Pre-commit Checks

The pre-commit hook runs:
- `cargo fmt --check`
- `cargo clippy`
- Updates Cargo.lock if needed

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for system design details.