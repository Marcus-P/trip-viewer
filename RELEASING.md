# Releasing the Marcus-P fork

This fork of [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer) currently
builds Ubuntu 24.04 LTS x86_64 AppImages/`.deb` installers, Windows 10/11 x64
NSIS installers and experimental universal macOS 11+ disk images. Native package
checks and real-recording acceptance are separate; Mac functional testing is
unconfirmed. Check published release assets rather than assuming an open PR is
already available as a stable download.
Keep the original icons, [MIT notice](LICENSE) and
[third-party notices](THIRD_PARTY_NOTICES.md).

## Stable versus candidate

Use [Latest Release](https://github.com/Marcus-P/trip-viewer/releases/latest)
as the authority for the published stable version. Owner-confirmed manual
checks cover VIOFO A229 Pro 3CH on Ubuntu 24.04 LTS and Windows 11 x64. Use
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
Keep current download/install instructions version-neutral. Release history and
merge records belong in [release notes](RELEASE_NOTES.md) and the
[project record](docs/PROJECT_STATE.md), not in current download instructions.

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
one small acceptance installer per relevant platform with checksums/build information,
retained for seven days. Do not upload a second AppImage copy for the same GUI installer
test. The stable AppImage remains supported and its helper is verified separately.
Native Rust tests run only when backend checks are selected; Windows/Mac package
jobs reuse the same workflow in PR CI and publishing. The required `ubuntu` job
is the final gate over scope checks and all relevant native/Linux jobs, so a
failed Windows/Mac job cannot be hidden by a successful Linux check.

Treat Actions storage as a limited resource. Upload only a package required for
an active acceptance test, with retention of seven days or less; do not upload
build/dependency directories or duplicate packages. Do not add caches without a
measured need and a size/expiry plan. After acceptance or verified publication,
remove obsolete test artifacts and caches when authorized. Keep published release
downloads/license material and useful CI run history; release assets are separate
from temporary Actions storage. Prefer existing retention and cleanup at release
handoff over an extra scheduled workflow. Limit cleanup to this repository unless
the owner explicitly extends its scope.

## Protected merge and release workflow

`main` requires a PR, resolved review threads, a current successful `ubuntu` check,
and owner-only updates via PR. History rewrite and branch deletion are blocked
without a bypass actor. **Only the repository owner merges.** Do not substitute
direct pushes, force pushes or `--admin` bypasses. Details and verification are in
[branch protection](docs/BRANCH_PROTECTION.md).

Every push to `main` starts `.github/workflows/release.yml` without a path
filter. It validates the version/notes and selects an immutable tag before
building the pushed head once for **all three platforms**. A push containing
several commits builds its final head, not separate installers for intermediate
commits. Normal owner merges still increment the app version once per PR.
Windows/Mac packages are built natively and validated before the Ubuntu publishing job. Only their small
installers use one-day transfer artifacts; large Ubuntu AppImage/bundle files
stay in the publishing job. Transfer checksums are verified, and successful
publication removes those temporary artifacts.
Relevant tests/audits already ran before the protected PR merge; they are not
repeated here. It checks package contents and launcher integration, includes
license inventories/texts and creates checksums. Version-changing merges publish
stable `vX.Y.Z` downloads. An additional main commit with an unchanged version
publishes `main-<full commit SHA>` as a prerelease, with the same app version in
its installer but an exact source commit in its release notes/build information.
These snapshots never become Latest Release or stable in-app update suggestions.
Their downloads persist in Releases; installation is an explicit choice.
Relative release-note links are resolved against the published commit. Existing
tags are rejected in the lightweight planning job rather than overwritten;
rerunning an already published commit does not rebuild or replace its downloads.

Stable filenames are `Trip.Viewer_X.Y.Z_Ubuntu-24.04_amd64.AppImage`,
`Trip.Viewer_X.Y.Z_Ubuntu-24.04_amd64.deb`,
`Trip.Viewer_X.Y.Z_Windows_x64_setup.exe` and
`Trip.Viewer_X.Y.Z_macOS_universal_experimental.dmg`. No duplicate Windows MSI
or separate Intel/Apple Silicon installers are needed. The release checksum
manifest covers all installers and the existing Ubuntu bundle.
CI test artifacts instead include a commit ID and expire after their retention
period. Published release assets persist until deliberately removed.

## Updater, desktop identity and licensing

The upstream private updater key is not available to this fork and must not be
used. The old native signed updater plugin/artifacts/endpoint are disabled and
the unused plugin dependency/permission are removed. The replacement release
hint reads the public fork release API and selects the exact matching installer
from the compiled target, for a user-confirmed install. Unknown architectures
fall back to the release page rather than a foreign-platform binary. Release SHA-256 manifests provide integrity checks
for downloaded assets. This is distinct from signed unattended updater trust
and depends on the GitHub release distribution channel.

All platform packages keep `com.tripviewer.app`, the existing icon and GTK app-ID support.
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

## Windows and Mac signing / functional acceptance

Windows uses the current-user NSIS installer with WebView2 bootstrap setup and
no publisher certificate. Mac uses an ad-hoc signed universal app, minimum
macOS 11, without Developer ID/notarization. No upstream key or new signing
secret is assumed. Signer enrollment/paid certificates are separate optional
work. Keep unknown-publisher/first-launch instructions in the installation guide;
do not imply a successful build is notarization or real-video certification.
Hands-on statements follow actual owner feedback: VIOFO A229 Pro 3CH on Ubuntu
24.04 LTS and a successful Windows 11 x64 report without an additional codec
installation. Keep Windows 10 playback after buying the HEVC extension explicitly
untested. Mac remains visibly experimental until a Mac tester confirms functionality.
Do not repeat accepted Windows playback tests for documentation-only changes.
