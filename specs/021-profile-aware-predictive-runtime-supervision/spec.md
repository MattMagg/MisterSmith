# Feature Specification: Profile-Aware Predictive Runtime Supervision

**Feature Branch**: `021-profile-aware-predictive-runtime-supervision`
**Created**: 2026-03-27
**Status**: Draft
**Input**: `docs/current-state.md`,
`docs/plans/2026-03-26-verifier-gated-adaptive-orchestration.md`,
`docs/plans/2026-03-27-runtime-planning-simplification.md`,
`docs/plans/2026-03-27-ms-110-ambiguous-prompt-evidence-freeze.md`,
`docs/research-output/ROUTING_MANIFEST.md`,
`docs/research-output/consolidated/00-MASTER-FINDINGS.md`,
`docs/research-output/consolidated/03-supervision-and-resilience.md`,
`docs/research-output/consolidated/06-streaming-architecture.md`,
`docs/research-output/consolidated/08-competitive-landscape-and-ecosystem.md`,
`docs/research-output/research/targeted-predictive-supervision-R6.md`, and the current runtime
surfaces in `crates/mister-smith-app/src/execution.rs`,
`crates/mister-smith-app/src/autonomy.rs`, `crates/mister-smith-agents/src/orchestrator.rs`,
`crates/mister-smith-agents/src/profile.rs`, `crates/mister-smith-agents/src/guard.rs`,
`crates/mister-smith-agents/src/intervention.rs`, and `apps/operator-console/src/views/RunsView.tsx`

## Current Truth & Scope

Current repo truth already includes:

- landed Phase 10 substrate for execution graphs, topology selection, stream monitoring,
  Guard/Advisor types, and intervention application
- packet `019` runtime-routing and budget-control-loop work when an explicit
  `llm.runtime_routing_profile` is configured
- packet `020` verifier-gated step decisions, handoff clarification, last-stable-checkpoint
  repair lineage, and operator-visible orchestration-quality provenance
- the March 27 runtime-planning simplification pass, which restored the smallest-workflow
  baseline and moved supported description-only repair telemetry onto an explicit runtime-owned
  record

The remaining gap is more specific than another broad research packet:

- the supported runtime ingress still does not exercise profile-aware predictive supervision as a
  first-class runtime contract
- the supervised planner path still seeds `LlmSupervision` with
  `GuardTarget::Provider(...)`, which is too coarse once graph, branch, and step context exist
- profile snapshots, guard decisions, and interventions are fully modeled in the agents layer, but
  the supported task path still defaults many result surfaces to empty supervisory state
- the operator console run detail still shows preview and raw payload, not supervisory evidence as
  a first-class operator affordance
- `MS-110` adaptive-topology planning remains intentionally dormant because the March 27 evidence
  freeze did not show new over-shaping regressions, so topology refinement is not the next honest
  packet

This packet therefore freezes one bounded next phase:

1. bring branch- and node-scoped predictive supervision onto the supported runtime-backed task
   path instead of relying on provider-only supervision targets
2. add bounded profile fingerprints that inform interventions using replayable runtime evidence,
   without requiring a new training stack
3. make profile, guard, and intervention lineage visible on task, autonomy, and operator run
   surfaces as first-class runtime evidence

This is not:

- default-runtime activation of packet `019` for no-profile routing
- new topology-selection work, `MS-110` reopen, or a dynamic-team-sizing rewrite
- CKM training, PPO policies, MAS^2 generation, or any recursive self-orchestration program
- CRDT coordination, MPST session-typing, or distributed consensus work
- a broad operator-console redesign or a new benchmark claim

## Clarifications

### Session 2026-03-27

- Q: What runtime targets are in scope for predictive supervision in this packet? → A: Planner,
  coordinator, executor, branch, and node scope when graph context exists; provider scope only
  before graph context is available.
- Q: Where do profile fingerprints live, and what do they store? → A: JetStream KV through the
  existing `mister-smith-persistence/src/kv/` seam, storing structured summaries and source
  references only, not duplicated raw transcripts.
- Q: Which operator surfaces must expose supervisory evidence? → A: Task result, autonomy status,
  and operator-console run detail only.
- Q: How does packet `021` interact with packet `020` repair lineage? → A: Packet `020` remains
  canonical for verifier-driven repair; packet `021` adds predictive-supervision evidence that
  must reconcile with, not replace, packet `020` lineage.

## User Scenarios & Testing

### User Story 1 - Run predictive supervision on the supported ingress (Priority: P1)

An operator submits a normal runtime task, and the supported runtime path records profile
snapshots and applies targeted Guard interventions before falling back to graph-wide restart or
provider-only escalation.

**Independent Test**: targeted `mister-smith-app` and `mister-smith-agents` coverage simulates
recoverable and unrecoverable degradation on the supported task path, proving that supervisory
events are emitted with branch- or node-scoped targets and that the current happy path stays
intact when no supervisory evidence exists.

**Acceptance Scenarios**:

1. **Given** a running workflow branch emits repetitive, stalled, or missing-context signals,
   **When** the runtime evaluates supervision, **Then** it records a `ProfileSnapshot` tied to a
   branch or node target rather than only to the provider.
2. **Given** the degradation is recoverable, **When** the Guard evaluates the evidence,
   **Then** the runtime applies `retry`, `context_refresh`, or `branch_isolation` locally before
   considering graph-wide restart.
3. **Given** no predictive signal or usable fingerprint exists, **When** the task completes,
   **Then** the current shipped happy path is preserved without synthetic guard activity.

### User Story 2 - Use bounded profile fingerprints to choose better interventions (Priority: P1)

A developer or operator can seed bounded performance fingerprints from replayable runtime evidence
so the Guard can choose interventions based on known failure tendencies instead of generic
fallback alone.

**Independent Test**: a deterministic fingerprint fixture plus targeted runtime tests prove that a
matching fingerprint can alter intervention notes or choice, while stale or low-confidence
fingerprints degrade gracefully to live-signal-only supervision.

**Acceptance Scenarios**:

1. **Given** replayable runtime evidence or deterministic fixtures for a supported role,
   **When** a fingerprint is generated, **Then** it records dominant failure tendencies,
   preferred interventions, confidence, and expiry without requiring a learned policy runtime.
2. **Given** a live degradation matches a current fingerprint, **When** the Guard evaluates the
   event, **Then** the decision evidence cites the fingerprint as advisory context instead of only
   using conservative fallback notes.
3. **Given** the fingerprint is missing, expired, or contradicted by live evidence,
   **When** supervision runs, **Then** the runtime falls back to live signals without blocking the
   workflow.

### User Story 3 - Inspect supervisory evidence without log archaeology (Priority: P2)

An operator inspects task, autonomy, or operator-console run details and can see the latest
profile state, guard decision, and intervention lineage that shaped the runtime outcome.

**Independent Test**: task and autonomy projections stay consistent with operator-console run
detail output, and packet proof notes keep deterministic validation explicitly separate from any
later live runtime proof.

**Acceptance Scenarios**:

1. **Given** a workflow exercised predictive supervision, **When** an operator inspects the task
   or autonomy view, **Then** they can see the latest fingerprint reference, profile health,
   guard decision, and intervention rationale.
2. **Given** the operator opens the run detail in `apps/operator-console/`, **When** the packet
   UI work lands, **Then** the supervisory evidence appears as a first-class summary instead of
   only inside raw payload inspection.
3. **Given** the packet closes with deterministic validation before a live rerun exists,
   **When** the docs are updated, **Then** they state that proof boundary explicitly rather than
   implying a broader runtime claim.

## Edge Cases

- supervision starts before a graph or branch target exists, requiring a provider-scoped fallback
- a profile fingerprint suggests one intervention while live step signals suggest a more severe one
- packet `020` verifier repair lineage and packet `021` guard lineage both apply to the same step
- a join or coordinator step receives degradation signals after upstream branch state was already
  normalized
- a stale fingerprint survives longer than the runtime evidence that originally justified it
- the operator surface changes land before a new live proof is captured

## Requirements

### Functional Requirements

- **FR-001**: System MUST evaluate predictive supervision on the supported runtime-backed task path
  using branch or node scope when graph context exists, with provider scope only as the pre-graph
  fallback.
- **FR-002**: System MUST record `ProfileSnapshot`, `GuardDecision`, and `InterventionRecord`
  evidence on the runtime path in a form that task and autonomy projections can consume directly.
- **FR-003**: System MUST preserve the current shipped happy path when no predictive evidence or
  usable fingerprint exists.
- **FR-004**: System MUST add a bounded `ProfileFingerprint` surface backed by existing
  JetStream KV infrastructure through `mister-smith-persistence/src/kv/` with explicit
  confidence, provenance, and expiry.
- **FR-005**: System MUST prefer existing typed local interventions before graph-wide restart when
  predictive evidence indicates the failure is recoverable.
- **FR-006**: System MUST reconcile packet `020` verifier/repair lineage with packet `021`
  predictive-supervision lineage so operator views do not present contradictory outcomes.
- **FR-007**: System MUST expose supervisory evidence on task result, autonomy status, and the
  operator-console run detail without requiring raw payload inspection.
- **FR-008**: System MUST keep the write set bounded to `mister-smith-core`,
  `mister-smith-agents`, `mister-smith-app`, `mister-smith-events`,
  `mister-smith-persistence` or KV glue if needed, `apps/operator-console/`, and state-bearing
  packet/router docs.
- **FR-009**: System MUST NOT require CKM training, PPO policies, CRDT coordination,
  topology-selection changes, or default-runtime routing-profile activation in this packet.
- **FR-010**: System MUST keep deterministic validation and any later live-proof claims
  explicitly separated.
- **FR-011**: System MUST fail explicitly when supervisory state cannot be reconciled with the
  active target scope or when a fingerprint payload is structurally invalid.
- **FR-012**: System MUST use replayable runtime evidence or deterministic fixtures to seed
  fingerprints so the packet can be validated without requiring a new frontier benchmark program.
- **FR-013**: System MUST store structured fingerprint summaries and source references only; it
  MUST NOT duplicate raw transcripts outside the existing audit and replay surfaces.

### Key Entities

- **ProfileFingerprint**: persisted supervisory summary keyed to a supported runtime role or
  target class, including dominant failure tendencies, preferred interventions, confidence,
  provenance, expiry, and source references without duplicating raw transcripts
- **ProfileSnapshot**: existing runtime-facing health snapshot derived from live stream or step
  signals, optionally referencing a contributing fingerprint
- **GuardDecision**: existing typed decision that classifies failure type and chosen
  intervention based on live evidence plus optional fingerprint reinforcement
- **InterventionRecord**: existing record of the applied recovery action and before/after state
- **SupervisionEvidenceView**: operator-facing projection joining the latest fingerprint,
  profile snapshot, guard decision, intervention, and proof boundary notes

## Success Criteria

- **SC-001**: the supported runtime ingress can emit non-empty profile, guard, and intervention
  evidence for at least one bounded degradation case without regressing the current happy path
- **SC-002**: a matching profile fingerprint changes or reinforces at least one intervention
  decision in deterministic validation
- **SC-003**: packet `020` repair lineage and packet `021` predictive-supervision lineage remain
  coherent in task and autonomy views
- **SC-004**: operator surfaces can expose supervisory evidence without requiring raw log
  archaeology
- **SC-005**: packet docs remain explicit that topology search, CKMs, CRDT coordination, and
  broader benchmark claims are still future work rather than silently implied by this packet
