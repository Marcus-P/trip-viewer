# Releasing the Marcus-P fork

This fork of [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer) currently
publishes Ubuntu 24.04 LTS x86_64 AppImages and `.deb` installers. Existing Windows/macOS target code
is retained; these are not newly validated/released platforms for this fork.
Keep the original icons, [MIT notice](LICENSE) and
[third-party notices](THIRD_PARTY_NOTICES.md).

## Stable versus candidate

Use [Latest Release](https://github.com/Marcus-P/trip-viewer/releases/latest)
as the authority for the published stable version. Version 0.6.0 was published
after PR #4 merged and its release workflow succeeded. Owner-confirmed manual
checks cover VIOFO A229 Pro 3CH / Ubuntu 24.04 LTS. Use
[compatibility evidence](docs/COMPATIBILITY.md) to describe other cameras; do not
promote parser tests or synthetic media into hardware certification.

A feature branch/test artifact is not a stable release. Merge, successful release
workflow, uploaded assets and installed local version are separate events.
Only advertise a new stable download after its release workflow succeeds.
Use the Latest Release link for current downloads and dynamically resolve the
release tag in installation commands, as in the README. Describe completed
releases in the past tense; do not leave an old PR's pending instructions as the
current project state. A checkout's package version may be newer than the latest
published download while its PR or release workflow is pending.

## Prepare the PR

Follow [versioning](docs/VERSIONING.md). Synchronize `package.json`, both root
version entries in `package-lock.json`, the app entries in `src-tauri/Cargo.toml`
and `src-tauri/Cargo.lock`, and `RELEASE_VERSION`. Update release notes and docs
so a candidate is not mislabeled as already published.

Run checks relevant to the diff; the required `ubuntu` job selects them automatically.

| Changed area | Relevant PR checks |
| --- | --- |
| Documentation plus version-only metadata | Documentation links/shell examples, synchronized version increment and mandatory release notes |
| Frontend | Frontend tests and typecheck/build; native package build for playback/desktop changes |
| Backend | Rust tests and frontend/backend command registry; package build |
| Ubuntu packaging/installer | Build and validate AppImage and `.deb`, including dependencies, desktop identity and notices |
| CI policy/scripts | Policy tests; workflow changes exercise the full pipeline once |
| Runtime Node dependencies | Runtime dependency audit |

Documentation-only PRs do not install native build dependencies, run frontend/Rust
suites, build test packages or upload artifacts. They still increase PATCH under
the owner's every-merge rule and update release notes. Their later merged-main
release must build packages once to embed that new version; it does not repeat
irrelevant unit tests. Manual dispatch deliberately selects the full pipeline.

For a deliberate local Ubuntu package build:

```bash
npm ci
node scripts/check-version.mjs
python3 scripts/prepare-release-licenses.py
NO_STRIP=true npm run tauri -- build --bundles appimage,deb
DEB="$(find src-tauri/target/release/bundle/deb -maxdepth 1 -type f -name '*.deb' -print -quit)"
python3 scripts/prepare-ubuntu-deb.py "$DEB"
python3 scripts/validate-ubuntu-deb.py "$DEB"
```

CI runs once per PR update and cancels superseded work. Packaging changes produce
one downloadable `.deb` test installer with checksums/build information, retained
for seven days. Do not upload a second AppImage copy for the same GUI installer
test. The stable AppImage remains supported and its helper is verified separately.

## Protected merge and release workflow

`main` requires a PR, resolved review threads, a current successful `ubuntu` check,
and owner-only updates via PR. History rewrite and branch deletion are blocked
without a bypass actor. **Only the repository owner merges.** Do not substitute
direct pushes, force pushes or `--admin` bypasses. Details and verification are in
[branch protection](docs/BRANCH_PROTECTION.md).

When a merged commit changes `RELEASE_VERSION`, `.github/workflows/release.yml`
validates the version/notes and builds the merged AppImage and Debian installer.
Relevant tests/audits already ran before the protected PR merge; they are not
repeated here. It checks package contents and launcher integration, includes
license inventories/texts, creates checksums and publishes `vX.Y.Z`. Relative
release-note links are resolved against the published commit. Existing tags are rejected rather than overwritten.

Stable filenames are `Trip.Viewer_X.Y.Z_Ubuntu-24.04_amd64.AppImage` and
`Trip.Viewer_X.Y.Z_Ubuntu-24.04_amd64.deb`.
CI test artifacts instead include a commit ID and expire after their retention
period. Published release assets persist until deliberately removed.

## Updater, desktop identity and licensing

The upstream private updater key is not available to this fork and must not be
used. The old native signed updater plugin/artifacts/endpoint are disabled and
the unused plugin dependency/permission are removed. The replacement release
hint reads the public fork release API and opens the Ubuntu package download
for a user-confirmed install. Release SHA-256 manifests provide integrity checks
for downloaded assets. This is distinct from signed unattended updater trust
and depends on the GitHub release distribution channel.

Both installers use `com.tripviewer.app`, the existing icon and GTK app-ID support.
The Debian launcher is normalized to that desktop identity, with its package
checksums regenerated. Package upgrades touch installed files, not user archives.
Do not change the identifier as release cleanup: it affects launcher grouping and
application compatibility. Installation paths and source-build updates are in
[Linux setup](LINUX_DEV_SETUP.md).

Release license inventories include resolved Node/Rust declarations and extracted
runtime notice/license texts. Third-party components retain their own terms,
including React Leaflet’s Hippocratic-2.1 and Symphonia’s MPL-2.0 declarations.
Trip Viewer’s MIT license does not relicense them. Preserve the inherited icon
and copyright attribution even when refreshing product documentation.
