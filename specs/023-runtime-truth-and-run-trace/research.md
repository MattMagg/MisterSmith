# Research Notes: Runtime Truth And Run Trace

## Current repo truth

- packet `019` remains the last bounded live-proof note for runtime routing on the supported path
- packet `020` remains the last bounded live-proof note for repair lineage and orchestration
  quality on the supported path
- packet `021` is landed on `main` with deterministic validation for predictive-supervision
  evidence and explicit proof-boundary wording
- packet `022` is landed on `main` with deterministic validation for durable workflow history,
  lifecycle projection, effect-boundary records, and bounded compaction
- `WorkflowStepTool` still echoes the incoming payload, adds `status=completed`,
  `execution_boundary=tool_bus`, and `tool_name=workflow.execute_step`, but does not prove
  grounded task work
- current task, session, autonomy, and operator surfaces do not yet share one packet-owned runtime
  truth contract separate from packet `021`

## Primary decisions

### Decision: packet `023` owns a new runtime-truth contract

**Rationale**: Predictive supervision and truthful execution proof are related but distinct.
Packet `023` needs a new shared block instead of stretching packet `021`
`supervision_evidence` past its current meaning.

**Alternatives considered**:

- Reuse `supervision_evidence`: rejected because it would blur packet ownership and make surfaces
  harder to read.
- Put the new truth fields inside `UnifiedResultEnvelope` only: rejected because task, session,
  autonomy, and operator projections need a typed surface contract, not more nested JSON guessing.

### Decision: packet `023` stays deterministic-only unless a new live rerun is actually captured

**Rationale**: Current repo proof supports deterministic validation for the new truth surface, not
an additional live runtime claim.

**Alternatives considered**:

- Treat the earlier packet `019` and packet `020` live proof as enough to declare packet `023`
  live: rejected because it overstates what this packet itself validates.
- Hide the split behind vague wording such as "partially live": rejected because it weakens the
  proof boundary packet `023` is meant to sharpen.

### Decision: run trace is a bounded summary, not a tracing platform

**Rationale**: The repo already has `workflow_id`, graph metadata, repair lineage, supervision
  state, and `trace_id`, but it does not yet have a full tracing platform. Packet `023` should
  summarize the relationships the runtime already knows instead of pretending that a span tree
  exists.

**Alternatives considered**:

- Add full span emission and transport propagation now: rejected because that is generic
  observability work outside packet scope.
- Ignore external tracing standards entirely: rejected because the packet still needs coherent
  naming and relationship vocabulary.

### Decision: transport schema stays unchanged in the first slice

**Rationale**: The current gap is naming and projection, not missing wire fields. Existing
`workflow_id` and `trace_id` are enough input for the bounded first slice.

**Alternatives considered**:

- Add packet-023 fields to `MessageEnvelope`: rejected because it widens transport schema before the
  runtime-truth contract itself is stable.

## Guidance from current proof notes

- Use `docs/plans/2026-03-19-session-restart-resume-live-proof.md` for the current live proof of
  session continuity and supported-path runtime execution.
- Use `docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md` for the current
  deterministic supervision-evidence boundary on `main`.
- Use `docs/plans/2026-04-01-packet-022-durable-workflow-core.md` for packet `022` ownership of
  durable lifecycle and history semantics.
- Use `docs/current-state.md` for the explicit live-versus-deterministic split on current `main`.

## External taxonomy guidance

- OpenTelemetry traces are the best reference for trace-root, event, and relationship naming.
- OpenTelemetry context propagation is the best reference for how trace identity moves across
  boundaries.
- W3C Trace Context is the best reference for stable wire-level trace correlation naming.

These sources guide naming and relationship shape only. They do not upgrade current repo proof.

## Bounded conclusion

Packet `023` is a truth-contract packet. Its job is to say what a run proved, what it did not
prove, and how that story should be projected consistently across existing task, session,
autonomy, and operator surfaces without widening into transport schema work, generic
observability, or packet-022 lifecycle semantics.
