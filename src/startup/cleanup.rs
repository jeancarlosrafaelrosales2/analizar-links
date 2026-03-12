//! startup/cleanup — Periodic cleanup task for old audio files.
//!
//! Runs as a background tokio task. Removes files older than JOB_TTL_SECONDS.

use std::path::Path;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Spawns a background task that periodically cleans old audio files.
///
/// Files older than `ttl_secs` are deleted. Runs every `interval_secs`.
pub fn spawn_cleanup_task(storage_path: String, ttl_secs: u64, interval_secs: u64) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
        info!(
            storage_path = %storage_path,
            ttl_secs = ttl_secs,
            interval_secs = interval_secs,
            "🧹 Cleanup task started"
        );

        loop {
            interval.tick().await;
            cleanup_old_files(&storage_path, ttl_secs).await;
        }
    });
}

/// Scans `storage_path` and removes files older than `ttl_secs`.
async fn cleanup_old_files(storage_path: &str, ttl_secs: u64) {
    let path = Path::new(storage_path);
    if !path.exists() {
        return;
    }

    let mut dir = match tokio::fs::read_dir(path).await {
        Ok(d) => d,
        Err(e) => {
            warn!(error = %e, "Cleanup: could not read storage dir");
            return;
        }
    };

    let now = std::time::SystemTime::now();
    let ttl = Duration::from_secs(ttl_secs);
    let mut removed = 0u32;

    while let Ok(Some(entry)) = dir.next_entry().await {
        let entry_path = entry.path();

        // Skip directories (like work/)
        if entry_path.is_dir() {
            continue;
        }

        // Only delete audio files
        let is_audio = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|ext| matches!(ext, "mp3" | "m4a" | "webm" | "ogg" | "flac"))
            .unwrap_or(false);

        if !is_audio {
            continue;
        }

        // Check file age
        if let Ok(metadata) = entry.metadata().await {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = now.duration_since(modified) {
                    if age > ttl {
                        debug!(file = %entry_path.display(), age_secs = age.as_secs(), "Removing old file");
                        if let Err(e) = tokio::fs::remove_file(&entry_path).await {
                            warn!(file = %entry_path.display(), error = %e, "Failed to remove old file");
                        } else {
                            removed += 1;
                        }
                    }
                }
            }
        }
    }

    if removed > 0 {
        info!(removed = removed, "🧹 Cleanup completed");
    }
}

// ============================================================
// TESTS
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_cleanup_removes_old_files() {
        let dir = "/tmp/analizar-links-cleanup-test";
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(dir).unwrap();

        // Create a fake old file
        let old_file = format!("{}/old-test.mp3", dir);
        fs::write(&old_file, b"fake audio").unwrap();

        // Set file time to 2 hours ago
        let two_hours_ago = std::time::SystemTime::now()
            - Duration::from_secs(7200);
        filetime::set_file_mtime(
            &old_file,
            filetime::FileTime::from_system_time(two_hours_ago),
        ).unwrap();

        // Create a recent file
        let new_file = format!("{}/new-test.mp3", dir);
        fs::write(&new_file, b"new audio").unwrap();

        // Cleanup with TTL of 1 hour
        cleanup_old_files(dir, 3600).await;

        // Old file should be gone, new file should remain
        assert!(!Path::new(&old_file).exists(), "Old file should be deleted");
        assert!(Path::new(&new_file).exists(), "New file should remain");

        let _ = fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn test_cleanup_skips_directories() {
        let dir = "/tmp/analizar-links-cleanup-test-dirs";
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(format!("{}/work", dir)).unwrap();

        cleanup_old_files(dir, 0).await;

        // work/ directory should still exist
        assert!(Path::new(&format!("{}/work", dir)).exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn test_cleanup_nonexistent_dir() {
        // Should not panic
        cleanup_old_files("/tmp/nonexistent-cleanup-test-9999", 3600).await;
    }
}
