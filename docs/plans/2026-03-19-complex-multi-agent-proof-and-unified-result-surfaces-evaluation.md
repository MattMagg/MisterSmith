# Complex Multi-Agent Proof And Unified Result Surfaces Evaluation Matrix

Date: March 19, 2026
Status: Active
Packet: `specs/015-complex-multi-agent-proof-and-unified-result-surfaces/`
Tasks: `T006` to `T008`

## Objective

Freeze one pre-runtime proof outcome matrix for packet 015 before harder workload proof-path work
starts.

This note is the durable reference for the only allowed packet-level proof outcome classes:

- `graph_formed_and_completed`
- `collapsed_to_sequential`
- `failed_before_graph`

## Baseline Authority

- `specs/015-complex-multi-agent-proof-and-unified-result-surfaces/spec.md`
- `specs/015-complex-multi-agent-proof-and-unified-result-surfaces/data-model.md`
- `docs/plans/2026-03-19-short-multi-agent-result-evaluation.md`
- `docs/plans/2026-03-19-framework-comparison-stress-test.md`

## Frozen Matrix

### `graph_formed_and_completed`

- Use when a real graph formed and the workflow reached terminal completion.
- Minimum stored evidence: `graph.state = Completed`, non-trivial graph evidence, and a
  non-collapsed topology or task shape.
- Representative March 19 source:
  `docs/plans/2026-03-19-short-multi-agent-result-evaluation.md`

### `collapsed_to_sequential`

- Use when the workflow completed, but the planner compressed the workload to a trivial
  sequential path.
- Minimum stored evidence: `graph.state = Completed`, `topology_kind = Sequential`,
  `parallelism_width <= 1`, and task-shape evidence that the workload asked for more than a
  strict chain.
- Representative March 19 source:
  `docs/plans/2026-03-19-framework-comparison-stress-test.md` trimmed benchmark

### `failed_before_graph`

- Use when the workflow failed or aborted before usable graph evidence existed.
- Minimum stored evidence: terminal failure plus no visible graph evidence such as branch, node,
  or routing history.
- Representative March 19 source:
  `docs/plans/2026-03-19-framework-comparison-stress-test.md` heavy benchmark

## Boundary Rules

1. This slice freezes the packet proof matrix only. It does not claim to classify every possible
   runtime failure.
2. `failed_before_graph` is reserved for failures where stored evidence does not show usable graph
   formation.
3. A run that already exposed a visible graph and then failed remains outside this matrix for now.
   Later runtime-proof work may define how to surface that case, but it must not invent a fourth
   packet-level proof outcome during this checkpoint.
4. `collapsed_to_sequential` still counts as a completed workflow. It is distinct from both full
   graph success and planner-time failure.

## Operator-Facing Interpretation

- `graph_formed_and_completed` means the stored evidence proves meaningful graph formation and
  terminal completion.
- `collapsed_to_sequential` means the system completed, but the proof packet must treat the run as
  a visible collapse rather than a frontier multi-agent success.
- `failed_before_graph` means the proof packet captured an honest planner-time or pre-graph
  failure boundary.

## Expected Code Contract

- `crates/mister-smith-core/src/autonomy.rs` is the source of truth for the frozen enum labels.
- `crates/mister-smith-events/src/autonomy.rs` may infer one of the three classes only when the
  typed autonomy projection matches the matrix rules above.
- `crates/mister-smith-events/tests/autonomy_event_tests.rs` must cover all three matrix outcomes
  and the intentional out-of-matrix visible-graph failure boundary.

## Stop Conditions

- Do not add a fourth proof outcome class in this packet.
- Do not widen this note into runtime implementation work, result projection work, or provider
  policy changes.
- Do not reinterpret the March 19 evidence notes as stronger than their stored proof.
