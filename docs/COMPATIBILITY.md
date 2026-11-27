# Camera compatibility and validation scope

## What the maintainer has actually tested

**The Marcus-P fork maintainer’s hands-on camera testing covers VIOFO A229 Pro
3CH on Ubuntu 24.04 LTS, Windows 11 x64 and Windows 10 with the required HEVC
codec installed.** The Ubuntu acceptance includes playback,
GPS preparation/resume, map seeking, fullscreen timeline, paused zoom, persistent
bookmarks and channel-checkbox checks. On 2026-10-23, the owner reported that
Trip Viewer works on Windows 11 without installing another codec or plugin; no
additional per-feature or hardware-matrix coverage is inferred from that report.
It is not a declaration that every
inherited import/encoding/platform workflow was exhaustively tested.

Existing support for other cameras is retained. Lack of a new real-camera test
means **unconfirmed here**, not “removed”, “unsupported now”, or “known broken”.
Their continued operation cannot be fully certified from filenames or source alone.

## Evidence by format

| Format | Implemented naming/channels | GPS implementation | Evidence in this fork |
| --- | --- | --- | --- |
| VIOFO A229 Pro 3CH | F/I/R normal and parking names; dedicated recognition | Novatek/VIOFO descriptors, timing and validity/boundary checks | Maintainer real-footage checks on Ubuntu 24.04 LTS; successful Windows 11 report without additional codecs; Windows 10 Front/8x encoding and output playback after codec installation; parser/validation tests |
| Wolf Box | Existing F/I/R names and grouping | Existing ShenShu metadata decoder retained | Naming/grouping regression tests; native 3-channel synthetic playback; no new real-camera GPS certification |
| Thinkware | Existing front/rear names and grouping | **No Thinkware GPS decoder implemented**, as before | Naming/prefix/grouping tests; native 2-channel synthetic playback; no new hardware test |
| Miltona MNCD60 | Existing single-channel `.MOV` naming | Existing Novatek `gps0` decoder retained | Naming tests and sample-derived record/timestamp/coordinate tests; native 1-channel synthetic playback; no new full-recording hardware test |
| Generic four-channel | Existing letter/digit channel names | Historical ShenShu fallback; not a universal GPS decoder | Naming/grouping tests; native 4-channel synthetic playback; no certification of arbitrary generic GPS formats |

VIOFO uses a specific recognizer before the historical naming fallback and keeps
the persisted `generic` camera-kind value. Other VIOFO models, including the A129
Duo discussed upstream, were not hands-on tested by this fork maintainer.
Choosing **Auto** preserves other-format detection; the VIOFO-specific model
preference is intended for VIOFO footage, not a universal camera override.

## Verification performed on 2026-10-21

- Full repository tests: **85 frontend tests; 339 Rust tests passed; one existing
  ignored test**. These exercise shared logic and camera-format cases, not every
  real device, firmware, codec or driver combination.
- Source comparison: `scan/naming.rs`, `scan/grouping.rs`, `gps/shenshu.rs` and
  `gps/miltona.rs` are unchanged from shared upstream ancestor
  `82c377d9cb6c6cc36d3f19001a2261327aa9b805` and the inspected upstream main
  `a9617fcb0ec53bd23a627fa7f5c30cbd3a65d148`. GPS dispatch still selects the existing
  Wolf Box/Miltona decoders, leaves Thinkware GPS empty, and retains the non-VIOFO
  Generic fallback. Shared player/cache changes still require regression checking.
- Native WebKit/GStreamer smoke: copied a short generated **H.264** video under
  representative Wolf Box (3), Thinkware (2), Miltona (1) and Generic (4) names;
  scanned each separate archive through real IPC; checked camera kind/channel
  labels/grouping; decoded all channels; exercised Play/Pause and clock progress.
  All four cases passed in the managed Linux/WebKit development environment;
  Ubuntu AppImage build CI is a separate check. The `.MOV` sample was the same generated container
  with that filename extension. **No proprietary camera GPS tracks or original
  recordings from those other cameras were present in these fixtures.**

Representative synthetic naming used for repeatability:

```text
2026_03_23_094634_00_F.MP4       # Wolf Box: also I/R
REC_2026_03_06_07_25_52_F.MP4   # Thinkware: also R
FILE211202-151504-000406F.MOV    # Miltona
2026_03_06_072552_A.MP4         # Generic: also B/C/D
```

Re-run the automated suite with `npm test` and
`cargo test --locked --manifest-path src-tauri/Cargo.toml`. To repeat the native
smoke, use a generated playable clip with each naming set in separate scratch
folders; verify recognition, all expected labels and actual advancing video.
This verifies the shared software path, not the original cameras’ encodings.

## Platforms, codecs and remaining confirmation

Native CI builds Ubuntu AppImage/Debian packages, a Windows x64 NSIS installer
and an experimental universal macOS disk image. Native tests, installed package
contents and basic app-process launches are checked; universal Mac binaries must
contain both Intel and Apple Silicon code. Actual run results are recorded on
the relevant PR, not inferred from workflow configuration. **Windows 11 x64 is
the recommended Windows platform**, following the owner’s successful report
without an additional codec/plugin installation. HEVC support can still differ
between computers.

On the tested Windows 10 computer, the device-manufacturer HEVC Store package
linked by the app was unavailable. Microsoft’s general **HEVC Video Extensions**
was offered for €0.99. If HEVC support is missing, purchasing that extension at
the Store’s current displayed price is one possible route. On 2026-11-24 the
owner confirmed installing the required HEVC codec, then successfully encoding
Front/8x for all segments of two recognized trips and playing the finished
output on Windows 10. The current report does not establish which Store edition
or purchase route was used. The running screenshot shows Intel/QSV encoding,
D3D11VA decoding and CPU scaling with two threads; the exact GPU model was not
provided. This is scoped acceptance, not certification of every channel, tier
or Windows 10 system. Installing VLC alone
does not supply WebView2 with VLC’s HEVC decoder.

No Mac real-footage acceptance is available. Mac SD-card auto-discovery remains
unimplemented; manual
folder selection is available. Other-Linux guidance remains inherited.
Hardware decoding/HEVC support depends on the host media stack and drivers.
A synthetic H.264 smoke does not verify real HEVC dashcam footage on other systems.

To extend confirmation, use representative original recordings for each model,
firmware and OS: grouping, every camera, decode/seek/segment transitions, sound,
GPS timing/position if implemented, and restart/cache behaviour. Keep files
non-destructive and exclude personal locations/paths from public test reports.

This fork originates from [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer).
Upstream’s historical reports and other contributors’ tests remain credited to
those contributors; they are not relabeled as new tests performed by Marcus-P.

## GPU timelapse encoding acceptance

The 0.11.0 candidate's real Intel UHD (Comet Lake) VA-API test passed on Ubuntu
24.04 LTS / FFmpeg 6.1.1 after installing `intel-media-va-driver-non-free`.
The owner confirmed completion and playable 8x output. CPU load remained 75–85%;
a speed gain was not measured. This confirms that specific encoding path, not all
Intel GPUs or all video acceleration. The owner confirmed the AMD Radeon 780M AMF app probe and a progressing real
Windows 11 encoding job; completion/playback was subsequently owner-confirmed.
A later Windows 10 Front/8x test also completed and played correctly after codec
installation, with Intel/QSV shown during the run; the exact Intel GPU is unknown. See the [focused GPU test](testing/AMD_TIMELAPSE_0110.md).

## 70mai T800 support inherited from upstream

- **70mai 4K T800** (3-channel: front / rear / cabin) — filenames like `NO20261018-221543-000352F.MP4`, where the last letter is `F` (front), `R` (rear) or `C` (cabin). The camera saves each channel in its own folder (`Front/`, `Rear/`, `Cabin/`); Trip Viewer groups them by timestamp and counter, so just point it at the parent folder (e.g. `Normal/`). GPS (position, speed, heading) is decoded from the per-frame telemetry track, 1 Hz, and was checked against the on-screen overlay of a real clip. Only northern/eastern-hemisphere footage has been tested; southern/western positions are assumed to be stored as negative values. Event-clip naming (`EV…`) is assumed, not yet confirmed — samples welcome. 70mai SD cards are not auto-detected in the SD-card import list, so open the card with the folder import instead (non-destructive; the card is not wiped).
