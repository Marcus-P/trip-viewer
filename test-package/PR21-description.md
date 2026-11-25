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
- A real, opt-in FFmpeg integration test creates two synthetic HEVC clips, runs the actual 8x generation path and decodes the finished output. This passed locally without owner footage. It does not certify a complete real three-channel batch or NVIDIA hardware support.

### Playback and archive compatibility

- Reuse the fork's playback intent, canonical audio, seek barriers and channel-local failure/recovery handling. Retain existing media elements across application-view and fullscreen transitions.
- Fullscreen uses one shared viewer root including transport. Play/Pause stays under the SyncEngine instead of native single-video controls. Returning from fullscreen re-anchors the existing cameras without rebuilding their media pipelines.
- Retain the Linux host-libva/DMABuf compatibility handling and explicitly include the GL media component in Linux builds. The previous isolated AppImage lacked the GL plugin and WebKit reported its accelerated sink dependencies missing.
- **The main archive schema and migration head remain 13. No migration files are added or changed.** Additional analysis lives beside the archive in `.tripviewer/analysis-cache-v1.sqlite`, not in a periodically cleared OS temporary cache. Tests compare the main schema and version before/after sidecar writes and reopen the main archive at schema 13. This is a schema compatibility check, not a packaged test of every older application. Existing archived GPS can seed the sidecar. This does not claim that schema-17 full-fork archives are readable by an unchanged schema-13 application.

The new commits separate review corrections, recording-mode/RO fixes, the original Linux media contribution, the related runtime reliability changes and build/test guidance. There is no intermediate schema upgrade in these new commits. Final scope and merge decisions remain with the upstream maintainer.

### Response to the maintainer's review

This addresses the numbered requests in [@chrisl8's review comment](https://github.com/chrisl8/trip-viewer/pull/21#issuecomment-7149083524), with one explicit alternative to the optional suggestion:

| Review request | Implementation and evidence |
| --- | --- |
| 1. Redundant iterator / zero-warning Clippy | The decoder now uses `.zip(validated)`. The recorded final Clippy run passed with warnings denied. |
| 2. Corrupt-year overflow | The year offset uses `checked_add`; `corrupt_years_do_not_overflow_or_invent_a_timestamp` covers values near `i32::MAX` and `u32::MAX`. Invalid dates remain unavailable rather than being invented. |
| 3. Rust formatting and parser/test placement | Changed Rust modules were formatted using the repository edition. `ViofoParser` and `viofo_integration_tests` precede the existing `mod tests` in `scan/naming.rs`. Unrelated inherited formatting differences were kept out; this is not a claim that every untouched upstream file was reformatted. |
| 4. Non-VIOFO shared dispatch | `non_viofo_points_and_request_order_pass_through_unchanged` checks 70mai, Wolf Box and Generic requests, preserving their output values and order through the shared batch path. This is an automated dispatch regression, not a new real-footage test of those cameras. |
| 5. Parser collisions | `viofo_does_not_collide_with_other_parsers` covers Wolf Box, Thinkware, Miltona, 70mai and Generic filenames. |
| 6. Parking versus normal and `is_event` | Parking/normal and protected/unprotected modes remain distinct during grouping and sibling lookup. Only protected modes set `is_event`; parking alone does not. Import retains `RO` path context. Regressions cover normal/parking and protected variants, timestamp skew across minute boundaries and interleaved modes. These are synthetic checks; I have no parking recordings for a hands-on test. |
| 7. Optional small-payload rejection | **Alternative implementation:** a hard 4-KB limit rejected valid padded GPS containers in my recordings. The reader instead uses a fixed 4-KB buffer with overlapping reverse windows, checks file/container bounds and terminates as the read position decreases. Memory does not grow with the declared payload size; work remains bounded by the container size. This preserves valid records, including those crossing window boundaries. |

The A129 layout regression is adapted from @alinnastac's published test, with source credit and synthetic telemetry in place of the original recording's time/location. Existing incorporated contributor commits retain their authorship. PR #20 remains open as requested; this update neither closes nor replaces it.

**Remaining review and validation boundaries:** the related playback/GPS reliability work makes this PR broader than its original camera-decoder scope. I included it because the camera-only candidate was not usable on my tested setup, while leaving unrelated features separate. The expanded scope and the alternative to request 7 still need the maintainer's review; the original review is not approval of these later changes. A current A129 full-application check is also still outstanding, and the earlier 15-point diagnostic difference remains unexplained. I am not presenting that difference as evidence of better filtering or claiming that the pending A129 check is a new maintainer-imposed merge condition.

### Validation and limits

- Rust library tests: **386 passed, 2 ignored in the default run** (one existing ignored test plus the opt-in FFmpeg test; the latter was run explicitly and passed), no failures.
- Frontend: **78 tests passed**; TypeScript and zero-warning Clippy pass. Changed Rust modules are formatted for the repository's edition.
- Main-schema compatibility and checkpoint/recovery regression tests pass. Current upstream main is included; freshness is checked again at handoff.
- I tested the isolated Ubuntu GPS candidate on the same A229 Pro 3CH recordings: all 39 segments completed, producing 6,450 points, and I confirmed visible tracks, the manually merged track and the moving marker. That initial GPS check preceded the playback checks below.
- The subsequent Ubuntu AppImage build/startup and bundled GL-factory checks passed. I have now also tested that isolated revision on my Ubuntu 24.04 notebook with my A229 Pro 3CH recordings: Original at x1, Play/Pause, seeking and fullscreen transitions worked. GPS tracks, merged-trip tracks and the moving marker are visible. I did not observe the previous large route jumps at the locations I checked. This is acceptance of those checks on that setup, not universal camera, GPU, driver or long-duration validation.
- My real Front/8x CPU timelapse finished successfully. I watched the entire generated video and confirmed clean playback. This validates that output on my Ubuntu setup, not the remaining camera jobs, other tiers, A129 recordings or other operating systems. The present progress bar counts completed jobs, not live FFmpeg frames: after the first of three jobs it shows one third while the next encode runs. Additional live progress and GPU-performance work from the full fork remains separate.
- The earlier A129 diagnostic comparison had 15 fewer output points. Those exclusions remain an open validation question; I am not claiming that fewer points prove better filtering. A decoder diagnostic does not validate complete application playback. The credited synthetic A129 record layout test is separate evidence.
- Channel association still uses the existing nearby-timestamp heuristic. It is not a GPS-proven recording identity or a universal synchronization guarantee. No parking footage is required for the available playback check.

### Existing full-fork additions outside this PR

The following additions and refinements already exist in my released v0.13.0 fork but are **not included in this PR**. This is an overview for possible later contributions, not an agreed roadmap or a promise to submit every item. The basic player, map, import, timelapse generation and updater already existed upstream. Necessary playback/GPS reliability and the runtime NVENC/CPU verification described above are already included here.

**Playback, map and timeline tools**

- Route-click seeking across segments, with handling for GPS gaps and repeated route positions.
- Refined map following and resize behavior, including initial zoom handling to reduce unnecessary rendering work.
- Persistent numbered bookmarks, previous/next navigation and position adjustment when trips are merged.
- Zoom, pan and reset while video is paused; an extended fullscreen dashboard with map/timeline controls and a collapsible timeline. Basic shared fullscreen is already in this PR.

**Preferences and everyday usability**

- General settings, km/h or mph selection and an optional manual camera preference alongside automatic detection.
- Saved camera visibility, map-only viewing and responsive layouts for one, two or three visible cameras.
- Resizable camera/map areas and timeline height, saved layout and sidebar collapse.
- Separately remembered archive and import-source picker locations.

**Optional display composition**

- Optional overlay layouts with cabin/rear-camera layers, mirror-style frames and masks.
- Crop, move and resize controls with a saved layout editor. These compose the application display, not an exported composite video; the classic layout remains available.

**Timelapse performance, feedback and robustness**

- Additional AMD AMF, Intel QSV and Linux VA-API encoding paths with runtime capability/device checks and fallback.
- Hardware decoding/scaling profiles and device-resident paths where supported, with bounded buffers and fallback; experimental VideoToolbox work does not have hands-on macOS validation.
- Short representative performance calibration, cached local profiles, and thread/concurrency/memory tuning. The aim is useful throughput, not a guaranteed GPU-utilization percentage.
- Live per-phase/output progress and more informative time estimates, beyond the current completed-job counter.
- Additional cancellation/retry, source-probe and memory-budget handling, including retaining a previous successful output until its replacement succeeds.

**Updates, installation and help**

- Extended release information and platform-matched download handling, installation/help/report links and release-asset download counts. Download counts are not unique users or application telemetry.
- Additional Ubuntu package dependencies/desktop integration and Windows installer upgrade/recovery handling. Experimental universal macOS packaging is distinct from hands-on media validation.

Some features have storage or design implications, notably persistent bookmarks; they would need their own compatibility review. The full fork's newer archive schema is not part of this PR. Fork branding, signing/release configuration and owner-specific agent rules are not proposed for upstream adoption.

Consolidating general settings and the timelapse FFmpeg gear into one consistent settings flow is a **proposed refinement**, not an existing v0.13.0 feature. Once this foundation is accepted, @chrisl8 can choose, reorder or combine useful follow-ups.

### Building and testing

[Complete Windows and Ubuntu instructions](https://github.com/Marcus-P/trip-viewer/blob/contrib/viofo-camera-only/docs/VIOFO_TESTING.md) include prerequisites, exact commands, the source commit to record, a separate test application identity, disabled test updater endpoints and a small optional playback check. Windows users can build a native executable without installer signing. The manual Ubuntu acceptance package is a prerelease, not Latest, and does not update the full fork's stable v0.13.0 installation.

Contributor authorship and licensing are retained. The Linux media change keeps @gubiq's original authorship; the A129 record-layout regression credits @alinnastac's published work. Codex assists with implementation and preparation; my hands-on tests and other contributors' tests remain separately attributed. PR #20 remains open for context; unrelated full-fork features are not included here.
