# Multi-stage Dockerfile for Upstream Stub
# Builds a minimal production image with non-root user

FROM rust:1.93-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy entire workspace (needed for workspace Cargo.toml)
COPY . .

# Build only upstream_stub in release mode
# Cargo builds binaries to workspace root target/ directory
RUN cargo build --release --bin upstream_stub

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 1000 app \
    && useradd --uid 1000 --gid 1000 --create-home --shell /bin/bash app

# Create app directory
WORKDIR /app
RUN chown -R app:app /app

# Copy binary from builder (workspace root target directory)
COPY --from=builder --chown=app:app /app/target/release/upstream_stub ./

# Switch to non-root user
USER app:app

# Expose port
EXPOSE 4000

# Set environment variables
ENV RUST_LOG=info

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:4000/health || exit 1

# Run the binary
ENTRYPOINT ["./upstream_stub"]
