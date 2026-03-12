//! domain/ports/audio_transcoder — Port para transcodificación de audio.

use async_trait::async_trait;
use crate::common::AppError;

/// Port: Transcodificador de audio.
///
/// Convierte un archivo de audio a MP3 u otro formato.
#[async_trait]
pub trait AudioTranscoder: Send + Sync {
    /// Transcodifica el archivo en `input_path` a MP3.
    ///
    /// # Arguments
    /// * `input_path` - Path al archivo de entrada
    /// * `output_path` - Path donde se guardará el MP3 resultante
    /// * `bitrate_kbps` - Bitrate en kbps (e.g. 192)
    ///
    /// # Returns
    /// Path al archivo de salida (`output_path`).
    async fn transcode_to_mp3(
        &self,
        input_path: &str,
        output_path: &str,
        bitrate_kbps: u32,
    ) -> Result<String, AppError>;

    /// Verifica si el transcodificador está disponible en el sistema.
    async fn is_available(&self) -> bool;
}
