# GPU encoding compatibility and community reports

Trip Viewer selects processing by successful FFmpeg tests, not a GPU-model
allowlist. Windows tries NVENC, AMF and Intel QSV; Linux supports NVENC, available
AMF runtimes and VA-API. A compatible HEVC encoder, driver/runtime and FFmpeg
build are required. All GPU probes use finite Full-HD input and drain to EOF.
Where CUDA processing or VA-API scaling is unavailable, a second profile checks
CPU processing while retaining GPU encoding. CPU/libx265 is the final fallback.
The selected profile is verified again before a batch. No fixed speed gain or
support for every GPU/driver/build combination is promised.

## Owner-reported hardware results

| GPU | OS | API | Evidence and scope |
| --- | --- | --- | --- |
| Intel UHD Graphics, Comet Lake / CML GT2 (Core i5-10210U) | Ubuntu 24.04 LTS | VA-API | Owner confirmed app encoding completion and playable 8x output. FFmpeg 6.1.1; the complete Intel media driver resolved an initial scaling failure. |
| AMD Radeon 780M (Ryzen 9 8945HS) | Windows 11 x64 | AMF | Owner confirmed a standalone 30-frame Full-HD NV12 encode. The old tiny test stalled. Updated in-app probe and real footage acceptance are still pending. |

These reports were supplied on 2026-11-17. They do not certify every recording,
driver or OS update. NVIDIA/NVENC and Windows Intel/QSV are implemented and
covered by automated selection/command tests; no owner hardware acceptance has
yet been reported for those paths. Other compatible models may work and reports
are welcome. Do not turn a synthetic probe result into a real-footage claim.

The Intel Linux notebook already uses its GPU for scaling and encoding. Decode
still runs on CPU there. The owner observed 75–85% CPU load and some swapping;
no controlled speed comparison was made. GPU decoding and further throughput
tuning are follow-ups, not prerequisites for the accepted encoding result.

## Report a setup or request a focused test

Open a [GPU hardware-test issue](https://github.com/Marcus-P/trip-viewer/issues/new?template=gpu-hardware-test.yml).
The form becomes available when this change reaches the default branch; before
then, use a [plain issue](https://github.com/Marcus-P/trip-viewer/issues/new).
Include GPU model, OS, Trip Viewer/test-build version, FFmpeg and driver version
if known, the displayed encoder, and the result from one short Front/8x trip.
See the [focused test guide](testing/AMD_TIMELAPSE_0110.md).

The discussion is asynchronous and replies may take several days. The maintainer
can review the report with coding assistance and suggest a bounded diagnostic
command or test build. You execute tests locally and share the output yourself;
there is no automatic remote control, issue-monitoring agent or telemetry service.
Share only technical details and redact personal paths, usernames, GPS data and
secrets. Original footage is not required for the initial report. Compatibility
entries will distinguish reported probe success from completed real encoding.
