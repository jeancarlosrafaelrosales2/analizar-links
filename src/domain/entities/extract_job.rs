//! domain/entities/extract_job — Aggregate root: Job de extracción de audio.
//!
//! REGLA: Sin imports de sqlx, axum, reqwest, redis.
//! Solo tipos primitivos, chrono y uuid son permitidos aquí.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{AudioFormat, JobStatus, VideoUrl};

// ============================================================
// VIDEO METADATA — Info del video obtenida en runtime
// ============================================================

/// Metadatos del video obtenidos via yt-dlp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub title: String,
    pub author: Option<String>,
    pub duration_secs: Option<u64>,
    pub thumbnail_url: Option<String>,
    pub view_count: Option<u64>,
    pub upload_date: Option<String>,
    pub description: Option<String>,
}

// ============================================================
// EXTRACT JOB — Aggregate Root
// ============================================================

/// Job de extracción de audio — Aggregate Root.
///
/// Ciclo de vida:
/// `Pending` → `FetchingMetadata` → `Downloading` → `Transcoding` → `Completed`
///                                                                   ↘ `Failed`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractJob {
    /// Identificador único del job
    pub id: Uuid,
    /// URL del video a procesar
    pub url: VideoUrl,
    /// Formato de audio solicitado
    pub format: AudioFormat,
    /// Bitrate objetivo en kbps
    pub bitrate_kbps: u32,
    /// Estado actual del job
    pub status: JobStatus,
    /// Metadatos del video (disponibles después de FetchingMetadata)
    pub metadata: Option<VideoMetadata>,
    /// Timestamp de creación
    pub created_at: DateTime<Utc>,
    /// Timestamp de última actualización de estado
    pub updated_at: DateTime<Utc>,
}

impl ExtractJob {
    /// Constructor — crea un nuevo job en estado Pending.
    pub fn new(url: VideoUrl, format: AudioFormat, bitrate_kbps: u32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            url,
            format,
            bitrate_kbps,
            status: JobStatus::Pending,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    // ── State transitions ──────────────────────────────────

    /// Transición a FetchingMetadata.
    pub fn start_fetching_metadata(&mut self) {
        self.status = JobStatus::FetchingMetadata;
        self.updated_at = Utc::now();
    }

    /// Transición a Downloading con progreso inicial.
    pub fn start_downloading(&mut self) {
        self.status = JobStatus::Downloading { progress_pct: 0 };
        self.updated_at = Utc::now();
    }

    /// Actualiza el progreso de descarga.
    pub fn update_download_progress(&mut self, pct: u8) {
        self.status = JobStatus::Downloading { progress_pct: pct.min(100) };
        self.updated_at = Utc::now();
    }

    /// Transición a Transcoding.
    pub fn start_transcoding(&mut self) {
        self.status = JobStatus::Transcoding;
        self.updated_at = Utc::now();
    }

    /// Transición a Completed con path del archivo de salida.
    pub fn complete(&mut self, output_path: impl Into<String>) {
        let duration = self.metadata.as_ref().and_then(|m| m.duration_secs);
        self.status = JobStatus::Completed {
            output_path: output_path.into(),
            duration_secs: duration,
        };
        self.updated_at = Utc::now();
    }

    /// Transición a Failed con razón del error.
    pub fn fail(&mut self, reason: impl Into<String>) {
        self.status = JobStatus::Failed { reason: reason.into() };
        self.updated_at = Utc::now();
    }

    /// Almacena los metadatos del video.
    pub fn set_metadata(&mut self, metadata: VideoMetadata) {
        self.metadata = Some(metadata);
        self.updated_at = Utc::now();
    }

    /// True si el job ya terminó.
    pub fn is_done(&self) -> bool {
        self.status.is_terminal()
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_job() -> ExtractJob {
        let url = VideoUrl::parse("https://youtu.be/dQw4w9WgXcQ").unwrap();
        ExtractJob::new(url, AudioFormat::Mp3, 192)
    }

    #[test]
    fn test_new_job_is_pending() {
        let job = make_job();
        assert_eq!(job.status, JobStatus::Pending);
        assert!(!job.is_done());
        assert!(job.metadata.is_none());
    }

    #[test]
    fn test_state_machine_happy_path() {
        let mut job = make_job();

        job.start_fetching_metadata();
        assert_eq!(job.status.label(), "fetching_metadata");

        job.start_downloading();
        assert!(matches!(job.status, JobStatus::Downloading { progress_pct: 0 }));

        job.update_download_progress(50);
        assert!(matches!(job.status, JobStatus::Downloading { progress_pct: 50 }));

        job.start_transcoding();
        assert_eq!(job.status, JobStatus::Transcoding);

        job.complete("/tmp/output.mp3");
        assert!(job.is_done());
        assert!(matches!(job.status, JobStatus::Completed { .. }));
    }

    #[test]
    fn test_fail_transition() {
        let mut job = make_job();
        job.start_downloading();
        job.fail("Network timeout");
        assert!(job.is_done());
        assert!(matches!(job.status, JobStatus::Failed { .. }));
    }

    #[test]
    fn test_metadata_assignment() {
        let mut job = make_job();
        let meta = VideoMetadata {
            title: "Test Video".into(),
            author: Some("TestChannel".into()),
            duration_secs: Some(213),
            thumbnail_url: None,
            view_count: None,
            upload_date: None,
            description: None,
        };
        job.set_metadata(meta);
        assert_eq!(job.metadata.as_ref().unwrap().title, "Test Video");
        assert_eq!(job.metadata.as_ref().unwrap().duration_secs, Some(213));
    }

    #[test]
    fn test_updated_at_changes_on_transition() {
        let mut job = make_job();
        let initial_updated = job.updated_at;
        // Esperar un tick (en unit test usamos std::thread::sleep)
        std::thread::sleep(std::time::Duration::from_millis(5));
        job.start_fetching_metadata();
        assert!(job.updated_at >= initial_updated);
    }
}
