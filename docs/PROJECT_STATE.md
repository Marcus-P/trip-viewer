# Trip Viewer project state

Current hand-off for the Marcus-P fork. Preserve upstream authorship, logos/icons,
MIT/dependency notices and original media. The [documentation index](README.md)
is the entry point; the [prior development record](history/PROJECT_STATE-before-doc-refresh.md)
retains detailed debugging history and earlier acceptance results.

## Final 0.6.0 integration and cleanup plan

The user authorized incorporating the two upstream commits, reducing redundant CI/artifacts, preparing a final single user test, and cleaning obsolete branches/artifacts. Preserve published releases/license assets. Main is owner-protected: **wait for the final user test, then the owner clicks Merge on PR #4**. Do not merge before that test or bypass rules.

Upstream `2534f65` (developer guidance) and `a9617fc` (bulk trip selection/deletion, archive-only merge safety and scoped orphan timelapse cleanup) are integrated as actual ancestry so the fork will no longer be two commits behind after PR #4 merges. Authorship remains intact. The upstream slow reconciliation/master-led Play behaviour already exists in our stronger, user-validated SyncEngine; retain its seek timeouts, generation cancellation, audio policy and one reconciler rather than adding a second timer or regressing transport. Added a rejected-slave regression check. Upstream archive-only guards run before modification; our bookmark rebasing remains inside the merge transaction.

Latest local verification: **85 frontend tests, typecheck/build, 339 Rust tests (one existing ignored)**. These include selection/range semantics, sequential bulk failures, rejected slave playback, archive-only merge-loss prevention and merge-scoped orphan cleanup. Native final checks passed: bookmarks/cross-segment seeking, channel visibility/fullscreen/Play-Pause, Ctrl/Shift trip selection, both bulk scope dialogs and merge-dialog cancellation with all nine scratch files intact. Legacy 1/2/3/4-channel synthetic smoke was repeated. Required Ubuntu AppImage CI is the last automated gate before handing off the final test build. The maintainer's real-camera scope remains exclusively VIOFO A229 Pro 3CH / Ubuntu 24.04 LTS.

CI now runs once per PR update (or manual dispatch), with concurrency cancelling superseded runs. No duplicate push+PR runs; main is validated/published by the release workflow after the owner merge. Test artifacts expire after 7 days. Remove obsolete CI test artifacts only after the final package is available, keeping that one candidate for testing. Existing stable releases/assets are version history, not temporary CI artifacts.

The user approved the outreach text, but requests the **upstream PR after successful final test and owner merge**. Do not post a duplicate issue comment now. A whole fork-main PR would also replace upstream updater/release configuration and flood the review with fork/history docs. Prepare a focused upstream contribution branch based on current chrisl8 main, retaining its updater/release setup and authorship; offer the useful camera/playback features and tests with the approved VIOFO/Ubuntu-only real-test scope. Coordinate with existing A129 PR #12; do not claim other real cameras have been tested here. After owner merge/release, remove only obsolete merged branches, keep main and any active upstream contribution branch, prune superseded test artifacts and publish the upstream PR with the approved message.

The concrete final test and merge location are in [FINAL_060_CHECK.md](FINAL_060_CHECK.md).

## Origin and release state

- Parent/source repository: [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer), verified through GitHub metadata.
- Stable/default branch: `main`. Published stable release: **0.5.0**, merge `f9681ea`.
- Open candidate: **PR #4**, branch `feat/route-seek-fullscreen-tools`, version **0.6.0**. New commits in this same unmerged PR keep that version. It is not a published stable 0.6.0 download.
- Only the owner merges protected main. The final owner merge is gated on the requested single test; accepted earlier feature tests remain established.
- Preserve `com.tripviewer.app` identity and upstream icon assets.

## Scope and accepted functional tests

**The fork maintainer has limited hands-on functional testing exclusively to
VIOFO A229 Pro 3CH on Ubuntu 24.04 LTS.** Other camera support remains implemented;
this limited test scope is not a report of other-camera breakage.

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
- Timelapse persisted speed-curve envelope remains **v1**. The inherited v2 proposal and old reported spike are historical, not an assertion that this checkout writes all proposed fields. See the corrected [alignment note](../TIMELAPSE_ALIGNMENT_REDESIGN.md).

## Validation evidence

- Latest local full suite: **85 frontend tests**, **339 Rust tests passed**, one pre-existing ignored. Prior frontend typecheck/build and native WebKit feature checks passed; the upstream integration keeps the validated transport implementation.
- Functional head `9045fcf` passed both complete Ubuntu CI runs including AppImage/installer checks. New documentation commits receive fresh CI; do not assert their status before checking.
- Native WebKit verified camera visibility 3→2→1→0 and restoration, saved preferences, dashboard controls, advancing Play/Pause with a hidden canonical master, and unchanged video nodes without media reloads. Native bookmark checks covered no-seek right-clicks, cross-segment navigation, trip-switch restore and persistent removal; SQLite reopen/merge/identity tests passed.
- This documentation refresh also verified unchanged legacy naming/grouping and Wolf Box/Miltona decoder files against shared ancestor `82c377d` and inspected upstream main `a9617fc`. Native synthetic H.264 scans/playback passed Wolf Box 3, Thinkware 2, Miltona 1 and Generic 4 channels. These fixtures contain no proprietary legacy GPS; no real-camera certification follows. Details: [compatibility](COMPATIBILITY.md).
- Upstream/current logos/icons, LICENSE and all existing third-party notice text are retained by this documentation refresh.

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

The approved follow-up to `gubiq` was posted:
https://github.com/Marcus-P/trip-viewer/pull/2#issuecomment-6315937674.

The user approved the report text to upstream owner **chrisl8** and asks to use it in the upstream PR **after final test and owner merge**. Do not post a duplicate issue comment before that stage. State origin/attribution and the
exclusive VIOFO A229 Pro 3CH / Ubuntu 24.04 LTS hands-on test scope, while
separating automated/synthetic evidence for other cameras from hardware tests.
Upstream issue #8 concerns A129 footage and PR #12 (alinnastac) adds A129 Duo
support; neither is proof that this fork was tested on A129. Coordinate
respectfully and avoid representing this fork’s Ubuntu changes as an upstream
regression fix for hardware not tested here.

## Remaining work / deferred ideas

- Owner review/merge of PR #4 after fresh CI. No separate application rebuild is needed solely to review documentation.
- The message is approved; wait for final test/owner merge, then create the focused upstream PR using that text and current release facts.
- Broader original-recording camera/firmware, codec and OS validation remains open; preserve existing support and test evidence.
- Maintenance: mixed static/dynamic `archive.ts` Vite import warning, large bundle, careful cargo-sweep setup, exact toolchain `NO_STRIP` effects, targeted dependency/audit review. Do not conflate these with a media regression or run blind audit fixes.
- Further feature proposals (unapproved): audio-source picker, mirror views, clip/snapshot/GPS exports; preserve historical design ideas without claiming implementation.
- Generally useful upstream contributions should branch from current upstream and isolate suitable changes from fork branding/release configuration. Preserve contributor authorship rather than submitting the entire fork history as a single patch.
