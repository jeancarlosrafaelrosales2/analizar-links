//! domain/value_objects/job_status — Estado de un job de extracción.

use serde::{Deserialize, Serialize};

/// Estado del job de extracción de audio.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum JobStatus {
    /// Encolado, esperando procesamiento
    Pending,
    /// Obteniendo metadatos del video
    FetchingMetadata,
    /// Descargando audio del video
    Downloading {
        /// Progreso de descarga 0–100
        progress_pct: u8,
    },
    /// Transcodificando a formato destino
    Transcoding,
    /// Completado — archivo disponible
    Completed {
        /// Path relativo o signed URL del archivo
        output_path: String,
        /// Duración en segundos
        duration_secs: Option<u64>,
    },
    /// Fallido con mensaje de error
    Failed {
        reason: String,
    },
}

impl JobStatus {
    /// True si el job ya terminó (éxito o fallo).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed { .. } | Self::Failed { .. })
    }

    /// True si el job está en progreso.
    pub fn is_in_progress(&self) -> bool {
        matches!(self, Self::FetchingMetadata | Self::Downloading { .. } | Self::Transcoding)
    }

    /// Nombre legible del estado.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::FetchingMetadata => "fetching_metadata",
            Self::Downloading { .. } => "downloading",
            Self::Transcoding => "transcoding",
            Self::Completed { .. } => "completed",
            Self::Failed { .. } => "failed",
        }
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_states() {
        assert!(JobStatus::Completed { output_path: "/tmp/a.mp3".into(), duration_secs: Some(180) }.is_terminal());
        assert!(JobStatus::Failed { reason: "error".into() }.is_terminal());
        assert!(!JobStatus::Pending.is_terminal());
        assert!(!JobStatus::Downloading { progress_pct: 50 }.is_terminal());
    }

    #[test]
    fn test_in_progress_states() {
        assert!(JobStatus::Downloading { progress_pct: 0 }.is_in_progress());
        assert!(JobStatus::Transcoding.is_in_progress());
        assert!(JobStatus::FetchingMetadata.is_in_progress());
        assert!(!JobStatus::Pending.is_in_progress());
    }

    #[test]
    fn test_label() {
        assert_eq!(JobStatus::Pending.label(), "pending");
        assert_eq!(JobStatus::Transcoding.label(), "transcoding");
        assert_eq!(JobStatus::Failed { reason: "x".into() }.label(), "failed");
    }

    #[test]
    fn test_serialization_roundtrip() {
        let status = JobStatus::Downloading { progress_pct: 42 };
        let json = serde_json::to_string(&status).unwrap();
        let back: JobStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, back);
    }
}
