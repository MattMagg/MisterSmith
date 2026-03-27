# Tasks: Verifier-Gated Adaptive Orchestration

## Status Reconciliation

- packet `019` is complete on `main`
- the current runtime path already ships supervised planner/executor execution and task/autonomy
  provenance
- packet `020` is now landed on `main` through `MS-104` through `MS-107`
- the runtime-backed task path now includes verifier-gated workflow-step control, first-class
  clarification, preserved failure-context repair lineage, and orchestration-quality provenance

## T1. Freeze packet and state-doc routing

- [x] Add packet `020` artifacts
- [x] Add repo planning note in `docs/plans/`
- [x] Update current router docs to point at this packet as the next bounded phase

## T2. Add verifier-gated workflow-step contract

- [x] Define verifier verdict and repair directive entities in `crates/mister-smith-core/src/`
- [x] Wire step acceptance and rejection gating into
      `crates/mister-smith-app/src/execution.rs`
- [x] Add targeted coverage for accept, reject, and disabled-policy fallback behavior

## T3. Add bounded clarification and contextual repair

- [x] Add a first-class clarification request path for weak handoffs
- [x] Preserve failure context and last stable checkpoint for retry or re-plan
- [x] Add targeted coverage for clarification loops, retry budgets, and checkpoint-based repair

## T4. Surface orchestration-quality provenance

- [x] Extend task and autonomy inspection with verifier verdict, repair action, and stable
      checkpoint lineage
- [x] Add or extend coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- [x] Refresh proof guidance or deterministic transcript notes with honest boundaries

## T5. Final validation and docs sync

- [x] Run targeted Rust tests and clippy for touched crates
- [x] Run markdownlint on touched docs and packet files
- [x] Run `git diff --check`
- [x] Run `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`
