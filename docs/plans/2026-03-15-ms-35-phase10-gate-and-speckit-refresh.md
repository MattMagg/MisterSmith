# MS-35 Phase 10 Gate And SpecKit Refresh

## Objective

- Close the `MS-35` Phase 10 verification/docs gate on top of current `origin/main`.
- Re-apply only the still-valid parts of the stashed SpecKit/doc work after `e97d54a`
  landed bounded delegation provenance.

## Scope

- Phase 10 gate artifacts under `specs/012-phase10-frontier-autonomy/`
- High-level status docs: `ROADMAP.md`, `README.md`, `CLAUDE.md`
- SpecKit scaffolding under `.specify/` and `.codex/prompts/`
- Validation commands named by `MS-35` and the Phase 10 quickstart

## Assumptions

- `e97d54a feat(autonomy): enforce bounded delegation provenance (#190)` is now the
  authoritative repo baseline for Phase 10.6.
- The stash `phase10-readiness-cleanup-20260315-103853` is reference material, not a
  patch to replay wholesale.
- `crates/mister-smith-mcp/src/compatibility.rs` and the queue-governance notes are
  separate control-plane work unless this gate proves they are required.

## Constraints

- Do not mix unrelated smith MCP queue-governance work into the Phase 10 gate branch.
- Preserve reversibility with adjacent timestamp backups before overwriting tracked
  files.
- Keep the gate scoped to validation and documentation; do not reopen settled design
  decisions without proof from current validation.
- If Ralph is used later, rewrite `PROMPT.md` from the active issue context then; do
  not treat the checked-in prompt as current execution truth.

## Non-Goals

- Do not land the stashed `compatibility.rs` patch in this branch unless it becomes a
  verified gate blocker.
- Do not redesign the Phase 10 roadmap or create new queue slices here.
- Do not modify the live Linear workflow beyond what is required for the gate.

## Milestones

### 1. Refresh baseline context

- Confirm current Phase 10 code/doc state after PR #190.
- Separate stash contents into gate-relevant versus unrelated work.

**Validation**

- `git log --oneline HEAD..origin/main`
- `linear.get_issue(MS-35)`
- direct file inspection of current Phase 10 docs and stash references

### 2. Reconcile SpecKit and Phase 10 docs

- Refresh SpecKit scaffolding on the current baseline.
- Update Phase 10 spec/task/analyze/research/plan artifacts plus high-level docs to
  reflect completed Phase 10 implementation and the gate evidence.

**Validation**

- targeted diff review
- `markdownlint` on touched docs

### 3. Run the Phase 10 gate

- Execute the validation matrix named in `MS-35`.
- Record which quickstart scenarios are directly validated by automated tests and any
  remaining manual evidence.

**Validation**

- `cargo test -p mister-smith-agents`
- `cargo test -p mister-smith-persistence`
- `cargo test -p mister-smith-security`
- `cargo test -p mister-smith-llm`
- `cargo test -p mister-smith-core`
- `cargo test -p mister-smith-app`
- `cargo build --workspace`
- deploy artifact syntax validation

### 4. Land if clean

- Commit only the gate/spec refresh work.
- Push, open/update the PR, and merge if validation is clean.

**Validation**

- clean `git status`
- PR/merge success
- local `main` fast-forwards cleanly after merge

## Status

- **Current milestone**: Complete
- **Completed work**:
  - Confirmed local `main` had been behind `origin/main` and fast-forwarded the repo
    baseline to `e97d54a feat(autonomy): enforce bounded delegation provenance (#190)`.
  - Confirmed `MS-35` remains the active Phase 10 gate and that its scope is
    validation plus docs readiness, not additional runtime implementation.
  - Refreshed SpecKit in place with
    `uvx --from git+https://github.com/github/spec-kit.git specify init --here --force --ai codex --no-git`
    and recorded `speckit_version: 0.3.0` in `.specify/init-options.json`.
  - Reconciled `ROADMAP.md`, `README.md`, `CLAUDE.md`, and the Phase 10 artifact set
    so the roadmap, research, architecture references, quickstart, tasks, and analyze
    report all reflect the post-`e97d54a` repo baseline.
  - Ran the `MS-35` validation bundle:
    `cargo test -p mister-smith-agents`,
    `cargo test -p mister-smith-persistence`,
    `cargo test -p mister-smith-security`,
    `cargo test -p mister-smith-llm`,
    `cargo test -p mister-smith-core`,
    `cargo test -p mister-smith-app`,
    `python3 scripts/validate_deploy_assets.py deploy/dashboards deploy/alerts`,
    and `cargo build --workspace`.
  - Confirmed the stash `phase10-readiness-cleanup-20260315-103853` contains two
    separate streams:
    the Phase 10 gate/SpecKit work now reapplied on this branch, and a separate smith
    MCP queue-governance follow-up (`crates/mister-smith-mcp/src/compatibility.rs`
    plus throughput notes) that remains out of scope for `MS-35`.
- **Decisions**:
  - Land the SpecKit refresh together with the `MS-35` docs gate because both change
    the same Phase 10 artifact set and share the same validation evidence.
  - Do not replay or merge the stashed smith MCP queue-governance patch in this
    branch; treat it as separate validated-backlog follow-up work unless a later gate
    proves it is required.
  - Reuse this note as the durable closeout record instead of restoring the older
    pre-`e97d54a` audit note from the stash.
- **Open verification gap**:
  - `vet` remains unavailable in this environment because history-backed runs hit
    missing `ANTHROPIC_API_KEY`, `--no-history` required `OPENAI_API_KEY`, and the
    agentic Codex harness path did not complete.
  - `ROADMAP.md`, `README.md`, and `CLAUDE.md` still have substantial pre-existing
    markdownlint debt outside this scoped pass.
- **Next step**:
  - Stage only the `MS-35`/SpecKit refresh files, remove the temporary adjacent backup
    files, and land the gate branch.

## Stop Conditions

- Stop before merge if current validation contradicts the “Phase 10 complete” claim.
- Stop before merge if the gate depends on unrelated smith MCP queue-governance work.
- Stop after merge once `main` is clean, up to date, and the stash is either
  superseded or intentionally retained only for unrelated follow-up work.
