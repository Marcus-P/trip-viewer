# Versioning and merging

The stable branch is `main`. Every pull request merged into `main` must increase
its version once relative to the current target branch. A PR uses the highest
applicable change category:

| Change | Increment | Example |
| --- | --- | --- |
| Bug fix, performance improvement with unchanged behaviour, maintenance | PATCH | 0.5.0 → 0.5.1 |
| New backwards-compatible capability | MINOR; reset PATCH | 0.5.1 → 0.6.0 |
| Intentional incompatible change to supported behaviour or compatibility | MAJOR; reset MINOR and PATCH | 0.6.2 → 1.0.0 |

These rules also apply while MAJOR is zero. Version numbers identify merged
application changes; the GPS parser version and database schema version have
separate purposes and must not be bumped merely to match the app version.

Choose and commit the new version before merging, so CI tests and packages the
exact version users will receive. Further commits to the same PR keep that
version; they do not increment it repeatedly. If another PR merges first,
reconcile the version with the updated `main` before merging. Do not merge PRs
concurrently against the same release version. Keep a merge commit to preserve
contributor authorship. Require green CI and relevant manual validation.

Keep these files synchronized:

- `package.json` (Tauri reads this through `tauri.conf.json`)
- both root version entries in `package-lock.json`
- the application package in `src-tauri/Cargo.toml` and `src-tauri/Cargo.lock`
- `RELEASE_VERSION`

Update `RELEASE_NOTES.md` to describe the resulting release. Run
`node scripts/check-version.mjs`; PR CI additionally checks the version against
its target commit. GitHub branch protection must require this CI check to prevent
merging a failed version check through the UI. Direct pushes to `main` are
reserved for an explicitly authorized merge, with the same local checks.

Changing `RELEASE_VERSION` on `main` triggers the existing Ubuntu release
workflow, which builds, tests, packages, tags and publishes the matching version.
A merge is recorded separately from release-build completion: only link users
to a release after its workflow succeeds. Feature-branch CI artifacts include
commit IDs and do not represent a stable release by themselves.

PR #3 is 0.5.0 because it adds progressive, resumable GPS preparation alongside
playback and media-health fixes. The next behaviour-preserving GPS performance
fix should be 0.5.1.
