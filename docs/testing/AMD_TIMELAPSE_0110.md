# AMD / Linux GPU timelapse acceptance (0.11.0)

This tests encoding, not playback decoding or NPU acceleration. Actual Radeon
780M/VA-API operation cannot be established in a cloud without those devices.

## Windows 11 / Radeon 780M

1. Close Trip Viewer. Install the candidate Windows x64 setup executable from
   the feature PR's **CI → Artifacts → Trip-Viewer-windows…** download (the
   artifact is a ZIP; extract it and double-click the setup executable).
2. Open **Timelapse → gear**, keep/select your FFmpeg executable and press
   **Test**. Expect **Encoding verified: AMD / AMF** on a compatible AMD system.
   A separate working NVIDIA GPU takes priority. If CPU is shown, check that
   this FFmpeg includes `hevc_amf` and the AMD graphics driver is installed.
3. Choose a single short trip, **Front**, **8x** and start encoding. Rebuild only
   that selection if it already has an output. Expect progressing video time,
   an eventual ETA and **AMD / AMF** in live progress, then a playable output.
4. Optionally open Task Manager → Performance → GPU and select **Video Encode**.
   CPU load is expected: AMF currently decodes and scales on CPU. GPU 3D/NPU
   graphs do not establish video-encoder use. Report the displayed encoder and
   success/error; no full playback/GPS regression session is requested.

## Linux with a compatible API

Use the candidate Ubuntu DEB/AppImage from the same CI run. Install an FFmpeg
build with `hevc_vaapi` (or `hevc_amf` and its AMD runtime), a compatible graphics
driver and normal render-device access. On AMD/Mesa systems this commonly means
the distro's VA-API driver package; package names and codec availability vary.
Do not change device access to world-writable as a workaround.

In the gear dialog press **Test**. Expect **GPU / VA-API**, or **AMD / AMF** if
that path verifies first, unless working NVENC takes priority. Multiple render
nodes are checked automatically and the passing node is used for jobs. Repeat
only the short Front/8x encoding above and confirm the output plays.

## Fallback / limits

If no GPU pipeline passes, **CPU / HEVC** is a valid result. If the GPU becomes
unavailable during a job, it retries once using CPU and later batch jobs stay
on CPU. Cancel must stop work without starting a CPU retry. Driver/build/device
errors can be read in terminal logs and saved job errors; avoid repeated full
library rebuilds for diagnosis. Synthetic checks do not certify every GPU, OS,
HEVC bit depth or footage combination, and no fixed speed gain is promised.

Main publishes durable installers only after owner-controlled promotion; these
acceptance artifacts expire after seven days. Permanent `develop` is retained.
