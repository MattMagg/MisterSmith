# Phase 10 SpecKit Refresh and Spec Audit

> Historical note: this captures the pre-PR-190 / pre-`MS-35` reconciliation pass
> that originally lived only in the local stash. The authoritative closeout for the
> merged Phase 10 gate is
> `docs/plans/2026-03-15-ms-35-phase10-gate-and-speckit-refresh.md`.

## Objective

- Refresh the repository's SpecKit scaffolding to the latest upstream release.
- Audit and reconcile `specs/012-phase10-frontier-autonomy/` against `ROADMAP.md`, canonical
  architecture sources under `spec/`, the consolidated research corpus under
  `docs/research-output/`, and current repo implementation reality.

## Scope

- SpecKit project scaffolding under `.specify/` and Codex prompt files under `.codex/prompts/`
- Phase 10 SpecKit artifacts under `specs/012-phase10-frontier-autonomy/`
- Supporting alignment documents:
  - `ROADMAP.md`
  - `README.md`
  - `CLAUDE.md`
  - relevant `spec/` architecture sources
  - relevant `docs/research-output/consolidated/` findings
  - current Phase 10 implementation and planning notes under `docs/plans/`

## Assumptions

- Upstream SpecKit CLI and template release `0.3.0` is the current latest stable release as of 2026-03-15.
- The repo's existing `.specify/memory/constitution.md` is authoritative and must be preserved unless the review proves a real governance defect.
- Unrelated working-tree changes in `crates/mister-smith-mcp/src/compatibility.rs` and other untracked docs are out of scope and must remain untouched.

## Constraints

- Follow repo-local guidance before home-level defaults.
- Preserve reversibility with adjacent timestamp backups before overwriting existing tracked files.
- Keep Phase 10 grounded in supervised autonomy and control-plane leverage; do not let the refresh or audit drift into unrelated implementation work.
- Use targeted validation only; do not run `cargo test --workspace`.

## Non-goals

- Implement Phase 10 runtime code beyond documentation and artifact reconciliation
- Re-scope finished phases or reopen Phase 9 / 9.1 implementation without evidence
- Modify unrelated local worktree changes

## Milestones

### 1. Refresh SpecKit scaffolding

- Confirm the latest upstream SpecKit release and supported in-place upgrade path.
- Rehearse the upgrade in a temp copy to identify the exact file touch set.
- Back up affected files in-place, run the in-repo refresh, and verify the resulting diffs are limited to the expected scaffolding files.

**Validation**

- `uvx --from git+https://github.com/github/spec-kit.git specify version`
- temp-copy rehearsal of `specify init --here --force --ai codex --no-git`
- post-refresh diff review of `.specify/` and `.codex/prompts/`

### 2. Audit Phase 10 alignment

- Read `spec.md`, `plan.md`, `tasks.md`, `research.md`, `analyze.md`, `data-model.md`, `quickstart.md`, and relevant contracts/checklists.
- Cross-check those artifacts against `ROADMAP.md`, canonical architecture docs, consolidated research findings, and current repo/planning state.
- Record concrete mismatches, stale claims, and missing traceability.

**Validation**

- Direct file-to-file trace checks with absolute paths and line references captured in the working notes
- Current repo file existence and implementation-surface checks
- Frontier-legitimacy judgment kept aligned with supervised autonomy rather than drift

### 3. Reconcile the artifact set

- Update only the necessary Phase 10 docs and supporting repo docs to eliminate verified drift.
- Keep the spec/task/analyze set mutually consistent and explicit about any remaining implementation gaps.

**Validation**

- Targeted markdown diff review
- `rg` checks for stale paths, obsolete status claims, and missed cross-references
- `vet` run immediately after each logical edit unit

### 4. Close out with evidence

- Summarize what changed, what was validated, what remains unverified, and what follow-up work belongs in backlog rather than this docs pass.

**Validation**

- Final git diff review
- follow-up classification only if new actionable gaps remain after reconciliation

## Status

- **Current milestone**: Complete
- **Completed work**:
  - Inspected repo guidance, current worktree state, current Phase 10 artifact set, and related
    roadmap, architecture, research, and control-plane docs.
  - Verified upstream SpecKit CLI release `0.3.0` (`2026-03-13`) and rehearsed the supported
    refresh path before touching the repo.
  - Created adjacent timestamp backups, refreshed SpecKit in place, and confirmed the touch set is
    limited to `.specify/scripts/bash/*.sh`, `.specify/init-options.json`, and
    `.codex/prompts/speckit.*.md`, with `.specify/memory/constitution.md` preserved.
  - Reconciled `ROADMAP.md` plus the Phase 10 `spec.md`, `plan.md`, `research.md`, `tasks.md`,
    `analyze.md`, and checklist files so they all reflect current repo reality: 10.0-10.5 landed
    and validated, 10.6 still partial.
  - Ran targeted implementation validation covering events, agents, persistence, app surfaces, LLM
    monitoring, and deploy assets.
- **Decisions**:
  - Use the upstream refresh path instead of manual file cherry-picking.
  - Preserve the repo constitution and existing non-Codex agent files unless the refresh actually
    needs them.
  - Treat this as a documentation/spec reconciliation pass, not a Phase 10 implementation pass.
- **Open verification gap**:
  - `vet` did not complete in this environment. History-backed runs hit missing
    `ANTHROPIC_API_KEY` plus unsupported Codex history schema, `--no-history` required
    `OPENAI_API_KEY`, and the agentic Codex harness path hung without producing usable output.
- **Evidence**:
  - `uvx --from git+https://github.com/github/spec-kit.git specify version`
  - temp rehearsal in `/tmp/ms-specify-rehearsal.6XUR7Y`
  - `uvx --from git+https://github.com/github/spec-kit.git specify init --here --force --ai codex --no-git`
  - `cargo build --workspace`
  - `cargo test -p mister-smith-events --test autonomy_event_tests`
  - `cargo test -p mister-smith-agents --test execution_graph_tests --test topology_tests --test checkpoint_tests --test context_manager_tests --test guard_tests --test gate10_tests --test team_tests`
  - `cargo test -p mister-smith-persistence --test memory_manager_tests --test repository_tests --test performance_tests`
  - `cargo test -p mister-smith-app --test autonomy_status_tests`
  - `cargo test -p mister-smith-llm --test stream_monitor_tests`
  - `cargo test -p mister-smith-security --test delegation_tests`
  - `python3 scripts/validate_deploy_assets.py deploy/dashboards deploy/alerts`
  - targeted markdownlint pass showing the new Phase 10 docs are wrapped while older `ROADMAP.md`
    formatting debt remains
- **Next step**:
  - Optional follow-up only: finish Phase 10.6 delegation/provenance enforcement and rerun the
    final gate plus quickstart scenario.

## Stop Conditions

- The SpecKit refresh lands with bounded, reviewable diffs and preserved local constitution.
- Every verified Phase 10 spec misalignment found in this pass is either corrected or explicitly called out as a remaining risk.
- The final note states what changed, what was validated, and what follow-up remains.
