//! startup — Bootstrap del servidor: configuración, DI, rutas, server.

pub mod cleanup;
pub mod routes;
pub mod server;
pub mod services;

pub use cleanup::spawn_cleanup_task;
pub use routes::build_router;
pub use server::run_server;
pub use services::init_services;

/// Re-exporta logging init para main.rs
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "analizar_links=info,tower_http=info".into()),
        )
        .with_target(false)
        .compact()
        .init();
}
