# Research Notes: First Real Coordinator-Subagent Runtime

## Current repo truth

- the supported runtime path already proves workflow graph formation, topology selection, routing
  metadata, supervision summaries, runtime truth, and same-agent session continuity
- the March 27 runtime-planning simplification pass restored the smallest-workflow rule and made
  sequential collapse the honest default when fan-out is not justified
- packet `022` now owns durable lifecycle, event-history, compaction, and effect-boundary seams on
  current `main`
- packet `023` now owns runtime-truth, proof-boundary, and bounded run-trace projection on current
  `main`
- packet `024` now owns delegated-authority and boundary-hardening semantics on current `main`
- packet `025` now owns deterministic step-policy summaries on task, autonomy, and operator
  run-detail surfaces
- the session-context report still marks the key gap clearly: current graph success is still not
  the same thing as visible coordinator-subagent runtime behavior

## Why current graph and proof surfaces are still not enough

The runtime can already show graph shape, topology rationale, branch counts, routing history,
runtime truth, and result envelopes. That is useful, but it is still insufficient for packet
`026`.

Packet `026` exists because a real coordinator-subagent runtime must also show:

- which bounded job the coordinator delegated
- which child identity owns that job
- what state that delegated job moved through
- what subordinate inbox events re-entered the coordinator loop
- what grounded evidence the delegated job produced
- what merge or recovery decision the coordinator made in response

Without those things, the runtime can still look successful while failing the user's expected
meaning of coordination.

## Upstream ownership packet `026` must consume

- packet `022` owns durable workflow and lifecycle semantics
- packet `023` owns run-trace and proof-boundary semantics
- packet `024` owns security-boundary and delegated-authority semantics
- packet `025` owns step-policy and escalation semantics

Packet `026` must reuse those seams rather than redefine them.

## OpenClaude transfer decisions worth keeping

The refreshed OpenClaude transfer analysis produced five packet-026 inputs worth keeping:

1. a bounded subordinate-runtime inbox and event-drain rule
2. stable child identity plus explicit follow-up actions
3. child context isolation with shared root-only channels
4. deterministic ordered parallel batches with sibling-abort semantics
5. role-bounded child execution instead of prompt-only child specialization

These are worth keeping because they strengthen coordination-runtime truth, operator clarity, and
execution safety without widening packet `026` into shell parity or interoperability work.

## Proof standard for "real coordinator-subagent runtime"

Packet `026` uses this proof standard:

1. a run must show at least one coordinator-owned delegation record
2. a run must show visible delegated child state
3. a run must show grounded delegated work evidence for at least one delegated job
4. a run must show at least one visible coordinator merge, clarify, reassign, stop, or collapse
   decision when the run requires it
5. a run that only reaches placeholder delegated completion must remain explicitly non-grounded
6. a task that does not justify branching may still satisfy the packet honestly by collapsing back
   to sequential work instead of inventing fake child activity

## Bounded conclusion

The right implementation-ready packet for `026` is not generic multi-agent expansion. It is one
bounded packet that makes local coordinator-subagent runtime behavior honest and visible while
preserving the current runtime's smallest-workflow discipline and reusing the ownership boundaries
already assigned to packets `022` through `025`.
