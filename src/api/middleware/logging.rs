//! api/middleware/logging — Request logging middleware.

use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::info;

/// Middleware: Registra método, path, status y duración de cada request.
pub async fn request_logging_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let start = Instant::now();

    let response = next.run(req).await;

    let duration_ms = start.elapsed().as_millis();
    let status = response.status().as_u16();

    info!(
        method = %method,
        path = %uri.path(),
        status = status,
        duration_ms = duration_ms,
        "HTTP request"
    );

    response
}
