# Trip Viewer 0.6.0

Playback tools and bounded GPS analysis, building on the user-validated 0.5.0.

## New playback tools

- Click the recorded route to seek the video and timeline to the corresponding
  trip time, including between samples and across video segments. GPS gaps are
  not bridged; repeated locations prefer the traversal nearest the playhead.
- The F/I/R + GPS fullscreen dashboard includes the timeline and speed graph,
  with an expand/collapse button and a saved preference. Single-camera/back
  navigation keeps the same video elements and fullscreen root.
- While paused, zoom using the wheel or +/- controls and drag the enlarged
  image to pan. Reset returns to 100%. Resuming playback or changing source
  restores the normal view without replacing/reloading the video element.
- Archived timelapses can seek via trip time even when no original segment
  list remains, using the existing speed-curve mapping.

## GPS analysis

- Independent candidate files share a read pool capped at two workers across
  trip and batch jobs. Candidate order/ties, raw GPS source order, best-camera
  selection and adjacent VIOFO validation remain deterministic.
- VIOFO descriptor pairs are read through a bounded sequential buffer instead
  of seeking for each pair. Segment and trip caches remain valid (parser v8).
- Completed checkpoints bypass reader jobs. Source/health revalidation and
  short DB lock sections are preserved.
- Optional command-line diagnostics: `TRIPVIEWER_GPS_TIMING=1` reports metadata,
  GPS read/decode, validation and candidate-selection times. Worker comparison
  can use `TRIPVIEWER_GPS_WORKERS=1` or `2` (values are capped to this range).

A real USB-SSD speedup has not yet been measured. The reported baseline is
about 20 seconds per 3-minute, three-camera segment. Test uncached comparable
footage; existing cached trips intentionally do not repeat extraction.

## Validation and release status

Automated frontend/Rust tests and native WebKit interaction checks are recorded
in `docs/PROJECT_STATE.md`. This feature branch is a test build until its Ubuntu
CI and the focused real-footage checks have passed and merging is authorized.

## License

Trip Viewer remains MIT licensed. Third-party components retain their licenses;
see `THIRD_PARTY_NOTICES.md` and the release dependency-license inventories.
