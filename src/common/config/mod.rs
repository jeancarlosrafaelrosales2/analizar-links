//! common/config — Carga de configuración desde variables de entorno.
//!
//! Todos los valores se leen UNA sola vez al startup.
//! Defaults son dev-safe; override en producción vía env vars.

/// Configuración unificada de la aplicación.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Puerto donde escucha el servidor HTTP
    pub port: u16,
    /// Host bind address
    pub host: String,
    /// Orígenes CORS permitidos (comma-separated)
    pub cors_allowed_origins: String,
    /// URL de Redis (opcional — si None, usa caché en memoria)
    pub redis_url: Option<String>,
    /// Directorio de almacenamiento de archivos de audio
    pub storage_path: String,
    /// Path al binario yt-dlp
    pub ytdlp_path: String,
    /// Path al binario ffmpeg
    pub ffmpeg_path: String,
    /// Browser para cookies de yt-dlp (firefox, chrome, chromium, brave, etc.)
    pub ytdlp_browser: Option<String>,
    /// TTL en segundos para jobs cacheados
    pub job_ttl_seconds: u64,
    /// Rate limit: requests por segundo por IP
    pub rate_limit_rps: u64,
}

impl AppConfig {
    /// Carga configuración desde variables de entorno con defaults dev-safe.
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3100),
            host: std::env::var("HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            cors_allowed_origins: std::env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "*".to_string()),
            redis_url: std::env::var("REDIS_URL").ok(),
            storage_path: std::env::var("STORAGE_PATH")
                .unwrap_or_else(|_| "/tmp/analizar-links".to_string()),
            ytdlp_path: std::env::var("YTDLP_PATH")
                .unwrap_or_else(|_| "yt-dlp".to_string()),
            ffmpeg_path: std::env::var("FFMPEG_PATH")
                .unwrap_or_else(|_| "ffmpeg".to_string()),
            ytdlp_browser: std::env::var("YTDLP_BROWSER").ok(),
            job_ttl_seconds: std::env::var("JOB_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
            rate_limit_rps: std::env::var("RATE_LIMIT_RPS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        }
    }

    /// Defaults para tests — no requiere env vars.
    #[allow(dead_code)]
    pub fn test_defaults() -> Self {
        Self {
            port: 0,
            host: "127.0.0.1".to_string(),
            cors_allowed_origins: "http://localhost:3100".to_string(),
            redis_url: None,
            storage_path: "/tmp/analizar-links-test".to_string(),
            ytdlp_path: "yt-dlp".to_string(),
            ffmpeg_path: "ffmpeg".to_string(),
            ytdlp_browser: None,
            job_ttl_seconds: 60,
            rate_limit_rps: 100,
        }
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_no_panic() {
        let cfg = AppConfig::test_defaults();
        assert_eq!(cfg.port, 0);
        assert!(cfg.redis_url.is_none());
        assert_eq!(cfg.job_ttl_seconds, 60);
    }

    #[test]
    fn test_from_env_with_defaults() {
        // Limpiar vars para test reproducible
        std::env::remove_var("PORT");
        std::env::remove_var("REDIS_URL");
        let cfg = AppConfig::from_env();
        assert_eq!(cfg.port, 3100);
        assert!(cfg.redis_url.is_none());
    }
}
