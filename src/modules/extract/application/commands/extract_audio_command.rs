//! application/commands/extract_audio_command — Comando para extraer audio.

use crate::common::AppError;
use crate::common::constants::{
    ALLOWED_BITRATES_KBPS, DEFAULT_MP3_BITRATE_KBPS, DEFAULT_OUTPUT_FORMAT,
    MAX_VIDEO_DURATION_MINUTES,
};
use crate::domain::value_objects::{AudioFormat, VideoUrl};

/// Comando CQRS: Solicitar extracción de audio de un video.
///
/// Validado en construcción — si `new()` retorna Ok, el comando es invariablemente válido.
#[derive(Debug, Clone)]
pub struct ExtractAudioCommand {
    pub url: VideoUrl,
    pub format: AudioFormat,
    pub bitrate_kbps: u32,
}

impl ExtractAudioCommand {
    /// Constructor con validación completa.
    pub fn new(
        raw_url: impl Into<String>,
        format_str: Option<&str>,
        bitrate: Option<u32>,
    ) -> Result<Self, AppError> {
        // Validar URL
        let url = VideoUrl::parse(raw_url)?;

        // Validar formato (default: mp3)
        let format = match format_str {
            Some(f) => AudioFormat::parse(f)?,
            None => AudioFormat::parse(DEFAULT_OUTPUT_FORMAT).unwrap(),
        };

        // Validar bitrate (default: 320kbps — máxima calidad)
        let bitrate_kbps = bitrate.unwrap_or(DEFAULT_MP3_BITRATE_KBPS);
        if !ALLOWED_BITRATES_KBPS.contains(&bitrate_kbps) {
            return Err(AppError::InvalidParam {
                field: "bitrate".to_string(),
                reason: format!(
                    "Bitrate {} no válido. Permitidos: {:?}",
                    bitrate_kbps, ALLOWED_BITRATES_KBPS
                ),
            });
        }

        Ok(Self { url, format, bitrate_kbps })
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_command() {
        let cmd = ExtractAudioCommand::new(
            "https://youtu.be/dQw4w9WgXcQ",
            Some("mp3"),
            Some(192),
        );
        assert!(cmd.is_ok());
        let c = cmd.unwrap();
        assert_eq!(c.format, AudioFormat::Mp3);
        assert_eq!(c.bitrate_kbps, 192);
    }

    #[test]
    fn test_defaults() {
        let cmd = ExtractAudioCommand::new(
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            None,
            None,
        );
        assert!(cmd.is_ok());
        let c = cmd.unwrap();
        assert_eq!(c.format, AudioFormat::Mp3);
        assert_eq!(c.bitrate_kbps, 320); // default bitrate es 320 kbps (máxima calidad)
    }

    #[test]
    fn test_invalid_url() {
        let cmd = ExtractAudioCommand::new("https://vimeo.com/12345", None, None);
        assert!(matches!(cmd, Err(AppError::InvalidUrl(_))));
    }

    #[test]
    fn test_invalid_bitrate() {
        let cmd = ExtractAudioCommand::new(
            "https://youtu.be/dQw4w9WgXcQ",
            None,
            Some(999),
        );
        assert!(matches!(cmd, Err(AppError::InvalidParam { .. })));
    }

    #[test]
    fn test_invalid_format() {
        let cmd = ExtractAudioCommand::new(
            "https://youtu.be/dQw4w9WgXcQ",
            Some("wav"),
            None,
        );
        assert!(matches!(cmd, Err(AppError::InvalidParam { .. })));
    }
}
