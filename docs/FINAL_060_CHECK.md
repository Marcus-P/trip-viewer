# Completed 0.6.0 acceptance and release

The owner accepted the final integration test, merged
[PR #4](https://github.com/Marcus-P/trip-viewer/pull/4), and the
[release workflow](https://github.com/Marcus-P/trip-viewer/actions/runs/39664733283)
succeeded. [Version 0.6.0 is published](https://github.com/Marcus-P/trip-viewer/releases/tag/v0.6.0).
For the current stable download and installation, use
[Latest Release](https://github.com/Marcus-P/trip-viewer/releases/latest) and the
[README](../README.md#download-and-install).

This is a completed acceptance record, not a request to repeat the test or merge
an already merged PR. The deleted feature branch and expired/removed CI packages
are not current installation sources.

## Accepted integration test

The final build combined the fork features with upstream commits 2534f65 and
a9617fc. Hands-on footage testing covered exclusively VIOFO A229 Pro 3CH on
Ubuntu 24.04 LTS; existing camera support remains without a new hardware claim.

The owner confirmed:

1. Playback, seeking across a segment boundary and Pause/Play, with a brief
   check of bookmarks, camera checkboxes and dashboard timeline.
2. Ctrl-click and Shift-click multi-trip selection, opening the new deletion
   confirmation, checking its scope and cancelling without deleting originals.
3. Restarting the app and confirming the bookmarked trip retained its mark.

Archive-only merge-loss guards, scoped orphan cleanup and partial bulk failures
were covered by automated tests. No destructive test on valuable footage was
required. The final PR check and the merged-tree release workflow both passed.

## Merge and cleanup record

The owner merged PR #4 with a normal merge commit,
`865bfcd841cfd7e1be6a54799943ac06f7b1f80a`, preserving contributor/upstream
ancestry. The stable tag points to that commit. The release includes the AppImage,
installer, checksums and license material. Publishing does not automatically
replace an app already installed on a user's computer.

Obsolete merged branches and all temporary CI packages were removed after
release verification; published release/license history remains. The approved
message is now in [upstream PR #16](https://github.com/chrisl8/trip-viewer/pull/16),
whose separate contribution branch retains upstream updater/release configuration.
Keep that contribution branch while the PR is open. See
[project state](PROJECT_STATE.md) for the current hand-off.
