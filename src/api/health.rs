//! api/health — Health check endpoint.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;

use crate::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    active_jobs: usize,
}

/// GET /health — Liveness probe.
pub async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let active_jobs = state.extract_service
        .list_jobs("http://localhost")
        .await
        .map(|r| r.total)
        .unwrap_or(0);

    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok",
            service: "analizar-links",
            version: env!("CARGO_PKG_VERSION"),
            active_jobs,
        }),
    )
}

/// GET /health/ready — Readiness probe (verifica tools externas).
pub async fn ready_handler() -> impl IntoResponse {
    // Por simplicidad, siempre retorna 200 — en prod verificar yt-dlp + ffmpeg
    (StatusCode::OK, Json(serde_json::json!({
        "status": "ready",
        "checks": {
            "server": "ok"
        }
    })))
}
