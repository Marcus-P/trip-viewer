# Consistency review for 0.7.0

Review scope: the Marcus-P main tree at the published 0.6.0 merge, the open
release/documentation correction and the requested Ubuntu GUI installation/update
and relevant-check changes. This checks cross-component consistency and build
behaviour, not a claim that all possible camera/platform/media cases are proven.

## Findings and corrections

| Area | Finding | Correction |
| --- | --- | --- |
| Release documentation | README and related guides still called 0.5.0 latest and 0.6.0 unmerged | Current downloads resolve Latest Release; 0.6.0 is a completed record; source versions and published assets are distinct |
| Issue reporting | The fork's Report a bug opened the upstream tracker | Targets Marcus-P/trip-viewer; upstream attribution links remain |
| Camera guidance | An empty library told every user to use Wolf Box filenames | Camera-neutral folder/model guidance |
| IPC | The UI invoked dump_miltona_gps_debug but Tauri did not register it | Registers the existing diagnostic implementation; the static interface check now resolves 56 calls |
| Storage | Migrations/parser/documentation needed cross-checking | All 17 migrations registered; archive schema 17 and GPS parser 8 remain unchanged |
| Updates | Fork had no installer/update route for non-terminal users; unused signed-updater dependency remained | Ubuntu Debian installer, bounded public release checks, explicit download/install; unused upstream updater dependency/permission removed |
| Desktop | Tauri's Debian entry used a different filename/WM class from the existing GTK identity | Normalize com.tripviewer.app desktop identity and regenerate package checksums; retain original icons |
| CI | Every docs commit ran frontend, Rust, audit and AppImage work | Diff-scoped checks with a lightweight required ubuntu job for docs/version metadata; no docs test artifacts |
| Release policy | Mandatory notes and increments needed enforcement without irrelevant tests | Check synchronized versions, one increment and matching changed release notes; release packaging runs once after protected merge |
| Artifacts | AppImage and installer test copies would duplicate download/storage | One small Debian installer artifact for packaging tests, seven-day retention; stable AppImage remains supported |
| Release notes | Relative source links could be broken on the GitHub Release page | Resolve release-note links against the published source commit |

## Evidence

- 92 frontend tests and typecheck/build pass. The separate Node CI/version suite
  has thirteen tests, including docs/metadata-only diffs, dependency changes,
  unknown/workflow paths, frontend/backend selection and mandatory release notes.
- 339 Rust tests pass; one pre-existing ignored test remains. IPC consistency
  checks resolve 56 literal frontend calls and all 17 migration files.
- Local .deb build and payload validation pass: correct package/version/amd64,
  explicit GStreamer playback dependencies, normalized desktop identity, icon and
  complete application/dependency notice material. The local package is about
  7 MB; release build sizes may differ. CI additionally proves installation and
  dependency resolution on Ubuntu 24.04 and retains AppImage integration checks.
- Native WebKit verifies the update UI with synthetic release responses, offline
  handling, no downgrade, unchanged video nodes and the registered diagnostic IPC.
  A live GitHub check returned the actual stable 0.6.0 and correctly identified
  installed 0.7.0 as newer. Link validation is unit-tested; the headless test does
  not activate the OS browser.
  Existing synthetic multi-camera playback/visibility/fullscreen checks also pass.
- Final Ubuntu CI status and the downloadable test installer are recorded on the
  [pull request](https://github.com/Marcus-P/trip-viewer/pull/5). Do not claim a
  successful CI run solely from these local results.

## Limits and retained decisions

Real-camera functional testing remains exclusively VIOFO A229 Pro 3CH on Ubuntu
24.04 LTS. Older camera naming/decoder implementations remain; synthetic fixtures
are not new real-recording/GPS/firmware certification. The archive format is not
changed by this release. The upstream contribution in
[PR #16](https://github.com/chrisl8/trip-viewer/pull/16) is retained separately.

An Ubuntu App Center double-click installation and the resulting launcher still
need the owner's focused acceptance test. There is no APT subscription or unattended
installer. Updates use published releases built from main, not raw PR/source commits.
The existing bundle-size/mixed-import warnings and unused timelapse helper warning
remain maintenance items; they are not evidence of a new playback failure.

Pure docs work still increments PATCH and updates release notes under the owner's
explicit policy. Its PR does not run runtime suites or create packages. Publishing
a new app version nevertheless requires one merged-main package build. Original
logos/icons, source copyright, MIT notice and third-party licenses are preserved.
