# Releasing the Marcus-P fork

This fork of [chrisl8/trip-viewer](https://github.com/chrisl8/trip-viewer) currently
publishes Ubuntu 24.04 LTS x86_64 AppImages. Existing Windows/macOS target code
is retained; these are not newly validated/released platforms for this fork.
Keep the original icons, [MIT notice](LICENSE) and
[third-party notices](THIRD_PARTY_NOTICES.md).

## Stable versus candidate

Published stable: **0.5.0**. The open 0.6.0 candidate is **PR #4**. Owner-confirmed
manual checks cover VIOFO A229 Pro 3CH / Ubuntu 24.04 LTS. Use
[compatibility evidence](docs/COMPATIBILITY.md) to describe other cameras; do not
promote parser tests or synthetic media into hardware certification.

A feature branch/test artifact is not a stable release. Merge, successful release
workflow, uploaded assets and installed local version are separate events.
Only advertise a new stable download after its release workflow succeeds.

## Prepare the PR

Follow [versioning](docs/VERSIONING.md). Synchronize `package.json`, both root
version entries in `package-lock.json`, the app entries in `src-tauri/Cargo.toml`
and `src-tauri/Cargo.lock`, and `RELEASE_VERSION`. Update release notes and docs
so a candidate is not mislabeled as already published.

```bash
npm ci
node scripts/check-version.mjs
npm test
npm run build
npm audit --omit=dev --audit-level=high
cargo test --locked --manifest-path src-tauri/Cargo.toml
NO_STRIP=true npm run tauri -- build --bundles appimage
```

CI additionally compares the version to the PR’s base. Validate the Ubuntu desktop
integration helper against the built AppImage using an isolated test home, as in
`.github/workflows/ci.yml`. Record relevant real-footage checks and their actual
hardware/OS scope, retaining authorship and a merge commit for contributions.

## Protected merge and release workflow

`main` requires a PR, resolved review threads, a current successful `ubuntu` check,
and owner-only updates via PR. History rewrite and branch deletion are blocked
without a bypass actor. **Only the repository owner merges.** Do not substitute
direct pushes, force pushes or `--admin` bypasses. Details and verification are in
[branch protection](docs/BRANCH_PROTECTION.md).

When a merged commit changes `RELEASE_VERSION`, `.github/workflows/release.yml`
validates the tree, tests/audits it, builds the AppImage and installer, checks the
launcher, generates license inventories/checksums, creates `vX.Y.Z`, and publishes
a GitHub Release. Existing tags are rejected rather than overwritten.

The stable AppImage name is `Trip.Viewer_X.Y.Z_Ubuntu-24.04_amd64.AppImage`.
CI test artifacts instead include a commit ID and expire after their retention
period. Published release assets persist until deliberately removed.

## Updater, desktop identity and licensing

The upstream private updater key is not available to this fork and must not be
used. Updater artifacts/UI/endpoint are disabled; release SHA-256 manifests provide
integrity checks for manually downloaded assets. This is distinct from upstream
signed-updater trust and depends on the release distribution channel.

The installer uses `com.tripviewer.app`, the existing icon and GTK app-ID support.
Do not change the identifier as release cleanup: it affects launcher grouping and
application compatibility. Installation paths and source-build updates are in
[Linux setup](LINUX_DEV_SETUP.md).

Release license inventories include resolved Node/Rust declarations and extracted
runtime notice/license texts. Third-party components retain their own terms,
including React Leaflet’s Hippocratic-2.1 and Symphonia’s MPL-2.0 declarations.
Trip Viewer’s MIT license does not relicense them. Preserve the inherited icon
and copyright attribution even when refreshing product documentation.
