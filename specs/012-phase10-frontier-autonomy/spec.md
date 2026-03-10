# Feature Specification: Phase 10 — Frontier Autonomy & Advanced Agent Patterns

**Feature Branch**: `012-phase10-frontier-autonomy`
**Created**: 2026-03-10
**Status**: Draft
**Input**: Linear issue `MS-26`, `ROADMAP.md`, `docs/2026-03-05-architectural-grounding-audit.md`,
`docs/2026-03-05-implementation-deviation-report.md`,
`docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`, the consolidated research corpus in
`docs/research-output/`, and canonical framework sources in `spec/`.

## Scope & Traceability

### Phase Position

`ROADMAP.md` currently defines phases through Phase 9 only, while the repository already contains
completed implementation artifacts for Phase 9 and Phase 9.1. This specification defines **Phase 10
as the next roadmap extension after Phase 9.1**, using the strongest repo-local evidence for what
should come next:

- `docs/2026-03-05-implementation-deviation-report.md` proposes **Phase 10: Advanced Agent
  Patterns**
- `specs/009-phase9-llm-provider-integration/` and
  `specs/011-phase9.1-security-hardening/` both defer multiple capabilities to **Phase 10+**
- `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md` states that zero-trust is
  substrate work in service of frontier autonomy, not the entire product roadmap

### Governing Sources

This Phase 10 specification is constrained by the following precedence order:

1. `ROADMAP.md` through completed Phase 9 and the repo's active Phase 9.1 follow-on work
2. `docs/plans/2026-03-09-frontier-autonomy-zero-trust-design.md`
3. `docs/2026-03-05-implementation-deviation-report.md`
4. `docs/2026-03-05-architectural-grounding-audit.md`
5. Consolidated research findings in `docs/research-output/consolidated/`
6. Canonical architecture sources in `spec/`
7. Supporting repo context in `README.md`, `CLAUDE.md`, and `.specify/memory/constitution.md`

### Canonical Architecture Citations

Phase 10 artifacts MUST trace back to these framework sources:

- `spec/core-architecture/system-architecture.md` for async-first, fault-tolerant system design
- `spec/core-architecture/async-patterns.md` for stream processing, ToolBus, and bounded async
  execution patterns
- `spec/core-architecture/supervision-trees.md` for restart policies, failure isolation, and
  supervisory boundaries
- `spec/data-management/agent-orchestration.md` for planner/executor/critic/router/memory roles,
  context-management patterns, and parallel coordination hooks
- `spec/data-management/message-schemas.md` for workflow coordination, hook-event schemas, and
  future consensus message boundaries
- `spec/operations/observability-monitoring-framework.md` for operator-visible tracing, metrics,
  and intervention telemetry
- `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` for domain-adaptive topology and
  future neural-operations boundaries

### Research Grounding

This Phase 10 specification directly incorporates the highest-value findings already identified in
the repo's research corpus:

| Finding | Source | Phase 10 Impact |
| ------- | ------ | --------------- |
| **#1 Dynamic Topology** | `00-MASTER-FINDINGS.md`, `02-orchestration-and-self-organization.md` | Adds topology-aware planning, graph compilation, and dynamic team shape selection. |
| **#2 Step-Level Intelligence** | `00-MASTER-FINDINGS.md`, `06-streaming-architecture.md` | Adds step-boundary awareness and streaming monitors without guided or speculative decoding. |
| **#4 Predictive Supervision** | `00-MASTER-FINDINGS.md`, `03-supervision-and-resilience.md` | Adds Guard/Advisor interventions, failure classification, and agent performance profiling. |
| **Managed Memory as an OS Resource** | `07-memory-and-context.md` | Adds a managed-memory layer over JetStream KV + PostgreSQL with paging, consolidation, and role-aware context routing. |
| **Zero-Trust Substrate** | `frontier-autonomy-zero-trust-design.md`, `04-security-and-trust.md` | Keeps delegation, capability scoping, and authority lineage in scope only as autonomy substrate. |

### In Scope

- Topology-aware orchestration that compiles planner output into explicit execution graphs with
  dependency tracking
- Dynamic execution modes for parallel, sequential, pipeline, hierarchical, and hybrid workflows
- Resilience-aware routing that considers task shape, dependency depth, health state, and bounded
  execution budgets
- Managed memory/context layer above existing persistence primitives, including role-aware context
  routing, paging, consolidation, summaries, and checkpoint-ready context snapshots
- Guard/Advisor supervision layer that classifies failures, observes step boundaries, and chooses
  intervention strategies before resorting to full restarts
- Operator-facing observability and intervention surfaces for topology state, memory pressure,
  health, checkpoints, routing decisions, and supervisory actions
- Delegation/provenance substrate for privileged autonomous actions, with bounded authority chains,
  expiry, revocation, and operator-visible attribution
- Integration boundaries that preserve existing provider-neutral and transport-neutral architecture

### Explicitly Deferred

The following items are intentionally **not** Phase 10 acceptance scope:

- Learned routing via RouteLLM, kNN/ONNX semantic routing, or other model-trained router policies
- Guided decoding via XGrammar or Outlines
- Full speculative decoding / rejection-sampling pipelines beyond step-boundary monitoring inputs
- Local model inference, disaggregated serving, shared KV-cache serving, or PrefillShare-style
  infrastructure
- Full CRDT coordination, MPST session types, or general-purpose distributed consensus suites
- MAS^2-style recursive architecture generation and market-based agent auctions
- eBPF-based observability, ML-based anomaly detection, and distributed backdoor correlation
  monitoring
- Cross-organization federation, Biscuit/ZCAP identity federation, or blockchain-backed
  attestations

### Prerequisites & Dependencies

- Phase 9's provider-neutral LLM contracts, dual-stream event layer, and budget primitives must be
  stable before Phase 10 layers topology and supervision intelligence on top
- Phase 9.1's message signing, Auth Callout, state validation, and sandbox boundaries remain the
  security substrate for Phase 10 rather than being redefined here
- Phase 6 persistence remains the backing store for managed memory; Phase 10 adds the missing
  management layer, not a replacement storage system
- Phase 8 operations remains the system entry point and observability substrate; Phase 10 extends
  it with topology- and autonomy-specific visibility

## Clarifications

### Session 2026-03-10

- Q: Does Phase 10 continue Phase 9.1 security hardening as the phase identity? → A: No. Phase 10
  is the frontier-autonomy and advanced-agent-patterns extension; zero-trust work stays in scope
  only where it enables supervised autonomy.
- Q: Should Phase 10 absorb all "Phase 10+" items deferred from Phase 9 and Phase 9.1? → A: No.
  Only the highest-leverage items that advance topology-aware orchestration, managed memory,
  predictive supervision, operator visibility, and bounded delegation belong here.
- Q: Does Phase 10 require learned routing, local inference, or full disaggregated serving? → A:
  No. Phase 10 defines the structural control plane and memory/supervision primitives first; those
  more speculative serving and model-selection capabilities remain deferred.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Topology-Aware Workflow Execution (Priority: P1)

A framework operator submits a decomposed task or workflow plan and the system compiles it into an
explicit execution graph, selects the appropriate topology, and runs it without violating declared
dependencies or duplicating already-completed work after failure.

**Why this priority**: Static, fixed-topology orchestration is the main architectural limit called
out by the research corpus and the March 9 design note. Phase 10 must first improve how the system
shapes and executes work.

**Independent Test**: Submit representative workloads that are clearly parallel, sequential, and
mixed-dependency. Verify the system selects a compatible topology, preserves dependency order,
parallelizes eligible branches, and resumes from checkpoints after a node failure without
re-running completed branches.

**Acceptance Scenarios**:

1. **Given** a workflow with independent branches, **When** the system compiles the planner output,
   **Then** it chooses a topology that executes the independent branches concurrently while
   preserving shared dependencies.
2. **Given** a workflow with tightly coupled sequential steps, **When** the topology compiler
   evaluates it, **Then** it chooses a sequential or pipeline topology instead of an unnecessary
   parallel swarm.
3. **Given** a running execution graph with one failed branch, **When** a checkpoint exists,
   **Then** the system restarts or reallocates only the affected work and preserves all completed
   branches.
4. **Given** a malformed or cyclic execution graph, **When** the compiler validates it, **Then**
   the workflow is rejected before execution starts and the operator sees the reason.

---

### User Story 2 — Managed Memory and Role-Aware Context (Priority: P1)

A framework developer enables a managed memory layer so that each agent role receives bounded,
task-relevant context assembled from tiered memory rather than an ever-growing shared transcript.
Older context is paged, consolidated, summarized, and checkpointed without blocking active work.

**Why this priority**: The repo already has persistence primitives; the missing layer is the memory
management abstraction that keeps long-running autonomous workflows tractable.

**Independent Test**: Run a multi-step workflow that exceeds the per-agent working-context budget.
Verify the system summarizes or pages older context, routes only relevant context to each role,
persists consolidated memory with metadata, and restores a valid context snapshot on resume.

**Acceptance Scenarios**:

1. **Given** a long-running workflow, **When** an agent exceeds its working-context budget,
   **Then** older context is summarized or paged without losing the active task state.
2. **Given** different agent roles participating in the same workflow, **When** the memory manager
   assembles context, **Then** each role receives only the subset relevant to its function and
   budget.
3. **Given** historical fragments promoted out of the active working set, **When** consolidation
   runs, **Then** the resulting memory record preserves provenance, freshness, and access policy.
4. **Given** a resumed workflow, **When** the agent reconstructs its state, **Then** it receives a
   checkpoint-compatible context snapshot rather than replaying the entire raw history.

---

### User Story 3 — Predictive Supervision and Operator Visibility (Priority: P1)

A framework operator can see how the autonomy system is behaving in real time: which topology is
active, which branches are healthy or degraded, why an intervention occurred, and what step-level
signals or checkpoints informed the supervisory decision.

**Why this priority**: High-autonomy execution without actionable visibility is explicitly rejected
by the March 9 design note. The operator must stay coupled to the system's execution pulse.

**Independent Test**: Induce transient, structural, streaming, and semantic-style failure
conditions. Verify the Guard/Advisor layer classifies them correctly, emits intervention records,
and the operator can inspect topology, checkpoint, health, and intervention state without reading
raw logs.

**Acceptance Scenarios**:

1. **Given** a transient provider or transport failure, **When** the Guard layer evaluates the
   event, **Then** it chooses a retry or failover action instead of a full workflow restart.
2. **Given** repeated low-quality or stalled step output, **When** step-level supervision signals
   degrade past policy thresholds, **Then** the Guard layer triggers a targeted intervention such as
   context refresh, branch isolation, or escalation.
3. **Given** a running autonomous workflow, **When** the operator inspects it, **Then** they can
   view the active topology, branch state, checkpoint lineage, budgets, and intervention history.
4. **Given** two branches competing for constrained capacity, **When** routing makes a decision,
   **Then** the operator can inspect the health, budget, and dependency rationale behind it.

---

### User Story 4 — Bounded Delegation and Provenance (Priority: P2)

A framework operator permits autonomous multi-agent execution only when every privileged action
preserves a bounded authority chain. Delegated authority can be inspected, expires on time, and is
revocable without collapsing the entire workflow.

**Why this priority**: The March 9 design note makes revoked, observable capability a core design
principle. Phase 10 must complete the autonomy substrate enough to support real delegated
execution.

**Independent Test**: Execute a workflow that requires delegation across multiple agents and tool
calls. Verify each authority transfer records provenance, invalid or expired delegation is blocked,
and the operator can trace any privileged action back to its originating policy context.

**Acceptance Scenarios**:

1. **Given** an agent delegates a privileged action to another agent, **When** the action is
   executed, **Then** the delegation chain records issuer, recipient, scope, expiry, and parent
   link.
2. **Given** a revoked or expired delegation, **When** a downstream agent attempts execution,
   **Then** the system blocks the action and records an operator-visible reason.
3. **Given** a completed privileged workflow, **When** an operator audits it, **Then** they can
   reconstruct the authority chain for each privileged step without relying on ambient trust.

### Edge Cases

- The topology compiler receives a graph with an unresolved dependency, cycle, or unsupported node
  type.
- Health, budget, and task-shape signals disagree on the best topology or branch allocation.
- An intervention fires while a branch is already restoring from checkpoint, risking duplicate
  recovery work.
- The memory manager cannot assemble enough safe context within the current role budget.
- Delegation expires in the middle of a long-running workflow branch.
- Control-plane state is stale or partially unavailable while the data plane continues running.
- A semantic degradation signal is weak or noisy rather than a clean "hard failure."
- Two operators or supervisory actors attempt conflicting intervention actions at the same time.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-201**: Phase 10 MUST compile planner output into an explicit execution graph with nodes,
  dependencies, branch state, and topology-selection inputs before execution begins.
- **FR-202**: The system MUST support topology selection across sequential, parallel, pipeline,
  hierarchical, and hybrid execution modes based on dependency structure and policy.
- **FR-203**: The topology compiler MUST reject invalid, cyclic, or unsupported execution graphs
  before runtime work is dispatched.
- **FR-204**: The execution system MUST checkpoint graph progress and support branch-local resume or
  reassignment without re-running already-completed branches.
- **FR-205**: Phase 10 MUST maintain bounded context budgets per agent role and route only
  task-relevant context to each role.
- **FR-206**: The memory layer MUST treat stored context as managed memory fragments carrying
  provenance, freshness, access policy, and version metadata.
- **FR-207**: The memory layer MUST support asynchronous paging, summarization, consolidation, and
  checkpoint-ready snapshot generation without blocking active execution.
- **FR-208**: Phase 10 MUST introduce a Guard/Advisor supervision layer that classifies failures
  into at least transient, structural, streaming, and semantic categories and maps them to distinct
  interventions.
- **FR-209**: The Guard/Advisor layer MUST accept step-level or stream-level degradation signals as
  intervention inputs without requiring full speculative decoding to be implemented.
- **FR-210**: The system MUST maintain profile or telemetry snapshots that support resilience-aware
  routing and supervisory decisions.
- **FR-211**: Phase 10 MUST expose operator-visible state for topology choice, branch health,
  checkpoint lineage, context pressure, routing rationale, and intervention history.
- **FR-212**: Operators MUST be able to inspect why a topology, routing, or intervention decision
  was taken without reading raw transport or application logs.
- **FR-213**: Every privileged autonomous action MUST preserve delegation provenance including
  issuer, recipient, scope, expiry, and parent authority link.
- **FR-214**: Delegation provenance MUST be enforceable at execution time, including expiry,
  revocation, and invalid-chain rejection.
- **FR-215**: Phase 10 MUST integrate with existing Phase 9 and Phase 9.1 security and routing
  substrate without redefining provider-specific or transport-specific public contracts.
- **FR-216**: When optional inputs such as profile data, memory metadata, or fresh control-plane
  state are unavailable, the system MUST degrade to a conservative, operator-visible execution
  posture rather than silently widening autonomy.

### Key Entities *(include if feature involves data)*

- **ExecutionGraph**: The explicit workflow graph containing nodes, dependencies, branch state,
  policy inputs, and checkpoint lineage.
- **TopologyPlan**: The selected execution shape for a workflow, including parallelism width,
  sequencing, coordination strategy, and rationale.
- **ExecutionBranch**: A checkpointable unit of work within the graph that can be resumed,
  reallocated, or isolated independently.
- **MemoryFragment**: A managed context unit carrying content plus provenance, freshness, access
  policy, and version metadata.
- **ContextBudget**: A bounded context allowance tied to a role, task stage, or branch.
- **GuardDecision**: A supervisory decision that records failure class, intervention type, evidence,
  and escalation outcome.
- **ProfileSnapshot**: Telemetry and performance state used to guide routing and supervisory
  decisions.
- **InterventionRecord**: An operator-visible record of a supervisory action, why it happened, and
  what changed afterward.
- **DelegationCapability**: A bounded unit of delegated authority with scope, expiry, and
  revocation semantics.
- **ProvenanceChain**: The linked record of authority transfers attached to privileged execution.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-201**: For representative mixed-dependency validation workflows, the system selects a
  topology that preserves declared dependencies and executes all independent branches concurrently
  where allowed.
- **SC-202**: Role-aware context management reduces per-agent delivered context volume by at least
  30% versus broadcasting the full workflow history on the same validation scenarios.
- **SC-203**: In checkpoint-enabled validation runs, at least 95% of single-branch failures are
  recovered through branch-local resume or reassignment without re-running completed branches.
- **SC-204**: Operators can inspect topology state, branch health, checkpoint lineage, and
  intervention rationale for 100% of validation scenarios without consulting raw logs.
- **SC-205**: Every privileged action in validation workflows carries a reconstructable delegation
  provenance chain, and invalid or expired delegation attempts are rejected in all such scenarios.
