<img src="icon/icon-128.png" align="left" width="96" alt="Trip Viewer icon"/>

# Trip Viewer

**Dashcam playback with synchronized multi-camera video and GPS, with native VIOFO A229 Pro 3CH support.**

This repository is a fork of [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer). The current fork release is focused on **Ubuntu 24.04 LTS** and has been tested with **VIOFO A229 Pro 3CH** footage.

<br clear="left"/>

## Download

Go to this repository's [Releases](https://github.com/Marcus-P/trip-viewer/releases) page and download:

```text
Trip.Viewer_0.4.0_Ubuntu-24.04_amd64.AppImage
```

GitHub Releases are used for the distributable build, so the AppImage remains available there until the release itself is deliberately removed. It is not an expiring GitHub Actions artifact.

### Run directly

```bash
chmod +x Trip.Viewer_0.4.0_Ubuntu-24.04_amd64.AppImage
./Trip.Viewer_0.4.0_Ubuntu-24.04_amd64.AppImage
```

### Install for Ubuntu / pin to the Dock

Download `install-trip-viewer-ubuntu.sh` from the same release and run:

```bash
chmod +x install-trip-viewer-ubuntu.sh
./install-trip-viewer-ubuntu.sh ./Trip.Viewer_0.4.0_Ubuntu-24.04_amd64.AppImage
```

The installer copies the AppImage to a stable per-user location, installs the existing Trip Viewer icon and a desktop entry, and uses Trip Viewer's GTK application ID so GNOME can associate the running window with the launcher. Start Trip Viewer from the Ubuntu application overview once, then right-click its Dock icon and choose **Add to Favorites**.

To remove that per-user integration:

```bash
rm -rf ~/.local/opt/trip-viewer
rm -f ~/.local/share/applications/com.tripviewer.app.desktop
rm -f ~/.local/share/icons/hicolor/256x256/apps/trip-viewer.png
```

## Ubuntu requirements

The release is built and validated on **Ubuntu 24.04 LTS x86_64**. HEVC playback uses WebKitGTK/GStreamer. Install the commonly required plugins with:

```bash
sudo apt update
sudo apt install -y \
  gstreamer1.0-libav \
  gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad \
  gstreamer1.0-plugins-ugly
```

## VIOFO A229 Pro 3CH support

The VIOFO implementation recognizes normal and parking recordings with the A229 Pro F/I/R naming scheme, groups the three channels, and extracts/validates embedded GPS without relying on geographic filters or fixed recording durations.

Examples:

```text
2026_0831_065229_000646F.MP4
2026_0831_065229_000647I.MP4
2026_0831_065229_000648R.MP4

2023_0821_180010_062PF.MP4
2023_0821_180010_063PI.MP4
2023_0821_180010_064PR.MP4
```

The release also includes:

- synchronized Front / Interior / Rear playback
- VIOFO GPS extraction with timestamp-based validation across continuous segment boundaries
- speed graph with km/h or mph scale, grid and hover readout
- GPS map with near-center follow behavior and resize-aware layout
- resizable player, secondary cameras, GPS panel and timeline
- persisted sidebar and player layout
- single-camera fullscreen plus F/I/R + GPS dashboard fullscreen with deterministic navigation between both modes
- one-shot re-synchronization at relevant playback/view transitions
- stable Play/Pause behavior across speed changes and tab changes
- audio at Original 1×; audio is intentionally muted at 0.5×, 2×, 4× and timelapse modes on Linux to avoid WebKitGTK/GStreamer time-stretch corruption
- Player kept mounted while visiting Scan/Review/Timelapse tabs to avoid repeated WebKitGTK media-pipeline teardown
- camera model and speed-unit preferences

## Known Linux behavior

A short visible interruption can occur when playback crosses from one original MP4 segment to the next. This is accepted for this release because WebKitGTK/GStreamer must initialize the next media pipeline.

Audio is intentionally available only in **Original / 1×** mode. Returning to 1× rebuilds the master audio pipeline at the same playback position before sound is enabled again.

The **Scan** tab runs analysis/tag scans. With **New segments only**, pressing **Start scan** can finish almost immediately when every segment has already been analyzed. This is separate from choosing/scanning a VIOFO footage folder.

## Other dashcams

The upstream Trip Viewer code already contains support for Wolf Box, Thinkware, Miltona MNCD60 and generic channel naming. This fork has specifically been validated for VIOFO A229 Pro 3CH on Ubuntu 24.04.

## Development

### Prerequisites

- Node.js 20+
- Rust via rustup
- Ubuntu 24.04 build dependencies for Tauri/WebKitGTK
- GStreamer plugins listed above

A typical Ubuntu development setup also needs:

```bash
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  pkg-config \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libxdo-dev \
  patchelf
```

### Build and test

```bash
npm ci
npm test
cargo test --manifest-path src-tauri/Cargo.toml
NO_STRIP=true npm run tauri -- build --bundles appimage
```

The local AppImage is written under `src-tauri/target/release/bundle/appimage/`.

## Release process

The Ubuntu release process is documented in [RELEASING.md](RELEASING.md). The workflow builds on Ubuntu 24.04, runs the automated tests, produces an AppImage and checksum, and creates a normal GitHub Release. This fork deliberately does **not** use the upstream auto-updater signing key.

Windows packaging is intentionally deferred to a later, separate feature branch.

## License and attribution

Trip Viewer source code in this repository remains under the [MIT License](LICENSE), preserving the upstream copyright and permission notice. Third-party dependencies keep their own licenses; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and the license inventories attached to each release.

The existing Trip Viewer icon is retained from the MIT-licensed upstream repository. No separate icon-license file is present in this repository.

**VIOFO** and **A229 Pro** are used only to describe hardware compatibility. This project is not affiliated with or endorsed by VIOFO.

The original Trip Viewer project and its upstream work remain credited to its original authors and contributors.

Technical implementation of the VIOFO-specific work and related fork changes was generated with ChatGPT by OpenAI based on requirements, sample data, testing and validation provided by Marcus P.
