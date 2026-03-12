//! common/error — Manejo canónico de errores.
//!
//! REGLA: Todo handler DEBE retornar `Result<T, AppError>`.
//! NUNCA retornar `(StatusCode, String)`.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

// ============================================================
// API ERROR RESPONSE — Envelope JSON canónico
// ============================================================

/// Respuesta de error estándar enviada a clientes.
///
/// ```json
/// {
///   "error_code": "INVALID_URL",
///   "message": "La URL proporcionada no es soportada",
///   "details": null,
///   "timestamp": "2026-03-11T12:00:00Z"
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    pub error_code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    pub timestamp: String,
}

impl ApiErrorResponse {
    fn now() -> String {
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
    }
}

// ============================================================
// APP ERROR — Tipo de error unificado
// ============================================================

#[derive(Debug, Error)]
pub enum AppError {
    // ── Validación ──────────────────────────────────────────
    #[error("URL inválida o no soportada: {0}")]
    InvalidUrl(String),

    #[error("Parámetro inválido: {field} — {reason}")]
    InvalidParam { field: String, reason: String },

    // ── Recursos ────────────────────────────────────────────
    #[error("Job no encontrado: {0}")]
    JobNotFound(String),

    // ── Herramientas externas ────────────────────────────────
    #[error("yt-dlp no encontrado en PATH. Instala con: pip install yt-dlp")]
    YtDlpNotFound,

    #[error("ffmpeg no encontrado en PATH. Instala con: apt-get install ffmpeg")]
    FfmpegNotFound,

    #[error("Error de descarga: {0}")]
    DownloadFailed(String),

    #[error("Error de transcoding: {0}")]
    TranscodeFailed(String),

    #[error("Error de almacenamiento: {0}")]
    StorageFailed(String),

    // ── Límites ─────────────────────────────────────────────
    #[error("Video demasiado largo. Máximo: {max_minutes} minutos")]
    VideTooLong { max_minutes: u64 },

    #[error("Rate limit excedido — demasiadas solicitudes")]
    RateLimitExceeded,

    // ── Interno ─────────────────────────────────────────────
    #[error("Error interno del servidor")]
    Internal(#[from] anyhow::Error),

    #[error("Error de IO: {0}")]
    Io(String),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidUrl(_) | Self::InvalidParam { .. } => StatusCode::BAD_REQUEST,
            Self::JobNotFound(_) => StatusCode::NOT_FOUND,
            Self::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            Self::YtDlpNotFound | Self::FfmpegNotFound => StatusCode::SERVICE_UNAVAILABLE,
            Self::VideTooLong { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidUrl(_) => "INVALID_URL",
            Self::InvalidParam { .. } => "INVALID_PARAM",
            Self::JobNotFound(_) => "JOB_NOT_FOUND",
            Self::YtDlpNotFound => "YTDLP_NOT_FOUND",
            Self::FfmpegNotFound => "FFMPEG_NOT_FOUND",
            Self::DownloadFailed(_) => "DOWNLOAD_FAILED",
            Self::TranscodeFailed(_) => "TRANSCODE_FAILED",
            Self::StorageFailed(_) => "STORAGE_FAILED",
            Self::VideTooLong { .. } => "VIDEO_TOO_LONG",
            Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Io(_) => "IO_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.error_code();

        // Log errores internos (no exponer detalle al cliente en 5xx)
        if status.is_server_error() {
            error!(error = %self, status = %status, "Internal server error");
        }

        let message = if status.is_server_error() {
            "Error interno del servidor. Por favor intenta de nuevo.".to_string()
        } else {
            self.to_string()
        };

        let body = ApiErrorResponse {
            error_code: code.to_string(),
            message,
            details: None,
            timestamp: ApiErrorResponse::now(),
        };

        (status, Json(body)).into_response()
    }
}

// ── Conversiones automáticas ────────────────────────────────

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_url_status() {
        let e = AppError::InvalidUrl("https://invalid.com/video".to_string());
        assert_eq!(e.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(e.error_code(), "INVALID_URL");
    }

    #[test]
    fn test_job_not_found_status() {
        let e = AppError::JobNotFound("abc-123".to_string());
        assert_eq!(e.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_ytdlp_not_found() {
        let e = AppError::YtDlpNotFound;
        assert_eq!(e.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert!(e.to_string().contains("yt-dlp"));
    }

    #[test]
    fn test_rate_limit_status() {
        let e = AppError::RateLimitExceeded;
        assert_eq!(e.status_code(), StatusCode::TOO_MANY_REQUESTS);
    }
}
