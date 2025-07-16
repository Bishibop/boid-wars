# Tests Directory

This directory contains all test files and test-related scripts for Boid Wars.

## Structure

```
tests/
├── integration/        # Integration tests (reserved for future use)
├── scripts/           # Test-related shell scripts
│   ├── test_compile.sh         # Compilation test script
│   ├── test-connection.sh      # Connection testing
│   └── test-webtransport.sh    # WebTransport testing
├── test-webtransport.html      # WebTransport test page
└── bevy-client-webtransport.html  # Bevy client WebTransport test
```

## Running Tests

### Unit Tests
Unit tests are kept with the source code and run with:
```bash
make test
# or
cargo test --all
```

### Integration Tests
Server integration tests are in `server/tests/`:
```bash
cargo test --package server
```

### Test Scripts
Located in `tests/scripts/`:
- `test_compile.sh` - Verifies all components compile
- `test-connection.sh` - Tests client-server connectivity
- `test-webtransport.sh` - Tests WebTransport setup

### Manual Test Pages
- `test-webtransport.html` - Basic WebTransport connection test
- `bevy-client-webtransport.html` - Bevy WASM client WebTransport test

## Adding New Tests

- Unit tests: Add to the relevant source file
- Integration tests: Add to `server/tests/` or create new test crates
- Test utilities: Add to `tests/scripts/`
- Manual test pages: Add to this directory with descriptive names