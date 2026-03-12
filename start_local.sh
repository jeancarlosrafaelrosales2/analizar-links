#!/usr/bin/env bash
# ─────────────────────────────────────────────────────
# start_local.sh — Start full local stack (Docker infra + API)
# ─────────────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# ── Load .env if present ──
if [ -f .env ]; then
  echo "📂 Loading .env"
  set -a
  source .env
  set +a
else
  echo "⚠️  No .env file found. Copy .env.example → .env and configure."
  echo "   cp .env.example .env"
  exit 1
fi

# ── Verify external dependencies ──
check_dep() {
  if ! command -v "$1" &>/dev/null; then
    echo "❌ $1 not found. Install it:"
    echo "   $2"
    exit 1
  fi
  echo "✅ $1 found"
}

check_dep "${YTDLP_PATH:-yt-dlp}" "pip install yt-dlp"
check_dep "${FFMPEG_PATH:-ffmpeg}" "sudo apt-get install ffmpeg"
check_dep "docker" "https://docs.docker.com/engine/install/"

# ── Check Docker Compose availability ──
if docker compose version &>/dev/null; then
  COMPOSE="docker compose"
elif command -v docker-compose &>/dev/null; then
  COMPOSE="docker-compose"
else
  echo "❌ Docker Compose not found."
  exit 1
fi
echo "✅ Using: $COMPOSE"

# ── Ensure storage directory ──
STORAGE="${STORAGE_PATH:-/tmp/analizar-links}"
mkdir -p "$STORAGE"

# ── Start infrastructure ──
echo ""
echo "🐳 Starting infrastructure services..."

# Only start infra services (redis + nginx), NOT the api container
# We run the API natively for local dev
if grep -q "redis:" docker-compose.yml 2>/dev/null; then
  $COMPOSE up -d redis nginx 2>/dev/null || {
    echo "⚠️  Docker services not started (no redis/nginx in compose or Docker not running)"
    echo "   Continuing with API only..."
  }
fi

# ── Set environment for local stack ──
export RUST_LOG="${RUST_LOG:-analizar_links=debug}"
export PORT="${PORT:-3100}"
export HOST="${HOST:-0.0.0.0}"

echo ""
echo "🚀 Starting API server in LOCAL mode"
echo "   ┌────────────────────────────────────────┐"
echo "   │ API:   http://localhost:${PORT}              │"
if docker ps --format '{{.Names}}' 2>/dev/null | grep -q redis; then
  echo "   │ Redis: redis://localhost:6379          │"
fi
if docker ps --format '{{.Names}}' 2>/dev/null | grep -q nginx; then
  echo "   │ NGINX: http://localhost:8080 (files)   │"
fi
echo "   │ Storage: $STORAGE"
echo "   └────────────────────────────────────────┘"
echo ""

# ── Cleanup on exit ──
cleanup() {
  echo ""
  echo "🛑 Shutting down..."
  if [ "${COMPOSE:-}" ]; then
    $COMPOSE down 2>/dev/null || true
  fi
  echo "✅ Done"
}
trap cleanup EXIT INT TERM

# ── Run ──
cargo run
