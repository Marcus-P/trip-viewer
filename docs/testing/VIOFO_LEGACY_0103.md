# Counter-free VIOFO filenames: focused 0.10.3 test

This is a candidate from `fix/viofo-legacy-filenames`, not a published release.
Use a copy of the recordings; originals need not be renamed or changed. The
fix uses the same automatic recognition for numbered and counter-free files.

1. Install the Ubuntu DEB or Windows installer from the fix PR's successful CI
   artifacts, closing the current app first. Confirm version 0.10.3.
2. Open/rescan a folder containing the two owner-reported timestamp/channel
   files. Both must appear in one segment as Front/Interior rather than invalid
   filenames. Verify that actual video playback starts.
3. Check a folder containing numbered and counter-free recordings. Both must be
   recognized without switching a firmware/model setting. A genuinely absent
   Rear channel must stay absent.
4. If GPS was recorded, verify normal GPS analysis completes and the route is
   positioned relative to the same filename timestamp. This does not guarantee
   GPS in recordings that contain no usable GPS data.
5. Optionally encode one short segment/channel at 8x and confirm a playable
   output. The automated FFmpeg test already checks all nine channel/tier
   combinations and multi-window progress using mixed filename shapes.

For an extracted Ubuntu installer artifact, in its directory:

```bash
sha256sum --ignore-missing -c SHA256SUMS.txt
sudo apt install ./Trip.Viewer_0.10.3_test-*_Ubuntu-24.04_amd64.deb
tripviewer
```

The full earlier overlay/bookmark/map/GPS-resume acceptance does not need to be
repeated. Firmware-history review and structural compatibility are separate:
a successful scan/playback of these files does not certify every past firmware.
