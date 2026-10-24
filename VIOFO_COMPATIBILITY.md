# VIOFO and playback contribution: validation scope

These additions were developed/tested in the Marcus-P fork of chrisl8/trip-viewer.
Original project attribution, icons and licenses are retained. Björn Fries
(`gubiq`) authored the archive-auto-scan and Linux libva/DMABuf fixes; their
commits and co-author notices are preserved in this contribution branch.

The fork maintainer's hands-on camera testing covers **VIOFO A229 Pro 3CH on
Ubuntu 24.04 LTS and Windows 11 x64**. The Ubuntu acceptance covers playback/GPS
preparation, segment transitions, map seeking, dashboard timeline, paused zoom,
bookmarks including restart and channel visibility. On 2026-10-23 the owner also
reported successful Windows 11 operation without installing another codec or
plugin; this does not infer an exhaustive per-feature/hardware matrix.

Windows HEVC availability depends on the existing decoder and drivers. On the
tested Windows 10 computer, the OEM HEVC Store package linked by the app was
unavailable. The general Microsoft HEVC Video Extensions package was offered
for €0.99; the Store's displayed price/availability may vary. **Windows 10 playback
after buying it is untested.** Installing VLC alone does not give WebView2 VLC's
decoder. No new Mac real-footage test is claimed; upstream's Mac packaging and
signing claims remain its own. No other real camera is newly certified.

Existing Wolf Box, Thinkware, Miltona and generic implementations remain. Their
naming/grouping and Wolf Box/Miltona decoder files are unchanged from upstream.
Automated tests pass, and generated H.264 clips in the legacy naming formats
were scanned/decoded/played in native WebKit with 3/2/1/4 channels respectively.
Those synthetic clips contain no proprietary camera GPS tracks; they cannot
confirm real device/firmware/codec behaviour. Lack of confirmation does not mean
those cameras stopped working. Thinkware still has no GPS decoder as before.
A129 models were not tested by the fork maintainer; coordinate with PR #12's work.

Source changes include VIOFO normal/parking recognition and GPS validation,
resumable progressive extraction, bounded reads, persistent camera health,
playback recovery, shared trip-time route/bookmark seeking, paused zoom and saved
camera visibility. Database schema advances to 17; upgraded archives cannot be
opened by schema-13-only upstream builds. GPS parser version is 8 and the
existing speed-curve envelope remains v1.

This branch retains upstream package/application versions, updater, release
workflows, signing configuration and existing documentation rather than applying
the fork's separate distribution and release policy. The upstream maintainer controls the
release/version decision. No original user media is rewritten by these features.

Follow-up consistency fixes from the fork's 0.7.0 work make the empty-library
message camera-neutral and register the existing Miltona diagnostic-export
command used by the UI. They do not change the GPS parser, schema, upstream
release/updater configuration or the real-camera validation scope above.

Checkpoint test fixtures now use the canonical archive root for both recording
paths and media-health keys, matching production scans on Windows/macOS. The
relative checkpoint path is checked after JSON decoding. All four checkpoint
tests pass with an aliased temporary root, which reproduced the three original
fixture failures before the correction. Production parser/cache behaviour and
upstream distribution configuration are unchanged by this test-only follow-up.


## Application changes aligned with fork 0.8.1

This contribution uses upstream main `a9617fcb0ec53bd23a627fa7f5c30cbd3a65d148`
and incorporates the shared application changes from fork release
[0.8.1](https://github.com/Marcus-P/trip-viewer/releases/tag/v0.8.1), source
`97e1858262b6c255ad637c1116455358f4312d63`. The contribution keeps upstream's
own application/package version; the fork's release number is a source reference.

The GPS parser, player synchronization, archive schema and checkpoint fixes match
that fork release. The latest follow-up groups camera visibility and Show map in
**Settings → Display layout**, replacing the row above the player. Front,
Interior, Rear and additional archive channel labels are available. Save applies
the draft to the mounted player; Cancel discards it. The existing layout key and
saved selections are retained. Small-window settings/navigation can scroll.

Upstream's updater, update endpoint, signing configuration, package dependencies,
release workflows and original icons/license material are retained. The fork's
installer/release/update implementation is separate from these shared changes.

## Review map

| Area | Main entry points |
| --- | --- |
| VIOFO recognition and GPS | `scan/viofo.rs`, `gps/viofo.rs`, `gps/viofo_timing.rs`, `gps/validation.rs`, `gps/mod.rs` |
| Progressive GPS persistence and camera health | DB migrations 0014–0016, `db/gps_segment_checkpoints.rs`, `media_health.rs`, `state/gpsPreparation.ts` |
| Shared playback/recovery | `engine/SyncEngine.ts`, `components/video/PlayerShell.tsx`, `VideoGrid.tsx`, `mediaRecovery.ts` |
| Map seeking and bookmarks | `components/map/routeSeek.ts`, `components/timeline/useBookmarks.ts`, `bookmarks.rs`, DB migration 0017 |
| Latest settings follow-up | `components/settings/SettingsDialog.tsx`, `settings/layout.ts`, its test, `PlayerShell.tsx`, `MainNavTabs.tsx` |

Frontend paths above are relative to `src/`; Rust paths are relative to
`src-tauri/src/`. Original contributor commits remain intact, with the settings
and evidence update added as a separate follow-up for review.

The original thirteen migration scripts are unchanged from upstream. A separate
in-memory SQL smoke upgraded an upstream schema-13 fixture through migrations
14–17, preserving its trip and GPS rows and passing integrity/foreign-key checks.
This verifies the SQL upgrade path, not every real archive or the old binary's
ability to reopen an upgraded archive. Back up archives before testing.

Display-settings follow-up validation: TypeScript typecheck and Vite production build pass;
ten focused tests cover saved-layout notifications/persistence, camera placement
and map visibility. Static comparisons confirm that shared GPS/sync/database
code matches fork 0.8.1 and upstream's versions, dependency locks, updater,
signing/release scripts, workflows, icons and license files are retained.
The follow-up changes no Rust runtime code or migration scripts.


## Archive-switch bugfix (fork 0.8.2 candidate)

The prepared contribution also includes the shared bugfix from
[fork PR #10](https://github.com/Marcus-P/trip-viewer/pull/10), a follow-up to
published 0.8.1. It is not a claim that 0.8.2 is already published or accepted.

- Selecting another archive, or reselecting the same folder, automatically
  reloads the recording list. Old playback/library/cache state is cleared.
- Native archive opens are serialized. Delayed startup, scan, GPS and metadata
  replies cannot overwrite a newer selection. Import completion requests a
  fresh scan after any older in-flight scan, so copied recordings are included.
- The last reachable archive is restored on startup. Archive and import-source
  picker locations are remembered separately; cancelling preserves the choice.
- The backend saves settings before replacing its active archive slot. Failed
  saves retain prior in-memory settings. Queued scans of unrelated previous
  roots are rejected before they can persist or garbage-collect database rows.

Review entry points: `src/state/archiveNavigation.ts` and its tests,
`src/components/loader/TripLoader.tsx`, `src/ipc/dialog.ts`, `src/state/store.ts`,
`src/state/gpsPreparation.ts`, `src-tauri/src/archive.rs`,
`src-tauri/src/app_settings.rs` and `src-tauri/src/scan/mod.rs`.

This updated contribution passes **100 frontend tests** and the TypeScript/Vite
production build. Native regression cases cover remembered-folder persistence,
failed settings writes and scan-root isolation; native/package results are
recorded in fork PR #10. The focused owner A → B → restart acceptance remains
pending; the prepared upstream publication is paused until it is confirmed.
The fork version files and release metadata are separate from this bugfix commit
and are not part of this contribution. Upstream's updater/signing/versioning
remain intact. No new DB migration, parser version or original-media rewrite.
