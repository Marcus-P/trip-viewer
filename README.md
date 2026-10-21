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

Download the latest published stable version from
[Latest Release](https://github.com/Marcus-P/trip-viewer/releases/latest).

### Install and update without a terminal

Published Ubuntu 24.04 LTS x86_64 releases include a `.deb`
installer. Open [Latest Release](https://github.com/Marcus-P/trip-viewer/releases/latest),
expand **Assets**, download `Trip.Viewer_<version>_Ubuntu-24.04_amd64.deb`, close
Trip Viewer and double-click the package. Choose **Install** (or **Update**) in
Ubuntu's App Center/Software Install, then launch **Trip Viewer** from Applications.
No GitHub account or command line is needed for a public release download.

The app checks published stable releases and shows an update hint in the navigation
bar. **Settings → Application updates** also has **Check for updates** and a download
button. Download the new package, close the app and double-click it to upgrade.
This is a user-confirmed package update, not an unattended background installation.
Only releases built from merged `main` are offered, not every source commit or PR.

If your release has no `.deb` asset, use the AppImage option below.
Existing AppImage users should follow the
[migration instructions](docs/INSTALLATION.md#switching-from-an-appimage)
to avoid an older user shortcut masking the installed package.

### AppImage and command-line installation

Ubuntu 24.04 LTS x86_64 command-line installation of the latest release
(requires the [GitHub CLI](https://cli.github.com/), `gh`). Close the running app
first. The command resolves one release tag and downloads its bundle, including
the original license and dependency notices:

```bash
TRIPVIEWER_TAG="$(gh api repos/Marcus-P/trip-viewer/releases/latest --jq .tag_name)" &&
TRIPVIEWER_VERSION="${TRIPVIEWER_TAG#v}" &&
mkdir -p "$HOME/Downloads/TripViewer-$TRIPVIEWER_VERSION" &&
cd "$HOME/Downloads/TripViewer-$TRIPVIEWER_VERSION" &&
gh release download "$TRIPVIEWER_TAG" --repo Marcus-P/trip-viewer \
  --pattern "Trip.Viewer_${TRIPVIEWER_VERSION}_Ubuntu-24.04_bundle.tar.gz" \
  --pattern SHA256SUMS.txt --clobber &&
sha256sum --ignore-missing -c SHA256SUMS.txt &&
tar -xzf "Trip.Viewer_${TRIPVIEWER_VERSION}_Ubuntu-24.04_bundle.tar.gz" &&
bash install-trip-viewer-ubuntu.sh \
  "./Trip.Viewer_${TRIPVIEWER_VERSION}_Ubuntu-24.04_amd64.AppImage"
```

The helper installs the AppImage under `~/.local/opt/trip-viewer/`, retaining
the existing icon and `com.tripviewer.app` desktop identifier. Start it from
the application overview, then add it to GNOME Favorites if desired.
For direct execution, `chmod +x` the downloaded AppImage and run it.
Source builds do not automatically replace an already installed app.
Without `gh`, download the Ubuntu bundle and `SHA256SUMS.txt` from Latest Release,
verify the bundle checksum, extract it, then run the included installer.

Archive databases use schema 17. Older builds that only support schema 16
cannot open an upgraded archive. Use a compatible stable release rather than
downgrading that archive.

## What is available

Trip Viewer includes synchronized multi-camera playback, VIOFO embedded GPS,
a speed graph, map follow, resizable views, per-camera error recovery,
progressive/resumable GPS preparation, imports, scans/tags, review, places and
optional ffmpeg timelapses. Some features originate upstream; this is not a
claim that every workflow has been manually retested by the fork maintainer.

Playback tools include route-click seeking, collapsible fullscreen timeline,
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
