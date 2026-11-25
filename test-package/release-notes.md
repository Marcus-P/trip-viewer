Isolated Ubuntu 24.04 x86_64 playback candidate for PR21.

**Manual prerelease only. Not Latest, no stable updater metadata, separate settings/app identity. Stable v0.13.0 is unchanged.**

Double-click fullscreen now enlarges the shared viewer, including transport. Camera elements remain mounted and transitions use browser completion events followed by the existing SyncEngine re-anchor. Native video controls cannot independently pause a camera or bypass the shared viewer. Optional overlays, layout editors, settings and bookmarks remain excluded.

The preceding published package lacked libgstopengl.so, matching WebKit's missing-GL-sink warning. This package explicitly includes the GL component; packaging verifies the bundled glupload, glcolorconvert, gldownload and appsink factories before publication. GPU/codec/driver compatibility and real 4K smoothness still require hands-on validation.

The main archive schema remains13, with no new main-database migrations. Persistent analysis stays in the separate sidecar. The previously owner-confirmed GPS reader/version9 is retained. This package reuses the existing scratch archive and manual trip merges; no cache deletion, footage copy or timelapse encode is required.

Close the previous PR21 test, extract this package, run `TRIPVIEWER_GPS_TIMING=1 bash ./start-test.sh`, then follow the small playback test in README-DE.txt. The source includes docs/VIOFO_TESTING.md with isolated Windows and Ubuntu build instructions; no OS is inferred for another contributor. This build's physical playback acceptance remains pending.

Contributor authorship and licensing remain intact, including @gubiq's Linux media change and the credited synthetic A129 layout regression adapted from @alinnastac. Codex assists with implementation; owner and contributor hardware results remain separate evidence. Upstream PR scope and merge decisions remain with @chrisl8.

Existing timelapse generation also receives a runtime NVENC check with CPU fallback. Batch start rejects old cached false positives rather than selecting an unusable NVIDIA encoder for all camera jobs. The API/settings fields and main schema13 stay unchanged. A synthetic two-segment HEVC-to-8x encode and finished-output decode passed locally; real owner batches and physical NVIDIA support remain separate validation. Full-fork AMD/Intel/VA-API extensions and calibration are not included.
