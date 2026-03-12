//! infrastructure — Implementaciones concretas de los ports del dominio.
//!
//! REGLA: La infraestructura implementa traits de domain/ports/.
//! NUNCA expone sus tipos internos directamente a la capa application.

pub mod cache;
pub mod downloader;
pub mod storage;
pub mod transcoder;
