# Linux development and runtime setup

The fork’s hands-on functional checks cover **VIOFO A229 Pro 3CH on Ubuntu
24.04 LTS**. CI also builds on Ubuntu 24.04 x86_64. Other camera/platform support
is retained but has a different [evidence level](docs/COMPATIBILITY.md).
The [inherited Linux guide](docs/history/inherited-LINUX_DEV_SETUP.md) preserves
upstream Fedora/Bazzite/distrobox recipes and old AppImage reports; those reports
are not assertions about the current Ubuntu-built fork on every distribution.

## Ubuntu 24.04 build dependencies

Install Node.js **22.12+ LTS or a newer supported LTS**, stable Rust through rustup,
and the native packages:

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev build-essential pkg-config libssl-dev \
  libayatana-appindicator3-dev librsvg2-dev libxdo-dev patchelf \
  gstreamer1.0-libav gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly
```

Use the project lockfiles:

```bash
npm ci
npm test
npm run build
cargo test --locked --manifest-path src-tauri/Cargo.toml
npm run tauri -- dev
```

For an AppImage, close the running development instance and use:

```bash
NO_STRIP=true npm run tauri -- build --bundles appimage
```

Output: `src-tauri/target/release/bundle/appimage/Trip Viewer_<version>_amd64.AppImage`.
This is the fork’s CI-tested bundling command. `NO_STRIP` is a bundling environment
setting, not a promise that it controls Rust’s Cargo release stripping settings.
Its complete toolchain-specific effects remain a maintenance investigation.
The optional `pretauri` cargo-sweep hook warns and skips when cargo-sweep is absent;
that warning alone is not a build failure.

## Install a local test build

Check out the intended branch and pull before building. Building a new AppImage
leaves any previously installed binary unchanged. To update the manual path used
for testing, after a successful build:

```bash
VERSION="$(node -p "require('./package.json').version")"
install -Dm755 "src-tauri/target/release/bundle/appimage/Trip Viewer_${VERSION}_amd64.AppImage" \
  "$HOME/Applications/TripViewer.AppImage"
"$HOME/Applications/TripViewer.AppImage"
```

Close the installed app before replacement. For GNOME/Dock integration instead,
run `bash scripts/install-trip-viewer-ubuntu.sh /path/to/the.AppImage`; it uses
`~/.local/opt/trip-viewer/Trip.Viewer.AppImage`, a different per-user path.
See [README](README.md) for published release downloads/checksums.

## Runtime requirements and diagnostics

WebKitGTK/GStreamer codec availability and compatible GPU drivers affect
playback. The AppImage bundles media/framework components but still interacts
with host libraries; it is not a guarantee of codec or ABI portability to every
distribution. Install the GStreamer packages above if required. Ubuntu 24.04
systems needing AppImage FUSE mounting can install `libfuse2t64`; alternatively
AppImage’s `--appimage-extract-and-run` mode avoids FUSE mounting.

The fork preserves the libva/DMABuf changes from contributor `gubiq`, including
respecting explicit environment overrides. Do not remove these validated Linux
workarounds as incidental cleanup. Inspect actual video/decoder errors rather
than treating GTK optional-module or Vite bundle-size warnings as media failures.

For GPS timing, launch a new process:

```bash
TRIPVIEWER_GPS_TIMING=1 ~/Applications/TripViewer.AppImage
```

`TRIPVIEWER_GPS_WORKERS=1` or `2` selects the bounded candidate-reader count.
Compare uncached comparable footage; cached trips intentionally skip extraction.
No controlled hardware speedup factor has been established.

Optional timelapse creation needs a separately installed ffmpeg, selected in the
app. Decoder availability and hardware encoders must be checked for that build.

## Other Linux distributions

Fedora/atomic-distro toolbox or distrobox and Arch dependency recipes remain in
the [historical guide](docs/history/inherited-LINUX_DEV_SETUP.md). Treat them as
starting points to update for current package versions. The fork does not claim
new hands-on validation on those systems, nor that they are necessarily broken.
Use native dependency/codec probing and a matching development environment when
diagnosing ABI/plugin errors. Windows 11 has a successful owner test report;
Windows 10 with a paid HEVC extension and Mac real-footage playback remain
unconfirmed. See the [compatibility record](docs/COMPATIBILITY.md).
