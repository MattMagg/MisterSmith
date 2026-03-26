# Tasks: Verifier-Gated Adaptive Orchestration

## Status Reconciliation

- packet `019` is complete on `main`
- the current runtime path already ships supervised planner/executor execution and task/autonomy
  provenance
- no verifier-gated workflow-step control loop, clarification path, or contextual repair contract
  is yet landed on the runtime-backed task path

## T1. Freeze packet and state-doc routing

- [x] Add packet `020` artifacts
- [x] Add repo planning note in `docs/plans/`
- [x] Update current router docs to point at this packet as the next bounded phase

## T2. Add verifier-gated workflow-step contract

- [ ] Define verifier verdict and repair directive entities in `crates/mister-smith-core/src/`
- [ ] Wire step acceptance and rejection gating into
      `crates/mister-smith-app/src/execution.rs`
- [ ] Add targeted coverage for accept, reject, and disabled-policy fallback behavior

## T3. Add bounded clarification and contextual repair

- [ ] Add a first-class clarification request path for weak handoffs
- [ ] Preserve failure context and last stable checkpoint for retry or re-plan
- [ ] Add targeted coverage for clarification loops, retry budgets, and checkpoint-based repair

## T4. Surface orchestration-quality provenance

- [ ] Extend task and autonomy inspection with verifier verdict, repair action, and stable
      checkpoint lineage
- [ ] Add or extend coverage in
      `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- [ ] Refresh proof guidance or deterministic transcript notes with honest boundaries

## T5. Final validation and docs sync

- [ ] Run targeted Rust tests and clippy for touched crates
- [ ] Run markdownlint on touched docs and packet files
- [ ] Run `git diff --check`
- [ ] Run `scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync`
