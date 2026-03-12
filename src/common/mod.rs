//! common — Cross-cutting concerns
//!
//! Contiene: config, error, constants, response helpers.
//! REGLA: Nada en common/ puede importar de modules/ ni de infrastructure/.

pub mod config;
pub mod constants;
pub mod error;
pub mod response;

// Re-exports ergonómicos
pub use config::AppConfig;
pub use error::{AppError, ApiErrorResponse};
pub use response::ApiResponse;
