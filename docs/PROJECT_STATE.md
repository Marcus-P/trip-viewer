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

A follow-up thank-you/comment to `gubiq` is intentionally deferred until PR #3 is fully finished and manually validated. His public GitHub communication is predominantly English, so English is the default language for that final follow-up.

## Active work: PR #3

- PR: #3, **Stabilize WebKit multi-camera seeking and recovery**
- Branch: `fix/webkit-sync-stability`
- Base: `main`
- CI before the follow-up audit was reported green, including the Ubuntu AppImage build. The follow-up changes below require fresh CI; local cloud validation is recorded separately.

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

## Immediate verification goal

Before merging PR #3, manually verify the latest branch, especially:

- a known corrupt camera becomes isolated without stopping healthy cameras,
- healthy cameras remain synchronized through the failed interval,
- a temporarily bad camera can rejoin when probing later succeeds,
- a known cached bad interval does not cause repeated expensive rediscovery on the next run,
- GPS is available from the persisted cache after restart,
- if the original GPS-providing channel is known bad, GPS comes from a healthy sibling channel where usable,
- segment auto-advance still works with a promoted sync master.

Do not merge PR #3 until the user explicitly confirms the relevant final test.

## On hold / later work

These are intentionally deferred. Do not silently implement them while finishing PR #3.

1. **Paused-video zoom/pan.** When an individual camera is paused, allow zooming and panning the current image. Prefer stable video-element zoom/pan; if WebKit proves unreliable, capture/extract the current frame to a temporary still while paused, then return to video on resume.
2. **Vite mixed import warning for `src/ipc/archive.ts`.** The stable cleanup is to use a static `currentArchive` import in `App.tsx` because the module is already statically imported elsewhere. This is cleanup, not a playback fix.
3. **Large Vite chunk warning (~500+ kB).** Investigate separately. It is not considered the cause of the Linux video flicker/synchronization problems.
4. **`cargo-sweep`.** Inspect `scripts/cargo-sweep.mjs`, then install/configure `cargo-sweep` and clean old Cargo artifacts carefully. Do not blindly delete build state.
5. **`NO_STRIP=true`.** Determine its exact effect in the Tauri/AppImage/linuxdeploy path and document concrete release guidance. Do not assume it is the same as Cargo release stripping.
6. **npm audit findings.** Review `npm audit --omit=dev` and full `npm audit`; make targeted changes only. Do not run `npm audit fix` blindly.
7. **Upstream contribution cleanup.** When contributing generally useful fixes back to `chrisl8/trip-viewer`, branch from upstream main and cherry-pick/reapply only upstream-suitable changes rather than submitting the whole fork branch.
8. **Contributor follow-up.** After PR #3 is complete and validated, post a concise English follow-up to `gubiq` on PR #2 explaining that his libva/DMABuf fix removed the flicker and that subsequent WebKit synchronization/media-health robustness was added in PR #3.

## Development principles established by the debugging work

- Three separate `<video>` elements are independent media pipelines; application-level synchronization is required.
- On WebKitGTK, frequent `currentTime` corrections can flush GStreamer pipelines. Prefer coarse reconciliation over continuous seek correction.
- A media decode error can be channel-local. Healthy channels should continue.
- Never infer media integrity from GPS availability or vehicle movement timing. GPS timing and video decoding may correlate in time without sharing a cause.
- Cached analysis must be invalidated when the underlying source file changes.
