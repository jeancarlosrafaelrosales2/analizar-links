//! domain/value_objects/video_url — URL de video validada.
//!
//! Value Object inmutable que garantiza que la URL es válida
//! y pertenece a un dominio soportado. No importa nada de infra.

use crate::common::constants::SUPPORTED_DOMAINS;
use crate::common::AppError;
use serde::{Deserialize, Serialize};

/// URL de video validada — Value Object.
///
/// Una vez construida vía `VideoUrl::parse()`, la URL es garantizadamente
/// válida y su dominio está en la lista de soportados.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoUrl(String);

impl VideoUrl {
    /// Parsea y valida una URL de video.
    ///
    /// # Errors
    /// - `AppError::InvalidUrl` si el formato es inválido o el dominio no está soportado.
    pub fn parse(raw: impl Into<String>) -> Result<Self, AppError> {
        let url = raw.into().trim().to_string();

        if url.is_empty() {
            return Err(AppError::InvalidUrl("URL vacía".to_string()));
        }

        // Validación básica de esquema
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(AppError::InvalidUrl(format!(
                "URL debe comenzar con https://. Recibido: {}",
                &url[..url.len().min(50)]
            )));
        }

        // Verificar dominio soportado
        let is_supported = SUPPORTED_DOMAINS.iter().any(|domain| {
            url.contains(&format!("://{}/", domain))
                || url.contains(&format!("://{}", domain))
                || url.ends_with(domain)
        });

        if !is_supported {
            return Err(AppError::InvalidUrl(format!(
                "Dominio no soportado. Soportados: {}",
                SUPPORTED_DOMAINS.join(", ")
            )));
        }

        Ok(Self(url))
    }

    /// Retorna la URL como string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for VideoUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_youtube_url() {
        let url = VideoUrl::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert!(url.is_ok());
        assert!(url.unwrap().as_str().contains("youtube.com"));
    }

    #[test]
    fn test_valid_youtu_be_short_url() {
        let url = VideoUrl::parse("https://youtu.be/dQw4w9WgXcQ");
        assert!(url.is_ok());
    }

    #[test]
    fn test_invalid_domain() {
        let url = VideoUrl::parse("https://vimeo.com/123456789");
        assert!(matches!(url, Err(AppError::InvalidUrl(_))));
    }

    #[test]
    fn test_empty_url() {
        let url = VideoUrl::parse("");
        assert!(matches!(url, Err(AppError::InvalidUrl(_))));
    }

    #[test]
    fn test_no_https_scheme() {
        let url = VideoUrl::parse("ftp://youtube.com/watch?v=abc");
        assert!(matches!(url, Err(AppError::InvalidUrl(_))));
    }

    #[test]
    fn test_display() {
        let raw = "https://youtu.be/dQw4w9WgXcQ";
        let url = VideoUrl::parse(raw).unwrap();
        assert_eq!(url.to_string(), raw);
    }
}
