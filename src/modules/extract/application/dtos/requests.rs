//! application/dtos/requests — DTOs de entrada del API.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request para POST /api/v1/extract
///
/// ```json
/// {
///   "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
///   "format": "mp3",
///   "bitrate": 192
/// }
/// ```
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ExtractRequest {
    /// URL del video a procesar (YouTube, youtu.be)
    pub url: String,
    /// Formato de audio deseado: mp3, m4a, webm, ogg, flac (default: mp3)
    #[serde(default)]
    pub format: Option<String>,
    /// Bitrate en kbps: 128, 192, 256, 320 (default: 192)
    #[serde(default)]
    pub bitrate: Option<u32>,
}

/// Request para POST /api/v1/extract/batch
///
/// ```json
/// {
///   "urls": [
///     "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
///     "https://youtu.be/9bZkp7q19f0"
///   ],
///   "format": "mp3",
///   "bitrate": 192
/// }
/// ```
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct BatchExtractRequest {
    /// Lista de URLs de videos a procesar (máximo 10)
    pub urls: Vec<String>,
    /// Formato de audio deseado para todos los jobs (default: mp3)
    #[serde(default)]
    pub format: Option<String>,
    /// Bitrate en kbps para todos los jobs (default: 192)
    #[serde(default)]
    pub bitrate: Option<u32>,
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_request_deserialize() {
        let json = r#"{"url":"https://youtu.be/dQw4w9WgXcQ","format":"mp3","bitrate":192}"#;
        let req: ExtractRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.url, "https://youtu.be/dQw4w9WgXcQ");
        assert_eq!(req.format, Some("mp3".to_string()));
        assert_eq!(req.bitrate, Some(192));
    }

    #[test]
    fn test_extract_request_defaults() {
        let json = r#"{"url":"https://youtu.be/dQw4w9WgXcQ"}"#;
        let req: ExtractRequest = serde_json::from_str(json).unwrap();
        assert!(req.format.is_none());
        assert!(req.bitrate.is_none());
    }

    #[test]
    fn test_batch_request_deserialize() {
        let json =
            r#"{"urls":["https://youtu.be/a","https://youtu.be/b"],"format":"m4a","bitrate":256}"#;
        let req: BatchExtractRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.urls.len(), 2);
        assert_eq!(req.format, Some("m4a".to_string()));
        assert_eq!(req.bitrate, Some(256));
    }

    #[test]
    fn test_batch_request_defaults() {
        let json = r#"{"urls":["https://youtu.be/a"]}"#;
        let req: BatchExtractRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.urls.len(), 1);
        assert!(req.format.is_none());
        assert!(req.bitrate.is_none());
    }
}
