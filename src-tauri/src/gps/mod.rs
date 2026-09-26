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
/// v7: gives VIOFO boundary records neighbouring GPS evidence from the adjacent
///     segment when the embedded UTC cadence itself proves local continuity.
pub const GPS_PARSER_VERSION: i32 = 7;

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

/// Internal batch representation. VIOFO keeps its embedded UTC until the whole
/// selected trip has been decoded so a terminal record can be checked against
/// the first trustworthy records of the next segment. Other camera paths remain
/// byte-for-byte on their existing extraction route.
enum BatchGps {
    Plain(GpsBatchItem),
    Viofo {
        file_path: String,
        timed: Vec<viofo::TimedGpsPoint>,
    },
}

#[tauri::command]
pub async fn extract_gps_batch(requests: Vec<GpsRequest>) -> Result<Vec<GpsBatchItem>, AppError> {
    let mut extracted: Vec<BatchGps> = requests
        .par_iter()
        .map(|req| {
            let path = Path::new(&req.path);
            if matches!(req.camera_kind, CameraKind::Generic) && is_viofo_path(path) {
                let timed = extract_viofo_timed(path).unwrap_or_default();
                BatchGps::Viofo {
                    file_path: req.path.clone(),
                    timed,
                }
            } else {
                let points = extract_for_kind(path, req.camera_kind).unwrap_or_default();
                BatchGps::Plain(GpsBatchItem {
                    file_path: req.path.clone(),
                    points,
                })
            }
        })
        .collect();

    validate_viofo_batch_boundaries(&mut extracted);

    Ok(extracted
        .into_iter()
        .map(|item| match item {
            BatchGps::Plain(item) => item,
            BatchGps::Viofo { file_path, timed } => GpsBatchItem {
                file_path,
                points: timed.into_iter().map(|item| item.point).collect(),
            },
        })
        .collect())
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

fn is_viofo_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(crate::scan::viofo::is_viofo_filename)
        .unwrap_or(false)
}

fn extract_viofo_timed(path: &Path) -> Result<Vec<viofo::TimedGpsPoint>, AppError> {
    // The consistency validator needs the clip's measured duration only as an
    // upper bound on how far apart two UTC records from this same MP4 can be.
    // It does not use the filename clock or infer any timezone.
    let duration_s = crate::metadata::mp4_probe::probe(path)?.duration_s;
    let mut timed = viofo::extract_timed(path, Some(duration_s))?;
    let Some(first_gps_utc) = timed.first().map(|item| item.utc) else {
        return Ok(timed);
    };

    // Synchronization with the local camera clock remains a separate concern
    // from data validation. For now keep the existing best-effort civil-offset
    // inference, but feed it the first observation that survived consistency
    // validation so malformed raw records cannot become the timing anchor.
    match viofo_timing::first_fix_delay_s(path, first_gps_utc, duration_s) {
        Some(delay_s) if delay_s > 0.0 => {
            for item in &mut timed {
                item.point.t_offset_s += delay_s;
            }
        }
        _ => {}
    }

    Ok(timed)
}

fn extract_viofo(path: &Path) -> Result<Vec<GpsPoint>, AppError> {
    Ok(extract_viofo_timed(path)?
        .into_iter()
        .map(|item| item.point)
        .collect())
}

/// Validate only directly adjacent VIOFO batch entries. TripList submits the
/// selected trip's segment masters in segment order. Empty GPS segments are not
/// skipped here: without observations on both sides there is no boundary proof.
fn validate_viofo_batch_boundaries(batch: &mut [BatchGps]) {
    for index in 0..batch.len().saturating_sub(1) {
        let (left_slice, right_slice) = batch.split_at_mut(index + 1);
        let left = &mut left_slice[index];
        let right = &mut right_slice[0];

        let (
            BatchGps::Viofo {
                file_path: left_path,
                timed: left_timed,
            },
            BatchGps::Viofo {
                file_path: right_path,
                timed: right_timed,
            },
        ) = (left, right)
        else {
            continue;
        };

        validate_viofo_boundary(left_path, left_timed, right_path, right_timed);
    }
}

/// A file-end observation has no following neighbour inside its own MP4. The
/// next segment may supply that missing evidence, but only when GPS UTC itself
/// shows that the boundary belongs to the same local sampling sequence.
///
/// No fixed cadence is assumed. The cross-file UTC gap must be no larger than
/// the nearest positive sampling gap observed on either side of the boundary.
/// A delayed next fix or a real recording stop therefore disables the check.
fn validate_viofo_boundary(
    left_path: &str,
    left: &mut Vec<viofo::TimedGpsPoint>,
    right_path: &str,
    right: &mut Vec<viofo::TimedGpsPoint>,
) {
    if left.len() < 2 || right.len() < 2 || !boundary_has_local_utc_continuity(left, right) {
        return;
    }

    let left_last = left.len() - 1;
    let points = [&left[left_last - 1], &left[left_last], &right[0], &right[1]];
    let observations: Vec<validation::GpsObservation> = points
        .iter()
        .enumerate()
        .map(|(source_order, item)| boundary_observation(item, source_order))
        .collect();
    let validated = validation::validate(&observations, validation::ValidationContext::default());

    let drop_left = !validated[1].usable_timed_position();
    let drop_right = !validated[2].usable_timed_position();

    if drop_left {
        left.pop();
        eprintln!(
            "viofo gps: boundary validation excluded terminal route point in {left_path} using UTC evidence from {right_path}"
        );
    }
    if drop_right {
        right.remove(0);
        eprintln!(
            "viofo gps: boundary validation excluded initial route point in {right_path} using UTC evidence from {left_path}"
        );
    }
}

fn boundary_observation(
    item: &viofo::TimedGpsPoint,
    source_order: usize,
) -> validation::GpsObservation {
    // A zero speed is valid data, but after the first validation pass it can
    // also be the neutral UI fallback for an unusable speed/heading field. Do
    // not use such a value as an independent cross-file witness. This only
    // weakens the boundary proof; it never rejects a route point by itself.
    let moving = item.point.speed_mps > 0.0;
    validation::GpsObservation {
        source_order,
        time_ms: Some(item.utc.and_utc().timestamp_millis()),
        latitude: Some(item.point.lat),
        longitude: Some(item.point.lon),
        speed_mps: moving.then_some(item.point.speed_mps),
        heading_deg: moving.then_some(item.point.heading_deg),
    }
}

fn boundary_has_local_utc_continuity(
    left: &[viofo::TimedGpsPoint],
    right: &[viofo::TimedGpsPoint],
) -> bool {
    let left_times: Vec<i64> = left
        .iter()
        .map(|item| item.utc.and_utc().timestamp_millis())
        .collect();
    let right_times: Vec<i64> = right
        .iter()
        .map(|item| item.utc.and_utc().timestamp_millis())
        .collect();
    boundary_times_are_locally_continuous(&left_times, &right_times)
}

fn boundary_times_are_locally_continuous(left: &[i64], right: &[i64]) -> bool {
    if left.len() < 2 || right.len() < 2 {
        return false;
    }

    let cross_gap = right[0] - left[left.len() - 1];
    if cross_gap < 0 {
        return false;
    }

    let left_gap = left
        .windows(2)
        .rev()
        .find_map(|pair| positive_time_gap(pair[0], pair[1]));
    let right_gap = right
        .windows(2)
        .find_map(|pair| positive_time_gap(pair[0], pair[1]));

    match (left_gap, right_gap) {
        (Some(left_gap), Some(right_gap)) => cross_gap <= left_gap.max(right_gap),
        _ => false,
    }
}

fn positive_time_gap(a: i64, b: i64) -> Option<i64> {
    b.checked_sub(a).filter(|gap| *gap > 0)
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
            if is_viofo_path(path) {
                extract_viofo(path)
            } else {
                shenshu::extract(path)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timed(ms: i64, lon: f64, speed_mps: f64, heading_deg: f64) -> viofo::TimedGpsPoint {
        let utc = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(ms)
            .expect("valid test timestamp")
            .naive_utc();
        viofo::TimedGpsPoint {
            utc,
            point: GpsPoint {
                t_offset_s: ms as f64 / 1000.0,
                lat: 0.0,
                lon,
                speed_mps,
                heading_deg,
                altitude_m: 0.0,
                fix_ok: true,
            },
        }
    }

    #[test]
    fn boundary_cadence_is_derived_from_observed_utc() {
        assert!(boundary_times_are_locally_continuous(
            &[0, 2_000, 4_000],
            &[6_000, 8_000, 10_000]
        ));
        assert!(boundary_times_are_locally_continuous(
            &[0, 1_000, 1_000],
            &[2_000, 2_000, 3_000]
        ));
        assert!(!boundary_times_are_locally_continuous(
            &[0, 1_000, 2_000],
            &[60_000, 61_000]
        ));
    }

    #[test]
    fn boundary_context_rejects_corrupt_terminal_position() {
        let mut left = vec![
            timed(0, 0.0, 11.12, 90.0),
            timed(1_000, 0.0001, 11.12, 90.0),
            timed(2_000, 20.0, 0.0, 0.0),
        ];
        let mut right = vec![
            timed(3_000, 0.0003, 11.12, 90.0),
            timed(4_000, 0.0004, 11.12, 90.0),
        ];

        validate_viofo_boundary("left", &mut left, "right", &mut right);

        assert_eq!(left.len(), 2);
        assert_eq!(right.len(), 2);
        assert_eq!(left.last().unwrap().point.lon, 0.0001);
    }

    #[test]
    fn boundary_context_does_not_bridge_a_real_utc_pause() {
        let mut left = vec![
            timed(0, 0.0, 11.12, 90.0),
            timed(1_000, 0.0001, 11.12, 90.0),
            timed(2_000, 20.0, 0.0, 0.0),
        ];
        let mut right = vec![
            timed(60_000, 0.0003, 11.12, 90.0),
            timed(61_000, 0.0004, 11.12, 90.0),
        ];

        validate_viofo_boundary("left", &mut left, "right", &mut right);

        assert_eq!(left.len(), 3);
        assert_eq!(right.len(), 2);
    }
}
