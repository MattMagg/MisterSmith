# MS-72 Step Routing Visibility And Evaluation

Date: March 18, 2026
Status: implemented and locally revalidated

## Objective

Expose why step-level routing changed through workflow-visible state and add one repeatable,
deterministic evaluation harness for step-level control decisions without changing routing
semantics.

## Scope

- project step-routing deltas into workflow-visible autonomy status
- publish live planner and critic step-routing updates into orchestrator autonomy snapshots
- persist step-routing history in workflow metadata during runtime planning
- compare representative step bundles with and without carryover control

## Assumptions

- `MS-70` already landed the raw `StepRoutingSignal` contract and carryover semantics
- deterministic fixture proof is the correct validation level for this slice
- workflow-visible state can be satisfied through live supervision snapshots plus persisted workflow
  metadata recovery
- persisted metadata must remain fallback-only when a fresher live status snapshot already carries
  step-routing history

## Constraints

- no new router policies, thresholds, or tier semantics
- no live-provider benchmarking
- keep branch-routing history intact and add step-routing visibility beside it

## Non-Goals

- new model-selection heuristics
- broad supervision/runtime redesign
- queue or lifecycle changes outside `MS-72`

## Projection Surface

The workflow-visible step-routing history now records, per step:

- `step_id`, `step_index`, `step_kind`
- `model_id`, `tier`, `reason`
- `previous_step_id`, `previous_action`, `previous_tier`
- `action`, `action_changed`, `preferred_tier_after`
- `estimated_cost_tokens`, `confidence_score`
- `triggered_checkpoints`
- `change_rationale`

The status renderer surfaces the new `step routing:` section beside the existing branch-routing
history, and planner or critic supervision now refreshes that section immediately when routing
control changes during live execution.
The app-facing status path now treats `metadata.step_routing_history` as a recovery fallback
instead of an unconditional overwrite, so live snapshots keep the freshest visible rationale.

## Deterministic Harness

Harness file:

- `crates/mister-smith-agents/tests/step_routing_benchmark_tests.rs`

Strategies compared:

- `stateless_baseline`: do not carry step-routing state between steps
- `adaptive_carryover`: apply the real `StepRoutingControl` carryover between steps

Metrics recorded:

- provider calls across the bundle
- triggered verification checkpoints across the bundle
- count of operator-visible action changes between steps

## Representative Bundle Results

| workload class | strategy | provider calls | triggered checkpoints | action changes | outcome |
| --- | --- | ---: | ---: | ---: | --- |
| `confidence_escalation_bundle` | `stateless_baseline` | 4 | 2 | 0 | baseline |
| `confidence_escalation_bundle` | `adaptive_carryover` | 3 | 1 | 1 | improved |
| `provider_failure_bundle` | `stateless_baseline` | 3 | 1 | 0 | baseline |
| `provider_failure_bundle` | `adaptive_carryover` | 3 | 1 | 1 | matched |

## Interpretation

- The confidence-escalation bundle shows a concrete reliability/resource improvement from step
  carryover: one fewer provider attempt and one fewer triggered checkpoint across the two-step
  bundle.
- The provider-failure bundle is intentionally recorded as a neutral match. After the first failing
  attempt, the router's circuit-breaker state already prevents another failing call, so carryover
  does not reduce provider calls further.
- Even in the matched case, the projected `change_rationale` remains useful because operators can
  inspect the step transition from `fallback` to `continue` instead of inferring it from raw router
  internals.
- The follow-up live snapshot publication fix keeps the same rationale visible before persistence,
  which closes the gap between in-flight supervision state and recovered workflow metadata.
- The app-side fallback fix closes the last stale-overwrite path: status rendering and persisted
  autonomy snapshots now preserve fresher live step-routing history and only use planning metadata
  when the live view has no step-routing entries yet.

## Validation

```bash
cargo test -p mister-smith-events --test autonomy_event_tests -- --nocapture
cargo test -p mister-smith-app --test autonomy_status_tests -- --nocapture
cargo test -p mister-smith-app \
  recover_persisted_autonomy_status_enriches_step_routing_history_from_metadata -- --nocapture
cargo test -p mister-smith-app \
  recover_persisted_autonomy_status_preserves_fresher_snapshot_history -- --nocapture
cargo test -p mister-smith-agents --features llm --test gate9_tests -- --nocapture
cargo test -p mister-smith-agents --features llm --test step_routing_benchmark_tests -- --nocapture
cargo build --workspace
```

## Stop Conditions

- stop before claiming routing visibility if step-level deltas are not visible in autonomy status
- stop before claiming benchmark improvement if the deterministic harness cannot reproduce the same
  bundle metrics across repeated runs
- stop before widening into new routing semantics; `MS-72` is visibility and proof only
