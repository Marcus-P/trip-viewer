# Trip Viewer project state

This file is the durable hand-off for future Codex/ChatGPT sessions. Keep it current and project-only. Do not add personal information, secrets, or machine-specific paths.

## Repository / upstream

- This repository is a fork of `chrisl8/trip-viewer`.
- Preserve upstream compatibility where reasonable and keep generally useful fixes suitable for upstream contribution.
- The Tauri identifier remains `com.tripviewer.app`. Do not change it as incidental cleanup.
- Upstream-oriented changes should be isolated from fork-specific release/branding changes.

## Stable baseline

PR #2 from contributor `gubiq` was reviewed and merged into `main` with a merge commit so the contributor's commits/authorship were preserved.

That PR:
- fixed startup when the sidebar was collapsed,
- uses host libva in Linux AppImages where appropriate,
- disables the WebKitGTK DMABuf sink by default while respecting an explicitly set environment value.

Real-world Ubuntu testing confirmed that the prior video flicker disappeared after PR #2. Keep that fix unless a demonstrated regression requires changing it.

A follow-up to `gubiq` should be drafted in English after PR #3 is merged. The user explicitly requested a proposed message for review first; do not post it without a further instruction to send.

## Completed stabilization: PR #3 / 0.5.0

- PR: #3, **Stabilize WebKit multi-camera seeking and recovery**
- Branch: `fix/webkit-sync-stability`
- Merged into `main` with a merge commit for version 0.5.0; source head `9453be2`.
- The user confirmed both final tests (A → B → A during GPS analysis, and app restart before completion), confirmed successful playback, and explicitly authorized the merge.
- The versioned head passed both complete Ubuntu CI runs, including version-policy checks, 65 frontend tests, 326 Rust tests (one existing ignored), and AppImage packaging: https://github.com/Marcus-P/trip-viewer/actions/runs/39580478558 and https://github.com/Marcus-P/trip-viewer/actions/runs/39580486824.
- The merge changes `RELEASE_VERSION` to 0.5.0 and triggers the existing Ubuntu Release workflow. Release-build completion/publication is checked separately; do not confuse a merged PR with an already available release artifact.
- New GPS performance work belongs on a separate branch from `main`; the investigation below changes no decoder behaviour.

Earlier pending-test/unmerged notes below document intermediate states; this completed status supersedes them.

### Playback synchronization work already implemented

- Coarse WebKit slave reconciliation instead of per-frame `currentTime` correction.
- Transactional seek behavior: pause channels, seek, wait with bounded timeouts, then resume together.
- Segment/source transition seeks happen before autoplay.
- Timeline drag previews locally and performs one real seek on release instead of seek-thrashing on every pointer move.
- Watchdog recovery re-anchors stalled slaves.
- Seek/metadata waits have timeouts so lost WebKit events cannot hang transport forever.
- Runtime media diagnostics can be enabled without rebuilding.
- The first-GPS-fix map behavior no longer triggers an unnecessary animated zoom during playback.

### Per-camera failure behavior

A real-world user test **succeeded** for the channel-local failure design:

- a corrupt Front stream showed its error overlay,
- Interior and Rear continued playing,
- the healthy cameras remained synchronized.

This behavior is generic per channel, not Front-specific. Any failed channel is removed from the active sync group while its panel remains visible with the error state. If the current sync master fails, the next healthy channel becomes master. If another camera subsequently fails, the same logic applies recursively. A new source/segment gets a fresh attempt for every camera.

Do not regress this into “one camera error stops the whole trip.”

### Proven corrupt-media case

The motivating sample is a 180-second, 3840x2160, H.264, 30 fps VIOFO front clip.

Independent diagnostics established:
- FFmpeg decodes test windows through approximately 88 seconds.
- Test windows from approximately 89 seconds onward fail, including much later positions.
- VLC visibly shows corrupted image data later in the same clip.
- The failure still occurs with VA/VAAPI decoders disabled.

Conclusion: this is a real source/bitstream corruption case, not merely a GPS event, synchronization bug, or hardware-decoder-only fault.

### Adaptive recovery / media health

The branch now contains follow-up support for channel-local recovery and persistence:

- failed cameras are probed independently while healthy cameras keep playing,
- retry backoff is 1s → 2s → 4s → 8s → 16s → 30s,
- when a camera becomes decodable again, it rejoins at the current master position as a slave,
- a promoted replacement master remains master until the next source boundary,
- per-file bad intervals are stored in `media_health_ranges`,
- cached health is fingerprinted using file size + modification time so replaced/repaired files invalidate stale results,
- known bad intervals can be reused on later playback instead of rediscovering the same failure every time,
- closed bad ranges wait until their known end before a verification probe rather than hammering the decoder.

The earlier “guess and skip 2/5 seconds” recovery strategy is obsolete and should not be reintroduced.

The channel-local failover behavior has been manually validated. The newer adaptive rejoin/cache behavior should still be treated as needing explicit real-world validation unless a later entry records that test.


### Follow-up audit: media health and GPS persistence (2026-10-20)

The audit preserved the already user-validated per-camera degradation and the Linux libva/DMABuf workarounds. The persistent trip-select GPS path, generic failover, adaptive delay values, and size/mtime health lookup already existed; they were extended rather than reimplemented.

Corrections:
- Recovery uses a separate muted probe pipeline. Repeated failures no longer reload/seek the visible failed camera; that pipeline is retried only after the background probe succeeds.
- A probe accepts a new video frame callback or an increase in presented-frame counters. WebKitGTK under Xvfb was observed to decode/present frames without delivering the callback. Readiness events and a moving clock alone are insufficient evidence.
- Source changes cancel outstanding probes and their frame listeners/timers. A normal `load()` abort of the previous resource is no longer treated as failure of the new probe.
- Closed cached ranges suppress probes while the playhead remains inside them. Probes outside them retain exponential backoff; continued failures outside a recorded range are persisted as new open ranges. A backward seek before a failure cannot close that later interval.
- Cached health is read before attaching the visible media source. Recovery updates the session's open interval as well as the DB, avoiding immediate re-isolation from a stale in-memory range.
- Batched independent failures and rejoin preserve the promoted clock. Recovery remains possible while transport is paused after every camera has failed; a same-segment timeline seek can update that paused recovery target even without an active engine.
- Migration 0015 adds `trip_gps.source_signature`. The normal `ensure_trip_gps` cache check combines parser version with segment ordering/durations, archive-relative paths, file size/mtime, and matching persisted media-health intervals. Current cache hits stat files and read DB metadata; they do not parse MP4 contents. Parser version remains v8 because the decoder semantics did not change.
- Persisted partial or full camera failures exclude that file from the segment's GPS candidates; healthy siblings still compete on usable GPS quality. Repaired/replaced fingerprints invalidate the old exclusion and GPS cache.
- Archived GPS is retained when originals are unavailable, including stale-parser/forced-refresh cases and archived spans of mixed trips. GPS-less/tombstone segment durations remain on the trip time axis.
- Concurrent frontend GPS refreshes are serialized per trip so earlier failure responses cannot overwrite later, more complete exclusions.

Cloud validation:
- `npm test`: 54 tests passed.
- `npm run build`: passed. Existing mixed-import and chunk-size warnings remain intentionally deferred.
- `cargo test --manifest-path src-tauri/Cargo.toml --locked`: 318 passed, one existing test ignored, no failures. Coverage includes persistent DB reopen, parser/size/mtime/health invalidation, archived spans, camera isolation, recovery cancellation and backoff.
- `NO_STRIP=true npm run tauri -- build --bundles appimage`: passed on Debian 13 using workspace-local native dependencies and the bundler's extraction mode. This is local Linux build evidence, not a new Ubuntu 24.04 CI result or a release upload.
- Headless WebKit smoke: the real app rendered, Tauri IPC responded, a scratch archive opened/closed, and a synthetic H.264 clip successfully passed the background decode probe. The cloud's read-only system paths required local runtime relocation helpers; those are environment setup, not application changes.

Still requires the final real-world user test: isolate a corrupt channel while healthy cameras remain synchronized, verify temporary recovery/rejoin without changing the promoted clock, cross a segment boundary, restart and reuse known health/GPS caches, and confirm GPS fallback to a healthy sibling. The synthetic smoke does not replace this test. PR #3 must remain unmerged pending fresh CI and explicit user confirmation.


### Follow-up: long segment loading and unavailable Play

A real Ubuntu test of the previous changes reported that playback of the problem footage eventually worked, but an early segment transition stayed on Loading for a long time and Play later became unavailable. The user confirmed that Play was greyed out and not clickable, consistent with the engine still waiting for camera readiness. Treat that as a regression report, not as successful completion of the final manual test. The supplied console output contains extensive GPS extraction but no timestamped transport-state trace, so it does not prove every instance of unavailable Play has the same cause.

The audit found a concrete blocking path: `ensure_trip_gps` parsed whole-trip MP4 data while holding the shared archive DB mutex, and the visible camera source waited for a health lookup using that same mutex. GPS work now runs on a blocking worker with only short DB sections before and after extraction. Source/health provenance is rechecked before publication; a changed snapshot discards the stale result and retries with a bounded retry count.

Transport now shows background GPS preparation/error status separately from waiting for a playable camera. GPS preparation never controls the Play button. Concurrent refresh status stays busy until the latest request completes, and failed refreshes retain existing GPS. Engine initialization also polls readiness until initialization, so a missing WebKit `loadeddata` event cannot permanently prevent an otherwise ready camera from enabling Play; this poll neither reloads nor seeks the videos.

Local checks for this follow-up: 57 frontend tests and 319 Rust tests passed (one existing ignored test), frontend build passed, and real headless WebKit UI interaction verified Play/Pause and Play after changing segment while the GPS preparation indicator was active. A deterministic concurrency test verifies DB health writes during a deliberately suspended GPS extraction and rejects the stale GPS result after that health change. Local AppImage packaging also passed. Ubuntu CI for source commit `5e8a4cc` is checked separately before reporting readiness.

The user subsequently confirmed that the loading/Play fix works well with the problem footage. Treat that real-world result as established. PR #3 remains unmerged; this confirmation does not authorize merging.

### Progressive GPS preparation

GPS preparation now publishes completed segments to the map and speed graph immediately, together with an actual completed/total segment count and progress bar. GPS-less segments also advance the count; current cached trips still return immediately. The strongest healthy camera remains selected independently for each segment. When VIOFO boundary validation revises the previous segment, that preview is replaced rather than duplicated.

Progress is scoped to each request and extraction attempt. A retry discards its earlier preview; superseded requests cannot overwrite the active preview. Failed refreshes restore the previously committed GPS. Missing original segments retain archived GPS in the preview. The complete trip track is published atomically after source/health revalidation. Durable segment checkpoints were added in the follow-up below; extraction keeps the DB mutex available to playback.

Validation: 62 frontend tests, 322 Rust tests (one existing ignored), and frontend typecheck/build pass, including progressive publication, GPS-less gaps, boundary correction, retry reset, stale events, and failure rollback. The real headless WebKit app verified native progress-channel delivery (start, segment 1, segment 2), no events on a cache hit, working IPC, and decoded video playback. AppImage packaging and Ubuntu CI are checked separately before reporting readiness.

### Follow-up: returning to an unfinished GPS analysis

The user validated progressive GPS counts, Play/Pause (including playback ahead of analysed GPS), and segment transitions on the initial selection. Switching to another trip and back before analysis finished reset the preview and appeared stuck. The supplied terminal output showed ongoing extraction and media requests; it did not establish a database error.

The frontend previously superseded an in-flight request on every selection, discarded its preview, ignored its remaining progress, and queued another invocation behind it. Normal reselection now rejoins the existing promise for the same trip sources, keeping its GPS and count. Explicit health refreshes and changed segment sources still supersede older requests. Returning to a forced refresh joins it without discarding its camera exclusions.

Migration 0016 adds `gps_segment_checkpoints`, separate from the complete `trip_gps` track. Each completed segment saves its raw best-camera selection (including VIOFO UTC and empty/GPS-less results). Checkpoints use parser version plus segment identity, duration, camera kind, archive-relative candidate paths, file fingerprints, and media-health provenance. They survive an interrupted analysis and DB reopen, so a later selection reuses completed segments and parses only unfinished/changed ones. The saved raw selection precedes neighbour validation: stitching always revalidates VIOFO boundaries against the current neighbours. Paths are archive-relative; trip deletion cascades to checkpoints. Parsing and JSON encoding/decoding stay outside the DB mutex, with short read/write sections and a source/health recheck before each save. The full trip track is still committed atomically after whole-trip revalidation.

Local automated validation: 65 frontend tests and 326 Rust tests pass (one existing ignored), plus frontend typecheck/build. Coverage includes A → B → A without duplicate invocation or progress reset, returning to forced refreshes, interrupted analysis and DB reopen, GPS-less gaps, targeted source/parser/health invalidation, and raw VIOFO boundary revalidation after a neighbour changes. The real headless WebKit app decoded video and exercised A → B → A after the first GPS segment, reused the same active promise and progress, and completed all 256 synthetic segments; all 256 checkpoints were present in the on-disk archive DB at schema v16. Local AppImage packaging and fresh Ubuntu CI are checked separately before reporting readiness.

### Obtaining the local Ubuntu test version

Publishing the Codex environment does not update an installed desktop app. CI now uploads a downloadable test artifact after the Ubuntu build and desktop-integration checks pass. Open the latest successful CI run for `fix/webkit-sync-stability` (or PR #3) and download its `Trip-Viewer-Ubuntu-24.04-<commit>` artifact. Artifacts are retained for 30 days and require a GitHub login to download.

Extract the ZIP, run `sha256sum -c SHA256SUMS.txt`, and launch the included `Trip.Viewer_*_test-*_Ubuntu-24.04_amd64.AppImage` after making it executable. The filename and `BUILD_INFO.txt` identify the tested commit; the in-app version follows `package.json` (0.5.0 for the PR #3 release). The installer helper is included if the user wants to replace their launcher installation after testing. This path neither merges PR #3 nor creates a stable GitHub Release.

## Version and release policy

`docs/VERSIONING.md` defines MAJOR.MINOR.PATCH and synchronized version files.
Every PR merged into `main` increments once: patch for fixes/maintenance, minor
for backwards-compatible features, major for incompatible changes, resetting
lower components. PR #3 is 0.5.0 because of the progressive/resumable GPS feature.
CI validates file agreement and a valid increase against the PR base or main's
previous commit. Changing `RELEASE_VERSION` on main triggers the existing Ubuntu
release workflow. A completed merge and a published release are separate states.

## GPS performance investigation

The user reports slow analysis per segment; the supplied log has no per-file
elapsed timings, so no measured speedup or single proven bottleneck is claimed.
The current implementation parses each candidate camera sequentially per segment.
The VIOFO reader uses unbuffered `File` reads and seeks to the GPS descriptors
and record positions. MP4 metadata probing passes a `File` to `mp4::read_mp4`,
but the library itself already wraps it in an 8 KiB `BufReader`. Concurrent video playback
and an external rotating disk can make more I/O workers counterproductive.

Next steps, separately from the validated PR #3 merge: measure metadata probing,
GPS reads and validation independently; benchmark VIOFO read/seek coalescing
and data-block access, then bounded file-level parallelism (initially two workers, shared
across jobs). Keep decoded records in physical source order and assemble results
in segment/candidate order; never sort malformed camera timestamps to hide a
contradiction. Keep best-camera selection/tie-breaking, adjacent VIOFO validation,
checkpoint provenance, short DB locks and the responsive transport unchanged.
Compare parallel and sequential outputs on corrupt timestamps, missing GPS,
late fixes, gaps, source/health changes, restart and cache hits before proposing a
performance change for merge. Do not advertise a speed factor without measurements
on representative media/storage. A behaviour-preserving optimization is 0.5.1.

A cloud microbenchmark used a synthetic 180-second, 30 fps H.264 file, 100
warm-cache metadata parses, and counted underlying `File::read` calls. The
existing 8 KiB-buffered path used 25 reads/parse and about 7.6 ms total; 64 KiB
used 23 reads/parse and about 13.7 ms total. Dimensions and duration matched in
all cases. This does not measure real VIOFO GPS extraction or external-disk I/O,
but it supplies no evidence for increasing the metadata buffer as an optimization.
No extraction or parallelism change is included in 0.5.0.

The latest log reports one segment's three camera files with descriptors but no
decodable GPS records. This is compatible with absent valid fixes (possibly a
short recording), not proof of clip duration or a damaged video. GStreamer
startup/teardown warnings in the same log do not establish a GPS parsing failure.

## GPS architecture

GPS should be treated as trip data, not as a Front-camera-only feature.

Current intended behavior:
- choose the strongest usable GPS source per segment from healthy available camera files,
- prefer canonical order only as a tie-breaker; do not assume Front is the only GPS provider,
- persist stitched trip GPS in the existing `trip_gps` table,
- normal Original playback should use persisted trip GPS as well as timelapse playback,
- do not re-extract all GPS on every application start,
- when the GPS-providing camera for a segment is later found corrupt, rebuild that segment/trip GPS from the remaining healthy camera candidates and persist the repaired result,
- GPS parser/cache version is currently v8 so older Front-only cache rows are recomputed when source files are available,
- archive-only trips must retain usable cached GPS even when source files no longer exist.

The backend already exposes `ensure_trip_gps` and `load_trip_gps`, and `trip_gps` exists in the database. Prefer this persistent path over legacy per-launch `gpsByFile` extraction.

## Historical verification checklist for PR #3

This list guided stabilization; the final focused GPS resume tests and merge authorization are recorded above. Preserve the validation already established and do not assume unreported manual scenarios were individually tested:

- a known corrupt camera becomes isolated without stopping healthy cameras,
- healthy cameras remain synchronized through the failed interval,
- a temporarily bad camera can rejoin when probing later succeeds,
- a known cached bad interval does not cause repeated expensive rediscovery on the next run,
- GPS is available from the persisted cache after restart,
- if the original GPS-providing channel is known bad, GPS comes from a healthy sibling channel where usable,
- segment auto-advance still works with a promoted sync master.

Both final GPS resume tests were confirmed and both Ubuntu CI runs for the versioned head were green before the authorized merge. Earlier pending-test notes are historical.

## On hold / later work

These are intentionally deferred. Do not silently implement them while finishing PR #3.

**Next user priorities (recorded, implementation not yet authorized):**

- **Seek by clicking the map route.** Clicking a point on the displayed route should seek the video and timeline to its corresponding GPS time. Resolve the nearest position on the route (including between samples), preserving segment/trip time mapping; a general map click must not arbitrarily choose an unrelated route point.
- **Collapsible timeline in dashboard fullscreen.** In the F/I/R + Map fullscreen layout, allow the timeline and speed graph area to be expanded/collapsed.

1. **Paused-video zoom/pan.** When an individual camera is paused, allow zooming and panning the current image. Prefer stable video-element zoom/pan; if WebKit proves unreliable, capture/extract the current frame to a temporary still while paused, then return to video on resume.
2. **Vite mixed import warning for `src/ipc/archive.ts`.** The stable cleanup is to use a static `currentArchive` import in `App.tsx` because the module is already statically imported elsewhere. This is cleanup, not a playback fix.
3. **Large Vite chunk warning (~500+ kB).** Investigate separately. It is not considered the cause of the Linux video flicker/synchronization problems.
4. **`cargo-sweep`.** Inspect `scripts/cargo-sweep.mjs`, then install/configure `cargo-sweep` and clean old Cargo artifacts carefully. Do not blindly delete build state.
5. **`NO_STRIP=true`.** Determine its exact effect in the Tauri/AppImage/linuxdeploy path and document concrete release guidance. Do not assume it is the same as Cargo release stripping.
6. **npm audit findings.** Review `npm audit --omit=dev` and full `npm audit`; make targeted changes only. Do not run `npm audit fix` blindly.
7. **Upstream contribution cleanup.** When contributing generally useful fixes back to `chrisl8/trip-viewer`, branch from upstream main and cherry-pick/reapply only upstream-suitable changes rather than submitting the whole fork branch.
8. **Contributor follow-up.** After PR #3 is merged, prepare a concise English follow-up for user review before posting to `gubiq` on PR #2 explaining that his libva/DMABuf fix removed the flicker and that subsequent WebKit synchronization/media-health robustness was added in PR #3.

## Development principles established by the debugging work

- Three separate `<video>` elements are independent media pipelines; application-level synchronization is required.
- On WebKitGTK, frequent `currentTime` corrections can flush GStreamer pipelines. Prefer coarse reconciliation over continuous seek correction.
- A media decode error can be channel-local. Healthy channels should continue.
- Never infer media integrity from GPS availability or vehicle movement timing. GPS timing and video decoding may correlate in time without sharing a cause.
- Cached analysis must be invalidated when the underlying source file changes.
