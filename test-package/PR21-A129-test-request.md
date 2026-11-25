@alinnastac, I have updated my PR #21 with the review corrections, protected `RO` handling and the related playback/GPS reliability work. The main archive schema stays at 13. The additional features from my full fork remain separate.

If you have time, even a quick check with a couple of complete A129 camera sets would help. You can build this PR directly, without installing my full fork. The guide contains the native prerequisites, exact Windows and Ubuntu commands, an isolated test application identity and a small playback check:

https://github.com/Marcus-P/trip-viewer/blob/contrib/viofo-camera-only/docs/VIOFO_TESTING.md

The useful checks are Original at x1, Play/Pause and seeking, entering/exiting fullscreen, and whether the camera views remain aligned. If your samples include protected clips, their `RO` event marking would also be useful to confirm. GPS should produce a visible track or a clear empty/error result rather than stay on “Loading GPS”. No parking recordings or timelapse encoding are needed.

A short result with the commit, OS and what worked or failed is enough. Please keep your footage and location data private; there is no need to upload them. My A229 GPS test is confirmed, but I am not treating it as an A129 playback result or as an explanation of the earlier 15-point diagnostic difference. Codex is assisting with preparation; hardware observations remain separate evidence.
