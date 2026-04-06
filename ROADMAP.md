# Mister Smith — Build Roadmap

Linear build order for the operating-system substrate, end to end. Each phase depends on the
phases above it. Within a phase, subphases are ordered but items at the same level can often be
built in parallel.

This is not an implementation plan. It does not prescribe tasks, timelines, or code. It is a
dependency-aware map of what to build and in what order, with references to the specifications that
define each component.

Current scope note: this roadmap remains the architectural build map through the completed
implementation phases. Phase 10 is now implemented and validated in the repo, and the active
frontier-autonomy artifact set now also includes landed packet
[`023`](specs/023-runtime-truth-and-run-trace/spec.md),
[`024`](specs/024-agent-boundary-security-hardening/spec.md),
[`025`](specs/025-step-level-intelligence-v2/spec.md), and
[`026`](specs/026-first-real-coordinator-subagent-runtime/spec.md), while later packet material
under [`027`](specs/027-capability-discovery-and-interoperability/spec.md),
[`028`](specs/028-selective-strong-coordination/spec.md), and
[`029`](specs/029-session-first-user-shell/spec.md) remains draft or pre-spec planning, plus
[`specs/012-phase10-frontier-autonomy/`](specs/012-phase10-frontier-autonomy/spec.md),
[`WORKFLOW.md`](WORKFLOW.md), [`docs/linear/LINEAR.md`](docs/linear/LINEAR.md), and the dated
plans under [`docs/plans/`](docs/plans/). Use
[`docs/direction.md`](docs/direction.md) for overall system direction,
[`docs/current-state.md`](docs/current-state.md) for current repo truth and what is live on the
default runtime path, and the latest packet notes for bounded closure evidence.

## How to Read This

- **Phases** are major architectural layers. They must be completed roughly in order.
- **Subphases** are components within a phase. Their ordering reflects dependency flow.
- **References** link to the spec files that define each component's contract.
- **Depends on** shows what must exist before a component can be built.
- **Produces** shows what becomes available to downstream phases.
- **Gate** marks a checkpoint — downstream work should not begin until the gate criteria are met.

---

## Phase 1: Foundation

Everything compiles against this layer. No external services, no async runtime, no I/O. Pure types, traits, and error definitions.

### 1.1 Core Types & Error Hierarchy

The type system that every other crate imports. Define it once, get it right — changes here cascade everywhere.

- Core identifier types (`AgentId`, `TaskId`, `MessageId`, `ToolId` — UUID-based newtypes)
- `SystemError` enum and the `thiserror` error hierarchy
- `Result<T>` type alias
- `MessagePriority` enum (Critical=0 through Bulk=4)
- `AgentState` enum (Initializing, Running, Paused, Stopping, Terminated, Error, Restarting)
- `AgentAvailability` enum (Idle, Busy, Error, Offline, Starting, Stopping) for transport status/heartbeat channels
- `AgentType` enum (Supervisor, Worker, Coordinator, Monitor, Planner, Executor, Critic, Router, Memory)

**References**:
- [type-definitions.md](spec/core-architecture/type-definitions.md) — canonical type definitions
- [runtime-and-errors.md](spec/core-architecture/runtime-and-errors.md) — error hierarchy, Result patterns
- [module-organization-type-system.md](spec/core-architecture/module-organization-type-system.md) — module tree and visibility

**Depends on**: Nothing
**Produces**: `mister-smith-core` crate — the universal import

### 1.2 Core Traits

Abstract contracts that define the system's extension points. These are trait definitions only —
no implementations yet.

- `Actor` trait (handle_message, lifecycle hooks)
- `Agent` trait (extends Actor with orchestration capabilities)
- `Tool` trait (execute, capabilities, permissions)
- `Resource` trait (acquire, release, health_check)
- `Supervisor` trait (restart strategies, failure handling)
- `Transport` trait (publish, subscribe, request-reply)

**References**:
- [component-architecture.md](spec/core-architecture/component-architecture.md) — component hierarchy and trait relationships
- [integration-contracts.md](spec/core-architecture/integration-contracts.md) — trait-based contracts between subsystems
- [async-patterns.md](spec/core-architecture/async-patterns.md) — Actor trait definition (Section 3)
- [module-organization-type-system.md](spec/core-architecture/module-organization-type-system.md) — canonical `Tool`/`Agent`/`Resource` trait signatures

**Depends on**: 1.1 (types referenced in trait signatures)
**Produces**: Trait definitions consumed by every implementation crate

### 1.3 Configuration System

Typed configuration loading, validation, and hot-reload support.

- `RuntimeConfig`, `AgentConfig`, `TransportConfig`, `SecurityConfig` structs
- Config file parsing (TOML/YAML)
- Environment variable overlay
- Validation at load time

**References**:
- [implementation-config.md](spec/core-architecture/implementation-config.md) — configuration patterns
- [configuration-management.md](spec/operations/configuration-management.md) — operational config

**Depends on**: 1.1 (config structs reference core types)
**Produces**: `mister-smith-config` crate

> **Gate 1** ✅ (validated 2026-03-04): Core types compile. Trait definitions compile. Configuration loads and validates. No runtime behavior yet — this is all compile-time structure. Run `cargo build -p mister-smith-core` and `cargo build -p mister-smith-config` cleanly.
> Validation checklist: [Phase 1 Deep Dive](plans/roadmap-phases/phase-1-foundation.md).

---

## Phase 2: Runtime & Async Infrastructure

Stand up the Tokio runtime, async execution patterns, and the monitoring/event plumbing that everything else observes through.

### 2.1 Tokio Runtime Manager

The single runtime that hosts all async work. Configuration-driven worker threads, blocking thread pool, shutdown coordination.

- `RuntimeManager` (builder pattern, lifecycle)
- Worker thread configuration
- Graceful shutdown with timeout
- Task spawning and handle tracking

**References**:
- [tokio-runtime.md](spec/core-architecture/tokio-runtime.md) — runtime builder API, task spawning, shutdown
- [system-architecture.md](spec/core-architecture/system-architecture.md) — overall runtime design

**Depends on**: 1.1, 1.3
**Produces**: `mister-smith-runtime` crate

### 2.2 Monitoring & Health

Metrics collection, health checks, and failure detection. This is plumbing — components register health checks and emit metrics; the monitoring system aggregates.

- `HealthMonitor` trait and registry
- `MetricsRegistry` (Prometheus-compatible counters, histograms, gauges)
- Phi accrual failure detector
- Liveness and readiness probe endpoints

**References**:
- [monitoring-and-health.md](spec/core-architecture/monitoring-and-health.md) — health check patterns
- [observability-monitoring-framework.md](spec/operations/observability-monitoring-framework.md) — OpenTelemetry, Prometheus

**Depends on**: 2.1 (async health checks need the runtime)
**Produces**: `mister-smith-monitoring` crate

### 2.3 Event System

In-process pub/sub for system events. This is not NATS — it is internal `tokio::sync::broadcast` channels for components to observe each other.

- `EventBus` (typed publish/subscribe)
- `EventHandler` trait
- System event types (agent spawned, agent failed, config changed, etc.)
- Dead letter handling for undelivered events

**References**:
- [supervision-and-events.md](spec/core-architecture/supervision-and-events.md) — event-driven supervision
- [async-patterns.md](spec/core-architecture/async-patterns.md) — stream processing, backpressure

**Depends on**: 2.1, 2.2 (events emit metrics)
**Produces**: `mister-smith-events` crate

### 2.4 Async Execution Patterns

Reusable async building blocks used across the system. Not a crate consumers interact with directly
— internal infrastructure.

- `TaskExecutor` (structured task spawning with cancellation)
- `TaskGuard` (RAII cleanup for async tasks)
- Stream processing with backpressure
- Circuit breaker
- Timeout and retry combinators
- `DeadlockPreventingMutex`, `CountdownLatch`

**References**:
- [async-patterns.md](spec/core-architecture/async-patterns.md) — comprehensive async patterns (Sections 1-2, 4-5)
- [coding-standards.md](spec/core-architecture/coding-standards.md) — Rust async idioms

**Depends on**: 2.1, 2.2
**Produces**: `mister-smith-async` crate

### 2.5 Resource Management

Generic connection pooling and resource lifecycle. Used by transport, persistence, and any component that holds external connections.

- `ConnectionPool<R: Resource>` (generic over resource type)
- Pool sizing, health checks, eviction
- `ResourceManager` (lifecycle coordination)

**References**:
- [connection-management.md](spec/data-management/connection-management.md) — pooling, reconnection, health
- [component-architecture.md](spec/core-architecture/component-architecture.md) — resource ownership model

**Depends on**: 2.2, 1.3 (pool config, health integration)
**Produces**: `mister-smith-resources` crate

> **Gate 2** ✅ (validated 2026-03-04): The runtime starts, shuts down gracefully, and reports health. Events flow through the bus. Metrics are collected. You can write `#[tokio::test]` tests that exercise the async patterns. No actors, no agents, no external I/O yet.
> Validation checklist: [Phase 2 Deep Dive](plans/roadmap-phases/phase-2-runtime-and-async-infrastructure.md).

---

## Phase 3: Actor System & Supervision

The core concurrency model. Actors communicate through mailboxes; supervisors watch actors and restart them on failure. This is the hardest phase — Erlang/OTP semantics in Rust's ownership model.

### 3.1 Actor Primitives

The basic actor machinery: mailboxes, references, and the actor system that manages lifecycle.

- `Mailbox<M>` (bounded async channel with backpressure)
- `ActorRef<M>` (typed handle — `tell` for fire-and-forget, `ask` for request-response)
- `ActorSystem` (spawns actors, tracks handles, coordinates shutdown)
- Actor lifecycle: spawn → running → stopping → terminated
- Message routing and dispatch

**References**:
- [async-patterns.md](spec/core-architecture/async-patterns.md) — Actor model implementation (Section 3)
- [component-architecture.md](spec/core-architecture/component-architecture.md) — actor hierarchy

**Depends on**: 2.3 (actors emit events), 2.4 (task spawning), 2.2 (actor metrics)
**Produces**: `mister-smith-actor` crate

### 3.2 Supervision Trees

The fault tolerance layer. Supervisors form a tree; when a child fails, the supervisor's restart policy determines what happens.

- `SupervisionTree` (root structure, tree traversal)
- `SupervisorNode` (individual supervisor in the tree)
- `RestartPolicy` enum: OneForOne, OneForAll, RestForOne
- `RestartScope` enum: Permanent, Transient, Temporary
- `SupervisionStrategy` struct (policy + max_restarts + window + escalation)
- Failure detection, backoff, and escalation to parent

**References**:
- [supervision-trees.md](spec/core-architecture/supervision-trees.md) — supervision model, restart strategies
- [supervision-and-events.md](spec/core-architecture/supervision-and-events.md) — event-driven supervision, NodeRestartPolicy
- [type-definitions.md](spec/core-architecture/type-definitions.md) — canonical SupervisionStrategy struct

**Depends on**: 3.1 (supervises actors), 2.3 (supervision events), 2.2 (failure detection)
**Produces**: `mister-smith-supervision` crate

> **Gate 3** ✅ (validated 2026-03-04): Actors can be spawned, communicate via mailboxes, and be supervised. A failing actor triggers its supervisor's restart policy. Supervision trees can be composed hierarchically. This is the architectural proof point — if supervision works, the runtime's concurrency model is sound.
> Validation checklist: [Phase 3 Deep Dive](plans/roadmap-phases/phase-3-actor-system-and-supervision.md).

---

## Phase 4: Transport & Messaging

Connect the actor system to the outside world. NATS is the primary transport; HTTP and gRPC are external-facing interfaces.

### 4.1 Transport Abstraction

The `Transport` trait and message envelope — protocol-agnostic layer that NATS, HTTP, and gRPC implement.

- `Transport` trait (publish, subscribe, request, reply)
- Message envelope: headers, priority, correlation ID, payload
- Serialization/deserialization (serde + Bytes)
- Transport-level error types

**References**:
- [transport-core.md](spec/transport/transport-core.md) — core transport types
- [transport-layer-specifications.md](spec/transport/transport-layer-specifications.md) — transport abstraction

**Depends on**: 1.1, 1.2 (Transport trait, message types), 2.3 (event integration)
**Produces**: Transport trait and envelope types

### 4.2 NATS Transport

The primary inter-agent communication layer. This is the most critical transport — agents talk to each other through NATS subjects.

- async-nats 0.46 `Client` integration (feature-gated: jetstream, kv, object-store, service)
- Hierarchical subject-based routing (`agents.{id}.commands.{type}`, `tasks.{type}.assignment`, etc.)
- Queue groups for load balancing
- Request-reply with timeout
- Publish with backpressure (async-nats 0.46 publish returns a future)
- Reconnection handling
- JetStream for durable messaging (streams, consumers)

**References**:
- [nats-transport.md](spec/transport/nats-transport.md) — async-nats 0.46 API, subject taxonomy, JetStream
- [connection-management.md](spec/data-management/connection-management.md) — reconnection, health monitoring

**Depends on**: 4.1, 2.5 (connection pooling)
**Produces**: NATS transport implementation

### 4.3 Message Schemas

The concrete message types that flow through the transport layer. Defined as Rust structs with serde derives.

- Core messages: TaskAssignment, TaskResult, AgentHeartbeat, SystemEvent
- Status messages use `AgentAvailability` (transport presence/capacity), not lifecycle `AgentState`
- Workflow messages: WorkflowStart, StepComplete, WorkflowResult
- System messages: AgentSpawn, AgentTerminate, ConfigUpdate
- LLM backend integration messages

**References**:
- [message-schemas.md](spec/data-management/message-schemas.md) — all message type definitions
- [core-message-schemas.md](spec/data-management/core-message-schemas.md) — system-level messages
- [workflow-message-schemas.md](spec/data-management/workflow-message-schemas.md) — workflow messages
- [system-message-schemas.md](spec/data-management/system-message-schemas.md) — control plane messages
- [message-framework.md](spec/data-management/message-framework.md) — routing, validation

**Depends on**: 1.1, 4.1 (message envelope)
**Produces**: Typed message structs used by agents and persistence

### 4.4 HTTP Transport

External REST API for management, monitoring, and client integration.

- Axum 0.8 router and handlers
- WebSocket support for streaming
- Middleware: rate limiting, request ID tracking, and security hooks (auth enforced in Phase 5)
- OpenAPI-compatible endpoint structure

**References**:
- [http-transport.md](spec/transport/http-transport.md) — Axum handlers, routing, middleware

**Depends on**: 4.1 (transport contract and envelope)
**Produces**: HTTP API server

### 4.5 gRPC Transport

High-performance inter-service communication for service mesh deployments.

- Tonic 0.14 service definitions
- Protobuf message definitions (prost 0.14)
- Streaming RPCs for agent communication
- Status → FrameworkError mapping

**References**:
- [grpc-transport.md](spec/transport/grpc-transport.md) — Tonic patterns, protobuf schemas, error mapping

**Depends on**: 4.1 (service definitions and transport contracts)
**Produces**: gRPC service layer

> **Gate 4** ✅ (validated 2026-03-04): Agents can communicate over NATS. Messages serialize, route, and deserialize
> correctly. A basic integration test sends a TaskAssignment through NATS and receives a
> TaskResult back. HTTP and gRPC endpoints accept requests with pluggable security middleware
> points ready for Phase 5 enforcement. JetStream stores durable messages. Transport status
> channels use `AgentAvailability` semantics (idle/busy/offline), while lifecycle control uses
> Phase 7 `AgentState`.
> Validation checklist: [Phase 4 Deep Dive](plans/roadmap-phases/phase-4-transport-and-messaging.md).

---

## Phase 5: Security

Authentication, authorization, and encrypted transport. Positioned here because transport and persistence (Phase 6) need security middleware, but security itself only needs core types.

### 5.1 Authentication

Identity verification for agents and external clients.

- JWT token generation and validation (jsonwebtoken 10)
- Agent identity tokens (embedded AgentType, permissions)
- Token refresh and expiry
- API key support for external clients

**References**:
- [authentication-specifications.md](spec/security/authentication-specifications.md) — auth flows, token formats
- [authentication-implementation.md](spec/security/authentication-implementation.md) — JWT implementation

**Depends on**: 1.1, 1.3 (security config)
**Produces**: `AuthService`, token types

### 5.2 Authorization

Permission checking for agent operations and API access.

- RBAC model (roles map to permissions)
- Permission types: agent operations, tool access, resource access, API endpoints
- Middleware for transport layers (Axum extractors, Tonic interceptors)
- Policy evaluation engine

**References**:
- [authorization-specifications.md](spec/security/authorization-specifications.md) — RBAC/ABAC model
- [authorization-implementation.md](spec/security/authorization-implementation.md) — permission checking

**Depends on**: 5.1 (auth tokens carry role claims)
**Produces**: `PermissionSystem`, auth middleware

### 5.3 TLS & Certificate Management

Encrypted transport and mutual TLS for agent-to-agent communication.

- rustls 0.23 integration (CertificateDer, builder_with_provider, WebPkiClientVerifier)
- Certificate generation for dev/test (rcgen 0.14)
- mTLS for NATS connections
- Certificate rotation

**References**:
- [security-framework.md](spec/security/security-framework.md) — overall security architecture
- [security-integration.md](spec/security/security-integration.md) — security ↔ transport integration, mTLS
- [security-patterns.md](spec/security/security-patterns.md) — rustls/ring APIs, threat model

**Depends on**: 5.1, 5.2
**Produces**: `mister-smith-security` crate (TLS configs consumed by transport)

> **Gate 5** ✅ (validated 2026-03-04): Agents authenticate with JWT tokens. Authorization middleware rejects unauthorized requests. NATS connections use mTLS. HTTP and gRPC endpoints enforce auth. Security is now wired into all transport paths.
> Validation checklist: [Phase 5 Deep Dive](plans/roadmap-phases/phase-5-security.md).

---

## Phase 6: Persistence & State

Durable storage for agent state, task history, and system configuration. Two storage backends: PostgreSQL for relational data, JetStream KV for distributed ephemeral state.

### 6.1 PostgreSQL Integration

Relational storage for agent state, task records, and audit logs.

- sqlx connection pool (via Resource trait from 2.5)
- Database schema and migrations
- Query patterns (prepared statements, transactions)
- Connection health monitoring

**References**:
- [postgresql-implementation.md](spec/data-management/postgresql-implementation.md) — connection management, query patterns
- [database-schemas.md](spec/data-management/database-schemas.md) — SQL schemas, migrations
- [data-persistence.md](spec/data-management/data-persistence.md) — persistence strategy

**Depends on**: 2.5 (connection pooling), 5.1 (credential management)
**Produces**: PostgreSQL storage backend

### 6.2 JetStream KV Store

Distributed key-value store for ephemeral agent state, coordination, and caching.

- async-nats 0.46 JetStream KV API
- Agent state snapshots
- Distributed locking
- TTL-based expiry

**References**:
- [jetstream-kv.md](spec/data-management/jetstream-kv.md) — JetStream KV API patterns
- [storage-patterns.md](spec/data-management/storage-patterns.md) — caching, write-ahead patterns

**Depends on**: 4.2 (NATS connection), 2.5 (resource management)
**Produces**: KV storage backend

### 6.3 Persistence Operations

CRUD patterns, transactions, and event sourcing that sit on top of the storage backends.

- Repository pattern for agent state
- Event sourcing for audit trail
- Data integration patterns (agent ↔ storage)

**References**:
- [persistence-operations.md](spec/data-management/persistence-operations.md) — CRUD, transactions
- [data-integration-patterns.md](spec/data-management/data-integration-patterns.md) — data flow between agents and storage

**Depends on**: 6.1, 6.2
**Produces**: Persistence layer consumed by the agent system

> **Gate 6** ✅ (validated 2026-03-05): Agent state persists across restarts. Task history is queryable. JetStream KV provides distributed coordination. Database migrations run cleanly.
> Validation checklist: [Phase 6 Deep Dive](plans/roadmap-phases/phase-6-persistence-and-state.md).

---

## Phase 7: Agent System

The agent orchestration layer — the reason the system exists. Agents are actors with
orchestration capabilities: they form teams, decompose tasks, communicate through the transport
layer, and are supervised.

### 7.1 Agent Lifecycle

The state machine that governs agent existence from spawn to termination.

- `AgentState` transitions (Initializing → Running → Paused → Stopping → Terminated)
- Spawn and terminate operations
- Health level tracking (Healthy, Degraded, Unhealthy, Critical)
- Restart integration with supervision (Phase 3.2)

**References**:
- [agent-lifecycle.md](spec/data-management/agent-lifecycle.md) — state machine, spawn/terminate, restart policies
- [agent-operations.md](spec/data-management/agent-operations.md) — runtime operations, commands, status

**Depends on**: 3.1 (Actor), 3.2 (Supervision), 4.2 (NATS for heartbeats)
**Produces**: Agent lifecycle management

### 7.2 Agent Communication

How agents talk to each other through the transport layer.

- Inter-agent messaging patterns (request-reply, publish-subscribe, fan-out)
- Subject-based routing conventions
- Priority-aware message queuing
- Conversation tracking (correlation IDs)

**References**:
- [agent-communication.md](spec/data-management/agent-communication.md) — messaging patterns, priority queuing
- [agent-integration.md](spec/data-management/agent-integration.md) — agent ↔ external system integration

**Depends on**: 4.2 (NATS), 4.3 (message schemas), 7.1 (agent must be alive to communicate)
**Produces**: Agent communication layer

### 7.3 Agent Orchestration

Team composition, task decomposition, and multi-agent coordination.

- `AgentOrchestrator` (spawning, team management)
- Team patterns: supervisor-worker, pipeline, consensus
- Task decomposition and assignment
- Result aggregation

**References**:
- [agent-orchestration.md](spec/data-management/agent-orchestration.md) — orchestration patterns, team composition
- [SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md](spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md) — 15 specialized agent domains

**Depends on**: 7.1, 7.2, 3.2 (supervision for teams)
**Produces**: Multi-agent orchestration

### 7.4 Tool System

The agent-as-tool pattern — any agent can be wrapped and exposed as a callable tool to other agents.

- `ToolBus` (central tool registry)
- `AgentTool` wrapper (agent → tool interface)
- Permission-gated tool access
- Tool execution metrics and timeouts

**References**:
- [async-patterns.md](spec/core-architecture/async-patterns.md) — agent-as-tool pattern (Section 6), tool system core (Section 7)
- [integration-patterns.md](spec/core-architecture/integration-patterns.md) — inter-component communication

**Depends on**: 7.1, 5.2 (tool permissions)
**Produces**: Tool system enabling hierarchical agent composition

### 7.5 Specialized Agent Implementations

The 9 concrete agent types, each with domain-specific behavior.

- **Supervisor**: Manages child agent teams, applies restart policies
- **Worker**: Executes assigned tasks, reports results
- **Coordinator**: Decomposes complex tasks, delegates to workers
- **Monitor**: Observes system health, triggers alerts
- **Planner**: Generates execution plans from high-level goals
- **Executor**: Carries out planned steps
- **Critic**: Evaluates outputs, provides feedback
- **Router**: Directs messages based on content/priority
- **Memory**: Manages shared context and knowledge

**References**:
- [SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md](spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md) — all 15 domains
- [agent-orchestration.md](spec/data-management/agent-orchestration.md) — agent type definitions

**Depends on**: 7.1–7.4 (all agent infrastructure)
**Produces**: `mister-smith-agents` crate — the complete agent system

> **Gate 7** ✅ (validated 2026-03-05): A multi-agent team can be spawned: a Coordinator decomposes a task, assigns subtasks to Workers via NATS, Workers execute and report results, a Supervisor restarts any Worker that fails, and results aggregate back to the Coordinator. This is the end-to-end proof of the operating-system substrate.
> Validation checklist: [Phase 7 Deep Dive](plans/roadmap-phases/phase-7-agent-system.md).

---

## Phase 8: Operations & Production Readiness

Everything needed to deploy, observe, and operate the system in production.

### 8.1 Observability

Distributed tracing, structured logging, and metrics export.

- OpenTelemetry integration (OTLP export)
- Distributed trace propagation across agents
- Structured logging with tracing crate
- Prometheus metrics endpoint
- Dashboard definitions

**References**:
- [observability-monitoring-framework.md](spec/operations/observability-monitoring-framework.md) — tracing, metrics, alerting
- [async-patterns.md](spec/core-architecture/async-patterns.md) — distributed tracing integration (Section 9)

**Depends on**: 2.2, 4.4 (metrics endpoint via HTTP)
**Produces**: Observability pipeline

### 8.2 Process Management

System startup, shutdown, signal handling, and process lifecycle.

- Startup sequencing (config → runtime → transport → agents)
- Graceful shutdown (drain connections, stop agents, flush state)
- Signal handling (SIGTERM, SIGINT)
- PID management

**References**:
- [process-management-specifications.md](spec/operations/process-management-specifications.md) — process lifecycle, signals
- [system-architecture.md](spec/core-architecture/system-architecture.md) — startup/shutdown sequences

**Depends on**: All prior phases (orchestrates the full stack)
**Produces**: Main binary entry point

### 8.3 Deployment

Container images, Kubernetes manifests, and deployment configuration.

- Dockerfile (multi-stage, rust:1.88)
- Kubernetes Deployment, Service, ConfigMap, Secret manifests
- Helm chart (optional)
- Health check probes (liveness → 2.2, readiness → 4.2)
- Resource limits and scaling policies

**References**:
- [deployment-architecture-specifications.md](spec/operations/deployment-architecture-specifications.md) — Kubernetes deployment
- [configuration-deployment-specifications.md](spec/operations/configuration-deployment-specifications.md) — deployment configs
- [build-specifications.md](spec/operations/build-specifications.md) — Cargo build, Docker, cross-compilation

**Depends on**: 8.2 (binary to deploy), 8.1 (observability endpoints)
**Produces**: Deployable artifacts

> **Gate 8** ✅ (validated 2026-03-06): The system runs as a containerized service. Health probes respond. Traces appear in the collector. Metrics are scraped. Graceful shutdown completes without message loss. The system is production-ready.
> Validation checklist: [Phase 8 Deep Dive](plans/roadmap-phases/phase-8-operations-and-production-readiness.md). Implementation: [specs/010-phase8-operations/](specs/010-phase8-operations/).

---

## Phase 9: LLM Provider Integration

Model-agnostic LLM connectivity — the layer that turns the operating system into a runtime that can
call real models.

### 9.1 Core Types & MockProvider

`ModelProvider` trait, unified message types, and a mock provider for testing.

- `ModelProvider` trait: `complete()`, `stream()`, `embed()`, `capabilities()`
- `CompletionRequest` / `CompletionResponse` / `ChatMessage` unified types
- `ToolDefinition` / `ToolCall` / `ToolResult` for function calling
- `StreamChunk` with async `Stream` support (SSE parsing)
- `MockProvider` for deterministic testing (always available, no feature flag)
- `LlmError` in mister-smith-core (follows SecurityError/PersistenceError pattern)

**References**:
- [agent-orchestration.md](spec/data-management/agent-orchestration.md) §10.4 — LLM task coordination patterns
- [message-schemas.md](spec/data-management/message-schemas.md) §5 — Hook event schemas (informational, not implemented here)

**Depends on**: Phase 1 (core types)
**Produces**: `mister-smith-llm` crate with `MockProvider`, all trait tests passing

### 9.2 Anthropic Provider

Claude integration via the Anthropic Messages API.

- `AnthropicProvider` implementing `ModelProvider`
- Completions + streaming (SSE)
- Tool use support
- Embeddings
- Rate limit handling with retry-after

**Depends on**: 9.1
**Produces**: Working Claude provider, env-gated integration tests

### 9.3 OpenAI Provider

GPT integration via the OpenAI Chat Completions API.

- `OpenAiProvider` implementing `ModelProvider`
- Completions + streaming (SSE)
- Function calling / tool use
- Embeddings API

**Depends on**: 9.1
**Produces**: Working GPT provider, env-gated integration tests

### 9.4 Agent–LLM Bridge

Wire `ModelProvider` into the agent system as an optional capability.

- `llm` feature flag in `mister-smith-agents`
- `AgentRuntime::with_model()` constructor
- Planner, Critic, Executor roles gain LLM-powered implementations
- Orchestrator can call models during decompose/aggregate phases

**References**:
- [agent-orchestration.md](spec/data-management/agent-orchestration.md) — Agent trait, orchestration flow
- [SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md](spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md) §15 — Neural/AI Ops domain (informational)

**Depends on**: 9.1, Phase 7 (agents)
**Produces**: Agent roles that call real models

### 9.5 Tool Calling Bridge

Bidirectional bridge between `ToolBus` and LLM tool calling.

- `ToolBus::to_tool_definitions()` exports registered tools as JSON Schema
- `ToolBus::execute_tool_call()` dispatches LLM tool calls to handlers
- Round-trip: model requests tool → ToolBus executes → result returns to model

**Depends on**: 9.2 or 9.3 (needs a real provider), 9.4
**Produces**: End-to-end tool calling

> **Gate 9** ✅ (validated 2026-03-07, stabilized 2026-03-08 via PRs #118-#128): A Planner agent receives a task, calls a real LLM via `ModelProvider`, gets a structured subtask decomposition, and the Orchestrator assigns subtasks to Workers. The same flow works with at least 2 providers (Anthropic + OpenAI). Tool calls round-trip through the ToolBus. No provider-specific code leaks outside the providers/ module.
> Design document: [LLM Provider Integration Design](docs/plans/2026-03-05-llm-provider-integration-design.md). Implementation: [specs/009-phase9-llm-provider-integration/](specs/009-phase9-llm-provider-integration/).

---

## Phase Summary

| Phase | What | Depends On | Key Risk |
|-------|------|------------|----------|
| 1. Foundation | Types, traits, config | — | Type design decisions cascade everywhere |
| 2. Runtime | Tokio, monitoring, events, async patterns | Phase 1 | Runtime configuration and shutdown semantics |
| 3. Actors & Supervision | Actor system, supervision trees | Phase 2 | Erlang/OTP semantics in Rust ownership model |
| 4. Transport | NATS, HTTP, gRPC, message schemas | Phases 1-2 | async-nats 0.46 API surface, backpressure handling |
| 5. Security | Auth, authz, TLS, certificates | Phase 1 | mTLS + NATS integration, token lifecycle |
| 6. Persistence | PostgreSQL, JetStream KV | Phases 2, 4, 5 | Schema migrations, distributed state consistency |
| 7. Agents | Lifecycle, communication, orchestration, tools | Phases 3-6 | Multi-agent coordination, the supervision-to-orchestration bridge |
| 8. Operations | Observability, process mgmt, deployment | All phases | Startup sequencing, graceful shutdown under load |
| 9. LLM Providers | Model-agnostic LLM connectivity, tool calling | Phases 1, 7 | Provider API instability, streaming reliability |

## Parallelism Opportunities

While the phases are sequential at the macro level, some work can overlap:

- **Phases 4 and 5** can proceed in parallel (transport and security are mostly independent until integration)
- **Phase 6** can begin once 4.2 (NATS) and 5.1 (auth) are done — does not need all of Phase 5
- **Phase 8.1** (observability) can begin alongside Phase 7 — it depends on Phase 2, not Phase 7
- **Phase 9.1–9.3** (LLM core + providers) can proceed alongside Phase 8 — they depend only on Phase 1
- **Phase 9.4–9.5** (agent bridge + tool calling) require Phase 7 but not Phase 8
- Within any phase, items at the same subphase level with different dependency chains can be built concurrently

## Critical Path

The longest dependency chain — the sequence that determines minimum calendar time:

```
1.1 Core Types
 → 2.1 Runtime
  → 2.4 Async Patterns
   → 3.1 Actor Primitives
    → 3.2 Supervision Trees
     → 7.1 Agent Lifecycle
      → 7.3 Agent Orchestration
       → 8.2 Process Management
        → 9.4 Agent–LLM Bridge
         → 9.5 Tool Calling Bridge
```

Supervision (3.2) is the architectural chokepoint. It depends on actors, events, and monitoring — and everything downstream (agents, orchestration, operations, LLM integration) depends on it. This is the highest-risk component and the one most likely to force design revisions upstream.

Note: Phase 9.1–9.3 (LLM core types and providers) are off the critical path — they depend only on Phase 1 and can be built in parallel with Phases 8.

## Crate Map

| Crate | Phase | Primary Spec |
|-------|-------|-------------|
| `mister-smith-core` | 1.1–1.2 | [type-definitions.md](spec/core-architecture/type-definitions.md) |
| `mister-smith-config` | 1.3 | [implementation-config.md](spec/core-architecture/implementation-config.md) |
| `mister-smith-runtime` | 2.1 | [tokio-runtime.md](spec/core-architecture/tokio-runtime.md) |
| `mister-smith-monitoring` | 2.2 | [monitoring-and-health.md](spec/core-architecture/monitoring-and-health.md) |
| `mister-smith-events` | 2.3 | [supervision-and-events.md](spec/core-architecture/supervision-and-events.md) |
| `mister-smith-async` | 2.4 | [async-patterns.md](spec/core-architecture/async-patterns.md) |
| `mister-smith-resources` | 2.5 | [connection-management.md](spec/data-management/connection-management.md) |
| `mister-smith-actor` | 3.1 | [async-patterns.md](spec/core-architecture/async-patterns.md) (Section 3) |
| `mister-smith-supervision` | 3.2 | [supervision-trees.md](spec/core-architecture/supervision-trees.md) |
| `mister-smith-security` | 5.1–5.3 | [security-framework.md](spec/security/security-framework.md) |
| `mister-smith-transport` | 4.1–4.5 | [transport-core.md](spec/transport/transport-core.md) |
| `mister-smith-nats` | 4.2 | [nats-transport.md](spec/transport/nats-transport.md) |
| `mister-smith-http` | 4.4 | [http-transport.md](spec/transport/http-transport.md) |
| `mister-smith-grpc` | 4.5 | [grpc-transport.md](spec/transport/grpc-transport.md) |
| `mister-smith-mcp` | 4.6 | [mcp-specifications.md](spec/transport/mcp-specifications.md) |
| `mister-smith-persistence` | 6.1–6.3 | [persistence-layer.md](spec/data-management/persistence-layer.md) |
| `mister-smith-agents` | 7.1–7.5 | [agent-orchestration.md](spec/data-management/agent-orchestration.md) |
| `mister-smith-app` | 8.1–8.3 | [process-management-specifications.md](spec/operations/process-management-specifications.md) |
| `mister-smith-llm` | 9.1–9.6 | [LLM Provider Integration Design](docs/plans/2026-03-05-llm-provider-integration-design.md) |
| `mister-smith-integration-tests` | All | Cross-crate validation |

## Existing Implementation Plans

Detailed implementation plans exist for the first batch (core architecture):

| Plan | Covers |
|------|--------|
| [agent01 — System Architecture](plans/batch1-core-architecture/agent01-system-architecture-implementation.md) | Phase 2.1 |
| [agent02 — Component Architecture](plans/batch1-core-architecture/agent02-component-architecture-implementation.md) | Phases 1.2, 2.5 |
| [agent04 — Supervision Trees](plans/batch1-core-architecture/agent04-supervision-trees-implementation.md) | Phase 3.2 |
| [agent05 — Module Organization](plans/batch1-core-architecture/agent05-module-organization-implementation.md) | Workspace structure |
| [agent06 — Type System](plans/batch1-core-architecture/agent06-type-system-implementation.md) | Phase 1.1 |
| [agent07 — Actor Model](plans/batch1-core-architecture/agent07-actor-model-implementation.md) | Phase 3.1 |
| [agent08 — Core Integration](plans/batch1-core-architecture/agent08-core-integration-implementation.md) | Cross-phase integration |
| [agent16 — Data Flow](plans/batch2-data-management/agent16-data-flow-integration-implementation.md) | Phase 6.3 |
| [Planning Tracker](plans/IMPLEMENTATION_PLANNING_TRACKER.md) | Overall status |

## Phase Deep-Dive Documents

- [Phase 1 — Foundation](plans/roadmap-phases/phase-1-foundation.md)
- [Phase 2 — Runtime and Async Infrastructure](plans/roadmap-phases/phase-2-runtime-and-async-infrastructure.md)
- [Phase 3 — Actor System and Supervision](plans/roadmap-phases/phase-3-actor-system-and-supervision.md)
- [Phase 4 — Transport and Messaging](plans/roadmap-phases/phase-4-transport-and-messaging.md)
- [Phase 5 — Security](plans/roadmap-phases/phase-5-security.md)
- [Phase 6 — Persistence and State](plans/roadmap-phases/phase-6-persistence-and-state.md)
- [Phase 7 — Agent System](plans/roadmap-phases/phase-7-agent-system.md)
- [Phase 8 — Operations and Production Readiness](plans/roadmap-phases/phase-8-operations-and-production-readiness.md)
- [Phase 9 — LLM Provider Integration](docs/plans/2026-03-05-llm-provider-integration-design.md)

## Technology Stack

| Component | Version | Phase | Notes |
|-----------|---------|-------|-------|
| Rust | 1.88.0 (MSRV) | All | Driven by async-nats 0.46 |
| Tokio | 1.49.0 | 2+ | Runtime foundation (validated baseline; 1.50.0 available upstream) |
| async-nats | 0.46.0 | 4+ | Feature-gated: jetstream, kv, object-store, service |
| Axum | 0.8.8 | 4.4 | HTTP transport |
| Tonic | 0.14.5 | 4.5 | gRPC transport |
| sqlx | 0.8.6 | 6.1 | PostgreSQL async driver |
| jsonwebtoken | 10.3.0 | 5.1 | JWT with aws_lc_rs backend |
| rustls | 0.23.37 | 5.3 | TLS implementation |
| serde | 1.0.228 | All | Serialization |
| thiserror | 1.0.69 | 1.1 | Error derives (staying on 1.x) |
| tracing | 0.1.44 | 8.1 | Structured logging |
| opentelemetry | 0.31.0 | 8.1 | Distributed tracing |
| reqwest | 0.12+ | 9.1 | HTTP client for LLM provider APIs (feature-gated per provider) |

See [VERSION_REFERENCE.md](VERSION_REFERENCE.md) for the complete version matrix and migration notes.
See [VALIDATION_REPORT.md](docs/code-review/VALIDATION_REPORT.md) for specification readiness assessment (95/100).
