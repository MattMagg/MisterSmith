# Feature Specification: Task-Shape-Aware Orchestration and Dynamic Team Sizing

**Feature Branch**: `014-task-shape-aware-orchestration`  
**Created**: 2026-03-16  
**Status**: Draft  
**Input**: `MS-45`, landed `MS-60`, backlog slices `MS-61` and `MS-62`,
`docs/plans/2026-03-16-frontier-direction.md`, `specs/012-phase10-frontier-autonomy/`,
`specs/013-multi-turn-same-agent-conversations/`, current orchestration and autonomy code in
`crates/mister-smith-agents/`, `crates/mister-smith-app/`, and `crates/mister-smith-events/`

## Current Truth & Scope

This packet formalizes the next operating-system feature after the March 16 recovery and the
bounded same-agent session slice.

Current repo truth on `main` already includes the first `MS-45` slice:

- execution graphs are compiled before dispatch
- dependency shape is classified into task-shape signals
- topology rationale is preserved in typed autonomy status
- routing already reasons about dependency depth, health, and budget pressure

That landed work is the durable baseline from `MS-60`, not future scope.

The unfinished feature gap is narrower:

- the active worker pool is still treated as an input to routing rather than an adaptive operating
  decision
- team sizing is not yet a first-class, operator-visible contract
- there is no repeatable evaluation harness that proves when adaptive sizing beats a fixed or
  sequential posture

This packet therefore treats `MS-45` as one feature with three bounded stories:

1. landed task-shape classification and topology rationale (`MS-60`)
2. unfinished dynamic team sizing and lifecycle integration (`MS-61`)
3. unfinished operator-visible rationale and evaluation harness (`MS-62`)

This is **not** a new roadmap phase claim. It is the first full post-`013` SpecKit packet layered
on top of the completed Phase 10 substrate.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Classify Task Shape Before Dispatch (Priority: P1)

An operator submits a decomposed workflow and the runtime classifies dependency shape before any
branch is dispatched, selecting a topology whose rationale can be inspected later.

**Why this priority**: This is the minimum substrate for the rest of the feature and is already
the honest current truth on `main`.

**Independent Test**: Compile representative parallel, sequential, and mixed-dependency plans and
verify the resulting execution graph exposes task shape plus topology rationale before branch
dispatch begins.

**Acceptance Scenarios**:

1. **Given** a workflow with multiple independent branches, **When** the planner output is
   compiled, **Then** the graph records a parallel-capable task shape and a non-sequential
   topology choice.
2. **Given** a workflow with strict chained dependencies, **When** the graph is compiled,
   **Then** the runtime records a sequential or pipeline-compatible topology instead of widening
   the team without need.
3. **Given** malformed or cyclic planner output, **When** topology selection runs, **Then** the
   workflow is rejected before dispatch and no branch routing begins.

---

### User Story 2 - Size Active Teams From Task Structure (Priority: P1)

An operator runs a workflow and the runtime chooses how many workers to activate from the graph's
branch width, dependency depth, available capacity, and current conservative posture instead of
defaulting to a fixed role fan-out.

**Why this priority**: Task-shape-aware topology is not enough if the runtime still behaves like a
static team shell. Dynamic team sizing is the main missing operating-system behavior.

**Independent Test**: Run at least two representative workflow shapes with different dependency
profiles and verify the runtime chooses different active worker counts while preserving scheduling
and supervision invariants.

**Acceptance Scenarios**:

1. **Given** a wide fan-out workflow and enough available workers, **When** the runtime prepares
   dispatch, **Then** it activates more than one worker and records why that width was justified.
2. **Given** a narrow or deeply sequential workflow, **When** the runtime prepares dispatch,
   **Then** it keeps the active team minimal instead of inflating the worker pool.
3. **Given** conservative mode, degraded health, or high budget pressure, **When** the sizing
   decision is made, **Then** the runtime narrows the active team size and records the cap
   rationale.
4. **Given** branch completion, join convergence, or recovery on a later frontier, **When** the
   runtime re-evaluates active work, **Then** scheduler and supervision behavior stay coherent and
   no completed branch is reopened solely because team size changed.

---

### User Story 3 - Expose Adaptive Decisions And Prove Them (Priority: P2)

An operator can inspect why the runtime chose a topology and team size, and a developer can run a
repeatable evaluation harness that compares adaptive execution against a fixed or sequential
baseline for representative workload classes.

**Why this priority**: Without explicit operator visibility and repeatable evidence, adaptive team
selection is just a hidden heuristic.

**Independent Test**: Inspect a workflow autonomy status view that includes team-size rationale,
then run the evaluation harness and verify it records a reproducible comparison artifact for at
least one sequential and one parallel workload class.

**Acceptance Scenarios**:

1. **Given** a completed or running workflow, **When** the operator inspects autonomy status,
   **Then** they can see task shape, selected topology, selected worker count, cap reason, and the
   main structural signals behind the decision.
2. **Given** a deterministic evaluation workload bundle, **When** the harness compares adaptive
   sizing against a fixed or sequential baseline, **Then** it records the chosen team size,
   workload class, and observed result in a reproducible artifact.
3. **Given** the adaptive path does not beat the baseline for one workload class, **When** the
   harness completes, **Then** the artifact records the neutral or negative result honestly rather
   than claiming improvement everywhere.

### Edge Cases

- worker availability is lower than the desired parallel width
- conservative mode or degraded health forces the selected team size to `1` even when the graph is
  wide
- a graph is structurally parallel but budget pressure requires a narrower team than the maximum
  parallelism width
- recovery resumes a branch after some workers were already released or reassigned
- evaluation runs execute in environments without live provider credentials and must still produce
  deterministic comparison evidence
- operator status is inspected before any routing event has been emitted for a newly accepted
  workflow

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST preserve the landed `MS-60` behavior that classifies task shape before
  branch dispatch and records topology rationale on the execution graph.
- **FR-002**: System MUST compute an explicit team-sizing decision from task-shape structure,
  frontier width, dependency depth, available workers, and conservative operating signals.
- **FR-003**: System MUST activate fewer or more workers based on that decision instead of
  treating the worker set as a fixed fan-out for every workflow.
- **FR-004**: System MUST keep scheduling, supervision, checkpoint, and recovery behavior coherent
  when the active team size varies across workflow shapes or frontiers.
- **FR-005**: System MUST cap selected team size when worker availability, conservative mode,
  health state, or budget pressure requires a narrower posture.
- **FR-006**: System MUST keep current one-shot runtime and workflow-autonomy surfaces valid; this
  feature MUST extend the existing orchestration path rather than introducing a parallel runtime.
- **FR-007**: System MUST expose the selected team size, desired team size, and cap rationale in
  operator-visible autonomy status.
- **FR-008**: System MUST preserve enough typed event data for operator tooling to reconstruct why
  the topology and team size were chosen.
- **FR-009**: System MUST provide a deterministic evaluation harness that compares adaptive sizing
  against at least one non-adaptive baseline on representative workload classes.
- **FR-010**: System MUST record evaluation output in a durable repo artifact under `docs/plans/`
  so later sessions can review the evidence without replaying the whole implementation history.
- **FR-011**: System MUST remain provider-neutral; evaluation may use deterministic planner-output
  fixtures when live provider proof is unnecessary.
- **FR-012**: System MUST not reopen `MS-60` as speculative future work; landed topology-compiler
  behavior is the baseline for this packet.
- **FR-013**: System MUST define explicit write-set boundaries for `[P]` tasks so Symphony only
  runs parallel lanes when the files are disjoint.
- **FR-014**: System MUST treat shared contract files and shared evidence artifacts as single-owner
  choke points that cannot be edited by multiple active Symphony lanes at once.
- **FR-015**: System MUST allow the evaluation harness to report neutral or negative adaptive
  results honestly when a workload class does not benefit from wider team sizing.

### Key Entities *(include if feature involves data)*

- **TaskShapeAssessment**: A durable or reconstructible summary of dependency shape, frontier
  width, depth, and topology rationale derived from the execution graph before dispatch.
- **TeamSizingDecision**: The operator-visible decision object that records desired workers,
  selected workers, capping factors, and rationale used to assemble the active team.
- **AdaptiveTeamPlan**: The runtime-facing mapping from a team-sizing decision to coordinator,
  optional supervisor, and active worker membership for a workflow frontier.
- **AdaptiveDecisionView**: The autonomy-status projection that joins task-shape assessment,
  topology choice, team-sizing decision, routing history, and conservative reasons for operator
  inspection.
- **EvaluationHarnessRun**: The deterministic evidence record comparing adaptive execution against
  a fixed or sequential baseline for one workload class.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The packet preserves the current `MS-60` truth by keeping task-shape classification
  and topology rationale visible on at least one compiled workflow before branch dispatch.
- **SC-002**: At least two representative workload classes produce different selected worker counts
  under the adaptive sizing logic.
- **SC-003**: Conservative mode or degraded operating signals can reduce selected worker count
  deterministically without breaking scheduler or recovery invariants.
- **SC-004**: `mister-smith autonomy status` or the equivalent status surface shows selected team
  size and rationale without requiring raw log inspection.
- **SC-005**: A deterministic evaluation harness writes a durable artifact under `docs/plans/`
  comparing adaptive execution against a baseline and records whether the adaptive path improved,
  matched, or regressed each workload class.
