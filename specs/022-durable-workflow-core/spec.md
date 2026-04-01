# Feature Specification: Durable Workflow Core

**Feature Branch**: `022-durable-workflow-core`
**Created**: 2026-04-01
**Status**: Implementation-ready
**Input**: `docs/direction.md`, `docs/current-state.md`,
`docs/research-output/analysis/2026-03-28-durable-workflows-transfer-brief.md`, and current
durability seams in `crates/mister-smith-agents/src/branch_checkpoint.rs`,
`crates/mister-smith-persistence/src/kv/state.rs`,
`crates/mister-smith-persistence/src/hybrid/manager.rs`,
`crates/mister-smith-app/src/conversation.rs`, and
`docs/plans/2026-03-19-session-restart-resume-live-proof.md`

## Current Truth & Scope

Packet `022` is now the frozen implementation packet for durable workflow core on current
`main`.

It is ready for `/speckit.implement`. The open design points called out below are first-slice
narrowing decisions, not a reason to stop or defer implementation.

Current repo truth already includes:

- live workflow execution on the supported runtime path
- bounded same-agent sessions with stable `session_id` and `coordinator_agent_id`
- restart-resume continuity already proven on the supported `openai_chatgpt` / `gpt-5.4` path
- branch checkpoint capture, resume metadata, and KV-plus-SQL durability helpers already landed in
  the repo
- packet `020` verifier and repair lineage, plus packet `021` deterministic supervision evidence,
  as adjacent runtime context

What is still missing is one frozen durable workflow contract for:

- event-history semantics
- replay-safe state transitions
- lifecycle verbs and their meanings
- idempotent effect boundaries
- bounded compaction and replay-governance rules

This packet stays strictly inside that seam. It does not cover:

- coordinator-runtime expansion
- interoperability or federation work
- strong coordination or consensus work
- a Temporal clone or Azure Durable Functions clone
- any claim that this packet itself makes the feature live by default

## Clarifications

### Session 2026-04-01

- Q: Is packet `022` implementation-ready now? → A: Yes. It is the active implementation packet
  for durable workflow semantics, effect boundaries, and lifecycle control on current `main`.
- Q: Are Temporal and Azure Durable Functions the target architecture? → A: No. They are semantic
  comparators only.
- Q: Which adjacent areas are deliberately deferred out of this packet? → A: Coordinator-runtime
  expansion, interoperability, strong coordination, and any new live-default claim.

## Open Design Questions For The First Slice

These questions are explicit on purpose. They should be resolved in the first bounded
implementation slice instead of being hidden behind fake certainty.

- What is the exact repo-native shape of one durable workflow history event?
- What is the first bounded compaction mechanism: rollup event, snapshot record, KV pointer, or a
  hybrid?
- Where should the first intent/effect boundary live across PostgreSQL, JetStream, and existing
  runtime ownership seams?
- What replay-regression fixture model should the repo use to keep history-version changes honest?

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Recover From Durable History (Priority: P1)

An operator or runtime service can rebuild workflow state from durable history after interruption
without losing the accepted branch, node, or session lineage that already exists today.

**Why this priority**: This is the core substrate gain. If replay-safe recovery is not frozen
first, later autonomy work keeps building on partial durability.

**Independent Test**: An interrupted workflow can be replayed more than once from the same durable
history and land in the same accepted lifecycle state without duplicating already-accepted
transitions.

**Acceptance Scenarios**:

1. **Given** a workflow already recorded accepted history for branch and node progress, **When**
   the runtime reconstructs the workflow after interruption, **Then** it reaches the same durable
   state without reapplying already-accepted transitions.
2. **Given** a session turn already has restart-resume lineage on the supported path, **When**
   durable workflow recovery is applied, **Then** the same `session_id` and
   `coordinator_agent_id` continuity remains intact.

---

### User Story 2 - Protect External Effects During Replay (Priority: P1)

An operator or runtime service can retry and replay workflow progress without treating external
side effects as safe to duplicate.

**Why this priority**: Broker deduplication alone is not enough. Durable state and effect
correctness must be separated before long-running workflows widen.

**Independent Test**: Reprocessing the same effect boundary after interruption does not create a
second operator-visible external outcome when durable completion evidence already exists.

**Acceptance Scenarios**:

1. **Given** a workflow already recorded effect completion for one external action, **When** the
   same boundary is encountered during replay or retry, **Then** the system treats that effect as
   already handled instead of applying it again.
2. **Given** the workflow recorded effect intent but not durable completion, **When** recovery
   runs, **Then** the lifecycle state makes the effect boundary explicit rather than silently
   assuming success.

---

### User Story 3 - Control Lifecycle With Clear Verbs (Priority: P2)

An operator can use one clear lifecycle vocabulary for durable workflow control instead of
different meanings on task, session, and autonomy surfaces.

**Why this priority**: Long-running durability is not trustworthy if pause, resume, cancel,
terminate, and reset-like behavior mean different things on different surfaces.

**Independent Test**: A bounded lifecycle scenario can be driven through task, session, and
autonomy views with one consistent meaning for the selected lifecycle verb and resulting state.

**Acceptance Scenarios**:

1. **Given** a workflow is in a resumable durable state, **When** an operator applies a lifecycle
   command, **Then** task, session, and autonomy surfaces agree on the resulting state and meaning.
2. **Given** the same lifecycle command is issued more than once, **When** the command is handled,
   **Then** the durable result remains stable instead of oscillating or duplicating state changes.

---

### User Story 4 - Keep Replay Bounded Over Time (Priority: P3)

An operator or runtime service can keep workflow replay cost bounded for long-running histories
without losing the lineage needed for inspection and recovery.

**Why this priority**: Durable workflows become operationally unsafe if replay cost grows without a
bound.

**Independent Test**: One long-running workflow scenario can compact or roll up history at least
once and still resume correctly from the compacted lineage.

**Acceptance Scenarios**:

1. **Given** a workflow history crosses the packet's bounded compaction threshold, **When**
   compaction runs, **Then** active execution remains resumable from the compacted lineage.
2. **Given** a workflow used compaction, **When** an operator inspects its recovery path, **Then**
   enough prior lineage remains available to explain the current durable state.

### Edge Cases

- A lifecycle command arrives while an external effect is in an unknown completion state.
- The same lifecycle command is repeated after the workflow is already terminal.
- A replay path encounters history written before a later packet changes adjacent runtime seams.
- A compacted workflow must still preserve existing session restart-resume lineage.
- A late effect-completion signal arrives after recovery already rebuilt the workflow state.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST define one canonical durable workflow history model for accepted
  workflow, branch, node, and lifecycle state changes.
- **FR-002**: System MUST define replay-safe transition rules so the same accepted history always
  produces the same durable workflow state.
- **FR-003**: System MUST preserve current session continuity and restart-resume behavior and MUST
  NOT require new session identifiers or coordinator reassignment for the current happy path.
- **FR-004**: System MUST define explicit effect boundaries that separate accepted state
  transitions from external side effects.
- **FR-005**: System MUST record enough durable effect intent and effect completion state to tell
  the difference between "not started", "completion unknown", and "already completed" outcomes.
- **FR-006**: System MUST freeze one explicit lifecycle vocabulary for pause, resume, cancel,
  terminate, and reset or rewind posture, including any verbs that are intentionally deferred.
- **FR-007**: System MUST keep lifecycle meanings consistent across task, session, and autonomy
  surfaces.
- **FR-008**: System MUST define a bounded compaction or rollup rule so replay cost does not grow
  without limit for long-running workflows.
- **FR-009**: System MUST define version-safe replay rules and replay-regression gates before the
  durable history surface widens further.
- **FR-010**: System MUST keep exactly-once state-transition claims separate from effectively-once
  effect-outcome claims.
- **FR-011**: System MUST treat Temporal and Azure Durable Functions as comparator sources only
  and MUST NOT require structural cloning of either system.
- **FR-012**: System MUST stay scoped to durable workflow semantics, effect boundaries, and
  lifecycle control, and MUST NOT absorb coordinator-runtime, interoperability, or strong
  coordination work.
- **FR-013**: System MUST keep unresolved first-slice choices explicit and resolve them in the
  first bounded implementation slice instead of hiding them behind vague wording.
- **FR-014**: System MUST keep live truth, landed-not-default substrate, deterministic-only proof,
  and planned packet work clearly separated in packet wording and downstream notes.

### Key Entities *(include if feature involves data)*

- **WorkflowHistoryEvent**: One accepted durable record of workflow progress or lifecycle change,
  carrying the minimum identity and lineage needed to rebuild state.
- **WorkflowProjection**: The current durable view rebuilt from workflow history and used to show
  task, session, and autonomy state consistently.
- **EffectBoundaryRecord**: Durable intent and outcome tracking for one external side effect,
  allowing replay and retry to avoid duplicate outcomes.
- **LifecycleCommand**: One operator-visible request to change durable workflow state, such as
  pause, resume, cancel, terminate, or reset-like recovery action.
- **LifecycleDecision**: The accepted durable result of one lifecycle command, including any
  allowed no-op or deferred outcome.
- **HistoryCompactionRecord**: Durable lineage that lets a compacted workflow stay resumable and
  still explain how the current state was reached.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A bounded interrupted workflow can be replayed at least twice from the same durable
  history and reaches the same accepted lifecycle state each time.
- **SC-002**: A bounded replay or retry scenario with an already-completed effect boundary does
  not create a second operator-visible external outcome.
- **SC-003**: Task, session, and autonomy surfaces expose one consistent lifecycle meaning for the
  same workflow outcome in bounded validation scenarios.
- **SC-004**: A long-running workflow scenario can compact or roll up history at least once and
  still resume correctly from the compacted lineage.
- **SC-005**: The existing session restart-resume proof case remains valid after the durable
  workflow core is implemented.

## Assumptions

- Packet `022` is the current implementation authority for durable workflow core on `main`, even
  if adjacent packet work continues elsewhere.
- The first compaction mechanism may be intentionally minimal as long as it keeps replay bounded
  and keeps recovery lineage inspectable.
- The first effect-boundary slice covers runtime-visible external side effects and does not attempt
  to solve general distributed transactions.
- Existing session continuity and restart-resume proof remain the baseline behavior that packet
  `022` must preserve rather than redesign.
