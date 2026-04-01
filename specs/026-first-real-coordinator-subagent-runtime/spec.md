# Feature Specification: First Real Coordinator-Subagent Runtime

**Feature Branch**: `026-first-real-coordinator-subagent-runtime`
**Created**: 2026-04-01
**Status**: Scaffold Draft
**Input**: `docs/direction.md`, `docs/current-state.md`, `docs/packet-prep/README.md`,
`docs/packet-prep/026-first-real-coordinator-subagent-runtime.md`,
`docs/packet-prep/022-durable-workflow-core.md`,
`docs/packet-prep/023-runtime-truth-and-run-trace.md`,
`docs/packet-prep/024-agent-boundary-security-hardening.md`,
`docs/packet-prep/025-step-level-intelligence-v2.md`,
`docs/2026-03-28-session-context-report.md`,
`docs/plans/2026-03-27-runtime-planning-simplification.md`, and
`docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`

## Current Truth & Scope

This packet is a provisional scaffold written before packets `022` through `025` are complete.
It uses their packet-prep contract ownership as the current source of truth for scope boundaries,
not their final landed implementations.

This scaffold exists to save time later. It is meant to lock the packet goal, non-goals, proof
standard, and main artifact shape now so future work is mostly revision instead of starting from
zero.

Current repo truth already includes:

- live graph and topology formation on the supported runtime path
- routing, provenance, repair, and supervision summaries on operator-facing surfaces
- same-agent session continuity with stable `session_id` and `coordinator_agent_id`
- smallest-workflow behavior that stays sequential unless branching is clearly justified

Current repo truth does not yet include:

- real coordinator-owned delegation records during a run
- real subagent state that operators can inspect during a run
- grounded delegated work below the current placeholder step boundary
- an honest local coordinator-subagent runtime proof standard

This packet therefore freezes one bounded future packet shape:

- visible coordinator-owned delegation
- visible subagent state
- grounded delegated work
- visible feedback, merge, and recovery loops
- explicit proof language that separates graph success from real coordinator-subagent success

### Pre-Implementation Revision Gate

Before any implementation starts for packet `026`, the owner of the packet must:

1. reread `docs/current-state.md`
2. reread `docs/direction.md`
3. confirm what packet `022` through `025` actually landed
4. revise `spec.md`, `plan.md`, `tasks.md`, and `analyze.md` to match that landed truth

This scaffold is not implementation-ready until that revision gate is completed.

### Out Of Scope

- federation, capability discovery, or generic interoperability work
- mandatory fan-out or a fixed multi-worker shape
- redefining packet `022` durability ownership
- redefining packet `023` run-trace or proof-boundary ownership
- redefining packet `024` boundary-hardening ownership
- redefining packet `025` step-policy ownership

## Clarifications

### Session 2026-04-01

- Q: Who owns merge, reassign, clarify, stop, and collapse decisions in this packet? → A: The
  coordinator owns those decisions and the packet must keep them visible.
- Q: What subagent state model does this scaffold assume? → A: `queued`, `delegated`, `running`,
  `blocked`, `clarified`, `reassigned`, `merged`, `completed`, `failed`, and `collapsed`.
- Q: What session continuity carries across coordinator-led follow-up runs? → A: Preserve
  `session_id`, `coordinator_agent_id`, and evidence references, but do not assume transcript
  duplication or unlimited carry-forward.
- Q: What counts as placeholder-only execution for this packet? → A: If delegated work stops at a
  `workflow.execute_step`-style envelope without grounded evidence, it does not satisfy packet
  `026` success.

## User Scenarios & Testing

### User Story 1 - See Real Delegation And Subagent State (Priority: P1)

An operator inspects a runtime-backed run and can see that a coordinator delegated real bounded
work to named subagents, along with each subagent's current state and any coordinator response to
that state.

**Why this priority**: This is the first honest difference between graph metadata and a real
coordinator-subagent runtime. Without it, the packet does not deliver its core value.

**Independent Test**: A bounded run that justifies delegation shows at least one coordinator-owned
delegation record and at least two visible subagent state transitions without requiring log
archaeology.

**Acceptance Scenarios**:

1. **Given** a task that clearly benefits from bounded fan-out, **When** the runtime delegates
   work, **Then** the operator can see a coordinator-owned delegation record for each delegated
   subagent job.
2. **Given** a delegated subagent job moves from waiting to active or blocked work, **When** the
   operator inspects the run, **Then** the current subagent state is visible and attributable to
   that delegated job.
3. **Given** a task does not justify branching, **When** the runtime keeps the work sequential,
   **Then** the run still succeeds honestly and shows collapse or non-delegation instead of fake
   subagent activity.

---

### User Story 2 - Prove Grounded Delegated Work And Feedback Loops (Priority: P1)

An operator or reviewer can tell whether delegated work was actually grounded, how the
coordinator reacted when a subagent stalled or failed, and whether a merge or recovery decision
was taken on real evidence instead of placeholder completion.

**Why this priority**: Packet `026` must prove more than delegation theater. It must show
grounded delegated work and visible feedback loops.

**Independent Test**: A bounded delegated run records grounded evidence for at least one
subagent-owned job and records a visible coordinator decision for one merge, clarify, reassign,
stop, or collapse case.

**Acceptance Scenarios**:

1. **Given** a delegated subagent completes real bounded work, **When** the coordinator inspects
   the result, **Then** the run records grounded delegated work rather than only placeholder step
   completion.
2. **Given** a delegated subagent becomes blocked, fails, or needs clarification, **When** the
   coordinator responds, **Then** the follow-up decision is visible as clarify, reassign, stop, or
   collapse.
3. **Given** delegated work only returns placeholder completion, **When** the run reaches a
   terminal state, **Then** the proof boundary explicitly says the run did not satisfy real
   coordinator-subagent success.

---

### User Story 3 - Inspect Proof Boundaries And Session-Aware Follow-Up (Priority: P2)

An operator can inspect task, autonomy, and run-detail views and understand both the packet's
proof boundary and what session-aware follow-up will preserve if a later coordinator-led run
continues the work.

**Why this priority**: The packet must stay honest. Operators need to know what was proven and
what still depends on later work or on the revision gate.

**Independent Test**: Task result, autonomy status, and run detail all show the same proof
boundary story and the same session carry-forward assumptions for coordinator-led follow-up.

**Acceptance Scenarios**:

1. **Given** a run used real coordinator-owned delegation, **When** an operator inspects task,
   autonomy, or run detail, **Then** all three views present the same proof-boundary story.
2. **Given** a run ended with sequential collapse or partial delegation only, **When** the
   operator inspects the proof view, **Then** the packet clearly states what was and was not
   proven.
3. **Given** a later coordinator-led follow-up run resumes related work, **When** session context
   is reused, **Then** the packet preserves stable identifiers and evidence references without
   implying unlimited transcript carry-forward.

### Edge Cases

- a task starts as a candidate for delegation but collapses back to sequential execution because
  the smallest-workflow rule says fan-out is unnecessary
- a subagent becomes blocked after a delegation record exists but before grounded evidence exists
- a merge decision combines one grounded branch and one failed or placeholder-only branch
- upstream packet `022` through `025` implementations change field names or proof-boundary wording
  before packet `026` implementation begins
- a coordinator-led follow-up run tries to reuse session context after the upstream revision gate
  changes the contract
- operator surfaces show graph completion for a run that still fails the packet `026` proof
  standard

## Requirements

### Functional Requirements

- **FR-001**: System MUST define packet `026` as a provisional scaffold packet and MUST state
  that it requires revision before implementation begins.
- **FR-002**: System MUST keep current live graph, topology, routing, provenance, and session
  continuity truth separate from the still-missing grounded coordinator-subagent runtime.
- **FR-003**: System MUST require coordinator-owned delegation records as first-class runtime
  evidence for any run that claims coordinator-subagent behavior.
- **FR-004**: System MUST require visible subagent state transitions for delegated work.
- **FR-005**: System MUST require grounded delegated work evidence before a run can satisfy the
  packet `026` proof standard.
- **FR-006**: System MUST treat placeholder-only delegated completion as non-grounded and
  insufficient for packet success.
- **FR-007**: System MUST preserve the smallest-workflow rule and MUST allow honest sequential
  collapse when fan-out is not justified.
- **FR-008**: System MUST make coordinator merge, clarify, reassign, stop, and collapse decisions
  visible to the operator when they occur.
- **FR-009**: System MUST expose packet `026` proof-boundary language on task result, autonomy
  status, and operator-console run detail.
- **FR-010**: System MUST consume packet `022` through `025` ownership by reference and MUST NOT
  redefine lifecycle, run-trace, security-boundary, or step-policy ownership inside packet `026`.
- **FR-011**: System MUST keep federation, capability discovery, and generic interoperability work
  out of this packet.
- **FR-012**: System MUST include an explicit pre-implementation revision gate that reconciles
  packet `022` through `025` landed truth before any coding starts.
- **FR-013**: System MUST define the session-aware follow-up contract in terms of stable
  identifiers and evidence references, not unlimited transcript reuse.
- **FR-014**: System MUST keep the scaffold decision-useful enough that future work mainly revises
  it instead of reauthoring the packet from scratch.
- **FR-015**: System MUST defer any implementation-ready validation or live runtime proof claims
  until the revision gate is completed.

### Key Entities

- **CoordinatorDelegationRecord**: a visible record that the coordinator assigned one bounded job
  to a specific subagent, including job intent, scope, and downstream evidence references
- **SubagentStateRecord**: the current and previous visible state for one delegated subagent job,
  including blocked, clarified, reassigned, merged, failed, or collapsed outcomes
- **DelegatedWorkEvidenceRef**: a reference that ties one delegated job to the grounded evidence,
  proof boundary, or placeholder-only result that job produced
- **CoordinatorMergeDecision**: a visible coordinator-owned decision that explains how delegated
  outputs were merged, clarified, reassigned, stopped, or collapsed
- **CoordinatorRuntimeProofView**: the operator-facing proof summary that joins delegation,
  subagent state, delegated work evidence, merge or recovery decisions, and the current proof
  boundary

## Success Criteria

### Measurable Outcomes

- **SC-001**: `spec.md`, `plan.md`, `tasks.md`, and `analyze.md` all repeat the same packet goal,
  non-goals, proof standard, and pre-implementation revision gate with no conflicting language.
- **SC-002**: The scaffold clearly distinguishes current live graph/runtime truth from the
  missing grounded coordinator-subagent runtime truth in every core artifact.
- **SC-003**: The scaffold names all required evidence types and all required operator surfaces so
  a later implementer can map them to concrete work without re-scoping the packet.
- **SC-004**: The scaffold contains no language that implies packet `022` through `025` are
  already complete, frozen, or safe to ignore before packet `026` implementation.
- **SC-005**: The scaffold contains no language that widens packet `026` into federation,
  capability discovery, generic interoperability, or mandatory fan-out behavior.

## Assumptions

- Packets `022` through `025` are still in progress, so packet `026` uses dossier truth and
  current repo truth rather than final upstream packet outputs.
- The supported runtime path still proves graph and workflow mechanics more strongly than grounded
  delegated work.
- Task result, autonomy status, and operator-console run detail remain the main operator-facing
  surfaces for this packet.
- Session-aware follow-up will preserve `session_id`, `coordinator_agent_id`, and evidence
  references unless upstream packet revisions explicitly change that contract.
- Packet `026` will be revised before implementation, so exact field names and operator wording
  may change once packet `022` through `025` land.
