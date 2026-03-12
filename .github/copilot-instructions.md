# 🎵 analizar-links — YouTube Audio Extraction API

## Guía para AI Agents & Desarrolladores

---

## 📊 ESTADO ACTUAL

| Categoría         | Valor                                                         |
| ----------------- | ------------------------------------------------------------- |
| **Stack**         | Rust 1.93+ / Axum 0.7 / Tokio                                |
| **Arquitectura**  | Clean Architecture DDD (inspirada en simple_api_gateway)      |
| **Tests**         | 37 lib tests, 0 failed                                        |
| **Warnings**      | 4 (unused imports menores — sin impacto funcional)            |
| **Binary**        | `analizar-links` (port 3000)                                  |
| **Source Path**   | `src/`                                                        |

---

## 🏗️ ARQUITECTURA

### Capas (top → bottom — dependencias van solo hacia abajo)

```
src/
├── api/              # HTTP global: health, middleware (CORS, logging)
├── modules/          # Bounded contexts
│   └── extract/      # Único BC: extracción de audio
│       ├── api/      # controllers/ + routes/
│       ├── application/ # commands/ + queries/ + dtos/ + service.rs
│       ├── domain/   # Re-exports desde domain/ global
│       └── infrastructure/ # Re-exports desde infrastructure/ global
├── domain/           # Tipos puros: entities, value_objects, events, ports (traits)
├── infrastructure/   # Implementaciones concretas de ports
│   ├── cache/        # MemoryJobCache (DashMap)
│   ├── downloader/   # YtDlpDownloader (subprocess)
│   ├── transcoder/   # FfmpegTranscoder (subprocess)
│   └── storage/      # LocalAudioStorage (filesystem)
├── common/           # Cross-cutting: config, error, constants, response
├── startup/          # Bootstrap: DI, routes, server
├── lib.rs            # AppState + crate root
└── main.rs           # Entry point
```

### Module Internal Structure (Extract BC — DDD 8-depth)

```
modules/extract/
├── api/
│   ├── controllers/extract_controller.rs   # Handlers thin (POST + GET)
│   └── routes/extract_routes.rs            # URL → handler mapping
├── application/
│   ├── commands/extract_audio_command.rs   # CQRS write: ExtractAudioCommand
│   ├── queries/get_job_status_query.rs     # CQRS read: GetJobStatusQuery
│   ├── dtos/{requests,responses}.rs        # HTTP DTOs
│   └── service.rs                          # ExtractService (orquesta pipeline)
├── domain/mod.rs                           # Re-exports desde domain/ global
└── infrastructure/mod.rs                  # Re-exports desde infrastructure/ global
```

---

## 🔴 REGLAS INQUEBRANTABLES

### REGLA 1: Route Namespace Ownership
El módulo `extract` **SOLO** registra rutas en estos namespaces:
- `/api/v1/extract/*`
- `/api/v1/audio/*`
- `/health`, `/health/*`

### REGLA 2: Domain Purity
`src/domain/` **NUNCA** importa axum, reqwest, serde(+proc macro), sqlx.
Solo tipos primitivos, chrono, uuid y async-trait.

### REGLA 3: Error Canónico
Todo handler DEBE retornar `Result<T, AppError>`.
NUNCA `(StatusCode, String)` ni `String` directo.

### REGLA 4: Thin Controllers
Los handlers en `api/controllers/` SOLO:
1. Deserializan input
2. Construyen Command/Query
3. Llaman al service
4. Convierten a HTTP response

### REGLA 5: Pipeline Async
`ExtractService::submit_extraction()` retorna job_id inmediatamente.
El procesamiento real ocurre en `tokio::spawn` — fire-and-forget.

### REGLA 6: Tests No Decrece
El count de tests **nunca** baja de 37. Cada nuevo feature agrega ≥1 test.

---

## 🛠️ COMANDOS

```bash
# Instalar dependencias externas (OBLIGATORIO para funcionar)
pip install yt-dlp           # Descargador de video
sudo apt-get install ffmpeg  # Transcodificador de audio

# Desarrollo
cp .env.example .env         # Configurar entorno
cargo run                    # Arrancar servidor (port 3000)

# Tests
cargo test --lib             # 37 unit tests
cargo check                  # Compile check

# Verificación post-cambio
cargo check && cargo test --lib && echo "✅ OK"

# Smoke test (con servidor corriendo)
curl http://localhost:3000/health
curl -X POST http://localhost:3000/api/v1/extract \
     -H "Content-Type: application/json" \
     -d '{"url": "https://youtu.be/dQw4w9WgXcQ", "format": "mp3", "bitrate": 192}'
```

---

## 📡 API ENDPOINTS

| Método | Endpoint                          | Descripción                     |
|--------|-----------------------------------|---------------------------------|
| GET    | `/health`                         | Liveness probe                  |
| GET    | `/health/ready`                   | Readiness probe                 |
| POST   | `/api/v1/extract`                 | Encolar job de extracción       |
| GET    | `/api/v1/extract/{job_id}/status` | Consultar estado de job         |
| GET    | `/api/v1/extract/jobs`            | Listar jobs activos             |
| GET    | `/api/v1/audio/{filename}`        | Descargar audio procesado       |

### Flujo Típico

```
POST /api/v1/extract
  { "url": "https://youtu.be/...", "format": "mp3", "bitrate": 192 }
  → 202 { "job_id": "uuid", "status": "pending", "poll_url": "..." }

GET /api/v1/extract/{job_id}/status
  → 200 { "status": "downloading", "progress_pct": 45, ... }

GET /api/v1/extract/{job_id}/status
  → 200 { "status": "completed", "download_url": "http://...audio/uuid.mp3", ... }

GET /api/v1/audio/{uuid.mp3}
  → 200 audio/mpeg (descarga del archivo)
```

---

## 🔧 VARIABLES DE ENTORNO

```env
PORT=3000                                    # Puerto del servidor
HOST=0.0.0.0                                 # Bind address
CORS_ALLOWED_ORIGINS=http://localhost:3000   # Orígenes CORS
STORAGE_PATH=/tmp/analizar-links             # Directorio de audio
YTDLP_PATH=yt-dlp                           # Path a yt-dlp
FFMPEG_PATH=ffmpeg                           # Path a ffmpeg
JOB_TTL_SECONDS=3600                        # TTL de jobs en cache
RUST_LOG=analizar_links=debug               # Nivel de logs
```

---

## 📐 FLUJO PARA AI AGENTS

```
1. Leer este archivo (arquitectura + reglas)
2. Identificar la capa a modificar:
   - Nueva fuente de video → domain/ports/video_downloader.rs + nueva infra
   - Nuevo formato de audio → domain/value_objects/audio_format.rs
   - Nuevo endpoint → modules/extract/api/controllers/ + routes/
   - Lógica de negocio → modules/extract/application/service.rs
3. Pre-edit checklist:
   □ ¿Mi cambio respeta namespace /api/v1/extract/* ? (Regla 1)
   □ ¿domain/ no importa infra? (Regla 2)
   □ ¿Retorno AppError? (Regla 3)
   □ ¿El handler es thin? (Regla 4)
   □ ¿Agregué ≥1 test? (Regla 6)
   □ cargo check && cargo test --lib → 0 errors, 0 failures
```

---

## 🗺️ ROADMAP / FEATURES PENDIENTES

- [ ] **Caché Redis** — Activar feature `redis-cache` para persistencia entre reinicios
- [ ] **S3 Storage** — Reemplazar `LocalAudioStorage` con `S3AudioStorage`
- [ ] **Múltiples plataformas** — Ampliar `SUPPORTED_DOMAINS` con Vimeo, SoundCloud
- [ ] **Progress WebSocket** — Stream de progreso en tiempo real
- [ ] **API Key auth** — Middleware de autenticación básica
- [ ] **OpenAPI/Swagger UI** — Endpoint `/api-docs` con `utoipa`
- [ ] **Docker** — Dockerfile multi-stage con yt-dlp + ffmpeg incluidos
- [ ] **Cleanup job** — Tokio task periódico para eliminar archivos antiguos

---

**Última Actualización:** 11 Marzo 2026 | **Tests:** 37 | **Warnings:** 4 | **Errores:** 0
