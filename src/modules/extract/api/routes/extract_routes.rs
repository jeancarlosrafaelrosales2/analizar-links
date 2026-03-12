//! api/routes/extract_routes — Wiring de rutas del módulo extract.
//!
//! Namespace EXCLUSIVO: /api/v1/extract/*, /api/v1/audio/*

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::AppState;
use crate::modules::extract::api::controllers::{
    get_job_status_handler,
    list_jobs_handler,
    serve_audio_handler,
    sse_progress_handler,
    submit_batch_handler,
    submit_extraction_handler,
};

/// Construye el Router del módulo extract (sin state — se setea en build_router).
pub fn extract_routes() -> Router<Arc<AppState>> {
    Router::new()
        // ── Extract endpoints ──────────────────────────────
        .route("/api/v1/extract", post(submit_extraction_handler))
        .route("/api/v1/extract/batch", post(submit_batch_handler))
        .route("/api/v1/extract/jobs", get(list_jobs_handler))
        .route("/api/v1/extract/:job_id/status", get(get_job_status_handler))
        .route("/api/v1/extract/:job_id/sse", get(sse_progress_handler))
        // ── Audio serving ──────────────────────────────────
        .route("/api/v1/audio/:filename", get(serve_audio_handler))
}
