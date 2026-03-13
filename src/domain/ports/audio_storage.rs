//! domain/ports/audio_storage — Port para almacenamiento de archivos de audio.

use crate::common::AppError;
use async_trait::async_trait;

/// Port: Almacenamiento de archivos de audio.
///
/// Abstracciona el backend de storage (local filesystem, S3, GCS, etc.).
#[async_trait]
pub trait AudioStorage: Send + Sync {
    /// Mueve o copia un archivo al storage definitivo.
    ///
    /// # Arguments
    /// * `temp_path` - Path temporal del archivo procesado
    /// * `filename` - Nombre final del archivo (sin path)
    ///
    /// # Returns
    /// URL o path relativo para acceder al archivo.
    async fn store(&self, temp_path: &str, filename: &str) -> Result<String, AppError>;

    /// Genera una URL de descarga para el archivo (firmada o directa).
    async fn get_download_url(&self, filename: &str) -> Result<String, AppError>;

    /// Elimina un archivo del storage.
    async fn delete(&self, filename: &str) -> Result<(), AppError>;
}
