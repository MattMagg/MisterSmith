# Multi-stage build for Mister Smith framework binary.
# Target: <100MB compressed image (SC-006), <5s startup.

# ---------- Build stage ----------
FROM rust:1.88-slim AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy workspace manifests first for layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

# Build release binary
RUN cargo build --release --bin mister-smith \
    && strip target/release/mister-smith

# ---------- Runtime stage ----------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Non-root user for security
RUN useradd --create-home --shell /bin/bash mistersmith
USER mistersmith

WORKDIR /home/mistersmith

# Copy binary from build stage
COPY --from=builder /app/target/release/mister-smith /usr/local/bin/mister-smith

# Default configuration (can be overridden via env vars or mounted config)
ENV MISTER_SMITH_LOG_LEVEL=info
ENV MISTER_SMITH_LOG_FORMAT=json
ENV MISTER_SMITH_HTTP_PORT=8080

EXPOSE 8080 50051

HEALTHCHECK --interval=10s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health/live || exit 1

ENTRYPOINT ["mister-smith"]
