# Phase 7: Agent System

## Purpose and Scope

Define the multi-agent orchestration layer that consumes supervision, transport, security, and
persistence foundations to deliver coordinated task execution.

### In Scope

- Agent lifecycle and operational state transitions
- Inter-agent communication and coordination semantics
- Team orchestration and task decomposition
- Tool system and agent-as-tool composition
- Specialized agent role definitions

### Out of Scope

- Runtime primitives and supervision internals
- New transport protocol definition
- Deployment and production operations policy

## Inputs and Dependencies

### Upstream Dependencies

- Phase 3 (actor and supervision guarantees)
- Phase 4 (transport and schema contracts)
- Phase 5 (authz and permission enforcement)
- Phase 6 (state and persistence outputs)

### Key Source Inputs

- `ROADMAP.md` Phase 7 and Gate 7
- `VALIDATION_REPORT.md` terminology-consistency findings and readiness context

### Required Specification Anchors

- `spec/data-management/agent-lifecycle.md`
- `spec/data-management/agent-operations.md`
- `spec/data-management/agent-communication.md`
- `spec/data-management/agent-integration.md`
- `spec/data-management/agent-orchestration.md`
- `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md`
- `spec/core-architecture/async-patterns.md`
- `spec/core-architecture/integration-patterns.md`

## Outputs and Downstream Consumers

### Produces

- Agent lifecycle and operational behavior model
- Team orchestration contract for decomposition, delegation, and aggregation
- Tooling interface model for hierarchical agent composition
- Specialized role responsibilities tied to orchestration behavior

### Consumed By

- Phase 8 process startup/shutdown and operational observability
- Future implementation planning artifacts and integration tests

## Gate Criteria and Validation

### Gate Criteria

- Lifecycle states/restart behavior align with supervision contracts
- Communication patterns use standardized schemas and correlation handling
- Orchestration patterns define coordinator/worker/supervisor responsibilities clearly
- Tool permissions align with security policy model
- Specialized roles do not conflict with canonical agent-type semantics

### Validation Approach

- Cross-check lifecycle/orchestration/communication docs for enum/state consistency
- Verify references to persistence and transport boundaries are explicit
- Confirm naming consistency across agent-domain and orchestration docs

### Validation Evidence

- End-to-end scenario trace from task decomposition to result aggregation
- Explicit supervision-restart behavior references for failed worker paths

## Official-Doc Best Practices

- Use subject naming and wildcard strategy consistent with NATS routing guidance to keep orchestration predictable ([NATS subjects](https://docs.nats.io/nats-concepts/subjects)).
- Keep async orchestration and cancellation behavior explicit using Tokio task/channel primitives ([Tokio task](https://docs.rs/tokio/1.49.0/tokio/task/) and [Tokio sync](https://docs.rs/tokio/1.49.0/tokio/sync/)).
- Keep inter-agent payload schemas versioned and backward-compatible with serde contracts ([Serde data model](https://serde.rs/data-model.html)).

## Known Risks / Unknowns

### Risks

- Multi-agent coordination semantics can drift across documents
- Role taxonomy and trust/category taxonomies can be conflated
- Tool permission boundaries can be underspecified at execution edges

### Required Follow-ups

- Preserve canonical role/lifecycle definitions in data-management specs
- Revalidate orchestration docs when supervision or security semantics change

## Phase 7 Prerequisites

Phase 7 builds directly on transport, persistence, and security contracts from Phases 4-6. An
external evaluation ([docs/session_analysis_report_prephase7.md](../../docs/session_analysis_report_prephase7.md))
identified six contract-level decisions that must be locked down before writing the Phase 7 specification.
Deferring these to Phase 7 would force architectural retrofits once agents, workflows, and scheduling
logic depend on them.

### Prerequisite 1: Transport Durability Semantics

**Problem**: The `Transport` trait exposes only fire-and-forget pub/sub (publish, subscribe,
queue_subscribe, request). `JetStreamManager` exists in `mister-smith-nats` but is completely
disconnected from the `Transport` abstraction. Phase 7 agents need delivery guarantees to reason
about task ownership, completion, and failure recovery.

**Decision**: Extend the transport layer with a `DurableTransport` trait that surfaces JetStream
consumer semantics alongside the existing `Transport` trait. The existing fire-and-forget trait
remains for ephemeral messaging (heartbeats, status updates).

**Contract**:
- `DurableTransport` exposes: `durable_subscribe(subject, consumer_name)` returning a stream of
  `DurableMessage` with `ack()`, `nak(delay)`, and `term()` methods
- Pull consumers with `AckPolicy::Explicit` as the default (per NATS official recommendation for
  new projects — [NATS Consumer Docs](https://docs.nats.io/using-nats/developer/develop_jetstream/consumers))
- Publisher-side deduplication via `MsgId` header with configurable dedup window
  ([NATS Dedup](https://nats.io/blog/new-per-subject-discard-policy))
- `JetStreamConfig` fields (`max_ack_inflight`, `ack_timeout`) wired into actual consumer creation
- Delivery guarantee: at-least-once with explicit ack. Exactly-once semantics achieved at the
  application layer via idempotent processing (see Prerequisite 2)

**Scope**: Code changes to `mister-smith-transport` (new trait) and `mister-smith-nats` (wire
JetStreamManager into the abstraction).

### Prerequisite 2: Message Idempotency and Deduplication

**Problem**: Phase 6 persistence supports state-level idempotency (`upsert_state` uses
`ON CONFLICT`), but message insertion allows duplicates. No inbox/outbox pattern exists despite
being specified in `spec/data-management/data-integration-patterns.md`. With NATS JetStream
at-least-once delivery, redeliveries will cause duplicate processing without dedup enforcement.

**Decision**: Add message-level deduplication at the persistence boundary using `message_id` as
the idempotency key.

**Contract**:
- Add UNIQUE constraint on `messages.records(message_id)` or equivalent partial index
- Message insert uses `INSERT ... ON CONFLICT (message_id) DO NOTHING` (reject duplicates)
- `correlation_id` used for workflow-level grouping (not dedup) via existing index
- Workflow step processing checks `message_id` before side effects
- Publisher sets `MsgId` header (maps to `message_id` in envelope) for JetStream server-side dedup

**Scope**: New migration in `mister-smith-persistence`, modification to message insert query.

### Prerequisite 3: Subject Taxonomy Stabilization

**Problem**: 14 subject patterns implemented vs 25+ documented in specs. README/ROADMAP advertise
`agent.<type>.<id>.<action>` which doesn't match the actual `agents.{agent_id}.commands.{command_type}`
format. Legacy Claude-specific CLI patterns exist in transport specs but should not be implemented.

**Decision**: Freeze the implemented taxonomy as the stable API contract. Prune legacy patterns.
Add Phase 7-required patterns.

**Contract** (canonical patterns, versioned as `taxonomy.v1`):
- **Agent**: `agents.{id}.commands.{type}`, `agents.{id}.status`, `agents.{id}.heartbeat`,
  `agents.{id}.events.{type}`, `agents.{id}.capabilities` (new)
- **Task**: `tasks.{type}.assignment`, `tasks.{type}.queue.{priority}`, `tasks.{id}.progress`,
  `tasks.{id}.result`
- **System**: `system.events.{type}`, `system.config.{component}`, `system.health`,
  `system.metrics.{component}` (new)
- **Workflow**: `workflow.{id}.start`, `workflow.{id}.step.{step_id}`, `workflow.{id}.result`
- **Wildcards**: `agents.>`, `tasks.*.assignment`, `system.>`, `workflow.>`
- CLI-specific patterns (`cli.startup`, `cli.hooks.*`) are **removed** from specs
- README/ROADMAP updated to match actual taxonomy

**Scope**: Spec updates (prune CLI patterns, fix README/ROADMAP), optional code additions for
`.capabilities` and `.metrics` subjects.

### Prerequisite 4: Spec Type Reconciliation

**Problem**: Critical type inconsistencies across specification files that would cause confusion
during Phase 7 implementation.

| Type | Conflict | Resolution |
|------|----------|------------|
| `AgentId` | `String` in agent-orchestration.md, `type Uuid` in agent-lifecycle.md, `struct AgentId(Uuid)` in code | Canonical: `struct AgentId(pub Uuid)` per `type-definitions.md` and `mister-smith-core` |
| `Agent` trait | `&mut self` in agent-lifecycle.md, `&self` in agent-orchestration.md | Canonical: `&self` (interior mutability via Arc/Mutex per Phase 3 actor pattern) |
| `AgentStatus` vs `AgentState` | Different enums in storage vs test vs wire layers | `AgentState` is canonical (7 variants in core). `AgentStatus` is a composite struct containing state + health + uptime |
| Method names | `tool_id()` in code (inherited from Tool), `agent_id()` in specs | Code is canonical; Phase 7 Agent trait will provide `agent_id()` that delegates to inner identity |

**Scope**: Update 4 spec files to match code. No code changes needed.

### Prerequisite 5: Tool Permission Model Definition

**Problem**: RBAC infrastructure exists (Phase 5 PolicyEngine with `action:resource:scope` format),
but no tool-specific permission patterns are defined. MCP crate has zero security integration.
Phase 7.4 (Tool System) will implement integration, but the model must be defined first.

**Decision**: Define tool permissions as an extension of the existing RBAC permission model,
incorporating capability-based scoping per current best practices for AI agent frameworks
([Microsoft Multi-Agent Reference Architecture](https://microsoft.github.io/multi-agent-reference-architecture/docs/security/Security.html),
[MCP Permissions / Permit.io](https://docs.permit.io/mcp-permissions)).

**Contract**:
- Permission patterns: `execute:tool:{namespace}`, `discover:tool:{namespace}`, `register:tool:{namespace}`
- Scope qualifiers: `own` (agent's own tools), `team` (team-scoped), `all` (global)
- Tool invocation requires `AuthorizationRequest` evaluation before execution
- Tool discovery filtered by `discover:tool:*` permission per principal
- All tool invocations produce audit events via existing `AuditLogger`
- Built-in roles extended: `admin` gets `*:tool:*`, `developer` gets `execute:tool:own`,
  `operator` gets `discover:tool:all`

**Scope**: Documentation only. Implementation deferred to Phase 7.4.

### Prerequisite 6: Performance Benchmarks (Deferred)

**Problem**: No latency or throughput benchmarks exist. The evaluator flagged this, but benchmarks
are not blocking for specification writing.

**Decision**: Defer to Phase 8 (Operations). Note as a Phase 8 requirement.

**Action items for Phase 8**:
- Message encoding/decoding benchmarks (MessagePack round-trip)
- NATS pub/sub round-trip latency under load (p99/p999)
- Supervisor restart latency
- Mailbox backpressure behavior
- JetStream consumer throughput with explicit ack

### Research Sources

The following current sources informed these prerequisites:
- NATS JetStream consumer patterns: [NATS Docs](https://docs.nats.io/using-nats/developer/develop_jetstream/consumers) — pull consumers recommended for new projects
- NATS message deduplication: [NATS Blog](https://nats.io/blog/new-per-subject-discard-policy) — `DiscardNewPerSubject` for infinite dedup
- async-nats 0.46 `Acker` for message acknowledgment: [docs.rs](https://docs.rs/async-nats/latest/async_nats/jetstream/message/struct.Acker.html)
- Rust actor framework comparison (Ractor, Kameo, Actix, Coerce, Xtra): [Ari Seyhun, 2025](https://tqwewe.com/blog/comparing-rust-actor-libraries) — Ractor and Kameo favored for supervision and scalability
- Microsoft Multi-Agent Reference Architecture: [GitHub](https://microsoft.github.io/multi-agent-reference-architecture/docs/reference-architecture/Reference-Architecture.html) — orchestrator + registry + classifier + MCP pattern
- AI agent RBAC design: [Pylar.ai](https://pylar.ai/blog/designing-rbac-for-ai-agents-complete-framework) — context-aware, fine-grained controls for autonomous agents
- MCP Permissions (ReBAC for AI agents): [Permit.io](https://docs.permit.io/mcp-permissions)
- Dynamic authorization for MCP servers: [Cerbos](https://cerbos.dev/blog/dynamic-authorization-for-ai-agents-guide-to-fine-grained-permissions-mcp-servers)

## Authoritative Spec Files

- `spec/data-management/agent-orchestration.md`
- `spec/data-management/agent-lifecycle.md`
- `spec/data-management/agent-communication.md`
- `spec/data-management/agent-operations.md`
- `spec/data-management/agent-integration.md`
- `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md`
- `spec/core-architecture/async-patterns.md`
