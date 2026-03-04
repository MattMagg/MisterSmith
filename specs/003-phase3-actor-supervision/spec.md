# Feature Specification: Phase 3 — Actor System & Supervision Trees

**Feature Branch**: `003-phase3-actor-supervision`
**Created**: 2026-03-04
**Status**: Draft
**Input**: ROADMAP.md Phase 3, spec/core-architecture/async-patterns.md, supervision-trees.md, supervision-and-events.md, type-definitions.md, component-architecture.md

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Spawn and Communicate with Actors (Priority: P1)

A framework developer creates actors that run concurrently, send messages to each other, and receive responses. Each actor has isolated state, a unique identity, and processes messages sequentially from its mailbox. This is the foundational concurrency primitive — without it, nothing else in the framework works.

**Why this priority**: Actors are the building block for all higher-level constructs (supervision, agents, orchestration). Every downstream phase depends on being able to spawn actors and exchange messages.

**Independent Test**: Spawn two actors, send a message from one to the other using tell (fire-and-forget), verify receipt. Then send a message using ask (request-response) and verify the reply. All within a single `#[tokio::test]`.

**Acceptance Scenarios**:

1. **Given** an ActorSystem is running, **When** a developer spawns an actor with a message handler, **Then** the actor receives a unique ActorId and transitions to Running state.
2. **Given** two running actors A and B, **When** A sends a tell message to B's ActorRef, **Then** B's `handle_message` is invoked with that message and A does not block.
3. **Given** two running actors A and B, **When** A sends an ask message to B's ActorRef, **Then** A receives B's response within the configured timeout.
4. **Given** a running actor with a bounded mailbox (capacity N), **When** N+1 messages are sent before any are processed, **Then** the sender receives a mailbox-full error.
5. **Given** a running actor, **When** the ActorSystem initiates shutdown, **Then** the actor's `post_stop` hook is called and the actor transitions to Terminated.

---

### User Story 2 - Supervise Actors with Restart Policies (Priority: P1)

A framework developer organizes actors into supervision trees where parent supervisors monitor child actors and apply restart policies when failures occur. The three restart policies (OneForOne, OneForAll, RestForOne) determine which siblings are affected when one child fails. This is the fault-tolerance mechanism that distinguishes the framework.

**Why this priority**: Supervision is the framework's core architectural commitment (Constitution Principle V). Without supervision trees, actor failures are unrecoverable and the framework cannot deliver on its fault-tolerance promise.

**Independent Test**: Create a supervisor with three child actors. Kill one child. Verify the supervisor's restart policy is applied correctly — for OneForOne, only the failed child restarts; for OneForAll, all children restart; for RestForOne, the failed child and younger siblings restart.

**Acceptance Scenarios**:

1. **Given** a supervisor with OneForOne policy and three children A, B, C, **When** B fails, **Then** only B is restarted; A and C continue running undisturbed.
2. **Given** a supervisor with OneForAll policy and three children A, B, C, **When** B fails, **Then** all three children are stopped and restarted.
3. **Given** a supervisor with RestForOne policy and children A, B, C (started in that order), **When** B fails, **Then** B and C are restarted; A continues running.
4. **Given** a supervisor with max_restarts=3 and time_window=60s, **When** a child fails 4 times within 60 seconds, **Then** the supervisor escalates the failure to its parent supervisor.
5. **Given** a child actor with RestartScope::Temporary, **When** that child fails, **Then** the supervisor does NOT restart it (regardless of restart policy).
6. **Given** a child actor with RestartScope::Transient, **When** that child terminates normally (returns Stop), **Then** the supervisor does NOT restart it. **When** that child fails with an error, **Then** the supervisor DOES restart it.
7. **Given** a restarted actor, **When** it completes restart, **Then** it has a fresh state instance, the same ActorId, and its `pre_start` hook has been called.

---

### User Story 3 - Compose Hierarchical Supervision Trees (Priority: P2)

A framework developer builds multi-level supervision trees where supervisors can themselves be supervised. Failures escalate up the tree when a supervisor exhausts its restart budget. The tree can be traversed, queried for health status, and shut down gracefully from the root.

**Why this priority**: Hierarchical composition is essential for real-world systems where different subsystems have different fault-tolerance requirements, but it builds on top of the basic supervision mechanics from US2.

**Independent Test**: Create a three-level tree (root → mid-level supervisors → worker actors). Trigger a failure cascade that escalates from a worker through a mid-level supervisor to the root. Verify each level applies its own policy before escalating.

**Acceptance Scenarios**:

1. **Given** a root supervisor R with child supervisor S, and S has child worker W, **When** W fails repeatedly and S exhausts its restart budget, **Then** S escalates the failure to R, and R applies its own restart policy to S (which triggers S to restart all its children).
2. **Given** a supervision tree with 10+ nodes, **When** the developer queries the tree, **Then** the tree returns the status of every node (Running, Failed, Restarting, etc.).
3. **Given** a running supervision tree, **When** the root initiates graceful shutdown, **Then** all nodes are stopped in reverse-start order (leaves first, root last) with `post_stop` hooks called.
4. **Given** a supervision tree, **When** an actor is restarting, **Then** a system event is emitted containing the actor's ID, failure reason, restart count, and correlation ID.

---

### User Story 4 - Integrate Actors with Event System and Monitoring (Priority: P2)

Actor lifecycle transitions and supervision events integrate with the existing Phase 2 EventBus and monitoring infrastructure. Health checks can probe actor system health. Metrics track actor counts, message throughput, mailbox depth, and failure rates.

**Why this priority**: Observability is required for production use but depends on the actor system (US1-US3) existing first.

**Independent Test**: Spawn actors, subscribe to lifecycle events on the EventBus, trigger a failure and restart, verify the expected events were published with correct metadata.

**Acceptance Scenarios**:

1. **Given** an ActorSystem with an EventBus attached, **When** an actor is spawned, **Then** an `AgentEventType::Created` event is published with the actor's ID and type.
2. **Given** an ActorSystem with an EventBus attached, **When** an actor fails and is restarted by its supervisor, **Then** `AgentEventType::Failed` and `AgentEventType::Started` events are published with matching correlation IDs.
3. **Given** an ActorSystem integrated with the HealthMonitor, **When** a health check runs, **Then** it reports the count of active actors, actors in error state, and supervision tree depth.
4. **Given** a running actor system, **When** metrics are collected, **Then** they include messages processed, mailbox depths, restart counts, and actor lifecycle durations.

---

### Edge Cases

- What happens when a supervisor itself panics during a restart operation? The parent supervisor handles it via escalation, same as any other failure.
- What happens when an ask message times out? The caller receives a timeout error; the actor continues processing (the response is discarded if it arrives late).
- What happens when an actor's `pre_start` hook fails during restart? The actor transitions to Error state and the failure is reported to its supervisor as a restart failure.
- What happens when the root supervisor exhausts its restart budget? The entire supervision tree initiates graceful shutdown and the ActorSystem reports a fatal error.
- What happens when a message is sent to a terminated actor's ActorRef? The sender receives an actor-not-found error; the message is not delivered.
- What happens during shutdown when actors have pending ask responses? Pending oneshot channels are dropped, causing callers to receive a channel-closed error.
- What happens when two actors deadlock by sending ask messages to each other simultaneously? The timeout mechanism prevents permanent deadlock — both asks eventually time out.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST provide an Actor trait with `handle_message`, `pre_start`, `post_stop`, and `actor_id` methods, where the message type and state type are associated types.
- **FR-002**: The system MUST provide a Mailbox with FIFO message ordering and optional bounded capacity.
- **FR-003**: The system MUST provide ActorRef as a typed handle supporting both tell (fire-and-forget) and ask (request-response with timeout) communication patterns.
- **FR-004**: The system MUST provide an ActorSystem that spawns actors, tracks their handles, and coordinates graceful shutdown.
- **FR-005**: The system MUST provide a SupervisionTree with hierarchical supervisor-child relationships and per-node restart policies.
- **FR-006**: The system MUST implement three restart policies: OneForOne (restart failed child only), OneForAll (restart all children), and RestForOne (restart failed child and younger siblings).
- **FR-007**: The system MUST implement three restart scopes: Permanent (always restart), Transient (restart only on abnormal termination), and Temporary (never restart).
- **FR-008**: The system MUST escalate failures to the parent supervisor when a child exceeds its max_restarts within its time_window.
- **FR-009**: The system MUST apply exponential backoff between restart attempts using the configurable backoff_multiplier.
- **FR-010**: The system MUST preserve ActorId across restarts while providing a fresh state instance.
- **FR-011**: The system MUST reject messages to a bounded mailbox that is at capacity, returning an appropriate error.
- **FR-012**: The system MUST emit lifecycle events (Created, Started, Failed, Stopped, StateChanged) to the EventBus for every actor state transition.
- **FR-013**: The system MUST include correlation_id and causation_id in supervision events to enable distributed tracing across restart boundaries.
- **FR-014**: The system MUST shut down supervision trees in reverse-start order (leaves first, root last), calling post_stop on every actor.
- **FR-015**: The system MUST integrate with the existing Phase 2 HealthMonitor to report actor system health status.
- **FR-016**: The system MUST track metrics for actor counts (by state), messages processed, mailbox depths, restart counts, and failure rates.
- **FR-017**: Actor state MUST map to the canonical AgentState enum (Initializing, Running, Paused, Stopping, Terminated, Error, Restarting) defined in Phase 1.
- **FR-018**: All inter-actor communication MUST occur exclusively through message passing — no shared mutable state between actors.

### Key Entities

- **Actor**: An isolated concurrent unit with private state, a unique identity (ActorId), and a message handler. Processes messages sequentially from its Mailbox.
- **ActorRef**: A lightweight, cloneable handle to an actor that supports tell and ask communication patterns. The only way to interact with an actor from outside.
- **Mailbox**: A bounded or unbounded FIFO queue that buffers incoming messages for an actor. Supports backpressure when bounded.
- **ActorSystem**: The registry and lifecycle manager for all actors. Spawns actors, tracks their references, coordinates shutdown.
- **SupervisionTree**: A hierarchical structure of supervisor and worker nodes. Each supervisor monitors its children and applies restart policies on failure.
- **SupervisorNode**: A node in the supervision tree with an identity, parent reference, children list, restart policy, and restart history.
- **RestartPolicy**: Determines which siblings are affected when a child fails (OneForOne, OneForAll, RestForOne).
- **RestartScope**: Determines whether a specific child should be restarted at all (Permanent, Transient, Temporary).
- **SupervisionStrategy**: Combines RestartPolicy, max_failures, failure_window, escalation_policy, and backoff_strategy into a complete supervision configuration.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer can spawn 1,000 actors, each processing messages independently, within the ActorSystem — demonstrating that the actor model scales without degradation.
- **SC-002**: When an actor fails, the supervisor's configured restart policy is applied and the actor resumes processing within the configured backoff delay — no manual intervention required.
- **SC-003**: Failure escalation propagates through a 3+ level supervision tree correctly, with each level applying its own restart policy before escalating.
- **SC-004**: All actor lifecycle transitions produce corresponding events on the EventBus with correct correlation IDs, enabling full reconstruction of failure-restart chains.
- **SC-005**: Ask (request-response) messages receive replies or timeout errors within the configured timeout — no permanent deadlocks or lost responses.
- **SC-006**: Graceful shutdown of a supervision tree with 100+ actors completes with all post_stop hooks called and no message loss for in-flight tell messages.
- **SC-007**: The actor system passes all Gate 3 criteria defined in ROADMAP.md: actors spawn, communicate via mailboxes, are supervised, and supervision trees compose hierarchically.

## Assumptions

- The canonical types RestartPolicy, RestartScope, and SupervisionStrategy are defined in `spec/core-architecture/type-definitions.md` and will be implemented in the `mister-smith-core` crate if not already present.
- Phase 2 crates (runtime, monitoring, events, async, resources) are complete and available as dependencies.
- The ActorId type aliases to AgentId (UUID-based newtype) from the core crate.
- MessagePack wire format for messages is deferred to Phase 4 (Transport). Phase 3 actors communicate in-process only.
- The actor system does not need location transparency (remote actors) in Phase 3 — that is a Phase 4/7 concern.
- Mailbox implementation uses Tokio channels (mpsc for tell, oneshot for ask) — this is an implementation detail but noted as an assumption since the spec references these patterns.

## Dependencies

- **Phase 1 (Complete)**: `mister-smith-core` (AgentId, AgentState, SystemError, traits), `mister-smith-config` (SupervisionConfig)
- **Phase 2 (Complete)**: `mister-smith-runtime` (Tokio runtime), `mister-smith-events` (EventBus, SystemEvent), `mister-smith-monitoring` (HealthMonitor, metrics), `mister-smith-async` (TaskExecutor, CircuitBreaker)
- **Produces**: `mister-smith-actor` crate (3.1), `mister-smith-supervision` crate (3.2)
