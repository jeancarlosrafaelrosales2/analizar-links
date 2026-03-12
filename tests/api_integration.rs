//! Integration tests — HTTP-level tests for all API endpoints.
//!
//! These tests build the real Router with real services (in-memory cache,
//! mock-safe infrastructure) and test the full HTTP request→response cycle.
//!
//! Run: cargo test --test api_integration

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // oneshot

use analizar_links::common::AppConfig;
use analizar_links::startup::services::init_services;
use analizar_links::startup::routes::build_router;
use analizar_links::AppState;

/// Helper: Build a fully wired Router with real services (in-memory cache).
async fn test_app() -> axum::Router {
    let config = AppConfig::test_defaults();

    let services = init_services(&config).await;

    let state = Arc::new(AppState {
        config: Arc::new(config),
        extract_service: services.extract_service,
    });

    build_router(state)
}

// ════════════════════════════════════════════════════════════
// HEALTH ENDPOINTS
// ════════════════════════════════════════════════════════════

#[tokio::test]
async fn health_returns_200_with_status_ok() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "analizar-links");
    assert!(json["version"].is_string());
}

#[tokio::test]
async fn health_ready_returns_200() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health/ready")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "ready");
}

// ════════════════════════════════════════════════════════════
// EXTRACT ENDPOINTS
// ════════════════════════════════════════════════════════════

#[tokio::test]
async fn submit_extraction_returns_202_with_job_id() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "url": "https://youtu.be/dQw4w9WgXcQ",
                        "format": "mp3",
                        "bitrate": 192
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Response wraps in ApiResponse { data: ..., message: ... }
    assert!(json["data"]["job_id"].is_string());
    assert_eq!(json["data"]["status"], "pending");
    assert!(json["data"]["poll_url"].is_string());
}

#[tokio::test]
async fn submit_extraction_with_invalid_url_returns_400() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "url": "https://vimeo.com/12345",
                        "format": "mp3",
                        "bitrate": 192
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should be 400 (unsupported domain) or 422
    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 400 or 422, got {}",
        response.status()
    );
}

#[tokio::test]
async fn submit_extraction_with_missing_url_returns_422() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "format": "mp3"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn submit_extraction_defaults_format_and_bitrate() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "url": "https://youtu.be/dQw4w9WgXcQ"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

// ════════════════════════════════════════════════════════════
// BATCH ENDPOINT
// ════════════════════════════════════════════════════════════

#[tokio::test]
async fn batch_extraction_returns_202_with_results() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract/batch")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "urls": [
                            "https://youtu.be/dQw4w9WgXcQ",
                            "https://youtu.be/Ks-_Mh1QhMc"
                        ],
                        "format": "mp3",
                        "bitrate": 192
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["success"].as_bool().unwrap_or(false));
    assert!(json["data"]["jobs"].is_array(), "Expected 'jobs' array in data, got: {json}");
    assert_eq!(json["data"]["jobs"].as_array().unwrap().len(), 2);
    assert_eq!(json["data"]["total_submitted"].as_u64().unwrap(), 2);
    assert_eq!(json["data"]["total_errors"].as_u64().unwrap(), 0);
}

#[tokio::test]
async fn batch_extraction_with_empty_urls_returns_400() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract/batch")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "urls": [],
                        "format": "mp3"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 400 or 422 for empty batch, got {}",
        response.status()
    );
}

#[tokio::test]
async fn batch_extraction_with_too_many_urls_returns_400() {
    let app = test_app().await;

    // Create 15 URLs (more than MAX_BATCH_URLS = 10)
    let urls: Vec<String> = (0..15)
        .map(|i| format!("https://youtu.be/video{}", i))
        .collect();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract/batch")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "urls": urls,
                        "format": "mp3"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 400 or 422 for too many URLs, got {}",
        response.status()
    );
}

// ════════════════════════════════════════════════════════════
// JOB STATUS ENDPOINT
// ════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_job_status_returns_404_for_nonexistent_job() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/extract/00000000-0000-0000-0000-000000000000/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_job_status_returns_400_for_invalid_uuid() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/extract/not-a-uuid/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        response.status() == StatusCode::BAD_REQUEST
            || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Expected 400 or 422 for invalid UUID, got {}",
        response.status()
    );
}

// ════════════════════════════════════════════════════════════
// LIST JOBS ENDPOINT
// ════════════════════════════════════════════════════════════

#[tokio::test]
async fn list_jobs_returns_200_with_empty_jobs() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/extract/jobs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(json["data"]["total"].is_number());
}

// ════════════════════════════════════════════════════════════
// AUDIO SERVE ENDPOINT
// ════════════════════════════════════════════════════════════

#[tokio::test]
async fn get_audio_returns_404_for_nonexistent_file() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/audio/nonexistent-file.mp3")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ════════════════════════════════════════════════════════════
// FULL FLOW: Submit → Status
// ════════════════════════════════════════════════════════════

#[tokio::test]
async fn submit_then_check_status_returns_job_data() {
    // We need a clone-able router — build state once, use Service
    let config = AppConfig::test_defaults();
    let services = init_services(&config).await;
    let state = Arc::new(AppState {
        config: Arc::new(config),
        extract_service: services.extract_service,
    });

    let app = build_router(state);

    // Step 1: Submit
    let submit_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "url": "https://youtu.be/dQw4w9WgXcQ",
                        "format": "mp3",
                        "bitrate": 192
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(submit_response.status(), StatusCode::ACCEPTED);

    let body = axum::body::to_bytes(submit_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let job_id = json["data"]["job_id"].as_str().unwrap();

    // Step 2: Check status — the job should exist
    let status_response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/extract/{}/status", job_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(status_response.status(), StatusCode::OK);

    let status_body = axum::body::to_bytes(status_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let status_json: serde_json::Value = serde_json::from_slice(&status_body).unwrap();

    assert_eq!(status_json["data"]["job_id"], job_id);
    // Job should have a status field
    assert!(status_json["data"]["status"].is_string());
}

// ════════════════════════════════════════════════════════════
// CORS / CONTENT-TYPE TESTS
// ════════════════════════════════════════════════════════════

#[tokio::test]
async fn missing_content_type_returns_415() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract")
                .body(Body::from(
                    serde_json::json!({"url": "https://youtu.be/test"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Without Content-Type: application/json, axum should reject
    assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

#[tokio::test]
async fn nonexistent_route_returns_404() {
    let app = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/nonexistent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// ════════════════════════════════════════════════════════════
// SUBMIT + LIST JOBS FLOW
// ════════════════════════════════════════════════════════════

#[tokio::test]
async fn submit_then_list_shows_job() {
    let config = AppConfig::test_defaults();
    let services = init_services(&config).await;
    let state = Arc::new(AppState {
        config: Arc::new(config),
        extract_service: services.extract_service,
    });

    let app = build_router(state);

    // Submit a job
    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/extract")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "url": "https://youtu.be/dQw4w9WgXcQ",
                        "format": "mp3",
                        "bitrate": 192
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // List jobs — should have at least 1
    let list_response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/extract/jobs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(list_response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    let total = json["data"]["total"].as_u64().unwrap_or(0);
    assert!(total >= 1, "Expected at least 1 job, got {}", total);
}
