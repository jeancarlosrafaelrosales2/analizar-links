//! infrastructure/storage/local_storage — AudioStorage en filesystem local.
//!
//! Almacena archivos en `storage_path` y genera URLs via endpoint HTTP proxy.
//! Para producción, reemplazar con S3Storage (AWS/GCS/MinIO).

use async_trait::async_trait;
use std::path::Path;
use tracing::{debug, info};

use crate::common::AppError;
use crate::domain::ports::AudioStorage;

pub struct LocalAudioStorage {
    /// Directorio base de almacenamiento
    storage_path: String,
    /// Base URL del servidor para generar download URLs
    base_url: String,
}

impl LocalAudioStorage {
    pub fn new(storage_path: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            storage_path: storage_path.into(),
            base_url: base_url.into(),
        }
    }

    fn full_path(&self, filename: &str) -> String {
        format!("{}/{}", self.storage_path, filename)
    }
}

#[async_trait]
impl AudioStorage for LocalAudioStorage {
    /// Mueve el archivo temporal al directorio de storage definitivo.
    async fn store(&self, temp_path: &str, filename: &str) -> Result<String, AppError> {
        let dest_path = self.full_path(filename);

        // Asegurar que el directorio destino existe
        tokio::fs::create_dir_all(&self.storage_path)
            .await
            .map_err(|e| AppError::StorageFailed(format!("Error creando directorio: {}", e)))?;

        debug!(from = %temp_path, to = %dest_path, "Storing audio file");

        // Intentar rename (mismo filesystem: O(1)); sino copiar+eliminar
        if tokio::fs::rename(temp_path, &dest_path).await.is_err() {
            tokio::fs::copy(temp_path, &dest_path)
                .await
                .map_err(|e| AppError::StorageFailed(format!("Error copiando archivo: {}", e)))?;
            let _ = tokio::fs::remove_file(temp_path).await;
        }

        info!(path = %dest_path, filename = %filename, "Audio file stored");
        Ok(dest_path)
    }

    /// Retorna la URL de descarga HTTP para el archivo.
    async fn get_download_url(&self, filename: &str) -> Result<String, AppError> {
        let full_path = self.full_path(filename);

        // Verificar que el archivo existe
        if !Path::new(&full_path).exists() {
            return Err(AppError::StorageFailed(format!(
                "Archivo no encontrado: {}",
                filename
            )));
        }

        let url = format!(
            "{}/api/v1/audio/{}",
            self.base_url,
            urlencoding::encode(filename)
        );
        Ok(url)
    }

    /// Elimina un archivo del storage.
    async fn delete(&self, filename: &str) -> Result<(), AppError> {
        let full_path = self.full_path(filename);
        tokio::fs::remove_file(&full_path)
            .await
            .map_err(|e| AppError::StorageFailed(format!("Error eliminando archivo: {}", e)))
    }
}
