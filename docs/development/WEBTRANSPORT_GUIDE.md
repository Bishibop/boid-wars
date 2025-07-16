# WebTransport Development Guide

## Overview

This guide documents our journey implementing WebTransport in Boid Wars, the challenges encountered, solutions attempted, and our final approach. WebTransport is a modern networking protocol built on HTTP/3 (QUIC) that promises low-latency, multiplexed connections ideal for real-time games. However, its strict security requirements create significant challenges for local development.

## The WebTransport Journey

### Initial Goal
We chose WebTransport for Boid Wars because:
- Low latency compared to WebSocket
- Built-in multiplexing and prioritization
- Native browser support without plugins
- Ideal for real-time multiplayer games

### The Certificate Challenge

WebTransport requires valid TLS certificates, but with stricter requirements than regular HTTPS:
- Certificates must be valid for < 14 days (or from a trusted CA)
- Browsers enforce these requirements strictly
- Self-signed certificates face significant hurdles

## What We Tried

### 1. Self-Signed Certificates with mkcert
**Approach**: Generated certificates using mkcert with 2+ year validity
```bash
mkcert -install
mkcert localhost 127.0.0.1 ::1
```
**Result**: ❌ Chrome rejected - certificates exceeded 14-day validity limit

### 2. Short-Lived Certificates
**Approach**: Generated 13-day certificates using OpenSSL
```bash
openssl req -x509 -newkey rsa:4096 -sha256 -days 13 \
  -nodes -keyout key.pem -out cert.pem \
  -subj "/CN=localhost" -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"
```
**Result**: ❌ Chrome still rejected - requires certificates from "known roots"

### 3. Certificate Hash Approach
**Approach**: Provided certificate hash directly to WebTransport API
```javascript
const transport = new WebTransport(url, {
  serverCertificateHashes: [{
    algorithm: "sha-256",
    value: hexToBytes("6c749523ac1cfc90801edd60960a700b51400adc582a61386748b6f471e65c27")
  }]
});
```
**Result**: ❌ Chrome rejected with `CERTIFICATE_VERIFY_FAILED`

### 4. Chrome Flags and Developer Mode
**Approaches tried**:
- `chrome://flags/#allow-insecure-localhost`
- WebTransport Developer Mode flag
- `--ignore-certificate-errors` command line flag
- `--origin-to-force-quic-on=localhost:5000`

**Result**: ❌ None affected WebTransport certificate validation

### 5. Alternative Browsers
**Firefox**: Accepts certificates but has protocol version mismatch
- Error: `QUIC_CRYPTO_VERSION_NOT_SUPPORTED`
- Known issue: wtransport GitHub issue #241
- Affects all Firefox versions

### 6. Tunneling Services
**ngrok**: Doesn't support HTTP/3 (QUIC)
- Only supports HTTP/1.1 and HTTP/2
- WebTransport requires HTTP/3

## What Worked

### Development: WebSocket Fallback
For local development, we use WebSocket which doesn't require certificates:

```rust
// Debug builds use WebSocket
#[cfg(debug_assertions)]
let transport = ClientTransport::WebSocketClient { 
    server_addr: SocketAddr::from(([127, 0, 0, 1], 5001))
};

// Release builds use WebTransport
#[cfg(not(debug_assertions))]
let transport = ClientTransport::WebTransportClient {
    client_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
    server_addr: SocketAddr::from(([127, 0, 0, 1], 5000)),
    #[cfg(target_family = "wasm")]
    certificate_digest: "...".to_string(),
};
```

### Production: Real Certificates
WebTransport works perfectly in production with:
- Real domain name
- Let's Encrypt certificates
- Proper DNS setup

## Current Architecture

### Server Configuration
```rust
// WebTransport for production
let (certificate, private_key) = load_certificates()?;
let transport = ServerTransport::WebTransportServer {
    server_addr: SocketAddr::from(([0, 0, 0, 0], 5000)),
    certificate,
    private_key,
};

// WebSocket for development
#[cfg(debug_assertions)]
let transport = ServerTransport::WebSocketServer {
    server_addr: SocketAddr::from(([127, 0, 0, 1], 5001)),
};
```

### Client Configuration
The client automatically selects transport based on build configuration:
- Development: WebSocket on port 5001
- Production: WebTransport on port 5000

## Lessons Learned

1. **WebTransport is cutting-edge** - The ecosystem isn't mature for local development
2. **Browser security is strict** - Chrome enforces certificate requirements with no developer escape hatch
3. **Protocol fragmentation exists** - Different browsers and libraries use incompatible versions
4. **WebSocket is a valid fallback** - Similar performance for local development without certificate hassles

## Development Workflow

### Local Development
```bash
# Start server with WebSocket
make dev

# Client connects automatically via WebSocket
# No certificates needed!
```

### Production Deployment
```bash
# Deploy to server with real domain
# Let's Encrypt handles certificates
# WebTransport works seamlessly
```

## Future Improvements

As the WebTransport ecosystem matures, we expect:
- Better browser developer tools
- Standardized local development workflows
- Improved certificate handling for development
- Better cross-browser compatibility

## Technical Details

### Certificate Generation Scripts
We maintain several certificate scripts for experimentation:
- `scripts/generate-webtransport-cert.sh` - Basic certificate generation
- `scripts/get-cert-digest.sh` - Extract certificate hash
- `scripts/generate-cert-with-san.sh` - Certificates with Subject Alternative Names

### Browser Workarounds Attempted
Multiple Chrome launch scripts were created:
- `scripts/chrome-dev.sh` - Basic development flags
- `scripts/chrome-ignore-cert-errors.sh` - Certificate bypass attempts
- `scripts/chrome-webtransport-dev.sh` - WebTransport-specific flags

None successfully bypassed WebTransport certificate validation.

## Conclusion

While WebTransport offers superior performance for production deployments, its strict security requirements make local development challenging. Our hybrid approach - WebSocket for development, WebTransport for production - provides the best developer experience while maintaining production performance benefits.

The key insight: Don't fight the security model during development. Use the right tool for each environment.