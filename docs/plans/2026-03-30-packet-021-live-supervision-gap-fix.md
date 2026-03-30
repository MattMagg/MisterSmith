# 2026-03-30 Packet 021 Live Supervision Gap Fix

## Objective

Populate packet-021 `repair_lineage_ref` and `proof_boundary` on the supported live runtime path so
real autonomy/task payloads match the landed contract and proof-boundary note.

## Scope

- `crates/mister-smith-agents/src/orchestrator.rs`
- `crates/mister-smith-events/src/bus.rs`
- direct agent/event regression tests for live supervision evidence projection

## Assumptions

- `proof_boundary` should be explicit for the supported live task path
- `repair_lineage_ref` is only emitted when the runtime already has checkpoint lineage for the
  supervised target
- unrelated live-evaluation prep files in the primary checkout stay untouched

## Non-Goals

- no new packet or Linear reshaping
- no new endpoint or console redesign
- no fresh live rerun in this fix branch

## Milestones

1. Trace the live supervision builders and the existing packet-021 contract.
2. Patch the live builder path to project checkpoint-backed repair lineage and proof boundary.
3. Run targeted Rust validation for agents/events plus affected app regression coverage.

## Validation

- `cargo test -p mister-smith-agents`
- `cargo test -p mister-smith-events --test autonomy_event_tests`
- `cargo test -p mister-smith-app --test autonomy_status_tests`

## Stop Conditions

- Stop if the runtime lacks a trustworthy checkpoint source for packet-020 lineage.
- Stop if the fix requires widening packet scope beyond the live supervision projection path.
