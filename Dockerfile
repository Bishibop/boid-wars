# Multi-stage Dockerfile for Boid Wars
# Build stage
FROM rust:1.88.0 AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    clang \
    lld \
    git \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    libudev-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/
COPY shared/Cargo.toml ./shared/
COPY bevy-client/Cargo.toml ./bevy-client/

# Copy actual shared source files first (they're small and change less frequently)
COPY shared/src ./shared/src

# Create dummy source files for server
RUN mkdir -p server/src bevy-client/src server/benches && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "// dummy" > bevy-client/src/lib.rs && \
    echo "// dummy benchmark" > server/benches/physics_benchmark.rs && \
    echo "// dummy benchmark" > server/benches/spatial_grid_bench.rs

# Build dependencies (this layer will be cached)
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/app/target/ \
    cargo build --release --bin boid-wars-server --locked

# Remove dummy files and copy actual source
RUN rm -rf server/src bevy-client/src server/benches
COPY server/src ./server/src
COPY server/benches ./server/benches
COPY server/tests ./server/tests
COPY bevy-client/src ./bevy-client/src

# Build the actual application
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/app/target/ \
    cargo build --release --bin boid-wars-server --locked && \
    cp ./target/release/boid-wars-server /bin/server

# WASM client build stage
FROM rust:1.88.0 AS wasm-builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    clang \
    lld \
    git \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Add wasm32 target
RUN rustup target add wasm32-unknown-unknown

# Install wasm-pack via cargo
RUN cargo install wasm-pack

# Copy workspace structure
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/
COPY shared/Cargo.toml ./shared/
COPY bevy-client/Cargo.toml ./bevy-client/

# Copy source code and cargo config
COPY bevy-client ./bevy-client
COPY shared ./shared

# Create dummy server directory to satisfy workspace
RUN mkdir -p server/src server/benches && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "// dummy benchmark" > server/benches/physics_benchmark.rs && \
    echo "// dummy benchmark" > server/benches/spatial_grid_bench.rs

# Build WASM client
# The getrandom crate needs the js feature for wasm32-unknown-unknown
# We need to ensure all dependencies use the correct features
RUN cd bevy-client && \
    wasm-pack build --target web --out-dir pkg --release --locked && \
    echo "✅ WASM build completed" && \
    ls -la pkg/ && \
    echo "📦 WASM package contents:" && \
    ls -la pkg/

# Runtime stage
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    tzdata \
    python3 \
    curl \
    libasound2 \
    libx11-6 \
    libx11-xcb1 \
    libxcb1 \
    libxcb-render0 \
    libxcb-shape0 \
    libxcb-xfixes0 \
    libxi6 \
    libxcursor1 \
    libxrandr2 \
    libxkbcommon0 \
    libxkbcommon-x11-0 \
    libxrender1 \
    libxext6 \
    libxfixes3 \
    libxinerama1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Create non-root user
RUN groupadd -g 1001 appuser && \
    useradd -u 1001 -g appuser -m -s /bin/bash appuser

# Copy server binary
COPY --from=builder /bin/server /app/server

# Copy WASM client assets
# Debug: List what we're copying
RUN mkdir -p /app/static
COPY --from=wasm-builder /app/bevy-client/pkg /app/static/pkg/
COPY --from=wasm-builder /app/bevy-client/index.html /app/static/
COPY --from=wasm-builder /app/bevy-client/demo.html /app/static/

# Verify files were copied
RUN echo "📁 Static files in /app/static:" && \
    ls -la /app/static/ && \
    echo "📦 Package files:" && \
    ls -la /app/static/pkg/ || echo "No pkg directory!"

# Copy game assets
# TODO: Update this to "assets" when assets-worktree gets merged back
COPY assets-worktree/assets /app/static/assets/

# Copy startup scripts
COPY scripts/start-production.sh /app/start-production.sh
COPY scripts/simple-http-ws-proxy.py /app/scripts/simple-http-ws-proxy.py
RUN chmod +x /app/start-production.sh

# Set ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080


# Run both servers using the startup script
CMD ["/app/start-production.sh"]