//! domain/ports/video_downloader — Port para descarga de video/audio.

use crate::common::AppError;
use crate::domain::entities::VideoMetadata;
use crate::domain::value_objects::AudioFormat;
use async_trait::async_trait;

/// Port: Descargador de video/audio.
///
/// Las implementaciones concretas (yt-dlp, etc.) viven en infrastructure/.
#[async_trait]
pub trait VideoDownloader: Send + Sync {
    /// Obtiene metadatos del video sin descargar el contenido.
    async fn get_metadata(&self, url: &str) -> Result<VideoMetadata, AppError>;

    /// Descarga el mejor stream de audio disponible.
    ///
    /// Retorna el path al archivo descargado (en storage_path configurado).
    async fn download_audio(
        &self,
        url: &str,
        format: AudioFormat,
        output_path: &str,
    ) -> Result<String, AppError>;
}
