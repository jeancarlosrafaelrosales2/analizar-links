//! api/middleware/cors — Configuración CORS.

use tower_http::cors::{Any, CorsLayer};

/// Construye la capa CORS con los orígenes permitidos.
pub fn build_cors_layer(allowed_origins: &str) -> CorsLayer {
    if allowed_origins == "*" {
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
    }

    let origins: Vec<_> = allowed_origins
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    if origins.is_empty() {
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);
    }

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers(Any)
}
