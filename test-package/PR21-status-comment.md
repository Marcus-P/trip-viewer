@chrisl8, I have updated this PR with the review corrections and the related VIOFO playback/GPS reliability fixes. The main archive schema remains **13**, and migration files are unchanged. The replacement description also groups the existing v0.13.0 additions kept outside this PR as options for later review, rather than an agreed roadmap.

On Ubuntu 24.04 with my A229 Pro 3CH recordings, I confirmed Original playback, Play/Pause, seeking and fullscreen transitions, visible GPS/merged-trip tracks and the moving marker. My **Front/8x timelapse also finished successfully**, and I watched its entire output with clean playback. This does not establish A129 or other-platform compatibility.

@alinnastac, I addressed the protected-clip `RO` handling you reported. Whenever you have time, a quick check of the updated PR with a couple of your A129 channel sets, including a protected clip in `RO`, would be helpful. Please check that the protected clip is recognized/highlighted, the channels stay aligned through Play/Pause, seeking and fullscreen changes, and the GPS track appears without obviously misplaced lines. No full-archive or timelapse run is needed; a short result or any remaining issue is enough, without sharing footage or location data.

[Windows/Ubuntu prerequisites and isolated build/test instructions](https://github.com/Marcus-P/trip-viewer/blob/contrib/viofo-camera-only/docs/VIOFO_TESTING.md) are available; no particular operating system is assumed for your setup.

Codex is assisting with implementation and preparation; the Ubuntu observations above are my hands-on results.
