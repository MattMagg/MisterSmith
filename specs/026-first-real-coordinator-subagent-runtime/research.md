# Research Notes: First Real Coordinator-Subagent Runtime

## Current repo truth

- the supported runtime path already proves workflow graph formation, topology selection, routing
  metadata, supervision summaries, and same-agent session continuity
- the March 27 runtime-planning simplification pass restored the smallest-workflow rule and made
  sequential collapse the honest default when fan-out is not justified
- the March 28 session-context report made the remaining gap explicit: current graph success is
  still not the same thing as visible coordinator-subagent runtime behavior
- the current placeholder step boundary still allows delegated-looking work to complete without
  grounded delegated evidence

## Upstream packet assumptions

This packet is scaffolded before packets `022` through `025` are finished.

Current working assumptions are:

- packet `022` will own durable workflow semantics, lifecycle verbs, and effect boundaries
- packet `023` will own run-trace taxonomy and proof-boundary wording
- packet `024` will own boundary hardening, quarantine, and delegated authority rules
- packet `025` will own step scoring, escalation, retry, clarify, and downgrade policy

These assumptions must be rechecked before implementation starts.

## Why current graph/runtime proof is not enough

The current runtime can already show graph shape, branch counts, topology rationale, selected
workers, routing history, and result envelopes. That is useful, but it is still insufficient for
packet `026`.

Packet `026` exists because a real coordinator-subagent runtime must also show:

- which bounded job the coordinator delegated
- which subagent owned that job
- what state that delegated job moved through
- what grounded evidence the delegated job produced
- what merge or recovery decision the coordinator made in response

Without those things, the runtime can still look successful while failing the user's expected
meaning of coordination.

## Proof standard for "real coordinator-subagent runtime"

The scaffold uses this proof standard:

1. a run must show at least one coordinator-owned delegation record
2. a run must show visible state for delegated subagent work
3. a run must show grounded delegated work evidence for at least one delegated job
4. a run must show at least one visible coordinator merge, clarify, reassign, stop, or collapse
   decision when the run requires it
5. a run that only reaches placeholder delegated completion must remain explicitly non-grounded
6. a task that does not justify branching may still satisfy the packet honestly by collapsing back
   to sequential work instead of inventing fake subagent activity

## Why comparator frameworks are only reference points

OpenAI Agents SDK, Google ADK, and LangGraph are useful comparators for how delegation can be
surfaced, how state can be shown to operators, and how bounded coordination loops can be
described. They are not the Mister Smith contract.

Packet `026` must stay grounded in:

- `docs/current-state.md`
- the packet-prep dossiers
- the session-context report
- the smallest-workflow rule already landed in this repo

## Bounded conclusion

The right scaffold for packet `026` is not generic multi-agent expansion. It is one bounded future
packet that makes local coordinator-subagent runtime behavior honest and visible while preserving
the current runtime's smallest-workflow discipline and while reusing the ownership boundaries
already assigned to packets `022` through `025`.
