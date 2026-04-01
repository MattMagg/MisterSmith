# Research Notes: Runtime Truth And Run Trace

## Current repo truth

- packet `019` provides the last bounded live-proof note for budget-aware runtime routing on the
  supported path
- packet `020` provides the last bounded live-proof note for orchestration-quality and repair
  lineage on the supported path
- packet `021` provides landed supervision-evidence projection with deterministic validation and
  explicit proof-boundary language, but it does not by itself create a new supported-path live
  proof claim
- the March 19 live run-trace evaluation and March 28 session context report both document the
  same honesty gap: graph completion and tool-bus step completion can appear semantically richer
  than they really are
- `WorkflowStepTool` currently echoes the incoming payload, adds `status=completed`,
  `execution_boundary=tool_bus`, and `tool_name=workflow.execute_step`, and returns that payload
  without grounded task execution proof

## Primary decisions

### Decision: packet `023` should freeze proof-boundary and trace taxonomy, not durable lifecycle

**Rationale**: The packet-prep README and packet `023` dossier place packet `023` after packet
`022`, with packet `022` still owning durable workflow semantics, history, compaction, and effect
boundaries.

**Alternatives considered**:

- Let packet `023` also freeze lifecycle semantics: rejected because it would steal packet `022`
  ownership and widen scope.
- Delay packet `023` entirely until packet `022` is done: rejected for this scaffold because the
  user wants speed later, not immediate implementation.

### Decision: the scaffold must preserve the current proof split exactly

**Rationale**: `docs/current-state.md`, the packet `021` closure note, and the packet `021` live
evaluation note all show that packet `019` and `020` remain the last fresh live-proof baseline,
while packet `021` is landed and deterministically validated for its newer claim surface.

**Alternatives considered**:

- Call all three packets “live enough”: rejected because it overstates current proof.
- Collapse the distinction into “partial”: rejected because it hides the real boundary.

### Decision: external tracing docs are taxonomy references only

**Rationale**: OpenTelemetry traces, context propagation, and W3C Trace Context provide the best
shape for naming roots, links, and propagation, but the dossier explicitly forbids treating those
docs as proof that the repo already emits a complete span model.

**Alternatives considered**:

- Import external tracing vocabulary as if it already matched repo behavior: rejected because it
  would create fake observability claims.
- Ignore external tracing docs entirely: rejected because the packet still needs a coherent
  taxonomy baseline.

### Decision: placeholder completion must remain a first-class non-grounded evidence class

**Rationale**: The March 28 session report and current `WorkflowStepTool` implementation make the
truth gap explicit. This packet must freeze language that can say a run completed at the workflow
graph layer while still saying semantic completion is unproven.

**Alternatives considered**:

- Treat placeholder tool-bus completion as semantic task proof: rejected because current code does
  not support that claim.
- Hide the placeholder boundary behind generic “success” wording: rejected because it misleads the
  operator.

## Guidance from current proof notes

- Use `docs/plans/2026-03-19-live-run-trace-evaluation.md` as the main live-gap anchor for why
  graph success is not enough.
- Use `docs/2026-03-28-session-context-report.md` for the conservative wording that packet `023`
  should freeze.
- Use `docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md` for deterministic
  proof-boundary language.
- Use `docs/plans/2026-03-29-packet-021-live-evaluation.md` for the explicit “deterministic yes,
  fresh live default-path proof no” split.

## External taxonomy guidance

- OpenTelemetry traces are the best reference for span-like causal structure, events, and links.
- OpenTelemetry context propagation is the best reference for how trace identity moves across
  boundaries.
- W3C Trace Context is the best reference for wire-level trace correlation semantics.

These sources guide naming and relationship shape only. They do not upgrade current repo proof.

## Bounded conclusion

Packet `023` is a truth-contract packet. Its honest job is to say what a run proved, what it did
not prove, and how that story should be named consistently across existing result and operator
surfaces. It should not become a packet about new runtime behavior, durable lifecycle semantics,
UI redesign, or generic observability tooling.
