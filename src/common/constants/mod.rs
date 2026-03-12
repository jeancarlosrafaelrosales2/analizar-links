//! common/constants — Constantes de dominio para extracción de audio.

/// Duración máxima de video permitida (minutos)
pub const MAX_VIDEO_DURATION_MINUTES: u64 = 180;

/// Bitrate por defecto para MP3 (kbps)
pub const DEFAULT_MP3_BITRATE_KBPS: u32 = 320;

/// Bitrates permitidos (kbps)
pub const ALLOWED_BITRATES_KBPS: &[u32] = &[128, 192, 256, 320];

/// Formatos de audio soportados por defecto
pub const SUPPORTED_AUDIO_FORMATS: &[&str] = &["mp3", "m4a", "webm", "ogg", "flac"];

/// Formato de salida por defecto
pub const DEFAULT_OUTPUT_FORMAT: &str = "mp3";

/// Timeout para subprocesos externos (segundos)
pub const EXTERNAL_PROCESS_TIMEOUT_SECS: u64 = 300;

/// Prefijo de nombre de archivo de audio temporal
pub const TEMP_FILE_PREFIX: &str = "analizar_links_";

/// Dominios de video soportados
pub const SUPPORTED_DOMAINS: &[&str] = &[
    "youtube.com",
    "youtu.be",
    "www.youtube.com",
    "music.youtube.com",
];

/// Máximo de URLs en un batch request
pub const MAX_BATCH_URLS: usize = 40;

/// Intervalo de limpieza de archivos antiguos (segundos)
pub const CLEANUP_INTERVAL_SECS: u64 = 3600;
