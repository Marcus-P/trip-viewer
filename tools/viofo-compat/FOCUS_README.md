# VIOFO GPS focus diagnostic — Ubuntu 24.04 x64

This is a read-only diagnostic, not a Trip Viewer app update. It locates the
largest adjacent GPS distances in one recording and compares the result with
fresh decoding of the 0.13.0 channel-selection, segment-boundary and stitching
functions. Original media and archives stay unchanged. Nothing is uploaded.

## Run

Python 3 is required. Extract this bundle, verify its checksums, then run:

```sh
sha256sum -c SHA256SUMS.txt
python3 diagnose-gps.py /path/to/video-folder --anchor EXACT_FRONT_FILENAME.MP4 --report /path/to/new-report.json
```

Replace the folder, exact Front basename and output path. The input folder must
contain the target recording and its immediately preceding/following recording,
with their available Front/Interior/Rear files. It can be the existing video
library: the tool catalogs filenames, then decodes only those three groups
(normally nine MP4s). Neighbor discovery stays in the anchor's directory and
normal/parking mode. Duplicate anchors, duplicate channels and absent neighbors
produce an explicit error rather than a guessed grouping. The report path must
not already exist. The output folder must exist and be writable.

No exact coordinates are written to the report by default. To deliberately
include them in a separate report, add `--include-coordinates`. The ordinary
report still includes filenames, recording/GPS times and distance summaries;
review it before sharing. No media upload is needed.

## What the report means

- Per-file results identify the two decoded points forming the largest distance,
  their decoded-point numbers, video-relative offsets, surrounding points and
  estimated camera clock times. Actual embedded UTC is retained for the
  validated app decoder. The older decoder does not expose UTC to this harness.
- `camera_time_estimate` is filename start plus decoded video offset; it is not
  an independent reading of the burned-in video overlay and can be affected by
  gaps or camera-clock behavior. `gps_utc` is the embedded validated UTC.
- All available channels are decoded. Their complete validated decoded series
  are compared, with position differences also compared at matching UTC values.
  This does not claim raw MP4 telemetry bytes are identical.
- Channel selection uses the exact 0.13.0 function: usable point count within
  the segment duration, then time coverage; Front wins equal-quality ties.
- The exact 0.13.0 incremental boundary functions and duration-trim/stitch
  functions are replayed. Removed points are listed separately from points
  removed by duration trimming. Before/after file-boundary UTC gaps help
  distinguish real recording interruptions from single-file anomalies.
- This is a fresh decode with available candidates, not a read of the running
  app's cached route, media-health exclusions or map rendering. It does not
  establish camera firmware behavior, actual missing road distance or a
  visible playback defect. Three recording groups cannot reproduce additional
  context outside that window.

Candidate ordering and segment duration mirror normal healthy-channel app
inputs. The first channel's measured video duration is the segment duration.
No file timestamps or geographic locations are used to guess grouping.

## Safety and provenance

Workers have a 15-second timeout for the original decoder and a 60-second
limit for the three-group app replay. Linux workers have a 512 MiB address-space
cap. Failures are explicit. The input files are opened for reading only;
reports use exclusive creation and cannot overwrite an existing report.

PROVENANCE.json identifies @alinnastac commit 919abfd, isolated PR21 eb0a5a3,
and application 0.13.0 commit b7d5fd6. The app decoder, validator and timing
files are exact copies. The selection/boundary/stitch function bodies are
copied verbatim with per-function hashes; a no-op profiling callback and small
model/IPC adapters replace the GUI/database infrastructure. No production
application/parser code is modified. LICENSE and dependency notices are bundled.

Prepared with Codex by OpenAI. Tests cover source selection, boundary removal,
true UTC pauses, channel differences, coordinate opt-in, original-file hashes,
report overwrite refusal and a real synthetic H.264 MP4 with embedded GPS.
Real-camera validation of this focused report is still pending.

The older `run-compare.py` remains available for the per-file comparison. Its
results deliberately omit the app selection and boundary stages.
