# Feature Specification: First Real Coordinator-Subagent Runtime

**Feature Branch**: `026-first-real-coordinator-subagent-runtime`
**Created**: 2026-04-01
**Status**: Implementation-ready
**Input**: `docs/current-state.md`, `docs/direction.md`,
`docs/2026-03-28-session-context-report.md`,
`docs/plans/2026-03-27-runtime-planning-simplification.md`,
`docs/plans/2026-03-15-first-live-multi-agent-runtime-proof.md`,
`docs/plans/2026-04-02-openclaude-transfer-analysis.md`,
`docs/research-output/analysis/2026-04-02-openclaude-transfer/04-priority-backlog.md`,
`specs/022-durable-workflow-core/`, `specs/023-runtime-truth-and-run-trace/`,
`specs/024-agent-boundary-security-hardening/`, `specs/025-step-level-intelligence-v2/`, and the
current runtime seams in `crates/mister-smith-core/src/autonomy.rs`,
`crates/mister-smith-agents/src/execution_graph.rs`,
`crates/mister-smith-agents/src/orchestrator.rs`,
`crates/mister-smith-agents/src/roles/coordinator.rs`,
`crates/mister-smith-agents/src/roles/planner.rs`,
`crates/mister-smith-agents/src/roles/worker.rs`,
`crates/mister-smith-agents/src/roles/critic.rs`,
`crates/mister-smith-app/src/execution.rs`, `crates/mister-smith-app/src/autonomy.rs`,
`crates/mister-smith-app/src/conversation.rs`, `crates/mister-smith-events/src/autonomy.rs`,
`crates/mister-smith-events/src/bus.rs`, `crates/mister-smith-app/tests/autonomy_status_tests.rs`,
`crates/mister-smith-app/tests/effect_boundary_projection_tests.rs`,
`crates/mister-smith-agents/tests/execution_graph_tests.rs`,
`crates/mister-smith-agents/tests/team_tests.rs`,
`crates/mister-smith-events/tests/autonomy_event_tests.rs`,
`apps/operator-console/src/types.ts`, `apps/operator-console/src/views/RunsView.tsx`, and
`apps/operator-console/src/App.test.tsx`

## Current Truth And Scope

Packet `026` is now the frozen implementation packet for the first real
coordinator-subagent runtime on current `main`.

It is ready for `/speckit.implement`. The open choices called out below are bounded coding choices,
not a reason to stop for another scaffold pass.

Current repo truth already includes the foundations this packet must extend rather than reopen:

- packet `022` already owns durable workflow lifecycle, event-history, compaction, and
  effect-boundary semantics
- packet `023` already owns `runtime_truth`, `proof_boundary`, and bounded `run_trace`
  projection across task, session, autonomy, and operator surfaces
- packet `024` already owns least-privilege delegated authority, quarantine reporting, and
  boundary-hardening semantics
- packet `025` already owns step-policy summaries on task, autonomy, and operator run-detail
  surfaces
- the current runtime path already forms execution graphs, classifies topology, preserves the
  smallest-workflow rule, and defaults to honest sequential collapse unless branching is clearly
  justified
- same-agent session continuity already preserves stable `session_id` and
  `coordinator_agent_id` on the supported path

What is still missing is one bounded packet that makes the runtime honestly look like a real
coordinator-subagent system:

- visible coordinator-owned delegation records during the run
- visible subordinate inbox intake for child completion, blocked, clarify, cancel, and sibling
  abort signals
- stable delegated child identity that survives clarify, resume, stop, and inspect follow-up
- grounded delegated work evidence below the current placeholder step boundary
- visible coordinator merge, clarify, reassign, stop, and collapse decisions
- one packet-owned proof view that tells operators whether the run actually satisfied
  coordinator-subagent success

Packet `026` therefore owns exactly four bounded outcomes:

1. visible coordinator-owned delegation and subordinate inbox intake
2. visible delegated child identity and subagent state
3. grounded delegated work plus explicit coordinator feedback and merge or recovery loops
4. one honest proof-boundary summary projected through existing task, autonomy, and run-detail
   surfaces

Packet `026` does not own:

- federation, capability discovery, or generic interoperability work
- mandatory fan-out or a fixed multi-worker shape
- a runtime or operator redesign outside the existing read surfaces
- packet `022` durability ownership
- packet `023` runtime-truth or proof-boundary ownership
- packet `024` boundary-hardening ownership
- packet `025` step-policy ownership
- a new live runtime-proof claim unless a real rerun is executed later

## OpenClaude Transfer Inputs

Packet `026` should take a few narrow runtime ideas from the refreshed OpenClaude transfer bundle
and ignore the rest of the app-shell toolbox.

Take these into packet `026`:

- a coordinator-owned subordinate inbox so child completion, blocked, clarify, cancel, and
  sibling-abort signals can re-enter the parent run visibly
- stable child identity so one delegated work unit can be clarified, resumed, stopped, or
  inspected without inventing a new child
- private child scratch context with only root-owned shared channels for registration,
  cancellation, runtime-truth projection, and capability enforcement
- deterministic ordered parallel child execution, including explicit sibling-cancel and
  user-interrupt outcomes
- small role-bounded child types instead of prompt-only specialization, starting with explorer,
  planner, and verifier-style child profiles

Do not take these into packet `026`:

- generic shell, file, web, cron, task-list, or worktree tool parity
- command-palette or app-shell UX features
- provider-compatibility or interoperability work that belongs to packet `027`
- secret-minimized remote worker bridges that belong to a later remote-executor packet

## Clarifications

### Session 2026-04-03

- Q: Is packet `026` implementation-ready now? → A: Yes. It is the next active packet for
  stronger real coordinator-subagent runtime truth on current `main`.
- Q: Who owns merge, reassign, clarify, stop, and collapse decisions in this packet? → A: The
  coordinator owns those decisions and the packet must keep them visible.
- Q: What delegated child state model does this packet assume? → A: `queued`, `delegated`,
  `running`, `blocked`, `clarified`, `reassigned`, `merged`, `completed`, `failed`, and
  `collapsed`.
- Q: What session continuity carries across coordinator-led follow-up runs? → A: Preserve
  `session_id`, `coordinator_agent_id`, delegated child identity, and evidence references, but do
  not imply transcript duplication or unlimited carry-forward.
- Q: What counts as placeholder-only execution for this packet? → A: If delegated work stops at a
  `workflow.execute_step`-style envelope without grounded evidence, it does not satisfy packet
  `026` success.

## User Scenarios And Testing

### User Story 1 - See Real Delegation And Child State (Priority: P1)

An operator inspects a runtime-backed run and can see that a coordinator delegated bounded work to
named subagents, along with each delegated child's current state and inbox activity.

**Why this priority**: This is the first honest difference between graph metadata and a real
coordinator-subagent runtime. Without it, the packet does not deliver its core value.

**Independent Test**: A bounded run that justifies delegation shows at least one coordinator-owned
delegation record, at least two visible child state transitions, and one honest sequential-collapse
path without requiring log archaeology.

**Acceptance Scenarios**:

1. **Given** a task that clearly benefits from bounded fan-out, **When** the runtime delegates
   work, **Then** the operator can see a coordinator-owned delegation record for each delegated
   child job.
2. **Given** delegated child work moves from waiting to active or blocked execution, **When** the
   operator inspects the run, **Then** the current child state and the latest subordinate inbox
   event are visible and attributable to that delegated job.
3. **Given** a task does not justify branching, **When** the runtime keeps the work sequential,
   **Then** the run still succeeds honestly and shows collapse or non-delegation instead of fake
   child activity.

---

### User Story 2 - Prove Grounded Delegated Work And Feedback Loops (Priority: P1)

An operator or reviewer can tell whether delegated work was actually grounded, how the coordinator
reacted when a child stalled or failed, and whether merge or recovery decisions were taken on real
evidence instead of placeholder completion.

**Why this priority**: Packet `026` must prove more than delegation theater. It must show
grounded delegated work and visible feedback loops.

**Independent Test**: A bounded delegated run records grounded evidence for at least one
subagent-owned job, records one visible coordinator decision for merge, clarify, reassign, stop,
or collapse, and keeps placeholder-only delegated work explicitly non-grounded.

**Acceptance Scenarios**:

1. **Given** a delegated child completes real bounded work, **When** the coordinator inspects the
   result, **Then** the run records grounded delegated work rather than only placeholder step
   completion.
2. **Given** a delegated child becomes blocked, fails, needs clarification, or is aborted because
   a sibling batch is cancelled, **When** the coordinator responds, **Then** the follow-up
   decision is visible as clarify, reassign, stop, merge, or collapse with an explicit reason.
3. **Given** delegated work only returns placeholder completion, **When** the run reaches a
   terminal state, **Then** the proof boundary explicitly says the run did not satisfy real
   coordinator-subagent success.

---

### User Story 3 - Inspect Proof Boundaries And Session-Aware Follow-Up (Priority: P2)

An operator can inspect task, autonomy, and run-detail views and understand both the packet proof
boundary and what session-aware follow-up will preserve if a later coordinator-led run continues
the work.

**Why this priority**: The packet must stay honest. Operators need to know what was proven and
what later work still depends on a fresh runtime rerun.

**Independent Test**: Task result, autonomy status, and run detail all show the same proof story,
the same stable delegated child identity, and the same session carry-forward assumptions.

**Acceptance Scenarios**:

1. **Given** a run used real coordinator-owned delegation, **When** an operator inspects task,
   autonomy, or run detail, **Then** all three views present the same proof-boundary story.
2. **Given** a run ended with sequential collapse or partial delegation only, **When** the
   operator inspects the proof view, **Then** the packet clearly states what was and was not
   proven.
3. **Given** a later coordinator-led follow-up run resumes related work, **When** session context
   is reused, **Then** the packet preserves stable identifiers and evidence references without
   implying unlimited transcript carry-forward.

## Edge Cases

- a task starts as a delegation candidate but collapses back to sequential execution because the
  smallest-workflow rule says fan-out is unnecessary
- a child becomes blocked after a delegation record exists but before grounded evidence exists
- a merge decision combines one grounded branch and one failed or placeholder-only branch
- the coordinator receives multiple child updates in one turn and must project them without losing
  child identity or event ordering
- sibling cancellation fires during a parallel child batch and the runtime must project a visible
  abort reason for every affected child
- a user interrupt lands during active delegated work and the runtime must preserve child identity
  and partial proof state without inventing success
- a later coordinator-led follow-up run reuses stable identifiers but should not imply transcript
  replay beyond the preserved references
- operator surfaces show graph completion for a run that still fails the packet `026` proof
  standard

## Requirements

### Functional Requirements

- **FR-001**: System MUST define packet `026` as the frozen implementation packet for the first
  real coordinator-subagent runtime on current `main`.
- **FR-002**: System MUST keep current graph, topology, routing, and session continuity truth
  separate from real coordinator-subagent success.
- **FR-003**: System MUST require coordinator-owned delegation records as first-class runtime
  evidence for any run that claims coordinator-subagent behavior.
- **FR-004**: System MUST expose a coordinator-owned subordinate inbox for child completion,
  blocked, clarify, cancel, sibling-abort, and user-interrupt signals that re-enter the parent run
  visibly.
- **FR-005**: System MUST require visible child state transitions for delegated work.
- **FR-006**: System MUST keep delegated child identity stable enough that the coordinator can
  clarify, resume, stop, or inspect the same delegated work unit across follow-up actions.
- **FR-007**: System MUST require grounded delegated work evidence before a run can satisfy the
  packet `026` proof standard.
- **FR-008**: System MUST treat placeholder-only delegated completion as non-grounded and
  insufficient for packet success.
- **FR-009**: System MUST preserve the smallest-workflow rule and MUST allow honest sequential
  collapse when fan-out is not justified.
- **FR-010**: System MUST make coordinator merge, clarify, reassign, stop, and collapse decisions
  visible to the operator when they occur.
- **FR-011**: System MUST expose packet `026` proof-boundary language on task result, autonomy
  status, and operator-console run detail.
- **FR-012**: System MUST consume packet `022` through `025` ownership by reference and MUST NOT
  redefine lifecycle, run-trace, security-boundary, or step-policy ownership inside packet `026`.
- **FR-013**: System MUST keep federation, capability discovery, and generic interoperability work
  out of this packet.
- **FR-014**: System MUST define the session-aware follow-up contract in terms of stable
  identifiers and evidence references, not unlimited transcript reuse.
- **FR-015**: System MUST keep child scratch context private by default and MUST restrict shared
  channels to root-owned registration, cancellation, runtime-truth projection, and capability
  enforcement.
- **FR-016**: System MUST define deterministic ordered projection for parallel child work,
  including explicit sibling-cancel and user-interrupt outcomes.
- **FR-017**: System MUST define role-bounded child execution for explorer, planner, and
  verifier-style child work instead of leaving specialization as prompt-only behavior.
- **FR-018**: System MUST keep the first implementation slice bounded to current
  `mister-smith-core`, `mister-smith-agents`, `mister-smith-app`, `mister-smith-events`, and
  operator-console seams rather than widening into a runtime redesign or a new endpoint.
- **FR-019**: System MUST tie major packet claims to the exact repo anchors named in this packet,
  its contract, and its task plan.
- **FR-020**: System MUST keep deterministic implementation readiness separate from any fresh live
  runtime-proof claim and MUST NOT overstate coordinator-subagent success without a bounded rerun.

### Key Entities

- **CoordinatorDelegationRecord**: a visible record that the coordinator assigned one bounded job
  to a specific child, including job intent, role, scope, and downstream evidence references
- **CoordinatorSubordinateInboxRecord**: the visible ordered intake record for one delegated child
  event, including completion, blocked, clarify, cancel, sibling-abort, or user-interrupt signals
- **SubagentStateRecord**: the current and previous visible state for one delegated child job,
  including blocked, clarified, reassigned, merged, failed, or collapsed outcomes
- **DelegatedWorkEvidenceRef**: a reference that ties one delegated job to the grounded evidence,
  proof boundary, or placeholder-only result that job produced
- **CoordinatorMergeDecision**: a visible coordinator-owned decision that explains how delegated
  outputs were merged, clarified, reassigned, stopped, or collapsed
- **CoordinatorRuntimeProofView**: the operator-facing proof summary that joins delegation,
  subordinate inbox activity, child state, delegated work evidence, coordinator decisions, and the
  current proof boundary

## Success Criteria

### Measurable Outcomes

- **SC-001**: Targeted deterministic validation can show at least one delegation record and at
  least two visible child state transitions on current task, autonomy, or run-detail surfaces.
- **SC-002**: Targeted deterministic validation can show one honest sequential-collapse path that
  does not fabricate delegated child activity.
- **SC-003**: Targeted deterministic validation can show one grounded delegated-work path and one
  placeholder-only delegated-work path with explicit non-grounded proof wording.
- **SC-004**: Task result, autonomy status, and operator run detail show the same proof story and
  the same stable child identity plus session-follow-up references.
- **SC-005**: Packet `022` through `025` ownership boundaries remain unchanged across the spec,
  plan, contract, tasks, and analysis artifacts.

## Assumptions

- packets `022` through `025` are already landed on current `main` and remain the adjacent
  authorities packet `026` must consume
- the supported runtime path still proves graph and workflow mechanics more strongly than grounded
  delegated work below the current placeholder step boundary
- task result, autonomy status, and operator-console run detail remain the canonical read surfaces
  for the first slice
- the first implementation slice can extend current workflow metadata, autonomy projection, and
  event-bus surfaces without introducing a new standalone packet-owned service
- live runtime proof for real coordinator-subagent success still requires a later bounded rerun
  after implementation lands
