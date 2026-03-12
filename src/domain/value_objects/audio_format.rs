//! domain/value_objects/audio_format — Formato de audio soportado.

use crate::common::AppError;
use serde::{Deserialize, Serialize};

/// Formato de audio de salida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    #[default]
    Mp3,
    M4a,
    Webm,
    Ogg,
    Flac,
}

impl AudioFormat {
    /// Parsea el formato desde un string.
    pub fn parse(s: &str) -> Result<Self, AppError> {
        match s.to_lowercase().as_str() {
            "mp3" => Ok(Self::Mp3),
            "m4a" => Ok(Self::M4a),
            "webm" => Ok(Self::Webm),
            "ogg" => Ok(Self::Ogg),
            "flac" => Ok(Self::Flac),
            other => Err(AppError::InvalidParam {
                field: "format".to_string(),
                reason: format!("Formato '{}' no soportado. Usa: mp3, m4a, webm, ogg, flac", other),
            }),
        }
    }

    /// Retorna la extensión de archivo correspondiente.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Mp3 => "mp3",
            Self::M4a => "m4a",
            Self::Webm => "webm",
            Self::Ogg => "ogg",
            Self::Flac => "flac",
        }
    }

    /// Retorna el codec de ffmpeg para este formato.
    pub fn ffmpeg_codec(&self) -> &'static str {
        match self {
            Self::Mp3 => "libmp3lame",
            Self::M4a => "aac",
            Self::Webm => "libvorbis",
            Self::Ogg => "libvorbis",
            Self::Flac => "flac",
        }
    }

    /// True si este formato ya es "nativo" y puede ser descargado sin transcoding.
    pub fn is_native_audio(&self) -> bool {
        matches!(self, Self::M4a | Self::Webm)
    }
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_formats() {
        assert_eq!(AudioFormat::parse("mp3").unwrap(), AudioFormat::Mp3);
        assert_eq!(AudioFormat::parse("M4A").unwrap(), AudioFormat::M4a);
        assert_eq!(AudioFormat::parse("WEBM").unwrap(), AudioFormat::Webm);
        assert_eq!(AudioFormat::parse("ogg").unwrap(), AudioFormat::Ogg);
        assert_eq!(AudioFormat::parse("flac").unwrap(), AudioFormat::Flac);
    }

    #[test]
    fn test_parse_invalid_format() {
        let result = AudioFormat::parse("wav");
        assert!(matches!(result, Err(AppError::InvalidParam { .. })));
    }

    #[test]
    fn test_extension() {
        assert_eq!(AudioFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioFormat::M4a.extension(), "m4a");
    }

    #[test]
    fn test_default_is_mp3() {
        assert_eq!(AudioFormat::default(), AudioFormat::Mp3);
    }

    #[test]
    fn test_native_audio_detection() {
        assert!(AudioFormat::M4a.is_native_audio());
        assert!(AudioFormat::Webm.is_native_audio());
        assert!(!AudioFormat::Mp3.is_native_audio());
        assert!(!AudioFormat::Flac.is_native_audio());
    }
}
