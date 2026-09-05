//! VIOFO / Novatek GPS decoder for A229-family MP4 files.
//!
//! Novatek MP4s used by VIOFO contain a `gps ` descriptor box inside `moov`.
//! After an 8-byte vendor header, the box contains `(offset, size)` pairs
//! pointing to top-level `free` boxes. A referenced `free` box starts with
//! the magic `GPS ` and contains one GPS record.
//!
//! The record layout is the long-established Novatek layout used by VIOFO:
//! six LE u32 time/date values, fix + hemisphere bytes, LE f32 latitude and
//! longitude in NMEA degrees/minutes form, speed in knots, and bearing.

use crate::error::AppError;
use crate::model::GpsPoint;
use chrono::{NaiveDate, NaiveDateTime};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const KNOT_TO_MPS: f64 = 0.514_444;
const EARTH_RADIUS_M: f64 = 6_371_000.0;

// VIOFO files can contain isolated, otherwise-valid GPS records that jump far
// away for a single sample before immediately returning to the real route.
// Keep the thresholds deliberately generous so normal driving data cannot be
// removed: a candidate must be hundreds of metres from both neighbours while
// those neighbours remain mutually reachable at up to 200 m/s (720 km/h).
const MAX_SPIKE_NEIGHBOR_GAP_S: f64 = 5.0;
const MAX_PLAUSIBLE_TRAVEL_MPS: f64 = 200.0;
const MIN_SPIKE_DISTANCE_M: f64 = 300.0;

#[derive(Debug, Clone, Copy)]
struct BoxHeader {
    size: u64,
    kind: [u8; 4],
    body_start: u64,
}

impl BoxHeader {
    fn end(self, start: u64) -> Option<u64> {
        start.checked_add(self.size)
    }
}

pub fn extract(path: &Path) -> Result<Vec<GpsPoint>, AppError> {
    let mut file = File::open(path)?;
    let file_len = file.metadata()?.len();
    let descriptors = read_gps_descriptors(&mut file, file_len)?;
    if descriptors.is_empty() {
        eprintln!("viofo gps: no gps descriptor box in {}", path.display());
        return Ok(vec![]);
    }

    let mut decoded: Vec<(NaiveDateTime, GpsPoint)> = Vec::with_capacity(descriptors.len());
    for (offset, size) in descriptors {
        if let Some(item) = read_gps_free_box(&mut file, file_len, offset, size)? {
            decoded.push(item);
        }
    }

    if decoded.is_empty() {
        eprintln!(
            "viofo gps: descriptor box found but no valid GPS records in {}",
            path.display()
        );
        return Ok(vec![]);
    }

    // Descriptor order is normally chronological, but sorting by the embedded
    // UTC timestamp makes the result robust to a malformed descriptor table.
    decoded.sort_by_key(|(ts, _)| *ts);

    // Keep the original first timestamp as the video-relative time origin even
    // if a later filtering pass removes a bad GPS record. This means removing a
    // spike never shifts the remaining samples on the playback timeline.
    let first_ts = decoded[0].0;
    let before_filter = decoded.len();
    let decoded = filter_isolated_position_spikes(decoded);
    let removed = before_filter.saturating_sub(decoded.len());
    if removed > 0 {
        eprintln!(
            "viofo gps: filtered {removed} isolated position spike(s) in {}",
            path.display()
        );
    }

    let mut points = Vec::with_capacity(decoded.len());
    for (ts, mut point) in decoded {
        point.t_offset_s = (ts - first_ts).num_milliseconds() as f64 / 1000.0;
        points.push(point);
    }

    Ok(points)
}

fn filter_isolated_position_spikes(
    points: Vec<(NaiveDateTime, GpsPoint)>,
) -> Vec<(NaiveDateTime, GpsPoint)> {
    if points.len() < 3 {
        return points;
    }

    let mut keep = vec![true; points.len()];

    for i in 1..points.len() - 1 {
        let (prev_ts, prev) = &points[i - 1];
        let (current_ts, current) = &points[i];
        let (next_ts, next) = &points[i + 1];

        if !prev.fix_ok || !current.fix_ok || !next.fix_ok {
            continue;
        }

        let dt_prev = (*current_ts - *prev_ts).num_milliseconds() as f64 / 1000.0;
        let dt_next = (*next_ts - *current_ts).num_milliseconds() as f64 / 1000.0;
        if dt_prev <= 0.0
            || dt_next <= 0.0
            || dt_prev > MAX_SPIKE_NEIGHBOR_GAP_S
            || dt_next > MAX_SPIKE_NEIGHBOR_GAP_S
        {
            continue;
        }

        let prev_current_m = gps_distance_m(prev, current);
        let current_next_m = gps_distance_m(current, next);
        let prev_next_m = gps_distance_m(prev, next);

        let prev_limit_m =
            MIN_SPIKE_DISTANCE_M.max(MAX_PLAUSIBLE_TRAVEL_MPS * dt_prev);
        let next_limit_m =
            MIN_SPIKE_DISTANCE_M.max(MAX_PLAUSIBLE_TRAVEL_MPS * dt_next);
        let bridge_limit_m = MIN_SPIKE_DISTANCE_M
            .max(MAX_PLAUSIBLE_TRAVEL_MPS * (dt_prev + dt_next));

        if prev_current_m > prev_limit_m
            && current_next_m > next_limit_m
            && prev_next_m <= bridge_limit_m
        {
            keep[i] = false;
        }
    }

    points
        .into_iter()
        .enumerate()
        .filter_map(|(i, point)| keep[i].then_some(point))
        .collect()
}

fn gps_distance_m(a: &GpsPoint, b: &GpsPoint) -> f64 {
    let lat1 = a.lat.to_radians();
    let lat2 = b.lat.to_radians();
    let dlat = (b.lat - a.lat).to_radians();
    let dlon = (b.lon - a.lon).to_radians();

    let sin_dlat = (dlat / 2.0).sin();
    let sin_dlon = (dlon / 2.0).sin();
    let h = sin_dlat * sin_dlat + lat1.cos() * lat2.cos() * sin_dlon * sin_dlon;
    let h = h.clamp(0.0, 1.0);
    let central_angle = 2.0 * h.sqrt().atan2((1.0 - h).sqrt());
    EARTH_RADIUS_M * central_angle
}

fn read_gps_descriptors(file: &mut File, file_len: u64) -> Result<Vec<(u64, u64)>, AppError> {
    let mut top = 0u64;
    while top + 8 <= file_len {
        let Some(header) = read_box_header(file, top, file_len)? else {
            break;
        };
        let Some(end) = header.end(top) else {
            return Err(AppError::Parse("MP4 box size overflow".into()));
        };
        if end > file_len || header.size < header.body_start.saturating_sub(top) {
            break;
        }

        if &header.kind == b"moov" {
            if let Some(found) = find_gps_box(file, header.body_start, end, file_len)? {
                return Ok(found);
            }
        }
        if header.size == 0 {
            break;
        }
        top = end;
    }
    Ok(vec![])
}

fn find_gps_box(
    file: &mut File,
    mut pos: u64,
    moov_end: u64,
    file_len: u64,
) -> Result<Option<Vec<(u64, u64)>>, AppError> {
    while pos + 8 <= moov_end {
        let Some(header) = read_box_header(file, pos, file_len)? else {
            break;
        };
        let Some(end) = header.end(pos) else {
            return Err(AppError::Parse("MP4 child box size overflow".into()));
        };
        if end > moov_end || header.size == 0 {
            break;
        }

        if &header.kind == b"gps " {
            // Novatek stores an 8-byte vendor header before the descriptor
            // pairs. This corresponds to the established parser convention of
            // starting 16 bytes after the `gps ` box start (8 box + 8 vendor).
            let mut cursor = header.body_start.saturating_add(8);
            let mut out = Vec::new();
            while cursor + 8 <= end {
                file.seek(SeekFrom::Start(cursor))?;
                let mut pair = [0u8; 8];
                file.read_exact(&mut pair)?;
                let offset = u32::from_be_bytes(pair[0..4].try_into().unwrap()) as u64;
                let size = u32::from_be_bytes(pair[4..8].try_into().unwrap()) as u64;
                if offset != 0 && size >= 12 && offset.saturating_add(size) <= file_len {
                    out.push((offset, size));
                }
                cursor += 8;
            }
            return Ok(Some(out));
        }

        pos = end;
    }
    Ok(None)
}

fn read_gps_free_box(
    file: &mut File,
    file_len: u64,
    offset: u64,
    descriptor_size: u64,
) -> Result<Option<(NaiveDateTime, GpsPoint)>, AppError> {
    if offset.saturating_add(descriptor_size) > file_len || descriptor_size < 12 {
        return Ok(None);
    }

    file.seek(SeekFrom::Start(offset))?;
    let mut head = [0u8; 12];
    file.read_exact(&mut head)?;
    let box_size = u32::from_be_bytes(head[0..4].try_into().unwrap()) as u64;
    if box_size != descriptor_size || &head[4..8] != b"free" || &head[8..12] != b"GPS " {
        return Ok(None);
    }

    let payload_len = descriptor_size.saturating_sub(12) as usize;
    let mut payload = vec![0u8; payload_len];
    file.read_exact(&mut payload)?;
    Ok(decode_packet(&payload))
}

fn decode_packet(data: &[u8]) -> Option<(NaiveDateTime, GpsPoint)> {
    let offset = find_record_offset(data)?;
    if offset + 44 > data.len() {
        return None;
    }

    let hour = read_u32_le(data, offset)?;
    let minute = read_u32_le(data, offset + 4)?;
    let second = read_u32_le(data, offset + 8)?;
    let year = read_u32_le(data, offset + 12)?;
    let month = read_u32_le(data, offset + 16)?;
    let day = read_u32_le(data, offset + 20)?;

    let active = data[offset + 24];
    let lat_hemi = data[offset + 25];
    let lon_hemi = data[offset + 26];
    let lat_raw = read_f32_le(data, offset + 28)? as f64;
    let lon_raw = read_f32_le(data, offset + 32)? as f64;
    let speed_knots = read_f32_le(data, offset + 36)? as f64;
    let bearing = read_f32_le(data, offset + 40)? as f64;

    let ts = NaiveDate::from_ymd_opt(2000 + year as i32, month, day)?
        .and_hms_opt(hour, minute, second)?;

    let lat = nmea_coord_to_degrees(lat_raw, lat_hemi)?;
    let lon = nmea_coord_to_degrees(lon_raw, lon_hemi)?;
    let fix_ok = active == b'A'
        && lat.is_finite()
        && lon.is_finite()
        && lat.abs() <= 90.0
        && lon.abs() <= 180.0;

    Some((
        ts,
        GpsPoint {
            t_offset_s: 0.0,
            lat: if fix_ok { lat } else { 0.0 },
            lon: if fix_ok { lon } else { 0.0 },
            speed_mps: if speed_knots.is_finite() {
                speed_knots.max(0.0) * KNOT_TO_MPS
            } else {
                0.0
            },
            heading_deg: if bearing.is_finite() {
                bearing.rem_euclid(360.0)
            } else {
                0.0
            },
            altitude_m: 0.0,
            fix_ok,
        },
    ))
}

fn find_record_offset(data: &[u8]) -> Option<usize> {
    if data.len() < 27 {
        return None;
    }
    // Find the `A N/S E/W` fix marker. The marker is 24 bytes after the
    // beginning of the canonical Novatek record.
    for marker in (24..=data.len().saturating_sub(3)).rev() {
        let active = data[marker];
        let lat_hemi = data[marker + 1];
        let lon_hemi = data[marker + 2];
        if active == b'A'
            && matches!(lat_hemi, b'N' | b'S')
            && matches!(lon_hemi, b'E' | b'W')
        {
            return marker.checked_sub(24);
        }
    }
    None
}

fn nmea_coord_to_degrees(raw: f64, hemi: u8) -> Option<f64> {
    if !raw.is_finite() || raw < 0.0 {
        return None;
    }
    let degrees = (raw / 100.0).floor();
    let minutes = raw - degrees * 100.0;
    if !(0.0..60.0).contains(&minutes) {
        return None;
    }
    let mut value = degrees + minutes / 60.0;
    match hemi {
        b'N' | b'E' => {}
        b'S' | b'W' => value = -value,
        _ => return None,
    }
    Some(value)
}

fn read_u32_le(data: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        data.get(off..off + 4)?.try_into().ok()?,
    ))
}

fn read_f32_le(data: &[u8], off: usize) -> Option<f32> {
    Some(f32::from_le_bytes(
        data.get(off..off + 4)?.try_into().ok()?,
    ))
}

fn read_box_header(
    file: &mut File,
    start: u64,
    file_len: u64,
) -> Result<Option<BoxHeader>, AppError> {
    if start + 8 > file_len {
        return Ok(None);
    }
    file.seek(SeekFrom::Start(start))?;
    let mut hdr = [0u8; 8];
    file.read_exact(&mut hdr)?;
    let size32 = u32::from_be_bytes(hdr[0..4].try_into().unwrap());
    let kind = hdr[4..8].try_into().unwrap();

    let (size, body_start) = match size32 {
        0 => (file_len - start, start + 8),
        1 => {
            if start + 16 > file_len {
                return Ok(None);
            }
            let mut ext = [0u8; 8];
            file.read_exact(&mut ext)?;
            (u64::from_be_bytes(ext), start + 16)
        }
        n => (n as u64, start + 8),
    };
    if size < body_start - start {
        return Ok(None);
    }
    Ok(Some(BoxHeader {
        size,
        kind,
        body_start,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn sample_packet() -> Vec<u8> {
        let mut p = vec![0u8; 48];
        p[0..4].copy_from_slice(&6u32.to_le_bytes());
        p[4..8].copy_from_slice(&52u32.to_le_bytes());
        p[8..12].copy_from_slice(&29u32.to_le_bytes());
        p[12..16].copy_from_slice(&26u32.to_le_bytes());
        p[16..20].copy_from_slice(&8u32.to_le_bytes());
        p[20..24].copy_from_slice(&31u32.to_le_bytes());
        p[24] = b'A';
        p[25] = b'N';
        p[26] = b'E';
        p[27] = 0;
        p[28..32].copy_from_slice(&(5332.5f32).to_le_bytes());
        p[32..36].copy_from_slice(&(1001.5f32).to_le_bytes());
        p[36..40].copy_from_slice(&(10.0f32).to_le_bytes());
        p[40..44].copy_from_slice(&(123.0f32).to_le_bytes());
        p
    }

    fn gps_point(lat: f64, lon: f64) -> GpsPoint {
        GpsPoint {
            t_offset_s: 0.0,
            lat,
            lon,
            speed_mps: 10.0,
            heading_deg: 0.0,
            altitude_m: 0.0,
            fix_ok: true,
        }
    }

    #[test]
    fn decodes_novatek_packet() {
        let (ts, p) = decode_packet(&sample_packet()).unwrap();
        assert_eq!(
            ts,
            NaiveDate::from_ymd_opt(2026, 8, 31)
                .unwrap()
                .and_hms_opt(6, 52, 29)
                .unwrap()
        );
        assert!((p.lat - 53.5416667).abs() < 0.00001);
        assert!((p.lon - 10.025).abs() < 0.00001);
        assert!((p.speed_mps - 5.14444).abs() < 0.0001);
        assert_eq!(p.heading_deg, 123.0);
        assert!(p.fix_ok);
    }

    #[test]
    fn converts_south_and_west() {
        assert!((nmea_coord_to_degrees(5332.5, b'S').unwrap() + 53.5416667).abs() < 0.00001);
        assert!((nmea_coord_to_degrees(1001.5, b'W').unwrap() + 10.025).abs() < 0.00001);
    }

    #[test]
    fn filters_isolated_position_spike() {
        let base = NaiveDate::from_ymd_opt(2026, 8, 31)
            .unwrap()
            .and_hms_opt(4, 53, 35)
            .unwrap();
        let points = vec![
            (base, gps_point(53.5418701171875, 10.083750406901)),
            (
                base + chrono::Duration::seconds(1),
                gps_point(0.0833333333333333, 0.0),
            ),
            (
                base + chrono::Duration::seconds(2),
                gps_point(53.5419189453125, 10.083745320638),
            ),
        ];

        let filtered = filter_isolated_position_spikes(points);
        assert_eq!(filtered.len(), 2);
        assert!((filtered[0].1.lat - 53.5418701171875).abs() < 0.0000001);
        assert!((filtered[1].1.lat - 53.5419189453125).abs() < 0.0000001);
    }

    #[test]
    fn keeps_plausible_middle_point() {
        let base = NaiveDate::from_ymd_opt(2026, 8, 31)
            .unwrap()
            .and_hms_opt(4, 53, 35)
            .unwrap();
        let points = vec![
            (base, gps_point(53.54187, 10.08375)),
            (
                base + chrono::Duration::seconds(1),
                gps_point(53.54190, 10.08375),
            ),
            (
                base + chrono::Duration::seconds(2),
                gps_point(53.54193, 10.08375),
            ),
        ];

        let filtered = filter_isolated_position_spikes(points);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn extracts_from_minimal_novatek_mp4_layout() {
        let packet = sample_packet();
        let free_size = 12 + packet.len() as u32;

        // ftyp (8) + moov (32) => referenced free box starts at byte 40.
        let free_offset = 40u32;
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&8u32.to_be_bytes());
        bytes.extend_from_slice(b"ftyp");

        bytes.extend_from_slice(&32u32.to_be_bytes());
        bytes.extend_from_slice(b"moov");
        bytes.extend_from_slice(&24u32.to_be_bytes());
        bytes.extend_from_slice(b"gps ");
        bytes.extend_from_slice(&[0u8; 8]);
        bytes.extend_from_slice(&free_offset.to_be_bytes());
        bytes.extend_from_slice(&free_size.to_be_bytes());

        bytes.extend_from_slice(&free_size.to_be_bytes());
        bytes.extend_from_slice(b"free");
        bytes.extend_from_slice(b"GPS ");
        bytes.extend_from_slice(&packet);

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("2026_0831_065229_000646F.MP4");
        let mut f = File::create(&path).unwrap();
        f.write_all(&bytes).unwrap();
        drop(f);

        let points = extract(&path).unwrap();
        assert_eq!(points.len(), 1);
        assert_eq!(points[0].t_offset_s, 0.0);
        assert!(points[0].fix_ok);
    }
}
