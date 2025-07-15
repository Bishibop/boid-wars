# Lightyear 0.21 API Documentation

## Overview

Lightyear 0.21 has introduced significant API changes from previous versions. This document outlines the correct API structure based on research and the errors encountered during implementation.

## Key API Changes

### 1. Plugin Structure

**Old API (pre-0.21):**
```rust
lightyear::server::ServerPlugin::new(config)
```

**New API (0.21):**
```rust
use lightyear::prelude::server::*;
ServerPlugins::new(config)  // Note: ServerPlugins (plural)
```

### 2. Configuration Structure

The configuration now requires a more complex structure:

```rust
use std::net::SocketAddr;
use std::time::Duration;

let config = ServerConfig {
    shared: SharedConfig {
        server_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
        tick: TickConfig {
            tick_duration: Duration::from_secs_f64(1.0 / 30.0),
        },
        mode: Mode::Separate,
    },
    net: NetConfig::WebTransport {
        server_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
        certificate: "path/to/cert.pem".to_string(),
        private_key: "path/to/key.pem".to_string(),
    },
    ping: PingConfig::default(),
};
```

### 3. Connection Events

**Old API:**
- `ConnectEvent`
- `ClientId` (simple type)

**New API:**
- `Connected` event
- `Disconnected` event
- `ClientId` with `.raw()` method to get the underlying ID

Example:
```rust
fn handle_connections(mut events: EventReader<Connected>) {
    for event in events.read() {
        let client_id = event.client_id;
        let raw_id = client_id.raw() as u32;
    }
}
```

### 4. Replication Component

**Old API:**
```rust
Replicated  // Simple marker component
```

**New API:**
```rust
Replicate {
    target: ReplicationTarget::All,
    ..default()
}
```

### 5. Protocol Registration

The protocol registration API has changed significantly. The exact new API is still being researched, but it appears to involve:
- No more derive macros for `Channel` and `Message`
- Different approach to registering components and messages
- Possibly using a plugin-based approach

## WebTransport Configuration

For WebTransport support, you need:

1. SSL certificates (can be self-signed for development)
2. Configure the `NetConfig::WebTransport` variant
3. Ensure certificates are accessible at runtime

Example:
```rust
net: NetConfig::WebTransport {
    server_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
    certificate: std::env::var("GAME_SERVER_CERT")
        .unwrap_or_else(|_| "deploy/localhost+2.pem".to_string()),
    private_key: std::env::var("GAME_SERVER_KEY")
        .unwrap_or_else(|_| "deploy/localhost+2-key.pem".to_string()),
}
```

## Current Implementation Status

The server has been simplified to run without Lightyear networking while we research the exact 0.21 API. The basic game logic (boid movement, AI) is implemented and functional.

## Next Steps

1. Find complete Lightyear 0.21 examples from the official repository
2. Implement proper protocol registration
3. Add client-server communication
4. Implement entity replication
5. Add WebTransport support

## Resources

- [Lightyear GitHub Repository](https://github.com/cBournhonesque/lightyear)
- [Lightyear Documentation](https://docs.rs/lightyear/latest/lightyear/)
- [Lightyear Examples](https://github.com/cBournhonesque/lightyear/tree/main/examples)