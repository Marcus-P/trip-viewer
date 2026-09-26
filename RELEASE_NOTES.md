# Trip Viewer 0.4.0

This is the first release of the Marcus-P fork and the first release focused on Ubuntu 24.04 LTS with native VIOFO A229 Pro 3CH support.

## Highlights

- Native VIOFO A229 Pro Front / Interior / Rear recognition, including parking-mode filenames.
- VIOFO GPS extraction with timestamp-based validation and cross-segment boundary handling.
- Resizable and persisted multi-camera/GPS/timeline layout.
- km/h and mph speed graph scale with grid and hover readout.
- GPS vehicle marker stays close to the visual map center without panning on every GPS sample.
- Deterministic single-camera and F/I/R + GPS fullscreen navigation.
- Playback synchronization fixes across camera changes, fullscreen changes, Play/Pause and playback-rate changes.
- Linux audio policy: clean audio at Original 1×; intentionally muted at 0.5×, 2×, 4× and timelapse modes.
- Player remains mounted while switching to Scan/Review/Timelapse, avoiding repeated WebKit media-pipeline teardown.
- Ubuntu desktop integration helper for a stable launcher, icon and Dock pinning.

## Validated environment

- Ubuntu 24.04 LTS x86_64
- VIOFO A229 Pro 3CH
- Frontend transport/fullscreen/audio tests plus Rust/VIOFO tests

## Known behavior

A short interruption at an original MP4 segment boundary is accepted in this release. Windows packaging is not part of 0.4.0 and will be handled separately later.

## License

Trip Viewer code remains MIT licensed. Third-party components retain their own licenses. See `THIRD_PARTY_NOTICES.md` and the dependency-license inventory files attached to the release.

VIOFO and A229 Pro are descriptive compatibility names. This project is not affiliated with or endorsed by VIOFO.
