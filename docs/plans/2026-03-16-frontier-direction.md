# Frontier Direction

Date: March 19, 2026
Status: Historical

Use `docs/plans/2026-03-21-post-packet-016-development-checkpoint.md` for the current repo-wide
forward-development authority and
`docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md` for the last
completed frontier-packet closure evidence. This note is now the March 19 frontier-recovery
direction snapshot that led into the later checkpoint chain.

Use `docs/current-state.md` for the current whole-repo overview. This note is the forward-direction
artifact, not the general repo-state document.

## Objective

Refresh the frontier direction after the post-recovery backlog landed on `main`, update the repo's
durable planning surfaces to match what is now true, and identify the next honest bounded slice
instead of treating already-complete epics as future work.

This note now captures the direction after `MS-45`, `MS-46`, `MS-47`, `MS-75`, and `MS-76`
landed: keep Mister Smith on the operating-system path, but only stage the remaining gaps that are
still real on `main`.

## Scope

- current mainline truth after the March 16 recovery reconciliation
- the frontier mandate that should govern the next phase of development
- the completed frontier epics and the remaining open frontier gap
- future Symphony staging posture for the next bounded slice only

## Assumptions

- `main` is now the only durable development branch that matters
- the recovered runtime-backed task path and bounded session path are both landed on `main`
- the runtime wiring pass from `MS-76` is landed on `main`
- Symphony's watched queue should remain empty until new work is explicitly staged
- future work should be judged by operating-system leverage, not by framework feature parity

## Constraints

- do not move new work into `Todo` during this refresh pass
- keep the watched project limited to explicitly staged runnable work
- prefer issue slices that can validate honestly with local runtime or targeted crate proof
- keep the frontier mandate intact: Mister Smith is an operating system, not a generic framework

## Non-Goals

- re-open landed Phase 10 or March 16 recovery work unless current validation shows a defect
- bulk-stage the backlog just to keep Symphony busy
- widen the next phase into generic hardening or undifferentiated SDK parity work

## Current Mainline State

- the runtime-backed task path is live on `main` through `mister-smith run`,
  `POST /api/v1/tasks`, and the autonomy inspection surfaces
- the first real provider-backed proof was completed locally on `openai_chatgpt` with `gpt-5.4`
- the first bounded same-agent session slice is live on `main` through the session HTTP and CLI
  surfaces
- the default runtime path now uses supervised planner and executor lifecycles, a Tokio workflow
  runner, and a ToolBus-backed execution boundary
- `MS-45`, `MS-46`, and `MS-47` are complete in Smith and on `main`
- `MS-48` is partially complete on `main` through `MS-73`, `MS-74`, and `MS-75`
- the watched Symphony queue is currently empty, which is correct because no new runnable slice has
  been staged into `Todo`
- the remaining open frontier gap is the additive external-agent interoperability surface inside
  `MS-48`

## Directional Mandate

The next phase should optimize for one outcome:

> Mister Smith should outperform other agent systems by behaving like an adaptive orchestration
> operating system: it should compile task structure into execution topology, size teams to the
> work, preserve state across turns and restarts, make routing decisions at step granularity, and
> expose interoperable capability boundaries without surrendering supervised control.

That direction follows the accepted mandate in
`docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`:

- topology and coordination must beat static teams
- state and memory must survive long-running operation
- routing must become budget-aware, capability-aware, and step-aware
- operator visibility and revocable authority remain mandatory runtime properties

## Completed Frontier Epics

### `MS-45` Task-Shape-Aware Orchestration And Dynamic Team Sizing

This epic is complete on `main`.

Delivered outcome:

- topology and team size now adapt to task structure instead of staying fixed
- operator-visible autonomy status explains the topology and sizing choice
- the evaluation harness for representative parallel versus sequential workloads is landed

### `MS-46` Session Restart-Resume And Distributed Operating State

This epic is complete on `main`.

Delivered outcome:

- a restarted runtime can continue the same idle session
- `session_id` and `coordinator_agent_id` remain stable across the resumed turn
- resume provenance is operator-visible in status surfaces

### `MS-47` Step-Level Intelligence And Model Routing Control Loop

This epic is complete on `main`.

Delivered outcome:

- routing decisions can change at step boundaries based on live signals
- operators can inspect why routing changed
- the benchmark and validation evidence for routing control landed on `main`

## Remaining Open Frontier Epic

### `MS-48` Capability Kernel And External Agent Interoperability

Why it matters:

- the frontier mandate requires revocable capability, not ambient trust
- the research corpus says A2A-style external interoperability is the long-term standard for
  agent-to-agent delegation
- an operating system should be able to expose and consume external agent capability boundaries
  without giving up local supervision or provenance

Primary surfaces:

- `crates/mister-smith-security/src/delegation.rs`
- `crates/mister-smith-mcp/`
- `crates/mister-smith-http/`
- `crates/mister-smith-agents/src/tool_bus.rs`
- `crates/mister-smith-app/src/autonomy.rs`

What is already landed on `main`:

- capability descriptors and revocation checks for privileged delegation (`MS-73`)
- external delegation envelopes that preserve provenance and policy (`MS-74`)
- operator-visible allow/reject boundary decisions plus deterministic proof (`MS-75`)

What is still missing:

- a first-class external-agent interoperability surface on top of the zero-trust substrate
- additive capability discovery for external agents without ambient trust
- a bounded proof that this external surface preserves local policy, provenance, and operator
  control end to end

Acceptance shape for the remaining gap:

- capability descriptions are discoverable and enforceable
- external delegation surfaces preserve provenance and local policy
- interoperability remains additive to the zero-trust substrate instead of bypassing it

## Recommended Staging Posture

- treat `MS-45`, `MS-46`, and `MS-47` as complete backlog history, not as current frontier work
- keep `MS-48` in `MisterSmith Validated Backlog` until the remaining external-agent surface is
  split into one bounded runnable slice
- do not move new work into `Todo` during this refresh pass
- when the next execution cycle starts, stage only that bounded `MS-48` follow-up instead of
  reopening already-complete frontier epics

## Decision Rule For Future Work

Before staging any new issue into the watched queue, ask:

1. Does it strengthen supervised autonomy?
2. Does it improve coordination, supervision, execution, memory, routing, observability, state, or
   distributed behavior enough to make Mister Smith more like an operating system and less like a
   framework clone?

If the answer to both questions is not clearly yes, the issue should stay out of `Todo`.

## Validation For This Note

- repo docs updated to point at this note as the current forward-direction artifact
- Linear strategic and backlog surfaces updated to match the same direction without staging new
  work
- historical recovery and runtime/session notes updated so they point forward here instead of
  implying March 16 work is still pending
