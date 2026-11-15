# Focused 0.10.2 timelapse acceptance

This is a candidate from `fix/viofo-timelapse`, not yet a stable release. Close
the installed app before starting the test package. Use a copy of a small VIOFO
archive. Original recordings are read only; timelapse output is separate.

## Get the Ubuntu test installer

Open the fix PR's successful CI run and download its Ubuntu installer artifact.
It contains a commit-labelled `.deb`, `SHA256SUMS.txt` and `BUILD_INFO.txt`.
Extract the artifact, then run in its directory:

```bash
sha256sum --ignore-missing -c SHA256SUMS.txt
sudo apt install ./Trip.Viewer_0.10.2_test-*_Ubuntu-24.04_amd64.deb
tripviewer
```

With an installed, authenticated GitHub CLI, the same artifact can be retrieved
using `gh run download RUN_ID --repo Marcus-P/trip-viewer --pattern
'Trip-Viewer-Ubuntu-24.04-*' --dir /path/to/test-download`; replace RUN_ID with
the exact successful run linked from the fix PR and use the extracted artifact
subdirectory. This avoids compiling locally. The test package stays in Actions
for at most seven days; permanent downloads follow the main release.

The owner accepted working encoding and authorized stable promotion after the
ETA follow-up passes automated checks. The steps below remain useful optional
post-install checks; a new manual acceptance round is not a promotion gate.

## Small acceptance test

1. Confirm version 0.10.2. In Timelapse → gear, run Test once. Older encoder-list
   caches are invalidated; the FFmpeg path is kept. Expect either verified GPU
   encoding (NVENC/CUDA) or verified CPU encoding. A CPU-only machine is valid.
2. Select one short VIOFO trip, Front only, 8x and Retry failed / Rebuild all as
   appropriate. Start encoding and confirm a playable output is produced.
3. Select Interior/Rear for the same trip and verify their outputs. Missing real
   camera files must stay missing, not be substituted with another recording.
4. If convenient, check 16x/60x on that short trip. CPU encoding can be much
   slower than GPU encoding; the status must not claim unavailable GPU support.
   During the first channel, phase progress and a current-step ETA must update
   before any job completes. Elapsed time is always visible. A stale progress
   message withholds the step ETA; preparation/encoding/finalization reset their
   phase indicators explicitly. No fixed encoding-speed guarantee is implied.
5. If anything fails, expand Encoding error details and copy its message. Test
   cancellation on one running job if a CPU encode is long enough; it must not
   restart the job via fallback after cancellation.

There is no need to repeat accepted map, GPS, bookmarks, overlay or archive-switch
workflows. New positive GPU hardware testing and Windows/macOS footage testing
are not established by the cloud's CPU/synthetic checks.
