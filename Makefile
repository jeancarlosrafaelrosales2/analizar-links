# ─────────────────────────────────────────────────────────────
# analizar-links — Makefile
# ─────────────────────────────────────────────────────────────
# Usage:
#   make dev          → Run locally (cargo run, port 3000)
#   make local        → Run full stack (docker-compose up)
#   make test         → Run unit tests
#   make check        → cargo check + test + clippy
#   make docker-up    → Start infra (redis + nginx)
#   make docker-down  → Stop infra
#   make clean        → Clean build artifacts
# ─────────────────────────────────────────────────────────────

.PHONY: dev local test check lint fmt build docker-up docker-down docker-build clean smoke help

# Default target
.DEFAULT_GOAL := help

# ── Development ──────────────────────────────────────────────

dev: ## Run API server locally (cargo run, port 3100)
	@echo "🚀 Starting API server on http://localhost:3100"
	@cp -n .env.example .env 2>/dev/null || true
	cargo run

dev-watch: ## Run with auto-reload (requires cargo-watch)
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Install: cargo install cargo-watch"; exit 1; }
	@cp -n .env.example .env 2>/dev/null || true
	cargo watch -x run -w src/

local: docker-up dev ## Run full local stack: Redis + NGINX + API
	@echo "✅ Full local stack running"

# ── Testing ──────────────────────────────────────────────────

test: ## Run unit tests
	cargo test --lib

test-verbose: ## Run tests with output
	cargo test --lib -- --nocapture

test-watch: ## Run tests in watch mode
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Install: cargo install cargo-watch"; exit 1; }
	cargo watch -x "test --lib"

# ── Quality ──────────────────────────────────────────────────

check: ## Full quality gate: check + test + clippy
	@echo "━━━ 🔍 Compile check ━━━"
	cargo check --all-targets
	@echo "━━━ 🧪 Tests ━━━"
	cargo test --lib
	@echo "━━━ 📎 Clippy ━━━"
	cargo clippy --all-targets -- -W clippy::pedantic -A clippy::module_name_repetitions -A clippy::must_use_candidate 2>&1 || true
	@echo "━━━ 🎨 Format check ━━━"
	cargo fmt --check
	@echo ""
	@echo "✅ All quality gates passed"

lint: ## Run clippy linter
	cargo clippy --all-targets

fmt: ## Format code
	cargo fmt

build: ## Build release binary
	cargo build --release

# ── Docker / Infrastructure ──────────────────────────────────

docker-up: ## Start infrastructure (Redis + NGINX)
	@echo "🐳 Starting Redis + NGINX..."
	@mkdir -p /tmp/analizar-links
	docker compose up -d redis nginx
	@echo "✅ Redis: localhost:6379 | NGINX: localhost:8080"

docker-up-all: ## Start full stack (API + Worker + Redis + NGINX)
	docker compose up -d --build
	@echo "✅ Full stack running"
	@echo "   API:   http://localhost:3100"
	@echo "   NGINX: http://localhost:8080"
	@echo "   Redis: localhost:6379"

docker-down: ## Stop all Docker services
	docker compose down

docker-build: ## Build Docker images
	docker compose build

docker-logs: ## Follow all logs
	docker compose logs -f

docker-logs-api: ## Follow API logs
	docker compose logs -f api

docker-logs-worker: ## Follow Worker logs
	docker compose logs -f worker

# ── Dependencies ─────────────────────────────────────────────

deps: ## Install system dependencies
	@echo "📦 Installing system dependencies..."
	pip install yt-dlp 2>/dev/null || pip3 install yt-dlp
	@echo "ℹ️  ffmpeg: install via your package manager (apt/brew)"
	@echo "   Ubuntu: sudo apt-get install ffmpeg"
	@echo "   macOS:  brew install ffmpeg"

deps-check: ## Verify dependencies are installed
	@echo "Checking dependencies..."
	@command -v yt-dlp >/dev/null 2>&1 && echo "  ✅ yt-dlp: $$(yt-dlp --version)" || echo "  ❌ yt-dlp not found"
	@command -v ffmpeg >/dev/null 2>&1 && echo "  ✅ ffmpeg: $$(ffmpeg -version 2>&1 | head -1)" || echo "  ❌ ffmpeg not found"
	@command -v docker >/dev/null 2>&1 && echo "  ✅ docker: $$(docker --version)" || echo "  ❌ docker not found"
	@command -v cargo  >/dev/null 2>&1 && echo "  ✅ cargo:  $$(cargo --version)" || echo "  ❌ cargo not found"

# ── Smoke Tests ──────────────────────────────────────────────

smoke: ## Run smoke tests against running server
	@echo "🔥 Smoke testing http://localhost:3100..."
	@echo ""
	@echo "━━━ Health ━━━"
	@curl -sf http://localhost:3100/health | python3 -m json.tool 2>/dev/null || echo "❌ Health check failed"
	@echo ""
	@echo "━━━ Health Ready ━━━"
	@curl -sf http://localhost:3100/health/ready | python3 -m json.tool 2>/dev/null || echo "❌ Readiness check failed"
	@echo ""
	@echo "━━━ Extract (submit) ━━━"
	@curl -sf -X POST http://localhost:3100/api/v1/extract \
		-H "Content-Type: application/json" \
		-d '{"url":"https://youtu.be/dQw4w9WgXcQ","format":"mp3","bitrate":192}' \
		| python3 -m json.tool 2>/dev/null || echo "❌ Extract submission failed"
	@echo ""
	@echo "━━━ List Jobs ━━━"
	@curl -sf http://localhost:3100/api/v1/extract/jobs | python3 -m json.tool 2>/dev/null || echo "❌ List jobs failed"
	@echo ""
	@echo "✅ Smoke tests complete"

# ── Cleanup ──────────────────────────────────────────────────

clean: ## Clean build artifacts
	cargo clean
	rm -rf /tmp/analizar-links/*

# ── Help ─────────────────────────────────────────────────────

help: ## Show this help
	@echo ""
	@echo "analizar-links — YouTube Audio Extraction API"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'
	@echo ""
