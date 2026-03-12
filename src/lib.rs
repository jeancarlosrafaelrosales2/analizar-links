//! analizar-links — YouTube Audio Extraction API
//!
//! Arquitectura: Clean Architecture DDD inspirada en simple_api_gateway.
//!
//! Capas (top → bottom):
//!   api/         → HTTP: controllers, middleware, routes
//!   modules/     → Bounded contexts (extract)
//!   application/ → CQRS: commands, queries, services, DTOs
//!   domain/      → Entidades, value objects, ports (traits)
//!   infrastructure → Implementaciones: yt-dlp, ffmpeg, storage, cache
//!   common/      → Cross-cutting: config, error, constants
//!   startup/     → Bootstrap: DI, routes, server

// ============================================================
// MODULE DECLARATIONS
// ============================================================

pub mod api;
pub mod common;
pub mod domain;
pub mod infrastructure;
pub mod modules;
pub mod startup;

// ============================================================
// APP STATE — Compartido entre todos los handlers via Arc
// ============================================================

use std::sync::Arc;

use common::AppConfig;
use modules::extract::application::ExtractService;

/// Estado compartido de la aplicación — inyectado en todos los handlers.
#[derive(Clone)]
pub struct AppState {
    /// Configuración de la aplicación
    pub config: Arc<AppConfig>,
    /// Servicio de extracción de audio
    pub extract_service: Arc<ExtractService>,
}
