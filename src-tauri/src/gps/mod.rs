//! GPS extraction — dispatches to a brand-specific decoder based on
//! `CameraKind` since each dashcam stores GPS in its own proprietary layout.

pub mod miltona;
pub mod shenshu;
pub mod validation;
pub mod viofo;
pub mod viofo_timing;

use crate::archive::{require_db, ArchiveSlot};
use crate::error::AppError;
use crate::model::{GpsBatchItem, GpsPoint};
use crate::scan::naming::CameraKind;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::Path;
use tauri::State;

/// Bump when GPS decoders change semantics so previously archived GPS becomes
/// stale. The encoder's `has_current` probe and the startup backfill both compare
/// against this; rows below the current version are re-extracted on the next
/// encode (or backfill pass) when the original MP4 is still on disk.
///
/// v2: trip-stitched GPS trims each segment's points to video duration.
/// v3: adds VIOFO / Novatek GPS extraction for A229-family footage and fixes
///     generic VIOFO files being sent through the Wolf Box ShenShu decoder.
/// v4: filters isolated, physically impossible VIOFO position spikes while
///     preserving each file's original GPS time axis.
/// v5: aligns VIOFO UTC GPS timestamps to the clip's filename start time so a
///     delayed first GPS fix remains delayed on the video timeline.
/// v6: validates VIOFO observations in physical source order with the reusable
///     camera-neutral consistency layer before they can become route anchors.
pub const GPS_PARSER_VERSION: i32 = 6;

/// A single path plus the camera brand the scanner identified for it. The
/// frontend builds one of these per segment (by pairing each master channel's
/// file path with its segment's `cameraKind`) and submits them in a batch.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpsRequest {
    pub path: String,
    pub camera_kind: CameraKind,
}

#[tauri::command]
pub async fn extract_gps(path: String, camera_kind: CameraKind) -> Result<Vec<GpsPoint>, AppError> {
    extract_for_kind(Path::new(&path), camera_kind)
}

#[tauri::command]
pub async fn extract_gps_batch(requests: Vec<GpsRequest>) -> Result<Vec<GpsBatchItem>, AppError> {
    let results: Vec<GpsBatchItem> = requests
        .par_iter()
        .map(|req| {
            let points =
                extract_for_kind(Path::new(&req.path), req.camera_kind).unwrap_or_default();
            GpsBatchItem {
                file_path: req.path.clone(),
                points,
            }
        })
        .collect();
    Ok(results)
}

/// Load archived trip-stitched GPS from the DB. Returns an empty vec
/// when no row exists for the trip — the frontend treats that as the
/// signal to fall back to the per-segment `extract_gps_batch` path
/// (which only succeeds when originals are still on disk).
#[tauri::command]
pub async fn load_trip_gps(
    trip_id: String,
    slot: State<'_, ArchiveSlot>,
) -> Result<Vec<GpsPoint>, AppError> {
    let db = require_db(&slot)?;
    let conn = db
        .lock()
        .map_err(|_| AppError::Internal("db mutex poisoned".into()))?;
    Ok(crate::db::trip_gps::load(&conn, &trip_id)?.unwrap_or_default())
}

/// Write a diagnostic dump of a Miltona file's `gps0` atom. Used by the
/// "Export GPS debug" UI button to collect ground-truth samples while the
/// lat/lon encoding is still being finalized.
#[tauri::command]
pub async fn dump_miltona_gps_debug(path: String) -> Result<String, AppError> {
    let out = miltona::dump_debug(Path::new(&path))?;
    Ok(out.to_string_lossy().into_owned())
}

fn extract_viofo(path: &Path) -> Result<Vec<GpsPoint>, AppError> {
    // The consistency validator needs the clip's measured duration only as an
    // upper bound on how far apart two UTC records from this same MP4 can be.
    // It does not use the filename clock or infer any timezone.
    let duration_s = crate::metadata::mp4_probe::probe(path)?.duration_s;
    let timed = viofo::extract_timed(path, Some(duration_s))?;
    if timed.is_empty() {
        return Ok(vec![]);
    }

    let first_gps_utc = timed[0].utc;
    let mut points: Vec<GpsPoint> = timed.into_iter().map(|item| item.point).collect();

    // Synchronization with the local camera clock remains a separate concern
    // from data validation. For now keep the existing best-effort civil-offset
    // inference, but feed it the first observation that survived consistency
    // validation so malformed raw records cannot become the timing anchor.
    match viofo_timing::first_fix_delay_s(path, first_gps_utc, duration_s) {
        Some(delay_s) if delay_s > 0.0 => {
            for point in &mut points {
                point.t_offset_s += delay_s;
            }
        }
        _ => {}
    }

    Ok(points)
}

pub fn extract_for_kind(path: &Path, kind: CameraKind) -> Result<Vec<GpsPoint>, AppError> {
    match kind {
        CameraKind::WolfBox => shenshu::extract(path),
        CameraKind::Miltona => miltona::extract(path),
        // Thinkware: no GPS decoder (the sample we have contains no GPS
        // data at all). If a GPS-equipped Thinkware model turns up, add a
        // decoder and flip `CameraKind::gps_supported` for that variant.
        CameraKind::Thinkware => Ok(vec![]),
        // VIOFO is currently represented by the persisted Generic enum value
        // to avoid a DB/frontend migration. Distinguish it by its specific
        // A229-family filename before using the historical generic fallback.
        CameraKind::Generic => {
            let is_viofo = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(crate::scan::viofo::is_viofo_filename)
                .unwrap_or(false);
            if is_viofo {
                extract_viofo(path)
            } else {
                shenshu::extract(path)
            }
        }
    }
}
