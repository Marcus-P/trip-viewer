# Final 0.6.0 test and owner merge

This final candidate combines the accepted fork features with upstream commits
2534f65 and a9617fc. The real-footage test scope is VIOFO A229 Pro 3CH on Ubuntu
24.04 LTS. Existing camera support remains implemented without a new hardware claim.

## One final test

Install the final test AppImage from PR #4's latest successful CI artifact, or
build the latest `feat/route-seek-fullscreen-tools` branch as in the Linux guide.
Check that version 0.6.0 is shown and that the build corresponds to the final PR head.

In one session:

1. Play a familiar trip, seek over a segment boundary, then Pause/Play. Check a
   bookmark, the camera checkboxes and dashboard timeline briefly; their earlier
   acceptance is retained, this is an integration smoke check.
2. In the trip sidebar, Ctrl-click two trips and Shift-click a range. Open the
   new Delete originals/Delete trips confirmation, verify its stated scope and
   press **Cancel**. No original recordings need to be deleted for this check.
3. Restart once and confirm the bookmarked trip still loads and keeps its mark.

Archive-only merge-loss guards, scoped orphan cleanup and partial bulk failures
are covered by automated tests; no destructive manual test on valuable footage
is required. Report "Test okay" or the specific failed step.

## Owner merge after the test

Open https://github.com/Marcus-P/trip-viewer/pull/4. Once the test is accepted
and the required `ubuntu` check is green, use **Merge pull request**, then
**Confirm merge**. The actual target is `main` (the repository's stable branch).
Use the normal merge commit to retain contributor and upstream ancestry.
Do not use squash/rebase/force/admin options to bypass a blocked check.

The release workflow then builds and publishes 0.6.0. Publication is complete
only after that workflow succeeds; updating GitHub main does not replace the
locally installed app automatically.

## After merge

The approved upstream message is used in a focused PR to chrisl8/trip-viewer,
without replacing that project's updater/release setup. Do not send a duplicate
issue notification beforehand. Remove obsolete merged fork branches after
ancestry verification; retain main and any active contribution branch. Remove
superseded test artifacts, retaining the final candidate until the stable 0.6.0
release is verified. Published release/license assets remain version history.
