//! infrastructure/downloader/ytdlp_downloader — VideoDownloader usando yt-dlp subprocess.
//!
//! yt-dlp es la herramienta más confiable para YouTube (actualizada constantemente).
//! Se invoca como subprocess para evitar bindings FFI complejos.
//!
//! Instalación: pip install yt-dlp  |  brew install yt-dlp
//! Documentación: https://github.com/yt-dlp/yt-dlp

use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

use crate::common::AppError;
use crate::domain::entities::VideoMetadata;
use crate::domain::ports::VideoDownloader;
use crate::domain::value_objects::AudioFormat;

// ============================================================
// YT-DLP JSON OUTPUT — Struct para parsear --dump-json
// ============================================================

#[derive(Debug, Deserialize)]
struct YtDlpInfo {
    title: Option<String>,
    uploader: Option<String>,
    duration: Option<f64>,
    thumbnail: Option<String>,
    view_count: Option<u64>,
    upload_date: Option<String>,
    description: Option<String>,
}

// ============================================================
// YtDlpDownloader — Implementación
// ============================================================

pub struct YtDlpDownloader {
    /// Path al binario yt-dlp (default: "yt-dlp" si está en PATH)
    ytdlp_path: String,
    /// Directorio de trabajo para archivos temporales
    work_dir: String,
    /// Browser para cookies (firefox, chrome, chromium, brave, etc.)
    browser: Option<String>,
}

impl YtDlpDownloader {
    pub fn new(
        ytdlp_path: impl Into<String>,
        work_dir: impl Into<String>,
        browser: Option<String>,
    ) -> Self {
        Self {
            ytdlp_path: ytdlp_path.into(),
            work_dir: work_dir.into(),
            browser,
        }
    }

    /// Construye los args base compartidos entre metadata y download.
    fn base_args(&self) -> Vec<String> {
        let mut args = vec![
            "--no-playlist".to_string(),
            "--js-runtimes".to_string(),
            "node".to_string(),
        ];
        if let Some(ref browser) = self.browser {
            args.push("--cookies-from-browser".to_string());
            args.push(browser.clone());
        }
        args
    }

    /// Verifica que yt-dlp esté disponible.
    pub async fn check_available(&self) -> bool {
        Command::new(&self.ytdlp_path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

#[async_trait]
impl VideoDownloader for YtDlpDownloader {
    /// Obtiene metadatos del video via `yt-dlp --dump-json --no-download`.
    async fn get_metadata(&self, url: &str) -> Result<VideoMetadata, AppError> {
        debug!(url = %url, "Fetching video metadata via yt-dlp");

        let mut args = vec![
            "--dump-json".to_string(),
            "--no-download".to_string(),
            "--quiet".to_string(),
        ];
        args.extend(self.base_args());
        args.push(url.to_string());

        let output = Command::new(&self.ytdlp_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    AppError::YtDlpNotFound
                } else {
                    AppError::DownloadFailed(format!("Error ejecutando yt-dlp: {}", e))
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Only treat ERROR lines as actual errors (ignore WARNINGs)
            let error_line = stderr
                .lines()
                .find(|l| l.contains("ERROR"))
                .unwrap_or_else(|| stderr.lines().next().unwrap_or("error desconocido"));
            error!(stderr = %stderr, "yt-dlp metadata failed");
            return Err(AppError::DownloadFailed(format!(
                "yt-dlp no pudo obtener metadatos: {}",
                error_line
            )));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let info: YtDlpInfo = serde_json::from_str(&json_str).map_err(|e| {
            AppError::DownloadFailed(format!("Error parseando JSON de yt-dlp: {}", e))
        })?;

        info!(
            title = ?info.title,
            duration = ?info.duration,
            "Metadata fetched successfully"
        );

        Ok(VideoMetadata {
            title: info.title.unwrap_or_else(|| "Sin título".to_string()),
            author: info.uploader,
            duration_secs: info.duration.map(|d| d as u64),
            thumbnail_url: info.thumbnail,
            view_count: info.view_count,
            upload_date: info.upload_date,
            description: info.description,
        })
    }

    /// Descarga el mejor stream de audio disponible.
    ///
    /// Usa `yt-dlp -x --audio-format <format>` para extraer el audio directamente.
    /// yt-dlp maneja internamente la selección de stream y conversión cuando es posible.
    async fn download_audio(
        &self,
        url: &str,
        format: AudioFormat,
        output_dir: &str,
    ) -> Result<String, AppError> {
        // Aseguramos que el directorio existe
        tokio::fs::create_dir_all(output_dir)
            .await
            .map_err(|e| AppError::StorageFailed(format!("No se pudo crear directorio: {}", e)))?;

        let output_template = format!("{}/%(id)s.%(ext)s", output_dir);
        let format_str = format.extension();

        info!(url = %url, format = %format_str, output_dir = %output_dir, "Downloading audio");

        let mut args = vec![
            "-x".to_string(),
            "--audio-format".to_string(), format_str.to_string(),
            "--audio-quality".to_string(), "0".to_string(),
            "--format".to_string(), "bestaudio/best".to_string(),
            "--no-part".to_string(),
            "--quiet".to_string(),
            "--no-warnings".to_string(),
        ];
        args.extend(self.base_args());
        args.extend([
            "-o".to_string(), output_template,
            url.to_string(),
        ]);

        let output = Command::new(&self.ytdlp_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    AppError::YtDlpNotFound
                } else {
                    AppError::DownloadFailed(format!("Error ejecutando yt-dlp: {}", e))
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(stderr = %stderr, "yt-dlp download failed");
            return Err(AppError::DownloadFailed(format!(
                "Error de descarga: {}",
                stderr.lines().next().unwrap_or("error desconocido")
            )));
        }

        // Buscar el archivo descargado en output_dir
        let ext = format.extension();
        let mut found_path = None;
        let mut dir = tokio::fs::read_dir(output_dir)
            .await
            .map_err(|e| AppError::StorageFailed(format!("Error leyendo directorio: {}", e)))?;

        while let Some(entry) = dir.next_entry().await.map_err(|e| {
            AppError::StorageFailed(format!("Error iterando directorio: {}", e))
        })? {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some(ext) {
                found_path = Some(path.to_string_lossy().to_string());
                break;
            }
        }

        found_path.ok_or_else(|| {
            AppError::DownloadFailed(format!(
                "No se encontró archivo .{} en {} después de descarga",
                ext, output_dir
            ))
        })
    }
}
