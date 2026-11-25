# AGENTS.md

These instructions apply to the whole repository.

## Origin, scope and adaptation

The owner-defined working and communication rules in this file were specified
by **@Marcus-P** for [Marcus-P/trip-viewer](https://github.com/Marcus-P/trip-viewer).
Their source is that repository's **AGENTS.md**; retain the copied revision's
commit or tag when reusing them. Codex helped organize the wording. This
attribution does not claim ownership of upstream code or other contributors'
work, whose authorship and license notices must also remain intact.

Copies, translations and adaptations of these rules must retain attribution to
@Marcus-P's original basis, the source repository/path and the source revision
when available. Changes are permitted: identify the adapting project/maintainer
and distinguish their changes from the original rules. Do not attribute changed
preferences to @Marcus-P or imply endorsement without confirmation. Keep this
provenance with the reused rules rather than relying on an unrelated mention.
This working-rule requirement does not replace or amend the repository's MIT
license or existing copyright/license notices.

### Owner-derived prompts, instructions and reusable AI material

The same provenance requirement covers prompts, custom instructions, rules,
Skills/reusable instruction packages, templates, checklists and workflow
instructions actually derived from @Marcus-P's supplied concepts or
requirements. It applies to reuse, extraction, summaries, translations,
reformulations and adaptations, including outside this file or repository.
Attribute the original concepts/requirements to @Marcus-P and distinguish
wording/implementation assistance from Codex and contributions from others.
Keep inherited or independently contributed material and general technical
knowledge distinct; do not invent an origin or exclusive-ownership claim.

Keep attribution with the reused material and identify subsequent adaptations,
as required above. Use the actual source document/repository and revision when
available. A private source does not authorize publishing its prompt text,
conversation, personal details or unpublished concepts. Retain source credit
without disclosing that material; use a public reference only when one exists
and is authorized for the purpose. Do not invent a public source link or copy
private chat into this repository to demonstrate provenance. Existing licenses
and other contributors' notices remain unchanged.

### Knowledge documents and retrieval/RAG sources

Knowledge documents and retrieval/RAG source material are excluded from the
owner-attribution requirement above. Supplying, selecting, possessing, uploading
or using a document does not establish that @Marcus-P authored or owns it.
Preserve the actual source, verified authorship, license and usage permissions;
if authorship is unknown, keep it unknown rather than assigning it to the
owner. Where a prompt or Skill includes third-party knowledge, distinguish any
confirmed owner-derived instructions from that enclosed material.

Treat non-public knowledge inputs as private. Permission to use a document for
an authorized task is not permission to publish its content, identifying source
metadata, names or private paths, expose it through a public index, or upload
it to another service. Such disclosure requires explicit authorization and
appropriate source rights; respect applicable data-protection/confidentiality
requirements. Preserve privacy when documenting provenance and share only the
minimum information authorized for the actual purpose.

General project safeguards below apply to work in this repository. The separate
owner-specific preferences describe work performed for @Marcus-P; they do not
assign personal schedules, devices or commitments to other contributors. When
adopting this basis elsewhere, explicitly choose the applicable local preferences
and roles while preserving the original source attribution. In this fork, do
not silently change the owner's preferences without authorization.

## Start here

Before changing code, read `docs/PROJECT_STATE.md`. It is the durable hand-off for the current development state, validated behavior, open work, and intentionally deferred ideas. Update it whenever a change materially alters those facts.

Before preparing external communication, follow the **External collaboration and communication** section below and refresh the relevant public discussion and development context. AGENTS.md is the binding entry point for future agent sessions; the linked project records supply evidence, not a parallel communication policy. Existing authorization, media-safety, branch and release rules remain in force.

## Working agreement

- Upstream-first development: before application changes or preparing any contribution, fetch current `chrisl8/trip-viewer` main and compare it with the intended source branch. Run `node scripts/check-upstream.mjs` before code work. Incorporate compatible upstream changes on an isolated working branch, preserving original authorship, existing camera support and accepted fork behavior. Resolve conflicts deliberately and run only relevant checks. Do not continue application work on a stale base without a recorded owner-approved exception. This routine does not authorize protected-branch merges, releases, force pushes, deleting branches/PRs or overriding existing approval requirements.
- Before presenting a PR as ready, refresh the upstream comparison and require the `Upstream freshness` code check to pass. It reports the actual upstream/head SHAs and ancestry; a failed network check is not a pass. Documentation/version/development-policy-only PRs still report drift but may proceed without an application integration, so policy maintenance does not force a runtime merge/build. Record that exception openly; it is not evidence that the application is current. A passing check is point-in-time evidence; refresh after upstream moves and before final handoff. The separate check is a working readiness requirement, not a claim that GitHub branch-protection settings were modified.
- Upstream-targeted PRs do not automatically run this fork's PR workflow. Before handing one off, also use the fork's manually dispatched `Upstream freshness` workflow with its `source_ref` set to the actual contribution branch (for example `origin/contrib/viofo-camera-only`) and retain its checked SHA/result. Manual checks are strict. The dispatch entry must first be available through the normal default-branch policy promotion; until then run the strict local check and explicitly report that the CI handoff is not yet available. Do not copy fork-specific CI/release machinery into an upstream contribution merely to manufacture a check, or claim this workflow changes upstream branch protection.
- Reliability before features: preserve synchronized playback, channel-local degradation/recovery, cancellation, archive/session isolation and GPS/map consistency before optional UI/performance expansion. Camera filename/GPS support alone does not certify reliable full-app playback. Surface any confirmed integration blockers before recommending a camera-support merge; do not hide a necessary safety fix merely to reduce PR size. Do not generalize the owner's original hardware problems to all VIOFO cameras without evidence.
- Prefer evidence over speculation. Inspect the relevant code, logs, tests, media metadata, or upstream behavior before changing architecture.
- Keep user media non-destructive. Never modify original dashcam files in place. Diagnostics may read them; generated/repaired derivatives must be separate files.
- Small, reviewable commits are preferred. Keep bug fixes, cleanup, and unrelated feature work separable.
- Run the relevant frontend tests/typecheck and Rust tests for code changes. For Linux playback/bundling changes, require the Ubuntu AppImage build to pass before considering the change ready.
- Run only checks relevant to the diff. Documentation-only changes need link/format checks, not frontend/Rust tests, AppImage builds or downloadable test artifacts. Check version-file consistency when release metadata changes. Do not rerun accepted playback tests for documentation edits. Keep the required `ubuntu` status green through a lightweight documentation path rather than skipping the entire workflow and leaving a required check pending.
- Every push to `main` must build Windows, experimental universal macOS and Ubuntu installers and publish durable GitHub Release downloads for that pushed head. Do not restrict publication to version-file paths. Normal version-increasing merges publish stable releases; additional unchanged-version commits use immutable commit-specific prereleases and never overwrite stable assets or become automatic stable updates. Keep permanent downloads separate from temporary Actions storage.
- Use GitHub Actions storage sparingly. Upload only the smallest artifact needed for an active acceptance test; avoid duplicate packages, dependency/build directories and unnecessary caches. Keep test artifacts at seven days or less, and remove obsolete artifacts/caches after acceptance or verified publication when authorized. Preserve current acceptance downloads until they are no longer needed, published release assets/license material and useful check history. Do not delete another repository's storage or add recurring cleanup runs without a concrete need and authorization.
- Do not merge a pull request, delete data, rewrite published history, or make destructive repository changes without explicit user authorization.
- Investigation, code review, web research, and other non-destructive checks may be performed without asking for permission first.
- New feature requests, UI improvements, and unrelated cleanup should be recorded under **On hold** in `docs/PROJECT_STATE.md` unless the user explicitly authorizes implementation.
- Preserve contributor authorship, source provenance and license information. Explicitly acknowledge earlier work and the actual code, fixtures, fixes, tests and findings incorporated from each contributor, regardless of which PR or branch becomes the integration base. Do not infer direct derivation, independent invention or co-authorship without evidence; retain original commit authorship when bringing commits across. Keep generally useful/upstream-suitable fixes separate from fork-specific branding or release configuration where practical.
- Do not casually change the Tauri identifier `com.tripviewer.app`; it currently matches upstream. Treat any identifier change as a separate compatibility/release decision.
- Do not run `npm audit fix` blindly. Inspect findings first and prefer targeted dependency changes.
- Do not conflate Vite bundle-size warnings with WebKitGTK/GStreamer playback faults unless evidence supports a link.
- Document hands-on camera validation as VIOFO A229 Pro 3CH on Ubuntu 24.04 LTS and the owner-confirmed Windows 11 x64 system (no additional codec/plugin installed). Do not generalize codec availability to all Windows 11 devices. Windows 10 Front/8x timelapse encoding and finished-output playback were owner-confirmed on 2026-11-24 after installing the required HEVC codec, for all segments of two recognized trips. Do not infer codec purchase/source, universal Windows 10 support or coverage of other channels/tiers; show the Store-displayed price, not a guaranteed fixed price. Retain existing camera support; distinguish automated/synthetic checks from hands-on hardware tests.
- Preserve upstream provenance, existing logos/icons, source copyright and dependency license notices when refreshing documentation.
- Identify the development assistance used for this fork as Codex, OpenAI's coding agent. Credit the maintainer's requirements, test footage, hands-on tests and acceptance separately, while preserving upstream/contributor authorship and license notices. Historical records mentioning ChatGPT need not be rewritten.
- Keep current download/install instructions version-neutral and point to Latest Release. Put release/merge history in release notes and project records; do not insert old release or PR narratives into current installation guidance.
- If a Linux/WebKit media workaround is already validated by the user, do not remove it merely to simplify the code. Regression evidence is required.

## Owner-specific working preferences (@Marcus-P)

These are @Marcus-P's collaboration preferences, not universal requirements for
other contributors' personal workflows. Apply them when acting for this owner.

- Long-term direction: reduce fork-specific divergence by offering useful changes upstream in small coherent contributions, with the owner's goal of eventually using upstream without maintaining this fork. This is the owner's goal, not an upstream commitment; @chrisl8 retains final scope, architecture and merge decisions. Upstream may choose, reorder or combine contributions. Keep fork-specific releases/branding and owner workflow rules separate unless requested.
- Current contribution order: keep the VIOFO work in upstream PR #21 first. Do not replace/close it or PR #20 without explicit authorization. After #21 is merged, refresh the remaining feature inventory against then-current upstream and prepare a concise menu for @chrisl8 to select from. Offer a full-fork installer demonstration only as an optional overview, with clear platform/testing limits and no obligation to install, test or adopt it. Prepare messages first; publication still needs explicit authorization.
- Prepare reliability follow-ups close behind camera support: the earliest accepted fork progression included Linux playback workarounds, synchronized channel recovery and progressive/restartable GPS preparation before later navigation/overlay enhancements. Keep follow-ups reviewable and identify their dependencies, migration costs and actual validation. Do not schedule those correctness fixes behind optional bookmarks/overlays merely because the latter are more visible. Treat them as a proposed sequence until the upstream maintainer agrees.
- Cost/work cadence: work for at most two hours at a time. If unfinished, checkpoint progress and pause for five hours before the next work block; repeat as needed. Finish immediately if the task completes earlier. Do not start builds, CI or background work during a pause. Prefer targeted checks and reuse existing dependencies/build output rather than redundant builds or artifacts.
- End each final response with concrete next steps for the user. If progress requires missing information or a decision, end with a focused question that makes the blocker clear. Continue independently whenever the existing authorization and available information suffice.
- When a new desktop build needs user testing, finish with actionable update/install instructions and a small, specific test; account for the fact that repository or cloud updates do not update the locally installed app.
- Work in concise, concrete steps. Reuse recorded preferences, decisions, environment details and accepted results when they still apply; refresh only what may have changed. Resolve missing context through available project evidence before asking the owner to repeat it. Ask only questions that materially affect the outcome, and continue independent authorized work where possible.
- Optimize the owner’s effort as well as machine time. Lead with the outcome, explain its significance and give the next concrete action. For any action the owner must perform, provide the complete current text or command block, prerequisites, paths/placeholders and the exact destination, as required for external handoffs below. Do not hand over replacement fragments or raw tool dumps that require the owner to reconstruct the task.
- Continue necessary reversible work already authorized by the task without redundant confirmation. Reuse permission within its granted scope; this does not authorize unapproved merges, external messages, releases or destructive actions.

## Communication and testing

- State uncertainty explicitly. Distinguish investigations performed, confirmed results, code-derived conclusions, hypotheses, proposals, offers, agreed actions and open decisions. Separate build success, automated/synthetic checks, real hardware tests and validated packages. Name the tested OS, artifact and commit where known; keep full-fork, isolated upstream-PR and standalone-diagnostic evidence separate. Do not generalize from limited samples or infer a package test from a source test. State unavailable information rather than inventing it.
- When a user reports that a test succeeded, treat that result as established until a later regression disproves it.
- For media failures, prefer channel-local degradation over stopping the entire trip whenever at least one healthy camera can continue.
- A recovered camera should rejoin as a slave during the current source. Do not switch the promoted sync master back mid-segment unless there is a strong reason.

### Verification goals and stopping rules

- Before an expensive check, identify the changed behavior, relevant risk or
  unresolved question, the smallest representative check that can answer it,
  and the stopping condition. Required CI, safety checks and explicit owner
  requirements still apply; resource preferences do not bypass them.
- Reuse valid evidence for unchanged relevant code, configuration and test scope.
  Once the relevant checks and mandatory gates pass, stop. Repeating an accepted
  test or expanding to a full suite needs a concrete reason: a relevant change,
  a failure, unresolved risk, missing evidence or an explicit requirement.
  Test counts, coverage percentages and broad suites are not proof of complete
  correctness or universal compatibility.
- Minimize combined compute and coordination cost. A small targeted check may
  be worthwhile when it prevents several clarification rounds or owner tests.
  Keep a compact evidence/decision handoff so future sessions can reuse what is
  still valid, while refreshing external context when needed. Use the existing
  project-state and scope rules, not another parallel tracking system.

## External collaboration and communication

These requirements apply to GitHub comments, PR descriptions, issue replies and
other external messages. Prepare drafts within the authorized scope. Publishing
or editing an external message requires explicit owner authorization; preparing
a draft or receiving approval of its wording alone is not an instruction to send
it. Reuse authorization already granted for the same action and scope rather
than requesting redundant approval. Never merge, publish releases or change
another project's settings merely to complete a communication task.

### Refresh context and make references explicit

- Before drafting, inspect the relevant existing PRs/issues, published comments,
  inline review comments and reviews, maintainer recommendations, contributor
  positions, current commits/changes and recorded decisions. Check the fork
  owner's previous statements, offers and commitments too. Refresh again before
  handoff if the relevant thread or development state has changed. Report any
  access limitation and distinguish cached context from a current check.
- Build on what each participant has actually said or can reasonably know. Avoid
  reintroducing resolved matters, contradicting prior statements or repeating
  documented results without a concrete purpose. Explain new evidence and the
  prerequisites needed to understand it. Do not attribute the owner's observed
  problem to another contributor's camera or setup without evidence.
- Verify actual GitHub handles before use. Refer to named GitHub participants
  consistently by their verified @username, not guessed first names, display
  names or alternating personal names. Avoid unnecessary repeated mentions and
  notifications; use clear pronouns once attribution is established.
- Verify links, issue/PR numbers and commit SHAs. Distinguish a PR number from a
  commit hash and identify the repository when numbers could be ambiguous.
  Prefer the exact comment permalink when responding to a particular statement.
- When drafting in the owner's voice, use natural first-person wording such as
  "I tested" and "my PR", explicitly attribute other implementations and shared
  upstream code, and explain what a numbered reference denotes on first use.
  Use "we" only for established joint work, not to imply shared authority or
  unagreed coordination. Do not invent other people's motives or expertise.

### Respect authority, autonomy and voluntary cooperation

- Distinguish the fork owner, upstream maintainer and other contributors, their
  technical contributions, individual goals, common interests and agreed
  responsibilities. For upstream Trip Viewer, @chrisl8 decides its development
  direction, architecture, scope, quality requirements, releases and merges.
  Respect the corresponding maintainer's authority in every external project.
  An offer to contribute is not responsibility for that project's direction.
- Do not recast a maintainer's goals as the fork owner's personal goals without
  evidence, or suggest joint decision-making where the final decision belongs
  to the maintainer. Describe ideas as proposals until actually agreed.
- Offer initiative, investigation, implementation or coordination when helpful;
  do not unilaterally appoint the owner as lead, assign work to others or claim
  responsibilities that were not agreed. Supporting another contributor's lead
  and offering to lead are equally constructive. Avoid both paternalism and
  unnecessary self-subordination.
- Do not invent personal goals, promises, obligations or responsibilities for
  the owner. Keep an offer distinct from an agreed commitment, and check that
  proposed wording does not silently turn one into the other.
- Respect volunteer time and autonomy. Welcome small contributions or a quick
  test with a couple of files when time permits. Extensive work on our side
  does not oblige someone else to match it. Avoid unnecessary deadlines,
  demands, required justifications or organizational/quality pressure. Preserve
  relevant technical findings and limitations rather than weakening them for
  speculative interpersonal concerns.
- Treat overlapping implementations as opportunities to compare evidence and
  combine complementary strengths, not as a contest between contributors.
  Do not claim preference for our PR without a technical or explicitly
  authorized basis. Respect others' independent initiative and choices; apply
  the authorship/provenance/credit rules above to the final combined work.

### Make each handoff actionable and proportionate

- Provide every prepared GitHub comment as one complete, directly copyable
  Markdown source block, including after revisions. Do not make the owner
  assemble replacement fragments. Use a longer outer fence when the text
  contains code fences so the copied text retains its Markdown formatting.
- Supply the verified destination link and exact posting steps. For replies,
  link the target comment and explain Quote reply or the available comment
  field; do not invent a direct-field URL or assume a button is visible.
  Explain removal of an automatic quote and the Preview/submit step as needed.
  Report accurately whether a message is prepared, posted or edited.
- For test requests, explain what must first be downloaded, extracted, built or
  installed, the supported OS/architecture, dependencies, exact commands,
  placeholders, relevant selection limits, expected output and how to return
  the smallest useful result. Verify prerequisites and instructions against
  the actual artifact. Do not assume a contributor has our tools, platform,
  samples or prior context. Offer alternatives according to known capabilities;
  do not guess their OS or skill level. Build availability is not validation.
- Keep media and personal data local where possible. Request only the evidence
  needed and explain report contents and automatic-upload behavior. Allow
  summaries or anonymized excerpts instead of footage, GPS tracks or a full
  report when sufficient; do not make a voluntary test contingent on sharing
  personal data. Apply the privacy rules below to repository documentation.
- Write factual, respectful, human, constructive, cooperative and appropriately
  confident messages. Prefer natural, clear, complete sentences and familiar
  wording. Avoid habitual em dashes, flattery, self-promotion, melodrama,
  needless apologies, confrontation, repeated points, presumed motives and
  imposed leadership. Reduce complexity without losing technical substance or
  important qualifications. Size the message to its purpose: detailed setup
  can be useful; a short status update should not repeat an entire report.
- Explain Codex's assistance and the owner's and other contributors' actual
  roles using the existing attribution rule, without blurring credit or
  presenting generated analysis as the owner's hands-on testing.

### Mandatory review before handing a draft to the owner

Check each draft against all of the following and correct it independently
within the authorized position and scope:

1. Current discussion/development context is reflected; no contradictions,
   unnecessary repetition or already-resolved debate is introduced.
2. Handles, destinations, repositories, PR/issue references and commits are
   verified; attribution and reader prerequisites are explicit.
3. Technical statements have appropriate evidence and scope; tests, inferences,
   hypotheses, proposals, offers and agreed actions remain distinguishable.
4. The owner's and others' goals, contributions, responsibilities and final
   decision authority are accurately represented. No unagreed role or new
   commitment is claimed, and all participants retain their choices.
5. Voluntary cooperation is respected, useful small contributions are welcome,
   credit is appropriate, and wording adds no unnecessary pressure or
   paternalism.
6. Tone, length, Markdown and practical instructions fit the actual purpose.
   Technical findings and combined metadata do not expose the owner's real-world
   identity or private activity; publication has the required authorization.

If a necessary substantive correction would change an explicitly specified
position or exceed authorization, flag that proposed change clearly instead of
silently making it or sending the draft.

## Privacy / repository hygiene

This repository is public-facing project state. Do not commit secrets or personal/private information.

- Never commit passwords, tokens, private account identifiers, personal addresses, private messages, family details, or unrelated user profile information.
- Avoid committing machine-specific absolute paths, local OS usernames, mount points, or home-directory names. Use generic examples such as `/path/to/archive`. Public, verified GitHub handles and public project URLs may be used when relevant to project attribution or coordination.
- Media sample notes should describe only the technical properties needed to reproduce a bug. Use a basename only when necessary and avoid unrelated location/time context.
- The owner permits relevant media basenames to be shared. This is not consent to disclose locations, routes, exact GPS positions, faces/voices, vehicle identifiers, private host/account/device/network identifiers, home/work links or movement patterns. Technical findings must not introduce information that identifies the owner in the real world. Assess both individual fields and combinations with other public information; aggregate, redact or omit unnecessary identifying/quasi-identifying details. If the remaining disclosure risk is unclear, withhold the detail and clarify authorization rather than claiming anonymity. Public handle/source attribution does not authorize enriching it with a real-world identity.
- Before publishing new commits on the owner's behalf, inspect the author/committer identity as well as file contents. Prefer the verified public GitHub handle and its verified GitHub no-reply address over a configured real name or personal/custom email unless the owner explicitly chooses otherwise. Preserve other contributors' actual authorship and license notices. Do not rewrite published history without explicit authorization, and do not claim that a history rewrite erases prior public copies.
- At the start of a session creating commits for this fork's owner, run `node scripts/commit-privacy.mjs setup` and verify the resulting repository-local identity. This installs shared local pre-commit, commit-message and pre-push guards without changing global Git settings. Existing unrelated hooks or a custom hooksPath require integration instead of silent replacement. Keep any known private addresses in repeated local `privacy.blockedEmail` settings only, never in tracked rules or test fixtures. Preserve those settings when refreshing the guards. Hooks do not travel with a clone; repeat setup in a new checkout before committing. Other projects choose their own policy; preserve other contributors' original author identities.
- The guards enforce the configured no-reply committer for new local commits and check configured private-address markers in new metadata, messages and added text. They do not inspect every possible private detail or remove published history. Review location data and other identifying content separately before publication. Do not bypass the guards to publish an owner commit containing private data.
- Match privacy remediation to the owner's stated goal. If the owner accepts residual historical discoverability, prioritize future prevention and small authorized reductions in exposure. Do not make Support requests, full-history rewrites or exhaustive audits a prerequisite for useful progress. Preserve active collaboration, protected branches, contributor authorship and useful current validation; report what remains accessible without promising invisibility or erasure.
- Do not copy private conversation text into the repository. Summarize only project-relevant technical decisions and working agreements.

## Branch / PR discipline

- `develop` is the permanent integration branch. Keep it and the active upstream contribution branch. Bugfix/feature PRs target `develop`; after acceptance prepare `develop` → `main` for the owner. Each stable promotion increments the main version once, not once per integration merge. Keep proposed versions synchronized in integration, with release notes; only main publishes installers.
- Acceptance installers must use a unique prerelease below the upcoming stable version; never reuse the stable version for a test package. Follow the RC stamping, Debian tilde mapping and finalization rules in `docs/VERSIONING.md`. Main must contain stable metadata, and the app must recognize RC → stable as an upgrade.
- `main` is the stable branch. The user reserves stable merges to the repository owner and requested force-push/merge protection. Never use force pushes or admin overrides to bypass protection; the active rules and verification are in `docs/BRANCH_PROTECTION.md`.
- Follow `docs/VERSIONING.md`: every merged PR increases PATCH, MINOR or MAJOR once as appropriate, with all version files synchronized and CI green. Every merge also updates release notes. Pure documentation PRs still increment PATCH by the owner’s explicit preference, but run only lightweight documentation/version checks; the later release builds packages once to embed that version. Version changes belong in the PR before merging.
- PR #3 (`fix/webkit-sync-stability`) was merged as version 0.5.0 after green CI and explicit user confirmation of the final playback/GPS-resume tests; see `docs/PROJECT_STATE.md`.
- Start subsequent feature and performance work on separate branches from current `main`. Preserve the validated playback and GPS semantics.
- Before merging a later PR, require green CI and explicit user authorization; request focused real-world validation when the change affects playback or GPS behaviour.
