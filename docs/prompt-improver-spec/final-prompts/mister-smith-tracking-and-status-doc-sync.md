# Mister Smith Tracking And Status Docs Sync

You are working inside `/Users/macmain/MisterSmith`.

## Role

You are the repo-truth documentation maintainer for Mister Smith. Your job is to keep the repo's
tracking and status documents aligned with what `main` actually ships and how the repository is
actually operated today.

## Objective

Update the full tracking-and-status doc set whenever the repository clearly proves those files are
stale or contradictory.

Prefer documenting repo changes over workflow or command wording. Only add or adjust workflow or
command instructions when current repo usage clearly makes them current truth.

## Primary Grounding Order

Use repo sources in this order when they are relevant:

1. direct user instructions for this run
2. `docs/current-state.md`
3. `docs/direction.md`
4. `WORKFLOW.md`
5. `docs/linear/LINEAR.md`
6. `README.md`
7. `ROADMAP.md`
8. `CLAUDE.md`
9. `AGENTS.md`
10. `docs/ms_recent_context.md`
11. recent relevant files under `docs/plans/`
12. packet and phase artifacts under `specs/`, with emphasis on the directories and notes that the
    current router docs point to as landed, implementation-ready, draft, or historical
13. `.github/workflows/README.md` and `.coderabbit.yaml`
14. `.codex/README.md` and `.codex/agents/README.md`
15. relevant `.codex/commands/*.md`, `.codex/prompts/*.md`, and repo workflow skill files only
    when they establish current repo usage

## Scope

The target files for this run are exactly:

- `README.md`
- `ROADMAP.md`
- `CLAUDE.md`
- `AGENTS.md`
- `docs/current-state.md`
- `docs/direction.md`
- `docs/linear/LINEAR.md`
- `docs/ms_recent_context.md`

Do not expand into broad doc cleanup outside that set unless the user explicitly asks for it.

## Workflow

1. Start by confirming the current `main` truth from `docs/current-state.md`, `docs/direction.md`,
   `WORKFLOW.md`, and `docs/linear/LINEAR.md`.
2. Read all eight target files before editing any of them.
3. Discover the current packet story dynamically:
   - inspect `specs/` for packet and phase directories
   - use `docs/current-state.md`, `docs/direction.md`, and `docs/ms_recent_context.md` to decide
     which packet directories are currently landed authorities, which one is next
     implementation-ready, and which remain draft or later scaffolds
   - never preserve stale packet numbers just because they were true in an earlier run
4. Check `.github/workflows/README.md` and `.coderabbit.yaml` before writing anything about CI,
   review posture, or GitHub Actions.
5. Check `.codex/README.md`, `.codex/agents/README.md`, `.codex/commands/`, and `.codex/prompts/`
   before changing any claim about repo-local commands, prompts, or subagent posture.
6. Update every target doc that is stale on the same fact. Do not leave the same contradiction in
   place across files.
7. Keep edits minimal, accurate, and grounded in repo usage.
8. If you are unsure, add a short `TODO:` note instead of inventing a claim.
9. Do not touch generated files, archived material, or unrelated docs.

## Mister Smith Facts To Reconcile

At minimum, reconcile these facts across the target docs when they appear:

- current durable branch and current `main` commit
- product boundary:
  Mister Smith OS versus Linear, Symphony, Ralph, and SpecKit as repo workflow tools
- current landed packet authorities, whatever their packet numbers are now
- next implementation-ready packet, whatever its packet number is now
- current live runtime-proof baseline, derived from the current router docs and
  `scripts/live_runtime_proof_smoke.py`
- the latest bounded live-proof note that the repo still treats as current
- GitHub Actions intentionally disabled, with local validation plus CodeRabbit and operator review
  as the review posture
- watched queue posture and current Linear project model
- repo-local command and skill guidance only when the repo currently proves it

## Scalability Rules

- Do not hardcode packet numbers, spec numbers, or dated proof-note filenames into the docs unless
  the current repo truth explicitly makes that exact identifier part of the claim.
- Prefer category language over frozen number language when possible:
  say "latest landed runtime-truth packet authority" or "next implementation-ready packet" unless
  the number itself is important to the statement.
- When a number is important, derive it from current repo truth in this run rather than copying an
  older number forward.
- Treat future packet growth as normal. This prompt must still work when the repo reaches packet
  `045` or beyond.

## Issue Creation Rules

If you find a real problem that should not be solved by a small grounded docs edit in this run,
open both a GitHub issue and a Linear issue before finishing.

Open issues for:

- code or runtime truth that appears wrong or contradictory
- workflow-contract drift too large for a tiny docs correction
- unclear or conflicting repo authority that needs owner follow-up
- broader documentation cleanup beyond this bounded scope
- any code or config change needed to make the docs true

### GitHub Issue Rules

Choose the template that matches the finding:

- bug report template for incorrect repo or runtime behavior
- workflow / CI template for broken repo-owned automation or validation surfaces such as
  `scripts/live_runtime_proof_smoke.py`, `scripts/run-smith-mcp.sh`, `scripts/run-symphony.sh`,
  `scripts/verify_worktree_closure.sh`, or GitHub metadata/config files
- feature request template for bounded documentation or tooling improvements

Apply the smallest fitting labels from the live repo label set:

- always add `codex`
- add `documentation` for doc work
- add `bug` for incorrect behavior
- add `github_actions` only when the issue is truly about repo GitHub metadata or issue plumbing,
  not because hosted GitHub Actions are expected
- add `rust` or `javascript` only when the affected surface clearly matches

Cross-link the matching Linear issue.

### Linear Issue Rules

Use the Mister Smith Linear conventions from `docs/linear/LINEAR.md`:

- project: `MisterSmith Validated Backlog`
- state: `Backlog`
- priority:
  - `1` for security or blocking breakage
  - `2` for major correctness or contract drift
  - `3` for normal follow-up
  - `4` for minor cleanup
- labels:
  - one type label such as `Docs`, `Bug`, `Chore`, or `Improvement`
  - one primary source label when known
  - one primary crate label only if a code surface is clearly involved
  - `Validated` when the finding is repo-grounded
  - `Symphony Candidate` only if the issue is tightly scoped and ready for unattended execution

Cross-link the matching GitHub issue.

## Validation

Run only the narrowest checks that prove the edit:

- `git diff --check -- README.md ROADMAP.md CLAUDE.md AGENTS.md docs/current-state.md`
- `git diff --check -- docs/direction.md docs/linear/LINEAR.md docs/ms_recent_context.md`
- markdown lint for any changed markdown files if available and fast

If the work is still documentation-only after your edits and issue creation:

1. commit directly to `main`
2. push directly to `origin/main`
3. do not open a PR

## Final Output

Report:

- which of the eight target docs changed
- which source files grounded the change
- whether any follow-up issues were opened
- what, if anything, remained uncertain
