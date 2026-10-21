# Install and update on Ubuntu without a terminal

The GUI installer is introduced by 0.7.0 for **Ubuntu 24.04 LTS x86_64**. It is
available once that release workflow has succeeded. If the latest release has
no `.deb` asset, use its AppImage option instead. Other distributions, cameras
and operating systems are not newly certified by this installation guide.

## First installation

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

## Updates

The navigation bar checks GitHub's latest **published stable release** and shows
an update when it is newer than the installed version. In **Settings → Application
updates**, use **Check for updates** to refresh or **Open Releases** to visit GitHub.
For an available package, **Download Ubuntu installer** opens the download.

Close Trip Viewer, double-click the new `.deb`, and choose **Update/Install**.
The same package is upgraded in place. Your archive database, bookmarks, media
and user preferences are separate from the installed executable and are not
removed by this installer. Schema/parser versions are independent of app versions.

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
