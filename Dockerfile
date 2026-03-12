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

# ── Stage 1: Chef (pre-built image — no compile needed) ──
FROM lukemathwalker/cargo-chef:latest-rust-1.83-slim-bookworm AS chef
WORKDIR /app

# ── Stage 2: Planner ──
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Stage 3: Builder ──
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies (cached layer)
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin analizar-links

# ── Stage 4: Runtime ──
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
