# Trip Viewer 0.8.0

## Windows and experimental macOS packages

- Adds a current-user Windows 10/11 x64 NSIS installer with WebView2 setup,
  Start Menu integration and bundled application/dependency license material.
  Existing user settings/archives and the original application identity remain.
- Adds one macOS 11+ universal disk image containing Intel and Apple Silicon
  code. It is ad-hoc signed, not Apple-notarized, and explicitly experimental
  until real-device playback/GPS validation is available. Manual folder import
  remains available; Mac automatic SD-card detection is not implemented.
- Update checks select the matching installer using the compiled application
  target, rather than sending every platform to an Ubuntu package. Unknown
  architectures and missing packages fall back to the release page. Downloads
  still require user-confirmed installation and never suggest downgrades.
- Retains Ubuntu AppImage/Debian installers, existing icons, authorship, MIT and
  dependency notices. No Windows publisher certificate, Apple Developer ID,
  upstream signing key or unattended updater is assumed.

## Native validation and storage

- Fixes cross-platform checkpoint test setup to use the canonical archive root,
  matching production scan paths on Mac/Windows. The relative-path assertion now
  checks decoded JSON instead of matching raw strings with escaped separators.
  Cache/parser implementation and the tested health/restart assertions remain.
- Native Windows/Mac jobs build and validate their own packages, license payloads,
  architecture and a basic process launch. Mac checks both universal slices,
  ad-hoc signature and mounted disk-image contents. Windows silently installs
  into a temporary directory and checks the installed executable/shortcut.
- Run native Rust tests when backend checks are relevant; share package workflow
  code between PR validation and release publishing. The required ubuntu status
  now waits for every relevant platform, retaining a lightweight docs-only path.
- Retain only one small acceptance installer per platform for seven days. Native
  release transfers expire after one day and are removed after publication; large
  Ubuntu AppImages/bundles stay in the publishing job. No build caches or duplicate
  Windows MSI / separate Mac architecture installers are uploaded.

The app version increases MINOR for the new distribution targets. Dependencies,
archive schema 17 and GPS parser 8 are unchanged. Hands-on camera testing covers
VIOFO A229 Pro 3CH on Ubuntu 24.04 LTS and, following owner feedback, Windows 11
x64. Windows 11 worked without an additional codec/plugin installation on the
tested computer and is the recommended Windows platform. On Windows 10, missing
HEVC support may require purchasing Microsoft’s HEVC Video Extensions at the
Store’s displayed price (€0.99 was reported); playback after purchase is untested.
The linked device-manufacturer extension was unavailable on the tested Windows
10 computer. A successful native build or empty-app launch is not a
real-camera/codec/GPS certification. Mac remains experimental. See [installation](docs/INSTALLATION.md) and
[compatibility](docs/COMPATIBILITY.md) for platform requirements and limits.

---

# Trip Viewer 0.7.2

## Current download guidance

- Removes the historical 0.6.0/PR #4 narrative from the README download section.
  Current download and installation instructions use Latest Release without a
  manually maintained application version; the link follows new publications.
- Makes the installation/user/release guides consistent with that approach.
  Keeps release history in these notes and the project record, and preserves
  the archive compatibility warning, provenance, icons and license notices.
- Records the successful 0.7.1 publication and a standing documentation rule
  against putting old release narratives into current installation guidance.

Application behaviour, dependencies, archive schema 17 and GPS parser 8 are
unchanged. This documentation-only PATCH follows the owner's every-merge version
and release-note policy. PR validation needs only documentation/version checks;
no application test, package build or test artifact is required before merge.

---

# Trip Viewer 0.7.1

## Documentation and repository maintenance

- Records the completed 0.7.0 merge and its verified release status separately
  from source versions and temporary test builds.
- Makes conservative GitHub Actions storage use a standing project rule: only
  necessary small acceptance artifacts, at most seven days of retention, no
  duplicate packages or unnecessary caches, and authorized cleanup after use.
- Preserves published downloads, license material and useful check history.
  Updates the upstream contribution record with the suitable consistency fixes;
  fork-specific installer/update policy remains separate.

Application behaviour, dependencies, archive schema 17 and GPS parser 8 are
unchanged. This documentation-only PATCH follows the owner's version/notes
policy and requires only documentation/version checks, without a PR test build.
The post-merge release packages the new version once. Hands-on camera validation
remains exclusively VIOFO A229 Pro 3CH on Ubuntu 24.04 LTS.

---

# Trip Viewer 0.7.0

## Ubuntu installation and updates

- Adds an Ubuntu 24.04 LTS x86_64 `.deb` installer for download and double-click
  installation/upgrades. The package includes playback dependencies, the original
  icon, the stable desktop/application identity and source/dependency notices.
- The navigation bar and Settings check the fork's published stable GitHub release.
  New releases offer an installer download; installation is confirmed by the user
  through Ubuntu. Failed/offline checks do not stop playback, requests time out,
  concurrent checks share a cache, and newer test builds are not offered downgrades.
- Keeps the standalone AppImage and existing launcher helper. The upstream
  signed-updater integration remains disabled; its unused dependency/permission is
  removed. No private upstream signing key is used and no automatic background
  package installation is introduced.

## Consistency fixes

- Fixes the Report a bug link to target this fork, and makes the empty-library
  message camera-neutral rather than implying only Wolf Box is supported.
- Registers the existing Miltona GPS diagnostic-export backend command used by
  the visible UI. This is an IPC consistency fix, not new Miltona hardware validation.
- Corrects stale pre-release documentation and old 0.5.0 installation instructions.
  Current downloads use Latest Release; the 0.6.0 acceptance/merge/release record
  is completed. Upstream attribution, logos and license notices are retained.
- Release notes are mandatory and must match the proposed version. Every merge,
  including documentation-only work, still increments the app version as requested.

## Relevant checks and artifacts

- PR CI selects checks by changed area. Documentation plus version-only metadata
  uses light version/release-note/link/shell checks, with no frontend/Rust suites,
  native build dependencies, package build or test artifact.
- Frontend, backend, runtime dependency and packaging changes select their own
  checks. CI-policy tests cover documentation, version-only changes, dependency
  changes, unknown files and the mandatory release-note/version gates. The required
  ubuntu status remains active for light PRs instead of being skipped entirely.
- Packaging PRs upload one small Debian test installer, not duplicate AppImage and
  installer artifacts. Retention remains seven days. The release workflow builds
  the actual merged packages once without repeating the protected PR's unit suites.
- Release-note links point to the published source commit. Installer tests use an
  explicit isolated installation root without replacing the process HOME variable.

The database schema remains 17 and GPS parser remains 8. Hands-on camera testing
remains exclusively VIOFO A229 Pro 3CH on Ubuntu 24.04 LTS; automated/synthetic
legacy checks do not certify other original recordings or operating systems.
See the [installation guide](docs/INSTALLATION.md) and
[consistency review](docs/CONSISTENCY_REVIEW.md). A source version is not proof of
publication; stable assets appear on [Latest Release](https://github.com/Marcus-P/trip-viewer/releases/latest)
after the owner merge and successful release workflow.

---

# Trip Viewer 0.6.0

Released after the owner accepted the final integration test and merged PR #4.
Builds on stable 0.5.0 and the original [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer) project.
The upstream logos/icons, MIT notice and dependency license notices are retained.

## New playback tools

- Click the recorded route to seek the video and timeline to the corresponding
  trip time, including between samples and across video segments. GPS gaps are
  not bridged; repeated locations prefer the traversal nearest the playhead.
- The F/I/R + GPS fullscreen dashboard includes the timeline and speed graph,
  with an expand/collapse button and a saved preference. Single-camera/back
  navigation keeps the same video elements and fullscreen root.
- While paused, zoom using the wheel or +/- controls and drag the enlarged
  image to pan. Reset returns to 100%. Resuming playback or changing source
  restores the normal view without replacing/reloading the video element.
- Archived timelapses can seek via trip time even when no original segment
  list remains, using the existing speed-curve mapping.

## Camera visibility

- Checkboxes select which available camera channels are displayed, with saved
  preferences. Hidden tiles occupy no layout space; two cameras fill the full
  height and one camera fills the video area.
- Hiding every camera gives the map the full viewing area. Controls remain
  available to show cameras again. Visibility changes retain the video elements
  and current playback position.

## Timeline bookmarks

- Right-click the graph/timeline to save a numbered bookmark. Click its marker
  to seek; right-click the marker to remove it. Previous/next arrow-and-bar
  buttons jump between bookmarks without wrapping. `+ Bookmark` saves the
  current position.
- Bookmarks persist per trip in the archive database across trip changes and
  restarts. They do not depend on GPS analysis and use the existing seek mapping
  in both Original and timelapse playback. Manual trip merges rebase positions.
- Archive database schema advances to 17; older builds supporting only schema
  16 cannot open an archive upgraded by this build.

## Upstream integration and CI

- Integrates chrisl8 commits `2534f65` and `a9617fc`, preserving their authorship:
  Ctrl/Cmd/Shift trip selection, bulk originals/trip deletion with confirmation,
  hard guards against losing archive-only timelapses during merges, and scoped
  cleanup of newly redundant timelapse files.
- Retains the fork's tested synchronization, seek timeouts, audio handling and
  bookmark rebasing while incorporating the upstream behaviour.
- One CI run per PR update, cancellation of superseded runs and seven-day test
  artifacts replace duplicate push/PR builds. The merged-main release workflow
  still verifies and publishes the stable package.

## GPS analysis

- Independent candidate files share a read pool capped at two workers across
  trip and batch jobs. Candidate order/ties, raw GPS source order, best-camera
  selection and adjacent VIOFO validation remain deterministic.
- VIOFO descriptor pairs are read through a bounded sequential buffer instead
  of seeking for each pair. Segment and trip caches remain valid (parser v8).
- Completed checkpoints bypass reader jobs. Source/health revalidation and
  short DB lock sections are preserved.
- Optional command-line diagnostics: `TRIPVIEWER_GPS_TIMING=1` reports metadata,
  GPS read/decode, validation and candidate-selection times. Worker comparison
  can use `TRIPVIEWER_GPS_WORKERS=1` or `2` (values are capped to this range).

A real USB-SSD speedup has not yet been measured. The reported baseline is
about 20 seconds per 3-minute, three-camera segment. Test uncached comparable
footage; existing cached trips intentionally do not repeat extraction.

## Validation and release status

The maintainer’s functional checks used **only VIOFO A229 Pro 3CH on Ubuntu
24.04 LTS**. Route seeking, fullscreen timeline, paused zoom, bookmarks including
restart, and camera checkboxes have been accepted. Other implemented cameras
remain available; [automated/synthetic compatibility checks](docs/COMPATIBILITY.md)
do not certify their real recordings or imply they stopped working.

The final PR CI and the merged-main Ubuntu release workflow passed. Local
verification has 85 frontend and 339 Rust tests passing (one existing ignored).
The owner accepted the final integration test before merging. Main protection
remains active. The release is built from merge commit
`865bfcd841cfd7e1be6a54799943ac06f7b1f80a`.

## License

Trip Viewer remains MIT licensed. Third-party components retain their licenses;
see `THIRD_PARTY_NOTICES.md` and the release dependency-license inventories.
