# VIOFO and playback contribution: validation scope

These additions were developed/tested in the Marcus-P fork of chrisl8/trip-viewer.
Original project attribution, icons and licenses are retained. Björn Fries
(`gubiq`) authored the archive-auto-scan and Linux libva/DMABuf fixes; their
commits and co-author notices are preserved in this contribution branch.

The fork maintainer's hands-on functional testing was limited exclusively to
**VIOFO A229 Pro 3CH on Ubuntu 24.04 LTS**. This covers playback/GPS preparation,
segment transitions, map seeking, dashboard timeline, paused zoom, bookmarks
including restart and channel visibility. No other real camera/OS is newly certified.

Existing Wolf Box, Thinkware, Miltona and generic implementations remain. Their
naming/grouping and Wolf Box/Miltona decoder files are unchanged from upstream.
Automated tests pass, and generated H.264 clips in the legacy naming formats
were scanned/decoded/played in native WebKit with 3/2/1/4 channels respectively.
Those synthetic clips contain no proprietary camera GPS tracks; they cannot
confirm real device/firmware/codec behaviour. Lack of confirmation does not mean
those cameras stopped working. Thinkware still has no GPS decoder as before.
A129 models were not tested by the fork maintainer; coordinate with PR #12's work.

Source changes include VIOFO normal/parking recognition and GPS validation,
resumable progressive extraction, bounded reads, persistent camera health,
playback recovery, shared trip-time route/bookmark seeking, paused zoom and saved
camera visibility. Database schema advances to 17; upgraded archives cannot be
opened by schema-13-only upstream builds. GPS parser version is 8 and the
existing speed-curve envelope remains v1.

This branch retains upstream package/application versions, updater, release
workflows, signing configuration and existing documentation rather than applying
the fork's Ubuntu-only release policy. The upstream maintainer controls the
release/version decision. No original user media is rewritten by these features.
