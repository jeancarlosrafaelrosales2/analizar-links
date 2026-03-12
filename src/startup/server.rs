//! startup/server — Bind, CORS, rate limiting, cleanup, graceful shutdown.

use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use crate::api::middleware::{build_cors_layer, RateLimiter, rate_limit_middleware};
use crate::common::AppConfig;
use crate::common::constants::CLEANUP_INTERVAL_SECS;
use crate::AppState;

use super::cleanup::spawn_cleanup_task;
use super::routes::build_router;

/// Arranca el servidor HTTP con graceful shutdown.
pub async fn run_server(
    state: Arc<AppState>,
    config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let cors = build_cors_layer(&config.cors_allowed_origins);
    let rate_limiter = RateLimiter::new(config.rate_limit_rps);

    // Spawn cleanup task
    spawn_cleanup_task(
        config.storage_path.clone(),
        config.job_ttl_seconds,
        CLEANUP_INTERVAL_SECS,
    );
    info!("🧹 Cleanup task started (TTL: {}s, interval: {}s)", config.job_ttl_seconds, CLEANUP_INTERVAL_SECS);

    // Spawn rate limiter cleanup (evict stale IP entries every 5 min)
    let limiter_clone = rate_limiter.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            limiter_clone.cleanup_stale_entries(600);
        }
    });

    let app = build_router(state)
        .layer(axum::middleware::from_fn_with_state(rate_limiter, rate_limit_middleware))
        .layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!("🚀 analizar-links listening on http://{}", addr);
    info!("📖 API Docs: http://{}/api-docs", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("👋 Server shutdown complete");
    Ok(())
}

/// Espera señal CTRL+C o SIGTERM para graceful shutdown.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received CTRL+C"),
        _ = terminate => info!("Received SIGTERM"),
    }
}
