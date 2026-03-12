//! domain/events — Domain events del bounded context de extracción.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{AudioFormat, VideoUrl};

/// Eventos de dominio emitidos durante el ciclo de vida de un job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum ExtractionEvent {
    /// Un job de extracción fue solicitado
    ExtractionRequested {
        job_id: Uuid,
        url: VideoUrl,
        format: AudioFormat,
        occurred_at: DateTime<Utc>,
    },
    /// Los metadatos del video fueron obtenidos exitosamente
    MetadataFetched {
        job_id: Uuid,
        title: String,
        duration_secs: Option<u64>,
        occurred_at: DateTime<Utc>,
    },
    /// La extracción de audio se completó exitosamente
    ExtractionCompleted {
        job_id: Uuid,
        output_path: String,
        format: AudioFormat,
        occurred_at: DateTime<Utc>,
    },
    /// La extracción falló
    ExtractionFailed {
        job_id: Uuid,
        reason: String,
        occurred_at: DateTime<Utc>,
    },
}

impl ExtractionEvent {
    pub fn job_id(&self) -> Uuid {
        match self {
            Self::ExtractionRequested { job_id, .. }
            | Self::MetadataFetched { job_id, .. }
            | Self::ExtractionCompleted { job_id, .. }
            | Self::ExtractionFailed { job_id, .. } => *job_id,
        }
    }
}
