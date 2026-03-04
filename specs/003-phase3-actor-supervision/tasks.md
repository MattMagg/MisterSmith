# Tasks: Phase 3 — Actor System & Supervision Trees

**Input**: Design documents from `/specs/003-phase3-actor-supervision/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: Tests are included as this is a Rust library crate where `cargo test` is the primary validation mechanism. Each module includes inline `#[cfg(test)]` tests following established Phase 1/2 patterns.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Workspace crates**: `crates/mister-smith-actor/src/`, `crates/mister-smith-supervision/src/`
- **Integration tests**: `crates/mister-smith-integration-tests/`
- **Root config**: `Cargo.toml` (workspace)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create both new crates, configure workspace dependencies, establish module structure

- [X] T001 Add `mister-smith-actor` and `mister-smith-supervision` to workspace members in `Cargo.toml`
- [X] T002 Create `crates/mister-smith-actor/Cargo.toml` with dependencies: mister-smith-core, tokio (sync, time, rt), async-trait, uuid, serde, serde_json, tracing, thiserror
- [X] T003 [P] Create `crates/mister-smith-supervision/Cargo.toml` with dependencies: mister-smith-core, mister-smith-actor, mister-smith-events, mister-smith-monitoring, tokio, async-trait, uuid, serde, serde_json, tracing, thiserror
- [X] T004 Create `crates/mister-smith-actor/src/lib.rs` with module declarations (mailbox, actor_ref, actor_cell, system, context, errors) and public re-exports
- [X] T005 [P] Create `crates/mister-smith-supervision/src/lib.rs` with module declarations (tree, supervisor, strategy, escalation, health, events) and public re-exports
- [X] T006 Create `crates/mister-smith-actor/src/errors.rs` re-exporting ActorError from mister-smith-core
- [X] T007 [P] Create stub modules for both crates so workspace compiles: empty files for all declared modules
- [X] T008 Verify workspace builds with `cargo build` — all 10 crates compile with no errors

**Checkpoint**: Both crates exist in workspace, all modules declared, workspace compiles cleanly

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types shared across all user stories — Mailbox, Envelope, ActorRef, MailboxConfig, SpawnConfig

**CRITICAL**: No user story work can begin until this phase is complete

- [X] T009 Implement `MailboxConfig` and `SpawnConfig` structs with Default impls in `crates/mister-smith-actor/src/mailbox.rs`
- [X] T010 Implement `MailboxSender<M>` enum (Bounded/Unbounded) with `send()` and `try_send()` methods in `crates/mister-smith-actor/src/mailbox.rs`
- [X] T011 Implement `Envelope<M>` struct (message + optional oneshot reply sender) in `crates/mister-smith-actor/src/mailbox.rs`
- [X] T012 Implement `create_mailbox<M>(config: MailboxConfig) -> (MailboxSender<Envelope<M>>, mpsc::Receiver<Envelope<M>>)` factory function in `crates/mister-smith-actor/src/mailbox.rs`
- [X] T013 Implement `ActorRef<M>` struct with `tell()`, `is_alive()`, `actor_id()` methods in `crates/mister-smith-actor/src/actor_ref.rs` — tell wraps message in Envelope and sends via MailboxSender
- [X] T014 Add unit tests for Mailbox: bounded send/receive, unbounded send/receive, bounded capacity rejection (MailboxFull error), FIFO ordering in `crates/mister-smith-actor/src/mailbox.rs`
- [X] T015 [P] Add unit tests for ActorRef: tell to alive actor, tell to stopped actor (ActorStopped error), is_alive checks in `crates/mister-smith-actor/src/actor_ref.rs`
- [X] T016 Verify foundational types compile and tests pass with `cargo test -p mister-smith-actor`

**Checkpoint**: Foundation ready — Mailbox, Envelope, ActorRef, configs all tested and working

---

## Phase 3: User Story 1 — Spawn and Communicate with Actors (Priority: P1) MVP

**Goal**: Framework developers can spawn actors that run concurrently, send messages via tell/ask, and receive responses. Each actor has isolated state, unique identity, sequential message processing.

**Independent Test**: Spawn two actors, send tell message from A to B, verify receipt. Send ask message, verify reply. All within `#[tokio::test]`.

### Implementation for User Story 1

- [X] T017 [US1] Implement `ActorCell<A>` message processing loop in `crates/mister-smith-actor/src/actor_cell.rs` — owns actor + state, receives from mpsc::Receiver, calls handle_message sequentially, manages lifecycle state transitions (Initializing → Running → Stopping → Terminated)
- [X] T018 [US1] Implement `pre_start` and `post_stop` lifecycle hook invocation in ActorCell — pre_start called before first message, post_stop called on shutdown in `crates/mister-smith-actor/src/actor_cell.rs`
- [X] T019 [US1] Implement panic catching in ActorCell — wrap handle_message in catch_unwind or inspect JoinHandle for panic, transition to Error state on panic in `crates/mister-smith-actor/src/actor_cell.rs`
- [X] T020 [US1] Implement `ActorSystemConfig` struct with defaults (mailbox_capacity=1000, shutdown_timeout=5s, ask_timeout=30s, enable_events=true) in `crates/mister-smith-actor/src/system.rs`
- [X] T021 [US1] Implement `ActorSystem` struct with actor registry (RwLock<HashMap<AgentId, ActorHandle>>), shutdown signal, and start_order tracking in `crates/mister-smith-actor/src/system.rs`
- [X] T022 [US1] Implement `ActorSystem::spawn()` — creates mailbox channel, wraps actor in ActorCell, spawns tokio task, registers ActorHandle, returns ActorRef in `crates/mister-smith-actor/src/system.rs`
- [X] T023 [US1] Implement `ActorHandle` (type-erased) struct with actor_id, JoinHandle, lifecycle_state, stop_tx, and type-erased mailbox_sender in `crates/mister-smith-actor/src/system.rs`
- [X] T024 [US1] Implement ask pattern: add `ask()` method to ActorRef that creates oneshot channel, wraps message in Envelope with reply_tx, sends via tell, awaits reply with timeout in `crates/mister-smith-actor/src/actor_ref.rs`
- [X] T025 [US1] Implement ask reply routing in ActorCell — after handle_message, if Envelope has reply_tx, send Ok result through oneshot in `crates/mister-smith-actor/src/actor_cell.rs`
- [X] T026 [US1] Implement `ActorSystem::shutdown()` — sends stop signals to all actors in reverse-start order, awaits completion with per-actor timeout, calls post_stop hooks in `crates/mister-smith-actor/src/system.rs`
- [X] T027 [US1] Implement `ActorSystem::actor_count()` and `ActorSystem::get_ref()` lookup methods in `crates/mister-smith-actor/src/system.rs`
- [X] T028 [US1] Implement `ActorContext` struct (actor_id, system weak ref, self_ref) in `crates/mister-smith-actor/src/context.rs`
- [X] T029 [US1] Add unit tests for ActorCell: spawn actor, process tell message, verify state mutation in `crates/mister-smith-actor/src/actor_cell.rs`
- [X] T030 [US1] Add unit tests for ask pattern: ask with reply, ask timeout (AskTimeout error) in `crates/mister-smith-actor/src/actor_cell.rs`
- [X] T031 [US1] Add unit tests for ActorSystem: spawn actor gets unique ActorId, spawn transitions to Running, shutdown calls post_stop, actor_count accuracy in `crates/mister-smith-actor/src/system.rs`
- [X] T032 [US1] Add unit test for bounded mailbox rejection: send N+1 messages to capacity-N mailbox, verify MailboxFull error in `crates/mister-smith-actor/src/system.rs`
- [X] T033 [US1] Add unit test for message to terminated actor: stop actor, send tell, verify ActorStopped error in `crates/mister-smith-actor/src/system.rs`
- [X] T034 [US1] Verify all US1 acceptance scenarios pass with `cargo test -p mister-smith-actor`

**Checkpoint**: US1 complete — actors spawn, communicate via tell/ask, bounded mailbox enforces capacity, graceful shutdown works

---

## Phase 4: User Story 2 — Supervise Actors with Restart Policies (Priority: P1)

**Goal**: Framework developers organize actors into supervision trees with parent supervisors that monitor children and apply restart policies (OneForOne, OneForAll, RestForOne) when failures occur.

**Independent Test**: Create supervisor with three children. Kill one child. Verify correct restart policy behavior for each policy type.

**Dependencies**: Requires US1 (ActorSystem, ActorCell, spawn) to be complete

### Implementation for User Story 2

- [X] T035 [US2] Implement `ChildEntry` struct (actor_id, restart_scope, start_order, restart_count) in `crates/mister-smith-supervision/src/supervisor.rs`
- [X] T036 [US2] Implement `SupervisorNode` struct (id, parent_id, children Vec, strategy, restart_history VecDeque) in `crates/mister-smith-supervision/src/supervisor.rs`
- [X] T037 [US2] Implement `SupervisionEvent` and `SupervisionEventType` internal notification types in `crates/mister-smith-supervision/src/supervisor.rs`
- [X] T038 [US2] Implement `SupervisionDecision` enum (Restart, Escalate, Stop, Ignore, Shutdown) in `crates/mister-smith-supervision/src/strategy.rs`
- [X] T039 [US2] Implement strategy executor: `apply_restart_policy(node: &SupervisorNode, failed_child_id: AgentId) -> Vec<AgentId>` — returns list of children to restart based on OneForOne/OneForAll/RestForOne in `crates/mister-smith-supervision/src/strategy.rs`
- [X] T040 [US2] Implement RestartScope filtering: `should_restart(scope: RestartScope, termination_type: TerminationType) -> bool` — Permanent=always, Transient=error-only, Temporary=never in `crates/mister-smith-supervision/src/strategy.rs`
- [X] T041 [US2] Implement restart budget checking: `check_restart_budget(node: &mut SupervisorNode) -> bool` — checks restart_history against max_failures/failure_window, prunes expired entries in `crates/mister-smith-supervision/src/strategy.rs`
- [X] T042 [US2] Implement backoff delay computation: `compute_backoff(strategy: &BackoffStrategy, attempt: u32) -> Duration` consuming core's BackoffStrategy enum in `crates/mister-smith-supervision/src/strategy.rs`
- [X] T043 [US2] Implement `SupervisionTree::new()`, `add_supervisor()`, `add_child()`, `remove_child()` tree management methods in `crates/mister-smith-supervision/src/tree.rs`
- [X] T044 [US2] Implement `SupervisionTree::handle_failure()` — looks up child's supervisor, applies strategy/scope/budget, returns SupervisionDecision in `crates/mister-smith-supervision/src/tree.rs`
- [X] T045 [US2] Implement supervisor notification channel in ActorCell — on failure/stop, send SupervisionEvent to supervisor's mpsc channel in `crates/mister-smith-actor/src/actor_cell.rs`
- [X] T046 [US2] Implement `ActorSystem::spawn_supervised()` — spawns actor, registers in supervision tree as child of given supervisor in `crates/mister-smith-actor/src/system.rs`
- [X] T047 [US2] Implement `ActorSystem::create_supervisor()` — creates supervisor node in tree with given strategy, returns supervisor AgentId in `crates/mister-smith-actor/src/system.rs`
- [X] T048 [US2] Implement actor restart logic in ActorSystem — given SupervisionDecision::Restart(ids), stop affected actors, create fresh state instances, re-run pre_start, preserve ActorId and mailbox in `crates/mister-smith-actor/src/system.rs`
- [X] T049 [US2] Add unit tests for strategy executor: OneForOne returns only failed child, OneForAll returns all children, RestForOne returns failed + younger siblings in `crates/mister-smith-supervision/src/strategy.rs`
- [X] T050 [US2] Add unit tests for RestartScope: Permanent always restarts, Transient restarts on error not normal stop, Temporary never restarts in `crates/mister-smith-supervision/src/strategy.rs`
- [X] T051 [US2] Add unit tests for restart budget: 3 failures in 60s window OK, 4th triggers escalation, expired failures pruned from history in `crates/mister-smith-supervision/src/strategy.rs`
- [X] T052 [US2] Add unit tests for backoff computation: Exponential with initial/max/multiplier, Fixed, Linear in `crates/mister-smith-supervision/src/strategy.rs`
- [X] T053 [US2] Add unit tests for SupervisionTree: add supervisor, add children, handle failure returns correct decision in `crates/mister-smith-supervision/src/tree.rs`
- [X] T054 [US2] Add integration test: OneForOne — supervisor with 3 children, kill B, verify only B restarts in `crates/mister-smith-integration-tests/`
- [X] T055 [US2] Add integration test: OneForAll — supervisor with 3 children, kill B, verify all restart in `crates/mister-smith-integration-tests/`
- [X] T056 [US2] Add integration test: RestForOne — supervisor with children A,B,C, kill B, verify B and C restart, A undisturbed in `crates/mister-smith-integration-tests/`
- [X] T057 [US2] Add integration test: restarted actor has fresh state, same ActorId, pre_start called in `crates/mister-smith-integration-tests/`
- [X] T058 [US2] Verify all US2 acceptance scenarios pass with `cargo test -p mister-smith-supervision` and `cargo test -p mister-smith-integration-tests`

**Checkpoint**: US2 complete — actors are supervised, restart policies work correctly, restart scopes filter appropriately, budget exhaustion triggers escalation

---

## Phase 5: User Story 3 — Compose Hierarchical Supervision Trees (Priority: P2)

**Goal**: Framework developers build multi-level supervision trees where supervisors are themselves supervised. Failures escalate up the tree. Trees can be queried and shut down gracefully.

**Independent Test**: Create 3-level tree (root → mid-level → workers). Trigger failure cascade from worker through mid-level to root. Verify each level applies its own policy.

**Dependencies**: Requires US2 (supervision tree, restart policies) to be complete

### Implementation for User Story 3

- [X] T059 [US3] Implement `SupervisionTree::find_supervisor(child_id: AgentId) -> Option<AgentId>` to look up parent supervisor for escalation in `crates/mister-smith-supervision/src/tree.rs`
- [X] T060 [US3] Implement failure escalation: `escalate(supervisor_id: AgentId, error: ActorError)` — walks up tree, parent applies its own strategy, continues until handled or root reached in `crates/mister-smith-supervision/src/escalation.rs`
- [X] T061 [US3] Implement root exhaustion handling: when root supervisor exceeds restart budget, return SupervisionDecision::Shutdown triggering full tree graceful shutdown in `crates/mister-smith-supervision/src/escalation.rs`
- [X] T062 [US3] Implement `TreeStatus` struct and `SupervisionTree::query_status()` — returns total_nodes, nodes_by_state, tree_depth, total_restarts in `crates/mister-smith-supervision/src/tree.rs`
- [X] T063 [US3] Implement `SupervisionTree::shutdown_order()` — returns all actor IDs in reverse-start order (leaves first, root last) in `crates/mister-smith-supervision/src/tree.rs`
- [X] T064 [US3] Implement `ActorSystem::create_supervisor_under()` — creates supervisor as child of another supervisor, enabling hierarchical nesting in `crates/mister-smith-actor/src/system.rs`
- [X] T065 [US3] Implement `ActorSystem::tree_status()` — delegates to SupervisionTree::query_status() in `crates/mister-smith-actor/src/system.rs`
- [X] T066 [US3] Wire up escalation in the restart loop — when handle_failure returns Escalate, call escalate() which propagates up the tree in `crates/mister-smith-actor/src/system.rs`
- [X] T067 [US3] Update `ActorSystem::shutdown()` to use SupervisionTree::shutdown_order() for correct reverse-start ordering with tree-aware traversal in `crates/mister-smith-actor/src/system.rs`
- [X] T068 [US3] Add unit tests for escalation: mid-level exhausts budget, escalates to root, root applies its policy in `crates/mister-smith-supervision/src/escalation.rs`
- [X] T069 [US3] Add unit tests for tree status: correct node counts, depth calculation, state aggregation in `crates/mister-smith-supervision/src/tree.rs`
- [X] T070 [US3] Add unit tests for shutdown_order: leaves first, root last, reverse of start order in `crates/mister-smith-supervision/src/tree.rs`
- [X] T071 [US3] Add integration test: 3-level tree failure cascade — worker fails repeatedly, mid-level escalates to root, root applies its own policy in `crates/mister-smith-integration-tests/`
- [X] T072 [US3] Add integration test: tree with 10+ nodes, query status returns correct counts and depth in `crates/mister-smith-integration-tests/`
- [X] T073 [US3] Add integration test: graceful shutdown of tree — all post_stop hooks called in reverse-start order in `crates/mister-smith-integration-tests/`
- [X] T074 [US3] Verify all US3 acceptance scenarios pass with `cargo test`

**Checkpoint**: US3 complete — hierarchical trees compose, failures escalate correctly, tree status queryable, graceful shutdown works in reverse-start order

---

## Phase 6: User Story 4 — Integrate with Event System and Monitoring (Priority: P2)

**Goal**: Actor lifecycle events integrate with Phase 2 EventBus. Health checks report actor system health. Metrics track actor counts, throughput, mailbox depth, failure rates.

**Independent Test**: Spawn actors, subscribe to lifecycle events on EventBus, trigger failure and restart, verify expected events with correct metadata.

**Dependencies**: Requires US2 (supervision) for failure/restart events; uses Phase 2 EventBus and HealthMonitor

### Implementation for User Story 4

- [X] T075 [US4] Implement lifecycle event emission in ActorCell — emit AgentEventType::Created on spawn, Started on pre_start success, Failed on error, Stopped on termination, StateChanged on transitions via EventPublisher in `crates/mister-smith-actor/src/actor_cell.rs`
- [X] T076 [US4] Implement `EventBuilder` usage for supervision events — include correlation_id and causation_id linking failure → restart event chains in `crates/mister-smith-supervision/src/events.rs`
- [X] T077 [US4] Implement supervision event emission — emit events when supervisor restarts child, escalates failure, or exhausts budget in `crates/mister-smith-supervision/src/events.rs`
- [X] T078 [US4] Wire EventPublisher into SupervisionTree and propagate to event emission code in `crates/mister-smith-supervision/src/tree.rs`
- [X] T079 [US4] Implement `ActorSystemHealthCheck` struct implementing HealthCheck trait — reports Healthy/Degraded/Unhealthy based on error-state actor ratio, includes metadata (total_actors, actors_by_state, tree_depth, restart_count) in `crates/mister-smith-supervision/src/health.rs`
- [X] T080 [US4] Implement actor metrics collection — track messages_processed, mailbox_depths, restart_counts, failure_rates using MetricsCollector from monitoring crate in `crates/mister-smith-supervision/src/health.rs`
- [X] T081 [US4] Wire ActorSystem to accept and propagate EventPublisher to ActorCell and SupervisionTree via `with_event_publisher()` builder in `crates/mister-smith-actor/src/system.rs`
- [X] T082 [US4] Add unit tests for lifecycle events: spawn emits Created+Started, failure emits Failed, restart emits Started with correlation_id in `crates/mister-smith-supervision/src/events.rs`
- [X] T083 [US4] Add unit tests for health check: all healthy → Healthy, some errors → Degraded, many errors → Unhealthy in `crates/mister-smith-supervision/src/health.rs`
- [X] T084 [US4] Add integration test: subscribe to EventBus, spawn actor, trigger failure and restart, verify Created→Started→Failed→Started event sequence with matching correlation IDs in `crates/mister-smith-integration-tests/`
- [X] T085 [US4] Add integration test: register ActorSystemHealthCheck with HealthMonitor, verify health status reports correct actor counts and tree depth in `crates/mister-smith-integration-tests/`
- [X] T086 [US4] Verify all US4 acceptance scenarios pass with `cargo test`

**Checkpoint**: US4 complete — lifecycle events emitted with correlation IDs, health checks report actor system status, metrics tracked

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, performance validation, documentation, clippy, and Gate 3 criteria verification

- [X] T087 Add edge case test: ask timeout — actor delays response beyond timeout, caller receives AskTimeout error in `crates/mister-smith-actor/src/actor_cell.rs`
- [X] T088 Add edge case test: mutual ask deadlock — two actors send ask to each other, both timeout without permanent hang in `crates/mister-smith-integration-tests/`
- [X] T089 Add edge case test: pre_start failure on initial spawn — spawn returns error, actor not registered in `crates/mister-smith-actor/src/system.rs`
- [X] T090 Add edge case test: pre_start failure during restart — actor transitions to Error, supervisor notified in `crates/mister-smith-integration-tests/`
- [X] T091 Add edge case test: root supervisor exhaustion — triggers full tree graceful shutdown in `crates/mister-smith-integration-tests/`
- [X] T092 Add edge case test: message sent during Restarting state — message buffered in mailbox, processed after restart completes in `crates/mister-smith-integration-tests/`
- [X] T093 Add edge case test: concurrent child failures — supervisor processes failures sequentially, restart policy applied correctly for each in `crates/mister-smith-integration-tests/`
- [X] T094 Add performance test: spawn 1000 actors, each processes 10 messages, all complete within 5 seconds (SC-001) in `crates/mister-smith-integration-tests/`
- [X] T095 Add performance test: graceful shutdown of 100+ actor tree with all post_stop hooks called (SC-006) in `crates/mister-smith-integration-tests/`
- [X] T096 Update `crates/mister-smith-actor/src/lib.rs` public API re-exports — ensure all user-facing types are accessible
- [X] T097 [P] Update `crates/mister-smith-supervision/src/lib.rs` public API re-exports — ensure all user-facing types are accessible
- [X] T098 Run `cargo clippy --workspace` and fix any warnings in new crates
- [X] T099 Run `cargo test --workspace` and verify all tests pass (Phase 1 + Phase 2 + Phase 3)
- [X] T100 Validate Gate 3 criteria: actors spawn, communicate via mailboxes, are supervised, supervision trees compose hierarchically — document evidence

**Checkpoint**: Phase 3 complete — all edge cases tested, performance validated, workspace clean, Gate 3 criteria satisfied

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001-T008) — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational (T009-T016) — actor primitives
- **US2 (Phase 4)**: Depends on US1 (T017-T034) — supervision requires working actors
- **US3 (Phase 5)**: Depends on US2 (T035-T058) — hierarchical trees require basic supervision
- **US4 (Phase 6)**: Depends on US2 (T035-T058) — events/monitoring require supervision; can run parallel with US3
- **Polish (Phase 7)**: Depends on US1-US4 complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational — no dependencies on other stories
- **US2 (P1)**: Depends on US1 — supervision requires working actor spawn/communication
- **US3 (P2)**: Depends on US2 — hierarchical composition requires basic supervision
- **US4 (P2)**: Depends on US2 — event/health integration requires supervision; **can run in parallel with US3**

### Within Each User Story

- Types/structs before logic using them
- Internal types before public API
- Core implementation before edge case handling
- Unit tests alongside implementation
- Integration tests after all story tasks complete

### Parallel Opportunities

- **Phase 1**: T002 and T003 (Cargo.toml files), T004 and T005 (lib.rs files), T006 and T007 (stubs)
- **Phase 2**: T014 and T015 (mailbox and actor_ref tests)
- **Phase 6 + Phase 5**: US4 can run concurrently with US3 after US2 completes
- **Phase 7**: T096 and T097 (lib.rs re-exports for different crates)

---

## Parallel Example: User Story 2

```bash
# After US1 complete, launch US2 type definitions together:
Task: T035 "Implement ChildEntry struct in supervision/src/supervisor.rs"
Task: T036 "Implement SupervisorNode struct in supervision/src/supervisor.rs"  # same file, sequential
Task: T037 "Implement SupervisionEvent types in supervision/src/supervisor.rs"  # same file, sequential
Task: T038 "Implement SupervisionDecision enum in supervision/src/strategy.rs"  # different file, parallel

# After types defined, strategy logic tasks:
Task: T039 "Implement apply_restart_policy in strategy.rs"
Task: T040 "Implement should_restart in strategy.rs"  # same file, sequential
Task: T041 "Implement check_restart_budget in strategy.rs"  # same file, sequential
Task: T042 "Implement compute_backoff in strategy.rs"  # same file, sequential

# Tree management (parallel with strategy if types done):
Task: T043 "Implement SupervisionTree management in tree.rs"
```

---

## Parallel Example: US3 + US4

```bash
# After US2 complete, US3 and US4 can proceed in parallel:

# US3 track (escalation + hierarchy):
Task: T059-T074 in supervision/src/escalation.rs, tree.rs, actor/src/system.rs

# US4 track (events + monitoring):
Task: T075-T086 in actor/src/actor_cell.rs, supervision/src/events.rs, health.rs
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T008)
2. Complete Phase 2: Foundational (T009-T016)
3. Complete Phase 3: User Story 1 (T017-T034)
4. **STOP and VALIDATE**: actors spawn, communicate via tell/ask, bounded mailbox works, shutdown graceful
5. This is a usable actor system even without supervision

### Incremental Delivery

1. Setup + Foundational → Crates exist, mailbox/ref work
2. US1 → Basic actor system MVP — spawn, communicate, shutdown
3. US2 → Supervision added — restart policies, scopes, escalation
4. US3 → Hierarchical composition — multi-level trees, status queries
5. US4 → Observability — events, health checks, metrics
6. Polish → Edge cases, performance, Gate 3 validation

### Task Count Summary

| Phase | Tasks | Range |
|-------|-------|-------|
| Setup | 8 | T001-T008 |
| Foundational | 8 | T009-T016 |
| US1 (Spawn/Communicate) | 18 | T017-T034 |
| US2 (Supervision/Restart) | 24 | T035-T058 |
| US3 (Hierarchical Trees) | 16 | T059-T074 |
| US4 (Events/Monitoring) | 12 | T075-T086 |
| Polish | 14 | T087-T100 |
| **Total** | **100** | T001-T100 |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- US3 and US4 can run in parallel after US2 completes
- All types (RestartPolicy, RestartScope, SupervisionStrategy, BackoffStrategy, EscalationPolicy, AgentState, AgentId, ActorError, SupervisionError) are imported from mister-smith-core — never redefined
