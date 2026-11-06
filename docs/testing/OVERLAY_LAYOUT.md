# Overlay layout acceptance

This source candidate is on `feat/overlay-camera-layout`, based on stable 0.8.2.
Its provisional version is 0.9.0; it has not been merged or published as stable.
The current stable app does not update merely because the cloud branch changed.

## Acceptance status and reset semantics

The owner confirmed the position controls and mirror clipping, both paused and
playing. Those checks are accepted; the sequence below remains a reference for
future regressions, not a request to repeat it before this PR.

The final Reset crop follow-up restores 100% zoom and centers both position
sliders. The frame and other cameras stay unchanged. Save retains that reset;
Cancel restores the saved crop. Reset overlay restores all layer defaults,
including all crop offsets, as a draft until saved.

## Ubuntu: run directly from the branch

Close the installed Trip Viewer first. From the existing development checkout
with the previously installed Tauri/Rust/Linux build dependencies:

```bash
cd ~/trip-viewer
git fetch origin
git switch feat/overlay-camera-layout
git pull --ff-only origin feat/overlay-camera-layout
npm ci
CARGO_BUILD_JOBS=2 npm run tauri -- dev
```

Git switches to the remote branch automatically if it does not yet exist locally
and `origin/feat/overlay-camera-layout` is available. If the clone fetches only
`main`, fetch the test ref explicitly before switching:

```bash
git fetch origin refs/heads/feat/overlay-camera-layout:refs/remotes/origin/feat/overlay-camera-layout
```

`CARGO_BUILD_JOBS=2` limits concurrent Rust compiler jobs; it does not impose a
memory, CPU-time or billing limit. Running directly avoids packaging an installer
for the initial test. The normal installed app remains available. Layout choices
use the same application storage, so testing may change the saved display mode;
switch back to Classic before returning to the stable app if desired.

## Small real-footage check

1. Open a VIOFO A229 Pro 3CH trip and start playback. Switch **Camera layout**
   between **Classic** and **Overlay**. The channels should keep playing together.
2. Choose **Edit overlay**. With **Move frame**, move/resize Rear. With
   **Adjust video crop**, select Rear, increase **Crop zoom** and drag the video
   inside the mirror. Only Rear's crop should change; the frame and Front stay
   still. Curved mirror corners should show Front, not a black rectangle.
   Alternatively, move **Vertical crop** toward **Bottom / road** to bring the
   road into view. Zooming or using a crop-position slider automatically enables
   video-crop dragging. Changing zoom should retain the selected lower region.
3. Move Interior to the desired corner. Save. Test Pause/Play, a segment boundary,
   single-camera and dashboard fullscreen, and switching back to Classic.
4. Close the test app, rerun the final command above and check that Overlay,
   positions and crop were restored. Verify Cancel discards an unsaved edit.

Report the OS and any clipping, frozen frame, lost crop or unexpected source
reload. This check is targeted to the new display mode; it does not repeat the
already accepted GPS/bookmark/archive workflows.

## Return to stable source

Close the test app, then:

```bash
git switch main
git pull --ff-only origin main
```

Start the normally installed application as before. No original media or archive
database migration is part of this feature.
