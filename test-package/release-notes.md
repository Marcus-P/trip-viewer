Isolated Ubuntu 24.04 x86_64 acceptance package for the PR21 reliability follow-up.

**Manual prerelease only. Not an update for stable v0.13.0, not Latest, no updater manifest or signatures.**

This revision reuses the already implemented full-fork solutions for Linux media preparation, stable playback intent, canonical audio, channel-local failure/recovery and persistent segment-level GPS preparation. Optional overlay, zoom, layout and bookmark features are excluded. The existing three-second filename association remains heuristic.

The previous isolated package produced owner-reported severe Front-video stutter, no audible sound, intermittent Play/Pause/synchronization problems and very slow GPS loading. CPU usage reached 100% during playback and dropped after pausing. These are observed symptoms, not proof of a particular decoder or CPU cause. This corrected package still needs owner hardware acceptance.

**Upstream archive schema remains 13.** Existing migration files and the main trip_gps table are unchanged. Disposable GPS checkpoints, provenance and media-health state live separately in `.tripviewer/analysis-cache-v1.sqlite`. Tests verify cache writes do not change the main database schema/version and that archived legacy GPS remains available. Completed segments are published progressively, including timeline speed data, instead of waiting for the whole trip; merged/reordered trips invalidate the completed-track cache.

Close the previous PR21 test before launching. Run `start-test.sh` from the extracted package; it reuses the existing scratch archive `~/Downloads/TripViewer-PR21-Testarchiv-5dbdaf9` and uses fresh isolated app settings. Do not open a production archive. Start with Original at x1, where audio is enabled; other playback rates and timelapse modes intentionally remain muted. No parking footage or timelapse encode is required.

Focused playback/audio/recovery, progressive GPS, cache and archive-schema tests pass locally. Publication requires the Ubuntu24.04 AppImage build and isolated packaged startup/schema check. No universal camera/hardware certification is claimed.

The Linux media preparation commit retains @gubiq's authorship. Earlier A129 contributions from @alinnastac remain acknowledged in the synthetic regression fixture; contributor and upstream license notices remain intact. Codex assisted with the targeted backport and packaging from @Marcus-P's requirements. Neither upstream PR21 nor stable main/develop are updated by this test publication.
