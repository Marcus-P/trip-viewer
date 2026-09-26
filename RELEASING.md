# Releasing this fork

This fork currently publishes **Ubuntu 24.04 LTS x86_64** builds only. Windows support is intentionally deferred to a later feature branch.

## Release model

The file `RELEASE_VERSION` is the release trigger and must match:

- `package.json`
- the root package version in `package-lock.json`
- `src-tauri/Cargo.toml`
- the `tripviewer` package entry in `src-tauri/Cargo.lock`

When a commit that changes `RELEASE_VERSION` lands on `main`, `.github/workflows/release.yml` validates and tests the tree, builds the Ubuntu AppImage, generates checksums/license inventories, creates the `vX.Y.Z` tag, and creates a normal GitHub Release with permanent release assets.

## No upstream signing key or auto-updater

The upstream project uses a Tauri updater signing key owned by the upstream maintainer. This fork does not have and must not attempt to use that private key.

For this fork:

- `createUpdaterArtifacts` is disabled
- the upstream updater endpoint/public key is removed
- the updater UI/backend registration is disabled
- releases are verified by the SHA-256 checksum published next to the AppImage

This removes the previous local-build failure where Tauri produced an AppImage and then failed while trying to sign updater artifacts.

## Preparing a future release

Work in a feature branch. Before merging to `main`:

1. update the four version locations listed above
2. update `RELEASE_VERSION`
3. update `RELEASE_NOTES.md`
4. run `npm ci`, `npm test`, `npm audit --omit=dev --audit-level=high`, `cargo test --manifest-path src-tauri/Cargo.toml`, and `NO_STRIP=true npm run tauri -- build --bundles appimage`
5. validate the Ubuntu installer/launcher helper against the built AppImage
6. merge the tested branch to `main`

If `RELEASE_VERSION` did not change, no release is created.

## Ubuntu desktop integration

The release includes `install-trip-viewer-ubuntu.sh`. It extracts desktop metadata and the icon from the AppImage, installs the AppImage at a stable per-user path, installs the desktop entry under the Tauri application identifier (`com.tripviewer.app`), and refreshes the desktop database when available.

Tauri's `enableGTKAppId` setting is enabled so GNOME can associate the running application with that launcher and keep the Dock icon/group stable.

## Licensing

Before a release, review [LICENSE](LICENSE), [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md), and the generated Node/Rust dependency-license inventories. The repository MIT license covers Trip Viewer code and does not override dependency licenses.
