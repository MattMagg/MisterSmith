# Research Notes: Step-Level Intelligence v2

## Current repo truth

- packet `019` already landed one bounded budget-aware runtime routing proof and current
  budget-pressure routing semantics on `main`
- packet `020` already landed `StepEvaluationRecord`, repair directives, clarification requests,
  and failure-context lineage on the runtime-backed task path
- packet `021` already landed `supervision_evidence` on task inspect, autonomy status, and
  operator selected-run detail, with deterministic validation only unless a fresher live proof is
  produced
- packet `022` already landed durable workflow lifecycle, event-history, compaction, and
  effect-boundary ownership
- packet `023` already landed `runtime_truth`, proof-boundary wording, and bounded run-trace
  projections
- packet `024` already landed agent-boundary hardening and does not widen packet `025` scope
- `MS-72` already landed workflow-visible step-routing history and operator-visible routing
  rationale on current surfaces
- the current `workflow.execute_step` boundary still marks placeholder step completion and must
  stay below grounded task proof in packet wording

## Decision 1: Build step policy from current repo signals first

**Decision**: Packet `025` should use the runtime signals already landed on `main` as its first
packet-owned inputs.

**Rationale**:

- current repo truth already exposes step evaluation, repair lineage, routing history, supervision
  evidence, runtime truth, and bounded budget signals
- the first missing layer is not more raw telemetry; it is one coherent deterministic policy that
  uses those existing signals together
- this keeps the packet grounded in exact current seams instead of pulling in a larger learned
  policy system too early

**Alternatives considered**:

- building packet `025` around a new PRM or judge-heavy policy stack first: rejected because it
  would widen the packet before the deterministic contract exists
- delaying packet `025` until a new live rerun exists: rejected because the missing gap is a
  packet-owned contract and summary layer, not a prerequisite live-proof packet

## Decision 2: Keep packet ownership clean

**Decision**: Packet `025` should add one packet-owned `step_policy` summary beside the current
packet `020`, `021`, and `023` surfaces instead of replacing them.

**Rationale**:

- packet `020` already owns verifier outcomes, clarification requests, repair directives, and
  repair lineage
- packet `021` already owns predictive-supervision evidence
- packet `023` already owns runtime-truth, proof-boundary wording, and run-trace schema
- packet `025` only needs to explain the current step policy, not take over those adjacent
  contracts

**Alternatives considered**:

- folding packet `025` into `runtime_truth`: rejected because it would blur packet `023`
  ownership
- folding packet `025` into `supervision_evidence`: rejected because packet `021` already owns
  that contract and packet `025` needs a broader decision summary than supervision alone

## Decision 3: Keep the first slice deterministic and bounded

**Decision**: The first packet-025 implementation slice should be deterministic, heuristic, and
bounded to current repo signals.

**Rationale**:

- the repo already has enough structured current-state input to produce a useful deterministic
  action ladder
- deterministic policy is easier to audit and easier to validate honestly against packet `023`
  proof wording
- the deeper research prompt and `R6` output remain useful frontier context, but they are not a
  reason to block a bounded first slice

**Alternatives considered**:

- benchmark-first or training-first work: rejected because packet `025` is a packet-owned contract
  and runtime summary slice, not a benchmark packet
- provider-specific step intelligence: rejected because the first slice should stay model-agnostic
  and repo-grounded

## Decision 4: Existing read surfaces stay canonical

**Decision**: Packet `025` should project step-policy summaries through existing task inspect,
autonomy status, and operator selected-run detail surfaces only.

**Rationale**:

- those are already the current read surfaces on `main`
- the operator console already renders runtime truth and supervision evidence from the task inspect
  payload
- no new endpoint is needed for the first slice

**Alternatives considered**:

- adding a session projection in the first slice: rejected because it is useful but not necessary
  to deliver the bounded packet value
- adding a new trace or step-policy endpoint: rejected because it widens scope and competes with
  existing read surfaces

## Decision 5: Packet-023 proof honesty remains the guardrail

**Decision**: Packet `025` should consume packet-023 proof wording and never use step policy to
upgrade placeholder completion into grounded task proof.

**Rationale**:

- current repo truth already states that `workflow.execute_step` placeholder completion is
  orchestration proof only
- packet `025` adds policy interpretation, not stronger proof
- keeping proof honesty unchanged prevents later packets from inheriting a false stronger claim

**Alternatives considered**:

- using packet `025` to imply stronger semantic completion from policy confidence alone: rejected
  because policy confidence is not grounded evidence

## Decision 6: Frontier research stays background guidance, not a freeze blocker

**Decision**: `docs/research-prompts/09-step-level-intelligence.md` and
`docs/research-output/research/targeted-step-level-intelligence-R6.md` remain useful background
for follow-on work, but packet `025` should not depend on frontier PRM or speculative-decoding
features in its first implementation slice.

**Rationale**:

- the research strongly supports step-level intelligence as a frontier direction
- the current repo does not yet need that full architecture to ship a bounded deterministic packet
- keeping the research in a guidance role lets packet `025` stay implementation-ready today while
  leaving room for a stronger later packet if the deterministic contract proves valuable

## Bounded conclusion

The legitimate packet-025 implementation slice is a deterministic step-policy layer on top of
current packet `019` through packet `024` seams. It should freeze one packet-owned difficulty
summary, one bounded budget-pressure summary, and one bounded action ladder, then project that
summary through existing task inspect, autonomy status, and operator selected-run detail surfaces
without widening into proof ownership, training, benchmarks, coordinator runtime, or
interoperability work.
