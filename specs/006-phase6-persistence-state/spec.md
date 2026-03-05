# Feature Specification: Phase 6 — Persistence & State

**Feature Branch**: `006-phase6-persistence-state`
**Created**: 2026-03-05
**Status**: Draft
**Input**: User description: "Phase 6: Persistence and State — PostgreSQL integration, JetStream KV store, and persistence operations"

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Agent State Survives Restarts (Priority: P1)

A framework operator deploys an agent that accumulates runtime state (task progress, learned context, configuration). When the agent crashes or the system restarts, the agent recovers its last-known state automatically and resumes work without repeating completed steps.

**Why this priority**: Without durable state, the framework loses all agent progress on any failure. This is the foundational persistence promise and the prerequisite for every other persistence feature.

**Independent Test**: Deploy an agent, mutate its state through several operations, terminate the process, restart it, and verify the agent's state matches its pre-termination state.

**Acceptance Scenarios**:

1. **Given** an agent with accumulated runtime state, **When** the agent is terminated and restarted, **Then** the agent's state is fully restored from durable storage within a configurable timeout.
2. **Given** an agent writing state concurrently, **When** a crash occurs mid-write, **Then** the system recovers to the last consistent state without corruption.
3. **Given** multiple agents sharing a storage backend, **When** agents read and write state concurrently, **Then** each agent's state is isolated and no cross-contamination occurs.

---

### User Story 2 — Fast Distributed State Access (Priority: P2)

Agents need sub-millisecond access to frequently read state (session data, coordination locks, agent availability). A distributed key-value layer provides fast reads and writes with automatic expiration, backed by durable storage for recovery.

**Why this priority**: Agents communicate and coordinate at high frequency. Forcing every state read through a relational database adds unacceptable latency. A fast KV layer with SQL fallback gives both speed and durability.

**Independent Test**: Write state to the KV layer, read it back within latency bounds, let it expire, and verify the system falls back to the relational store for recovery.

**Acceptance Scenarios**:

1. **Given** state stored in the KV layer, **When** an agent reads it, **Then** the response completes within the configured latency budget (sub-millisecond for local, low-millisecond for distributed).
2. **Given** state with a configured TTL, **When** the TTL expires, **Then** the entry is automatically removed from the KV layer.
3. **Given** state that has expired from the KV layer, **When** an agent requests it, **Then** the system hydrates the entry from the relational store and populates the KV layer for subsequent reads.
4. **Given** state modified in the KV layer, **When** a configurable flush threshold is reached, **Then** dirty entries are asynchronously persisted to the relational store.

---

### User Story 3 — Schema-Managed Relational Storage (Priority: P3)

A framework operator provisions the database and runs versioned migrations to create or upgrade the schema. The schema supports agent registry, task tracking, message history, audit logging, and configuration — all with proper constraints, indexes, and partitioning for production workloads.

**Why this priority**: Relational storage is the authoritative data store. Without a managed schema and migration path, deployments are fragile and upgrades are manual.

**Independent Test**: Run migrations against an empty database, verify all tables/indexes/constraints exist, insert and query representative data, then run a subsequent migration and verify the upgrade succeeds without data loss.

**Acceptance Scenarios**:

1. **Given** an empty database, **When** migrations are executed, **Then** all schemas, tables, indexes, constraints, and functions are created successfully.
2. **Given** a database at schema version N, **When** migration N+1 is applied, **Then** the schema updates without data loss and the version tracker reflects N+1.
3. **Given** a failed migration, **When** rollback is triggered, **Then** the database returns to its pre-migration state.

---

### User Story 4 — Task and Message Persistence (Priority: P4)

Task assignments, results, and inter-agent messages are durably stored for auditability and replay. Operators can query task history by agent, status, time range, and priority.

**Why this priority**: Task history enables debugging, compliance, and operational insight. It depends on the relational schema (US3) and benefits from the KV cache (US2) for recent entries.

**Independent Test**: Create tasks and messages through the persistence API, query them by various criteria, verify completeness and ordering.

**Acceptance Scenarios**:

1. **Given** a completed task, **When** an operator queries task history by agent and time range, **Then** the task record is returned with full metadata (assignment, result, timing, priority).
2. **Given** high-volume message traffic, **When** messages are persisted, **Then** time-based partitioning keeps query performance stable as volume grows.
3. **Given** a message with a correlation ID, **When** queried by correlation, **Then** the full conversation thread (request, response, follow-ups) is returned in order.

---

### User Story 5 — Persistence Operations and Repository Layer (Priority: P5)

Framework developers use a repository abstraction to interact with storage. The repository handles dual-store routing (KV for hot data, SQL for durable data), optimistic concurrency, conflict resolution, and transactional boundaries — without exposing storage internals to agent code.

**Why this priority**: Agents should not know whether their state lives in KV or SQL. A clean repository interface enables future storage changes without breaking agent code.

**Independent Test**: Use the repository API to create, read, update, and delete entities. Verify CRUD operations work through the dual-store layer, including conflict resolution on concurrent updates.

**Acceptance Scenarios**:

1. **Given** an entity saved through the repository, **When** read back, **Then** the entity matches regardless of which storage layer served the read.
2. **Given** two concurrent updates to the same entity, **When** both attempt to save, **Then** the conflict resolution strategy (last-write-wins, reject, or version-based) is applied and the loser receives an appropriate error or the resolved state.
3. **Given** a repository transaction spanning multiple entities, **When** the transaction commits, **Then** all entities are atomically persisted; on failure, all are rolled back.

---

### Edge Cases

- What happens when the KV layer is unreachable but SQL is available? System degrades to SQL-only reads/writes with a performance warning.
- What happens when SQL is unreachable but KV is available? Writes continue to KV with dirty tracking; flushes are retried with backoff until SQL recovers.
- What happens when both storage backends are unreachable? Operations fail with explicit errors; circuit breaker prevents retry storms.
- What happens when a migration is partially applied? The migration framework detects incomplete state and refuses to proceed until manually resolved.
- What happens when KV entries expire before they are flushed to SQL? Dirty entries must be flushed before TTL expiration; the system tracks flush deadlines independently of KV TTL.
- What happens when the database runs out of disk space? Write operations fail with a storage-capacity error; the system does not silently drop data.
- What happens when optimistic concurrency detects a version conflict during flush? The flush retries with the current version, or escalates if the conflict persists beyond retry limits.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a relational persistence layer using PostgreSQL with connection pooling, health checks, and configurable pool sizing.
- **FR-002**: System MUST provide versioned schema migrations with forward and rollback support, tracked in a migration history table.
- **FR-003**: System MUST define database schemas for agent registry, agent state, tasks, messages, configuration, and audit logging — with UUID primary keys, JSONB for flexible metadata, and referential integrity constraints.
- **FR-004**: System MUST provide time-based partitioning for high-volume tables (messages, audit logs) with partition management functions. Runtime partition automation is deferred to Phase 8.
- **FR-005**: System MUST provide a distributed key-value store using JetStream KV with configurable TTL, bucket tiering (session, agent state, cache), and conflict resolution (last-write-wins, timestamp-based, reject).
- **FR-006**: System MUST implement a hybrid/dual-store pattern where KV serves as the fast-access layer and PostgreSQL serves as the authoritative durable store, with automatic fallback and lazy hydration.
- **FR-007**: System MUST implement dirty-key tracking and configurable flush thresholds to asynchronously persist KV changes to PostgreSQL.
- **FR-008**: System MUST provide a repository abstraction that routes reads and writes to the appropriate storage layer based on data type, hiding storage internals from consumers.
- **FR-009**: System MUST support optimistic concurrency control with version tracking for state updates in both KV and SQL stores.
- **FR-010**: System MUST provide transactional boundaries for multi-entity persistence operations with atomicity guarantees.
- **FR-011**: System MUST implement state hydration — loading agent state from PostgreSQL into JetStream KV on agent startup.
- **FR-012**: System MUST implement state checkpointing — periodic snapshots of agent state for recovery.
- **FR-013**: System MUST integrate with the Phase 2 resource management layer for connection pool lifecycle.
- **FR-014**: System MUST integrate with the Phase 5 security layer for credential management and audit trail persistence.
- **FR-015**: System MUST provide comprehensive error types for persistence operations (not found, duplicate key, version conflict, connection failure, TTL expired, serialization error) with retry classification.
- **FR-016**: System MUST implement circuit breaker and retry-with-backoff patterns for storage operation failures.
- **FR-017**: System MUST provide health check endpoints for both PostgreSQL and JetStream KV connectivity.
- **FR-018**: System MUST maintain index strategies (GIN for JSONB, B-tree for lookups, partial indexes for status filtering) for query performance.

### Key Entities

- **Agent Registry**: Persistent record of all agents — type, name, status, capabilities, configuration, parent relationship, heartbeat timestamp.
- **Agent State**: Key-value pairs associated with an agent — versioned, checksummed, optionally TTL-expiring. Partitioned by agent ID for load distribution.
- **Agent Checkpoint**: Point-in-time snapshot of an agent's full state for recovery, linked to a KV revision for sync tracking.
- **Task Record**: Task lifecycle data — ID, type, payload, metadata, status, assignment, result, timing, priority, correlation chain.
- **Message Record**: Inter-agent message history — sender, receiver, type, content, priority, status, retry tracking, correlation chain, TTL. Time-partitioned.
- **Configuration Record**: System and per-agent configuration entries — environment-scoped, versioned, with change history tracking.
- **Audit Log Entry**: Immutable record of system events — event type, actor (agent or user), resource, action, old/new values, IP, correlation ID. Time-partitioned.
- **KV Bucket**: Logical grouping of key-value entries with shared TTL, replication, and storage configuration (session data, agent state, query cache).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Agent state is fully recoverable after process termination — 100% of pre-termination state entries are present and correct after restart.
- **SC-002**: KV layer read latency is under 1 millisecond for locally-cached entries and under 5 milliseconds for distributed entries, measured at the 95th percentile.
- **SC-003**: Dirty state flushes from KV to SQL complete within 10 seconds of threshold being reached under normal load.
- **SC-004**: Schema migrations execute successfully against an empty database and against each prior schema version, with zero data loss on upgrade.
- **SC-005**: The system handles 100 concurrent agents reading and writing state without deadlocks, data corruption, or performance degradation below acceptable thresholds.
- **SC-006**: All persistence operations return typed, actionable errors — no silent failures, no swallowed exceptions.
- **SC-007**: When either storage backend becomes unavailable, the system degrades gracefully (documented fallback behavior) rather than crashing or losing data.
- **SC-008**: Task and message queries by agent, status, time range, and correlation ID return correct results within 100 milliseconds for tables with up to 1 million rows.

## Assumptions

- PostgreSQL 15+ is available as the relational backend; the framework does not support other relational databases.
- NATS server with JetStream enabled is available for KV operations; the same NATS infrastructure used for transport (Phase 4) is reused.
- Connection pooling leverages the Phase 2 `ConnectionPool` and `Resource` trait infrastructure.
- Credentials for database access are managed through Phase 5 security infrastructure or environment configuration.
- The persistence crate (`mister-smith-persistence` or similar) is a new workspace crate depending on `mister-smith-core`, `mister-smith-config`, `mister-smith-resources`, `mister-smith-nats`, and `mister-smith-security`.
- Audit log persistence (currently in-memory ring buffer from Phase 5) will be wired to the PostgreSQL audit schema in this phase.

## Dependencies

- **Phase 2**: `mister-smith-resources` (ConnectionPool, Resource trait), `mister-smith-async` (circuit breaker, retry), `mister-smith-events` (EventBus for change notifications)
- **Phase 4**: `mister-smith-nats` (JetStream KV access via async-nats 0.46), `mister-smith-transport` (message envelope types)
- **Phase 5**: `mister-smith-security` (credential management, audit logger)
- **External**: sqlx 0.8.6, async-nats 0.46.0 (jetstream + kv features), PostgreSQL 15+

## Scope Boundaries

### In Scope

- PostgreSQL connection management, migration framework, schema definitions
- JetStream KV bucket management, state operations, conflict resolution
- Hybrid dual-store pattern with dirty tracking and flush semantics
- Repository abstraction with data-type-based routing
- State hydration, checkpointing, and recovery
- Persistence error types, retry logic, circuit breaker integration
- Integration with existing Phase 2 resource management and Phase 5 security

### Out of Scope

- Redis caching (spec references exist but Phase 6 roadmap uses JetStream KV exclusively)
- Vector store / semantic search (mentioned in specs as optional future work)
- Agent orchestration logic (Phase 7)
- Observability and metrics export (Phase 8)
- Database backup and recovery automation (operational concern, not framework code)
- User management tables (users, roles, user_roles — deferred; framework agents authenticate via JWT, not database users)
