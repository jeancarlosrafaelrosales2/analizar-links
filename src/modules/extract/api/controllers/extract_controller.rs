//! api/controllers/extract_controller — Handlers HTTP (thin layer).
//!
//! REGLA: Handlers solo hacen:
//!   1. Deserializar input (Json, Path, Query, State)
//!   2. Construir Command/Query
//!   3. Llamar service
//!   4. Convertir a HTTP response
//!
//! Cero lógica de negocio aquí.

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tracing::instrument;

use crate::AppState;
use crate::common::{ApiResponse, AppError};
use crate::modules::extract::application::commands::ExtractAudioCommand;
use crate::modules::extract::application::dtos::requests::{BatchExtractRequest, ExtractRequest};
use crate::modules::extract::application::queries::GetJobStatusQuery;

// ── Helpers ──────────────────────────────────────────────────

/// Construye la base URL para URLs de respuesta (poll_url, download_url).
///
/// Orden de prioridad:
/// 1. `PUBLIC_URL` env var via AppState (producción con https://)
/// 2. `X-Forwarded-Proto` header (Railway/proxy termina TLS y re-emite como http)
/// 3. Fallback: http://host (desarrollo local)
fn get_base_url(state: &AppState, headers: &HeaderMap) -> String {
    // 1. PUBLIC_URL tiene prioridad absoluta
    if let Some(public_url) = &state.config.public_url {
        return public_url.clone();
    }

    let host = headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:3100");

    // 2. X-Forwarded-Proto detecta si el cliente usó HTTPS (Railway, nginx, etc.)
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");

    format!("{}://{}", proto, host)
}

// ── Handlers ─────────────────────────────────────────────────

/// POST /api/v1/extract
///
/// Encola un job de extracción de audio. Retorna job_id para sondear estado.
#[instrument(skip(state, _headers, payload), fields(url = %payload.url))]
pub async fn submit_extraction_handler(
    State(state): State<Arc<AppState>>,
    _headers: HeaderMap,
    Json(payload): Json<ExtractRequest>,
) -> Result<impl IntoResponse, AppError> {
    let base_url = get_base_url(&state, &_headers);

    let cmd = ExtractAudioCommand::new(
        &payload.url,
        payload.format.as_deref(),
        payload.bitrate,
    )?;

    let response = state.extract_service
        .submit_extraction(cmd, &base_url)
        .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(ApiResponse::ok_with_message(
            response,
            "Job de extracción encolado exitosamente",
        )),
    ))
}

/// POST /api/v1/extract/batch
///
/// Encola múltiples jobs de extracción. Retorna resultados parciales.
#[instrument(skip(state, headers, payload), fields(batch_size = payload.urls.len()))]
pub async fn submit_batch_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<BatchExtractRequest>,
) -> Result<impl IntoResponse, AppError> {
    let base_url = get_base_url(&state, &headers);

    let response = state.extract_service
        .submit_batch(
            payload.urls,
            payload.format.as_deref(),
            payload.bitrate,
            &base_url,
        )
        .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(ApiResponse::ok_with_message(
            response,
            "Batch de extracción procesado",
        )),
    ))
}

/// GET /api/v1/extract/:job_id/status
///
/// Consulta el estado actual de un job de extracción.
#[instrument(skip(state, headers), fields(job_id = %job_id))]
pub async fn get_job_status_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let base_url = get_base_url(&state, &headers);
    let query = GetJobStatusQuery::new(&job_id)?;

    let response = state.extract_service
        .get_job_status(query, &base_url)
        .await?;

    Ok((StatusCode::OK, Json(ApiResponse::ok(response))))
}

/// GET /api/v1/extract/jobs
///
/// Lista todos los jobs activos en caché.
#[instrument(skip(state, headers))]
pub async fn list_jobs_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let base_url = get_base_url(&state, &headers);
    let response = state.extract_service.list_jobs(&base_url).await?;
    Ok((StatusCode::OK, Json(ApiResponse::ok(response))))
}

/// GET /api/v1/extract/:job_id/sse
///
/// Server-Sent Events stream para progreso en tiempo real de un job.
/// El stream envía actualizaciones cada 500ms hasta que el job termine.
#[instrument(skip(state, headers), fields(job_id = %job_id))]
pub async fn sse_progress_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
    headers: HeaderMap,
) -> Result<Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>>, AppError> {
    let base_url = get_base_url(&state, &headers);
    let query = GetJobStatusQuery::new(&job_id)?;
    let uuid = query.job_id;

    // Verificar que el job existe antes de abrir el stream
    let _ = state.extract_service
        .get_job_status(query, &base_url)
        .await?;

    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        let mut retries_without_job = 0u8;

        loop {
            interval.tick().await;

            match state.extract_service.get_job_for_sse(uuid).await {
                Some(job) => {
                    retries_without_job = 0;
                    let status = job.status.label();
                    let progress_pct = match &job.status {
                        crate::domain::value_objects::JobStatus::Downloading { progress_pct } => Some(*progress_pct),
                        _ => None,
                    };

                    let download_url = match &job.status {
                        crate::domain::value_objects::JobStatus::Completed { output_path, .. } => {
                            let filename = std::path::Path::new(output_path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(output_path.as_str());
                            Some(format!(
                                "{}/api/v1/audio/{}",
                                base_url,
                                urlencoding::encode(filename)
                            ))
                        }
                        _ => None,
                    };

                    let error = match &job.status {
                        crate::domain::value_objects::JobStatus::Failed { reason } => Some(reason.clone()),
                        _ => None,
                    };

                    let data = serde_json::json!({
                        "job_id": job.id,
                        "status": status,
                        "progress_pct": progress_pct,
                        "title": job.metadata.as_ref().map(|m| &m.title),
                        "download_url": download_url,
                        "error": error,
                    });

                    yield Ok(Event::default()
                        .event("progress")
                        .data(data.to_string()));

                    // Terminar stream si el job finalizó
                    if job.is_done() {
                        yield Ok(Event::default().event("done").data(data.to_string()));
                        break;
                    }
                }
                None => {
                    retries_without_job += 1;
                    if retries_without_job > 10 {
                        yield Ok(Event::default()
                            .event("error")
                            .data(r#"{"error":"Job not found"}"#.to_string()));
                        break;
                    }
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// GET /api/v1/audio/:filename
///
/// Descarga el archivo de audio procesado.
/// Sirve archivos desde el storage local.
#[instrument(skip(state), fields(filename = %filename))]
pub async fn serve_audio_handler(
    State(state): State<Arc<AppState>>,
    Path(filename): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let decoded_name = urlencoding::decode(&filename)
        .map(|s| s.into_owned())
        .unwrap_or(filename);

    let file_path = format!("{}/{}", state.config.storage_path, decoded_name);

    let bytes = tokio::fs::read(&file_path)
        .await
        .map_err(|_| AppError::JobNotFound(decoded_name.clone()))?;

    let content_type = if decoded_name.ends_with(".mp3") {
        "audio/mpeg"
    } else if decoded_name.ends_with(".m4a") {
        "audio/mp4"
    } else if decoded_name.ends_with(".ogg") {
        "audio/ogg"
    } else if decoded_name.ends_with(".flac") {
        "audio/flac"
    } else {
        "audio/webm"
    };

    let disposition = format!("attachment; filename=\"{}\"", decoded_name);
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(content_type),
    );
    response_headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition).unwrap_or_else(|_| {
            HeaderValue::from_static("attachment")
        }),
    );

    Ok((StatusCode::OK, response_headers, bytes))
}
