# Research Notes: Step-Level Intelligence v2

## Current repo truth

- packet `020` already landed verifier-gated step decisions, clarification, and repair lineage on
  the runtime-backed task path
- packet `021` already landed supervision evidence on task, autonomy, and operator-visible
  surfaces, but the newer packet-021 proof remains deterministic-only unless fresher live proof is
  produced
- packet `023` is the scoped owner of run-trace taxonomy and proof-boundary language in the
  packet-prep layer
- the March 28 session context report still shows the core truth gap: `workflow.execute_step`
  proves orchestration-substrate completion, not grounded task proof

## Decision 1: Use the OpenAI Responses event taxonomy as the canonical event input

**Decision**: Packet `025` should treat the OpenAI Responses event taxonomy as the canonical event
input for streamed step-policy terms.

**Rationale**:

- the current official streaming guide says the Responses API uses semantic typed events for
  streaming
- the function-calling streaming guide names concrete tool-call events such as
  `response.output_item.added`, `response.function_call_arguments.delta`, and
  `response.function_call_arguments.done`
- packet `025` needs one event-language baseline for step-policy naming, and the packet-prep
  dossier already points to Responses docs as that baseline

**Alternatives considered**:

- older Chat Completions streaming docs: rejected because packet `025` should not backslide to an
  older event model
- repo-only ad hoc event names: rejected because the packet explicitly wants official Responses
  terminology as the input baseline

## Decision 2: Re-confirm the exact official streaming-events reference page before freeze

**Decision**: The final implementation freeze should re-confirm the exact current official
  streaming-events reference page before packet `025` treats individual event names as frozen.

**Rationale**:

- the packet-prep dossier names a standalone streaming-events reference path
- the current OpenAI docs still clearly document the Responses semantic event model, but the exact
  standalone reference URL may have moved
- packet `025` should preserve the Responses event taxonomy as canonical without pretending the
  current reference path is already stable forever

**Alternatives considered**:

- freezing the older packet-prep URL without re-checking: rejected because the docs path may move
- ignoring official docs entirely and relying only on the packet-prep note: rejected because the
  final packet must be grounded in current primary-source docs

## Decision 3: Keep the first slice heuristic and deterministic

**Decision**: The first real packet-025 implementation slice should use deterministic heuristics
and bounded policy rules rather than a judge-heavy or training-heavy control loop.

**Rationale**:

- the packet-prep dossier explicitly says the first slice should stay heuristic and deterministic
- the repo already has useful step-evaluation, routing, repair, and supervision seams to build on
- deterministic policy is easier to audit and keeps the proof boundary honest while the underlying
  step boundary is still placeholder-only

**Alternatives considered**:

- PRM or judge-heavy scoring from day one: rejected because it would widen the scope into training
  or heavier runtime claims before the packet owns a stable deterministic policy surface
- benchmark-first work: rejected because packet `025` is a policy packet, not a benchmark packet

## Decision 4: Packet `025` consumes packet `023` proof ownership instead of competing with it

**Decision**: Packet `025` should use packet-023-owned proof or grounding references and never
create a competing proof-boundary schema.

**Rationale**:

- packet `023` is the packet-prep owner of run-trace taxonomy and proof-boundary language
- packet `025` is about step scoring and action policy, not about redefining run truth
- keeping ownership boundaries clean prevents the same proof claim from drifting across packets

**Alternatives considered**:

- embedding a packet-025-specific proof schema in step policy: rejected because it would silently
  duplicate packet-023 scope
- treating step-policy summaries as self-sufficient proof: rejected because the March 28 session
  report shows why placeholder completion must stay explicit

## Decision 5: Existing inspect surfaces stay canonical

**Decision**: Packet `025` should project step-policy summaries through existing task inspect and
autonomy surfaces, with only a bounded operator-facing summary layered on top if needed.

**Rationale**:

- those are already the current runtime truth surfaces
- the packet goal is better step policy, not a new observability product
- keeping the first slice on current surfaces reduces scope and preserves current-state honesty

**Alternatives considered**:

- a new endpoint or a new trace dashboard: rejected because it widens scope and collides with
  packet `023`

## Bounded conclusion

The legitimate packet-025 scaffold is a deterministic step-policy packet layered on current packet
`020` and packet `021` seams. It should freeze one bounded score and action contract, carry
budget-aware hints, and project honest placeholder-vs-grounded wording through current inspect
surfaces without widening into trace ownership, grounded execution, training, benchmarks,
coordinator runtime, or interoperability.
