# Feature Specification: Phase 8 — Operations & Production Readiness

**Feature Branch**: `010-phase8-operations`
**Created**: 2026-03-06
**Status**: Draft
**Input**: Phase 8 of the Mister Smith build roadmap — observability, process management, deployment, application bootstrap, cross-phase integration wiring, health dashboards, and operational tooling.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Application Bootstrap and Process Lifecycle (Priority: P1)

A framework operator starts the Mister Smith process. The system initializes in a deterministic sequence — loading configuration, connecting to external services (messaging, database), starting the supervision tree, spawning initial agents, and registering health endpoints. On receiving a termination signal, the system drains in-flight messages, stops agents in reverse start order, flushes pending state, and exits cleanly with a zero exit code.

**Why this priority**: Without a working binary that starts and stops reliably, nothing else in Phase 8 matters. This is the foundation that all other operational capabilities depend on.

**Independent Test**: Start the process, verify health endpoints respond, send a termination signal, verify clean shutdown with no message loss.

**Acceptance Scenarios**:

1. **Given** a valid configuration file and reachable external services, **When** the operator starts the process, **Then** the system reaches a "ready" state within a configured timeout and all health probes respond healthy.
2. **Given** a running system with active agents and in-flight messages, **When** a termination signal is received, **Then** agents stop in reverse start order, pending messages drain, state flushes to persistence, and the process exits with code 0.
3. **Given** a configuration file referencing an unreachable service, **When** the operator starts the process, **Then** the system reports the specific failure, logs the details, and exits with a non-zero exit code within a bounded timeout.
4. **Given** a running system, **When** a second termination signal is received during graceful shutdown, **Then** the system performs an immediate forced shutdown and exits.

---

### User Story 2 - Observability Pipeline (Priority: P1)

A site reliability engineer (SRE) operating a Mister Smith deployment needs to understand what the system is doing in real time. Every agent operation, message exchange, task assignment, and supervision event emits structured telemetry — distributed traces that follow work across agent boundaries, metrics that show throughput and error rates, and structured logs that correlate with traces. All telemetry flows to standard collection infrastructure without requiring custom tooling.

**Why this priority**: Equal to P1 because an unobservable system is an undeployable system. The constitution (Principle IX) mandates observability by default.

**Independent Test**: Start the system, trigger agent operations, verify traces appear in a collector, metrics are scrapable, and logs include trace correlation IDs.

**Acceptance Scenarios**:

1. **Given** a running system with an observability collector configured, **When** an agent processes a task, **Then** a distributed trace spanning the full request lifecycle (receipt → decomposition → execution → response) appears in the collector.
2. **Given** a running system, **When** an SRE queries the metrics endpoint, **Then** they see counters for messages sent/received, task completions/failures, agent restarts, and active agent count — all with agent type and ID labels.
3. **Given** a running system with structured logging enabled, **When** any component logs an event, **Then** the log entry includes a trace ID, span ID, timestamp, severity, component name, and structured key-value context.
4. **Given** a multi-agent workflow crossing 3+ agents, **When** an SRE views the trace, **Then** the trace shows parent-child span relationships across all agents involved, with timing for each span.

---

### User Story 3 - Cross-Phase Integration Wiring (Priority: P2)

The framework operator expects all infrastructure built in prior phases to work together as a coherent system. Specifically: security audit events from Phase 5's in-memory logger drain to Phase 6's PostgreSQL audit repository. Phase 7 agents authenticate using Phase 5's JWT tokens and are authorized by the RBAC policy engine. Phase 2's health monitor and phi accrual failure detector receive heartbeats from Phase 7 agents and feed supervision decisions.

**Why this priority**: The deviation report identified three critical cross-phase gaps. Constitution Principle VIII mandates these integrations before production readiness. This is P2 because it requires the bootstrap (US1) to exist first.

**Independent Test**: Start the system, generate audit events, verify they persist to the database. Spawn an agent without valid credentials, verify it is rejected. Stop an agent's heartbeats, verify the failure detector triggers a supervision event.

**Acceptance Scenarios**:

1. **Given** a running system with persistence enabled, **When** security audit events are recorded by the audit logger, **Then** those events appear in the audit repository within the configured flush interval, with no events lost.
2. **Given** a running system with security enabled, **When** an agent attempts to perform an operation it is not authorized for, **Then** the operation is denied and an audit event is recorded.
3. **Given** a running system with monitoring enabled, **When** an agent stops sending heartbeats for longer than the failure detection threshold, **Then** the phi accrual failure detector marks the agent as suspected-failed and notifies the appropriate supervisor.
4. **Given** a running system, **When** a supervisor restarts a failed agent, **Then** the new agent instance receives fresh credentials and the restart event is recorded in both the monitoring metrics and the audit log.

---

### User Story 4 - Containerization and Deployment Artifacts (Priority: P3)

A DevOps engineer deploys Mister Smith to a container orchestration platform. The framework ships as a minimal container image with health check probes, configurable resource limits, and environment-based configuration overlay. Kubernetes manifests define the deployment, service, and configuration resources. The image starts quickly and responds to orchestrator health checks.

**Why this priority**: Deployment artifacts are the delivery mechanism. Important but depends on the binary (US1) and observability (US2) being complete first.

**Independent Test**: Build the container image, run it, verify health probes respond, verify configuration via environment variables works, deploy to a test cluster.

**Acceptance Scenarios**:

1. **Given** the framework source code, **When** a DevOps engineer builds the container image, **Then** the resulting image is under 100MB (compressed), contains only the runtime binary and minimal OS dependencies, and starts in under 5 seconds.
2. **Given** a container image deployed to a cluster, **When** the orchestrator sends liveness and readiness probes, **Then** the liveness probe confirms the process is alive and the readiness probe confirms all external dependencies are connected.
3. **Given** a deployed instance, **When** configuration values are provided via environment variables, **Then** those values override the corresponding file-based configuration without requiring an image rebuild.
4. **Given** a deployed instance receiving traffic, **When** the orchestrator scales to multiple replicas, **Then** each replica connects to shared messaging and persistence services, and work is distributed across replicas without duplication.

---

### User Story 5 - Health Dashboard and Alerting (Priority: P3)

An operator monitoring a Mister Smith deployment views a pre-built dashboard showing system health at a glance — active agents by type, message throughput, task completion rates, error rates, supervision tree status, and resource utilization. Alerts fire when critical thresholds are breached (agent failure rate spikes, message queue depth grows, heartbeat loss detected).

**Why this priority**: Dashboards and alerting are the operational UX layer on top of observability (US2). Valuable but can be delivered after the telemetry pipeline is working.

**Independent Test**: Import the dashboard definition into a visualization tool, connect it to the metrics endpoint, verify all panels render with live data.

**Acceptance Scenarios**:

1. **Given** a running system with metrics collection, **When** an operator imports the provided dashboard definition, **Then** all panels display live data for agent counts, message rates, task throughput, and error rates.
2. **Given** a dashboard connected to live metrics, **When** an agent fails and is restarted by its supervisor, **Then** the dashboard reflects the failure event, the restart, and the updated agent count within the metrics scrape interval.
3. **Given** configured alert rules, **When** the agent failure rate exceeds the threshold for the configured duration, **Then** an alert fires and is routed to the configured notification channel.

---

### Edge Cases

- What happens when the process starts but cannot connect to any external service (messaging, database, all unavailable)?
- How does the system handle a configuration file that is valid syntactically but contains contradictory settings (e.g., agent references a nonexistent role)?
- What happens when the observability collector is unreachable — does telemetry buffer, drop, or block the system?
- How does the system behave when disk space runs out during audit log persistence?
- What happens when a container orchestrator sends a SIGTERM during the startup sequence before the system reaches "ready" state?
- How does the system handle clock skew between replicas affecting distributed trace ordering?

## Clarifications

### Session 2026-03-06

No critical ambiguities detected. All categories (functional scope, domain model, interaction flows, non-functional attributes, integration dependencies, edge cases, constraints, terminology, completion signals) assessed as Clear. The spec was authored with full access to 65+ framework specification documents, the updated constitution (v1.1.0 with Principles VIII and IX), and the 2026-03-05 deviation report. All design decisions resolved via established patterns and best judgment.

## Requirements *(mandatory)*

### Functional Requirements

**Process Lifecycle:**

- **FR-001**: System MUST provide a single binary entry point that bootstraps the full framework stack in deterministic order: configuration loading, external service connections, supervision tree initialization, agent spawning, health endpoint activation.
- **FR-002**: System MUST handle SIGTERM and SIGINT signals, initiating graceful shutdown that drains in-flight messages, stops agents in reverse start order, flushes pending state, and exits with code 0.
- **FR-003**: System MUST support a configurable startup timeout after which it reports failure and exits non-zero if the "ready" state is not reached.
- **FR-004**: System MUST support a forced shutdown path (second signal during graceful shutdown) that terminates immediately.
- **FR-005**: System MUST validate configuration completeness and external service reachability at startup before spawning agents.

**Observability:**

- **FR-006**: System MUST emit distributed traces for all agent-to-agent message exchanges, with trace context propagated across messaging boundaries.
- **FR-007**: System MUST expose a metrics endpoint providing counters, gauges, and histograms for: messages sent/received, task completions/failures, agent restarts, active agent count (by type), and message queue depth.
- **FR-008**: System MUST produce structured log entries that include trace ID, span ID, timestamp, severity level, component identifier, and structured key-value metadata.
- **FR-009**: System MUST propagate trace context through the messaging layer so that a single distributed trace can span multiple agents participating in a workflow.
- **FR-010**: System MUST handle observability collector unavailability gracefully — telemetry MUST buffer locally and MUST NOT block framework operations.

**Cross-Phase Integration:**

- **FR-011**: System MUST drain security audit events from the in-memory audit logger to the persistent audit repository on a configurable interval, with zero event loss under normal operation.
- **FR-012**: System MUST authenticate agent operations through the JWT-based authentication system, rejecting unauthenticated or unauthorized requests.
- **FR-013**: System MUST wire agent heartbeats to the health monitoring subsystem such that heartbeat absence triggers the phi accrual failure detector and surfaces to the supervision tree.
- **FR-014**: System MUST record supervision events (agent failures, restarts, escalations) in both the metrics pipeline and the audit log.

**Deployment:**

- **FR-015**: System MUST support containerized deployment with a multi-stage build producing a minimal runtime image.
- **FR-016**: System MUST expose separate liveness and readiness health check endpoints for orchestrator probes.
- **FR-017**: System MUST support configuration via environment variables that override file-based configuration values.
- **FR-018**: System MUST support horizontal scaling where multiple replicas connect to shared messaging and persistence without work duplication.

**Dashboards and Alerting:**

- **FR-019**: System MUST provide exportable dashboard definitions covering agent health, message throughput, task status, supervision tree state, and resource utilization.
- **FR-020**: System MUST provide configurable alert rule definitions for critical operational thresholds (failure rate spikes, queue depth growth, heartbeat loss).

### Key Entities

- **Process**: The running framework instance. Has a lifecycle (starting → ready → draining → stopped), a configuration, and a set of managed agents.
- **Trace**: A distributed operation spanning one or more agents. Contains spans with parent-child relationships, timing, and metadata.
- **Metric**: A named measurement with labels. Types include counters (monotonically increasing), gauges (point-in-time values), and histograms (distribution of values).
- **Health Probe**: An endpoint that reports component readiness. Liveness confirms the process is alive; readiness confirms all dependencies are connected.
- **Dashboard**: A collection of visualizations connected to metrics. Includes panels, alert rules, and threshold definitions.
- **Alert Rule**: A condition definition (metric threshold + duration) that triggers a notification when breached.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: System reaches "ready" state from cold start in under 10 seconds with all external services available.
- **SC-002**: Graceful shutdown completes (all agents stopped, state flushed, messages drained) in under 30 seconds under normal load.
- **SC-003**: 100% of inter-agent message exchanges produce a corresponding distributed trace with correct parent-child relationships.
- **SC-004**: Metrics endpoint responds within 500ms and reports all required counters, gauges, and histograms.
- **SC-005**: Zero audit events lost during normal operation — every event recorded by the in-memory logger appears in the persistent repository.
- **SC-006**: Container image size is under 100MB compressed and starts in under 5 seconds.
- **SC-007**: All dashboard panels render correctly with live data when connected to a running system's metrics endpoint.
- **SC-008**: Unauthorized agent operations are rejected 100% of the time when security is enabled.
- **SC-009**: Heartbeat absence is detected and surfaced to the supervision tree within 2x the configured heartbeat interval.
- **SC-010**: System operates normally when the observability collector is temporarily unreachable (no blocked operations, telemetry buffers and resumes).

## Assumptions

- External services (NATS, PostgreSQL) are provisioned and reachable before the framework process starts. The framework does not manage external service lifecycle.
- The target container orchestration platform supports standard health check probes (HTTP GET) and environment variable injection.
- Dashboard definitions target widely-adopted visualization tooling compatible with the metrics format exposed by the framework.
- The Phase 9 LLM provider integration is developed in parallel and does not block Phase 8. Phase 8 provides the operational foundation that Phase 9 builds upon.
- Alert notification routing (email, Slack, PagerDuty) is handled by external alerting infrastructure. The framework provides the alert rule definitions but not the notification delivery mechanism.

## Dependencies

- **Phase 1-7 (all complete)**: Phase 8 orchestrates and wires together all prior phases.
- **Phase 2 (monitoring)**: HealthMonitor, MetricsCollector, phi accrual failure detector — consumed by US2 and US3.
- **Phase 3 (supervision)**: SupervisedSystem, restart strategies — consumed by US1 and US3.
- **Phase 4 (transport)**: NATS transport, message envelope with trace context — consumed by US2.
- **Phase 5 (security)**: AuditLogger, JwtManager, PolicyEngine — consumed by US3.
- **Phase 6 (persistence)**: AuditRepository, AuditPersister — consumed by US3.
- **Phase 7 (agents)**: AgentRuntime, agent roles, heartbeat emitter — consumed by US1 and US3.
