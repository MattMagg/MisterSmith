# 2026-03-15 Stash Reconciliation Cleanup

## Objective

- Reconcile the remaining local stash onto current `main` without replaying stale
  Phase 10 gate edits.
- Land the still-live smith MCP queue-governance source changes and preserve any
  historical notes from the stash that remain useful.
- Remove the stash only after the restored content is committed and pushed.

## Scope

- `crates/mister-smith-mcp/src/compatibility.rs`
- historical queue/throughput notes under `docs/plans/`
- stash hygiene and final repo cleanliness on `main`

## Assumptions

- `main` at `4f26d0e` is the authoritative repo baseline.
- The remaining stash content is dominated by the smith MCP queue-governance patch
  plus dated planning notes.
- The old Phase 10 SpecKit audit note is historical context only and must not
  override the merged `MS-35` closeout record.

## Constraints

- Do not replay the stash wholesale; current `main` has already absorbed the Phase
  10 gate and later fixes.
- Preserve reversibility with an adjacent backup before overwriting tracked source.
- Validate the affected crate/build scope before pushing and before dropping the
  stash.

## Non-Goals

- Do not reopen the Phase 10 gate work or revert `main` to the stash baseline.
- Do not touch unrelated runtime/workflow state outside the restored queue
  governance work.

## Milestones

### 1. Isolate live stash content

- Confirm which stash changes are still missing from `main`.
- Separate queue-governance source and historical notes from superseded Phase 10
  gate artifacts.

### 2. Restore and validate

- Restore `compatibility.rs` and the queue/throughput notes.
- Mark the older Phase 10 audit note as superseded historical context.
- Run narrow validation for the MCP crate plus a cross-crate build check.

### 3. Clean and finish

- Commit and push the reconciled stash content on `main`.
- Drop the stash and verify the repo is clean with no extra worktrees or open PRs.

## Status

- **Current milestone**: 3. Clean and finish
- **Completed work**:
  - Confirmed `main` at `4f26d0e` was clean before reconciliation.
  - Verified the stash could not be popped wholesale because it predates the merged
    Phase 10 gate and later `main` fixes.
  - Restored the still-live smith MCP queue-governance patch in
    `crates/mister-smith-mcp/src/compatibility.rs`.
  - Restored the dated queue/throughput notes from the stash and marked the older
    Phase 10 audit note as historical context rather than current authority.
  - Validated the restored scope with:
    - `cargo test -p mister-smith-mcp`
    - `cargo build --workspace`
    - `npx markdownlint-cli2 "docs/plans/2026-03-15-ms-33-dispatch-and-phase10-next-queue-prep.md" "docs/plans/2026-03-15-symphony-throughput-ramp.md" "docs/plans/2026-03-15-phase10-spec-kit-refresh-and-audit.md" "docs/plans/2026-03-15-stash-reconciliation-cleanup.md" --config .markdownlint.json`
    - `git diff --check`
