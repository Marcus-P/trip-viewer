## VIOFO support with schema-13 playback and GPS reliability fixes

This adds dedicated VIOFO camera recognition and GPS support, plus the related reliability changes reused from my full fork. Camera recognition alone did not provide usable playback on my A229 Pro 3CH setup, so this contribution now keeps the necessary playback/GPS work together. Optional fork settings, km/h preferences, map-follow behavior, overlays, bookmarks and additional AMD/Intel/VA-API encoders and calibration features remain separate.

### Camera recognition and GPS

- Recognize counter-bearing and older counter-free VIOFO filenames. Keep recording mode through grouping and preserve protected `RO` context during import, so protected clips remain event recordings. Parking and protected status are distinct.
- Decode Novatek descriptors in physical record order. Bound reads by file/container size, check corrupt years without overflow and validate GPS using neighboring observations and independent telemetry fields. Preserve the existing continuity safeguards and non-VIOFO dispatch.
- Read padded GPS containers with a fixed-size buffer rather than rejecting the entire container above 4 KB. Regression tests cover records crossing read-window boundaries and preserve the original last-marker selection.
- Prepare GPS progressively with a shared bounded read pool, reusable segment checkpoints and source/health provenance. Failed reads cannot be recorded as successful empty tracks; healthy sibling cameras can still supply GPS. Empty results, errors and active preparation have distinct UI states.

### Existing timelapse behavior

- A compiled-in `hevc_nvenc` entry is no longer accepted as proof of usable hardware. The full fork's finite synthetic encode/decode/scaling verification is adapted to the existing NVENC/CPU capability API. Unsupported NVENC uses the existing CPU/libx265 path.
- Batch start revalidates old capability settings, so a previously cached false positive cannot keep failing all selected camera jobs. The probe runs outside the async command executor and has a bounded failure deadline. No archive or settings schema fields are added.
- A real, opt-in FFmpeg integration test creates two synthetic HEVC clips, runs the actual 8x generation path and decodes the finished output. This passed locally without owner footage. It does not certify the owner's previously failed real three-channel batch or NVIDIA hardware support.

### Playback and archive compatibility

- Reuse the fork's playback intent, canonical audio, seek barriers and channel-local failure/recovery handling. Retain existing media elements across application-view and fullscreen transitions.
- Fullscreen uses one shared viewer root including transport. Play/Pause stays under the SyncEngine instead of native single-video controls. Returning from fullscreen re-anchors the existing cameras without rebuilding their media pipelines.
- Retain the Linux host-libva/DMABuf compatibility handling and explicitly include the GL media component in Linux builds. The previous isolated AppImage lacked the GL plugin and WebKit reported its accelerated sink dependencies missing.
- **The main archive schema and migration head remain 13. No migration files are added or changed.** Additional analysis lives beside the archive in `.tripviewer/analysis-cache-v1.sqlite`, not in a periodically cleared OS temporary cache. Tests compare the main schema and version before/after sidecar writes and reopen the main archive at schema 13. This is a schema compatibility check, not a packaged test of every older application. Existing archived GPS can seed the sidecar. This does not claim that schema-17 full-fork archives are readable by an unchanged schema-13 application.

The new commits separate review corrections, recording-mode/RO fixes, the original Linux media contribution, the related runtime reliability changes and build/test guidance. There is no intermediate schema upgrade in these new commits. Final scope and merge decisions remain with the upstream maintainer.

### Validation and limits

- Rust library tests: **386 passed, 2 ignored in the default run** (one existing ignored test plus the opt-in FFmpeg test; the latter was run explicitly and passed), no failures.
- Frontend: **78 tests passed**; TypeScript and zero-warning Clippy pass. Changed Rust modules are formatted for the repository's edition.
- Main-schema compatibility and checkpoint/recovery regression tests pass. Current upstream main is included; freshness is checked again at handoff.
- I tested the isolated Ubuntu GPS candidate on the same A229 Pro 3CH recordings: all 39 segments completed, producing 6,450 points, and I confirmed visible tracks, the manually merged track and the moving marker. This is GPS/map acceptance, not playback acceptance of this subsequent fullscreen/media-package revision.
- The subsequent Ubuntu AppImage build/startup and bundled GL-factory checks passed. I have now also tested that isolated revision on my Ubuntu 24.04 notebook with my A229 Pro 3CH recordings: Original at x1, Play/Pause, seeking and fullscreen transitions worked. GPS tracks, merged-trip tracks and the moving marker are visible. I did not observe the previous large route jumps at the locations I checked. This is acceptance of those checks on that setup, not universal camera, GPU, driver or long-duration validation.
- My real Front/8x CPU timelapse job is currently running. Its finished output has not yet been validated. The present progress bar counts completed jobs, not live FFmpeg frame progress, so it can remain unchanged during a long first encode. Additional live progress and GPU-performance work from the full fork remains separate.
- The earlier A129 diagnostic comparison had 15 fewer output points. Those exclusions remain an open validation question; I am not claiming that fewer points prove better filtering. A decoder diagnostic does not validate complete application playback. The credited synthetic A129 record layout test is separate evidence.
- Channel association still uses the existing nearby-timestamp heuristic. It is not a GPS-proven recording identity or a universal synchronization guarantee. No parking footage is required for the available playback check.

### Building and testing

[Complete Windows and Ubuntu instructions](https://github.com/Marcus-P/trip-viewer/blob/contrib/viofo-camera-only/docs/VIOFO_TESTING.md) include prerequisites, exact commands, the source commit to record, a separate test application identity, disabled test updater endpoints and a small optional playback check. Windows users can build a native executable without installer signing. The manual Ubuntu acceptance package is a prerelease, not Latest, and does not update the full fork's stable v0.13.0 installation.

Contributor authorship and licensing are retained. The Linux media change keeps @gubiq's original authorship; the A129 record-layout regression credits @alinnastac's published work. Codex assists with implementation and preparation; my hands-on tests and other contributors' tests remain separately attributed. PR #20 remains open for context; unrelated full-fork features are not included here.
