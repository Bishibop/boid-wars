# Lightyear 0.20 WebTransport Connection Issue

## Problem Summary

WebTransport local development is blocked by certificate validation issues and browser incompatibilities.

## Current Status (July 16, 2025)

- **Chrome**: Rejects self-signed certificates even with `serverCertificateHashes` 
  - Error: `CERTIFICATE_VERIFY_FAILED`
  - Chrome flags don't help (`--allow-insecure-localhost`, WebTransport Developer Mode)
  - Root cause: Chrome requires certificates from "known roots" for QUIC/WebTransport
  
- **Firefox**: Accepts certificates but has protocol version mismatch
  - Error: `Code::crypto(2b)` = `QUIC_CRYPTO_VERSION_NOT_SUPPORTED`
  - Known issue: wtransport GitHub issue #241
  - Affects all Firefox versions (tested 132.0.2 and latest)

## Investigation Progress

### 1. Initial Issue: Wrong Address
- **Problem**: Client was connecting to `0.0.0.0:0` instead of server address
- **Fix**: Updated client to use `Authentication::Manual` with correct server address
- **Status**: ✅ Fixed - client now attempts correct address

### 2. Authentication Mismatch
- **Problem**: Client and server using different authentication configurations
- **Fix**: Aligned authentication between client and server
- **Status**: ✅ Fixed - both use matching auth

### 3. Transport Protocol Mismatch 
- **Problem**: Both client and server configured for `NetConfig::Netcode` (UDP) instead of WebTransport
- **Status**: ✅ Fixed - Now using WebTransport configuration
  
### 4. Current Configuration Attempt

```rust
// Client side
let transport = ClientTransport::WebTransportClient {
    client_addr,
    server_addr,
    #[cfg(target_family = "wasm")]
    certificate_digest,
};

let io = IoConfig::from_transport(transport);

// Server side  
let transport = ServerTransport::WebTransportServer {
    server_addr,
    certificate: cert,
    private_key: key,
};

let io = IoConfig::from_transport(transport);
```

### 4. Certificate Trust Issue (Current Issue)
- **Problem**: Browser rejects self-signed certificate with `CERTIFICATE_VERIFY_FAILED`
- **Error**: `net::ERR_QUIC_PROTOCOL_ERROR.QUIC_TLS_CERTIFICATE_UNKNOWN`
- **Root Cause**: Chrome/Edge require trusted certificates for WebTransport

### 5. Fixed Issues

1. **Import Errors**: ✅ Fixed
   - `ClientTransport` is in `lightyear::prelude::client::`
   - `ServerTransport` is in `lightyear::prelude::server::`
   - `IoConfig` and `NetConfig` from respective preludes
   
2. **Certificate Configuration**: ✅ Fixed
   - Server uses `Identity::load_pemfiles()` from wtransport
   - Client includes certificate digest in WebTransport config

## Key Findings

1. **WebTransport is the goal** - Not WebSocket, not raw UDP Netcode
2. **Certificates are required** - Already generated via `mkcert`
3. **Transport configuration** - Needs to be set in `IoConfig`, not just `NetConfig`
4. **API confusion** - Lightyear 0.20 transport configuration API differs from examples found online

## Attempted Solutions (All Failed)

### 1. Chrome Flags
- ❌ `chrome://flags/#allow-insecure-localhost` - No effect
- ❌ WebTransport Developer Mode flag - No effect
- ❌ `--ignore-certificate-errors` - No effect
- ❌ `--origin-to-force-quic-on` - No effect

### 2. Certificate Approaches
- ❌ mkcert certificates (2+ year validity) - Rejected
- ❌ OpenSSL self-signed (13-day validity) - Rejected
- ❌ Certificate digest in client config - Still rejected
- ❌ SPKI hash format - Invalid format error

### 3. Alternative Browsers
- ❌ Firefox - Protocol version mismatch (wtransport issue)
- ❌ Firefox 132.0.2 - Same protocol mismatch

### 4. Tunneling Services
- ❌ ngrok - Doesn't support HTTP/3 (QUIC), only HTTP/1.1 and HTTP/2

## Root Cause Analysis

1. **WebTransport is too new** - Tooling isn't mature for local development
2. **Chrome's strict security** - Requires certificates from "known roots" even with explicit hashes
3. **Protocol fragmentation** - Firefox and wtransport use incompatible WebTransport versions
4. **No good tunneling solution** - Services like ngrok don't support HTTP/3

## Viable Solutions

### 1. Use WebSocket for Local Development (Recommended)
```rust
// Debug builds use WebSocket (no certificates!)
#[cfg(debug_assertions)]
let transport = ClientTransport::WebSocketClient { server_addr };

// Release builds use WebTransport
#[cfg(not(debug_assertions))]
let transport = ClientTransport::WebTransportClient { ... };
```
**Status**: In progress - need to add `websocket` feature to Cargo.toml

### 2. Deploy to Real Server
- Get a VPS with a domain
- Use Let's Encrypt certificates
- WebTransport works perfectly in production

### 3. Wait for Ecosystem Maturity
- Chrome to improve developer experience
- wtransport to fix Firefox compatibility
- Better local development tools for WebTransport

## Related Files

- Client config: `/bevy-client/src/lib.rs`
- Server config: `/server/src/main.rs`
- Certificates: `~/.boid-wars/certs/localhost.pem`
- Certificate digest script: `/scripts/get-cert-digest.sh`

## Resources

- Lightyear examples show WebTransport needs certificate digest
- WebTransport requires HTTPS/TLS unlike WebSocket
- Certificate must be valid for < 14 days for browser acceptance