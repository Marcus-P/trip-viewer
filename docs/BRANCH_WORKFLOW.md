# Branch inventory and integration workflow

Reviewed against GitHub on 2026-11-16. `main` is the stable branch (there is no
`master` branch). Preserve `develop` as the permanent integration branch.

| Branch | State / purpose | Decision |
| --- | --- | --- |
| `main` | Stable 0.10.3 at `62d014e` | Keep; owner-only protected merges |
| `develop` | Equal to stable main at the start of 0.11.0 work | Keep permanently; promote through a PR to main |
| `feat/amd-timelapse-encoding` | AMF, Windows QSV and Linux VA-API verified encoding | Active; PR to develop, then focused GPU acceptance |
| `fix/viofo-timelapse` | Fully contained in stable 0.10.2 | Obsolete; eligible for deletion |
| `fix/viofo-legacy-filenames` | Fully contained in stable 0.10.3 | Obsolete; eligible for deletion |
| `contrib/viofo-playback-081` | Open upstream [PR #20](https://github.com/chrisl8/trip-viewer/pull/20) | Keep; include the shared timelapse fix |
| `docs/codex-development-attribution` | Fully contained in main, merged PR #12 | Obsolete; eligible for deletion |
| `feat/download-statistics` | Fully contained in main, merged PR #13 | Obsolete; eligible for deletion |
| `fix/windows-installer-download` | Fully contained in main, merged PR #14 | Obsolete; eligible for deletion |

At review time upstream #20 was open and mergeable. The feature branch above
prepares the next integration PR. The obsolete branches were audited, not deleted; retain
all unmerged work and ask the owner to decide cleanup. Deferred ideas in
[project state](PROJECT_STATE.md) are not hidden branch changes. Installation
telemetry remains explicitly deferred; public download statistics already exist.

GitHub automatic head-branch deletion is disabled. Do not manually delete
develop after promotion. The integration could not create an additional develop
anti-deletion/non-fast-forward ruleset (GitHub returned HTTP 403); no main rule
was changed or bypassed. If desired, the owner can add that protection under
Settings → Rules → Rulesets, targeting `develop` and restricting deletion and
force pushes. The existing main protections remain independent.

## Version and merge sequence

1. Start from current main and prepare a separate bugfix/feature branch.
2. PR the fix to develop; run the normal relevant CI once and perform the focused
   owner acceptance using the relevant platform installer.
3. After acceptance, merge the fix PR into develop. Preserve develop and prepare
   develop → main with the same proposed version and required checks.
4. The owner merges the promotion to main. That push automatically builds and
   publishes Ubuntu, Windows and experimental universal Mac installers.

For the GPU feature the proposed version is **0.11.0**, one MINOR after stable 0.10.3.
An integration merge does not publish a release or consume another version.
After publication, bring develop forward to main through ordinary history (no
reset/force push). Keep fork-only release work out of Chris' contribution.
