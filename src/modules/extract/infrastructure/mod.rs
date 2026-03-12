//! modules/extract/infrastructure — Infraestructura específica del módulo.
//! Re-exporta desde infrastructure/ global.

pub mod persistence {
    // Por ahora el JobCache es global (DashMap).
    // Aquí iría un JobRepository con PostgreSQL si se necesita persistencia duradera.
    pub use crate::infrastructure::cache::MemoryJobCache;
}
