//! Bounded checks of metadata already parsed during import. No frame decoding,
//! external tool, database migration or assumption about capture speed.

use mp4::Mp4Track;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackEvidence {
    pub track_id: u32,
    pub media_time_scale: u32,
    pub declared_samples: u32,
    pub timed_samples: Option<u64>,
    pub header_duration_ticks: u64,
    pub decode_duration_ticks: Option<u64>,
    pub composition_offsets_present: bool,
    pub edit_list_present: bool,
    pub findings: Vec<&'static str>,
}

pub fn inspect(track: &Mp4Track, fragmented: bool) -> TrackEvidence {
    let table = &track.trak.mdia.minf.stbl;
    let mut evidence = TrackEvidence {
        track_id: track.trak.tkhd.track_id,
        media_time_scale: track.timescale(),
        declared_samples: table.stsz.sample_count,
        timed_samples: None,
        header_duration_ticks: track.trak.mdia.mdhd.duration,
        decode_duration_ticks: None,
        composition_offsets_present: table.ctts.is_some(),
        edit_list_present: track.trak.edts.is_some(),
        findings: vec![],
    };
    if evidence.media_time_scale == 0 {
        evidence.findings.push("invalid_zero_media_timescale");
    }
    if fragmented {
        evidence
            .findings
            .push("fragmented_sample_checks_not_applied");
        return evidence;
    }
    let totals = timing_totals(
        table
            .stts
            .entries
            .iter()
            .map(|entry| (entry.sample_count, entry.sample_delta)),
    );
    match totals {
        Some((samples, ticks)) => {
            evidence.timed_samples = Some(samples);
            evidence.decode_duration_ticks = Some(ticks);
            if samples != u64::from(table.stsz.sample_count) {
                evidence.findings.push("sample_count_tables_disagree");
            }
            if ticks != evidence.header_duration_ticks {
                // Edits, presentation offsets and writer conventions need
                // investigation. Do not reject a playable file on this alone.
                evidence.findings.push("header_and_decode_duration_differ");
            }
        }
        None => evidence.findings.push("sample_time_arithmetic_overflow"),
    }
    if table.stsz.sample_size == 0
        && table.stsz.sample_sizes.len() as u64 != u64::from(table.stsz.sample_count)
    {
        evidence.findings.push("sample_size_table_length_disagrees");
    }
    if table.stss.as_ref().is_some_and(|sync| {
        sync.entries
            .iter()
            .any(|sample| *sample == 0 || *sample > table.stsz.sample_count)
    }) {
        evidence.findings.push("sync_sample_index_out_of_range");
    }
    evidence
}

fn timing_totals(mut entries: impl Iterator<Item = (u32, u32)>) -> Option<(u64, u64)> {
    entries.try_fold((0u64, 0u64), |(samples, ticks), (count, delta)| {
        Some((
            samples.checked_add(u64::from(count))?,
            ticks.checked_add(u64::from(count).checked_mul(u64::from(delta))?)?,
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_sample_durations_are_accumulated_without_guessing_capture_rate() {
        assert_eq!(
            timing_totals([(20, 1000), (10, 500)].into_iter()),
            Some((30, 25000))
        );
    }

    #[test]
    fn metadata_arithmetic_overflow_is_unknown_not_wrapped_time() {
        assert_eq!(timing_totals([(u32::MAX, u32::MAX); 2].into_iter()), None);
    }

    #[test]
    fn reports_inconsistent_tables_without_rejecting_unknown_or_fragmented_layouts() {
        let mut track = Mp4Track {
            trak: Default::default(),
            trafs: vec![],
            default_sample_duration: 0,
        };
        track.trak.mdia.mdhd.timescale = 1000;
        track.trak.mdia.minf.stbl.stsz.sample_count = 1;
        let evidence = inspect(&track, false);
        assert!(evidence.findings.contains(&"sample_count_tables_disagree"));
        let fragmented = inspect(&track, true);
        assert_eq!(
            fragmented.findings,
            vec!["fragmented_sample_checks_not_applied"]
        );
    }
}
