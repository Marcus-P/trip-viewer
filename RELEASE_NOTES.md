# Trip Viewer 0.6.0 candidate

Open PR #4, not yet a published stable release. Builds on stable 0.5.0 and the
original [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer) project.
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

The latest functional head passed Ubuntu CI; local verification has 85 frontend
and 339 Rust tests passing (one existing ignored). Documentation changes receive
fresh CI. Main protection is active; the repository owner merges only after the
required checks. This candidate becomes a stable download only after merge and a
successful release workflow. See [project state](docs/PROJECT_STATE.md).

## License

Trip Viewer remains MIT licensed. Third-party components retain their licenses;
see `THIRD_PARTY_NOTICES.md` and the release dependency-license inventories.
