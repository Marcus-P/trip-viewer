# GPU decoding acceptance (0.12.0 preview)

This preview extends the accepted 0.11.0 GPU encoder paths. Install the test
package from the feature PR's successful CI run, not an old artifact or the
stable Latest Release. The displayed version must be `0.12.0-rc.<run ID>`.
Close Trip Viewer before installing. Ubuntu uses the `.deb` installer; its
package-control version intentionally spells the prerelease as `~rc.<run ID>`.
Windows uses the x64 setup installer. The future stable 0.12.0 sorts above both.

## First checkpoint: Ubuntu Intel notebook

1. Open Timelapse, open the FFmpeg gear and click **Test** using the existing
   working FFmpeg installation. It should show **GPU / VA-API** for the encoder
   and **Decode profile: GPU / VA-API** if hardware decoding passes. A CPU
   decode profile remains a safe, explicit fallback; report it if unexpected.
2. Select just **Front**, **8x**, and one short trip. Use **Rebuild all** only
   for that selected short trip if an output already exists. Do not rebuild
   the whole library for this checkpoint.
3. Check that the encoding phase names the decode profile, reaches completion
   and the 8x output plays correctly, including a segment transition if present.
   Note elapsed time and approximate CPU load; a controlled comparison is needed
   before claiming any speedup. Preparing source is a stream-copy phase and
   does not measure decoder throughput.

## Windows AMD checkpoint (completed)

The owner has already confirmed all nine 0.11.0 AMF outputs complete and play.
No repeat encoding is requested now. When testing this next stage, the FFmpeg
check should show **AMD / AMF** encoding and **GPU / D3D11VA** decoding when the
runtime test passes. Test just one short Front/8x output and playback then.
CPU scaling and frame transfer can still consume CPU on this path. The NPU is
not used. Other Windows GPUs are selected by verified APIs, not a model list.

## Report results

Use the [hardware-test issue form](https://github.com/Marcus-P/trip-viewer/issues/new?template=gpu-hardware-test.yml).
Include app/FFmpeg version, GPU, OS, encoder/decode profile, completion and
playback results. Replies can take days. No automatic diagnostics upload,
remote control or telemetry is enabled. Avoid publishing recordings, GPS data,
personal paths or credentials. macOS VideoToolbox remains experimental and
requires separate hardware acceptance.

## Memory follow-up: evidence and focused retest

The first Ubuntu RC passed its short probe and later completed three playable
8x outputs, but logged `Cannot allocate memory` and activated software
processing with the same GPU encoder. Treat that as recovery acceptance.
It does not establish that hardware decoding sustained the real workload.

FFmpeg 6.1's [VA-API VPP pool](https://github.com/FFmpeg/FFmpeg/blob/n6.1/libavfilter/vaapi_vpp.c)
uses a default of ten surfaces through
[filter initialization](https://github.com/FFmpeg/FFmpeg/blob/n6.1/libavfilter/avfilter.c).
The [VA-API allocator](https://github.com/FFmpeg/FFmpeg/blob/n6.1/libavutil/hwcontext_vaapi.c)
also returns allocation failure when its fixed pool is exhausted. This error
therefore does not prove host-RAM exhaustion.
[Equal-size scaling can bypass VPP](https://github.com/FFmpeg/FFmpeg/blob/n6.1/libavfilter/vf_scale_vaapi.c).
The follow-up uses an explicit 20-surface scaler pool, lower queue/thread depth,
real resizing in the synthetic 8x probe and a bounded actual-source preflight.
Codec reference surfaces and driver allocations still exist; this is not a
hard cap on total process/driver memory and does not promise every GPU works.

For the new RC, close the old app and install the new artifact using its exact
ZIP filename; older PR #21 ZIPs may also exist in Downloads. After the FFmpeg
Test, launch the app with a terminal log and test only one short Front/8x trip:

```bash
tripviewer 2>&1 | tee /tmp/tripviewer-gpu-memory.log
```

Prefer a short trip that contains normal full-resolution Front footage, not a
lower-resolution generated timelapse. If it is already completed, rebuild only
that short selected trip/channel/tier. Report whether the log shows
`decoder=vaapi software-scale=false`, whether a hardware-processing/preflight
failure appears, and whether the new output plays. The new preflight may take
up to twenty seconds; no full-library rebuild is needed. A successful short
check is the first checkpoint, not proof against all long-running driver leaks.

## Owner acceptance result

The revised RC passed the real Linux Front/16x workload without logged fallback,
and the owner confirmed playback. Windows 11 Radeon 780M also passed the observed
D3D11VA + AMF Front/16x run and finished-output playback. The Windows conversion
time was an approximate owner report; no saved Windows job log exists. These
checks are complete for this release. Do not ask the owner to repeat them for
release-note or version-only finalization. Keep broader hardware/performance
claims separate from this evidence.
