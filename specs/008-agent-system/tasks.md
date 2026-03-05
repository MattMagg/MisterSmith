# Tasks: Phase 7 — Agent System

**Input**: Design documents from `/specs/008-agent-system/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, data-model.md, contracts/

**Tests**: Tests are included as they are integral to the phase-gated build process. Each user story includes unit tests. Gate 7 integration test validates end-to-end orchestration.

**Organization**: Tasks grouped by user story in priority order.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the `mister-smith-agents` crate skeleton and wire workspace dependencies.

- [x] T001 Create crate directory structure at `crates/mister-smith-agents/` per plan.md layout (src/, src/roles/, tests/)
- [x] T002 Create `crates/mister-smith-agents/Cargo.toml` with workspace dependencies: mister-smith-core, mister-smith-actor, mister-smith-supervision, mister-smith-transport, mister-smith-nats, mister-smith-mcp, mister-smith-security, mister-smith-persistence, mister-smith-events, mister-smith-monitoring, mister-smith-config; dev-dependencies: tokio (test-util), serde_json
- [x] T003 Add `mister-smith-agents` to workspace members in root `Cargo.toml`
- [x] T004 Create `crates/mister-smith-agents/src/lib.rs` with module declarations and public re-exports
- [x] T005 Verify workspace builds cleanly: `cargo build --workspace` passes

**Checkpoint**: Empty crate compiles in workspace. All 882+ existing tests still pass.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Error types, configuration types, and the AgentRuntime bridge — core infrastructure that ALL user stories depend on.

**CRITICAL**: No user story work can begin until this phase is complete.

- [x] T006 [P] Define `AgentSystemError` enum in `crates/mister-smith-agents/src/errors.rs` with variants: SpawnFailed, MessageDeliveryFailed, RegistryError, SchedulingError, OrchestrationError, ToolBusError, ConfigError, PermissionDenied, Timeout, AgentNotFound, TeamError; implement Display, Error, From conversions to/from core FrameworkError
- [x] T007 [P] Define `AgentConfig` struct in `crates/mister-smith-agents/src/config.rs` with fields per data-model.md (agent_type, restart_policy, heartbeat_interval, mailbox_capacity, priority_mailbox, task_timeout, tool_permissions, role_config); implement Default for each AgentType; add deserialization from mister-smith-config
- [x] T008 [P] Define `HealthLevel` enum (Healthy, Degraded, Unhealthy, Critical) and `TeamPattern` enum (SupervisorWorker, Pipeline, Consensus) and `TaskState` enum (Pending, Assigned, Running, Completed, Failed, TimedOut, Cancelled) in `crates/mister-smith-agents/src/config.rs`
- [x] T009 Define `AgentRuntime<A: Actor>` struct in `crates/mister-smith-agents/src/agent.rs` implementing the Actor-Agent bridge pattern per contracts/agent-trait.md: holds ActorRef<A>, Arc<AgentContext> with agent_id, agent_type, state (AtomicU8 or RwLock<AgentState>), config; implement `spawn()` that creates ActorCell via ActorSystem, transitions Initializing→Running, returns AgentRuntime handle; implement `stop()` for graceful shutdown (drain → persist state → Stopping→Terminated); implement `status()` returning AgentEntry-compatible status info
- [ ] T010 Implement state persistence hooks in `crates/mister-smith-agents/src/agent.rs`: `persist_state()` using mister-smith-persistence upsert_state; `restore_state()` in pre_start loading from persistence; state persisted on significant transitions (Running, Paused, Error)
- [x] T011 Add unit tests for AgentRuntime spawn, stop, state transitions, and state persistence in `crates/mister-smith-agents/src/agent.rs` (inline #[cfg(test)] module)
- [x] T012 Verify: `cargo test --package mister-smith-agents` passes, `cargo clippy --workspace -- -D warnings` clean

**Checkpoint**: Foundation ready — AgentRuntime can spawn actors, manage lifecycle, persist state. User story implementation can begin.

---

## Phase 3: User Story 1 — Agent Lifecycle Management (Priority: P0) MVP

**Goal**: Agents can be spawned, stopped, paused, restarted via supervision, with state persistence across restarts.

**Independent Test**: Spawn agent → verify Initializing→Running → stop → verify Stopping→Terminated → restart via supervision → verify Running with restored state.

### Implementation for User Story 1

- [x] T013 [US1] Integrate AgentRuntime with SupervisedSystem in `crates/mister-smith-agents/src/agent.rs`: implement supervision notification on failure; add `restart()` that restores state from persistence and re-enters Running; add restart_count tracking
- [ ] T014 [US1] Implement `pause()` and `resume()` in `crates/mister-smith-agents/src/agent.rs`: Paused state suspends message processing (mailbox continues buffering); resume re-enters Running
- [ ] T015 [US1] Implement state transition event publishing in `crates/mister-smith-agents/src/agent.rs`: publish AgentState change events to `agents.{id}.status` via Transport on each transition; include old_state, new_state, timestamp, restart_count in event payload
- [x] T016 [US1] Add lifecycle tests in `crates/mister-smith-agents/tests/lifecycle_tests.rs`: test spawn→Running, stop→Terminated, pause→resume, failure→supervision restart with state recovery, state persistence across restart cycle, status query returns correct info (state, health, uptime, restart_count)

**Checkpoint**: Agent lifecycle fully functional. Agents spawn, stop, pause, resume, and survive restarts with state recovery.

---

## Phase 4: User Story 2 — Inter-Agent Communication (Priority: P0)

**Goal**: Agents communicate via NATS subjects with fire-and-forget, durable, and request-reply patterns. Messages are correlated and priority-aware.

**Independent Test**: Spawn two agents → Agent A sends message to Agent B → B receives and replies → A gets reply with correct correlation ID.

### Implementation for User Story 2

- [x] T017 [P] [US2] Create messaging helpers in `crates/mister-smith-agents/src/messaging.rs`: `send(transport, target_subject, envelope)` for fire-and-forget; `send_durable(durable_transport, target_subject, envelope)` for guaranteed delivery; `request(transport, target_subject, envelope, timeout)` for request-reply; `broadcast(transport, subject_pattern, envelope)` for pub to wildcard; all helpers set correlation_id, message_id, timestamp automatically
- [x] T018 [P] [US2] Implement heartbeat emitter in `crates/mister-smith-agents/src/heartbeat.rs`: `HeartbeatEmitter` spawns a background Tokio task publishing to `agents.{id}.heartbeat` at configurable interval; payload includes agent_id, state, health, uptime; `stop()` cancels the background task; integrates with AgentRuntime (started on Running, stopped on Terminated)
- [ ] T019 [US2] Wire messaging into AgentRuntime in `crates/mister-smith-agents/src/agent.rs`: AgentRuntime holds reference to Transport and DurableTransport; on spawn, subscribe to agent's command subject `agents.{id}.commands.{type}`; incoming messages deserialized and routed to Actor's handle_message; heartbeat emitter started on Running transition
- [ ] T020 [US2] Add messaging tests in `crates/mister-smith-agents/tests/messaging_tests.rs`: test fire-and-forget delivery between two agents using InMemoryTransport; test request-reply with correlation ID tracking; test heartbeat emission (verify messages published at expected interval); test message ordering (priority-aware if priority mailbox enabled)

**Checkpoint**: Agents communicate over transport. Heartbeats emit. Messages carry correlation IDs.

---

## Phase 5: User Story 6 — Agent Discovery and Registry (Priority: P1)

**Goal**: A centralized per-node registry tracks all active agents. Agents auto-register on spawn, deregister on stop. Discovery by type, capability, health, availability.

**Independent Test**: Spawn 3 agents of different types → query registry by type → verify correct results → stop one agent → verify registry updated.

### Implementation for User Story 6

- [x] T021 [US6] Implement `AgentRegistry` in `crates/mister-smith-agents/src/registry.rs`: DashMap<AgentId, AgentEntry> for concurrent access; `register(entry)` adds agent; `deregister(agent_id)` removes; `find_by_id(id)` returns Option<AgentEntry>; `find_by_type(agent_type)` returns Vec; `find_by_capability(cap)` returns Vec; `find_available(agent_type, capabilities)` returns healthy, non-busy agents; `update_health(id, health)` updates health level; `update_heartbeat(id, timestamp)` updates heartbeat_at
- [ ] T022 [US6] Implement heartbeat-based liveness monitoring in `crates/mister-smith-agents/src/registry.rs`: `LivenessMonitor` spawns background task checking heartbeat_at against phi accrual failure detector (from mister-smith-monitoring HealthMonitor); agents exceeding phi threshold marked Unhealthy then eventually deregistered; configurable heartbeat timeout and phi threshold
- [x] T023 [US6] Wire registry into AgentRuntime in `crates/mister-smith-agents/src/agent.rs`: auto-register on spawn (Initializing→Running transition); auto-deregister on stop (Terminated transition); update state in registry on each state transition; heartbeat emitter updates registry heartbeat_at on each pulse
- [x] T024 [US6] Add registry tests in `crates/mister-smith-agents/tests/lifecycle_tests.rs` (extend existing): test auto-registration on spawn; test auto-deregistration on stop; test find_by_type returns correct agents; test find_available filters unhealthy and busy agents; test liveness monitor marks stale agents as unhealthy

**Checkpoint**: Agent registry functional. Auto-registration, discovery queries, and liveness detection working.

---

## Phase 6: User Story 3 — Team Orchestration and Task Decomposition (Priority: P1)

**Goal**: Coordinators assemble teams, decompose tasks into subtasks, assign to workers, handle failures, and aggregate results.

**Independent Test**: Submit task to Coordinator → decompose into 3 subtasks → assign to Worker team → collect results → verify aggregated result. Inject Worker failure → verify reassignment and correct final result.

### Implementation for User Story 3

- [x] T025 [P] [US3] Define task scheduling types in `crates/mister-smith-agents/src/scheduler.rs`: `TaskAssignment` struct per data-model.md (task_id, task_type, priority, deadline, input, output, state, assigned_to, parent_task_id, team_id, message_id, timestamps, error_message); `TaskState` transitions; `TaskDecomposer` trait with `fn decompose(task: &TaskAssignment) -> Result<Vec<TaskAssignment>, AgentSystemError>` (pluggable); `ResultAggregator` trait with `fn aggregate(results: Vec<Value>) -> Result<Value, AgentSystemError>` (pluggable); default implementations: `IdentityDecomposer` (no decomposition) and `ArrayAggregator` (collect into JSON array)
- [x] T026 [P] [US3] Implement `Team` struct in `crates/mister-smith-agents/src/team.rs`: per data-model.md fields (team_id, coordinator_id, supervisor_id, pattern, task_id, members, timestamps); `assemble(coordinator_id, pattern, member_configs, system, transport)` spawns agents under shared supervisor; `disband()` stops all members and removes supervision subtree; `members()` returns current member refs; lifecycle bound to orchestrating task
- [x] T027 [US3] Implement task scheduler in `crates/mister-smith-agents/src/scheduler.rs`: `TaskScheduler` tracks active tasks in DashMap<TaskId, TaskAssignment>; `submit(task)` transitions to Pending; `assign(task_id, agent_id)` transitions Pending→Assigned, publishes via DurableTransport to agent's command subject; `complete(task_id, result)` transitions Running→Completed; `fail(task_id, error)` transitions Running→Failed; deadline monitor background task checks deadlines, applies timeout action (retry, reassign, fail)
- [x] T028 [US3] Implement orchestrator logic in `crates/mister-smith-agents/src/orchestrator.rs`: `Orchestrator` holds TaskDecomposer, ResultAggregator, Team, TaskScheduler; `execute(task)` → decompose → assemble_team → assign_subtasks → monitor_progress → aggregate_results; subtask dependency tracking (only assign when deps Completed); failure handling: on TeamMemberFailed event → check incomplete subtasks → reassign to restarted or different member; timeout handling per spec acceptance scenarios
- [x] T029 [US3] Add team orchestration tests in `crates/mister-smith-agents/tests/team_tests.rs`: test team assembly with SupervisorWorker pattern; test task decomposition and subtask assignment; test result aggregation from multiple workers; test worker failure → supervisor restart → coordinator reassignment → correct final result; test deadline timeout with retry action; test team disband on task completion

**Checkpoint**: End-to-end orchestration works. Coordinator decomposes, assigns, handles failures, and aggregates results.

---

## Phase 7: User Story 4 — Tool System and Agent Composition (Priority: P2)

**Goal**: Agents register as tools. Other agents discover and invoke tools via a central bus with RBAC permission checking. MCP tools accessible through same interface.

**Independent Test**: Register Worker as tool → Coordinator discovers it → invokes with valid permissions → result returned. Repeat without permissions → rejected with audit log.

### Implementation for User Story 4

- [x] T030 [P] [US4] Implement `ToolBus` in `crates/mister-smith-agents/src/tool_bus.rs`: DashMap<(String,String), ToolEntry> for registry; `register(name, namespace, agent_ref, schema, capabilities)` adds native tool; `register_mcp(name, namespace, mcp_session, schema, capabilities)` adds MCP tool; `deregister(namespace, name)` removes; `discover(principal, filter)` returns permitted tools filtered by discover:tool:{namespace} permission; `invoke(principal, namespace, name, params, timeout)` → permission check via PolicyEngine → dispatch to agent (ActorRef::ask) or MCP client → audit log via AuditLogger → return result; error variants per contracts/tool-bus.md (ToolNotFound, PermissionDenied, InvocationTimeout, InvocationFailed, ToolUnavailable)
- [ ] T031 [US4] Wire MCP bridge into ToolBus in `crates/mister-smith-agents/src/tool_bus.rs`: on MCP session connect, auto-register discovered MCP tools; on disconnect, auto-deregister; MCP tool invocation delegates to mister-smith-mcp client call_tool; map MCP Tool schema to ToolSchema
- [x] T032 [US4] Add invocation metrics tracking in `crates/mister-smith-agents/src/tool_bus.rs`: record per-tool invocation count, latency histogram (Duration), error rate; expose via method `metrics(namespace, name) -> ToolMetrics`
- [ ] T033 [US4] Add tool bus tests in `crates/mister-smith-agents/tests/tool_bus_tests.rs`: test registration and discovery with permission filtering; test successful invocation with valid permissions; test permission denied with audit log entry; test invocation timeout; test MCP tool registration and invocation (mock MCP session); test deregistration cleans up correctly

**Checkpoint**: Tool bus functional. Agent-backed and MCP tools discoverable and invocable with RBAC.

---

## Phase 8: User Story 5 — Specialized Agent Roles (Priority: P2)

**Goal**: Nine concrete agent role implementations built on the AgentRuntime infrastructure.

**Independent Test**: For each role, spawn instance → send role-appropriate message → verify correct behavior. Multi-role test: Coordinator + Workers + Supervisor in a team scenario.

### Implementation for User Story 5

- [x] T034 [P] [US5] Create roles module root `crates/mister-smith-agents/src/roles/mod.rs` with public re-exports for all 9 roles
- [x] T035 [P] [US5] Implement `SupervisorAgent` in `crates/mister-smith-agents/src/roles/supervisor.rs`: wraps Phase 3 SupervisedSystem; message types: RegisterChild, RemoveChild, QueryChildren, ChildFailed; manages child agent lifecycle; applies restart strategies; escalates on max_restarts exceeded
- [x] T036 [P] [US5] Implement `WorkerAgent` in `crates/mister-smith-agents/src/roles/worker.rs`: generic task executor; message types: AssignTask, CancelTask, QueryStatus; configurable task handler trait `TaskHandler: async fn execute(input: Value) -> Result<Value, AgentSystemError>`; on AssignTask: transitions task Running, calls handler, publishes result to `tasks.{id}.result`, returns to idle
- [x] T037 [P] [US5] Implement `CoordinatorAgent` in `crates/mister-smith-agents/src/roles/coordinator.rs`: orchestrates teams using Orchestrator; message types: SubmitTask, SubtaskResult, TeamMemberFailed, QueryProgress; holds TaskDecomposer and ResultAggregator; on SubmitTask: calls orchestrator.execute()
- [x] T038 [P] [US5] Implement `MonitorAgent` in `crates/mister-smith-agents/src/roles/monitor.rs`: subscribes to `agents.*.status` and `system.health`; message types: HealthUpdate, AlertThreshold, QueryAlerts; configurable alert thresholds; publishes alerts to `system.alerts.{severity}`
- [x] T039 [P] [US5] Implement `PlannerAgent` in `crates/mister-smith-agents/src/roles/planner.rs`: message types: PlanGoal, QueryPlan; holds pluggable `PlanGenerator` trait; receives goal, generates step-by-step execution plan, returns plan as structured JSON
- [x] T040 [P] [US5] Implement `ExecutorAgent` in `crates/mister-smith-agents/src/roles/executor.rs`: message types: ExecutePlan, StepComplete, QueryProgress; receives plan from Planner, executes steps sequentially, reports progress on `tasks.{id}.progress` per step
- [x] T041 [P] [US5] Implement `CriticAgent` in `crates/mister-smith-agents/src/roles/critic.rs`: message types: Evaluate, QueryHistory; holds pluggable `EvaluationCriteria` trait; receives output + criteria, returns scored feedback as structured JSON
- [x] T042 [P] [US5] Implement `RouterAgent` in `crates/mister-smith-agents/src/roles/router.rs`: message types: Route, AddRule, RemoveRule, QueryRules; configurable routing rules (content-based, priority-based, type-based); forwards messages to destination subjects based on rules
- [x] T043 [P] [US5] Implement `MemoryAgent` in `crates/mister-smith-agents/src/roles/memory.rs`: message types: Store, Retrieve, Search, Delete; key-value store backed by DashMap (hot) + persistence layer (cold); `Retrieve` by exact key; `Search` by prefix or metadata filter; supports TTL-based expiry
- [x] T044 [US5] Add specialized role tests in `crates/mister-smith-agents/tests/role_tests.rs`: test each role spawns and processes its message types; test SupervisorAgent child management and restart; test WorkerAgent task execution lifecycle; test CoordinatorAgent task submission and orchestration; test MonitorAgent alert generation on health degradation; test RouterAgent message forwarding based on rules; test MemoryAgent store/retrieve/search

**Checkpoint**: All 9 roles implemented and individually tested.

---

## Phase 9: Polish & Cross-Cutting Concerns

**Purpose**: Gate 7 validation, integration tests, documentation, and final quality checks.

- [x] T045 Implement Gate 7 end-to-end integration test in `crates/mister-smith-agents/tests/team_tests.rs` (or new `gate7_test.rs`): Coordinator receives complex task → decomposes into 3 subtasks → assembles Worker team under Supervisor → Workers execute subtasks → inject Worker failure mid-execution → Supervisor restarts Worker → Coordinator reassigns incomplete subtask → results aggregate back → verify correct final result with no duplicate work → verify audit trail complete
- [x] T046 [P] Add documentation: module-level doc comments on all public items in `crates/mister-smith-agents/src/lib.rs`; crate-level docs describing Phase 7 purpose, quick start, and Gate 7 criteria
- [x] T047 [P] Update `CLAUDE.md` implementation status table: add Phase 7 row as Complete with `mister-smith-agents` crate
- [x] T048 Run full verification: `cargo test --workspace` (all tests including new agents tests pass), `cargo clippy --workspace -- -D warnings` (clean)
- [x] T049 Commit and create PR for Phase 7 implementation

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies — start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 — BLOCKS all user stories
- **Phase 3 (US1 Lifecycle)**: Depends on Phase 2
- **Phase 4 (US2 Communication)**: Depends on Phase 3 (agents must exist)
- **Phase 5 (US6 Registry)**: Depends on Phase 3 (agents must exist)
- **Phase 6 (US3 Orchestration)**: Depends on Phase 3, 4, 5 (needs lifecycle, messaging, registry)
- **Phase 7 (US4 Tool System)**: Depends on Phase 3, 4 (needs agents and communication)
- **Phase 8 (US5 Roles)**: Depends on Phase 3, 4, 5, 6, 7 (needs all infrastructure)
- **Phase 9 (Polish)**: Depends on all user stories complete

### User Story Dependencies

```
US1 (Lifecycle) ─────────────────────────────────────────────────→ Required by all
US2 (Communication) ──────→ depends on US1 ──────────────────────→ Required by US3, US4, US5
US6 (Registry) ───────────→ depends on US1 ──────────────────────→ Required by US3, US5
US3 (Orchestration) ──────→ depends on US1, US2, US6
US4 (Tool System) ────────→ depends on US1, US2 ─────→ can parallel with US3, US6
US5 (Specialized Roles) ──→ depends on US1, US2, US3, US4, US6
```

### Within Each User Story

- Types/enums before logic
- Core implementation before integration hooks
- Tests after implementation (validate checkpoint)
- Story complete before dependent stories begin

### Parallel Opportunities

**Phase 2 (Foundational)**:
- T006, T007, T008 can all run in parallel (different files, independent types)

**Phase 4 (US2 Communication)**:
- T017, T018 can run in parallel (messaging.rs and heartbeat.rs are independent)

**Phase 6 (US3 Orchestration)**:
- T025, T026 can run in parallel (scheduler types and team struct are independent)

**Phase 7 (US4 Tool System)**:
- Can proceed in parallel with Phase 5 (US6 Registry) after Phase 4 completes

**Phase 8 (US5 Roles)**:
- T034-T043 can ALL run in parallel (each role is an independent file)

---

## Parallel Example: User Story 5 (Specialized Roles)

```bash
# Launch all role implementations in parallel (all independent files):
Task: T035 "Implement SupervisorAgent in src/roles/supervisor.rs"
Task: T036 "Implement WorkerAgent in src/roles/worker.rs"
Task: T037 "Implement CoordinatorAgent in src/roles/coordinator.rs"
Task: T038 "Implement MonitorAgent in src/roles/monitor.rs"
Task: T039 "Implement PlannerAgent in src/roles/planner.rs"
Task: T040 "Implement ExecutorAgent in src/roles/executor.rs"
Task: T041 "Implement CriticAgent in src/roles/critic.rs"
Task: T042 "Implement RouterAgent in src/roles/router.rs"
Task: T043 "Implement MemoryAgent in src/roles/memory.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Lifecycle)
4. **STOP and VALIDATE**: Spawn, stop, restart, state recovery all work
5. This is the minimum viable agent system

### Incremental Delivery

1. Setup + Foundational → Crate skeleton ready
2. US1 (Lifecycle) → Agents exist and survive restarts → **MVP**
3. US2 (Communication) → Agents talk to each other
4. US6 (Registry) → Agents discover each other
5. US3 (Orchestration) → Teams decompose and execute tasks → **Gate 7 ready**
6. US4 (Tool System) → Agent-as-tool composition
7. US5 (Specialized Roles) → 9 concrete role implementations → **Phase 7 complete**
8. Polish → Documentation, final verification, PR

### Gate 7 Minimum Scope

Gate 7 validation requires US1 + US2 + US6 + US3 (Phases 1-6). US4 and US5 complete the phase but are not required for the gate validation test.

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [Story] label maps task to specific user story for traceability
- Each user story independently testable at its checkpoint
- All agents use InMemoryTransport for unit tests (no NATS required)
- Integration tests requiring NATS are marked `#[ignore]` with `// Requires NATS` comment
- Pluggable handler traits (TaskHandler, TaskDecomposer, ResultAggregator, PlanGenerator, EvaluationCriteria) keep domain logic out of framework — Constitution Principle IV
