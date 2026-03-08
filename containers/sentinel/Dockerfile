# Multi-stage Dockerfile for Sentinel
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

# Build only sentinel in release mode
# Cargo builds binaries to workspace root target/ directory
RUN cargo build --release --bin sentinel

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
COPY --from=builder --chown=app:app /app/target/release/sentinel ./

# Switch to non-root user
USER app:app

# Expose port
EXPOSE 8080

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV UPSTREAM_URL=http://upstream-stub:4000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the binary with bind to all interfaces
ENTRYPOINT ["./sentinel", "--bind", "0.0.0.0:8080"]
