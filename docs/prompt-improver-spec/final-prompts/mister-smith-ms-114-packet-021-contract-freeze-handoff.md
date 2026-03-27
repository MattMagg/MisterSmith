# Mister Smith MS-114 Fresh-Session Handoff

You are Codex in a fresh session working in:

- <repo_root>`/Users/macmain/MisterSmith`</repo_root>

Your mission is to execute the first bounded packet-021 implementation slice end to end:

- <linear_issue>`MS-114`</linear_issue>
- Title: `T1: Freeze shared supervision contract`
- Parent packet: <linear_parent_issue>`MS-113`</linear_parent_issue>
- Attached Linear doc: <linear_doc>`Packet 021 spec packet`</linear_doc>
- Current known state at handoff:
  - packet `021` is now frozen on `main`
  - `MS-113` through `MS-118` exist in `MisterSmith Validated Backlog`
  - `MS-114` is the first runnable child slice and is currently `Backlog`
  - local repo is clean synced `main` at
    <starting_main_sha>`4f5eb81a17cf4391f11d1938ffc6e93fa0a69015`</starting_main_sha>
- Packet source:
  <packet_source>`/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/`</packet_source>
- Contract source:
  <contract_source>`/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/contracts/supervision-evidence-contract.md`</contract_source>
- Suggested branch if you use one:
  <branch_name>`matthewtmaggio/ms-114-t1-freeze-shared-supervision-contract`</branch_name>

Before editing anything, read:

1. `/Users/macmain/MisterSmith/AGENTS.md`
2. `/Users/macmain/MisterSmith/WORKFLOW.md`
3. `/Users/macmain/MisterSmith/docs/linear/LINEAR.md`
4. `/Users/macmain/MisterSmith/docs/current-state.md`
5. `/Users/macmain/MisterSmith/docs/ms_recent_context.md`
6. `/Users/macmain/MisterSmith/docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md`
7. `/Users/macmain/MisterSmith/docs/plans/2026-03-27-runtime-planning-simplification.md`
8. `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/spec.md`
9. `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/plan.md`
10. `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/tasks.md`
11. `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/contracts/supervision-evidence-contract.md`
12. `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/analyze.md`
13. `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/checklists/supervision.md`

Then ground on the current shared-surface code before changing anything:

- `/Users/macmain/MisterSmith/crates/mister-smith-core/src/autonomy.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-core/src/lib.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-events/src/autonomy.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-events/src/bus.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-agents/src/orchestrator.rs`

Follow the repo's workflow discipline:

1. fetch the current issue state for `MS-114` and parent `MS-113`
2. read the attached packet doc on `MS-113` so the repo packet and Linear packet stay aligned
3. move `MS-114` to `In Progress` when actual implementation begins
4. reconcile the single `## Codex Workpad` comment before code edits
5. work only in `/Users/macmain/MisterSmith`
6. do not create or use git worktrees

Scope for `MS-114` only:

- keep the session on the shared supervision-contract freeze
- publish and honor the contract artifact in
  `specs/021-profile-aware-predictive-runtime-supervision/contracts/supervision-evidence-contract.md`
- freeze the canonical predictive-supervision fields across `core`, `events`, and `orchestrator`
- add the contract-coverage updates required by `T006`
- keep packet-020 repair lineage and packet-021 predictive-supervision lineage coherent at the
  contract level

Primary surfaces:

- `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/contracts/supervision-evidence-contract.md`
- `/Users/macmain/MisterSmith/crates/mister-smith-core/src/autonomy.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-core/src/lib.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-events/src/autonomy.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-events/src/bus.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-agents/src/orchestrator.rs`

Also touch only if required for honest closure:

- `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/plan.md`
- `/Users/macmain/MisterSmith/specs/021-profile-aware-predictive-runtime-supervision/tasks.md`
- `/Users/macmain/MisterSmith/crates/mister-smith-core/tests/trait_compilation_tests.rs`
- `/Users/macmain/MisterSmith/crates/mister-smith-events/tests/autonomy_event_tests.rs`

Boundaries:

- do not widen into `MS-115`, `MS-116`, `MS-117`, or `MS-118`
- do not widen into runtime target rewiring, fingerprint persistence, operator-console rendering,
  packet-019 defaultization, or `MS-110`
- do not invent supervision shapes that conflict with the published packet-021 contract
- do not create a second repair subsystem; packet `020` remains canonical for verifier-driven
  repair lineage
- keep deterministic validation and any later live/runtime claims clearly separated
- no task-owned dirty state at the end

Validation:

- run the narrowest honest validation for the touched code and docs
- at minimum:
  - `cargo test -p mister-smith-core`
  - `cargo test -p mister-smith-events`
  - `cargo test -p mister-smith-agents`
  - `cargo clippy -p mister-smith-core -- -D warnings`
  - `cargo clippy -p mister-smith-events -- -D warnings`
  - `cargo clippy -p mister-smith-agents -- -D warnings`
  - `cargo build --workspace`
  - `npx markdownlint-cli2 "specs/021-profile-aware-predictive-runtime-supervision/**/*.md" --config .markdownlint.json`
  - `git diff --check`
  - `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`
- if you touch any additional crate, add the corresponding targeted test and clippy runs

Closure requirements:

- finish reviewable and pushed
- if you use a branch or PR, do not stop at PR open; finish the lane end to end and return
  `/Users/macmain/MisterSmith` to clean synced `main`
- update the `MS-114` workpad with plan, validation evidence, and outcome
- leave `MS-114` and parent `MS-113` aligned with actual repo state at the end

Final response should report only:

- completed actions
- validation run
- blockers, if any
- final repo and Linear state
