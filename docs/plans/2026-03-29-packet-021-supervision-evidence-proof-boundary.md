# 2026-03-29 Packet 021 Supervision Evidence Proof Boundary

## Summary

`MS-117` lands the bounded packet-021 projection work on the existing task inspect path. The
runtime now persists one first-class `supervision_evidence` projection on `task.result` while
keeping packet-020 `orchestration_quality` separate.

Changed surfaces:

- `crates/mister-smith-core/src/autonomy.rs`
- `crates/mister-smith-core/tests/trait_compilation_tests.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/tests/autonomy_status_tests.rs`
- `apps/operator-console/src/types.ts`
- `apps/operator-console/src/views/RunsView.tsx`
- `apps/operator-console/src/App.test.tsx`

## Closure Boundary

- no new endpoint was added
- `GET /api/v1/tasks/{task_id}` remains the selected-run detail source of truth
- packet-020 `orchestration_quality` remains unchanged and separate from packet-021 supervision
  evidence
- failed-before-graph synthesis still carries no packet-021 supervision claim
- this note records deterministic validation only; it does not claim a new live runtime rerun

## What Changed

- `TaskResultView` now exposes `supervision_evidence: Option<SupervisionEvidenceView>`
- runtime terminal result serialization now pulls supervision evidence from the live autonomy view
  or the preserved autonomy snapshot through one shared recovery helper
- restart-recovered failure paths now preserve stored packet-021 evidence instead of risking
  overwrite by a synthesized `None`
- the operator console renders one bounded predictive-supervision panel from the existing inspect
  payload, including target scope, decision basis, fingerprint, repair lineage, and proof boundary

## Deterministic Validation

These deterministic checks passed for `MS-117`:

```bash
cargo test -p mister-smith-core
cargo test -p mister-smith-events --test autonomy_event_tests
cargo test -p mister-smith-app --test autonomy_status_tests
npm --prefix apps/operator-console test
```

These checks prove the bounded result-contract, event-projection, runtime-rendering, and
operator-console wiring. They do not prove a fresh live runtime rerun.

## Live-Proof Boundary

No new packet-021 live runtime proof is claimed here.

Current honest boundary:

- packet-021 supervision evidence is deterministically validated on the landed runtime/task
  inspect/result surfaces
- any claim that a real live rerun emitted this projection on the supported provider path still
  requires a separate runtime-proof artifact
