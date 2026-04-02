# 2026-04-01 Packet 022 Durable Workflow Core

## Summary

Packet `022` is landed on `main` in commit `60f4ba2` (`Packet 022: durable workflow core (#260)`).
The repo now carries one bounded durable-workflow core across the current runtime path:

- canonical accepted workflow history
- replay-safe workflow reconstruction
- durable lifecycle verbs and lifecycle-state projection
- persistence-owned effect-boundary intent and outcome records
- bounded lineage-preserving compaction

## Closure Boundary

- packet `022` is limited to durable history, replay-safe state transitions, lifecycle verbs,
  effect boundaries, and bounded compaction
- `POST /api/v1/tasks/{task_id}/lifecycle` is now the bounded HTTP control entrypoint for durable
  lifecycle commands
- task, session, autonomy, and HTTP task views now project one shared durable lifecycle meaning
- lifecycle decisions are recorded durably with `applied`, `noop`, or `deferred` outcomes
- packet `022` does **not** claim live runner pause, resume, cancel, or terminate control just
  because those durable lifecycle decisions are now recorded and projected
- packet `022` does **not** widen into coordinator-runtime expansion, interoperability, or strong
  coordination work
- this note records deterministic validation only; it does not claim a new packet-022 live rerun

## What Changed

Changed runtime and persistence surfaces:

- `crates/mister-smith-core/src/enums.rs`
- `crates/mister-smith-agents/src/branch_checkpoint.rs`
- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-http/src/routes.rs`
- `crates/mister-smith-http/src/handlers.rs`
- `crates/mister-smith-http/src/server.rs`
- `crates/mister-smith-persistence/src/hybrid/manager.rs`
- `crates/mister-smith-persistence/src/kv/state.rs`
- `crates/mister-smith-persistence/src/repository/task.rs`

Changed packet docs:

- `specs/022-durable-workflow-core/spec.md`
- `specs/022-durable-workflow-core/design.md`
- `specs/022-durable-workflow-core/data-model.md`
- `specs/022-durable-workflow-core/contracts/durable-workflow-contract.md`
- `specs/022-durable-workflow-core/tasks.md`
- `specs/022-durable-workflow-core/quickstart.md`
- `specs/022-durable-workflow-core/research.md`
- `specs/022-durable-workflow-core/plan.md`

## Deterministic Validation

These checks passed for the landed packet-022 merge:

```bash
cargo fmt --all
git diff --check
cargo build --workspace
cargo test -p mister-smith-core
cargo test -p mister-smith-agents
cargo test -p mister-smith-persistence
cargo test -p mister-smith-app
cargo test -p mister-smith-events
cargo test -p mister-smith-http
cargo clippy -p mister-smith-core -- -D warnings
cargo clippy -p mister-smith-http -- -D warnings
cargo clippy -p mister-smith-agents -- -D warnings
cargo clippy -p mister-smith-persistence -- -D warnings
cargo clippy -p mister-smith-app -- -D warnings
cargo clippy -p mister-smith-events -- -D warnings
npx markdownlint-cli2 "specs/022-durable-workflow-core/**/*.md" --config .markdownlint.json
scripts/verify_worktree_closure.sh --fetch --require-upstream --require-sync
```

These checks prove the bounded durable-history contract, lifecycle projection, effect-boundary
tracking, compaction lineage, HTTP lifecycle handler, and packet-doc truth sync on `main`.

## Live-Proof Boundary

No new packet-022 live runtime proof is claimed here.

Current honest boundary:

- packet `022` is landed and deterministically validated on the current runtime/task/session/
  autonomy surfaces
- the supported-path live baseline still comes from the earlier runtime-proof family rather than a
  new packet-022-owned rerun
- durable lifecycle projection is live in the code and API surfaces, but full live runner pause,
  resume, cancel, and terminate control remains deferred beyond this packet
