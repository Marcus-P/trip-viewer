# Trip Viewer project state

Current hand-off for the Marcus-P fork of [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer).
Preserve upstream authorship, logos/icons, MIT/dependency notices and original
media. The [documentation index](README.md) is the entry point; the
[prior development record](history/PROJECT_STATE-before-doc-refresh.md) retains
the debugging and earlier acceptance history.

## Releases and documentation status

[Latest Release](https://github.com/Marcus-P/trip-viewer/releases/latest) is the
authority for the currently published stable download. A checkout's version in
`RELEASE_VERSION`, an open PR or a CI artifact is not proof of publication.
Keep current-download links and installation commands independent of a manually
maintained "latest version" number; resolve one published tag before downloading.

Release record: the owner accepted the final 0.6.0 integration test and merged
[PR #4](https://github.com/Marcus-P/trip-viewer/pull/4) as
`865bfcd841cfd7e1be6a54799943ac06f7b1f80a`. The final PR CI and
[merged-tree release run 39664733283](https://github.com/Marcus-P/trip-viewer/actions/runs/39664733283)
passed. [Stable 0.6.0](https://github.com/Marcus-P/trip-viewer/releases/tag/v0.6.0)
was published with ten assets; its tag points to that merge commit. The downloaded
checksum manifest matched GitHub asset digests for both AppImage and bundle.
The [acceptance record](FINAL_060_CHECK.md) is complete, not a pending test or merge.

The owner merged [PR #5](https://github.com/Marcus-P/trip-viewer/pull/5) as
`9b9ce06edc1ce51eab97e4b8cf19f8659d84d25d` after the successful
[final PR CI](https://github.com/Marcus-P/trip-viewer/actions/runs/39676647941).
The [merged-tree release run](https://github.com/Marcus-P/trip-viewer/actions/runs/39680040761)
also passed. [Stable 0.7.0](https://github.com/Marcus-P/trip-viewer/releases/tag/v0.7.0)
was published with eleven assets, including the Ubuntu Debian installer; its tag
points to the merge commit. It combines the documentation correction with GUI
installation, published-release awareness, relevant CI selection and the
consistency fixes in [the review](CONSISTENCY_REVIEW.md). No 0.6.1 release was
published.

The owner then merged [PR #6](https://github.com/Marcus-P/trip-viewer/pull/6) as
`e14cb67f36b210defaf884b88aca840a8ea9bf3c`. Its lightweight documentation/version
CI passed, and the [release workflow](https://github.com/Marcus-P/trip-viewer/actions/runs/39681561876)
successfully published [0.7.1](https://github.com/Marcus-P/trip-viewer/releases/tag/v0.7.1).
It records the storage policy and handoff without changing application behaviour.

Current download/install guidance must stay version-neutral and link to Latest
Release. Old release/merge narratives belong in release notes and project records;
they confused the owner when placed under the README's Download and install heading.
The documentation follow-up removes those distractions from installation guidance
while preserving authorship, license material and useful compatibility warnings.

The owner explicitly retained an increment and mandatory release notes for every
merge, including pure documentation. PR checks must still be relevant: docs plus
version-only metadata use lightweight checks and no test package. Their merged
release builds packages once to embed the new version. Do not repeat accepted
playback tests for a docs edit. Preserve this preference in subsequent work.

## Windows/Mac distribution candidate

Published 0.7.2 followed the owner's merge of
[PR #7](https://github.com/Marcus-P/trip-viewer/pull/7). Current work branches from
that main tree and prepares **0.8.0** for Windows 10/11 x64 and experimental macOS
11+ Intel/Apple Silicon, retaining the Ubuntu packages. This is an authorized
platform-distribution change, not a new claim of camera compatibility.

- On 2026-10-23 the owner confirmed Trip Viewer works on Windows 11 x64 without
  installing another codec/plugin. Treat that result as accepted; do not request
  a repeat for documentation changes or infer an exhaustive per-feature matrix.
  Windows 11 is the recommended Windows platform. On Windows 10 the linked OEM
  HEVC Store extension was unavailable; the general paid extension was offered
  for €0.99. Playback after purchase is untested and must remain labeled as such.
  Use the Store’s displayed price rather than promising a fixed amount. The
  owner chose documentation of this limit instead of a VLC/player rewrite.
- No Mac tester is available. The universal DMG is explicitly experimental and
  ad-hoc signed without Apple notarization. Native compilation, architecture,
  package notices/signature/contents and basic launch checks are automated;
  inspect actual PR job results before claiming success. No new Mac video/GPS
  functional validation is established. SD-card auto-detection remains absent on
  Mac; use manual folder selection. Original-recording/media behaviour is kept.
- Windows NSIS installs for the current account, bootstraps missing WebView2 and
  blocks installer downgrades. HEVC support remains a host codec/driver requirement,
  separate from WebView2. No publisher-signing key is configured.
- The native get_install_target command selects exact release assets for the
  compiled target. Unsupported architectures retain a release-page fallback.
  Raw API response caching is shared without mixing platform-specific selections.
- The reused Windows/Mac packaging workflow runs native Rust tests only when
  backend checks are selected. PR artifacts contain one installer per platform
  for seven days. Publishing rebuilds the merged native packages once, transfers
  only the small installers for at most one day and deletes transfers on success;
  large Ubuntu assets stay in the publishing job. No Actions cache is added.
- The existing required ubuntu status becomes the final aggregate gate over all
  relevant platforms. Docs/version metadata still skip native jobs and artifacts.
  Do not weaken or bypass the active owner/history rulesets.
- These distribution/update changes are fork-specific; do not copy them into the
  separate upstream contribution that retains Chris' own updater/release setup.

Local candidate checks pass: thirteen release/client tests, TypeScript typecheck,
thirteen policy tests, 58 frontend IPC calls/all 17 migrations, configuration
schemas and actionlint. Native Linux 0.8.0 confirms get_install_target returns
linux-x64, the update UI renders/selects its installer, offline errors do not
replace existing video nodes and no downgrade is offered. The headless check
does not activate an OS browser. The final native CI run
[39693022945](https://github.com/Marcus-P/trip-viewer/actions/runs/39693022945)
passed Linux, Windows and universal Mac package checks plus the required ubuntu
aggregate. The owner subsequently confirmed Windows 11 operation; Mac footage
and Windows 10 with the paid HEVC extension remain untested.

Initial native tests exposed noncanonical temporary roots in checkpoint fixtures
(Mac aliases / Windows short-name paths), not a parser change. The same three
failures were reproduced locally using a symlinked TMPDIR. Fixtures now derive
requests and media-health keys from db.archive_root(), matching production's
canonical archive scan contract. Decoded checkpoint JSON must contain front.mp4
rather than an absolute path. All four checkpoint tests pass under that alias;
none were skipped and production GPS/cache code is unchanged. Native failures
are emitted as compact check annotations as well as console logs so full error
context remains reviewable. Package build/validation errors use the same reporting.

## Integrated upstream work and cleanup

Upstream commits 2534f65 and a9617fc are ancestors of the 0.6.0 merge, preserving
their authorship. They add bulk trip selection/deletion, archive-only merge safety
and scoped orphan timelapse cleanup. The fork's stronger seek/audio/cancellation
safeguards and single reconciler remain; bookmark rebasing stays transactional.
The fork was zero commits behind upstream a9617fc after integration. Check live
upstream history before treating that observation as a permanent ahead/behind count.

The obsolete feat/route-seek-fullscreen-tools, feature/viofo-a229-pro,
fix/webkit-sync-stability and perf/gps-extraction branches were verified as
ancestors of main and deleted. All temporary CI artifacts were removed after
0.6.0 release verification. Keep published release/license history. Keep active
PR branches until they are merged or closed, including the upstream contribution.
CI runs once per PR update and cancels superseded work. It selects frontend,
backend, packaging, dependency and documentation checks from the diff. Packaging
changes upload one small installer per relevant platform for seven days; docs/metadata-only work
creates none. Main builds the actual release packages once without repeating the
protected PR’s unit suites.

Conservative Actions storage use is a standing owner requirement. Keep only
small artifacts needed for an active acceptance test, for at most seven days;
avoid duplicate uploads, build/dependency directories and unneeded caches.
With authorization, remove obsolete artifacts/caches after use, preserving
published releases/license material and useful CI history. Do not add a scheduled
cleanup workflow or delete another repository's storage without a concrete need
and authorization. After PR #5 merged, its 5,572,061-byte test artifact was deleted;
the repository API then reported zero Actions artifacts and zero caches.

## Scope and accepted functional tests

**The fork maintainer’s hands-on camera testing covers VIOFO A229 Pro 3CH on
Ubuntu 24.04 LTS and Windows 11 x64.** Windows 11 worked on the tested computer
without an additional codec/plugin installation. Windows 10 with a paid HEVC
extension and Mac real-footage playback remain unconfirmed. Other camera support
remains implemented; this limited test scope is not a report of other-camera breakage.

The user confirmed successful Linux playback after contributor `gubiq`’s PR #2
libva/DMABuf changes; keep those workarounds. PR #3 added channel-local error
recovery, health-aware GPS caching and progressive/durable GPS preparation.
The user accepted Play/Pause during analysis, segment transitions, switching
A→B→A mid-analysis and restart/resume. Version 0.5.0 was confirmed installed.

For 0.6.0, the user accepted route-click seeking across segment boundaries,
fullscreen timeline/speed graph collapse, paused zoom/pan, the final transport
visibility, bookmarks including persistence after restart, and channel checkboxes.
GPS preparation may be a few seconds faster than the earlier ~20-second segment
baseline on an external USB SSD; it was not precisely measured. Do not claim a
controlled speedup factor or exhaustively tested encoding/import/platform matrix.

## Current implementation

- Stable video elements; canonical sync clock is separate from visual primary/visibility. Failed cameras leave healthy ones running; recovered cameras rejoin as slaves during a source. Preserve validated media-health fingerprinting, probe/backoff and promoted-clock behaviour.
- GPS parser **v8**, archive schema **17**. Shared candidate/batch pool caps reads at two; ordered selection/boundary checks and raw per-segment checkpoints preserve deterministic results. Source/health fingerprints invalidate stale caches. Parsing stays outside long DB locks; trip selections rejoin in-flight work.
- Camera checkboxes save hidden labels in layout preferences. Visible cameras determine CSS placement, including full-height two-camera and full-grid single-camera views; all-hidden mode expands the available map. Videos and sync/audio master stay mounted/selected.
- Fullscreen uses one player root and logical dashboard/video navigation. Dashboard has the lower transport/timeline toggle; single-camera keeps upper Play/Pause. Paused zoom/pan resets on resume/source change.
- Timeline/map/bookmarks reuse trip-time seeking. Bookmarks are integer milliseconds in the archive DB, independently of GPS. Right-click graph adds; marker left-click seeks/right-click removes; arrow-bar buttons navigate without wrapping. Writes serialize; archive checks/stale-response guards protect switching. ID changes cascade and merges rebase positions transactionally.
- Update awareness reads only published stable fork releases through an eight-second bounded request, shares a fifteen-minute raw-response cache and offers a validated same-repository package for the compiled target. Unknown targets or absent matching installers use the release page. It does not install silently, upload media or suggest downgrades. No upstream private signing key/plugin is used.
- Ubuntu Debian packages include playback dependencies, original icons and complete license material. The normalized desktop ID remains com.tripviewer.app. Package upgrades do not alter archive/user data; old AppImage launchers may need user-level migration as documented in the installation guide.
- Report a bug targets the fork; empty-folder guidance is camera-neutral. The existing Miltona GPS export is registered in Tauri, matching the UI invocation. This is not additional real-Miltona validation.
- Timelapse persisted speed-curve envelope remains **v1**. The inherited v2 proposal and old reported spike are historical, not an assertion that this checkout writes all proposed fields. See the corrected [alignment note](../TIMELAPSE_ALIGNMENT_REDESIGN.md).

## Validation evidence

- Local 0.7.0 checks: 92 frontend tests, frontend typecheck/build, 339 Rust tests (one existing ignored), thirteen CI/version-policy tests, 56 IPC calls and 17 DB migrations checked. The Debian package built and its identity, declared dependencies, desktop entry, icon and six notice files were validated. Final PR CI passed, including Ubuntu package installation/dependency resolution and AppImage integration; the merged release workflow also succeeded.
- Native 0.7.0 WebKit checked update availability with synthetic GitHub responses, the download button, offline non-blocking behaviour, no downgrade and unchanged video elements. The native diagnostic IPC command responds. A live GitHub check returned the actual published 0.6.0 and correctly classified the installed 0.7.0 build as newer, without offering a downgrade. Camera visibility 3→2→1→0, map-only, saved choices, dashboard controls and hidden-master Play/Pause passed. GUI installation through Ubuntu App Center still needs the small owner acceptance check; these are not new hardware-camera certifications.

- The accepted 0.6.0 tree and the separate upstream contribution each passed 85 frontend tests, typecheck/build and 339 Rust tests (one existing ignored). Final PR CI and the merged-main Ubuntu release succeeded. Documentation-only corrections do not invalidate the accepted playback test.
- Native WebKit verified cross-segment bookmarks/seeking, camera visibility 3→2→1→0 and restoration, saved preferences, dashboard controls, advancing Play/Pause with a hidden canonical master, and unchanged video nodes without media reloads. Bulk-selection delete/merge dialogs were cancelled with all scratch files retained.
- Native synthetic H.264 scans/playback passed Wolf Box 3, Thinkware 2, Miltona 1 and Generic 4 channels. Their naming/grouping and Wolf Box/Miltona decoder files were unchanged from the shared upstream base. These fixtures contain no proprietary legacy GPS; no real-camera certification follows. Details: [compatibility](COMPATIBILITY.md).
- Original/current logos/icons, LICENSE and existing third-party notice text remain intact. Archive schema is 17 and GPS parser version is 8.

## Branch protection: active and verified

Both rulesets now apply to `refs/heads/main` with `enforcement: active`:

- `25574527`: deletion/non-fast-forward prohibited; PR, resolved threads and strict successful `ubuntu` check required; **no bypass actors**.
- `25574534`: branch updates restricted; only repository admin role 5 may bypass **via PR only**. This personal repository’s only administrator listed by GitHub is the owner.

Verified via ruleset detail, effective branch rules and collaborators endpoints on
2026-10-21. Earlier integration 403/setup-blocked notes are historical. The owner
activated the rules; do not report main as unprotected or attempt force/admin
bypasses. [Branch protection](BRANCH_PROTECTION.md) explains checks and the
owner-run setup/recovery helper. No destructive push/merge was used to test it.

## Contributor/upstream communication

The approved follow-up to gubiq was posted in
[fork PR #2](https://github.com/Marcus-P/trip-viewer/pull/2#issuecomment-6315937674).

The owner created [upstream PR #16](https://github.com/chrisl8/trip-viewer/pull/16)
after the cloud integration's create-PR request returned HTTP 403. It offers
Marcus-P:contrib/viofo-playback-tools at 8b634af, based on upstream a9617fc. The
message exactly matches the approved text, including exclusive VIOFO A229 Pro
3CH / Ubuntu 24.04 LTS hands-on testing and the limits of synthetic legacy checks.
Do not create a duplicate PR or issue notification. Preserve the contribution
branch while the upstream PR is open; use its page for current review/CI status.
Its first Verify updater dry-run awaited maintainer approval, not a failed test.
Do not approve or bypass the upstream maintainer gate.

After the 0.7.0 merge, the owner authorized suitable updates to that same PR.
Follow-up commit `757556226bf0bdfc3993f4d23e2e3292754192d0` adds camera-neutral
empty-library guidance and registers the existing Miltona diagnostic IPC command.
TypeScript typecheck, locked native compilation and static checks for 56 IPC calls
and 17 migrations pass. No new real-camera test is claimed. Upstream versions,
dependencies, updater/signing, release workflows and original notices remain
unchanged. The contribution branch push updated PR #16 successfully. Updating
its description through the GitHub App returned HTTP 403 (resource not accessible
by integration); the existing approved description remains and the follow-up is
documented in the commit and contribution compatibility note. Do not create a
duplicate notification or attempt to bypass upstream permissions.

The contribution preserves gubiq's commits/co-author notices and upstream's
updater, application versions, signing and release workflows. It excludes the
fork's release policy and large documentation refresh. Coordinate with A129
[PR #12](https://github.com/chrisl8/trip-viewer/pull/12); neither it nor issue #8
proves this fork was tested with A129 recordings.

## Remaining work / deferred ideas

- Await upstream review of PR #16; retain its branch and address concrete review findings when requested.
- Broader original-recording camera/firmware, codec and OS validation remains open; preserve existing support and test evidence.
- Maintenance: mixed static/dynamic `archive.ts` Vite import warning, large bundle, careful cargo-sweep setup, exact toolchain `NO_STRIP` effects, targeted dependency/audit review. Do not conflate these with a media regression or run blind audit fixes.
- Further feature proposals (unapproved): audio-source picker, mirror views, clip/snapshot/GPS exports; preserve historical design ideas without claiming implementation.
- Generally useful upstream contributions should branch from current upstream and isolate suitable changes from fork branding/release configuration. Preserve contributor authorship rather than submitting the entire fork history as a single patch.
