Isolated Ubuntu 24.04 x86_64 acceptance build for upstream VIOFO PR #21 with follow-up fixes, source `5dbdaf9c998df63385c979e7cb72e38bdbeb8d36`.

**Manual test only. This is not a v0.13.0 update.**

- Marked as a prerelease and explicitly excluded from Latest. No updater manifest or update signature is published.
- Separate application identifier `com.tripviewer.pr21test`, isolated settings and a dedicated scratch archive. Run `start-test.sh`; do not install over the stable app or open a production archive.
- Upstream-based test native version: `0.3.4-rc.20261125`. The unchanged frontend may display `0.3.3`. This is not a stable fork version.
- Includes the local GPS review fixes and preservation of VIOFO RO/protected-recording context through import and grouping. The existing three-second timestamp association remains heuristic; GPS-based association is not implemented in this build.
- No parking footage is needed for the requested owner test. No timelapse encoding is required initially.

Download the `.tar.gz`, extract it, run `sha256sum -c SHA256SUMS.txt`, then follow `README-DE.txt`. Use copied video files only.

Built on the Ubuntu 24.04 GitHub runner. Publication requires packaged startup with isolated settings/database to pass on that runner. Playback of this exact package with owner footage is pending. This is not universal hardware certification.

Upstream authorship and license notices are retained. The A129 regression fixture acknowledges the earlier work of @alinnastac and uses synthetic telemetry. Codex prepared the follow-up fixes and test packaging from @Marcus-P's requirements; hands-on acceptance is separate.

The release tag contains a standalone source snapshot with the same tree as the build source and safe owner commit metadata, without copying the older PR ancestry into this release tag. Existing contributor notices remain intact. Neither PR #21's branch nor the fork's main/develop branches are updated by this publication.
