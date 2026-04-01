# Feature Specification: Runtime Truth And Run Trace

**Feature Branch**: `023-runtime-truth-and-run-trace`
**Created**: 2026-04-01
**Status**: Draft
**Input**: `docs/direction.md`, `docs/current-state.md`,
`docs/research-output/analysis/2026-03-28-dynamic-orchestration-transfer-brief.md`,
`docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md`,
`docs/plans/2026-03-19-session-restart-resume-live-proof.md`, and the current runtime surfaces in
`crates/mister-smith-app/src/execution.rs`, `crates/mister-smith-core/src/autonomy.rs`,
`crates/mister-smith-events/src/bus.rs`, and `crates/mister-smith-transport/src/envelope.rs`

## Scaffold Status

This is a scaffold spec written ahead of upstream packet completion to speed later packet work.

- It freezes packet `023` scope, naming, proof-boundary language, and revalidation gates now.
- It does not claim packet `023` is implementation-ready or already validated for execution.
- It must be revised against then-current repo truth before any future `/speckit.implement`.

## Current Truth & Scope

Current repo truth already includes:

- packet `019` bounded live proof for the supported path under the documented
  `openai_chatgpt` / `gpt-5.4` baseline
- packet `020` live-proof-backed orchestration-quality and repair-lineage surfaces on the
  supported path
- packet `021` landed supervision-evidence projection with deterministic validation and explicit
  proof-boundary notes
- stable runtime identifiers such as `workflow_id`, `session_id`, and `coordinator_agent_id`
- transport-level trace and correlation fields on `MessageEnvelope`

The remaining gap is narrower than a broad observability or runtime-expansion packet:

- the runtime still allows a run to look successful at the orchestration-substrate layer while
  leaving semantic task completion unproven
- the current `workflow.execute_step` boundary still marks payloads as `completed` on the
  `tool_bus` path without proving grounded task work
- the repo does not yet have one frozen run-trace taxonomy spanning graph, branch, node, tool,
  repair, retry, fan-out, join, and supervision relationships
- proof-boundary wording is present in several places, but not yet frozen as one shared packet
  contract

This scaffold packet therefore freezes one bounded slice:

1. define one honest run-trace taxonomy rooted at `workflow_id` and one trace root per workflow
   run
2. define one explicit proof-boundary contract that separates substrate completion from grounded
   task proof
3. define how current task, session, autonomy, and operator surfaces should project truthful
   execution status without overclaiming grounded work

This is not:

- packet `022` durable lifecycle, event-history, compaction, or effect-boundary work
- UI polish, dashboard redesign, or generic observability-platform work
- coordinator-runtime, real subagent-runtime, or interoperability work
- proof that the repo already emits a complete OpenTelemetry-style span model

## Before Implementation Revalidation Gate

Before any future implementation starts, the next session must:

1. reread `docs/direction.md`
2. reread `docs/current-state.md`
3. reread `docs/research-output/analysis/2026-03-28-dynamic-orchestration-transfer-brief.md`
4. confirm packet `022` and any reused upstream packet work are complete enough to depend on
5. rerun `/speckit.clarify`, `/speckit.plan`, `/speckit.tasks`, and `/speckit.analyze` if repo
   truth moved

If those checks fail, this scaffold must be revised before code work begins.

## Deferred Revision Points

- finalize packet `022` lifecycle and durable-history ownership wording before packet `023`
  implementation starts
- re-check whether current proof-boundary wording across task, autonomy, session, and operator
  surfaces has drifted since this scaffold was written
- re-check whether newer live-proof or deterministic-only evidence changed the packet `019`,
  `020`, or `021` proof split before this packet is implemented

## User Scenarios & Testing

### User Story 1 - Report honest proof boundaries for one completed run (Priority: P1)

An operator inspects a completed run and can immediately tell whether the system proved only
workflow-graph execution or also proved grounded task work.

**Why this priority**: This is the core honesty gap the packet exists to close. If this stays
blurry, later runtime and operator claims stay misleading.

**Independent Test**: A task, session, or autonomy result that shows graph completion through the
current placeholder step boundary can be inspected and still clearly states that semantic
completion is not yet proven.

**Acceptance Scenarios**:

1. **Given** a run completed through `workflow.execute_step` with `execution_boundary=tool_bus`,
   **When** an operator inspects the result surface, **Then** it can say `workflow graph executed
   successfully` while also stating `semantic completion not yet proven`.
2. **Given** the run produced no grounded external action beyond the placeholder step boundary,
   **When** the proof-boundary block is rendered, **Then** it states `grounded tool execution:
   none/minimal`.
3. **Given** packet `019` and packet `020` remain the last fresh live-proof baseline while packet
   `021` remains deterministic-only for its newer supervision surface, **When** the packet
   describes current proof status, **Then** it preserves that split explicitly instead of merging
   them into one vague live-claim.

---

### User Story 2 - Freeze one canonical run-trace taxonomy (Priority: P1)

A future implementation team can use one packet-owned taxonomy for run traces and proof
boundaries instead of re-inventing names and relationships per surface.

**Why this priority**: Later packets depend on clear truth and trace language. If packet `023`
does not own this now, later packets will drift.

**Independent Test**: One written contract can classify graph, branch, node, tool, handoff,
repair, retry, fan-out, join, and supervision relationships without redefining packet `022`
lifecycle semantics.

**Acceptance Scenarios**:

1. **Given** a workflow run fans out into branches and later joins, **When** the packet describes
   trace relationships, **Then** it distinguishes parent-child flow from linked reconvergence
   without claiming the repo already emits a full span graph.
2. **Given** a run includes repair or retry behavior, **When** the trace taxonomy is applied,
   **Then** repair and retry edges are represented as explicit trace relationships rather than
   hidden status noise.
3. **Given** packet `022` still owns durable lifecycle and event-history semantics, **When**
   packet `023` names trace records and proof boundaries, **Then** it reuses those identifiers and
   ownership boundaries instead of redefining lifecycle behavior.

---

### User Story 3 - Keep operator surfaces consistent without widening packet scope (Priority: P2)

An operator or future packet author can compare task, session, autonomy, and operator views and
see the same proof-boundary story without turning packet `023` into a UI or platform packet.

**Why this priority**: The packet is about truthful projection, not a redesign. The value is
consistency across existing surfaces.

**Independent Test**: One written contract describes the same proof-boundary fields and wording
for task, session, autonomy, and operator run-detail views.

**Acceptance Scenarios**:

1. **Given** task, session, autonomy, and operator surfaces all show run results, **When** the
   packet defines proof-boundary projection, **Then** each surface uses the same bounded truth
   story rather than drifted wording.
2. **Given** OpenTelemetry and W3C tracing docs are used as guidance, **When** the packet defines
   surface language, **Then** it borrows taxonomy carefully without claiming the repo already has a
   complete emitted span model.
3. **Given** future implementation packets need this contract later, **When** this scaffold is
   handed off, **Then** it is clear what must be revised before code work starts and what can be
   reused as-is.

## Edge Cases

- a graph completes successfully while every step still uses the placeholder `workflow.execute_step`
  boundary
- task and autonomy surfaces use slightly different proof-boundary text for the same run
- a repair or retry edge exists, but the surface only shows final completion state
- a fan-out or join path is visible in runtime metadata, but no grounded work occurred below the
  step boundary
- a surface tries to treat OpenTelemetry terms as proof that the repo already emits a complete
  span hierarchy
- packet `022` later freezes lifecycle or history semantics that change the preferred wording or
  ownership boundary for packet `023`

## Requirements

### Functional Requirements

- **FR-001**: System documentation for packet `023` MUST define one run-trace taxonomy rooted at
  `workflow_id` with one trace root per workflow run.
- **FR-002**: Packet `023` MUST define one proof-boundary contract that keeps substrate
  completion separate from grounded task proof.
- **FR-003**: Packet `023` MUST preserve the current proof split in its written contract: packet
  `019` and packet `020` remain the last fresh live-proof baseline on the supported path, while
  packet `021` supervision evidence is landed and deterministically validated but does not create a
  new default-path live claim by itself.
- **FR-004**: Packet `023` MUST state the current placeholder-step limit explicitly: while
  `WorkflowStepTool` only echoes payload and marks `workflow.execute_step` as `completed` on the
  `tool_bus` boundary, that boundary does not prove grounded task work.
- **FR-005**: Packet `023` MUST preserve the exact conservative language below for current
  placeholder-boundary runs:
  `workflow graph executed successfully`,
  `semantic completion not yet proven`,
  `grounded tool execution: none/minimal`, and
  `result is orchestration proof, not substantive task proof`.
- **FR-006**: Packet `023` MUST define trace-taxonomy coverage for graph, branch, node, tool,
  handoff, repair, retry, fan-out, join, and supervision relationships.
- **FR-007**: Packet `023` MUST describe consistent proof-boundary projection expectations for
  task, session, autonomy, and operator run-detail surfaces.
- **FR-008**: Packet `023` MUST treat OpenTelemetry and W3C Trace Context as taxonomy guidance
  only and MUST NOT claim the repo already emits a complete span model.
- **FR-009**: Packet `023` MUST keep packet `022` as owner of durable lifecycle, event-history,
  compaction, and effect-boundary semantics.
- **FR-010**: Packet `023` MUST include a blocking revalidation gate before any future
  implementation begins.
- **FR-011**: Packet `023` MUST remain scoped to truthful naming, trace taxonomy, and
  proof-boundary projection and MUST NOT widen into UI polish, generic observability-platform
  work, or coordinator-runtime scope.
- **FR-012**: Packet `023` MUST stay explicitly revision-required until upstream packet work it
  depends on is complete enough to revalidate the scaffold against then-current repo truth.

### Key Entities

- **Run Trace Record**: the packet-owned description of one workflow run’s trace root,
  relationships, and proof-boundary story anchored to `workflow_id`
- **Trace Root**: the canonical top-level identifier for one workflow run’s trace view
- **Trace Event**: a typed traceable event such as graph formation, branch execution, tool
  boundary crossing, repair, retry, or supervision
- **Trace Link**: the relationship record used to describe parent-child execution, fan-out,
  reconvergence, repair, or retry connections between trace events
- **Proof Boundary View**: the bounded summary that states what was proven and what was not proven
  for one run
- **Grounded Evidence Reference**: the stable reference to files, endpoints, artifacts, or other
  grounded evidence actually touched during a run when such evidence exists
- **Execution Evidence Class**: the packet-owned classification that distinguishes substrate
  completion, placeholder or simulated completion, grounded tool execution, and grounded task proof

## Success Criteria

### Measurable Outcomes

- **SC-001**: A future reviewer can inspect the packet `023` scaffold and determine in one pass
  whether a run proves only orchestration-substrate completion or also proves grounded task work.
- **SC-002**: The packet provides one shared taxonomy that covers all required run-trace
  relationship types without widening into lifecycle ownership or platform scope.
- **SC-003**: The packet states the packet `019` and packet `020` live-proof baseline separately
  from packet `021` deterministic-only supervision proof with no contradictory wording.
- **SC-004**: The packet gives future implementation work one explicit revalidation gate so no
  session can honestly treat the scaffold as implementation-ready without rereading current repo
  truth first.
- **SC-005**: The packet leaves later implementers with enough contract language to begin a later
  revision pass without rediscovering the placeholder-step truth gap from scratch.

## Assumptions

- Packet `022` is still the owner of durable workflow semantics and will be completed or clarified
  before packet `023` implementation starts.
- Upstream packet work may still move repo truth before packet `023` is implemented, so this
  scaffold will need a revision pass.
- Existing task, session, autonomy, and operator run-detail surfaces remain the intended
  projection targets; this scaffold does not assume a new surface will be added first.
- OpenTelemetry and W3C Trace Context remain the right external taxonomy references for naming and
  structure, even if the final repo contract later narrows or renames parts of that taxonomy.
