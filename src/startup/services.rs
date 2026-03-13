//! startup/services — Inicialización de todos los servicios (DI manual).

use std::sync::Arc;
use tracing::info;

use crate::common::AppConfig;
use crate::infrastructure::{
    cache::MemoryJobCache,
    downloader::YtDlpDownloader,
    storage::LocalAudioStorage,
    transcoder::FfmpegTranscoder,
};
use crate::modules::extract::application::ExtractService;

/// Contiene todas las instancias de servicios inicializadas.
pub struct Services {
    pub extract_service: Arc<ExtractService>,
}

/// Inicializa todos los servicios con sus dependencias (DI manual).
pub async fn init_services(config: &AppConfig) -> Services {
    info!("🔧 Initializing services...");

    // Crear directorio de storage si no existe
    if let Err(e) = std::fs::create_dir_all(&config.storage_path) {
        tracing::warn!("Could not create storage_path {}: {}", config.storage_path, e);
    }

    // ── Infrastructure ───────────────────────────────────

    let downloader = Arc::new(YtDlpDownloader::new(
        &config.ytdlp_path,
        &config.storage_path,
        config.ytdlp_browser.clone(),
    ));

    let transcoder = Arc::new(FfmpegTranscoder::new(&config.ffmpeg_path));

    // PUBLIC_URL tiene prioridad (producción con https://)
    // Fallback: construir desde host:port (desarrollo local)
    let base_url = config.public_url.clone()
        .unwrap_or_else(|| format!("http://{}:{}", config.host, config.port));
    let storage = Arc::new(LocalAudioStorage::new(
        &config.storage_path,
        &base_url,
    ));

    let job_cache = Arc::new(MemoryJobCache::new());

    // ── Application ──────────────────────────────────────

    let work_dir = format!("{}/work", config.storage_path);
    let extract_service = Arc::new(ExtractService::new(
        downloader,
        transcoder,
        storage,
        job_cache,
        work_dir,
    ));

    info!("✅ Services initialized");

    Services { extract_service }
}
