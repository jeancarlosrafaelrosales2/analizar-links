//! domain/ports — Traits (interfaces) que la infraestructura debe implementar.
//!
//! REGLA: Los traits son puros — no importan implementaciones concretas.
//! Las implementaciones viven en infrastructure/.

pub mod audio_storage;
pub mod audio_transcoder;
pub mod job_cache;
pub mod video_downloader;

pub use audio_storage::AudioStorage;
pub use audio_transcoder::AudioTranscoder;
pub use job_cache::JobCache;
pub use video_downloader::VideoDownloader;
