# Research Notes: Step-Level Intelligence v2

## Current repo truth

- packet `020` already landed verifier-gated step decisions, clarification handling, and repair
  lineage on the runtime-backed task path
- packet `021` already landed bounded supervision evidence on task, autonomy, and operator-facing
  surfaces
- packet `022` already landed durable workflow lifecycle, history, compaction, and effect-boundary
  ownership on current `main`
- packet `023` already landed the shared `runtime_truth` contract, run-trace summary, and
  proof-boundary wording across task, session, autonomy, and operator surfaces
- packet `024` already landed least-privilege external capability enforcement, quarantine reasons,
  and auth-callout fallback clamping on current `main`
- current packet-025 seams already expose the raw inputs needed for a first deterministic slice:
  `StepEvaluationRecord`, `StepRoutingDecisionSummary`, `ContextPressureSummary`,
  `TeamSizingDecision`, `RuntimeTruthView`, `SupervisionEvidenceView`, and the existing result
  projections

## Decision 1: First slice uses landed internal step signals, not a new stream parser

**Decision**: Packet `025` should consume the landed internal step, routing, budget, supervision,
and runtime-truth seams instead of introducing a new raw streaming-event parser in the first
implementation slice.

**Rationale**:

- the current runtime already records `StepEvaluationRecord`
- step-routing history is already projected through workflow-visible state
- packet `023` already owns the runtime-truth and proof-boundary story
- a new raw event parser would widen scope before the packet has frozen a useful deterministic
  policy surface

**Alternatives considered**:

- using OpenAI Responses event taxonomy as the first implementation input: rejected because the
  landed internal seams are already the current repo authority for step-level decisions
- inventing repo-local ad hoc step signals outside the landed seams: rejected because it would
  compete with the current runtime truth

## Decision 2: Keep the first slice heuristic and deterministic

**Decision**: The first packet-025 implementation slice should use deterministic heuristics and
bounded policy rules rather than PRM, judge-heavy, or training-heavy control loops.

**Rationale**:

- the current repo already has useful verifier, routing, budget, and supervision seams to build on
- deterministic policy is easier to audit and keeps the proof boundary honest
- the external research corpus remains useful directionally, but it is not required for the first
  bounded implementation

**Alternatives considered**:

- PRM-backed scoring from day one: rejected because it widens scope into training or model-program
  work
- benchmark-first work: rejected because packet `025` is a policy packet, not a benchmark packet

## Decision 3: Packet `023` remains the owner of proof wording

**Decision**: Packet `025` should consume packet-023 runtime-truth and proof-boundary fields and
never create a competing proof schema.

**Rationale**:

- packet `023` already froze the placeholder-versus-grounded wording
- packet `025` is about step scoring and action policy, not about redefining run truth
- keeping ownership boundaries clean prevents proof claims from drifting across packets

**Alternatives considered**:

- embedding packet-025-specific proof wording in step policy: rejected because it silently
  duplicates packet-023 scope
- treating step-policy summaries as self-sufficient proof: rejected because the repo already
  records why placeholder completion must stay explicit

## Decision 4: Packet `022` and packet `024` stay upstream ownership layers

**Decision**: Packet `025` may read packet-022 durable lifecycle state and packet-024 boundary
decisions when they affect step-policy presentation, but it must not absorb those ownership
domains.

**Rationale**:

- packet `022` already owns lifecycle, history, compaction, and effect-boundary semantics
- packet `024` already owns capability boundary, quarantine, sandbox, and auth-callout posture
- packet `025` only needs those upstream layers as policy inputs or scope boundaries

**Alternatives considered**:

- widening packet `025` into durable lifecycle semantics: rejected because packet `022` already
  owns that contract
- widening packet `025` into security-boundary policy: rejected because packet `024` already owns
  that contract

## Decision 5: Existing result surfaces stay canonical

**Decision**: Packet `025` should project step-policy summaries through the existing task,
session, autonomy, and operator-facing result surfaces rather than a new endpoint.

**Rationale**:

- those are already the current runtime truth surfaces on `main`
- the packet goal is stronger step policy, not a new observability product
- using current surfaces reduces scope and keeps router docs honest

**Alternatives considered**:

- a new packet-owned endpoint: rejected because it widens scope and competes with current
  result-surface authority
- task-only projection: rejected because packet `023` and packet `021` already established a
  multi-surface projection pattern

## Bounded conclusion

The legitimate packet-025 implementation packet is a deterministic step-policy packet layered on
landed packet `020` through packet `024` seams. It should freeze one bounded difficulty
assessment, one bounded action ladder, and one coherent projection through current result surfaces
without widening into proof ownership, durable-workflow ownership, security ownership, grounded
execution, PRM training, benchmarks, coordinator runtime, or interoperability.
