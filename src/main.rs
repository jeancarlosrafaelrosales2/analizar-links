//! analizar-links — Entry point del servidor.
//!
//! Responsabilidades:
//!   1. Cargar .env
//!   2. Inicializar tracing
//!   3. Cargar configuración
//!   4. Inicializar servicios (DI)
//!   5. Construir AppState
//!   6. Arrancar servidor con graceful shutdown

use analizar_links::{startup, AppState};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Cargar .env (dev-safe: no falla si no existe)
    dotenvy::dotenv().ok();

    // 2. Inicializar tracing
    startup::init_tracing();

    tracing::info!(
        "🎵 analizar-links — YouTube Audio Extraction API v{}",
        env!("CARGO_PKG_VERSION")
    );

    // 3. Cargar configuración
    let config = analizar_links::common::AppConfig::from_env();
    tracing::info!(
        "⚙️  Port: {} | Storage: {}",
        config.port,
        config.storage_path
    );
    tracing::info!(
        "🔧 yt-dlp: {} | ffmpeg: {}",
        config.ytdlp_path,
        config.ffmpeg_path
    );

    // 4. Inicializar servicios
    let services = startup::init_services(&config).await;

    // 5. Construir AppState
    let state = Arc::new(AppState {
        config: Arc::new(config.clone()),
        extract_service: services.extract_service,
    });

    // 6. Arrancar servidor
    startup::run_server(state, &config).await?;

    Ok(())
}
