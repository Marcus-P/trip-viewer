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
/// v8: caches trip GPS for ordinary playback and can choose a healthy sibling
///     camera as the GPS source for each segment instead of assuming Front.
pub const GPS_PARSER_VERSION: i32 = 8;

/// A single path plus the camera brand the scanner identified for it. The
/// frontend builds one of these per segment (by pairing each master channel's
/// file path with its segment's `cameraKind`) and submits them in a batch.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GpsRequest {
    pub path: String,
    pub camera_kind: CameraKind,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripGpsChannelRequest {
    pub label: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TripGpsSegmentRequest {
    pub segment_id: String,
    pub duration_s: f64,
    pub camera_kind: CameraKind,
    pub channels: Vec<TripGpsChannelRequest>,
}

#[derive(Debug, Clone)]
pub(crate) struct TripGpsSegmentSource {
    pub duration_s: f64,
    pub camera_kind: CameraKind,
    pub paths: Vec<String>,
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

fn extract_batch_item(path: &str, camera_kind: CameraKind) -> BatchGps {
    let path_obj = Path::new(path);
    if matches!(camera_kind, CameraKind::Generic) && is_viofo_path(path_obj) {
        let timed = extract_viofo_timed(path_obj).unwrap_or_default();
        BatchGps::Viofo {
            file_path: path.to_string(),
            timed,
        }
    } else {
        let points = extract_for_kind(path_obj, camera_kind).unwrap_or_default();
        BatchGps::Plain(GpsBatchItem {
            file_path: path.to_string(),
            points,
        })
    }
}

fn batch_quality(item: &BatchGps, duration_s: f64) -> (usize, i64) {
    let offsets: Vec<f64> = match item {
        BatchGps::Plain(item) => item
            .points
            .iter()
            .filter(|p| p.fix_ok && p.t_offset_s >= 0.0 && p.t_offset_s <= duration_s)
            .map(|p| p.t_offset_s)
            .collect(),
        BatchGps::Viofo { timed, .. } => timed
            .iter()
            .filter(|p| {
                p.point.fix_ok && p.point.t_offset_s >= 0.0 && p.point.t_offset_s <= duration_s
            })
            .map(|p| p.point.t_offset_s)
            .collect(),
    };
    if offsets.is_empty() {
        return (0, 0);
    }
    let span_ms = ((offsets[offsets.len() - 1] - offsets[0]).max(0.0) * 1000.0) as i64;
    (offsets.len(), span_ms)
}

fn batch_into_points(item: BatchGps) -> Vec<GpsPoint> {
    match item {
        BatchGps::Plain(item) => item.points,
        BatchGps::Viofo { timed, .. } => timed.into_iter().map(|item| item.point).collect(),
    }
}

/// Extract and stitch one trip while allowing each segment to pick the camera
/// file with the strongest usable GPS trace. Candidate order is the tie-breaker,
/// so a healthy canonical Front remains preferred when all cameras are equal.
pub(crate) fn extract_best_trip_gps(segments: &[TripGpsSegmentSource]) -> Vec<GpsPoint> {
    let mut selected: Vec<(f64, BatchGps)> = Vec::with_capacity(segments.len());

    for segment in segments {
        let mut best: Option<((usize, i64), BatchGps)> = None;
        for path in &segment.paths {
            let item = extract_batch_item(path, segment.camera_kind);
            let quality = batch_quality(&item, segment.duration_s);
            if best.as_ref().map(|(q, _)| quality > *q).unwrap_or(true) {
                best = Some((quality, item));
            }
        }
        if let Some((_, item)) = best {
            selected.push((segment.duration_s, item));
        } else {
            selected.push((
                segment.duration_s,
                BatchGps::Plain(GpsBatchItem {
                    file_path: String::new(),
                    points: Vec::new(),
                }),
            ));
        }
    }

    let mut batch_only: Vec<BatchGps> = selected.into_iter().map(|(_, item)| item).collect();
    validate_viofo_batch_boundaries(&mut batch_only);

    let mut out = Vec::new();
    let mut cursor = 0.0;
    for (segment, item) in segments.iter().zip(batch_only.into_iter()) {
        for p in batch_into_points(item) {
            if p.t_offset_s < 0.0 || p.t_offset_s > segment.duration_s {
                continue;
            }
            out.push(GpsPoint {
                t_offset_s: cursor + p.t_offset_s,
                ..p
            });
        }
        cursor += segment.duration_s;
    }
    out
}

#[tauri::command]
pub async fn extract_gps_batch(requests: Vec<GpsRequest>) -> Result<Vec<GpsBatchItem>, AppError> {
    let mut extracted: Vec<BatchGps> = requests
        .par_iter()
        .map(|req| extract_batch_item(&req.path, req.camera_kind))
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

/// Return cached trip GPS when current; otherwise extract it once from the
/// supplied segment/channel candidates, persist it in trip_gps, and return it.
/// force=true is used after a runtime media failure so a known-bad camera can
/// be excluded for that segment and the persisted track can heal immediately.
#[tauri::command]
pub async fn ensure_trip_gps(
    trip_id: String,
    segments: Vec<TripGpsSegmentRequest>,
    force: Option<bool>,
    slot: State<'_, ArchiveSlot>,
) -> Result<Vec<GpsPoint>, AppError> {
    let db = require_db(&slot)?;
    tauri::async_runtime::spawn_blocking(move || {
        ensure_trip_gps_unlocked(
            &db,
            &trip_id,
            &segments,
            force.unwrap_or(false),
            extract_best_trip_gps,
        )
    })
    .await
    .map_err(|e| AppError::Internal(format!("GPS worker failed: {e}")))?
}

struct GpsWork {
    signature: String,
    sources: Vec<TripGpsSegmentSource>,
    available: Vec<bool>,
    cached: Option<Vec<GpsPoint>>,
}

enum GpsPreparation {
    Cached(Vec<GpsPoint>),
    Extract(GpsWork),
}

fn prepare_trip_gps(
    conn: &rusqlite::Connection,
    archive_root: &Path,
    trip_id: &str,
    segments: &[TripGpsSegmentRequest],
    force: bool,
) -> Result<GpsPreparation, AppError> {
    use crate::db::trip_gps;
    let cached = trip_gps::load(conn, trip_id)?;
    let (signature, sources, available) = trip_gps_sources(conn, archive_root, segments)?;
    if !available.iter().any(|exists| *exists) {
        return Ok(GpsPreparation::Cached(cached.unwrap_or_default()));
    }
    if !force
        && trip_gps::has_current(conn, trip_id, GPS_PARSER_VERSION)?
        && trip_gps::source_signature(conn, trip_id)?.as_deref() == Some(&signature)
    {
        return Ok(GpsPreparation::Cached(cached.unwrap_or_default()));
    }
    Ok(GpsPreparation::Extract(GpsWork {
        signature,
        sources,
        available,
        cached,
    }))
}

/// Heavy MP4 I/O must run outside both the async IPC executor and the archive
/// mutex: segment health reads/writes and transport must remain responsive.
fn ensure_trip_gps_unlocked(
    db: &crate::db::DbHandle,
    trip_id: &str,
    segments: &[TripGpsSegmentRequest],
    force: bool,
    mut extract: impl FnMut(&[TripGpsSegmentSource]) -> Vec<GpsPoint>,
) -> Result<Vec<GpsPoint>, AppError> {
    for attempt in 0..3 {
        let preparation = {
            let conn = db
                .lock()
                .map_err(|_| AppError::Internal("db mutex poisoned".into()))?;
            prepare_trip_gps(
                &conn,
                db.archive_root(),
                trip_id,
                segments,
                force && attempt == 0,
            )?
        };
        let work = match preparation {
            GpsPreparation::Cached(points) => return Ok(points),
            GpsPreparation::Extract(work) => work,
        };
        let points = extract(&work.sources);
        let conn = db
            .lock()
            .map_err(|_| AppError::Internal("db mutex poisoned".into()))?;
        // Health or fingerprints may change while parsing. Never publish a
        // result under provenance that no longer describes its input files.
        if trip_gps_sources(&conn, db.archive_root(), segments)?.0 != work.signature {
            continue;
        }
        return persist_trip_gps(&conn, trip_id, work, points);
    }
    Err(AppError::Internal(
        "GPS sources changed during preparation; select the trip again to retry".into(),
    ))
}

fn persist_trip_gps(
    conn: &rusqlite::Connection,
    trip_id: &str,
    work: GpsWork,
    mut points: Vec<GpsPoint>,
) -> Result<Vec<GpsPoint>, AppError> {
    use crate::db::trip_gps;
    let GpsWork {
        signature,
        sources,
        available,
        cached,
    } = work;
    // Preserve archived spans in mixed trips too. Empty/no-GPS segments still
    // contribute their duration, so later segments keep their trip time axis.
    if let Some(cached) = cached {
        let mut cursor = 0.0;
        for (index, source) in sources.iter().enumerate() {
            let end = cursor + source.duration_s;
            if !available[index] {
                points.retain(|p| p.t_offset_s < cursor || p.t_offset_s >= end);
                points.extend(
                    cached
                        .iter()
                        .filter(|p| p.t_offset_s >= cursor && p.t_offset_s < end)
                        .cloned(),
                );
            }
            cursor = end;
        }
        points.sort_by(|a, b| a.t_offset_s.total_cmp(&b.t_offset_s));
    }
    // Atomically publish both payload and provenance; a failed statement must
    // not leave a new payload carrying an old signature.
    let tx = conn.unchecked_transaction()?;
    trip_gps::upsert(&tx, trip_id, &points, GPS_PARSER_VERSION)?;
    tx.execute(
        "UPDATE trip_gps SET source_signature = ?2 WHERE trip_id = ?1",
        rusqlite::params![trip_id, signature],
    )?;
    tx.commit()?;
    Ok(points)
}

#[cfg(test)]
fn ensure_trip_gps_cached(
    conn: &rusqlite::Connection,
    root: &Path,
    trip_id: &str,
    segments: &[TripGpsSegmentRequest],
    force: bool,
) -> Result<Vec<GpsPoint>, AppError> {
    match prepare_trip_gps(conn, root, trip_id, segments, force)? {
        GpsPreparation::Cached(points) => Ok(points),
        GpsPreparation::Extract(work) => {
            let points = extract_best_trip_gps(&work.sources);
            persist_trip_gps(conn, trip_id, work, points)
        }
    }
}

fn trip_gps_sources(
    conn: &rusqlite::Connection,
    archive_root: &Path,
    segments: &[TripGpsSegmentRequest],
) -> Result<(String, Vec<TripGpsSegmentSource>, Vec<bool>), AppError> {
    use crate::media_health;
    let mut signature = Vec::new();
    let mut sources = Vec::new();
    let mut available = Vec::new();
    for segment in segments {
        let mut paths = Vec::new();
        let mut has_original = false;
        let mut files = Vec::new();
        for channel in &segment.channels {
            let path = Path::new(&channel.path);
            let stored = media_health::stored_path(path, archive_root);
            let fingerprint = media_health::fingerprint(path).ok();
            let mut ranges = Vec::<(i64, Option<i64>)>::new();
            if let Some((size, modified)) = fingerprint {
                has_original = true;
                let mut stmt = conn.prepare(
                    "SELECT start_ms, end_ms FROM media_health_ranges
                     WHERE file_path = ?1 AND file_size = ?2 AND modified_ms = ?3
                     ORDER BY start_ms, end_ms",
                )?;
                ranges = stmt
                    .query_map(rusqlite::params![stored, size, modified], |r| {
                        Ok((r.get(0)?, r.get(1)?))
                    })?
                    .collect::<Result<_, _>>()?;
                // Even partial video corruption makes a sibling the safer GPS
                // provider for this segment. Never assume GPS implies health.
                if ranges.is_empty() {
                    paths.push(channel.path.clone());
                }
            }
            files.push((stored, fingerprint, ranges));
        }
        signature.push(serde_json::json!([
            segment.segment_id,
            segment.duration_s,
            segment.camera_kind,
            files
        ]));
        available.push(has_original);
        sources.push(TripGpsSegmentSource {
            duration_s: segment.duration_s,
            camera_kind: segment.camera_kind,
            paths,
        });
    }
    let signature = serde_json::to_string(&signature)
        .map_err(|e| AppError::Internal(format!("GPS source signature: {e}")))?;
    Ok((signature, sources, available))
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

    fn cache_fixture(root: &Path) -> (crate::db::DbHandle, Vec<TripGpsSegmentRequest>) {
        let db = crate::db::open_in_memory_with_root(root).unwrap();
        db.lock().unwrap().execute(
            "INSERT INTO trips (id, start_time_ms, end_time_ms, camera_kind, gps_supported, last_seen_ms)
             VALUES ('trip', 0, 60000, 'generic', 1, 0)", []).unwrap();
        let channels = ["front", "interior", "rear"]
            .into_iter()
            .map(|label| {
                let path = root.join(format!("{label}.mp4"));
                std::fs::write(&path, b"not an MP4").unwrap();
                TripGpsChannelRequest {
                    label: label.into(),
                    path: path.to_string_lossy().into(),
                }
            })
            .collect();
        (
            db,
            vec![TripGpsSegmentRequest {
                segment_id: "seg".into(),
                duration_s: 60.0,
                camera_kind: CameraKind::Generic,
                channels,
            }],
        )
    }

    fn seed_cached(conn: &rusqlite::Connection, root: &Path, segments: &[TripGpsSegmentRequest]) {
        let signature = trip_gps_sources(conn, root, segments).unwrap().0;
        crate::db::trip_gps::upsert(
            conn,
            "trip",
            &[timed(1000, 1.0, 1.0, 0.0).point],
            GPS_PARSER_VERSION,
        )
        .unwrap();
        conn.execute(
            "UPDATE trip_gps SET source_signature = ?1 WHERE trip_id = 'trip'",
            [signature],
        )
        .unwrap();
    }

    #[test]
    fn health_reads_and_writes_continue_during_gps_extraction_and_stale_result_is_retried() {
        use std::sync::mpsc;
        use std::time::Duration;
        let root = tempfile::tempdir().unwrap();
        let (db, segments) = cache_fixture(root.path());
        let (started_tx, started_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let worker_db = db.clone();
        let worker_segments = segments.clone();
        let worker = std::thread::spawn(move || {
            let mut calls = 0;
            let result =
                ensure_trip_gps_unlocked(&worker_db, "trip", &worker_segments, false, |sources| {
                    calls += 1;
                    if calls == 1 {
                        assert_eq!(sources[0].paths.len(), 3);
                        started_tx.send(()).unwrap();
                        resume_rx.recv_timeout(Duration::from_secs(5)).unwrap();
                    } else {
                        assert_eq!(sources[0].paths.len(), 2);
                    }
                    vec![timed(1000, calls as f64, 1.0, 0.0).point]
                })
                .unwrap();
            (calls, result)
        });
        started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let (health_tx, health_rx) = mpsc::channel();
        let health_db = db.clone();
        let front = segments[0].channels[0].path.clone();
        let health = std::thread::spawn(move || {
            let conn = health_db.lock().unwrap();
            let path = Path::new(&front);
            let (size, modified) = crate::media_health::fingerprint(path).unwrap();
            conn.execute("INSERT INTO media_health_ranges (file_path, file_size, modified_ms, start_ms, error_kind, updated_at_ms)
                VALUES (?1, ?2, ?3, 1000, 'decode', 0)",
                rusqlite::params![crate::media_health::stored_path(path, health_db.archive_root()), size, modified]).unwrap();
            health_tx.send(()).unwrap();
        });
        let responsive = health_rx.recv_timeout(Duration::from_secs(2)).is_ok();
        resume_tx.send(()).unwrap();
        let (calls, result) = worker.join().unwrap();
        health.join().unwrap();
        assert!(responsive, "GPS parsing must not hold the archive DB mutex");
        assert_eq!(calls, 2, "health changed while parsing; discard stale GPS");
        assert_eq!(result[0].lon, 2.0);
        let conn = db.lock().unwrap();
        assert_eq!(
            crate::db::trip_gps::load(&conn, "trip").unwrap().unwrap()[0].lon,
            2.0
        );
        assert_eq!(
            crate::db::trip_gps::source_signature(&conn, "trip")
                .unwrap()
                .unwrap(),
            trip_gps_sources(&conn, root.path(), &segments).unwrap().0
        );
    }

    #[test]
    fn current_cache_survives_restart_without_reading_mp4_contents() {
        let root = tempfile::tempdir().unwrap();
        let (db, segments) = cache_fixture(root.path());
        let conn = db.lock().unwrap();
        seed_cached(&conn, root.path(), &segments);
        // The files contain invalid MP4 bytes. Re-extraction would return empty.
        assert_eq!(
            ensure_trip_gps_cached(&conn, root.path(), "trip", &segments, false)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            ensure_trip_gps_cached(&conn, root.path(), "trip", &segments, false)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn gps_cache_provenance_survives_database_reopen_and_preserves_archived_spans() {
        let root = tempfile::tempdir().unwrap();
        let (_, mut segments) = cache_fixture(root.path());
        let path = root.path().join("cache.sqlite");
        {
            let db = crate::db::open_at_path(&path, root.path()).unwrap();
            let conn = db.lock().unwrap();
            conn.execute("INSERT INTO trips (id, start_time_ms, end_time_ms, camera_kind, gps_supported, last_seen_ms)
                VALUES ('trip', 0, 120000, 'generic', 1, 0)", []).unwrap();
            seed_cached(&conn, root.path(), &segments);
        }
        let db = crate::db::open_at_path(&path, root.path()).unwrap();
        let conn = db.lock().unwrap();
        assert_eq!(
            ensure_trip_gps_cached(&conn, root.path(), "trip", &segments, false)
                .unwrap()
                .len(),
            1
        );
        // First segment has since been archived. Its cached point must survive
        // rebuilding a later original segment, with the original trip offset.
        segments[0].channels.clear();
        segments.push(TripGpsSegmentRequest {
            segment_id: "later".into(),
            duration_s: 60.0,
            camera_kind: CameraKind::Generic,
            channels: vec![TripGpsChannelRequest {
                label: "Rear".into(),
                path: root.path().join("rear.mp4").to_string_lossy().into(),
            }],
        });
        let points = ensure_trip_gps_cached(&conn, root.path(), "trip", &segments, false).unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].t_offset_s, 1.0);
        assert_eq!(
            crate::db::trip_gps::load(&conn, "trip")
                .unwrap()
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn size_mtime_parser_and_health_each_invalidate_gps_cache() {
        let root = tempfile::tempdir().unwrap();
        let (db, segments) = cache_fixture(root.path());
        let conn = db.lock().unwrap();
        for change in 0..4 {
            seed_cached(&conn, root.path(), &segments);
            match change {
                0 => std::fs::write(
                    &segments[0].channels[0].path,
                    b"replacement with different size",
                )
                .unwrap(),
                1 => {
                    std::fs::File::options()
                        .write(true)
                        .open(&segments[0].channels[0].path)
                        .unwrap()
                        .set_times(std::fs::FileTimes::new().set_modified(
                            std::time::UNIX_EPOCH + std::time::Duration::from_secs(1234),
                        ))
                        .unwrap()
                }
                2 => {
                    conn.execute("UPDATE trip_gps SET parser_version = 1", [])
                        .unwrap();
                }
                _ => {
                    let path = Path::new(&segments[0].channels[0].path);
                    let (size, modified) = crate::media_health::fingerprint(path).unwrap();
                    conn.execute("INSERT INTO media_health_ranges (file_path, file_size, modified_ms, start_ms, error_kind, updated_at_ms)
                        VALUES (?1, ?2, ?3, 1000, 'decode', 0)",
                        rusqlite::params![crate::media_health::stored_path(path, root.path()), size, modified]).unwrap();
                }
            }
            assert!(
                ensure_trip_gps_cached(&conn, root.path(), "trip", &segments, false)
                    .unwrap()
                    .is_empty(),
                "change {change}"
            );
        }
    }

    #[test]
    fn partial_health_failure_excludes_only_matching_camera_fingerprint() {
        let root = tempfile::tempdir().unwrap();
        let (db, segments) = cache_fixture(root.path());
        let conn = db.lock().unwrap();
        let path = Path::new(&segments[0].channels[0].path);
        let (size, modified) = crate::media_health::fingerprint(path).unwrap();
        conn.execute("INSERT INTO media_health_ranges (file_path, file_size, modified_ms, start_ms, end_ms, error_kind, updated_at_ms)
            VALUES (?1, ?2, ?3, 1000, 2000, 'decode', 0)",
            rusqlite::params![crate::media_health::stored_path(path, root.path()), size, modified]).unwrap();
        let (_, sources, _) = trip_gps_sources(&conn, root.path(), &segments).unwrap();
        assert_eq!(
            sources[0].paths,
            segments[0].channels[1..]
                .iter()
                .map(|c| c.path.clone())
                .collect::<Vec<_>>()
        );
        std::fs::write(path, b"repaired replacement").unwrap();
        assert_eq!(
            trip_gps_sources(&conn, root.path(), &segments).unwrap().1[0]
                .paths
                .len(),
            3
        );
    }

    #[test]
    fn archive_only_keeps_gps_even_with_old_parser_and_force_refresh() {
        let root = tempfile::tempdir().unwrap();
        let (db, segments) = cache_fixture(root.path());
        let conn = db.lock().unwrap();
        seed_cached(&conn, root.path(), &segments);
        conn.execute("UPDATE trip_gps SET parser_version = 1", [])
            .unwrap();
        for channel in &segments[0].channels {
            std::fs::remove_file(&channel.path).unwrap();
        }
        assert_eq!(
            ensure_trip_gps_cached(&conn, root.path(), "trip", &segments, true)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            ensure_trip_gps_cached(&conn, root.path(), "trip", &[], true)
                .unwrap()
                .len(),
            1
        );
    }

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
