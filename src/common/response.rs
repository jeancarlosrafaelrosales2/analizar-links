//! common/response — Helpers para respuestas HTTP exitosas.

use serde::Serialize;

/// Envelope de respuesta exitosa canónica.
///
/// ```json
/// {
///   "success": true,
///   "data": { ... },
///   "message": "Operación exitosa"
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, data, message: None }
    }

    pub fn ok_with_message(data: T, message: impl Into<String>) -> Self {
        Self { success: true, data, message: Some(message.into()) }
    }
}
