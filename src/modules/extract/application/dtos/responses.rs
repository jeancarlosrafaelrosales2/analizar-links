//! application/dtos/responses — DTOs de salida del API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Response de POST /api/v1/extract — Job encolado exitosamente.
///
/// ```json
/// {
///   "job_id": "550e8400-...",
///   "status": "pending",
///   "message": "Job encolado. Consulta el estado en GET /api/v1/extract/{job_id}/status"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ExtractJobResponse {
    pub job_id: Uuid,
    pub status: String,
    pub message: String,
    pub poll_url: String,
}

/// Response de GET /api/v1/extract/{job_id}/status — Estado actual del job.
///
/// ```json
/// {
///   "job_id": "550e8400-...",
///   "status": "completed",
///   "title": "Never Gonna Give You Up",
///   "author": "Rick Astley",
///   "duration_secs": 213,
///   "download_url": "http://localhost:3000/api/v1/audio/...",
///   "format": "mp3",
///   "bitrate_kbps": 192,
///   "created_at": "2026-03-11T10:00:00Z",
///   "error": null
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_pct: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_url: Option<String>,
    pub format: String,
    pub bitrate_kbps: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response para GET /api/v1/extract/jobs — Lista de jobs activos.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct JobListResponse {
    pub jobs: Vec<JobStatusResponse>,
    pub total: usize,
}

/// Response de POST /api/v1/extract/batch — Batch de jobs encolados.
///
/// ```json
/// {
///   "jobs": [
///     { "job_id": "uuid1", "status": "pending", ... },
///     { "job_id": "uuid2", "status": "pending", ... }
///   ],
///   "errors": [
///     { "url": "https://invalid.com", "error": "Dominio no soportado" }
///   ],
///   "total_submitted": 2,
///   "total_errors": 1
/// }
/// ```
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchExtractResponse {
    pub jobs: Vec<ExtractJobResponse>,
    pub errors: Vec<BatchItemError>,
    pub total_submitted: usize,
    pub total_errors: usize,
}

/// Error individual en una operación batch.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BatchItemError {
    pub url: String,
    pub error: String,
}
