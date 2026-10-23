# Install and update without a terminal

Download a published installer from
[Latest Release](https://github.com/Marcus-P/trip-viewer/releases/latest) → **Assets**.
Choose your platform's installer, not Source code ZIP/TAR. Packages appear after
the owner merges their PR and the release workflow succeeds. An Actions test
installer is an acceptance build, not a published stable release.

## Windows x64: Windows 11 recommended

1. Download `Trip.Viewer_<version>_Windows_x64_setup.exe` and close Trip Viewer.
2. Double-click the installer. It installs for your Windows account and adds a
   Start Menu shortcut. Missing WebView2 is downloaded by the installer, so an
   Internet connection is needed for that first-time runtime setup.
3. Start **Trip Viewer** from Start. The installer is not publisher-signed;
   SmartScreen may show an unknown-publisher prompt. Check that the file came
   from this repository's release before choosing Windows' More info / Run anyway
   option if offered. Do not turn off system-wide protection.
4. **Windows 11 x64 is the recommended, owner-tested Windows platform.** The
   owner confirmed that Trip Viewer works on their Windows 11 computer without
   installing any additional codec or plugin. This does not guarantee HEVC
   availability on every Windows 11 computer; decoding also depends on the
   existing system codecs and graphics drivers.
5. **Windows 10 x64: HEVC playback after purchasing a decoder is untested.** If
   HEVC/H.265 support is missing, search Microsoft Store for **HEVC Video
   Extensions** from Microsoft and, if desired, purchase it at the price shown
   there (€0.99 was reported during testing; price and availability can vary).
   Restart Trip Viewer afterwards. Playback is expected to be possible with a
   compatible decoder/driver combination, but has not been verified in this fork.
   The app’s existing Store button targets the device-manufacturer edition, which
   was unavailable on the tested Windows 10 computer; it is not a reliable free
   option for all devices. WebView2 and the HEVC decoder are separate requirements;
   installing Trip Viewer or VLC does not add VLC’s decoder to WebView2.

For updates, **Settings → Application updates → Download Windows installer**
opens the matching published `.exe`. Close Trip Viewer and run it to update in
place; the installer rejects older versions. Windows ARM-native packages are
not offered by this release. See the [compatibility record](COMPATIBILITY.md) for
the confirmed Windows 11 result and the untested Windows 10 paid-codec path.

## macOS: experimental Intel and Apple Silicon package

1. Download `Trip.Viewer_<version>_macOS_universal_experimental.dmg` for macOS 11+
   and open it. One universal app includes both Intel and Apple Silicon code.
2. Close Trip Viewer, drag **Trip Viewer** into **Applications**, then eject the
   disk image and open the app from Applications.
3. These builds are ad-hoc signed, without an Apple Developer ID/notarization.
   If macOS blocks a trusted download, use its Security & Privacy / Privacy &
   Security **Open Anyway** option after checking the release source. No command
   line or system-wide Gatekeeper disabling is part of this installation guide.

Use the matching download button in Settings for updates, close the app and
replace it in Applications. CI checks the universal executable, app identity,
notices, signature, disk-image payload and a basic process launch. **No Mac
real-recording playback, GPS, firmware or hardware functional test is claimed.**
Treat this package as experimental. SD-card auto-detection is not implemented on
macOS; choose recording/archive folders with the existing folder picker.

## Ubuntu

Download the `.deb` installer for **Ubuntu 24.04 LTS x86_64** from
[Latest Release](https://github.com/Marcus-P/trip-viewer/releases/latest).
If your release has no `.deb` asset, use its AppImage option instead.
Other distributions, cameras and operating systems are not newly certified by
this installation guide.

### First installation

1. Open [Latest Release](https://github.com/Marcus-P/trip-viewer/releases/latest)
   in your browser and expand **Assets**.
2. Download `Trip.Viewer_<version>_Ubuntu-24.04_amd64.deb`. The Source code ZIP
   and TAR files are developer source archives, not desktop installers.
3. Close Trip Viewer if it is running, double-click the downloaded `.deb`, and
   open it with Ubuntu App Center/Software Install. Choose **Install** and
   confirm Ubuntu's authorization prompt. The installer declares the required
   playback dependencies and includes the existing icon and license material.
4. Start **Trip Viewer** from Ubuntu's Applications overview. If desired,
   right-click its Dock icon and add it to Favorites.

No GitHub account or command line is needed to download a public release.
The package identity is `trip-viewer`; its application/desktop identity stays
`com.tripviewer.app`, matching the existing app data and preferences.

## Published-release checks and Ubuntu updates

The navigation bar checks GitHub's latest **published stable release** and shows
an update when it is newer than the installed version. In **Settings → Application
updates**, use **Check for updates** to refresh or **Open Releases** to visit GitHub.
For an available package, **Download Ubuntu installer** opens the download.

Close Trip Viewer, double-click the new `.deb`, and choose **Update/Install**.
The same package is upgraded in place. Your archive database, bookmarks, media
and user preferences are separate from the installed executable and are not
removed by this installer. Schema/parser versions are independent of app versions.

Windows and Mac use the same bounded checks with their own installer. Unknown
architectures or releases without a matching package fall back to the release
page; they are not sent to an Ubuntu installer.

The app checks release information only; it does not upload recordings or GPS.
Checks have a timeout, share a short cache and do not block playback when offline
or rate-limited. A build newer than the latest stable release is not offered a
downgrade. Downloads are restricted to the Marcus-P GitHub release and require
user confirmation; no unsigned executable is silently installed in the background.

This is not an APT repository subscription or unattended Ubuntu background update.
Main merges increase the app version and trigger a release, but a source commit
is offered to users only after the publication workflow succeeds. See
[versioning](VERSIONING.md) and [releasing](../RELEASING.md).

## Switching from an AppImage

Close the old AppImage before starting the installed package. Both builds use
the same app identity and archive format, so do not delete archive folders or
application settings as part of the switch.

An old per-user launcher can mask the newly installed system launcher. If
Applications or a Dock favorite still starts the AppImage:

1. In Files, open `~/.local/share/applications/` (Ctrl+L accepts a location).
2. Open `com.tripviewer.app.desktop` in a text editor. If its `Exec=` line points
   to an `.AppImage`, it is the old AppImage shortcut. Remove that shortcut file
   with Files; keep the archive and settings folders.
3. Start Trip Viewer again from Applications and pin that entry if desired.

The `.deb` owns its system launcher; do not manually remove system package files.
The standalone AppImage itself can be retained as a fallback, subject to database
compatibility. Older schema-16 builds cannot open schema-17 archives.

## AppImage alternative

The AppImage and its notice bundle remain available for existing users. Follow
[README command-line installation](../README.md#appimage-and-command-line-installation)
for the launcher helper. Replacing source files in a Git checkout does not update
an installed application.
