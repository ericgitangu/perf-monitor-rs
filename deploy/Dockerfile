FROM rust:1.83-slim as builder

WORKDIR /app

# Install dependencies for Rust compilation
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo manifest files
COPY Cargo.toml Cargo.lock ./

# Create dummy src and benches to cache dependencies layer
RUN mkdir -p src benches && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > benches/collection_benchmark.rs && \
    cargo build --release --features server,databases && \
    rm -rf src benches

# Copy actual source code
COPY src ./src
COPY benches ./benches

# Build release binary with server and database features enabled
RUN cargo build --release --features server,databases

# Runtime stage - Debian slim for smaller final image
FROM debian:bookworm-slim

# Install runtime dependencies (including curl for health checks)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binary from builder stage
COPY --from=builder /app/target/release/monitor-rs /usr/local/bin/

# Create data directory for metrics storage
RUN mkdir -p /data

# Expose Prometheus metrics port
EXPOSE 9100

# Run monitor-rs in server mode
CMD ["monitor-rs", "server", "--listen", "0.0.0.0:9100"]
