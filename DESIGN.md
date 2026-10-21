# Trip Viewer architecture

This is the current architecture of the [Marcus-P fork](https://github.com/Marcus-P/trip-viewer)
of [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer). The original
Tauri/React player, import pipeline, camera support, scans and timelapse work
remain upstream contributions. The [inherited design document](docs/history/inherited-DESIGN.md)
is preserved, including its original rationale and observations; its packaging,
updater and roadmap statements do not describe this fork automatically.

## Components

| Area | Current implementation |
| --- | --- |
| Desktop host | Tauri v2 / Rust; system WebView2, WKWebView or WebKitGTK by platform |
| UI and state | React 19, TypeScript, Zustand, Tailwind CSS |
| Playback | Stable HTML video elements, canonical channel clock, channel-local failover |
| Metadata | Pure Rust MP4 probing; no required ffprobe subprocess |
| GPS | Wolf Box ShenShu, Miltona `gps0`, VIOFO descriptor decoder; Thinkware GPS not implemented |
| Map | Leaflet/react-leaflet with OpenStreetMap tiles and gesture-aware following |
| Persistence | Per-archive SQLite, schema 17; app settings JSON and webview layout preferences |
| Imports | Upstream staging, SHA-256 verification and guarded source-wipe pipeline |
| Analysis | Tag scans, places, Symphonia audio analysis, review and issues views |
| Timelapse | Optional user-provided ffmpeg; per-channel speed curves and coverage |
| Fork packaging | Ubuntu 24.04 x86_64 AppImage and Debian packages; published-release hints, user-confirmed installs |

Hardware acceleration is provided by the available platform media stack, not a
universal guarantee. Windows/macOS code is retained, but this fork’s hands-on
validation is limited to VIOFO A229 Pro 3CH on Ubuntu 24.04 LTS.

## Player and clocks

`PlayerShell` owns source/trip mapping and the sync engine. `VideoGrid` retains
channel DOM order and changes CSS placement/visibility. Visual primary selection
and camera checkboxes do not replace video elements or choose a new sync clock.
The promoted master remains stable within a source when another camera recovers.

Original playback maps segment-local file time to concatenated trip time.
Timelapse uses `speedCurve.ts` to map file/concat time and identify channel gaps.
Timeline, map and bookmark seeks share the trip-time seek path. Avoid repeated
fine-grained currentTime assignments: on WebKitGTK these can flush pipelines.

Fullscreen uses one player root with logical dashboard/video views; the timeline
can collapse in dashboard mode. Zoom transforms are applied only to paused media
and reset on playback/source changes. Hidden channels retain their media elements.

## GPS preparation and recovery

Camera-specific extraction dispatch preserves the existing camera kinds; VIOFO
is distinguished by its A229 filename within the persisted Generic kind. Segment
candidates retain canonical tie-breaking. A shared pool caps candidate/batch reads
at two workers; ordered boundary validation follows extraction. Descriptor reads
are buffered. GPS parser version is 8, independent of the application/schema version.

Progressive updates publish completed segments and corrections to their left
neighbour. Raw per-segment checkpoints are persisted before stitching so boundary
corrections do not accumulate after restart. Cache checks include source and
health signatures; parsing runs outside long SQLite lock sections. Concurrent
trip selections can rejoin existing work. GPS absence does not stop playback.

Media-health records fingerprint source files. Failed cameras are removed from
the active sync group, with independent recovery probes and bounded backoff.
Healthy channels continue; recovered cameras rejoin as slaves during the source.
The Linux libva/DMABuf work from contributor `gubiq` remains in place.

## Storage and data integrity

An `ArchiveSlot` selects the current `DbHandle`; background workers retain their
own handle so switching archives does not redirect an existing worker’s writes.
Archive-relative paths are resolved through `paths.rs`. Schema migrations reject
newer archives rather than attempting an unsafe downgrade.

Bookmarks store integer milliseconds on the trip clock, scoped to the archive.
Archive-root checks and frontend response generations prevent stale UI updates;
writes are serialized. ID changes cascade; trip merges rebase bookmarked moments
transactionally. Original footage is not modified to store bookmarks or display preferences.

Import safety invariants are in [AGENTS.md](AGENTS.md) and
[CLAUDE.md](CLAUDE.md). Preserve upstream verify-before-wipe and cancellation
behaviour. User-facing scan/import/deletion scopes are documented in the
[user guide](docs/USER_GUIDE.md).

## Timelapse and distribution boundaries

The current persisted curve envelope is **v1**; see
[timelapse alignment status](TIMELAPSE_ALIGNMENT_REDESIGN.md). An inherited v2
proposal is not a claim that this checkout writes explicit v2 fileStart/fileEnd.

`RELEASE_VERSION` on protected `main` triggers the Ubuntu release workflow.
The upstream updater endpoint/key is not used by this fork. A bounded frontend
check reads the fork’s latest published GitHub release and offers a same-repository
Ubuntu installer download. It does not install packages automatically or offer
downgrades. The navigation hint and Settings share a cache; network failure does
not affect the playback state. NSIS/DMG target code
remains, but the fork does not claim tested Windows/macOS packages. Flatpak is
not an implemented delivery channel. See [releasing](RELEASING.md).

## Future work

Unimplemented ideas remain proposals: alternative audio source, mirrored video,
clip/snapshot/GPX exports and broader hardware/platform validation. Camera
visibility, bookmarks and route seeking were released in 0.6.0;
they are no longer roadmap-only items. Consult [project state](docs/PROJECT_STATE.md)
for active work rather than treating historical design aspirations as commitments.
