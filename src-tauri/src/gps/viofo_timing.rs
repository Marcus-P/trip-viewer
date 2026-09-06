//! Align VIOFO GPS UTC timestamps to the local camera timestamp in the filename.
//!
//! VIOFO A229-family files name each clip with the camera's local wall-clock
//! start time, while Novatek GPS records carry UTC timestamps. A camera that
//! has not acquired GPS yet may emit no GPS records at all for the beginning
//! of a clip. The main VIOFO decoder therefore cannot safely treat the first
//! GPS record as video t=0.
//!
//! We infer the camera's UTC offset by trying real-world civil offsets in
//! 15-minute steps. A candidate is accepted only when it places the first GPS
//! fix inside this MP4's actual video duration. If more than one offset fits,
//! alignment is ambiguous and we deliberately return None rather than guess.

use crate::error::AppError;
use chrono::{Duration, NaiveDate, NaiveDateTime};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const MIN_UTC_OFFSET_MINUTES: i64 = -12 * 60;
const MAX_UTC_OFFSET_MINUTES: i64 = 14 * 60;
const UTC_OFFSET_STEP_MINUTES: i64 = 15;
const ALIGNMENT_TOLERANCE_S: f64 = 2.0;

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

/// Return the video-relative time of the first valid VIOFO GPS fix.
///
/// Example from a real A229 Pro clip:
/// filename start 2026-08-31 18:37:41 local, first GPS 16:39:08 UTC,
/// duration 180 s -> unique UTC+02:00 candidate -> first fix at t=87 s.
pub fn first_fix_delay_s(path: &Path) -> Result<Option<f64>, AppError> {
    let Some(filename) = path.file_name().and_then(|n| n.to_str()) else {
        return Ok(None);
    };
    let Some(parsed) = crate::scan::viofo::parse(filename) else {
        return Ok(None);
    };

    let Some(first_gps_utc) = first_valid_gps_timestamp(path)? else {
        return Ok(None);
    };

    // The scanner already probes this metadata while building trips, but GPS
    // extraction is intentionally a separate command and does not currently
    // receive segment duration. Re-probing the MP4 header here is cheap and,
    // importantly, lets us reject timezone guesses that would place the first
    // fix outside the clip.
    let duration_s = crate::metadata::mp4_probe::probe(path)?.duration_s;
    let Some((utc_offset_minutes, delay_s)) = infer_alignment(
        parsed.start_time,
        first_gps_utc,
        duration_s,
    ) else {
        eprintln!(
            "viofo gps: could not uniquely align GPS UTC time to video start in {}",
            path.display()
        );
        return Ok(None);
    };

    if delay_s > ALIGNMENT_TOLERANCE_S {
        let sign = if utc_offset_minutes >= 0 { '+' } else { '-' };
        let abs_minutes = utc_offset_minutes.abs();
        eprintln!(
            "viofo gps: first valid fix at +{delay_s:.1}s in {} (camera UTC{sign}{:02}:{:02})",
            path.display(),
            abs_minutes / 60,
            abs_minutes % 60,
        );
    }

    Ok(Some(delay_s))
}

fn infer_alignment(
    video_start_local: NaiveDateTime,
    first_gps_utc: NaiveDateTime,
    duration_s: f64,
) -> Option<(i64, f64)> {
    if !duration_s.is_finite() || duration_s <= 0.0 {
        return None;
    }

    let mut candidates = Vec::new();
    let mut offset_minutes = MIN_UTC_OFFSET_MINUTES;
    while offset_minutes <= MAX_UTC_OFFSET_MINUTES {
        let gps_as_local = first_gps_utc + Duration::minutes(offset_minutes);
        let delay_s = (gps_as_local - video_start_local).num_milliseconds() as f64 / 1000.0;

        if delay_s >= -ALIGNMENT_TOLERANCE_S
            && delay_s <= duration_s + ALIGNMENT_TOLERANCE_S
        {
            candidates.push((offset_minutes, delay_s.max(0.0)));
        }

        offset_minutes += UTC_OFFSET_STEP_MINUTES;
    }

    if candidates.len() == 1 {
        candidates.into_iter().next()
    } else {
        None
    }
}

fn first_valid_gps_timestamp(path: &Path) -> Result<Option<NaiveDateTime>, AppError> {
    let mut file = File::open(path)?;
    let file_len = file.metadata()?.len();
    let descriptors = read_gps_descriptors(&mut file, file_len)?;

    let mut earliest: Option<NaiveDateTime> = None;
    for (offset, size) in descriptors {
        if let Some(ts) = read_gps_timestamp(&mut file, file_len, offset, size)? {
            if earliest.map(|current| ts < current).unwrap_or(true) {
                earliest = Some(ts);
            }
        }
    }
    Ok(earliest)
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

fn read_gps_timestamp(
    file: &mut File,
    file_len: u64,
    offset: u64,
    descriptor_size: u64,
) -> Result<Option<NaiveDateTime>, AppError> {
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
    Ok(decode_timestamp(&payload))
}

fn decode_timestamp(data: &[u8]) -> Option<NaiveDateTime> {
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

    // Mirror the main decoder's coordinate validation so the timestamp we use
    // for alignment belongs to the same first record that can become a
    // displayed GPS point.
    let lat_hemi = data[offset + 25];
    let lon_hemi = data[offset + 26];
    let lat_raw = read_f32_le(data, offset + 28)? as f64;
    let lon_raw = read_f32_le(data, offset + 32)? as f64;
    let lat = nmea_coord_to_degrees(lat_raw, lat_hemi)?;
    let lon = nmea_coord_to_degrees(lon_raw, lon_hemi)?;
    if !lat.is_finite() || !lon.is_finite() || lat.abs() > 90.0 || lon.abs() > 180.0 {
        return None;
    }

    NaiveDate::from_ymd_opt(2000 + year as i32, month, day)?
        .and_hms_opt(hour, minute, second)
}

fn find_record_offset(data: &[u8]) -> Option<usize> {
    if data.len() < 27 {
        return None;
    }
    for marker in (24..=data.len().saturating_sub(3)).rev() {
        if data[marker] == b'A'
            && matches!(data[marker + 1], b'N' | b'S')
            && matches!(data[marker + 2], b'E' | b'W')
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

    fn dt(y: i32, m: u32, d: u32, h: u32, min: u32, s: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(h, min, s)
            .unwrap()
    }

    #[test]
    fn aligns_real_late_fix_example() {
        let aligned = infer_alignment(
            dt(2026, 8, 31, 18, 37, 41),
            dt(2026, 8, 31, 16, 39, 8),
            180.0,
        );
        assert_eq!(aligned, Some((120, 87.0)));
    }

    #[test]
    fn aligns_immediate_fix_with_timezone() {
        let aligned = infer_alignment(
            dt(2026, 8, 31, 6, 52, 29),
            dt(2026, 8, 31, 4, 52, 29),
            180.0,
        );
        assert_eq!(aligned, Some((120, 0.0)));
    }

    #[test]
    fn supports_quarter_hour_civil_offsets() {
        let aligned = infer_alignment(
            dt(2026, 8, 31, 10, 0, 30),
            dt(2026, 8, 31, 4, 15, 0),
            180.0,
        );
        assert_eq!(aligned, Some((345, 30.0)));
    }

    #[test]
    fn refuses_ambiguous_long_clip_alignment() {
        let aligned = infer_alignment(
            dt(2026, 8, 31, 10, 0, 0),
            dt(2026, 8, 31, 8, 0, 0),
            1800.0,
        );
        assert_eq!(aligned, None);
    }
}
