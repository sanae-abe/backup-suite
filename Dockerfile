# Multi-stage build for backup-suite

# Stage 1: Build
FROM rust:1.75-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src
COPY benches ./benches
COPY tests ./tests

# Build the application
RUN cargo build --release && \
    strip target/release/backup-suite

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 backupuser

# Copy binary from builder
COPY --from=builder /app/target/release/backup-suite /usr/local/bin/backup-suite

# Create config directory
RUN mkdir -p /home/backupuser/.config/backup-suite && \
    chown -R backupuser:backupuser /home/backupuser

# Switch to non-root user
USER backupuser

# Set working directory
WORKDIR /home/backupuser

# Default command
CMD ["backup-suite", "--help"]

# Metadata
LABEL maintainer="backup-suite team"
LABEL version="1.0.0"
LABEL description="High-performance backup tool written in Rust"

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD ["backup-suite", "--version"]

# Volume for configuration
VOLUME ["/home/backupuser/.config/backup-suite"]

# Volume for backup source
VOLUME ["/data/source"]

# Volume for backup destination
VOLUME ["/data/backup"]
