# ─────────────────────────────────────────────────────────────
# Multi-stage Dockerfile for Download Music Backend
# ─────────────────────────────────────────────────────────────
# Stages:
#   1. chef     — Install cargo-chef for dependency caching
#   2. planner  — Compute the recipe (dependency graph)
#   3. builder  — Build the actual binary
#   4. runtime  — Minimal image with yt-dlp + ffmpeg
#   5. api      — API server target
#   6. worker   — Worker target (future)
# ─────────────────────────────────────────────────────────────

# ── Stage 1: Builder ──
FROM rust:1.94-slim-bookworm AS builder
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Cache dependencies layer (faster rebuilds)
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && echo "fn main(){}" > src/main.rs && \
    cargo build --release 2>&1 && \
    rm -f target/release/deps/analizar_links* target/release/analizar-links

# Build the real binary
COPY src ./src
RUN cargo build --release --bin analizar-links

# ── Stage 2: Runtime ──
FROM debian:bookworm-slim AS api
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    ffmpeg \
    python3 \
    python3-pip \
    && pip3 install --break-system-packages yt-dlp \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /data/audio

COPY --from=builder /app/target/release/analizar-links /usr/local/bin/analizar-links
EXPOSE 3000
CMD ["analizar-links"]
