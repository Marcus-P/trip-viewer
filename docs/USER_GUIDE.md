# User guide

This guide covers playback controls, installation and updates.
Download the latest published stable build from
[Latest Release](https://github.com/Marcus-P/trip-viewer/releases/latest).
The fork’s hands-on camera test scope is [VIOFO A229 Pro 3CH on Ubuntu 24.04 LTS
and Windows 11 x64](COMPATIBILITY.md). Windows 10 HEVC playback after buying the
Microsoft extension remains untested.

## Install and update

The [installation guide](INSTALLATION.md) explains Windows installers, experimental Mac disk images, the Ubuntu `.deb` installer,
AppImage migration and upgrades without a terminal. The navigation bar shows
new published releases; **Settings → Application updates** can check again and
open the download. Offline checks do not interrupt playback. A test build newer
than the current stable release is not offered a downgrade. The app opens the
installer download; Ubuntu asks you to confirm installation after you close the app.

## Open an archive and select a trip

Use the archive switcher and choose a footage/archive folder. Filename recognition
and metadata probing group channels into segments and segments into trips.
Choosing a trip starts GPS preparation in the background. The progress counter
advances per segment; playback can continue ahead of analysed GPS. Returning to
a trip or restarting the app reuses completed checkpoints and resumes unfinished
work. Footage without usable GPS can still play.

The **Scan** tab performs tag/analysis scans, a separate operation from opening a
footage folder. “New segments only” can complete immediately when nothing is new.

## Video views

- **Settings → Display layout** groups the Front, Interior, Rear and map
  checkboxes. Additional camera labels found in the archive are listed too.
  Select the views you want, then choose **Save**; **Cancel** discards changes.
  These settings persist across restarts, including the existing camera choices.
  They save a display preference, not a recording or deletion setting. Visible
  cameras reclaim hidden tiles; a single camera fills the video grid. If every
  camera is hidden, the map fills the viewing area when available. Re-enable a
  checkbox to restore video. Camera hiding does not select a new audio/sync master.
- Click a secondary camera to make it visually primary. Double-click the main
  camera for single-camera fullscreen.
- **F/I/R + GPS fullscreen** opens the dashboard. Its timeline/speed graph can be
  collapsed/expanded. Play/Pause is in the lower bar; single-camera fullscreen
  retains its own upper transport. Back/exit navigation restores the prior view.
- While paused, use the wheel or `+`/`−` to zoom, then drag to pan. The percentage
  control resets zoom. Playback resume or source change restores the normal view.

## Seeking and bookmarks

Click or drag the timeline; dragging previews locally and seeks on release.
Click the recorded blue map route to seek to its GPS time. Projection between
valid samples follows the displayed route; missing fixes are not bridged.
Repeated locations prefer the traversal closest to the current playhead.

Right-click the graph/timeline to add an amber numbered bookmark. `+ Bookmark`
saves the current position. Left-click a marker to seek, right-click the marker
to remove it. Arrow-and-bar buttons jump to the previous/next mark without
wrapping. Bookmarks use trip time, work without GPS, survive trip changes and
app restart, and are rebased when trips are merged. Selection mode disables
bookmark editing/navigation so segment selection remains unambiguous.

| Key | Action |
| --- | --- |
| Space | Play/Pause |
| Left / Right | Seek backward/forward 5 seconds |
| Shift + Left / Right | Seek backward/forward 30 seconds |
| `[` / `]` | Decrease/increase playback speed (0.5×, 1×, 2×, 4×) |
| D | Toggle drift display |

Arrow seeks use the existing engine path; use the trip timeline, bookmarks or
route for a specific position across segment boundaries. There is no current
`M` shortcut to enable secondary cameras; use their checkboxes.

## Audio, GPS and media errors

On Linux, audio is enabled only in Original / 1× mode; other speeds/timelapse
are muted to avoid validated WebKitGTK/GStreamer audio corruption. A channel-local
video failure leaves healthy cameras running. Recovered channels rejoin without
changing the promoted clock mid-segment. Persistent health records are checked
against the file fingerprint so repaired/replaced media can be re-evaluated.
A segment transition may still briefly initialize the next media pipeline.

A missing GPS log is not evidence of a broken video. Thinkware GPS is not decoded
by the existing implementation; see the compatibility table before reporting it
as a regression. GPS extraction performance depends on footage and storage;
the few seconds estimated improvement on the test SSD was not a controlled benchmark.

## Import, review and timelapse

Folder import preserves the source. The SD-card import workflow includes a
source-wipe phase after verification; review its confirmation and source selection
before using it. Hashing/verification and cancellation safeguards originate in
the upstream import pipeline. Opening/scanning a folder is separate from importing it.

In the trip sidebar, Ctrl/Cmd-click toggles selection and Shift-click adds the
visible range from the anchor. The selection toolbar offers Merge, Delete
originals and Delete trips with explicit scope/confirmation. Cancel leaves files
unchanged. A merge is blocked if it would discard an archive-only trip's only
timelapse copy; redundant merge outputs are cleaned only within that merge's scope.

Review and Scan provide tags, filters and segment selection. Places stores named
positions and radii for matching during analysis. Deleting original footage can
retain completed timelapses; **Delete trip** has a broader scope, so inspect its
confirmation. Deletions use the existing OS-trash workflow where supported.

Timelapse is optional and needs a user-installed ffmpeg configured in the app.
Choose completed 8×/16×/60× tiers through the source picker. Source/tier speed and
player speed combine; the bar displays the effective rate. Archive-only trips
can play their retained timelapses and GPS without original files.

## Where preferences and bookmarks live

Archive data is stored under `<archive>/.tripviewer/tripviewer.db`, including GPS,
health information, tags and bookmarks. Keep that metadata when copying/backing
up an archive. Machine settings use the Tauri app-data directory; layout/camera
preferences use webview local storage. Clearing application storage can reset
those display preferences but does not remove bookmarks from the archive DB.
Schema 17 archives cannot be opened by an older schema-16-only build.
