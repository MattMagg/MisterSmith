# Feature Specification: Complex Multi-Agent Proof and Unified Result Surfaces

**Feature Branch**: `015-complex-multi-agent-proof-and-unified-result-surfaces`  
**Created**: 2026-03-19  
**Status**: Draft  
**Input**: `docs/plans/2026-03-19-central-development-checkpoint.md`,
`docs/plans/2026-03-19-live-run-trace-evaluation.md`,
`docs/plans/2026-03-19-short-multi-agent-result-evaluation.md`,
`docs/plans/2026-03-19-framework-comparison-stress-test.md`,
`docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`,
`docs/plans/2026-03-19-ms-48-closure-audit.md`, current runtime and operator code in
`crates/mister-smith-app/`, `crates/mister-smith-agents/`, `crates/mister-smith-events/`, and
`crates/mister-smith-core/`

## Current Truth & Scope

This packet formalizes the next bounded operating-system feature after the March 19 checkpoint.

Current repo truth on `main` already includes the baseline that this packet must not reopen:

- the supervised planner and executor live path is wired into the default runtime
- the default runtime crosses the `tool_bus` execution boundary
- autonomy status already exposes topology, routing, step routing, interventions, delegation, and
  external capability decisions
- the bounded MCP capability discovery and enforcement surface from `MS-77` is already landed
- result material already exists in code:
  - `task.result` stores terminal task output
  - runtime metadata stores `final_result`
  - `final_result` already nests `aggregated_result`
  - session retained context already stores `assistant_result` and `last_assistant_result`

The unfinished differentiation gap is narrower:

- harder workload proof is still incomplete on the default live path
- existing result forms are not yet governed by one shared contract across task, session, and
  operator surfaces
- the current proof-relevant surfaces do not yet make final-result evidence easy to inspect and
  trust across success, collapse, and failure-visible outcomes

This packet therefore defines one bounded epic with four stories:

1. harder workloads produce real multi-step graphs when the planner can support them
2. existing final-result material is stored and surfaced through one unified contract
3. operator and evaluation surfaces expose enough result preview and provenance to verify behavior
   without dumping full payloads
4. collapse-to-sequential and failed-before-graph boundaries stay visible and classifiable

This is not a new router program, not a JetStream KV or budget follow-up, and not a broader
external-agent interoperability expansion.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Prove Harder Workload Graph Formation On The Default Path (Priority: P1)

An operator submits a harder workload on the default live path and the runtime forms a real
multi-step graph when the planner can support it.

**Why this priority**: the checkpoint says the remaining product gap is honest proof under harder
workloads, not more substrate.

**Independent Test**: run a representative harder workload and verify a real multi-step graph,
branching behavior, and a terminal task result are visible from stored runtime evidence.

**Acceptance Scenarios**:

1. **Given** a workload that the planner can decompose beyond the existing toy case, **When**
   execution succeeds, **Then** the runtime records a non-trivial graph plus a terminal result
   through the unified result contract.
2. **Given** a harder workload that still fits the current live-path guardrails, **When** the
   planner emits a graph, **Then** operator surfaces can distinguish graph formation from final
   completion without raw log scraping.
3. **Given** a harder workload run, **When** proof is reviewed later, **Then** stored evidence is
   sufficient to verify the runtime outcome class and inspect the final-result material at a
   bounded preview level.

---

### User Story 2 - Unify Existing Final-Result Material Across Task, Session, And Operator Views (Priority: P1)

An operator or developer inspects a completed workflow and sees one consistent result contract
instead of inferring relationships between several loosely related result shapes.

**Why this priority**: result material already exists, but it is not yet governed by one contract
that future proof and operator surfaces can depend on.

**Independent Test**: capture one completed workflow and verify that task, session, and operator
views map back to the same canonical result object without inventing competing result structures.

**Acceptance Scenarios**:

1. **Given** a completed workflow, **When** `GET /api/v1/tasks/{task_id}` is inspected, **Then**
   `task.result` is the task-facing result envelope derived from the canonical runtime result.
2. **Given** the same workflow, **When** runtime metadata is inspected, **Then**
   `metadata.final_result` is the authoritative persisted result object and
   `metadata.aggregated_result` remains its execution-produced payload rather than a competing
   top-level contract.
3. **Given** the same workflow in session context, **When** retained session data is inspected,
   **Then** `assistant_result` is a session-facing projection derived from the canonical result
   object rather than a separate ad hoc shape.

---

### User Story 3 - Expose Bounded Result Preview And Provenance On Proof-Relevant Surfaces (Priority: P2)

An operator can verify what happened without dumping full payloads or reconstructing the answer
from low-level step artifacts.

**Why this priority**: the current surfaces are strong on structural proof but weak on bounded
final-result inspection.

**Independent Test**: inspect one completed workflow through task, session, and operator surfaces
and verify they show enough preview and provenance to confirm the actual outcome without exposing
the full payload by default.

**Acceptance Scenarios**:

1. **Given** a completed workflow, **When** task status is inspected, **Then** the surface exposes
   the unified result envelope plus the provenance needed to trust where it came from.
2. **Given** a session with completed turns, **When** session state is inspected, **Then** the
   retained result view exposes the same canonical answer summary and provenance without dropping
   prior assistant-result material.
3. **Given** an operator autonomy inspection, **When** the workflow is rendered, **Then** the
   surface shows a compact result preview and provenance summary correlated with topology and
   outcome classification rather than a raw payload dump.

---

### User Story 4 - Keep Collapse And Failure Boundaries Visible And Classifiable (Priority: P2)

An operator or evaluator can distinguish success, collapse, and planner-time failure from stored
evidence without guessing.

**Why this priority**: the March 19 evaluation notes showed that the system can succeed, collapse
to sequential, or fail before graph formation, and all three outcomes matter to honest proof.

**Independent Test**: run or replay the proof matrix and verify each outcome is recorded as one of
the packet's explicit proof classes.

**Acceptance Scenarios**:

1. **Given** a harder workload that forms a graph and completes, **When** the run is classified,
   **Then** it is recorded as `graph_formed_and_completed`.
2. **Given** a workload the planner compresses to one sequential step, **When** the run is
   classified, **Then** it is recorded as `collapsed_to_sequential`.
3. **Given** a workload that times out or fails before graph formation, **When** the run is
   classified, **Then** it is recorded as `failed_before_graph`.

### Edge Cases

- a workload forms a graph but still yields a thin or compressed final answer
- `task.result` and `metadata.final_result` drift because one surface is updated without the other
- session retained context carries stale `assistant_result` after later projection changes
- operator status needs to preview the result without dumping the full answer payload
- a planner-visible failure occurs before autonomy status can expose a graph
- a workload collapses to sequential execution but still produces a valid final result
- the new result projections accidentally touch the bounded MCP capability surface and require a
  non-regression check

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST preserve the March 19 baseline truth: supervised planner and executor,
  `tool_bus` execution boundary, autonomy topology and routing visibility, and the bounded MCP
  capability surface remain current truth rather than new scope.
- **FR-002**: System MUST define one shared result contract that explicitly relates
  `task.result`, metadata `final_result`, metadata `aggregated_result`, session `assistant_result`,
  and operator preview and provenance output.
- **FR-003**: System MUST treat metadata `final_result` as the canonical persisted runtime result
  object for completed workflows.
- **FR-004**: System MUST treat `aggregated_result` as the execution-produced payload nested inside
  the canonical runtime result object, not as a competing top-level contract.
- **FR-005**: System MUST expose `task.result` as the task-facing result envelope derived from the
  canonical runtime result object.
- **FR-006**: System MUST expose session-facing retained result material as a projection derived
  from the canonical runtime result object rather than an unrelated shape.
- **FR-007**: System MUST expose an operator-facing preview and provenance projection derived from
  the same canonical runtime result object without requiring a raw full-payload dump by default.
- **FR-008**: System MUST define one proof outcome taxonomy with exactly these classes:
  `graph_formed_and_completed`, `collapsed_to_sequential`, and `failed_before_graph`.
- **FR-009**: System MUST make success, collapse, and failure-visible outcomes distinguishable from
  stored runtime-facing evidence and evaluation artifacts.
- **FR-010**: System MUST cover harder workload proof on the default live path and describe runtime
  adaptations only as needed to produce honest success, collapse, and failure-visible evidence.
- **FR-011**: System MUST preserve the current task and session runtime surfaces; this packet
  extends existing surfaces rather than introducing a new runtime or operator subsystem.
- **FR-012**: System MUST provide repeatable evaluation proof under `docs/plans/` that captures
  all three outcome classes for representative workloads.
- **FR-013**: System MUST keep provider-neutral routing, JetStream KV and budget follow-up, and
  broader external-agent expansion explicitly out of scope for this packet.
- **FR-014**: System MUST require an MCP and external-agent non-regression check only if the new
  result surfaces intersect the existing bounded post-`MS-77` capability surface.
- **FR-015**: System MUST define explicit write-set boundaries for `[P]` tasks so runtime proof,
  result projection, and evaluation lanes only run in parallel when their files are disjoint.

### Key Entities *(include if feature involves data)*

- **UnifiedResultEnvelope**: the canonical runtime result contract rooted at metadata
  `final_result` and projected onto task, session, and operator surfaces.
- **TaskResultView**: the task-facing result envelope exposed through `task.result`.
- **SessionRetainedResultView**: the retained session-facing projection derived from the canonical
  result object and stored as `assistant_result`.
- **OperatorResultPreview**: a compact result preview and provenance block rendered alongside the
  existing autonomy structure.
- **ProofOutcomeClassification**: one of `graph_formed_and_completed`,
  `collapsed_to_sequential`, or `failed_before_graph`.
- **EvaluationHarnessRun**: durable evidence tying workload class, outcome classification, and
  proof artifact path together.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The packet preserves current baseline truth on `main` and does not reopen provider,
  budget, KV, or broad external-agent programs as part of this epic.
- **SC-002**: The packet defines one explicit contract showing how `task.result`,
  `final_result`, `aggregated_result`, `assistant_result`, and operator preview/provenance relate
  to each other.
- **SC-003**: The packet defines a proof matrix that covers all three runtime outcomes:
  `graph_formed_and_completed`, `collapsed_to_sequential`, and `failed_before_graph`.
- **SC-004**: The packet requires at least one harder-workload proof case that forms a real graph
  and at least one case each for collapse and failure-visible behavior.
- **SC-005**: The packet requires result preview and provenance on task, session, and operator
  surfaces without turning operator inspection into a raw full-payload dump.
- **SC-006**: The packet defines a durable evaluation artifact under `docs/plans/` that records
  workload class, proof outcome, and enough result evidence to verify what happened later.
