# Trip Viewer 0.13.1-rc.1

## Central settings (local implementation, awaiting acceptance)

- Group preferences in one settings dialog by application area. Apply ordinary
  preferences immediately and remove the old Save/Cancel workflow.
- Reuse FFmpeg configuration in the central Timelapse section. A completed path
  is validated before persistence; Browse/Test/Clear retain their existing
  actions. Remove the separate Timelapse gear and route setup prompts centrally.
- Keep existing preference values, camera support and per-task scan/review
  controls. No database migration belongs to this independently portable change.

## Archive compatibility (local implementation, awaiting acceptance)

- Keep the shared archive database at upstream schema 13. Preserve known fork
  schema-14..17 additions in a versioned archive-local extension database before
  removing them from the shared schema, with a retained SQLite recovery backup.
- Keep places, manual trip merges, archived GPS and completed timelapses intact.
  Existing schema-13 archives do not require media reanalysis for this change.
- Refuse unknown layouts and newer extension schemas. Validate GPS provenance
  against the archived payload so an older application's writes invalidate it.
- Older released fork applications may still apply their existing schema-17
  migration; compatible opens support conversion again without losing retained
  extension data. Applications older than schema 13 are outside this guarantee.

## Feature-branch media evidence and parking GPS

- Integrate current upstream 70mai support while preserving its authorship.
- Use the explicit VIOFO P marker for conservative file-level parking locations.
  Keep clock/spatial validation without treating compressed playback duration
  as the maximum GPS wall-time span. No capture-factor or frame-time mapping
  is invented, and only affected parking analysis keys are refreshed.
- Add separately portable, metadata-only timing/sample-table diagnostics and
  protect zero video timescale before duration arithmetic. Other findings are
  diagnostic and do not introduce a broad file-rejection policy.
- Provide an explicitly isolated Windows Evidence Test build. Production
  identifier/updater configuration and the full fork's archive schema remain
  unchanged. PR #21 receives only its focused parking correction, retaining
  main archive schema 13. Five-second filename/channel alignment remains open.

## Upstream-first development and reliability priorities

- Check current upstream ancestry before code work and in a bounded, read-only
  PR workflow. Application/package changes fail the check when behind; policy
  and documentation maintenance report drift without forcing application builds.
- Keep VIOFO PR #21 first, then offer an updated feature menu for the upstream
  maintainer to choose or combine. Prioritize synchronized playback, recovery,
  archive isolation and GPS consistency before optional interface features.
- Preserve original contributor authorship and supported cameras. No upstream
  integration, application behavior change, release or PR closure is performed
  by this policy update.

## Contributor communication rules and scoped Windows 10 evidence

- Consolidate binding external collaboration and communication requirements in
  AGENTS.md: refresh discussion/commit context, verify references, respect
  maintainer authority and voluntary cooperation, preserve contribution credit,
  provide complete actionable Markdown drafts and review them before handoff.
- Identify the owner-defined rules as originating from Marcus-P/trip-viewer and
  require that attribution and change provenance remain with adapted versions.
  Separate reusable project safeguards from owner-specific work preferences;
  retain the existing MIT license and contributor notices. Extend the same
  provenance rule to owner-derived prompts, custom instructions, Skills,
  templates and related reusable instructions without exposing private sources.
  Exclude knowledge documents and retrieval/RAG sources from owner attribution;
  preserve their actual authorship/rights and require explicit authorization
  before disclosing private content or identifying source metadata.
- Clarify that relevant media filenames are permitted, while technical results
  must be checked for direct or combined real-world identification risk. Apply
  data minimization and privacy-aware new-commit metadata without rewriting
  existing contributor history or changing license notices.
- Add risk-based verification goals and stopping conditions. Reuse valid prior
  evidence, avoid unsupported full-test repetition, and minimize both compute
  and owner coordination effort without bypassing required checks.
- Keep existing privacy, media safety, Codex attribution, authorization and
  branch/release protections. Clarify permitted public GitHub handles versus
  private account identifiers; no parallel communication policy is created.
- Record owner-confirmed Windows 10 Front-only 8x timelapse encoding and output
  playback after installation of the required HEVC codec, for all segments of
  two recognized trips in the full fork. Isolated upstream PRs and diagnostic
  packages are not covered by that hardware result.
- Add opt-in owner commit/push privacy guards with repository-local no-reply
  configuration and private marker settings; preserve unrelated existing hooks
  and other contributors' author identities. Keep the checks limited to tooling.
- Documentation, development safeguards and synchronized version metadata. Application behavior,
  dependencies, schema and packaging remain unchanged. No new installer is
  published until an owner-approved stable promotion is merged to main.

# Trip Viewer 0.13.0

## Automatic local timelapse performance calibration

- Compare short equivalent real-input samples before production encoding. Select
  GPU processing, filter threads and batch concurrency only after a repeated
  measured improvement of at least 15%; retain verified safe settings otherwise.
- Discover compiled FFmpeg APIs/filters instead of adding GPU-model rules.
  Windows can test device-resident D3D11/AMF/QSV scaling when available;
  CUDA and Linux VA-API keep their existing bounded hardware pipelines.
- Limit calibration to 45 seconds per batch, allow cancellation, and cache up to
  eight local results for seven days. Runtime/driver, source format, FFmpeg and
  workload-size changes invalidate reuse. No diagnostic upload or telemetry.
- Check live RAM before parallel work, preserve finite GPU buffers, clear tuning
  after hardware fallback and reduce subsequent work to one process. Existing
  playable files and original recordings retain the established protections.
- Show calibration and selected settings in the English progress panel.

The 0.13.0 RC passed Windows, Ubuntu and universal macOS packaging CI.
The owner confirmed Windows Radeon 780M Front/8x with D3D11VA decoding and AMF
GPU scaling/encoding, and Ubuntu Intel UHD Interior/16x with VA-API and about
five to six minutes runtime. Both outputs play correctly. These are focused
workload acceptances, not a controlled cross-version benchmark or certification
of every hardware/codec combination. macOS remains physically untested and
experimental. Original media, logos, provenance and license notices are preserved.
Stable publication follows the owner-controlled main merge and release workflow.

# Trip Viewer 0.12.0

## Hardware decoding and reliable timelapse processing

- Selects verified decoding APIs alongside the existing encoders: Linux VA-API,
  Windows D3D11VA, NVIDIA CUDA and experimental macOS VideoToolbox, without a
  GPU-model allowlist. Linux VA-API can keep frames on the device through
  decoding/scaling/encoding. Windows AMF/QSV retain software filters where needed.
- Fixes a weak Full-HD probe that could bypass VA-API scaling. The finite NV12
  source now exercises actual resizing and eight looped seconds accelerated 8x.
  Before each hardware-decoded channel, checks a bounded sample of the actual
  source with production scaling and a twenty-second deadline.
- Gives VA-API scaling finite surface reserves, limits asynchronous encoding and
  hardware processing depth, and invalidates older capability caches without
  losing the FFmpeg path. Processing failures retain the GPU encoder with
  software processing before the established CPU-encoding fallback. Cancellation
  remains cancellation, and existing accepted output is preserved until success.
- Shows the verified/requested decode profile and logs each attempt. Bounded
  diagnostics retain decoder-selection evidence. These are not live GPU meters.
- Gives acceptance installers unique numeric RC versions below the final stable
  version. Debian uses equivalent `~rc.N` ordering; the app recognizes RC →
  stable as an upgrade. Main rejects unfinalized candidate metadata.

Linux acceptance: the owner completed a real Intel UHD/CML GT2 Front/16x VA-API
job in 850.6 seconds without memory error or CPU-processing fallback and confirmed
problem-free output playback on Ubuntu 24.04 LTS. This is not a controlled
speed comparison. The owner also confirmed the Windows 11 Radeon 780M
D3D11VA-decoding plus AMF-encoding Front/16x test completed and its output plays
correctly. Other GPUs and Mac hardware remain separately
unverified. Original media, playback/GPS behavior, archive schema, logos and
license notices remain unchanged.

Release preparation is not publication. Current downloads remain at Latest
Release until the owner merges the promotion and package publication succeeds.

---

# Trip Viewer 0.11.0

## AMD, Intel and Linux GPU timelapse encoding

- Adds HEVC encoding through AMD AMF, Windows Intel Quick Sync and Linux VA-API.
  Selection preserves NVIDIA NVENC priority, then tries AMF, Windows QSV and
  accessible Linux render devices before CPU/libx265. Each GPU choice must pass a short
  real encode/decode/scale test; a compiled-in encoder alone is insufficient.
- AMF/QSV initially use CPU decoding/scaling and the GPU video encoder. QSV
  explicitly derives a hardware session from an Intel D3D11 device. VA-API
  uploads software-decoded NV12 frames for GPU scaling/encoding and retains the
  device that passed verification. The NPU is not used.
- Limits automatic AMF/QSV/VA-API concurrency to one job. A GPU failure retries once
  on CPU and keeps later batch jobs on CPU; cancellation skips the retry.
- Uses one finite Full-HD GPU probe across vendors instead of relying on two
  tiny output frames. AMF production uses NV12. Verifies a CPU-processing/GPU-
  encoding profile when CUDA decode or VA-API scaling is unavailable, before
  abandoning hardware encoding. No per-model allowlist is required.
- Shows the verified and currently running encoder by name. Invalidates older
  capability results while preserving the configured FFmpeg path.

Requires a compatible FFmpeg build, working drivers/runtime and device access.
Cloud checks cover selection, command construction, fallback and real synthetic
CPU encoding. The owner confirmed real Intel UHD VA-API encoding and playable 8x
output on Ubuntu 24.04 after installing the complete Intel media driver; Windows
AMD AMF app probing and a progressing real encoding job are confirmed. Windows
output completion/playback and Intel QSV hardware acceptance remain open. No
speedup factor is promised. Playback, original media, GPS and archive schema are
unchanged.
See the [focused GPU test](docs/testing/AMD_TIMELAPSE_0110.md).

---

# Trip Viewer 0.10.3

## Counter-free VIOFO filenames

- Recognizes timestamp/channel names without a sequence counter, such as
  `YYYY_MMDD_HHMMSS_F.MP4`, alongside numbered VIOFO recordings. Detection is
  per file; mixed archives require no firmware selector or video renaming.
- Treats the counter and parking marker independently, keeps F/I/R grouping and
  normal/parking separation, and accepts case variants of the MP4 extension
  and channel/parking letters. Rejects malformed timestamps and unknown suffixes
  without panicking on unexpected Unicode filenames.
- Applies the same recognition to automatic/manual scans, GPS decoder dispatch
  and filename time anchors, and timelapse source/sibling lookup.
- Tests mixed filename forms with real synthetic HEVC input across all nine
  channel/tier outputs, including progress and multi-window encoding.

This is format compatibility, not a claim of hands-on validation of every past
firmware. A recording timestamp does not identify its firmware version. Existing
camera parsers, encoder quality, progress/ETA, archive schema and GPS byte decoder
remain unchanged. Original media is not renamed or modified.

---

# Trip Viewer 0.10.2

## VIOFO timelapse and runtime encoder validation

- Uses the shared VIOFO filename recognizer for timelapse channel lookup as well
  as scanning. Front/Interior/Rear files with different sequence numbers resolve
  correctly; normal and parking recordings remain separate.
- Verifies GPU HEVC encoding, CUDA decoding and GPU scaling with tiny synthetic
  input instead of treating an encoder-list entry as hardware availability.
  CPU fallback is also tested. Each test stage has a ten-second limit.
- Revalidates before each encoding run, invalidates the old list-only capability
  cache once, and retries a failed GPU job once on CPU. Further jobs in that
  batch use CPU, with bounded per-job CPU pools. Cancellation does not retry;
  completed job metadata records the encoder that actually succeeded.
- Shows expandable per-job encoding errors directly in Timelapse.
- Reads FFmpeg's machine-readable progress during source preparation, encoding
  and finalization. Shows per-phase video progress, CPU/GPU encoder, elapsed time
  and a current-step ETA before any whole job completes. Multi-window progress
  keeps its output offset; stale progress suppresses the step ETA rather than
  showing a misleading countdown. Batch ETA is approximate after one completed
  attempt; encoding quality and the existing CPU preset remain unchanged.
- Establishes long-lived `develop` integration before owner-controlled `main`
  releases. Only `main` publishes installers; integration uses the same proposed
  0.10.2 version until stable promotion, without an extra release increment.

Original footage, archive schema and GPS parser versions remain unchanged.
Synthetic tests do not constitute new hardware/camera certification.

---

# Trip Viewer 0.10.1

## Reliable Windows installer tool download

- Fetches the same pinned NSIS 3.11 and Tauri plugin through the GitHub Release
  API before compilation, with bounded download retries and SHA-256 verification.
  The 0.10.0 publication attempts failed on the original NSIS download with HTTP
  504 after successfully compiling the application.
- Fails early on unavailable or invalid installer tools, and keeps the actual
  native failure tail within GitHub's first ten check annotations.
- Runs installer validation for packaging-tooling edits without repeating
  unchanged frontend/Rust unit suites or dependency audits.
- Adds no persistent Actions cache or extra tool artifact. Existing installation,
  playback, public download statistics and update behaviour are retained.

---

# Trip Viewer 0.10.0

## Public download statistics in Settings

- Adds on-demand public GitHub download totals by platform and stable release,
  with expandable per-package filenames, formats and counts, including separate
  Ubuntu DEB, AppImage and bundle figures.
- Shows retrieval time and download-count limitations. Uses bounded pagination,
  shared caching and explicit refresh; errors keep previously displayed figures.
- Invites users to leave feedback in the GitHub project through their browser.
  The app does not submit comments automatically.

No installation telemetry or server is added. Package downloads do not establish
unique users, installed computers, actual user OS versions or successful updates.
Existing playback, archive data and updater behaviour are unchanged.

---

# Trip Viewer 0.9.1

## Clarify development-tool attribution

- Names Codex, OpenAI's coding agent, as the assistance used for the VIOFO-specific
  and related fork development, replacing the README's imprecise ChatGPT label.
- Separately credits the maintainer's requirements, sample footage, hands-on
  testing and final acceptance, with consistent wording in the provenance notice.
- Retains upstream/contributor credit, logos/icons, copyright and license terms.

Documentation and version metadata only; application behaviour, dependencies,
archive format, GPS parsing and the accepted 0.9.0 functionality are unchanged.

---

# Trip Viewer 0.9.0

## Optional overlay camera layout

- Keeps the existing Classic layout and its saved sizes, adding a switchable
  Overlay mode with Front as the background. Rear starts in a rear-view mirror
  mask and Interior in a separately positioned overlay; other channel labels
  and camera visibility settings remain supported.
- Adds an overlay editor with independent frame movement/resizing and persistent,
  aspect-preserving video cropping. The mirror's outside corners are transparent,
  revealing Front. Editing Rear does not transform Front or move the mirror frame.
  Explicit horizontal/vertical crop-position sliders select the visible area;
  Bottom / road brings the lower part of Rear into view. Zoom/position controls
  automatically enable video-crop dragging instead of moving the frame.
  Reset crop restores 100% zoom and centers both horizontal and vertical crop
  positions, while leaving the frame and other camera layers unchanged.
  Explicit masks also constrain the composited video itself when a webview does
  not honor rounded ancestor overflow. The mirror frame is drawn above the video;
  its outer corners remain transparent and reveal Front.
- Saves relative positions, crop/zoom, shape and stacking across restarts and
  segment changes. Save commits a draft; Cancel discards it. Reset is previewed
  until saved. Keyboard handles support fine adjustments.
- Reuses the mounted video elements, sources and SyncEngine when switching views
  or editing. Dashboard fullscreen retains the composition; single-camera
  fullscreen shows the full recording and restores the saved crop when exiting.

The owner accepted overlay playback, crop-position controls and mirror masking
with VIOFO A229 Pro 3CH footage on Ubuntu 24.04 LTS. Browser checks also cover
complete crop reset, frame independence and Save/Cancel. Original recordings,
archive schema 17, GPS parser 8, native playback workarounds, updater behaviour
and platform release workflows are unchanged. Existing camera/OS validation
limits still apply; these results do not certify additional cameras or platforms.

---

# Trip Viewer 0.8.2

## Reload selected archives and remember folders

- Automatically reloads the newly selected archive, including reselecting the
  same folder. The loading indicator stays active during the scan; old library,
  video-selection, GPS and archive-specific caches are cleared on a switch.
- Serializes archive opens so the latest choice is also the last saved archive.
  Startup restoration cannot overwrite a newer user selection. Late scan, GPS
  and archive-metadata responses from the previous archive are ignored.
- Reopens the last reachable archive on startup and starts the archive picker
  there. Archive and import-source picker locations are remembered separately;
  cancelling a picker preserves its previous location.
- Rescans imported files after an earlier scan finishes, rather than displaying
  that earlier scan's incomplete file list. Issues rescans use the same guarded
  reload path and report errors instead of leaving a permanent loading state.
- Persists the selected archive before replacing the active backend slot. A
  failed settings write leaves the previous settings intact and reports an error.
  A queued scan for an unrelated old root cannot persist or garbage-collect rows
  in the newly active archive's database.

Original recordings, archive schema 17, GPS parser 8, updater behaviour and
platform delivery are unchanged. This PATCH requires a focused owner check of
folder A → B, automatic list refresh and restart into B before stable merge.

---

# Trip Viewer 0.8.1

## Durable downloads for every main push

- Runs the desktop release workflow on every push to `main`, including commits
  that do not change `RELEASE_VERSION`. Each run builds the pushed head for
  Windows x64, universal experimental macOS and Ubuntu 24.04 x86_64. Ubuntu
  continues to include both the Debian installer and AppImage/bundle.
- Publishes a stable `vX.Y.Z` release for a version-changing merge. An extra main
  commit with the same app version gets an immutable `main-<full commit SHA>`
  prerelease containing all three platforms. Such builds never replace a stable
  version or appear as stable automatic-update suggestions.
- Plans and validates release identity before installing native build dependencies
  or building packages. Existing tags are rejected before those expensive jobs;
  published installers and their checksums are never silently replaced.
- Keeps permanent downloads in GitHub Releases rather than expiring Actions
  artifacts. Only small Windows/Mac handoff artifacts use temporary storage,
  removed after publication or expired after one day. PR test artifacts remain
  short-lived and obsolete ones are removed after the release is verified.

Playback, GPS, archive schema 17, parser 8 and dependencies are unchanged.
Windows 11 owner acceptance remains valid; Windows 10 with a purchased HEVC
extension is untested and Mac real-footage playback remains experimental.

---

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
- Groups the existing camera visibility checkboxes with Show map under
  **Settings → Display layout**, removing the camera-checkbox row above the
  player. Save applies the selection; Cancel discards edits. Existing saved
  camera choices remain in place, and hidden views still free up display space.

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
