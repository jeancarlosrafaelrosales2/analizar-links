//! infrastructure/cache/memory_cache — JobCache implementado con DashMap.
//!
//! Thread-safe, lock-free concurrent HashMap.
//! Para producción con múltiples instancias, usar Redis (feature `redis-cache`).

use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::common::AppError;
use crate::domain::entities::ExtractJob;
use crate::domain::ports::JobCache;

/// Implementación in-memory del JobCache usando DashMap.
///
/// Lifecycle: Los jobs viven en memoria mientras el proceso esté activo.
/// No hay persistencia entre reinicios — usar Redis para eso.
pub struct MemoryJobCache {
    store: Arc<DashMap<Uuid, ExtractJob>>,
}

impl MemoryJobCache {
    pub fn new() -> Self {
        Self {
            store: Arc::new(DashMap::new()),
        }
    }

    /// Retorna el número de jobs en caché (útil para métricas).
    pub fn len(&self) -> usize {
        self.store.len()
    }
}

impl Default for MemoryJobCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl JobCache for MemoryJobCache {
    async fn set(&self, job: &ExtractJob) -> Result<(), AppError> {
        self.store.insert(job.id, job.clone());
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Option<ExtractJob> {
        self.store.get(&id).map(|entry| entry.value().clone())
    }

    async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        self.store.remove(&id);
        Ok(())
    }

    async fn list_active_ids(&self) -> Vec<Uuid> {
        self.store
            .iter()
            .filter(|entry| !entry.value().is_done())
            .map(|entry| *entry.key())
            .collect()
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{AudioFormat, VideoUrl};

    fn make_job() -> ExtractJob {
        let url = VideoUrl::parse("https://youtu.be/dQw4w9WgXcQ").unwrap();
        ExtractJob::new(url, AudioFormat::Mp3, 192)
    }

    #[tokio::test]
    async fn test_set_and_get() {
        let cache = MemoryJobCache::new();
        let job = make_job();
        let id = job.id;

        cache.set(&job).await.unwrap();
        let retrieved = cache.get(id).await;

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, id);
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let cache = MemoryJobCache::new();
        let result = cache.get(Uuid::new_v4()).await;
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete() {
        let cache = MemoryJobCache::new();
        let job = make_job();
        let id = job.id;

        cache.set(&job).await.unwrap();
        assert!(cache.get(id).await.is_some());

        cache.delete(id).await.unwrap();
        assert!(cache.get(id).await.is_none());
    }

    #[tokio::test]
    async fn test_list_active_ids_excludes_done() {
        let cache = MemoryJobCache::new();

        let mut active_job = make_job();
        active_job.start_downloading();

        let mut done_job = make_job();
        done_job.complete("/tmp/out.mp3");

        cache.set(&active_job).await.unwrap();
        cache.set(&done_job).await.unwrap();

        let active_ids = cache.list_active_ids().await;
        assert_eq!(active_ids.len(), 1);
        assert_eq!(active_ids[0], active_job.id);
    }
}
