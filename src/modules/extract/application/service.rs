//! application/service — ExtractService: orquesta el ciclo de vida completo de un job.
//!
//! Patrón: Fire-and-forget con tokio::spawn para procesamiento async.
//! El handler HTTP retorna inmediatamente con job_id; el procesamiento ocurre en background.

use std::sync::Arc;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

use crate::common::AppError;
use crate::domain::entities::ExtractJob;
use crate::domain::ports::{AudioStorage, AudioTranscoder, JobCache, VideoDownloader};
use crate::domain::value_objects::AudioFormat;

use super::commands::ExtractAudioCommand;
use super::dtos::responses::{
    BatchExtractResponse, BatchItemError, ExtractJobResponse, JobListResponse, JobStatusResponse,
};
use super::queries::GetJobStatusQuery;

// ============================================================
// EXTRACT SERVICE
// ============================================================

/// Servicio de aplicación — orquesta extracción de audio.
///
/// Contiene referencias Arc a todos los ports necesarios.
/// Thread-safe: puede ser clonado y compartido entre handlers.
pub struct ExtractService {
    downloader: Arc<dyn VideoDownloader>,
    transcoder: Arc<dyn AudioTranscoder>,
    storage: Arc<dyn AudioStorage>,
    job_cache: Arc<dyn JobCache>,
    /// Directorio temporal para archivos intermedios
    work_dir: String,
}

impl ExtractService {
    pub fn new(
        downloader: Arc<dyn VideoDownloader>,
        transcoder: Arc<dyn AudioTranscoder>,
        storage: Arc<dyn AudioStorage>,
        job_cache: Arc<dyn JobCache>,
        work_dir: impl Into<String>,
    ) -> Self {
        Self {
            downloader,
            transcoder,
            storage,
            job_cache,
            work_dir: work_dir.into(),
        }
    }

    // ── Commands ───────────────────────────────────────────

    /// Encola un job de extracción y retorna inmediatamente con el job_id.
    ///
    /// El procesamiento ocurre en background via `tokio::spawn`.
    /// Consultar el estado vía `get_job_status(query)`.
    #[instrument(skip(self), fields(url = %cmd.url))]
    pub async fn submit_extraction(
        &self,
        cmd: ExtractAudioCommand,
        base_url: &str,
    ) -> Result<ExtractJobResponse, AppError> {
        // Crear job en estado Pending
        let job = ExtractJob::new(cmd.url.clone(), cmd.format, cmd.bitrate_kbps);
        let job_id = job.id;

        // Persistir en caché
        self.job_cache.set(&job).await?;

        info!(job_id = %job_id, url = %cmd.url, format = %cmd.format, "Job submitted");

        // Spawn async task — fire-and-forget
        let downloader = Arc::clone(&self.downloader);
        let transcoder = Arc::clone(&self.transcoder);
        let storage = Arc::clone(&self.storage);
        let cache = Arc::clone(&self.job_cache);
        let work_dir = self.work_dir.clone();
        let url_str = cmd.url.as_str().to_string();
        let format = cmd.format;
        let bitrate = cmd.bitrate_kbps;

        tokio::spawn(async move {
            process_job(
                job_id, url_str, format, bitrate,
                downloader, transcoder, storage, cache, work_dir,
            ).await;
        });

        let poll_url = format!("{}/api/v1/extract/{}/status", base_url, job_id);

        Ok(ExtractJobResponse {
            job_id,
            status: "pending".to_string(),
            message: "Job encolado exitosamente. Sondea el estado en poll_url.".to_string(),
            poll_url,
        })
    }

    // ── Queries ────────────────────────────────────────────

    /// Obtiene el estado actual de un job.
    #[instrument(skip(self), fields(job_id = %query.job_id))]
    pub async fn get_job_status(
        &self,
        query: GetJobStatusQuery,
        base_url: &str,
    ) -> Result<JobStatusResponse, AppError> {
        let job = self.job_cache
            .get(query.job_id)
            .await
            .ok_or_else(|| AppError::JobNotFound(query.job_id.to_string()))?;

        Ok(job_to_response(job, base_url))
    }

    /// Lista todos los jobs en caché (activos y completados).
    pub async fn list_jobs(&self, base_url: &str) -> Result<JobListResponse, AppError> {
        let active_ids = self.job_cache.list_active_ids().await;
        let mut jobs = Vec::new();

        for id in active_ids {
            if let Some(job) = self.job_cache.get(id).await {
                jobs.push(job_to_response(job, base_url));
            }
        }

        let total = jobs.len();
        Ok(JobListResponse { jobs, total })
    }

    /// Encola un batch de extracciones. Retorna resultados parciales (éxitos + errores).
    ///
    /// Cada URL se procesa independientemente. Si una URL falla validación,
    /// las demás continúan normalmente.
    #[instrument(skip(self), fields(batch_size = urls.len()))]
    pub async fn submit_batch(
        &self,
        urls: Vec<String>,
        format_str: Option<&str>,
        bitrate: Option<u32>,
        base_url: &str,
    ) -> Result<BatchExtractResponse, AppError> {
        use crate::common::constants::MAX_BATCH_URLS;

        if urls.is_empty() {
            return Err(AppError::InvalidParam {
                field: "urls".to_string(),
                reason: "La lista de URLs no puede estar vacía".to_string(),
            });
        }

        if urls.len() > MAX_BATCH_URLS {
            return Err(AppError::InvalidParam {
                field: "urls".to_string(),
                reason: format!("Máximo {} URLs por batch. Recibidas: {}", MAX_BATCH_URLS, urls.len()),
            });
        }

        let mut jobs = Vec::new();
        let mut errors = Vec::new();

        for url in &urls {
            match ExtractAudioCommand::new(url, format_str, bitrate) {
                Ok(cmd) => {
                    match self.submit_extraction(cmd, base_url).await {
                        Ok(response) => jobs.push(response),
                        Err(e) => errors.push(BatchItemError {
                            url: url.clone(),
                            error: e.to_string(),
                        }),
                    }
                }
                Err(e) => {
                    errors.push(BatchItemError {
                        url: url.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        let total_submitted = jobs.len();
        let total_errors = errors.len();

        info!(submitted = total_submitted, errors = total_errors, "Batch submission completed");

        Ok(BatchExtractResponse {
            jobs,
            errors,
            total_submitted,
            total_errors,
        })
    }

    /// Obtiene un job para SSE streaming. Retorna None si no existe.
    pub async fn get_job_for_sse(&self, job_id: uuid::Uuid) -> Option<ExtractJob> {
        self.job_cache.get(job_id).await
    }
}

// ============================================================
// PIPELINE PRIVADO — Procesamiento asíncrono del job
// ============================================================

/// Pipeline completo de extracción de audio.
///
/// Ejecutado en un tokio::spawn — actualiza el job en caché en cada paso.
async fn process_job(
    job_id: Uuid,
    url: String,
    format: AudioFormat,
    bitrate_kbps: u32,
    downloader: Arc<dyn VideoDownloader>,
    transcoder: Arc<dyn AudioTranscoder>,
    storage: Arc<dyn AudioStorage>,
    cache: Arc<dyn JobCache>,
    work_dir: String,
) {
    info!(job_id = %job_id, "Starting extraction pipeline");

    // ── PASO 1: Obtener metadatos ────────────────────────
    update_job_status(&cache, job_id, |j| j.start_fetching_metadata()).await;

    let metadata = match downloader.get_metadata(&url).await {
        Ok(m) => m,
        Err(e) => {
            error!(job_id = %job_id, error = %e, "Metadata fetch failed");
            update_job_status(&cache, job_id, |j| j.fail(e.to_string())).await;
            return;
        }
    };

    // Verificar duración máxima
    if let Some(duration_secs) = metadata.duration_secs {
        let max_secs = crate::common::constants::MAX_VIDEO_DURATION_MINUTES * 60;
        if duration_secs > max_secs {
            let reason = format!(
                "Video demasiado largo: {} min. Máximo: {} min",
                duration_secs / 60,
                crate::common::constants::MAX_VIDEO_DURATION_MINUTES
            );
            warn!(job_id = %job_id, reason = %reason, "Video too long");
            update_job_status(&cache, job_id, |j| j.fail(reason)).await;
            return;
        }
    }

    update_job_status(&cache, job_id, |j| j.set_metadata(metadata.clone())).await;

    // ── PASO 2: Descargar audio ──────────────────────────
    update_job_status(&cache, job_id, |j| j.start_downloading()).await;

    let job_work_dir = format!("{}/{}", work_dir, job_id);
    let downloaded_path = match downloader.download_audio(&url, format, &job_work_dir).await {
        Ok(p) => p,
        Err(e) => {
            error!(job_id = %job_id, error = %e, "Download failed");
            update_job_status(&cache, job_id, |j| j.fail(e.to_string())).await;
            cleanup_dir(&job_work_dir).await;
            return;
        }
    };

    update_job_status(&cache, job_id, |j| j.update_download_progress(100)).await;

    // ── PASO 3: Transcodificar si es necesario ───────────
    let final_audio_path = if format == AudioFormat::Mp3 {
        // yt-dlp ya entregó MP3, verificar si necesita reencoding
        if downloaded_path.ends_with(".mp3") {
            info!(job_id = %job_id, "Already MP3, skipping transcode");
            downloaded_path.clone()
        } else {
            update_job_status(&cache, job_id, |j| j.start_transcoding()).await;
            let mp3_out = format!("{}/{}.mp3", job_work_dir, job_id);
            match transcoder.transcode_to_mp3(&downloaded_path, &mp3_out, bitrate_kbps).await {
                Ok(p) => p,
                Err(e) => {
                    error!(job_id = %job_id, error = %e, "Transcode failed");
                    update_job_status(&cache, job_id, |j| j.fail(e.to_string())).await;
                    cleanup_dir(&job_work_dir).await;
                    return;
                }
            }
        }
    } else {
        // Formato no-MP3: ya descargado por yt-dlp
        downloaded_path.clone()
    };

    // ── PASO 4: Almacenar definitivamente ────────────────
    let clean_title = sanitize_title(&metadata.title);
    let filename = format!("{}.{}", clean_title, format.extension());
    let stored_path = match storage.store(&final_audio_path, &filename).await {
        Ok(p) => p,
        Err(e) => {
            error!(job_id = %job_id, error = %e, "Storage failed");
            update_job_status(&cache, job_id, |j| j.fail(e.to_string())).await;
            cleanup_dir(&job_work_dir).await;
            return;
        }
    };

    // ── PASO 5: Marcar completado ────────────────────────
    update_job_status(&cache, job_id, |j| j.complete(&stored_path)).await;
    cleanup_dir(&job_work_dir).await;
    info!(job_id = %job_id, path = %stored_path, "Extraction pipeline completed ✅");
}

// ── Helpers ─────────────────────────────────────────────────

/// Actualiza el estado de un job en caché aplicando una mutación.
async fn update_job_status<F>(cache: &Arc<dyn JobCache>, job_id: Uuid, mutate: F)
where
    F: FnOnce(&mut ExtractJob),
{
    if let Some(mut job) = cache.get(job_id).await {
        mutate(&mut job);
        if let Err(e) = cache.set(&job).await {
            error!(job_id = %job_id, error = %e, "Failed to update job in cache");
        }
    }
}

/// Elimina el directorio de trabajo temporal del job.
async fn cleanup_dir(dir: &str) {
    if let Err(e) = tokio::fs::remove_dir_all(dir).await {
        warn!(dir = %dir, error = %e, "Failed to cleanup work dir (non-fatal)");
    }
}

/// Sanitiza el título de un video para usarlo como nombre de archivo.
///
/// Elimina contenido entre paréntesis/corchetes (ej. "(Video Oficial)", "[HD]"),
/// caracteres especiales y acorta a 80 caracteres.
fn sanitize_title(title: &str) -> String {
    let mut result = String::with_capacity(title.len());
    let mut depth: i32 = 0;

    for ch in title.chars() {
        match ch {
            '(' | '[' => depth += 1,
            ')' | ']' => { if depth > 0 { depth -= 1; } }
            _ if depth == 0 => {
                if ch.is_alphanumeric() || ch == '-' || ch == ' ' || ch == '\'' {
                    result.push(ch);
                }
            }
            _ => {}
        }
    }

    // Eliminar sufijo "- Topic" (canal automático de YouTube Music)
    let result = if let Some(idx) = result.find(" - Topic") {
        result[..idx].to_string()
    } else {
        result
    };

    // Normalizar espacios y convertir a underscores
    let parts: Vec<&str> = result.split_whitespace().collect();
    let joined = parts.join("_");
    let trimmed = joined.trim_matches('_');

    if trimmed.is_empty() {
        "audio".to_string()
    } else {
        let s = trimmed.to_string();
        // Truncar a 80 caracteres en límite de caracter válido
        if s.len() > 80 {
            s[..80].trim_end_matches('_').to_string()
        } else {
            s
        }
    }
}

/// Convierte un ExtractJob a JobStatusResponse.
fn job_to_response(job: ExtractJob, base_url: &str) -> JobStatusResponse {
    use crate::domain::value_objects::JobStatus;

    let (download_url, error, progress_pct) = match &job.status {
        JobStatus::Completed { output_path, .. } => {
            let filename = std::path::Path::new(output_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(output_path.as_str());
            let url = format!(
                "{}/api/v1/audio/{}",
                base_url,
                urlencoding::encode(filename)
            );
            (Some(url), None, None)
        }
        JobStatus::Failed { reason } => (None, Some(reason.clone()), None),
        JobStatus::Downloading { progress_pct } => (None, None, Some(*progress_pct)),
        _ => (None, None, None),
    };

    JobStatusResponse {
        job_id: job.id,
        status: job.status.label().to_string(),
        progress_pct,
        title: job.metadata.as_ref().map(|m| m.title.clone()),
        author: job.metadata.as_ref().and_then(|m| m.author.clone()),
        duration_secs: job.metadata.as_ref().and_then(|m| m.duration_secs),
        thumbnail_url: job.metadata.as_ref().and_then(|m| m.thumbnail_url.clone()),
        download_url,
        format: job.format.extension().to_string(),
        bitrate_kbps: job.bitrate_kbps,
        created_at: job.created_at,
        updated_at: job.updated_at,
        error,
    }
}
