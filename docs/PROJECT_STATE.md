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

Documentation maintenance 0.6.1 corrects the stale pre-release descriptions and
0.5.0 installation commands that remained after publication. It changes no
application behaviour, parser version or database schema and needs no new
real-footage acceptance test. Under the existing every-merged-PR version rule,
this separate maintenance change increments PATCH. Publication is still verified
through Latest Release rather than inferred from the source version.

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
CI runs once per PR update, cancels superseded work and retains test packages for
seven days. Main's release workflow verifies the merged tree before publication.

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
