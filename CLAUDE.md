# Contributor entry point

Start with [AGENTS.md](AGENTS.md) and [current project state](docs/PROJECT_STATE.md).
Those instructions apply to this fork regardless of the coding assistant used.
The [inherited Claude guidance](docs/history/inherited-CLAUDE.md) is preserved for
provenance; its Windows-first, legacy settings-table and updater assumptions
must not override the current code or fork release policy.

## Commands

```bash
npm ci
npm test
npm run build
cargo test --locked --manifest-path src-tauri/Cargo.toml
npm run tauri -- dev
NO_STRIP=true npm run tauri -- build --bundles appimage
```

Native development needs the [Linux dependencies](LINUX_DEV_SETUP.md).
Use `cargo clippy` for relevant Rust work, inspect existing warnings, and do not
claim the whole repository is warning-free without checking.

## Preserve these invariants

- Keep channel video elements mounted and in canonical DOM order; change layout
  with CSS. Visual primary and display visibility are separate from sync/audio.
- Keep validated Linux workarounds and channel-local recovery. Do not turn one
  corrupt camera or a missing GPS log into a whole-trip playback failure.
- Preserve camera dispatch; see [compatibility evidence](docs/COMPATIBILITY.md).
  Do not infer hardware validation from synthetic samples or parser unit tests.
- Preserve upstream import verify-before-wipe: source wipe only after complete
  verified staging and not when cancelled/read-only. Hash on copy, verify at the
  destination, retain PID-lock recovery, skip preallocation files and respect
  the Videos/Photos import-root adjustment.
- Keep archive writes attached to the intended handle; avoid long DB locks while
  parsing media. Bookmarks and GPS caches must survive ordinary trip/restart use.
- Preserve contributor authorship, the upstream icon and MIT/dependency notices.
  No force pushes, admin bypasses or assistant merges into owner-protected main.

See [architecture](DESIGN.md), [user guide](docs/USER_GUIDE.md),
[release policy](RELEASING.md) and [versioning](docs/VERSIONING.md) for details.
