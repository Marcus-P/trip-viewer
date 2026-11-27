# Full local VIOFO metadata capture

Prepared with Codex for @Marcus-P. Standalone, read-only tooling under the
repository MIT license. This does not modify Trip Viewer, its database, PR #21,
recordings, installed codecs or the stable release. Original contributor and
external tool licenses remain applicable.

## Windows x64

Run the accompanying `run-windows.ps1` with `-TwoChannel`, `-ThreeChannel` and
`-ToolDir` pointing to the extracted package. The owner receives a complete
PowerShell command with their already supplied folders separately, not in this
public source. An existing FFmpeg distribution including `ffprobe.exe` is
required. The launcher finds it from PATH, known installation locations or the
configured Trip Viewer FFmpeg path; otherwise it opens a file picker. It does
not install or download another FFmpeg build.

The launcher downloads official portable ExifTool 13.64 for Windows x64 and
Python Software Foundation Python 3.12.10. It reuses a verified prior Python
archive from either diagnostic where available. Python is fetched directly
from the official NuGet flat-container endpoint first, avoiding the v2
redirect to `globalcdn.nuget.org`; v2 remains a bounded fallback. Both must
match the same pinned SHA256, and incomplete cached downloads are retried.
No administrator access, system Python, PATH change
or execution-policy change is needed. Tool files and their licenses remain
under the new diagnostic directory. No media or report is uploaded.

## Outputs

- **`PRIVATE-DO-NOT-SHARE/`** contains unredacted ExifTool and FFprobe JSON,
  tool errors, original file mapping, exact metadata field paths and verbatim
  MP4 metadata blocks. It can contain coordinates, recording dates, device
  identifiers and other identifying information. Keep it local.
- **`SHARE-metadata-report.json`** contains ordinal file/field IDs, owner-declared
  settings, field types, occurrence/distinct-value counts, standard ExifTool tag
  names and equality comparisons between the two sets. Custom field names are
  represented by IDs, because a name itself can contain private text. No source
  arbitrary values, hashes, original names/paths, coordinates or absolute times
  are copied. A closed list of explicit mode categories and common capture-rate
  values is retained only for specifically named mode/rate fields. These are
  metadata observations, not proof that a mode was active or a GPS mapping valid.
  Review this report before sharing; aggregate characteristics are not a promise
  of mathematical anonymity. These are declared test modes, not inferred modes.

For a promising field ID, its actual name is available in the local
`field-index-PRIVATE.json`; its unredacted value is in the corresponding tool
JSON. This allows follow-up inspection without reading the recordings again.
Sharing a raw field still needs a separate privacy assessment.

## Extraction coverage and stopping limits

ExifTool uses `-a -u -U -ee3 -G4:1 -struct -b -json`: duplicate groups,
recognized custom and unknown tags, embedded metadata and binary tag values are
requested. A blank explicit configuration avoids loading a personal ExifTool
configuration. FFprobe records format, streams, chapters, programs, private
fields and codec extra data. Neither tool is restricted to a GPS/parking
whitelist. No original-file checksum, decoded frame export or transcoding is
performed. ExifTool may read embedded packet metadata while examining the file.

All top-level MP4 boxes except `mdat` are copied verbatim when within limits;
`moov` includes its nested unknown fields. A private index records offsets,
types and limits. This preserves unknown binary data without inventing its
meaning. Arbitrary unrecognized structures inside compressed media payloads
are not separately extracted or guaranteed decoded. Existing originals remain
unchanged and are the authoritative source for those bytes. A PDF metadata
reader likewise cannot automatically explain arbitrary opaque binary payloads.

Limits: 180 seconds and 128 MiB output per tool per file, 32 MiB per copied box,
128 MiB copied boxes per recording, 100,000 top-level boxes. Source discovery
reuses the parking diagnostic's link avoidance and file/depth limits. A hit,
tool error, unsupported container or interruption is reported as partial, not
negative evidence of a field's absence. A problematic file does not stop other
files. Existing output directories are refused. Each file prints a neutral
ordinal progress line; do not infer a hang just because one file takes longer.

Linux/macOS can run `capture.py` with Python 3.9+, ExifTool and FFprobe installed,
using repeated `--set PROFILE FOLDER`, `--exiftool PATH`, `--ffprobe PATH` and
`--output-dir NEW_DIRECTORY`. Windows launcher validation is synthetic; actual
camera metadata results require the owner's recording run.

## Validation

Six targeted checks pass, including real ExifTool 13.64/FFprobe extraction of
a synthetic custom metadata field on Linux, raw-value privacy, exact unknown
block preservation, partial-file handling, timeout and overwrite protection.
The actual ZIP's portable Python/ExifTool launch and report collection passed
under Windows PowerShell in [run 45981247302](https://github.com/Marcus-P/trip-viewer/actions/runs/45981247302).
That Windows check used a small FFprobe stub to avoid another FFmpeg download;
it validates native invocation, not Windows FFprobe/media compatibility. The
only subsequent runtime change pins the successfully downloaded official
ExifTool archive's SHA256. No app build, original camera footage upload or
hardware certification is implied.
