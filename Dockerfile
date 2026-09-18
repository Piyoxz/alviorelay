# ==============================================================================
# AlvioRelay Production Multi-Stage Dockerfile
# Stage 1: Builder
# ==============================================================================
FROM rust:1.82-bookworm AS builder

WORKDIR /usr/src/alvio

# Install build dependencies for cryptographic and WebRTC compilation (aws-lc-sys requires cmake and clang)
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake \
    clang \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace dependencies and source trees
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY services ./services

# Compile stripped release binary for maximum throughput and minimal image size
RUN cargo build --release -p alvio-relay
RUN strip target/release/alvio-relay

# ==============================================================================
# Stage 2: Hardened Runtime Container
# ==============================================================================
FROM debian:bookworm-slim AS runtime

# Install CA certificates for TLS, curl for healthcheck, and ffmpeg for egress recording
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Create dedicated unprivileged non-root user
RUN groupadd -g 10001 alvio && \
    useradd -u 10001 -g alvio -m -d /app -s /bin/bash alvio

WORKDIR /app

# Install compiled executable
COPY --from=builder /usr/src/alvio/target/release/alvio-relay /usr/local/bin/alvio-relay

# Install baseline configuration
COPY alvio-relay.toml /app/alvio-relay.toml

# Prepare recordings directory and set file permissions
RUN mkdir -p /app/recordings && chown -R alvio:alvio /app

USER alvio:alvio

# Expose Signaling/HTTP/WHIP/Prometheus (7880) and WebRTC UDP Hot-Path (7882)
EXPOSE 7880/tcp
EXPOSE 7882/udp

# Automatic container health check probe
HEALTHCHECK --interval=15s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:7880/health || exit 1

ENTRYPOINT ["/usr/local/bin/alvio-relay"]
CMD ["start", "--config", "/app/alvio-relay.toml"]
