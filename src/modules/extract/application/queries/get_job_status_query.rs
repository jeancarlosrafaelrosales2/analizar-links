//! application/queries/get_job_status_query — Query para obtener estado de un job.

use uuid::Uuid;
use crate::common::AppError;

/// Query CQRS: Obtener el estado actual de un job de extracción.
#[derive(Debug, Clone)]
pub struct GetJobStatusQuery {
    pub job_id: Uuid,
}

impl GetJobStatusQuery {
    /// Constructor con validación del UUID.
    pub fn new(id_str: &str) -> Result<Self, AppError> {
        let job_id = Uuid::parse_str(id_str).map_err(|_| {
            AppError::InvalidParam {
                field: "job_id".to_string(),
                reason: format!("'{}' no es un UUID válido", id_str),
            }
        })?;
        Ok(Self { job_id })
    }

    pub fn from_uuid(job_id: Uuid) -> Self {
        Self { job_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_uuid() {
        let q = GetJobStatusQuery::new("550e8400-e29b-41d4-a716-446655440000");
        assert!(q.is_ok());
    }

    #[test]
    fn test_invalid_uuid() {
        let q = GetJobStatusQuery::new("not-a-uuid");
        assert!(matches!(q, Err(AppError::InvalidParam { .. })));
    }
}
