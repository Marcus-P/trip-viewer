# Timelapse channel alignment: implementation and inherited proposal

The [original upstream design note](docs/history/inherited-TIMELAPSE_ALIGNMENT_REDESIGN.md)
is preserved in full. It reports an upstream hold/overlay experiment and a v2
coverage-map proposal; those measurements were not repeated as real-camera
validation by this fork maintainer. This file distinguishes that history from
what the current checkout actually serializes.

## Current code

Rust `timelapse/speed_curve.rs` and frontend `utils/speedCurve.ts` use envelope
**version 1** with `concatStart`, `concatEnd` and `rate` segments. The frontend
also reads legacy bare arrays. File time is derived by accumulating covered
segment duration/rate. `coverageAt` detects gaps; the sync engine holds a channel
and displays its existing gap overlay instead of repeatedly reloading it.

The current format does not write the proposal’s explicit v2 `fileStart`,
`fileEnd`, `tripConcatDurationS` envelope or a `timelapse_format_version` migration.
Do not document an automatic v2 rebuild or a “last ever” alignment re-encode as
an implemented guarantee. The inherited note’s “implemented” heading describes
its historical context, not proof that every proposed field shipped here.

## Invariants to preserve

Keep per-channel video mounted, preserve concat/file clock mapping through gaps,
and validate monotonic coverage. GPS/speed-ramp changes can still require
re-encoding. Source MP4 transitions and expensive WebKit seeks remain a separate
playback issue; no gapless HEVC/MSE guarantee is made.

Bookmarks, map seeking and timeline navigation reuse the existing speed-curve
mapping, including archive-only timelapses. They do not change the persisted
curve format. New format/migration work requires its own design and tests.

See [architecture](DESIGN.md), [compatibility evidence](docs/COMPATIBILITY.md)
and [project state](docs/PROJECT_STATE.md) for the current fork’s validation scope.
