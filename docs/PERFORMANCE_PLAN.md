# Automatic performance calibration — on hold, not implemented in 0.12.0

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

Candidate implementation, requiring a separate feature branch/release:

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
