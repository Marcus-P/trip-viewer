Isolated Ubuntu 24.04 x86_64 GPS diagnostic candidate for PR21 follow-up.

**Manual prerelease only: not Latest, no stable updater metadata, no change to stable v0.13.0 or upstream PR21.**

The prior candidate stored empty GPS checkpoints for all tested segments. This revision replaces a whole-box size rejection with bounded-memory reading of padded VIOFO GPS boxes. Synthetic regression tests preserve the original last-marker selection across read-window boundaries. This is a plausible regression fix, not yet a confirmed diagnosis on owner recordings.

Reader failures now propagate instead of being cached as successful empty GPS. Healthy sibling-camera fallback remains available. Map status distinguishes active preparation, failure and completed empty results. Native diagnostics report request, progress, completion and errors. Reader version9 revalidates old version8 analysis once; the main archive schema and migration head remain13.

Close the old PR21 test, extract this package and run `TRIPVIEWER_GPS_TIMING=1 ./start-test.sh`. Select one existing scratch-archive trip with videos paused. No new footage copy, timelapse encode or manual cache deletion is needed. Output appears in the terminal and local last-run.log. Do not publish raw logs containing private paths or location data.

This targets GPS only. Previously reported playback stutter, high CPU and fullscreen transport drift remain unresolved. Successful packaging/startup and automated tests do not certify real hardware behavior.

Upstream/contributor authorship and license notices remain intact, including @gubiq's Linux media preparation and the synthetic A129 layout fixture adapted from @alinnastac's contribution. Codex assists with implementation; owner hardware acceptance remains pending.
