# Trip Viewer 0.5.0

Ubuntu 24.04 LTS release with more robust multi-camera playback and progressive,
resumable GPS preparation. Builds on the Linux playback fixes contributed in
PR #2 by `gubiq`; contributor authorship is preserved.

## Highlights

- Seek and segment-transition synchronization with bounded waits and recovery
  for WebKit readiness events that never arrive.
- A failed camera is isolated while healthy cameras continue. Background decode
  probes allow recovery without repeatedly restarting the visible camera.
- Fingerprinted media-health intervals persist across app restarts. A promoted
  synchronization master remains in charge until the next segment.
- GPS preparation runs independently of Play/Pause, without holding the archive
  database mutex while reading MP4 data.
- GPS route and speed data appear progressively, with a completed/total segment
  counter and progress bar.
- Switching away from an unfinished trip and back reuses its running analysis.
  Completed segment checkpoints survive an app restart; only unfinished or
  invalidated segments need analysis.
- Each segment selects GPS from the strongest healthy camera. VIOFO timestamps,
  delayed first fixes and cross-segment validation retain their existing rules.
- Version files and merge increments are checked in CI; the versioning policy is
  documented in `docs/VERSIONING.md`.

## Validation

Real-world Ubuntu tests confirmed progressive GPS preparation during Play/Pause,
segment transitions, returning to an unfinished trip, and continuation after an
app restart. Automated frontend, Rust, headless WebKit and Ubuntu AppImage checks
are recorded in `docs/PROJECT_STATE.md` and the matching CI run.

GPS extraction performance is being investigated separately. Map-route clicking
to seek the video and a collapsible fullscreen timeline remain planned work.

## License

Trip Viewer remains MIT licensed. Third-party components retain their licenses;
see `THIRD_PARTY_NOTICES.md` and the dependency-license inventories attached to
the release. VIOFO/A229 Pro are descriptive compatibility names; this project is
not affiliated with or endorsed by VIOFO.
