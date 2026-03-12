#!/usr/bin/env bash
# ─────────────────────────────────────────────────────
# start_dev.sh — Start analizar-links in development mode
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
  echo "✅ $1 found: $(command -v "$1")"
}

check_dep "${YTDLP_PATH:-yt-dlp}" "pip install yt-dlp"
check_dep "${FFMPEG_PATH:-ffmpeg}" "sudo apt-get install ffmpeg"

# ── Ensure storage directory ──
STORAGE="${STORAGE_PATH:-/tmp/analizar-links}"
mkdir -p "$STORAGE"
echo "📁 Storage: $STORAGE"

# ── Set Rust log level ──
export RUST_LOG="${RUST_LOG:-analizar_links=debug}"

echo ""
echo "🚀 Starting API server in DEV mode"
echo "   Port: ${PORT:-3100}"
echo "   Host: ${HOST:-0.0.0.0}"
echo "   CORS: ${CORS_ALLOWED_ORIGINS:-http://localhost:3100,http://localhost:4322}"
echo "   Logs: $RUST_LOG"
echo ""

# ── Run with cargo (dev mode — hot reload with cargo-watch if available) ──
if command -v cargo-watch &>/dev/null; then
  echo "👀 cargo-watch detected — running with auto-reload"
  echo "   (changes to src/ will trigger rebuild)"
  echo ""
  cargo watch -x run
else
  echo "💡 Tip: Install cargo-watch for auto-reload:"
  echo "   cargo install cargo-watch"
  echo ""
  cargo run
fi
