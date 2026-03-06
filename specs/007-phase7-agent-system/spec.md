# Feature Specification: Phase 7 — Agent System

**Feature Branch**: `007-phase7-agent-system`
**Created**: 2026-03-05
**Status**: Draft
**Input**: Phase 7: Agent System — Multi-agent orchestration with supervision trees, lifecycle management, agent-to-agent communication, task scheduling, and tool integration.

## User Scenarios & Testing *(mandatory)*

### User Story 1 — Agent Lifecycle Management (Priority: P0)

A framework operator spawns an agent by type (e.g., Worker, Coordinator) with a configuration. The agent initializes, registers with the system, begins accepting messages, and can be stopped, paused, or restarted. On failure, the supervision tree restarts the agent according to its configured strategy without operator intervention.

**Why this priority**: Without lifecycle management, no agent can exist in the system. Every other capability depends on agents being alive, discoverable, and restartable.

**Independent Test**: Spawn an agent, verify it transitions through Initializing → Running states, send it a stop command, verify it transitions through Stopping → Terminated, then restart it via supervision and verify it returns to Running with restored state.

**Acceptance Scenarios**:

1. **Given** a valid agent configuration, **When** the operator requests agent creation, **Then** the agent transitions through Initializing → Running and becomes discoverable by other agents within a bounded startup time.
2. **Given** a running agent, **When** the agent encounters an unrecoverable error, **Then** the supervision tree detects the failure, applies the configured restart strategy (OneForOne, OneForAll, RestForOne), and the agent resumes operation with its last persisted state.
3. **Given** a running agent, **When** the operator sends a stop command, **Then** the agent drains its in-flight messages, persists its state, transitions through Stopping → Terminated, and deregisters from the system registry.
4. **Given** an agent in any state, **When** the system queries its status, **Then** the response includes current state, health level, uptime, last heartbeat time, and restart count.

---

### User Story 2 — Inter-Agent Communication (Priority: P0)

Agents communicate through structured messages over NATS subjects. A Coordinator agent sends a task assignment to a Worker agent, the Worker processes it and replies with a result. Messages are correlated, prioritized, and durably delivered via JetStream when guaranteed delivery is required.

**Why this priority**: Communication is the second foundational capability. Without it, agents are isolated and cannot coordinate.

**Independent Test**: Spawn two agents (Coordinator and Worker). Coordinator publishes a task assignment. Worker receives, processes, and replies. Verify the Coordinator receives the reply with correct correlation ID. Repeat with durable delivery and verify message survives a simulated transport interruption.

**Acceptance Scenarios**:

1. **Given** a Coordinator and Worker agent, **When** the Coordinator sends a task assignment to the Worker's command subject, **Then** the Worker receives the message within a bounded latency, and the Coordinator can track delivery via correlation ID.
2. **Given** a message requiring guaranteed delivery, **When** the sender publishes via durable transport, **Then** the message is persisted in JetStream and the receiver explicitly acknowledges processing, with redelivery on failure.
3. **Given** multiple messages to the same agent, **When** messages arrive with different priority levels, **Then** higher-priority messages are processed before lower-priority ones.
4. **Given** a conversation spanning multiple exchanges, **When** any party queries by correlation ID, **Then** the full message thread is returned in chronological order.

---

### User Story 3 — Team Orchestration and Task Decomposition (Priority: P1)

A Coordinator agent receives a complex task, decomposes it into subtasks, assembles a team of specialized agents (Workers, Planners, Executors), assigns subtasks to team members, monitors progress, and aggregates results. If a team member fails, the supervisor restarts it and the Coordinator reassigns the incomplete subtask.

**Why this priority**: Orchestration is the primary value proposition of the framework. It composes lifecycle and communication into productive multi-agent workflows.

**Independent Test**: Submit a multi-step task to a Coordinator. Verify it decomposes the task, creates a Worker team, distributes subtasks, collects results, and returns an aggregated result. Inject a Worker failure mid-execution and verify recovery through reassignment.

**Acceptance Scenarios**:

1. **Given** a complex task submitted to a Coordinator, **When** the Coordinator plans execution, **Then** the task is decomposed into independent subtasks with defined dependencies, inputs, and expected outputs.
2. **Given** a set of subtasks, **When** the Coordinator assembles a team, **Then** agents are spawned or allocated by type, registered as a team under a shared supervisor, and assigned subtasks matching their capabilities.
3. **Given** an active team executing subtasks, **When** a Worker agent fails, **Then** the supervisor restarts the Worker, the Coordinator detects the incomplete subtask, and reassigns it — with no duplicate processing of already-completed subtasks.
4. **Given** all subtasks completed, **When** the Coordinator aggregates results, **Then** the final result combines all subtask outputs in dependency order and is returned to the original requester.
5. **Given** a subtask that exceeds its deadline, **When** the timeout fires, **Then** the Coordinator marks the subtask as timed out, optionally retries with a different agent or strategy, and continues aggregation with partial results if configured to do so.

---

### User Story 4 — Tool System and Agent Composition (Priority: P2)

Any agent can be registered as a tool, making its capabilities available to other agents through a central registry. An agent requests a tool by name, the system checks permissions, invokes the target agent, and returns the result. MCP-compatible tools are also accessible through the same interface.

**Why this priority**: The tool system enables hierarchical agent composition — agents using other agents as tools. This multiplies the framework's capabilities but depends on working agents, communication, and security.

**Independent Test**: Register a Worker agent as a tool. Have a Coordinator discover and invoke the tool. Verify permission checks occur, the invocation succeeds, and metrics are recorded. Repeat with an MCP-hosted tool and verify the same interface works.

**Acceptance Scenarios**:

1. **Given** an agent registered as a tool, **When** another agent queries the tool registry, **Then** the tool is discoverable with its name, description, input/output schema, and required permissions.
2. **Given** an agent with `execute:tool:{namespace}` permission, **When** the agent invokes a tool, **Then** the permission is validated, the tool agent receives the invocation, executes it, and returns the result within a configurable timeout.
3. **Given** an agent without sufficient tool permissions, **When** the agent attempts to invoke a tool, **Then** the invocation is rejected with a clear authorization error and the attempt is recorded in the audit log.
4. **Given** an MCP-compatible external tool, **When** an agent invokes it through the tool registry, **Then** the MCP bridge translates the request, invokes the external tool, and returns the result through the same interface as native agent tools.
5. **Given** a tool invocation in progress, **When** the invocation exceeds the configured timeout, **Then** the system cancels the invocation, returns a timeout error to the caller, and the tool agent's state is not corrupted.

---

### User Story 5 — Specialized Agent Roles (Priority: P2)

The framework provides nine specialized agent roles, each with domain-specific behavior built on the common agent infrastructure. A Supervisor manages child agents, a Worker executes tasks, a Coordinator orchestrates teams, a Monitor watches system health, a Planner generates execution plans, an Executor carries out planned steps, a Critic evaluates outputs, a Router directs messages, and a Memory agent manages shared knowledge.

**Why this priority**: Specialized roles give concrete implementations of the generic agent infrastructure. They depend on all prior user stories being functional.

**Independent Test**: For each agent role, spawn an instance, verify it accepts role-appropriate messages, performs its specialized behavior, and interoperates with other roles in a team scenario.

**Acceptance Scenarios**:

1. **Given** a Supervisor agent, **When** child agents are registered under it, **Then** the Supervisor monitors their health, applies restart strategies on failure, and escalates to its own supervisor if recovery fails.
2. **Given** a Worker agent with a task assignment, **When** the Worker completes the task, **Then** the result is published to the task result subject with timing metadata and the Worker returns to an idle state ready for the next assignment.
3. **Given** a Monitor agent, **When** system health degrades beyond a threshold, **Then** the Monitor detects the degradation through health checks, generates an alert, and publishes it to the system alerts subject.
4. **Given** a Planner and Executor pair, **When** the Planner receives a high-level goal, **Then** the Planner produces a step-by-step execution plan, and the Executor carries out each step in sequence, reporting progress on each step completion.
5. **Given** a Router agent receiving messages, **When** messages arrive on a routing subject, **Then** the Router inspects message content and priority, and forwards each message to the appropriate destination agent or subject based on configured routing rules.
6. **Given** a Memory agent, **When** agents request shared context, **Then** the Memory agent retrieves relevant knowledge from its store and returns it, supporting both key-value lookups and semantic search patterns.

---

### User Story 6 — Agent Discovery and Registry (Priority: P1)

The system maintains a registry of all active agents with their types, capabilities, health status, and subjects. Agents can discover each other by type, capability, or availability. The registry is kept current through heartbeats and lifecycle events.

**Why this priority**: Discovery enables dynamic team composition and routing decisions. Without it, agents can only communicate with hardcoded subjects.

**Independent Test**: Spawn several agents of different types. Query the registry by type, by capability, and by health status. Verify results are current and update when agents join, leave, or change state.

**Acceptance Scenarios**:

1. **Given** multiple running agents, **When** an agent queries the registry by type, **Then** all agents of that type are returned with their current status, capabilities, and communication subjects.
2. **Given** a running agent, **When** the agent stops sending heartbeats beyond the configured timeout, **Then** the registry marks the agent as unhealthy and eventually deregisters it.
3. **Given** a new agent starting up, **When** the agent completes initialization, **Then** it automatically registers with the registry including its type, capabilities, and command subject.
4. **Given** a Coordinator assembling a team, **When** the Coordinator queries for available Workers with specific capabilities, **Then** the registry returns only healthy, non-busy agents matching the capability filter.

---

## Functional Requirements *(mandatory)*

### FR-1: Agent Lifecycle State Machine

- Agents follow a defined state machine: Initializing → Running → (Paused | Stopping) → Terminated, with Error as a reachable state from any non-terminal state.
- State transitions are atomic and produce events on the agent's status subject.
- Agents persist their state on each significant transition for crash recovery.
- The supervision tree receives failure notifications and applies restart strategies.
- Each agent has a unique `AgentId` and is assigned an `AgentType`.

### FR-2: Agent Registry

- A centralized (or distributed) agent registry tracks all active agents.
- Registry entries include: AgentId, AgentType, capabilities list, health status, command subject, heartbeat timestamp, and metadata.
- Agents register on startup and deregister on shutdown.
- Heartbeat-based liveness detection with configurable timeout and phi accrual failure detection.
- Registry supports queries by type, capability, health status, and availability.

### FR-3: Inter-Agent Messaging

- Agents communicate through NATS subjects using the `taxonomy.v1` subject patterns.
- Fire-and-forget messaging (via `Transport` trait) for heartbeats, status updates, and events.
- Durable messaging (via `DurableTransport` trait) for task assignments, results, and critical coordination.
- Request-reply pattern for synchronous-style interactions with timeout.
- Messages carry `correlation_id` for conversation threading.
- Priority-aware message processing within agent mailboxes.

### FR-4: Task Scheduling and Assignment

- Tasks are submitted to the system with a type, priority, deadline, and input payload.
- Schedulers match tasks to available agents based on type, capabilities, and current load.
- Task assignment uses durable messaging to guarantee delivery.
- Tasks track state: Pending → Assigned → Running → (Completed | Failed | TimedOut).
- Deadline monitoring with configurable timeout actions (retry, reassign, fail).
- Idempotent task processing using `message_id` deduplication.

### FR-5: Team Orchestration

- Coordinators create teams by spawning or allocating agents under a shared supervisor.
- Team patterns supported: supervisor-worker (fan-out/fan-in), pipeline (sequential handoff), and consensus (parallel evaluation with voting).
- Task decomposition produces a dependency graph of subtasks.
- Result aggregation collects subtask outputs and combines them in dependency order.
- Team lifecycle is bound to the orchestrating task — teams are disbanded when the task completes.

### FR-6: Tool Registry and Invocation

- Any agent can register as a tool with a name, description, input/output schema, and namespace.
- Tool discovery is filtered by the caller's `discover:tool:{namespace}` permission.
- Tool invocation requires `execute:tool:{namespace}` permission (validated by PolicyEngine).
- Invocations are proxied through the tool bus: caller → tool bus → target agent → response.
- MCP-compatible tools (via `mister-smith-mcp`) are accessible through the same tool bus interface.
- All invocations produce audit events and are tracked with metrics (latency, success rate).

### FR-7: Health Monitoring and Heartbeats

- Every agent emits heartbeats at a configurable interval on `agents.{id}.heartbeat`.
- The monitoring system uses phi accrual failure detection (from Phase 2 `HealthMonitor`) to assess agent liveness.
- Health levels: Healthy, Degraded, Unhealthy, Critical — derived from heartbeat regularity, error rates, and custom health checks.
- Health status changes produce events on `agents.{id}.status`.
- The agent registry consumes health events to maintain current agent status.

### FR-8: Specialized Agent Implementations

- Nine agent roles implemented as concrete types over the common `Agent` trait:
  - **Supervisor**: Wraps Phase 3 `SupervisedSystem`, manages child agent lifecycle.
  - **Worker**: General-purpose task executor with configurable task handlers.
  - **Coordinator**: Orchestrates teams, decomposes tasks, aggregates results.
  - **Monitor**: Subscribes to health and system events, generates alerts.
  - **Planner**: Accepts goals, produces step-by-step execution plans.
  - **Executor**: Carries out execution plan steps in sequence.
  - **Critic**: Evaluates outputs against criteria, provides scored feedback.
  - **Router**: Content-based message routing with configurable rules.
  - **Memory**: Shared knowledge store with key-value and contextual retrieval.
- Each role defines its accepted message types, command subject patterns, and capability declarations.

### FR-9: Agent Configuration

- Agents are configured through the existing `mister-smith-config` system with agent-specific sections.
- Configuration includes: agent type, restart policy, heartbeat interval, mailbox capacity, tool permissions, and role-specific settings.
- Configuration supports runtime updates for non-structural parameters (e.g., heartbeat interval, log level) without agent restart.
- Default configurations exist for each agent role with sensible production defaults.

---

## Non-Functional Requirements

### Performance

- Agent spawn time under 50ms for a configured agent (excluding state recovery from persistence).
- Inter-agent message latency under 5ms for fire-and-forget NATS messaging (local server).
- Task assignment and acknowledgment round-trip under 20ms.
- Registry queries return results within 10ms for up to 1,000 registered agents.
- Heartbeat overhead less than 1% of an agent's processing capacity.

### Scalability

- Support at least 500 concurrent agents per node without degradation.
- Team sizes up to 50 agents per Coordinator.
- Task throughput of at least 1,000 assignments per second per scheduler.
- Registry supports at least 10,000 agent entries across a clustered deployment.

### Reliability

- Agent failures are detected within 2 heartbeat intervals.
- Restart completes within 3 seconds including state recovery.
- No message loss for durable-transport messages (at-least-once delivery guaranteed).
- Idempotent task processing prevents duplicate execution on redelivery.

### Security

- All agent-to-agent communication authenticated via JWT tokens (Phase 5).
- Tool invocations authorized through RBAC PolicyEngine.
- All tool invocations and agent lifecycle events produce audit log entries.
- Agents operate under the principle of least privilege — default permissions are minimal.

---

## Success Criteria *(mandatory)*

1. **End-to-end orchestration**: A Coordinator can decompose a task into subtasks, assign them to a team of Workers, collect results, and return an aggregated answer — all through NATS messaging.
2. **Fault tolerance**: When a Worker in an active team fails, the Supervisor restarts it, the Coordinator reassigns the incomplete subtask, and the final result is correct with no duplicate work.
3. **Agent discovery**: Agents can discover each other by type and capability, enabling dynamic team composition without hardcoded routing.
4. **Tool composition**: An agent can invoke another agent as a tool, with permissions checked and audit logged, and receive the result through the same interface used for MCP tools.
5. **Lifecycle observability**: An operator can query any agent's current state, health, uptime, restart history, and message throughput at any time.
6. **Scalable teams**: A Coordinator can orchestrate a team of 20+ Workers processing concurrent subtasks with correct dependency ordering and result aggregation.
7. **Configurable behavior**: All agent timing (heartbeat interval, task timeout, restart delay), capacity (mailbox size, team size), and security (permissions, role mappings) settings are configurable without code changes.

---

## Key Entities

### Agent

The central entity. An autonomous processing unit with identity, type, lifecycle state, capabilities, and communication endpoints. Wraps an actor from Phase 3 with orchestration-aware behavior.

### Team

A group of agents assembled by a Coordinator to accomplish a task. Has a shared supervisor, a lifecycle bound to the orchestrating task, and a defined team pattern (supervisor-worker, pipeline, consensus).

### Task

A unit of work with type, priority, deadline, input payload, and output. Tracks state through its lifecycle (Pending → Assigned → Running → Completed/Failed/TimedOut). Linked to an owning agent and optionally to parent/child tasks for decomposition.

### Tool

An agent capability exposed to other agents through the tool registry. Has a name, namespace, input/output schema, and required permissions. Can be backed by a native agent or an MCP-compatible external service.

### AgentRegistry

The system-wide directory of active agents. Maintains agent metadata, health status, capabilities, and communication subjects. Updated through lifecycle events and heartbeats.

---

## Scope and Boundaries *(mandatory)*

### In Scope

- Agent lifecycle state machine and supervision integration
- Agent registry with heartbeat-based liveness detection
- Inter-agent communication over NATS (fire-and-forget + durable)
- Task scheduling, assignment, and deadline monitoring
- Team orchestration with fan-out/fan-in, pipeline, and consensus patterns
- Tool registry with RBAC permission checking and MCP bridge
- Nine specialized agent role implementations
- Agent configuration through existing config system
- Health monitoring integration with phi accrual failure detection

### Out of Scope

- LLM/model integration — agents are model-agnostic containers; LLM binding is application-layer concern
- New transport protocols — uses existing NATS, HTTP, gRPC from Phase 4
- New persistence schemas — uses existing PostgreSQL and JetStream KV from Phase 6
- Production deployment tooling — deferred to Phase 8
- Distributed tracing and observability instrumentation — deferred to Phase 8
- Performance benchmarking — deferred to Phase 8
- UI or CLI for agent management — future work

---

## Dependencies and Assumptions

### Dependencies

| Phase | Component | What Phase 7 Uses |
|-------|-----------|-------------------|
| 1 | `mister-smith-core` | `AgentId`, `AgentType`, `AgentState`, error types |
| 2 | `mister-smith-runtime` | `RuntimeManager` for Tokio task spawning |
| 2 | `mister-smith-monitoring` | `HealthMonitor`, phi accrual failure detector |
| 2 | `mister-smith-events` | `EventBus` for lifecycle and system events |
| 3 | `mister-smith-actor` | `ActorCell`, `ActorRef`, mailbox, lifecycle management |
| 3 | `mister-smith-supervision` | `SupervisedSystem`, restart strategies |
| 4 | `mister-smith-transport` | `Transport`, `DurableTransport`, `MessageEnvelope` |
| 4 | `mister-smith-nats` | `NatsTransport`, JetStream integration |
| 4 | `mister-smith-mcp` | MCP client/server, tool registry bridge |
| 5 | `mister-smith-security` | `PolicyEngine`, `JwtManager`, `AuditLogger` |
| 6 | `mister-smith-persistence` | State persistence, task/message repositories |

### Assumptions

- NATS server is available and JetStream is enabled in the deployment environment.
- PostgreSQL is provisioned with the Phase 6 schema (migrations 00001-00005 applied).
- Phase 3 actor system's mailbox and supervision primitives are stable and tested.
- The `taxonomy.v1` subject patterns are frozen and will not change during Phase 7 development.
- Agent configurations are loaded at startup; runtime config changes are limited to non-structural parameters.
- The tool permission model (`execute:tool:{namespace}`, `discover:tool:{namespace}`) is defined in the RBAC policy store before tool operations are attempted.
- Each node in a distributed deployment runs its own agent registry; cross-node discovery uses NATS subject subscriptions, not a centralized database.

---

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Actor mailbox backpressure under high message volume | Medium | High | Bounded mailboxes with configurable overflow policy; priority-aware processing |
| Agent state recovery latency on large state payloads | Medium | Medium | Incremental state snapshots; lazy loading of non-critical state |
| Team coordination complexity leading to deadlocks | Low | High | Timeout-based deadlock detection; Coordinator monitors subtask progress |
| Subject namespace collisions between teams | Low | Medium | Team-scoped subject prefixes; registry validates uniqueness |
| Tool invocation latency hiding performance issues | Medium | Medium | Configurable timeouts per tool; circuit breaker on repeated failures |
| Supervision cascade (one failure triggers mass restarts) | Low | High | Rate-limited restart strategies; escalation thresholds before cascade |
