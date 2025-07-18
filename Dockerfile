# Multi-stage Dockerfile for Boid Wars
# Build stage
FROM rust:alpine AS builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache \
    clang \
    lld \
    musl-dev \
    git \
    pkgconfig \
    openssl-dev \
    alsa-lib-dev \
    eudev-dev

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/
COPY shared/Cargo.toml ./shared/
COPY bevy-client/Cargo.toml ./bevy-client/

# Create dummy source files for dependency caching
RUN mkdir -p server/src shared/src bevy-client/src server/benches && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "// dummy" > shared/src/lib.rs && \
    echo "// dummy" > bevy-client/src/lib.rs && \
    echo "// dummy benchmark" > server/benches/physics_benchmark.rs && \
    echo "// dummy benchmark" > server/benches/spatial_grid_bench.rs

# Build dependencies (this layer will be cached)
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/app/target/ \
    cargo build --release --bin boid-wars-server

# Remove dummy files and copy actual source
RUN rm -rf server/src shared/src bevy-client/src server/benches
COPY server/src ./server/src
COPY server/benches ./server/benches
COPY server/tests ./server/tests
COPY shared/src ./shared/src
COPY bevy-client/src ./bevy-client/src

# Build the actual application
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/app/target/ \
    cargo build --release --bin boid-wars-server && \
    cp ./target/release/boid-wars-server /bin/server

# WASM client build stage
FROM rust:alpine AS wasm-builder

WORKDIR /app

# Install build dependencies
RUN apk add --no-cache clang lld musl-dev git pkgconfig openssl-dev

# Add wasm32 target
RUN rustup target add wasm32-unknown-unknown

# Install wasm-pack via cargo
RUN cargo install wasm-pack

# Copy workspace structure
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/
COPY shared/Cargo.toml ./shared/
COPY bevy-client/Cargo.toml ./bevy-client/

# Copy source code
COPY bevy-client ./bevy-client
COPY shared ./shared

# Create dummy server directory to satisfy workspace
RUN mkdir -p server/src server/benches && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "// dummy benchmark" > server/benches/physics_benchmark.rs && \
    echo "// dummy benchmark" > server/benches/spatial_grid_bench.rs

# Build WASM client
RUN cd bevy-client && wasm-pack build --target web --out-dir pkg --release

# Runtime stage
FROM alpine:3.18 AS runtime

# Install runtime dependencies
RUN apk --no-cache add \
    ca-certificates \
    tzdata

WORKDIR /app

# Create non-root user
RUN addgroup -g 1001 -S appuser && \
    adduser -S -u 1001 -G appuser appuser

# Copy server binary
COPY --from=builder /bin/server /app/server

# Copy WASM client assets
COPY --from=wasm-builder /app/bevy-client/pkg /app/static/
COPY --from=wasm-builder /app/bevy-client/index.html /app/static/
COPY --from=wasm-builder /app/bevy-client/demo.html /app/static/

# Copy game assets
# TODO: Update this to "assets" when assets-worktree gets merged back
COPY assets-worktree/assets /app/static/assets/

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080


# Run the server
CMD ["./server"]