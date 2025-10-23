# ============================================================================
# Stage 1: Builder - Compilation Rust optimisée
# ============================================================================ 
FROM rust:1.80-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire project
COPY . .

# Build application
RUN cargo build --release --package server

# ============================================================================ 
# Stage 2: Runtime - Image minimale Debian
# ============================================================================ 
FROM debian:12-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libsqlite3-0 \
    curl \
    tor \
    gnupg \
    # Install sops
    && curl -LO https://github.com/getsops/sops/releases/download/v3.8.1/sops-v3.8.1.linux.amd64 \
    && mv sops-v3.8.1.linux.amd64 /usr/local/bin/sops \
    && chmod +x /usr/local/bin/sops \
    # Install age
    && curl -LO https://github.com/FiloSottile/age/releases/download/v1.1.1/age-v1.1.1-linux-amd64.tar.gz \
    && tar -xzf age-v1.1.1-linux-amd64.tar.gz \
    && mv age/age /usr/local/bin/age \
    && mv age/age-keygen /usr/local/bin/age-keygen \
    && rm -rf age age-v1.1.1-linux-amd64.tar.gz \
    && rm -rf /var/lib/apt/lists/**

# Create non-root user
RUN useradd -m -u 1000 marketplace && \
    mkdir -p /app/data /app/logs && \
    chown -R marketplace:marketplace /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/server /app/server

# Copy configuration
COPY server/.env.example /app/.env

# Switch to non-root user
USER marketplace

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 CMD curl -f http://localhost:8080/health || exit 1

# Expose ports
EXPOSE 8080

# Run application
CMD ["/app/server"]
