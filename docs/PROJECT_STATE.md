# Trip Viewer project state

Current hand-off for the Marcus-P fork. Preserve upstream authorship, logos/icons,
MIT/dependency notices and original media. The [documentation index](README.md)
is the entry point; the [prior development record](history/PROJECT_STATE-before-doc-refresh.md)
retains detailed debugging history and earlier acceptance results.

## Origin and release state

- Parent/source repository: [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer), verified through GitHub metadata.
- Stable/default branch: `main`. Published stable release: **0.5.0**, merge `f9681ea`.
- Open candidate: **PR #4**, branch `feat/route-seek-fullscreen-tools`, version **0.6.0**. New commits in this same unmerged PR keep that version. It is not a published stable 0.6.0 download.
- Only the owner merges protected main. No assistant merge is authorized by this documentation task; owner acceptance of tested features is recorded separately from merging/releasing.
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

- Latest local full suite: **80 frontend tests**, **335 Rust tests passed**, one pre-existing ignored. Prior frontend typecheck/build and native WebKit feature checks passed; this turn changes documentation only.
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

The user now requests a report to upstream owner **chrisl8**, with **explicit
approval of the message before sending**. Prepare the concrete text/channel for
review; do not post it without that approval. State origin/attribution and the
exclusive VIOFO A229 Pro 3CH / Ubuntu 24.04 LTS hands-on test scope, while
separating automated/synthetic evidence for other cameras from hardware tests.
Upstream issue #8 concerns A129 footage and PR #12 (alinnastac) adds A129 Duo
support; neither is proof that this fork was tested on A129. Coordinate
respectfully and avoid representing this fork’s Ubuntu changes as an upstream
regression fix for hardware not tested here.

## Remaining work / deferred ideas

- Owner review/merge of PR #4 after fresh CI. No separate application rebuild is needed solely to review documentation.
- Obtain user approval before sending the upstream message. Do not place an unapproved outreach draft in public repository comments/issues.
- Broader original-recording camera/firmware, codec and OS validation remains open; preserve existing support and test evidence.
- Maintenance: mixed static/dynamic `archive.ts` Vite import warning, large bundle, careful cargo-sweep setup, exact toolchain `NO_STRIP` effects, targeted dependency/audit review. Do not conflate these with a media regression or run blind audit fixes.
- Further feature proposals (unapproved): audio-source picker, mirror views, clip/snapshot/GPS exports; preserve historical design ideas without claiming implementation.
- Generally useful upstream contributions should branch from current upstream and isolate suitable changes from fork branding/release configuration. Preserve contributor authorship rather than submitting the entire fork history as a single patch.
