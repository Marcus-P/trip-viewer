<img src="icon/icon-128.png" align="left" width="96" alt="Trip Viewer icon"/>

# Trip Viewer

Synchronized dashcam video, GPS maps and trip review. This is the
[Marcus-P fork](https://github.com/Marcus-P/trip-viewer) of
[chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer), built on the
original authors’ work. The upstream icon, copyright and license notices are retained.

**The fork maintainer’s functional testing is limited to VIOFO A229 Pro 3CH
on Ubuntu 24.04 LTS.** Existing Wolf Box, Thinkware, Miltona MNCD60 and generic
camera support remains in the code. Other cameras/platforms are not newly
certified by this fork; lack of confirmation does not mean they stopped working.
See the [compatibility evidence](docs/COMPATIBILITY.md).

<br clear="left"/>

## Download and install

The latest published stable release is **0.5.0**. The **0.6.0** changes are in
[PR #4](https://github.com/Marcus-P/trip-viewer/pull/4), not yet a stable release.
Check [Releases](https://github.com/Marcus-P/trip-viewer/releases) for subsequent
published versions; Actions test artifacts identify their commit and expire.

Ubuntu 24.04 LTS x86_64 command-line installation of the published 0.5.0:

```bash
mkdir -p ~/Downloads/TripViewer-0.5.0 &&
cd ~/Downloads/TripViewer-0.5.0 &&
curl -fLO https://github.com/Marcus-P/trip-viewer/releases/download/v0.5.0/Trip.Viewer_0.5.0_Ubuntu-24.04_amd64.AppImage &&
curl -fLO https://github.com/Marcus-P/trip-viewer/releases/download/v0.5.0/install-trip-viewer-ubuntu.sh &&
curl -fLO https://github.com/Marcus-P/trip-viewer/releases/download/v0.5.0/SHA256SUMS.txt &&
sha256sum --ignore-missing -c SHA256SUMS.txt &&
bash install-trip-viewer-ubuntu.sh ./Trip.Viewer_0.5.0_Ubuntu-24.04_amd64.AppImage
```

The helper installs the AppImage under `~/.local/opt/trip-viewer/`, retaining
the existing icon and `com.tripviewer.app` desktop identifier. Start it from
the application overview, then add it to GNOME Favorites if desired.
For direct execution, `chmod +x` the downloaded AppImage and run it.
Source builds do not automatically replace an already installed app.

An archive upgraded by the 0.6.0 test build uses database schema 17. The older
0.5.0/schema-16 build cannot open that upgraded archive; use the current test
build while evaluating PR #4 rather than downgrading that archive.

## What is available

Stable 0.5.0 includes synchronized multi-camera playback, VIOFO embedded GPS,
a speed graph, map follow, resizable views, per-camera error recovery,
progressive/resumable GPS preparation, imports, scans/tags, review, places and
optional ffmpeg timelapses. Some features originate upstream; this is not a
claim that every workflow has been manually retested by the fork maintainer.

The 0.6.0 candidate adds route-click seeking, collapsible fullscreen timeline,
paused zoom/pan, persistent bookmarks, saved camera-visibility checkboxes and
bounded GPS reads. See [release notes](RELEASE_NOTES.md) and the
[user guide](docs/USER_GUIDE.md) for controls and storage behaviour.

## Run and build on Linux

Ubuntu media playback uses WebKitGTK/GStreamer and depends on available codecs:

```bash
sudo apt update
sudo apt install -y gstreamer1.0-libav gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly
```

See [Linux development setup](LINUX_DEV_SETUP.md) for build dependencies, FUSE,
other distributions, and troubleshooting. Optional timelapse encoding needs a
separately installed ffmpeg binary configured in the app.

```bash
npm ci
npm test
npm run build
cargo test --locked --manifest-path src-tauri/Cargo.toml
NO_STRIP=true npm run tauri -- build --bundles appimage
```

Use Node.js 22.12+ LTS or a newer supported LTS and stable Rust. The build output is
`src-tauri/target/release/bundle/appimage/Trip Viewer_<version>_amd64.AppImage`.
For release/version/protection rules, see [RELEASING.md](RELEASING.md).

## Documentation

The [documentation index](docs/README.md) links user, compatibility, architecture,
release and maintenance guides. Historical upstream design notes remain available
and are explicitly separated from the current implementation.

## License, origins and attribution

Trip Viewer remains [MIT licensed](LICENSE), with the original copyright and
permission notice intact. The original project is
[chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer); its authors and
contributors retain credit. Contributor [gubiq](https://github.com/gubiq)
supplied the Linux libva/DMABuf fixes incorporated via fork PR #2.

The existing Trip Viewer icon is retained from upstream; no separate icon
license file is present. Third-party licenses, including React Leaflet’s
Hippocratic-2.1 terms, remain applicable: see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)
and release license inventories. No VIOFO logo, firmware or proprietary app code
is included. VIOFO/A229 Pro are descriptive compatibility names, with no affiliation
or endorsement implied.

VIOFO-specific and related fork changes were developed with ChatGPT by OpenAI,
based on requirements, sample footage, testing and validation supplied by Marcus P.
This does not replace attribution for the original implementation or contributions.
