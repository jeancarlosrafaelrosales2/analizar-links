//! infrastructure/transcoder/ffmpeg_transcoder — AudioTranscoder usando ffmpeg subprocess.
//!
//! ffmpeg es el estándar de facto para procesamiento de audio/video.
//! Se invoca como subprocess para portabilidad máxima (sin bindings C).
//!
//! Instalación:
//!   Linux:  sudo apt-get install ffmpeg
//!   macOS:  brew install ffmpeg
//!   Windows: choco install ffmpeg  |  winget install ffmpeg

use async_trait::async_trait;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info};

use crate::common::AppError;
use crate::domain::ports::AudioTranscoder;

pub struct FfmpegTranscoder {
    /// Path al binario ffmpeg (default: "ffmpeg" si está en PATH)
    ffmpeg_path: String,
}

impl FfmpegTranscoder {
    pub fn new(ffmpeg_path: impl Into<String>) -> Self {
        Self {
            ffmpeg_path: ffmpeg_path.into(),
        }
    }
}

#[async_trait]
impl AudioTranscoder for FfmpegTranscoder {
    /// Verifica que ffmpeg esté disponible.
    async fn is_available(&self) -> bool {
        Command::new(&self.ffmpeg_path)
            .args(["-version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Transcodifica `input_path` a MP3 con bitrate especificado.
    ///
    /// Comando equivalente:
    /// `ffmpeg -i input.webm -vn -acodec libmp3lame -ab 192k -ar 44100 -ac 2 output.mp3 -y`
    async fn transcode_to_mp3(
        &self,
        input_path: &str,
        output_path: &str,
        bitrate_kbps: u32,
    ) -> Result<String, AppError> {
        debug!(
            input = %input_path,
            output = %output_path,
            bitrate = %bitrate_kbps,
            "Starting ffmpeg transcoding"
        );

        let bitrate_arg = format!("{}k", bitrate_kbps);

        // Cadena de filtros de audio para calidad profesional:
        // - loudnorm: normalización EBU R128 — más volumen percibido, sin distorsión ni clipping
        // - aresample: garantiza 44100 Hz de salida limpia
        // Sin costo adicional de infraestructura — filtros nativos de ffmpeg.
        let audio_filters = "loudnorm=I=-16:LRA=11:TP=-1.5,aresample=44100";

        let output = Command::new(&self.ffmpeg_path)
            .args([
                "-i",
                input_path,
                "-vn", // Sin video
                "-acodec",
                "libmp3lame", // Codec MP3
                "-b:a",
                &bitrate_arg, // Bitrate objetivo
                "-q:a",
                "0", // VBR calidad máxima
                "-ar",
                "44100", // Sample rate 44.1kHz
                "-ac",
                "2", // Estéreo
                "-af",
                audio_filters, // Filtros de calidad profesional
                "-threads",
                "0",  // Usar todos los núcleos del sistema
                "-y", // Sobreescribir si existe
                output_path,
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    AppError::FfmpegNotFound
                } else {
                    AppError::TranscodeFailed(format!("Error ejecutando ffmpeg: {}", e))
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(stderr = %stderr, "ffmpeg transcoding failed");
            return Err(AppError::TranscodeFailed(format!(
                "ffmpeg falló: {}",
                // Extraer línea de error relevante del stderr de ffmpeg
                stderr
                    .lines()
                    .filter(|l| l.contains("Error") || l.contains("error") || l.contains("Invalid"))
                    .last()
                    .unwrap_or("error desconocido")
            )));
        }

        info!(output = %output_path, bitrate = %bitrate_kbps, "Transcoding completed");
        Ok(output_path.to_string())
    }
}
