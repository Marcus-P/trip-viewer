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

## Later checkpoint: Windows AMD computer

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
