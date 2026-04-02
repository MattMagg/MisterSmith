# Feature Specification: Runtime Truth And Run Trace

**Feature Branch**: `023-runtime-truth-and-run-trace`
**Created**: 2026-04-01
**Status**: Implementation-ready
**Input**: `docs/direction.md`, `docs/current-state.md`,
`specs/022-durable-workflow-core/spec.md`,
`docs/plans/2026-03-29-packet-021-supervision-evidence-proof-boundary.md`,
`docs/plans/2026-03-19-session-restart-resume-live-proof.md`, and the current runtime surfaces in
`crates/mister-smith-agents/src/orchestrator.rs`, `crates/mister-smith-app/src/execution.rs`,
`crates/mister-smith-core/src/autonomy.rs`, and `crates/mister-smith-events/src/bus.rs`

## Current Truth & Scope

Current repo truth on `main` already includes:

- packet `019` bounded live proof for the supported `openai_chatgpt` / `gpt-5.4` path
- packet `020` landed orchestration-quality and repair-lineage projection
- packet `021` landed `supervision_evidence` projection with deterministic validation and no new
  live rerun claim
- packet `022` landed durable workflow core with deterministic validation and explicit ownership of
  lifecycle, event history, compaction, and effect boundaries
- stable runtime identifiers such as `workflow_id`, `session_id`, `coordinator_agent_id`, and
  existing transport trace and correlation fields

The remaining gap is narrower than generic observability or runtime-expansion work:

- the runtime still allows a run to look complete at the orchestration layer while semantic task
  completion remains unproven
- the current `workflow.execute_step` boundary still marks payloads as `completed` on the
  `tool_bus` path without proving grounded task work
- the repo has packet-021 predictive-supervision evidence, but it does not yet have one
  packet-023-owned runtime-truth contract that cleanly explains what a run actually proved
- task, session, autonomy, and operator surfaces still lack one shared run-trace summary and proof
  boundary model that stays separate from predictive supervision

Packet `023` owns one bounded slice:

1. define one packet-023-owned `runtime_truth` contract rooted at `workflow_id`
2. define one explicit proof-boundary view that separates substrate completion from grounded task
   proof
3. define one bounded run-trace summary that covers graph, branch, node, tool boundary, handoff,
   repair, retry, fan-out, join, and supervision relationships
4. project that same runtime-truth story across task, session, autonomy, and operator surfaces
   without overclaiming grounded work

This packet does **not** own:

- packet `022` durable lifecycle, event-history, compaction, or effect-boundary semantics
- packet `021` predictive-supervision semantics
- generic observability-platform or export-pipeline work
- coordinator-runtime, interoperability, or real subagent-runtime expansion
- any fresh live runtime proof claim unless a real rerun is explicitly executed

## First-Slice Decisions

These choices are frozen for the first packet-023 implementation pass.

- **Runtime truth container**: add one new packet-023-owned `RuntimeTruthView` and keep
  packet-021 `supervision_evidence` as a separate adjacent field.
- **Proof-boundary shape**: `RuntimeTruthView` carries a typed `ProofBoundaryView`, one
  `ExecutionEvidenceClass`, optional grounded evidence references, and a bounded run-trace summary.
- **Run-trace root**: `workflow_id` is the canonical run anchor. Existing transport `trace_id`
  remains transport metadata and is reused as input when present. Packet `023` does not widen
  `MessageEnvelope`.
- **Placeholder boundary wording**: the default non-grounded case stays:
  `workflow graph executed successfully`,
  `semantic completion not yet proven`,
  `grounded tool execution: none/minimal`, and
  `result is orchestration proof, not substantive task proof`.
- **Proof-status split**: packet `019` and packet `020` remain the last fresh live-proof baseline;
  packet `021` and packet `022` are landed and deterministically validated; packet `023` starts as
  deterministic projection work unless a new live rerun is actually executed.

## User Scenarios & Testing

### User Story 1 - Report honest proof boundaries for one completed run (Priority: P1)

An operator inspects a completed run and can immediately tell whether the system proved only
workflow-graph execution or also proved grounded task work.

**Why this priority**: This is the core honesty gap the packet exists to close. If this stays
blurry, later runtime and operator claims stay misleading.

**Independent Test**: A task, session, or autonomy result that shows graph completion through the
current placeholder-step boundary can be inspected and still clearly states that semantic
completion is not yet proven.

**Acceptance Scenarios**:

1. **Given** a run completed through `workflow.execute_step` with `execution_boundary=tool_bus`,
   **When** an operator inspects the result surface, **Then** it says
   `workflow graph executed successfully` while also stating
   `semantic completion not yet proven`.
2. **Given** the run produced no grounded external action beyond the placeholder step boundary,
   **When** the proof-boundary block is rendered, **Then** it states
   `grounded tool execution: none/minimal`.
3. **Given** packet `019` and packet `020` remain the last fresh live-proof baseline while packet
   `021` and packet `022` are deterministic-only for their newer claim surfaces, **When** the
   packet describes current proof status, **Then** it preserves that split explicitly instead of
   merging them into one vague live claim.

---

### User Story 2 - Freeze one canonical run-trace taxonomy (Priority: P1)

The runtime can expose one shared taxonomy for run traces and proof boundaries instead of
re-inventing names and relationships per surface.

**Why this priority**: Later packets depend on clear truth and trace language. If packet `023`
does not own this now, later packets will drift.

**Independent Test**: One shared contract can classify graph, branch, node, tool, handoff,
repair, retry, fan-out, join, and supervision relationships without redefining packet `022`
lifecycle semantics.

**Acceptance Scenarios**:

1. **Given** a workflow run fans out into branches and later joins, **When** the runtime-truth
   block is built, **Then** it distinguishes parent-child flow from linked reconvergence without
   claiming the repo already emits a full span graph.
2. **Given** a run includes repair or retry behavior, **When** the run-trace summary is rendered,
   **Then** repair and retry edges appear as explicit relationship kinds rather than hidden status
   noise.
3. **Given** packet `022` still owns durable lifecycle and event-history semantics, **When**
   packet `023` names trace records and proof boundaries, **Then** it reuses those identifiers and
   ownership boundaries instead of redefining lifecycle behavior.

---

### User Story 3 - Keep operator surfaces consistent without widening packet scope (Priority: P2)

An operator can compare task, session, autonomy, and operator views and see the same runtime-truth
story without turning packet `023` into a UI redesign or tracing-platform packet.

**Why this priority**: The packet is about truthful projection, not a redesign. The value is
consistency across existing surfaces.

**Independent Test**: The same run exposes one shared runtime-truth block across task, session,
autonomy, and operator run-detail views.

**Acceptance Scenarios**:

1. **Given** task, session, autonomy, and operator surfaces all show run results, **When** packet
   `023` defines runtime-truth projection, **Then** each surface uses the same bounded truth story
   rather than drifted wording.
2. **Given** OpenTelemetry and W3C tracing docs are used as guidance, **When** packet `023`
   defines surface language, **Then** it borrows taxonomy carefully without claiming the repo
   already has a complete emitted span model.
3. **Given** the operator console already renders predictive supervision, **When** packet `023`
   adds runtime truth, **Then** the new panel stays separate from predictive supervision instead of
   collapsing both concepts into one block.

## Edge Cases

- a graph completes successfully while every step still uses the placeholder `workflow.execute_step`
  boundary
- task, session, autonomy, and operator surfaces drift to different proof-boundary wording for the
  same run
- a repair or retry edge exists, but the surface only shows final completion state
- a fan-out or join path is visible in runtime metadata, but no grounded work occurred below the
  step boundary
- no grounded evidence reference exists and the runtime-truth block must say that explicitly
- a surface tries to treat OpenTelemetry terms as proof that the repo already emits a complete
  span hierarchy
- packet `022` lifecycle or history data is present, but packet `023` must not redefine its
  semantics

## Requirements

### Functional Requirements

- **FR-001**: System MUST define one packet-023-owned `RuntimeTruthView` rooted at `workflow_id`.
- **FR-002**: System MUST define one packet-023-owned `ProofBoundaryView` that keeps substrate
  completion separate from grounded task proof.
- **FR-003**: System MUST define one bounded `RunTraceSummaryView` that covers graph, branch,
  node, tool boundary, handoff, repair, retry, fan-out, join, and supervision relationship kinds.
- **FR-004**: System MUST define one `ExecutionEvidenceClass` that distinguishes substrate
  completion, placeholder or simulated completion, grounded tool execution, and grounded task
  proof.
- **FR-005**: System MUST keep `runtime_truth` separate from packet-021 `supervision_evidence`.
- **FR-006**: System MUST preserve the current proof split in its written and rendered contract:
  packet `019` and packet `020` remain the last fresh live-proof baseline on the supported path,
  while packet `021` and packet `022` are landed and deterministically validated but do not create
  a new default-path live claim by themselves.
- **FR-007**: System MUST state the current placeholder-step limit explicitly: while
  `WorkflowStepTool` only echoes payload and marks `workflow.execute_step` as `completed` on the
  `tool_bus` boundary, that boundary does not prove grounded task work.
- **FR-008**: System MUST preserve the exact conservative language below for current
  placeholder-boundary runs:
  `workflow graph executed successfully`,
  `semantic completion not yet proven`,
  `grounded tool execution: none/minimal`, and
  `result is orchestration proof, not substantive task proof`.
- **FR-009**: System MUST project the same `runtime_truth` block across task, session, autonomy,
  and operator run-detail surfaces.
- **FR-010**: System MUST keep packet `022` as owner of durable lifecycle, event-history,
  compaction, and effect-boundary semantics.
- **FR-011**: System MUST treat OpenTelemetry and W3C Trace Context as taxonomy guidance only and
  MUST NOT claim the repo already emits a complete span model.
- **FR-012**: System MUST NOT widen this packet into UI polish, generic observability-platform
  work, coordinator-runtime, or interoperability work.
- **FR-013**: System MUST keep deterministic projection proof separate from fresh live runtime
  proof and MUST NOT claim a new live rerun unless one is actually executed.
- **FR-014**: System MUST leave `MessageEnvelope` schema unchanged in the first slice and reuse
  existing `workflow_id`, `trace_id`, graph metadata, repair lineage, and supervision state as
  synthesis inputs instead.

### Key Entities

- **RuntimeTruthView**: the packet-owned result and inspection block for one workflow run
- **ProofBoundaryView**: the bounded summary that states what was proven and what was not proven
- **RunTraceSummaryView**: the bounded run-trace summary for one workflow run
- **ExecutionEvidenceClass**: the strongest evidence the run actually produced
- **GroundedEvidenceReference**: a stable reference to real files, endpoints, artifacts, or other
  grounded work touched during the run
- **RunTraceRelationshipKind**: one typed relationship kind such as graph, branch, node, tool
  boundary, handoff, repair, retry, fan-out, join, or supervision

## Success Criteria

### Measurable Outcomes

- **SC-001**: A reviewer can inspect the packet `023` contract and determine in one pass whether a
  run proves only orchestration-substrate completion or also proves grounded task work.
- **SC-002**: Task, session, autonomy, and operator run-detail surfaces expose the same
  `runtime_truth` contract for the same run in deterministic validation.
- **SC-003**: The packet states packet `019` and packet `020` live-proof baseline separately from
  packet `021` and packet `022` deterministic-only proof with no contradictory wording.
- **SC-004**: The packet and code keep packet `022` lifecycle ownership intact while adding no new
  `MessageEnvelope` schema fields.
- **SC-005**: Placeholder-boundary runs render the frozen conservative wording and do not claim
  grounded task proof without grounded evidence.

## Assumptions

- Packet `022` is landed on `main` and remains the owner of durable workflow semantics.
- Packet `021` is landed on `main` and deterministically validated, but it still does not carry a
  fresh live rerun claim.
- Existing task, session, autonomy, and operator run-detail surfaces remain the intended
  projection targets.
- OpenTelemetry and W3C Trace Context remain the right external taxonomy references for naming and
  relationship shape, even though packet `023` does not claim a full emitted tracing model.
