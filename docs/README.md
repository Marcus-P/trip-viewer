# Documentation

This documentation describes the Marcus-P fork of [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer).
The existing logos/icons and source/dependency license notices are retained.

| Document | Purpose |
| --- | --- |
| [Project README](../README.md) | Origin, current release, download and installation |
| [Installation and updates](INSTALLATION.md) | Windows/Mac/Ubuntu installers, release checks and AppImage migration |
| [Consistency review](CONSISTENCY_REVIEW.md) | Version, docs, IPC, schema, packaging and CI findings |
| [User guide](USER_GUIDE.md) | Player, map, timeline, bookmarks, camera visibility and library workflows |
| [VIOFO filenames](VIOFO_FILENAMES.md) | Mixed filename formats, automatic recognition and firmware-evidence limits |
| [Compatibility](COMPATIBILITY.md) | Camera/platform evidence and limits of functional testing |
| [Linux setup](../LINUX_DEV_SETUP.md) | Ubuntu build/runtime requirements and other-distro references |
| [Architecture](../DESIGN.md) | Current components, clocks, storage and inherited decisions |
| [Timelapse alignment](../TIMELAPSE_ALIGNMENT_REDESIGN.md) | Actual curve format versus the historical proposal |
| [0.6.0 acceptance record](FINAL_060_CHECK.md) | Completed integration test, owner merge, release and cleanup |
| [Releasing](../RELEASING.md) | Builds, assets, license inventories and owner-only merges |
| [Versioning](VERSIONING.md) | Version increments and synchronized version files |
| [Branch protection](BRANCH_PROTECTION.md) | Active main rules, owner bypass scope and verification |
| [Branch workflow](BRANCH_WORKFLOW.md) | Permanent develop integration, stable promotion and audited branches |
| [VIOFO filename acceptance](testing/VIOFO_LEGACY_0103.md) | Focused counter-free/mixed-format archive test |
| [GPU compatibility and reports](GPU_COMPATIBILITY.md) | Tested GPU setups, evidence limits and community test channel |
| [AMD/Intel/Linux GPU acceptance](testing/AMD_TIMELAPSE_0110.md) | Focused AMF/QSV/VA-API setup, encoding and fallback test |
| [Timelapse acceptance](testing/TIMELAPSE_0102.md) | Focused VIOFO/encoder checks for the 0.10.2 candidate |
| [Release notes](../RELEASE_NOTES.md) | Changes and compatibility implications; published assets are on Latest Release |
| [Project state](PROJECT_STATE.md) | Current hand-off, accepted tests and remaining work |
| [Agent instructions](../AGENTS.md) / [Claude entry point](../CLAUDE.md) | Binding agent entry point, external collaboration/communication policy and contributor working rules |
| [Third-party notices](../THIRD_PARTY_NOTICES.md) / [MIT license](../LICENSE) | Source, dependency and asset notices |

## Historical material

The inherited [design](history/inherited-DESIGN.md),
[Linux guide](history/inherited-LINUX_DEV_SETUP.md),
[Claude guidance](history/inherited-CLAUDE.md) and
[timelapse proposal](history/inherited-TIMELAPSE_ALIGNMENT_REDESIGN.md) are retained
with explicit historical labels. They contain upstream-era observations and
plans, not new validation by this fork. The
[prior fork development record](history/PROJECT_STATE-before-doc-refresh.md)
preserves the debugging and acceptance history; current state supersedes its
old pending-test or permission-blocked entries.
