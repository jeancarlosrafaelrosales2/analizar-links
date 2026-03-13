//! domain/ports/job_cache — Port para caché de jobs de extracción.

use crate::common::AppError;
use crate::domain::entities::ExtractJob;
use async_trait::async_trait;
use uuid::Uuid;

/// Port: Repositorio/caché de jobs de extracción.
///
/// Implementaciones: in-memory (DashMap) o Redis.
#[async_trait]
pub trait JobCache: Send + Sync {
    /// Guarda o actualiza un job en el caché.
    async fn set(&self, job: &ExtractJob) -> Result<(), AppError>;

    /// Obtiene un job por ID.
    async fn get(&self, id: Uuid) -> Option<ExtractJob>;

    /// Elimina un job del caché.
    async fn delete(&self, id: Uuid) -> Result<(), AppError>;

    /// Lista los IDs de todos los jobs activos.
    async fn list_active_ids(&self) -> Vec<Uuid>;
}
