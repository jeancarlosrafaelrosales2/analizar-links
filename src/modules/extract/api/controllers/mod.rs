//! api/controllers — HTTP handlers delegando toda lógica a ExtractService.

pub mod extract_controller;

pub use extract_controller::{
    get_job_status_handler, list_jobs_handler, serve_audio_handler, sse_progress_handler,
    submit_batch_handler, submit_extraction_handler,
};
