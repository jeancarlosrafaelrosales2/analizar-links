//! startup/routes — Wiring de todas las rutas en el Router global.

use axum::{middleware::from_fn, routing::get, Router};
use std::sync::Arc;

use crate::api::{
    health::{health_handler, ready_handler},
    middleware::request_logging_middleware,
};
use crate::modules::extract::api::routes::extract_routes;
use crate::AppState;

/// Construye el Router global con todas las rutas.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // ── Health ─────────────────────────────────────────
        .route("/health", get(health_handler))
        .route("/health/ready", get(ready_handler))
        // ── Módulos ────────────────────────────────────────
        .merge(extract_routes())
        // ── Middleware ─────────────────────────────────────
        .layer(from_fn(request_logging_middleware))
        // ── State (último — después de todos los layers) ──
        .with_state(state)
}
