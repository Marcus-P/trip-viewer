> Historical fork development record through 2026-10-21. Superseded by [current project state](../PROJECT_STATE.md); old pending-test and permission-blocked statements are historical.

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

The approved English follow-up to `gubiq` was posted on PR #2 after the environment network update: https://github.com/Marcus-P/trip-viewer/pull/2#issuecomment-6315937674.

## Active extension: channel visibility / PR #4

The user accepted bookmarks, including persistence across app restart, and authorized the previously deferred channel checkboxes. Front/Interior/Rear (and other available labels) can now be shown/hidden individually. Choices persist in layout preferences across app restart. Only visible cameras determine the visual primary/secondary layout; two-camera layouts fill both columns to full height and a single camera fills the video grid. All-hidden mode gives the map the full viewing area, or shows a recovery hint if no map exists. Checkboxes remain accessible outside the video grid in normal/dashboard mode. Existing video elements and sync/audio master selection are retained through visibility changes; this is display selection, not media deletion or a recording setting.

Validation: 80 frontend tests and typecheck/build, 335 Rust tests (one pre-existing ignored). Native WebKit verified three/two/one/zero visible cameras, full-height two-camera placement, map-only expansion/restoration, saved choices, dashboard checkboxes, Play/Pause with the canonical master hidden, and identical video nodes without `emptied` events. New checkbox behaviour awaits focused user confirmation; bookmark/route/zoom/fullscreen behaviour is already accepted.

### Stable branch protection requested, API permission blocked

The stable/default branch is `main`. The user requested rejecting force-push/forced merges and allowing only the owner to merge. The repository is personal; the collaborators endpoint currently lists only the owner with the admin role. The installed GitHub integration returns HTTP 403 `Resource not accessible by integration` for branch protection reads and ruleset creation despite the account having admin permissions. No protection was installed; do not report it as active.

`scripts/protect-main.sh` and `scripts/github/main-{history,owner}-rules.json` are ready for the owner to run with a GitHub CLI login that grants repository Administration write access. One active, non-bypassable ruleset requires PRs, strict successful `ubuntu` checks and resolved threads, and forbids non-fast-forward updates/deletion. A separate update restriction permits only repository admin role 5 via pull requests; for this personal repository that is the owner. No approval count is imposed that would prevent the owner merging their own PR. The script checks account/ownership/default branch, preserves existing same-named rules, and verifies the stored rules against the payloads; mismatches fail for review. Shell syntax and mocked creation/idempotent rerun/mismatch detection passed. Actual activation/verification requires owner credentials; do not use direct pushes or admin overrides to evade the intended rules.

## Active extension: persistent timeline bookmarks / PR #4

The user explicitly accepted the final dashboard/single-camera Play/Pause visibility: no upper transport in dashboard fullscreen; retain it in single-camera fullscreen. CI for that accepted head (`0e40e90`) passed. The user then authorized timeline bookmarks and previous/next bookmark navigation. This extends the still-open PR #4 on `feat/route-seek-fullscreen-tools`; version remains 0.6.0 because the PR has not been merged. No new merge authorization was given.

Bookmarks use the concatenated trip clock (integer milliseconds), independently of GPS readiness. Right-click graph/timeline to add; left-click a numbered amber marker to seek; right-click that marker to remove. Previous/next arrow-and-bar buttons do not wrap at the ends. `+ Bookmark` saves the current position as an accessible alternative. Selection mode disables editing/navigation. Existing original/tier seek mapping handles segment boundaries and archived timelapses.

Migration 0017 adds `trip_bookmarks` in the per-archive SQLite DB (schema head 17). Trip switching and app restart retain marks; trip deletion cascades, ID rewrites cascade, and manual merges rebase marks on the merged segment clock in the same transaction. Archive-only merges use the original speed-curve clock, falling back to the trip span. Archive-root checks reject stale commands aimed at a different archive; frontend generations discard stale asynchronous responses, writes are serialized, and reload waits for pending saves. Save/load errors are visible rather than presenting an unsuccessful save as persistent. Media originals and GPS cache/parser semantics are unchanged.

Validation: 76 frontend tests and typecheck/build; 335 Rust tests (one pre-existing ignored), including SQLite reopen, sorting/deduplication, isolation/deletion, stale archive rejection, trip merge rebasing and archive-only curve timing. Native WebKit exercised actual timeline right-clicks without seeking, next/previous seeks across a segment boundary, restore after trip switching, and persistent removal via IPC. Fresh CI checks follow the pushed bookmark head. The user subsequently accepted the bookmark feature including persistence across app restart; prior route/fullscreen/zoom behaviour remains accepted.

## Active work: authorized playback tools and GPS analysis / 0.6.0

The user confirmed that the installed 0.5.0 displays the correct version, approved
the contributor comment, requested merging any already-open PRs, and authorized
implementation of all pending user features. GitHub reported zero open PRs; PR #3
was already merged. No additional existing PR needed merging.

Branch: `feat/route-seek-fullscreen-tools`, based on merged 0.5.0 plus the GPS
research notes. The combined new capabilities require MINOR 0.6.0. No merge of
this newly implemented feature branch has yet been requested or performed.

Implemented:
- Route clicking projects onto valid recorded edges at the current map zoom,
  interpolates trip time and uses the same player seek path as the timeline.
  It does not interpolate gaps over 2 seconds, invalid fixes or dateline jumps;
  repeated positions prefer the current traversal. Route position arrays stay
  memoized while the playhead moves. Archived tiered seeks work with empty
  original segment lists and use the existing speed curve.
- Dashboard fullscreen uses the complete player as its browser fullscreen root,
  including the timeline/speed graph. A persistent collapse/expand preference
  affects dashboard mode; single-camera mode hides the lower strip, and back
  navigation restores the dashboard. Videos stay mounted through these changes.
- Paused video wheel/button zoom and pointer-drag pan operate on the same video
  element, constrain the actual letterboxed image, and reset on resume/source
  change. No transform is applied during normal playback. Pointer cancellation
  and lost capture stop dragging; zoom controls do not change camera selection.
- GPS candidate files and batch jobs use one shared Rayon pool of at most two
  readers. Indexed results retain source/candidate order and canonical ties;
  boundary validation remains ordered. A serial, locked fallback handles thread
  creation failure. Cached segments do not submit jobs. VIOFO descriptors use
  a 16 KiB buffer confined to the descriptor box, preserving descriptor order.
  Parser semantics/version and DB schema remain v8/v16. Optional environment
  timing/worker controls are documented in `RELEASE_NOTES.md`.

Validation so far: 74 frontend tests, 331 Rust tests (one existing ignored), and
frontend typecheck/build pass. Native headless WebKit verified an actual SVG
route click across a segment boundary, real X11 pointer zoom/pan while paused,
unchanged video source/time during zoom, dashboard timeline collapse/expand,
single-camera/dashboard back navigation, Play/Pause and fullscreen exit. No
video nodes were replaced and no media `emptied` events occurred through zoom
or fullscreen changes. A three-file native GPS batch handled missing GPS.
Ubuntu AppImage CI passed for `8817eaa`: https://github.com/Marcus-P/trip-viewer/actions/runs/39591307803. The user subsequently confirmed cross-segment map seeking, dashboard fullscreen timeline and paused zoom on real footage. GPS analysis may be roughly 2–5 seconds faster than the previous ~20-second segment baseline, but this was not measured precisely; the user accepts the observed performance with no reported regression. Do not claim a measured speedup factor.

The user requested removing the redundant upper Play/Pause button in dashboard fullscreen, where the lower transport already supplies it. The upper control now appears only in single-camera fullscreen, where the timeline/transport strip is hidden. The UI follow-up passed 74 frontend tests, typecheck/build and 331 Rust tests (one pre-existing ignored). The validated media/GPS code is unchanged.

## Completed stabilization: PR #3 / 0.5.0

- PR: #3, **Stabilize WebKit multi-camera seeking and recovery**
- Branch: `fix/webkit-sync-stability`
- Merged into `main` with a merge commit for version 0.5.0; source head `9453be2`.
- The user confirmed both final tests (A → B → A during GPS analysis, and app restart before completion), confirmed successful playback, and explicitly authorized the merge.
- The versioned head passed both complete Ubuntu CI runs, including version-policy checks, 65 frontend tests, 326 Rust tests (one existing ignored), and AppImage packaging: https://github.com/Marcus-P/trip-viewer/actions/runs/39580478558 and https://github.com/Marcus-P/trip-viewer/actions/runs/39580486824.
- Merge commit `f9681ea` is on `main`; its complete CI passed: https://github.com/Marcus-P/trip-viewer/actions/runs/39581960886.
- The Ubuntu release workflow succeeded, including tests, AppImage packaging, desktop integration, license inventories, checksums and GitHub publication: https://github.com/Marcus-P/trip-viewer/actions/runs/39581960789. Version 0.5.0 is published at https://github.com/Marcus-P/trip-viewer/releases/tag/v0.5.0.
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

Initial research notes are on `perf/gps-extraction`. Implementation now continues
in `feat/route-seek-fullscreen-tools` as part of the user-authorized 0.6.0 feature
bundle. The investigation below records the starting evidence and test criteria.

The user reports about **20 seconds per 3-minute segment**, with the originals
on an **external SSD connected over USB**. This is an approximate end-to-end
segment measurement, not a per-file/stage timing. The supplied log has no
per-file elapsed timings, so no measured speedup or single proven bottleneck is
claimed. Do not assume mechanical HDD seek cost explains this SSD report.
The current implementation parses each candidate camera sequentially per segment.
The VIOFO reader uses unbuffered `File` reads and seeks to the GPS descriptors
and record positions. MP4 metadata probing passes a `File` to `mp4::read_mp4`,
but the library itself already wraps it in an 8 KiB `BufReader`. Concurrent video playback
and an external rotating disk can make more I/O workers counterproductive.

Next steps, separately from the validated PR #3 merge: measure metadata probing,
GPS reads and validation independently; benchmark VIOFO read/seek coalescing
and data-block access, then bounded file-level parallelism (initially two workers, shared
across jobs). Compare one versus two workers on USB SSD during real playback;
record per-file probe/read/decode/validation time, whole-segment latency, peak
memory, and transport responsiveness before selecting a default. Keep cached
segments on the fast path without submitting unnecessary worker jobs. Keep decoded records in physical source order and assemble results
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

**Channel visibility checkboxes:** subsequently authorized and implemented in PR #4; see the active extension above.

The user features below were subsequently authorized and implemented in the 0.6.0 branch described above. Maintenance items remain tracked separately.

**User features implemented in the active 0.6.0 branch (user validation confirmed above):**

- **Seek by clicking the map route.** Clicking a point on the displayed route should seek the video and timeline to its corresponding GPS time. Resolve the nearest position on the route (including between samples), preserving segment/trip time mapping; a general map click must not arbitrarily choose an unrelated route point.
- **Collapsible timeline in dashboard fullscreen.** In the F/I/R + Map fullscreen layout, allow the timeline and speed graph area to be expanded/collapsed.

1. **Paused-video zoom/pan.** Implemented using the existing video element; native WebKit tests passed. No temporary image derivative was needed.
2. **Vite mixed import warning for `src/ipc/archive.ts`.** The stable cleanup is to use a static `currentArchive` import in `App.tsx` because the module is already statically imported elsewhere. This is cleanup, not a playback fix.
3. **Large Vite chunk warning (~500+ kB).** Investigate separately. It is not considered the cause of the Linux video flicker/synchronization problems.
4. **`cargo-sweep`.** Inspect `scripts/cargo-sweep.mjs`, then install/configure `cargo-sweep` and clean old Cargo artifacts carefully. Do not blindly delete build state.
5. **`NO_STRIP=true`.** Determine its exact effect in the Tauri/AppImage/linuxdeploy path and document concrete release guidance. Do not assume it is the same as Cargo release stripping.
6. **npm audit findings.** Review `npm audit --omit=dev` and full `npm audit`; make targeted changes only. Do not run `npm audit fix` blindly.
7. **Upstream contribution cleanup.** When contributing generally useful fixes back to `chrisl8/trip-viewer`, branch from upstream main and cherry-pick/reapply only upstream-suitable changes rather than submitting the whole fork branch.
8. **Contributor follow-up.** Completed: the approved follow-up was posted to `gubiq` on PR #2; see the comment URL above.

## Development principles established by the debugging work

- Three separate `<video>` elements are independent media pipelines; application-level synchronization is required.
- On WebKitGTK, frequent `currentTime` corrections can flush GStreamer pipelines. Prefer coarse reconciliation over continuous seek correction.
- A media decode error can be channel-local. Healthy channels should continue.
- Never infer media integrity from GPS availability or vehicle movement timing. GPS timing and video decoding may correlate in time without sharing a cause.
- Cached analysis must be invalidated when the underlying source file changes.
