use std::path::Path;
use std::time::UNIX_EPOCH;

use rusqlite::{params, OptionalExtension};
use serde::Serialize;
use tauri::State;

use crate::archive::{require_db, ArchiveSlot};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaHealthRange {
    pub path: String,
    pub start_s: f64,
    pub end_s: Option<f64>,
    pub error_kind: String,
}

fn fingerprint(path: &Path) -> Result<(i64, i64), AppError> {
    let meta = std::fs::metadata(path)?;
    let size = i64::try_from(meta.len())
        .map_err(|_| AppError::Internal("media file size exceeds i64".into()))?;
    let modified_ms = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    Ok((size, modified_ms))
}

fn stored_path(path: &Path, archive_root: &Path) -> String {
    crate::paths::to_archive_relative(path, archive_root)
        .unwrap_or_else(|_| path.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn load_media_health(
    paths: Vec<String>,
    slot: State<'_, ArchiveSlot>,
) -> Result<Vec<MediaHealthRange>, AppError> {
    let db = require_db(&slot)?;
    let archive_root = db.archive_root().to_path_buf();
    let conn = db
        .lock()
        .map_err(|_| AppError::Internal("db mutex poisoned".into()))?;

    let mut out = Vec::new();
    for original in paths {
        let path = Path::new(&original);
        let Ok((size, modified_ms)) = fingerprint(path) else {
            continue;
        };
        let stored = stored_path(path, &archive_root);
        let mut stmt = conn.prepare(
            "SELECT start_ms, end_ms, error_kind
             FROM media_health_ranges
             WHERE file_path = ?1 AND file_size = ?2 AND modified_ms = ?3
             ORDER BY start_ms ASC",
        )?;
        let rows = stmt.query_map(params![stored, size, modified_ms], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, Option<i64>>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (start_ms, end_ms, error_kind) = row?;
            out.push(MediaHealthRange {
                path: original.clone(),
                start_s: start_ms as f64 / 1000.0,
                end_s: end_ms.map(|v| v as f64 / 1000.0),
                error_kind,
            });
        }
    }
    Ok(out)
}

#[tauri::command]
pub async fn record_media_failure(
    path: String,
    start_s: f64,
    error_kind: String,
    slot: State<'_, ArchiveSlot>,
) -> Result<(), AppError> {
    if !start_s.is_finite() || start_s < 0.0 {
        return Ok(());
    }

    let db = require_db(&slot)?;
    let archive_root = db.archive_root().to_path_buf();
    let path_obj = Path::new(&path);
    let (size, modified_ms) = fingerprint(path_obj)?;
    let stored = stored_path(path_obj, &archive_root);
    let start_ms = (start_s * 1000.0).round() as i64;
    let now = chrono::Utc::now().timestamp_millis();

    let conn = db
        .lock()
        .map_err(|_| AppError::Internal("db mutex poisoned".into()))?;

    let open_id: Option<i64> = conn
        .query_row(
            "SELECT id
             FROM media_health_ranges
             WHERE file_path = ?1 AND file_size = ?2 AND modified_ms = ?3
               AND end_ms IS NULL
             ORDER BY start_ms DESC
             LIMIT 1",
            params![stored, size, modified_ms],
            |r| r.get(0),
        )
        .optional()?;

    if let Some(id) = open_id {
        conn.execute(
            "UPDATE media_health_ranges
             SET start_ms = MIN(start_ms, ?2),
                 error_kind = ?3,
                 updated_at_ms = ?4
             WHERE id = ?1",
            params![id, start_ms, error_kind, now],
        )?;
    } else {
        conn.execute(
            "INSERT INTO media_health_ranges
                (file_path, file_size, modified_ms, start_ms, end_ms, error_kind, updated_at_ms)
             VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)",
            params![stored, size, modified_ms, start_ms, error_kind, now],
        )?;
    }
    Ok(())
}

#[tauri::command]
pub async fn record_media_recovery(
    path: String,
    end_s: f64,
    slot: State<'_, ArchiveSlot>,
) -> Result<(), AppError> {
    if !end_s.is_finite() || end_s < 0.0 {
        return Ok(());
    }

    let db = require_db(&slot)?;
    let archive_root = db.archive_root().to_path_buf();
    let path_obj = Path::new(&path);
    let (size, modified_ms) = fingerprint(path_obj)?;
    let stored = stored_path(path_obj, &archive_root);
    let end_ms = (end_s * 1000.0).round() as i64;
    let now = chrono::Utc::now().timestamp_millis();

    let conn = db
        .lock()
        .map_err(|_| AppError::Internal("db mutex poisoned".into()))?;

    let open: Option<(i64, i64)> = conn
        .query_row(
            "SELECT id, start_ms
             FROM media_health_ranges
             WHERE file_path = ?1 AND file_size = ?2 AND modified_ms = ?3
               AND end_ms IS NULL
             ORDER BY start_ms DESC
             LIMIT 1",
            params![stored, size, modified_ms],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;

    if let Some((id, start_ms)) = open {
        conn.execute(
            "UPDATE media_health_ranges
             SET end_ms = ?2, updated_at_ms = ?3
             WHERE id = ?1",
            params![id, end_ms.max(start_ms), now],
        )?;
    }
    Ok(())
}
