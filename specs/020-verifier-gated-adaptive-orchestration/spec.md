# Feature Specification: Verifier-Gated Adaptive Orchestration

**Feature Branch**: `020-verifier-gated-adaptive-orchestration`
**Created**: 2026-03-26
**Status**: Draft
**Input**: `docs/current-state.md`,
`docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md`,
`docs/research-output/research/discovery-sweep-R4.md`,
`docs/research-output/research/discovery-sweep-R7b.md`,
`docs/research-output/research/targeted-step-level-intelligence-R6.md`,
`docs/research-output/research/targeted-supervision-fault-tolerance-R4.md`,
`docs/research-prompts/R8/06-predictive-supervision.md`, and the current runtime surfaces in
`crates/mister-smith-app/src/execution.rs`, `crates/mister-smith-app/src/autonomy.rs`, and
`crates/mister-smith-core/src/autonomy.rs`

## Current Truth & Scope

Current repo truth already includes:

- supervised planner and executor lifecycles on the default runtime-backed task path
- ToolBus-backed execution, task and autonomy provenance, and same-agent session handling
- bounded runtime routing and budget-control-loop work through packet `019`
- operator-visible workflow results, proof-outcome taxonomy, and repeatable local smoke proof on
  the shipped `openai_chatgpt` / `gpt-5.4` baseline

The remaining gap is narrower than a broad benchmark or research program:

- the runtime path still lacks a first-class verifier gate between intermediate workflow steps
- there is no explicit clarification path for bad inter-agent or inter-step handoffs
- there is no first-class repair directive that can preserve failure context and re-plan from the
  last stable checkpoint instead of restarting the whole task
- task and autonomy surfaces do not yet explain verifier and repair outcomes as first-class
  workflow provenance

This packet therefore freezes one bounded slice:

1. add a verifier-gated workflow-step contract on the runtime-backed task path
2. add bounded clarification and repair actions for weak or incomplete handoffs
3. preserve failure context and last stable checkpoint for retry or re-plan
4. surface orchestration-quality provenance on task and autonomy views

This is not:

- provider selection, provider benchmarking, or provider-proof widening
- budget policy expansion or budgeting-focused orchestration work
- a new benchmark harness, leaderboard claim, or broad workflow-evolution engine
- a decentralized topology rewrite, CRDT program, or operator-console redesign

## User Scenarios & Testing

### User Story 1 - Gate workflow progression with verifier verdicts (Priority: P1)

An operator or developer runs a task on the existing runtime path, and the system can accept or
reject a workflow step based on a first-class verifier verdict before blindly progressing.

**Independent Test**: targeted app/core tests prove that an accepted step progresses, a rejected
step blocks progression, and the current happy path stays intact when the verifier loop is not
active.

**Acceptance Scenarios**:

1. **Given** a step output that satisfies the verifier contract, **When** the workflow advances,
   **Then** the step is accepted and the next step receives the accepted context with preserved
   provenance.
2. **Given** a step output that fails verification, **When** the workflow reaches the gate,
   **Then** the system emits a bounded repair directive instead of silently continuing.
3. **Given** the verifier policy is omitted or disabled, **When** the current happy path runs,
   **Then** the runtime preserves today's shipped behavior.

### User Story 2 - Clarify and repair failing handoffs locally (Priority: P1)

An execution branch encounters a weak or incomplete handoff, and the runtime can clarify, retry,
or re-plan from the last stable checkpoint without throwing away the whole task.

**Independent Test**: targeted runtime tests exercise clarify, retry, re-plan, and stop outcomes
with bounded retry budgets and preserved failure context.

**Acceptance Scenarios**:

1. **Given** a handoff is missing required context, **When** the verifier rejects it, **Then**
   the runtime can emit a clarification request instead of guessing.
2. **Given** a step fails after a valid earlier checkpoint exists, **When** the repair policy
   chooses re-plan, **Then** the runtime resumes from the last stable checkpoint instead of
   restarting the whole workflow.
3. **Given** repeated clarification or retry attempts fail, **When** the retry budget is
   exhausted, **Then** the runtime stops or escalates honestly instead of looping forever.

### User Story 3 - Inspect orchestration-quality outcomes without log archaeology (Priority: P2)

An operator inspects a task or autonomy view and can see what the verifier accepted, what required
clarification or repair, and what stable checkpoint anchored the final result.

**Independent Test**: task/autonomy inspection output includes verifier and repair provenance, and
packet docs keep deterministic validation versus any live proof explicitly separated.

**Acceptance Scenarios**:

1. **Given** a workflow used the verifier-gated path, **When** an operator inspects status,
   **Then** the response includes verifier verdict, repair action, and last stable checkpoint.
2. **Given** the packet lands deterministic control-loop behavior before a live runtime proof,
   **When** the packet closes, **Then** the docs record the proof boundary honestly instead of
   implying benchmark or live-proof claims that were not earned.

## Edge Cases

- the verifier times out or returns an inconclusive verdict
- a clarification request loops without adding new information
- a downstream verifier discovers that an earlier accepted step omitted a critical constraint
- a repair directive arrives after the next step has already begun
- a session or runtime restart occurs between rejection and local repair

## Requirements

### Functional Requirements

- **FR-001**: System MUST add a typed verifier verdict surface for runtime workflow steps.
- **FR-002**: System MUST add a typed repair directive surface that can choose `retry_step`,
  `clarify_handoff`, `replan_from_checkpoint`, or `stop`.
- **FR-003**: System MUST preserve today's shipped happy path when the verifier-gated loop is not
  active.
- **FR-004**: System MUST preserve failure context and the last stable checkpoint when a step is
  rejected.
- **FR-005**: System MUST make clarification a first-class workflow action rather than implicit
  prompt drift.
- **FR-006**: System MUST bound local retry and clarification loops with explicit budgets or stop
  conditions.
- **FR-007**: System MUST surface verifier and repair provenance on task and autonomy views.
- **FR-008**: System MUST keep the write set bounded to workflow-state contracts, runtime
  execution, inspection surfaces, targeted validation, and state-bearing docs.
- **FR-009**: System MUST fail explicitly when verifier or repair state is invalid or cannot be
  reconciled with the active workflow step.
- **FR-010**: System MUST NOT widen into provider work, budgeting work, benchmark harness
  construction, or workflow-evolution search.
- **FR-011**: System MUST keep deterministic validation and any live-proof claims explicitly
  separated.

### Key Entities

- **StepEvaluationRecord**: verifier-owned record for one workflow step, including verdict,
  rationale, confidence, and referenced failure context
- **HandoffClarificationRequest**: structured request describing what a downstream step is missing
  before execution can continue safely
- **RepairDirective**: bounded runtime action describing how to proceed after rejection
- **FailureContextCheckpoint**: durable reference to the last accepted stable state plus rejection
  diagnostics
- **OrchestrationQualityView**: operator-facing projection of verifier and repair history

## Success Criteria

- **SC-001**: the runtime can accept or reject intermediate workflow steps without regressing the
  current happy path
- **SC-002**: one failing handoff can be clarified, retried, or replanned from the last stable
  checkpoint without full-task restart by default
- **SC-003**: task and autonomy views expose verifier and repair provenance without raw log
  archaeology
- **SC-004**: targeted deterministic validation proves the bounded control loop
- **SC-005**: packet docs remain honest about benchmark and live-proof boundaries
