# Troubleshooting Guide

## Common Issues

### Build Errors

#### "error: failed to select a version for the requirement"
**Cause**: Dependency version conflicts
**Solution**: 
```bash
rm Cargo.lock
cargo update
cargo build --all
```

#### "error[E0658]: `let` expressions in this position are unstable"
**Cause**: Using a crate that requires nightly Rust
**Solution**: Check if you're using the correct versions specified in `Cargo.toml`

#### WASM build fails with "wasm32-unknown-unknown target not found"
**Cause**: Missing WASM target
**Solution**: 
```bash
rustup target add wasm32-unknown-unknown
```

### Runtime Errors

#### "ERR_CERT_AUTHORITY_INVALID" in browser
**Cause**: Self-signed certificate not trusted
**Solution**: 
1. Make sure you ran `mkcert -install`
2. Restart your browser
3. Try accessing https://localhost:3000 directly and accept the certificate

#### "WebTransport connection failed"
**Cause**: Server not running or certificate issues
**Solution**:
1. Check server is running: `ps aux | grep boid-wars-server`
2. Check logs: `RUST_LOG=trace ./scripts/run-server.sh`
3. Verify certificates exist in `deploy/` directory

#### Client shows "WASM initialization failed"
**Cause**: WASM module not built or path incorrect
**Solution**:
```bash
./scripts/build-wasm.sh
# Check output exists
ls client/src/wasm/
```

### Performance Issues

#### First build takes forever
**Cause**: Building all Bevy dependencies
**Solution**: This is normal. Subsequent builds will be much faster due to caching.

#### High CPU usage when idle
**Cause**: Bevy running at uncapped framerate
**Solution**: Will be addressed when we add frame limiting to the server

### Development Environment

#### "cargo: command not found"
**Cause**: Rust not in PATH
**Solution**: 
```bash
source ~/.cargo/env
# Or add to your shell profile
```

#### Changes not reflecting in browser
**Cause**: Browser caching or Vite HMR issues
**Solution**:
1. Hard refresh: Cmd+Shift+R (Mac) / Ctrl+Shift+R (Windows/Linux)
2. Restart Vite dev server
3. Clear browser cache

## Getting Help

1. Check the [development guide](development.md)
2. Review [architecture decisions](architecture-decisions.md)
3. Search existing issues on GitHub
4. Check Bevy and Lightyear documentation

## Debugging Tips

### Enable Verbose Logging
```bash
RUST_LOG=trace ./scripts/run-server.sh
```

### Check Network Tab
In Chrome DevTools:
1. Open Network tab
2. Filter by "WS" to see WebTransport connections
3. Look for connection errors

### Rust Debugging
```bash
# Run with backtrace
RUST_BACKTRACE=full cargo run

# Use bacon for better errors
bacon
```