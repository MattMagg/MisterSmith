# Mister Smith MS-106 Fresh-Session Handoff

You are Codex in a fresh session working in:

- <repo_root>`/Users/macmain/MisterSmith`</repo_root>

Your mission is to execute the next bounded packet-020 implementation slice end to end:

- <linear_issue>`MS-106`</linear_issue>
- Title: `T4: Surface orchestration-quality provenance`
- Current known state at handoff:
  - `MS-105` is landed and `Done`
  - local repo is clean synced `main` at <starting_main_sha>`b9ac765cd53c680685752ecd1413135712d7e1a7`</starting_main_sha>
  - `MS-106` is currently known as `Backlog` in `MisterSmith Validated Backlog`
- Parent packet: `MS-103`
- Packet source: <packet_source>`/Users/macmain/MisterSmith/specs/020-verifier-gated-adaptive-orchestration/`</packet_source>
- Suggested branch if you use one: <branch_name>`matthewtmaggio/ms-106-t4-surface-orchestration-quality-provenance`</branch_name>

Before editing anything, read:

1. `/Users/macmain/MisterSmith/AGENTS.md`
2. `/Users/macmain/MisterSmith/WORKFLOW.md`
3. `/Users/macmain/MisterSmith/docs/linear/LINEAR.md`
4. `/Users/macmain/MisterSmith/docs/current-state.md`
5. `/Users/macmain/MisterSmith/docs/ms_recent_context.md`
6. `/Users/macmain/MisterSmith/docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md`
7. `/Users/macmain/MisterSmith/specs/020-verifier-gated-adaptive-orchestration/spec.md`
8. `/Users/macmain/MisterSmith/specs/020-verifier-gated-adaptive-orchestration/plan.md`
9. `/Users/macmain/MisterSmith/specs/020-verifier-gated-adaptive-orchestration/tasks.md`

Then ground on the already-landed T2/T3 surfaces before changing the operator-facing views:

- `/Users/macmain/MisterSmith/crates/mister-smith-app/src/execution.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-core/src/autonomy.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-core/src/supervision.rs`

Follow the Smith-first workflow:

1. fetch the current issue/control-plane state for `MS-106`
2. if `MS-106` is still in validated backlog and missing watched-queue staging, add the needed
   label/state transitions through the Smith control plane instead of skipping lifecycle steps
3. move `MS-106` to `In Progress`
4. reconcile the single `## Codex Workpad` comment before implementation
5. work only in `/Users/macmain/MisterSmith`
6. do not create or use git worktrees

Scope for `MS-106` only:

- extend task and autonomy inspection with verifier verdict, repair action, and stable checkpoint
  lineage
- add or extend coverage in
  `/Users/macmain/MisterSmith/crates/mister-smith-app/tests/autonomy_status_tests.rs`
- refresh proof guidance or deterministic transcript notes only as needed to keep boundaries honest

Primary surfaces:

- `/Users/macmain/MisterSmith/crates/mister-smith-app/src/autonomy.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-app/src/agent_inspection.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-app/tests/autonomy_status_tests.rs`

Also touch only if required for honest closure:

- `/Users/macmain/MisterSmith/docs/current-state.md`
- `/Users/macmain/MisterSmith/docs/ms_recent_context.md`
- packet `020` quickstart or proof notes under
  `/Users/macmain/MisterSmith/specs/020-verifier-gated-adaptive-orchestration/`

Boundaries:

- do not widen into provider work, budgeting work, benchmark harness work, UI redesign, or broad
  operator-surface churn
- do not invent provenance that is not actually present in the accepted or repaired workflow path
- keep deterministic validation and any live/runtime claims clearly separated
- if you discover follow-on work, record it as follow-up context, but do not silently expand scope
- no task-owned dirty state at the end

Validation:

- run the narrowest honest validation for touched code and docs
- at minimum:
  - `cargo test -p mister-smith-app`
  - `cargo clippy -p mister-smith-app -- -D warnings`
  - `cargo build --workspace`
  - `git diff --check`
  - `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`
- if you touch `mister-smith-core`, run the corresponding core test/clippy commands too
- if you touch markdown files, run targeted markdownlint on the touched docs/spec files

Closure requirements:

- finish reviewable and pushed
- if you use a branch/PR, do not stop at PR open; finish the lane end to end and return
  `/Users/macmain/MisterSmith` to clean synced `main`
- update the `MS-106` workpad with plan, validation evidence, and outcome
- leave `MS-106` and repo state aligned at the end

Final response should report only:

- completed actions
- validation run
- blockers, if any
- final repo/Linear state
