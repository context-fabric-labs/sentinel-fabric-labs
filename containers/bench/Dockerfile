# Multi-stage Dockerfile for Bench
# Includes AWS CLI for S3 uploads and runs as non-root user

FROM rust:1.93-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy entire workspace (needed for workspace Cargo.toml)
COPY . .

# Build only bench in release mode
# Cargo builds binaries to workspace root target/ directory
RUN cargo build --release --bin bench

# Runtime stage - based on Amazon Linux for better AWS CLI compatibility
FROM amazonlinux:2023

# Install AWS CLI v2 and runtime dependencies
# Use --allowerasing to handle curl-minimal conflicts
# Install shadow-utils for groupadd/useradd commands
# Use x86_64 version for AWS compatibility (not ARM)
RUN yum update -y && \
    yum install -y \
    python3 \
    python3-pip \
    unzip \
    shadow-utils \
    && yum install -y --allowerasing curl \
    && curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip" && \
    unzip awscliv2.zip && \
    ./aws/install && \
    rm -rf aws awscliv2.zip && \
    yum clean all && \
    groupadd --gid 1000 app && \
    useradd --uid 1000 --gid 1000 --create-home --shell /bin/bash app

# Create app directory
WORKDIR /app
RUN chown -R app:app /app

# Copy binary from builder (workspace root target directory)
COPY --from=builder --chown=app:app /app/target/release/bench ./

# Switch to non-root user
USER app:app

# Set environment variables
ENV RUST_LOG=info
ENV SENTINEL_URL=http://sentinel:8080

# Volume for output files
VOLUME /app/output

# Run the binary
ENTRYPOINT ["./bench"]
