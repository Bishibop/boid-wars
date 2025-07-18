# Fly.io Deployment Plan for Boid Wars

## Overview

This document outlines the complete deployment strategy for Boid Wars, a multiplayer browser-based twin-stick bullet-hell space shooter, on Fly.io. The deployment must support 10,000+ entities at 60 FPS with <150ms latency tolerance.

## Architecture

- **Server**: Rust + Bevy ECS + Lightyear networking (WebSocket/WebTransport)
- **Client**: Bevy WASM (served as static files)
- **Target Performance**: 10,000+ entities at 60 FPS, <150ms latency
- **Deployment**: Fly.io global edge deployment

## Phase 1: Core Infrastructure (Required)

### 1. Dockerfile Creation

Create an optimized multi-stage Dockerfile:

```dockerfile
# Build stage
FROM rust:1.75-alpine AS builder
WORKDIR /app

# Install build dependencies
RUN apk add --no-cache clang lld musl-dev git

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/
COPY shared/Cargo.toml ./shared/

# Create dummy main.rs files for dependency caching
RUN mkdir -p server/src shared/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "fn main() {}" > shared/src/lib.rs

# Build dependencies (cached layer)
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/app/target/ \
    cargo build --release --bin boid-wars-server

# Copy actual source code
COPY server/src ./server/src
COPY shared/src ./shared/src

# Build application
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/app/target/ \
    cargo build --release --bin boid-wars-server && \
    cp ./target/release/boid-wars-server /bin/server

# WASM build stage
FROM node:18-alpine AS wasm-builder
WORKDIR /app

# Install wasm-pack
RUN npm install -g wasm-pack

# Copy client source
COPY bevy-client ./bevy-client
COPY shared ./shared

# Build WASM client
RUN cd bevy-client && wasm-pack build --target web --out-dir pkg

# Runtime stage
FROM alpine:3.18 AS runtime
RUN apk --no-cache add ca-certificates
WORKDIR /app

# Create non-root user
RUN addgroup -g 1001 -S appuser && \
    adduser -S -u 1001 -G appuser appuser

# Copy server binary
COPY --from=builder /bin/server /app/server

# Copy WASM client assets
COPY --from=wasm-builder /app/bevy-client/pkg /app/static/
COPY assets /app/static/assets/

RUN chown -R appuser:appuser /app
USER appuser

EXPOSE 8080
CMD ["./server"]
```

**Key Features:**
- Multi-stage build for optimal image size
- Dependency caching for faster builds
- Non-root user for security
- Static linking for performance

### 2. fly.toml Configuration

Create production-ready Fly.io configuration:

```toml
app = "boid-wars"
primary_region = "ord"  # Choose closest to your primary users

[build]
  dockerfile = "Dockerfile"

[env]
  RUST_LOG = "info"
  GAME_SERVER_PORT = "8080"
  GAME_SERVER_HOST = "0.0.0.0"
  ASSET_PATH = "/app/static"
  MAX_PLAYERS = "100"
  TARGET_FPS = "60"
  MAX_ENTITIES = "10000"

[[services]]
  protocol = "tcp"
  internal_port = 8080
  processes = ["app"]

  [services.concurrency]
    type = "connections"
    hard_limit = 100      # Adjust based on your game's player capacity
    soft_limit = 80

  [[services.ports]]
    port = 80
    handlers = ["http"]

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

  # Health check for game server
  [[services.http_checks]]
    interval = "10s"
    timeout = "5s"
    grace_period = "10s"
    method = "GET"
    path = "/health"
    protocol = "http"
    tls_skip_verify = false

  # TCP check for raw socket health
  [[services.tcp_checks]]
    interval = "15s"
    timeout = "2s"

# Auto-scaling configuration
[services.auto_stop_machines]
  enabled = true
  min_machines_running = 1

[services.auto_start_machines]
  enabled = true

# Resource configuration
[[vm]]
  size = "shared-cpu-4x"  # For high-performance games
  memory = "2gb"
```

### 3. Server Configuration Updates

Update server to bind to Fly.io requirements:

**Current**: `127.0.0.1:5001`
**Target**: `0.0.0.0:8080`

**Required Changes:**
- Update `shared/src/config.rs` server address configuration
- Add health check endpoint (`/health`)
- Configure environment variables for production
- Enable structured logging (JSON format)

## Phase 2: Asset Strategy (Important)

### 4. Static Asset Serving

**Recommended Strategy**: Serve WASM client from same container

**Benefits:**
- Reduces latency by serving assets from the same edge location
- Simplifies deployment and reduces infrastructure complexity
- Ensures consistency between server and client versions

**Implementation:**
- Bundle client assets in Docker image
- Configure server to serve static files from `/app/static/`
- Include WASM build in Dockerfile

### 5. Build Process Integration

**WASM Build Integration:**
- Include WASM build in Dockerfile
- Optimize WASM bundle size with `wasm-opt`
- Asset versioning for cache invalidation

## Phase 3: Production Readiness (Critical)

### 6. Environment & Secrets Management

**Environment Variables:**
```bash
# Game Configuration
BOID_WARS_GAME_WIDTH=1200.0
BOID_WARS_GAME_HEIGHT=900.0
BOID_WARS_PLAYER_SPEED=200.0
BOID_WARS_BOID_SPEED=150.0
BOID_WARS_DEFAULT_HEALTH=100.0
BOID_WARS_SPAWN_X=600.0
BOID_WARS_SPAWN_Y=450.0

# Server Configuration
BOID_WARS_SERVER_ADDR=0.0.0.0:8080
BOID_WARS_PROTOCOL_ID=12345
RUST_LOG=info
```

**Secrets Management:**
```bash
# Set sensitive configuration
fly secrets set BOID_WARS_DEV_KEY="secure-key"
fly secrets set JWT_SECRET="jwt-secret-key"
fly secrets set ADMIN_TOKEN="admin-secret-token"
```

### 7. Performance Optimization

**Rust Release Profile:**
```toml
# In server/Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
```

**Memory Management:**
- Pre-allocate collections where size is known
- Use fixed-size arrays for bounded data
- Pool entities at system level
- Optimize for 10,000+ entities without degradation

## Phase 4: Deployment & Monitoring (Essential)

### 8. CI/CD Pipeline

Create GitHub Actions workflow:

```yaml
# .github/workflows/deploy.yml
name: Deploy to Fly.io

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt --check

  deploy:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: superfly/flyctl-actions/setup-flyctl@master
      - run: flyctl deploy --remote-only
        env:
          FLY_API_TOKEN: ${{ secrets.FLY_API_TOKEN }}
```

### 9. Multi-Region Setup

**Primary + Secondary Regions:**
```bash
# Scale to multiple regions
fly scale set min=1 max=3 --region ord  # Primary (US Central)
fly scale set min=1 max=2 --region lhr  # Europe (London)
fly scale set min=1 max=2 --region nrt  # Asia (Tokyo)
```

**Regional Configuration:**
- Primary region: US Central (ord)
- Secondary: Europe (lhr), Asia (nrt)
- Global load balancing with anycast

### 10. Monitoring & Observability

**Health Check Endpoint:**
```rust
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339(),
        "players": get_player_count(),
        "entities": get_entity_count(),
        "memory_usage": get_memory_usage()
    }))
}
```

**Monitoring Metrics:**
- Player count
- Entity count
- Memory usage
- Connection latency
- Frame rate performance

## Key Technical Decisions

1. **Monolithic Deployment**: Server + WASM client in one container (simplifies deployment)
2. **Auto-scaling**: 1-5 instances based on connection load
3. **Resource Allocation**: 2GB RAM, 4x CPU for performance targets
4. **Networking**: WebSocket primary, WebTransport future enhancement
5. **Logging**: Structured JSON logs for production debugging

## Deployment Commands

```bash
# Initial setup
fly auth login
fly launch --no-deploy  # Creates fly.toml
fly secrets set BOID_WARS_DEV_KEY="secure-key"

# Deploy
fly deploy

# Scale for production
fly scale set min=2 max=5
fly scale set --region ord min=1 max=3
fly scale set --region lhr min=1 max=2

# Monitor
fly status
fly logs
fly metrics
```

## Expected Outcomes

This deployment strategy ensures:
- **Performance**: 10,000+ entities at 60 FPS
- **Latency**: <150ms globally with multi-region deployment
- **Scalability**: Auto-scaling based on player load
- **Reliability**: Health checks and automated rollbacks
- **Security**: Non-root containers and secrets management

## Implementation Timeline

1. **Week 1**: Core infrastructure (Dockerfile, fly.toml, server config)
2. **Week 2**: Asset strategy and build process
3. **Week 3**: Production readiness and performance optimization
4. **Week 4**: Deployment pipeline and monitoring

This plan provides a production-ready deployment for Boid Wars with optimal performance, security, and global reach suitable for real-time multiplayer gaming.