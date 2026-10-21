# Stable branch protection

Verified against GitHub’s effective rules for `refs/heads/main` on **2026-10-21**.
The default/stable branch is `main`. The repository is personal and its sole
administrator listed by GitHub is the owner, `Marcus-P`.

## Active rulesets

| Ruleset | Rules | Bypass |
| --- | --- | --- |
| [History and checked PRs](https://github.com/Marcus-P/trip-viewer/rules/25574527) | Block deletion/non-fast-forward updates; require PR, resolved review threads and strict current successful `ubuntu` status | None, including owner |
| [Owner-only merges](https://github.com/Marcus-P/trip-viewer/rules/25574534) | Restrict branch updates | Repository admin role 5, **pull requests only** |

For this personal repository, the admin-role allowance is the owner. It does not
allow other writers to merge or allow direct owner pushes. It also cannot bypass
the separate history/CI ruleset, so `--admin` is not a way around a failed check.
No human approval count is required; the owner can merge their own checked PR.
The owner retains GitHub’s administrative ability to edit protection settings.
If repository ownership/roles change, review the role-based restriction again.

## Verify without changing settings

```bash
gh api repos/Marcus-P/trip-viewer/rulesets
gh api repos/Marcus-P/trip-viewer/rules/branches/main
gh api repos/Marcus-P/trip-viewer/rulesets/25574527
gh api repos/Marcus-P/trip-viewer/rulesets/25574534
```

Check `enforcement: active`, exact `refs/heads/main` scope, the empty bypass list
on the first ruleset, and the admin **pull_request** allowance on the second.
Do not test protection by attempting a force push or a destructive branch change.

## Setup/recovery helper

The owner-run `bash scripts/protect-main.sh` checks the authenticated account,
personal ownership/default branch, creates missing named rulesets and verifies
the stored configuration. Same-named mismatched rules fail for review rather than
being overwritten. Payloads are in `scripts/github/`. The helper needs owner
credentials with repository Administration write access; a content-writing GitHub
integration may lack that permission even when the owner’s account is admin.

The earlier 403 during setup is historical: the owner subsequently activated the
rules, and the actual effective rules are now verified. Do not report the old
permission failure as evidence that main remains unprotected.
