//! api/controllers — HTTP handlers delegando toda lógica a ExtractService.

pub mod extract_controller;

pub use extract_controller::{
    submit_extraction_handler,
    submit_batch_handler,
    get_job_status_handler,
    list_jobs_handler,
    sse_progress_handler,
    serve_audio_handler,
};
