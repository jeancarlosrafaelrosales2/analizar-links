//! modules/extract/domain — Tipos de dominio del módulo extract.
//! Re-exporta desde domain/ global para mantener coherencia DDD.

pub mod entities {
    pub use crate::domain::entities::{ExtractJob, VideoMetadata};
}

pub mod value_objects {
    pub use crate::domain::value_objects::{AudioFormat, JobStatus, VideoUrl};
}

pub mod events {
    pub use crate::domain::events::ExtractionEvent;
}
