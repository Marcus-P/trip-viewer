# Automatic performance calibration — 0.13.0 acceptance preview

Current evidence: Ubuntu Intel UHD/CML GT2 and Windows 11 Radeon 780M workloads
and playback are accepted. 0.12.0 detects supported FFmpeg APIs and validates
finite synthetic and real-input samples. It selects the first working encoder
by a fixed priority, then the first usable processing profile. It does not
benchmark every candidate or select the fastest measured one. AMF/QSV/VA-API
auto-concurrency remains one; NVENC and CPU use conservative legacy heuristics
and a RAM ceiling. Hardware-decode filters currently use two threads.

Requested direction: automatic selection of the fastest stable configuration
for supported hardware without per-model code additions. Absolute maximum
performance on arbitrary hardware or 100% utilization of every engine cannot
be guaranteed. The useful objective is valid output per unit wall time within
memory/session constraints.

The owner authorized implementation on `feat/automatic-performance-calibration`.
The preview uses the following bounded implementation:

1. Discover compiled encoder/decoder/filter APIs and actual device candidates;
   validate the whole chain. Evaluate available GPU scaling (D3D11/AMF under
   Windows, CUDA/VA-API where supported), retaining current proven fallbacks.
2. Compare short equivalent real-source samples using measured throughput and
   fixed output/quality settings; include frame-transfer costs. Tune filter
   threads and then batch concurrency only within conservative live memory and
   session limits. Single-job speed and batch throughput are separate measures.
3. Bound calibration time/work and use no full-trip benchmark. Reuse a local
   result keyed to runtime/device/driver and codec/resolution/bit-depth/rate
   characteristics, invalidating it when relevant dependencies change. No
   media/GPS/installation telemetry or automatic diagnostic upload.
4. Keep fallback/backoff and accepted output preservation. Raise concurrency
   at job boundaries only if measured aggregate throughput improves and memory
   permits; avoid oscillation and repeated failed probes. Do not blindly fill
   all GPU/CPU meters or remove the fixed memory safeguards.

Acceptance: automated selection/cache/budget/failure/cancellation tests and
focused measurements on the two available owner machines. More hardware can
report through the existing GitHub issue form. Platform support remains based
on working APIs and tests, not universal certification.

## Preview implementation and limits

The worker reads a representative requested channel before each trip and spends
at most 45 seconds cumulatively per batch on calibration. Measurements use four
seconds of real input at 8x, production 1920-pixel scaling, normal encoder quality
settings and null output. Startup, transfer and draining count toward wall time.
The baseline runs twice; improvements require two successful measurements at
least 15% faster than the previous selection. The fastest baseline result and
the slower candidate repeat make selection conservative. Exhausted budgets,
failed probes and cancellation never become failed production jobs by themselves.

Available encoder/decoder/filter APIs determine candidates. Windows tests native
D3D11, AMF VPP or QSV scaling if compiled and applicable; CUDA and VA-API retain
existing paths. Filter-thread candidates are 1/2/4/8 within core/memory limits.
Aggregate throughput, not individual latency, determines measured concurrency
(1–4). A user concurrency setting is a ceiling. A live 2-GiB OS reserve plus a
pessimistic 12-GiB allowance per job preserves the existing 4K safety margin.
Unknown memory availability means one job (currently including macOS). Low-RAM
machines remain sequential. Actual preflight/fallback remains mandatory.

The local settings cache holds eight results for seven days. Its key hashes
FFmpeg version/path/file metadata, OS/runtime/driver details and the input video
header (codec, resolution, pixel format/bit depth, frame rate), plus workload size.
No source path, media, GPS or installation data is uploaded. Changing source
characteristics, binaries or known driver fingerprints forces new measurement.
If driver fingerprinting fails, persistent reuse is disabled. On Linux, the
fingerprint includes kernel, DRM device/driver and userspace VA-driver metadata.
A hardware processing/encoding fallback clears calibration and retires extra
workers before they take new jobs; in-flight jobs finish or use existing fallback.

This is a bounded search for a useful measured improvement, not an exhaustive
benchmark of every adapter or certification of every codec/driver combination.
Candidates later in the queue may not fit the budget. Fixed 8x samples may not
predict every variable-speed event mix. Hardware allocation, disk speed and
thermal conditions can change after calibration; live memory checks and fallback
still apply. The cache is per-machine; it does not change archive database format.

## Focused acceptance

Install the unique RC package. Select Front and 16x, then use the row rebuild
button for one already-tested trip (expect one job, not the whole library).
Capture the terminal output where possible. Record calibration selection,
completion time and whether the finished output plays. Repeat that one job to
check cache reuse; cancel once during calibration if it is still measuring.
On the 8-GB Intel Ubuntu notebook, sequential operation is expected. On Windows
Radeon 780M, native scaling may be chosen only if that FFmpeg/driver combination
actually passes and beats the baseline. Neither result is required to show
100% GPU utilization. No additional long library-wide encode is requested.
