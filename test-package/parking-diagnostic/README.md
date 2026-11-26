# Complete VIOFO parking observations (v3)

Prepared with Codex for @Marcus-P. This is a standalone **read-only diagnostic**,
not Trip Viewer or a change to PR #21. It reuses the MIT-licensed Novatek layout
and a separately identified Python reproduction of the temporal rules at PR21
091b392. Preserve the included license and attribution when adapting it.

## Run and prerequisites

The owner launcher takes two folders, recursively discovers their MP4/MOV files,
and writes one JSON report covering both. TS inputs are recorded as unsupported;
other files are ignored. Originals, Trip Viewer databases and settings are not
modified. Existing reports are never overwritten. Symlink/reparse directories
are skipped. Discovery and per-file work are bounded, with explicit limits.

`run-windows.ps1` needs Windows 10/11 **x64**, PowerShell 5.1 or newer and internet
for the downloads. It downloads the pinned Python Software Foundation NuGet
package (about 14 MB) into its own working folder and verifies its SHA256 before
execution. There is no Windows/Python installation, administrator requirement,
PATH change or execution-policy change. The diagnostic Python process is isolated
from user-installed packages and does not use the network. Runtime source:
https://www.nuget.org/packages/python/3.12.10 . License remains in the package.

On another platform, with existing Python 3.9+, use:

```sh
python3 diagnose.py --set unknown "/path/to/recordings" --output new-report.json
```

For the owner's known settings, use `--set 2ch-lowbitrate FOLDER` and
`--set 3ch-time2fps FOLDER` together. These presets are **declared settings** from
the owner, not machine-read claims about every file: A229 Pro main firmware
V1.5_260813, 30-minute parking timer, parking GPS/HDR enabled; 2CH Low Bitrate
without entry delay, 3CH Time Lapse 2fps with 90-second entry delay. No camera
serial, installed rear firmware, PC identity or geography is inferred.

## What one run collects

1. Discovery/read failures with a fixed reason and exact processing stage;
   neither private paths nor exception messages enter the report.
2. Movie, track and media durations/timescales; track types, codecs, audio
   presence, sample counts, decoding intervals and composition-table version.
   Edit-list rates/durations/relative offsets are described, not silently applied.
3. GPS descriptor counts, duplicate references and backwards physical offsets;
   fixed-buffer examination of all matching A/V GPS record candidates per block;
   separate counts for inactive fixes, invalid dates and selected active records.
4. Relative GPS spans, adjacent-delta distributions, duplicates/backwards steps,
   late active-fix evidence and differences between physical record order and the
   application's last-active-record-per-descriptor selection.
5. Temporal-rule results with/without media-duration limits, overlapping endpoint
   evidence counts for backwards edges/adjacent spans/total spans, and the count
   that would still exceed video duration if raw UTC deltas were used directly.
   Filename clock-hypothesis candidate counts before/after time filtering are
   reported without timezone offsets or absolute recording times. Spatial
   filtering is NOT reproduced in that filename-clock comparison.
6. Coordinate validity, repeated positions, speed distribution and distance
   dispersion around a coordinate median, including coverage at several stated
   radii. **The center and coordinates never leave memory.** These are descriptive
   statistics, not the app spatial validator, a chosen true location, or proof
   that a vehicle was stationary.
7. Exact standard-track byte-range references to whole GPS blocks, known block
   bodies and decoded 44-byte records; media PTS spans and observed relative
   clock ratios where such references exist. No nearest-chunk, frame-index,
   filename-counter or constant-speed assumption is used.
8. Within each set, candidate cross-channel GPS-time overlaps and relative
   filename/GPS-start differences. Overlap is not declared a recording identity.

## Privacy and limits

The shareable JSON has ordinal set/file/track IDs, counts, relative intervals,
technical format categories and the declared test settings. **No coordinates,
absolute recording dates/times, file/folder names, paths, sizes/hashes of input
files, camera serials or user/computer IDs** are emitted. Input order is not a
published filename mapping. Nothing uploads automatically. Recording statistics
can still be characteristic; inspect the JSON before sharing. This is data
minimization, not a promise of mathematical anonymity.

`complete` means the collection finished for that item, not all interpretations
are established. Inspect nested `partial`/`unavailable` states and reason codes.
A missing GPS list is legitimate observed absence, not proof that GPS was
switched off. Unsupported structures are not negative evidence of GPS timing.
A failed file does not prevent analysis of the other inputs; interruption writes
available partial results where possible.

The script does not decode full video frames, read burned-in text, interpret
codec SEI/vendor-private fields, apply edit lists to GPS, prove content sync,
or change filtering. Fragmented MP4 and compact `stz2` are identified/limited,
not claimed fully supported. Marker patterns can have false matches and are
reported as candidates. Descriptive proximity or majority is not location truth.

Consequently a report can establish explicit container links, show multiple
clock domains, or leave mapping unresolved. Do not infer a fixed time-lapse
factor, discard valid parking GPS, or prescribe app changes from a single hint.

## Tests and provenance

Targeted synthetic tests cover regular/compressed clocks, padded blocks and
window seams, multiple records, inactive fixes, invalid dates, missing GPS,
64-bit offsets and signed composition times, edit-list presence, filename-clock
hypotheses, privacy, per-file failure isolation and report no-overwrite. They do
not constitute a physical A229/A129/Windows acceptance test.

Source layout: `time_core.py` (reused diagnostic parsing/rules),
`parking_analysis.py` (expanded observations), `diagnose.py` (discovery/report),
`run-windows.ps1` (portable runtime launcher). The referenced runtime and bundle
are verified by separate fixed SHA256 values. No application build is required.
