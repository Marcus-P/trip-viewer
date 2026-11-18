# GPU encoding compatibility and community reports

Trip Viewer selects processing by successful FFmpeg tests, not a GPU-model
allowlist. Windows tries NVENC, AMF and Intel QSV; Linux supports NVENC, available
AMF runtimes and VA-API. A compatible HEVC encoder, driver/runtime and FFmpeg
build are required. All GPU probes use finite Full-HD input and drain to EOF.
Where CUDA processing or VA-API scaling is unavailable, a second profile checks
CPU processing while retaining GPU encoding. CPU/libx265 is the final fallback.
The selected profile is verified again before a batch. Version 0.13.0 adds
[local performance calibration](PERFORMANCE_PLAN.md); the focused owner results
below record the accepted workloads and their evidence limits. No fixed speed gain or
support for every GPU/driver/build combination is promised.

## Owner-reported hardware results

| GPU | OS | API | Evidence and scope |
| --- | --- | --- | --- |
| Intel UHD Graphics, Comet Lake / CML GT2 (Core i5-10210U) | Ubuntu 24.04 LTS | VA-API | Owner confirmed app encoding completion and playable 8x output. FFmpeg 6.1.1; the complete Intel media driver resolved an initial scaling failure. |
| AMD Radeon 780M (Ryzen 9 8945HS) | Windows 11 x64 | AMF | Standalone 30-frame Full-HD NV12 test and updated in-app AMF probe pass. The owner confirmed all nine 0.11.0 jobs completed and play correctly. In the revised 0.12.0 RC, D3D11VA decoding + AMF encoding were observed in the Front/16x run, which completed with correct playback. Driver 32.0.12033.1030. |

These reports were supplied on 2026-11-17. They do not certify every recording,
driver or OS update. NVIDIA/NVENC and Windows Intel/QSV are implemented and
covered by automated selection/command tests; no owner hardware acceptance has
yet been reported for those paths. Other compatible models may work and reports
are welcome. Do not turn a synthetic probe result into a real-footage claim.

In stable 0.11.0 the Intel Linux notebook uses its GPU for scaling and encoding,
while decoding runs on CPU. The owner observed 75–85% CPU load and some swapping;
no controlled speed comparison was made. The 0.12.0 acceptance preview adds hardware decoding: VA-API under Linux,
D3D11VA under Windows, CUDA for NVENC and experimental VideoToolbox on Mac.
Selection uses the available APIs and verified processing profiles. The owner tested 0.12.0-rc.44180688812 on the Intel notebook on 2026-11-18:
VA-API decoding passes the short probe, but the long real run reported
"Cannot allocate memory" during hardware filtering and retried with software
processing while retaining GPU encoding. All three 8x channel outputs completed
and 8x playback was confirmed. This validates recovery, not sustained GPU decoding
on that system. The precise runtime allocation cause remains unconfirmed; the next RC adds
finite VA-API surface reserves, bounded processing depth and a real-source
preflight. The revised Linux and Windows results below supersede the earlier pending
acceptance status; no controlled speedup claim follows.
The table above records encoding results.
Linux VA-API prefers device-resident frames through scaling/encoding; Windows
D3D11VA downloads native 8/10-bit frames for the existing software filters.
A processing failure retries with software processing before dropping GPU encoding.
The displayed decode profile is the requested pipeline, not live utilization.
See the [decoding acceptance test](testing/GPU_DECODE_0120.md).

## Report a setup or request a focused test

Open a [GPU hardware-test issue](https://github.com/Marcus-P/trip-viewer/issues/new?template=gpu-hardware-test.yml).
You can also open a [plain issue](https://github.com/Marcus-P/trip-viewer/issues/new).
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

## Revised Ubuntu decoding result (2026-11-18)

The revised 0.12.0-rc.44263923946 completed a real Intel UHD/CML GT2 Front/16x
job with VA-API encoding/decoding and GPU scaling in 850.6 seconds. The supplied
log contains no memory error or processing fallback. This is a successful real
job after the surface-pool follow-up; the owner also explicitly confirmed
problem-free playback of the finished 16x video.
It does not certify Windows D3D11VA, every recording or a speedup factor.

## Windows decoding acceptance (2026-11-18)

The owner accepted the revised 0.12.0-rc.44263923946 on Windows 11 x64 with
Radeon 780M. The probe and running Front/16x screenshot show AMD / AMF encoding
and GPU / D3D11VA decoding; completion and correct output playback were then
confirmed. For a trip reported as 23m54s and about 11 GB, Front conversion was
estimated at two to three minutes. This is an owner estimate, not a controlled
benchmark or verified Front-only byte count. There is no saved Windows run log
for a stronger claim about every phase or transient fallback. Previous 0.11.0
AMF encoding acceptance remains valid. No additional owner run is required for
version finalization. Other GPU/driver setups and Mac hardware remain unverified.

## 0.13.0 owner acceptance

The owner accepted Windows 11 x64 / Radeon 780M Front/8x encoding and finished
playback. The running screenshots show D3D11VA decoding, AMF GPU scaling and
AMF encoding, about 100% Video Codec utilization and about 3–4% CPU use.
No exact final Windows elapsed time or controlled same-workload speedup was
reported. Ubuntu 24.04 LTS / Intel UHD CML GT2 Interior/16x also completed:
the owner reported about five to six minutes and correct finished playback;
the running screenshot shows VA-API decoding/encoding. That report does not
separately establish cache reuse, selected scaler, increased concurrency or
absence of transient fallback. No additional owner encoding run is required.
macOS universal packaging and basic launch passed CI; physical Mac/GPU
acceptance remains unavailable, so macOS remains experimental.
