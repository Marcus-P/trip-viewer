# AGENTS.md

These instructions apply to the whole repository.

## Start here

Before changing code, read `docs/PROJECT_STATE.md`. It is the durable hand-off for the current development state, validated behavior, open work, and intentionally deferred ideas. Update it whenever a change materially alters those facts.

## Working agreement

- Prefer evidence over speculation. Inspect the relevant code, logs, tests, media metadata, or upstream behavior before changing architecture.
- Keep user media non-destructive. Never modify original dashcam files in place. Diagnostics may read them; generated/repaired derivatives must be separate files.
- Small, reviewable commits are preferred. Keep bug fixes, cleanup, and unrelated feature work separable.
- Run the relevant frontend tests/typecheck and Rust tests for code changes. For Linux playback/bundling changes, require the Ubuntu AppImage build to pass before considering the change ready.
- Do not merge a pull request, delete data, rewrite published history, or make destructive repository changes without explicit user authorization.
- Investigation, code review, web research, and other non-destructive checks may be performed without asking for permission first.
- New feature requests, UI improvements, and unrelated cleanup should be recorded under **On hold** in `docs/PROJECT_STATE.md` unless the user explicitly authorizes implementation.
- Preserve contributor authorship. Keep generally useful/upstream-suitable fixes separate from fork-specific branding or release configuration where practical.
- Do not casually change the Tauri identifier `com.tripviewer.app`; it currently matches upstream. Treat any identifier change as a separate compatibility/release decision.
- Do not run `npm audit fix` blindly. Inspect findings first and prefer targeted dependency changes.
- Do not conflate Vite bundle-size warnings with WebKitGTK/GStreamer playback faults unless evidence supports a link.
- If a Linux/WebKit media workaround is already validated by the user, do not remove it merely to simplify the code. Regression evidence is required.

## Communication and testing

- End each final response with concrete next steps for the user. If progress requires missing information or a decision, end with a focused question that makes the blocker clear. Continue independently whenever the existing authorization and available information suffice.
- When a new desktop build needs user testing, finish with actionable update/install instructions and a small, specific test; account for the fact that repository or cloud updates do not update the locally installed app.
- Work in concise, concrete steps. When user-side testing is needed, request the smallest reproducible test rather than a long diagnostic session.
- State uncertainty explicitly. Distinguish observed facts, code-derived conclusions, and hypotheses.
- When a user reports that a test succeeded, treat that result as established until a later regression disproves it.
- For media failures, prefer channel-local degradation over stopping the entire trip whenever at least one healthy camera can continue.
- A recovered camera should rejoin as a slave during the current source. Do not switch the promoted sync master back mid-segment unless there is a strong reason.

## Privacy / repository hygiene

This repository is public-facing project state. Do not commit secrets or personal/private information.

- Never commit passwords, tokens, account identifiers, personal addresses, private messages, family details, or unrelated user profile information.
- Avoid committing machine-specific absolute paths, usernames, mount points, or home-directory names. Use generic examples such as `/path/to/archive`.
- Media sample notes should describe only the technical properties needed to reproduce a bug. Use a basename only when necessary and avoid unrelated location/time context.
- Do not copy private conversation text into the repository. Summarize only project-relevant technical decisions and working agreements.

## Branch / PR discipline

- `main` is the stable branch.
- Follow `docs/VERSIONING.md`: every merged PR increases PATCH, MINOR or MAJOR once as appropriate, with all version files synchronized and CI green. Version changes belong in the PR before merging so the packaged version is tested.
- Active stabilization work is tracked in PR #3 on `fix/webkit-sync-stability`; see `docs/PROJECT_STATE.md` for the current status.
- Keep PR #3 focused on playback/media-health/GPS robustness. Put unrelated feature work in a later branch.
- Before merging PR #3, require green CI and explicit user confirmation after the relevant real-world playback test.
