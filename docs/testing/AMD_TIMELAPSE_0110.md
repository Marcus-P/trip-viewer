# AMD / Intel / Linux GPU timelapse acceptance (0.11.0)

This tests encoding, not playback decoding or NPU acceleration. Positive GPU operation cannot be established in the cloud without those devices.
The owner has confirmed Intel UHD VA-API encoding and playable 8x output on Ubuntu;
Windows AMF/QSV hardware validation remains pending.

## Windows 11 / Radeon 780M

1. Close Trip Viewer. Install the candidate Windows x64 setup executable from
   the feature PR's **CI → Artifacts → test-desktop-Windows-x64-…** download (the
   artifact is a ZIP; extract it and double-click the setup executable).
2. Open **Timelapse → gear**, keep/select your FFmpeg executable and press
   **Test**. Expect **Encoding verified: AMD / AMF** on a compatible AMD system.
   A separate working NVIDIA GPU takes priority. If CPU is shown, check that
   this FFmpeg includes `hevc_amf` and the AMD graphics driver is installed.
3. Select **Front** and **8x**, then use the **Rebuild** arrow on one short trip. Expect progressing video time,
   an eventual ETA and **AMD / AMF** in live progress, then a playable output.
4. Optionally open Task Manager → Performance → GPU and select **Video Encode**.
   CPU load is expected: AMF currently decodes and scales on CPU. GPU 3D/NPU
   graphs do not establish video-encoder use. Report the displayed encoder and
   success/error; no full playback/GPS regression session is requested.

## Windows 11 / other GPU vendors

The same installer automatically prefers verified NVIDIA NVENC, AMD AMF and then
Intel Quick Sync. Use a compatible FFmpeg build with the corresponding HEVC
encoder and a working graphics driver. For mixed-vendor Intel adapter selection,
use a current FFmpeg build; this does not combine all installed GPUs or claim
that the first passing backend is fastest. Hardware without supported HEVC encode
may use CPU even if it supports GPU-accelerated playback.

For Intel expect **Encoding verified: Intel / QSV**. Repeat only the short
Front/8x test using the trip row's **Rebuild** arrow, then play the output. AMF
and QSV still decode/scale on CPU. Record the verified/active encoder and result;
there is no need to run the whole library or repeat accepted playback/GPS tests.

## Linux with a compatible API

Use the candidate Ubuntu DEB from the same CI run. Install an FFmpeg
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

## Confirmed Intel Linux test

On 2026-11-17 the owner confirmed encoding completion and playable 8x output on
Intel UHD Graphics (Comet Lake), Ubuntu 24.04 LTS, FFmpeg 6.1.1, 8 GiB RAM.
`intel-media-va-driver` initially failed at GPU scaling with unsupported
VAProfile. Replacing it with Ubuntu's `intel-media-va-driver-non-free`, restarting
the app and retesting enabled **GPU / VA-API**. This package is not a purchase;
see the [user guide](../USER_GUIDE.md) for the installation command.

Observed CPU use was 75–85% with no obvious speed gain. Decoding is still on CPU;
no controlled performance comparison was made. Treat GPU decode/transfer tuning
as follow-up work, not proof that the hardware encoder failed or that it is
universally faster.
