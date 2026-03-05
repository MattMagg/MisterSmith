# Tasks: Phase 6 — Persistence & State

**Input**: Design documents from `/specs/006-phase6-persistence-state/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included — env-gated integration tests for PostgreSQL and JetStream KV per research.md R6.

**Organization**: Tasks grouped by user story. Each story is independently testable after foundational phase completes.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Crate root**: `crates/mister-smith-persistence/`
- **Source**: `crates/mister-smith-persistence/src/`
- **Migrations**: `crates/mister-smith-persistence/migrations/`
- **Tests**: `crates/mister-smith-persistence/tests/`
- **Integration tests**: `crates/mister-smith-integration-tests/tests/`
- **Core error expansion**: `crates/mister-smith-core/src/error.rs`
- **Config expansion**: `crates/mister-smith-config/src/types.rs`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create crate scaffolding and workspace integration

- [x] T001 Add `sqlx` and `chrono` to workspace dependencies in `Cargo.toml` per plan.md Dependency Changes section
- [x] T002 Create `crates/mister-smith-persistence/Cargo.toml` with workspace deps, features (`sqlx`, `security`), and dev-dependencies per plan.md
- [x] T003 Add `"crates/mister-smith-persistence"` to workspace members in root `Cargo.toml`
- [x] T004 Create `crates/mister-smith-persistence/src/lib.rs` with module declarations (`postgres`, `kv`, `hybrid`, `repository`, `config`, `error`, `health`) and public re-exports
- [x] T005 [P] Create `crates/mister-smith-persistence/src/config.rs` with `PersistenceConfig`, `PostgresConfig`, `KvConfig`, `FlushConfig` (including `max_flush_retries: u32`), `CheckpointConfig` (including `interval_secs: u64`) structs per quickstart.md Configuration section — all `Serialize`/`Deserialize` with `#[serde(default)]`
- [x] T006 [P] Create `crates/mister-smith-persistence/src/error.rs` with `PersistenceError` re-export from core and conversion helper functions `from_sqlx_error()` and `from_kv_error()` per research.md R5

**Checkpoint**: `cargo build -p mister-smith-persistence` compiles (empty modules)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T007 Expand `PersistenceError` in `crates/mister-smith-core/src/error.rs` — add `NotFound(String)`, `DuplicateKey(String)`, `VersionConflict { key: String, expected: u64, actual: u64 }`, `ConnectionFailed(String)`, `TtlExpired(String)`, `MigrationFailed(String)` variants per research.md R5
- [x] T008 Add `PersistenceConfig` field to `FrameworkConfig` in `crates/mister-smith-config/src/types.rs` — `#[serde(default)] pub persistence: PersistenceConfig`
- [x] T009 Create `crates/mister-smith-persistence/src/postgres/mod.rs` with submodule declarations (`pool`, `migrations`, `queries`)
- [x] T010 Create `crates/mister-smith-persistence/src/postgres/pool.rs` — `PostgresConnection` struct wrapping `sqlx::PgPool`, implementing core `Resource` trait per research.md R2: `acquire()` creates pool via `PgPoolOptions`, `release()` calls `pool.close()`, `health_check()` runs `pool.acquire()` test query
- [x] T011 Create SQL migration files in `crates/mister-smith-persistence/migrations/`:
  - `00001_initial_schema.sql` — schemas (`agents`, `tasks`, `messages`), domain types (`agent_status_type`, `task_status_type`, `task_priority_type`, `message_type`), core tables (`agents.registry`, `agents.state`, `agents.checkpoints`, `tasks.records`, `messages.records`, `configurations`) per data-model.md
  - `00002_indexes.sql` — all B-tree and GIN indexes defined in data-model.md
  - `00003_partitions.sql` — hash partitioning for `agents.state` (8 partitions), range partitioning for `messages.records` and `audit_log` (monthly, current + 3 months) per research.md R7
  - `00004_audit_schema.sql` — `audit_log` table with time partitioning per data-model.md AuditLogEntry
- [x] T012 Create `crates/mister-smith-persistence/src/postgres/migrations.rs` — `MigrationRunner` struct with `run()`, `current_version()`, `status()`, `verify()` methods per contracts/migrations.md, using `sqlx::migrate!()` macro
- [x] T013 Create `crates/mister-smith-persistence/src/kv/mod.rs` with submodule declarations (`buckets`, `state`, `watch`)
- [x] T014 Create `crates/mister-smith-persistence/src/kv/buckets.rs` — `KvBucketManager` with `new()`, `initialize_buckets()`, `bucket()`, `health_check()` per contracts/kv-store.md, creating SESSION_DATA/AGENT_STATE/QUERY_CACHE buckets with TTLs from data-model.md KV Buckets table
- [x] T015 Create `crates/mister-smith-persistence/src/health.rs` — health check implementations for PostgreSQL (pool acquire test) and JetStream KV (bucket accessibility test) per spec FR-017, returning `HealthStatus` from core

**Checkpoint**: Foundation ready — `cargo build -p mister-smith-persistence` compiles, migration files exist, PG pool and KV bucket manager are functional types. User story implementation can now begin.

---

## Phase 3: User Story 1 — Agent State Survives Restarts (Priority: P1) MVP

**Goal**: Agents can persist state to durable storage and recover it after restart

**Independent Test**: Write state via repository, terminate, restart, verify state is intact

### Implementation for User Story 1

- [x] T016 [P] [US1] Create `crates/mister-smith-persistence/src/kv/state.rs` — `StateManager` with `save()`, `get()`, `update()`, `delete()` methods per contracts/kv-store.md, including `ConflictStrategy` enum and CAS-based `update()`
- [x] T017 [P] [US1] Create `crates/mister-smith-persistence/src/postgres/queries.rs` — prepared query helpers for agent registry CRUD (`insert_agent`, `find_agent`, `update_agent_status`, `find_agents_by_type`, `find_agents_by_status`) and agent state CRUD (`upsert_state`, `get_state`, `get_all_state`, `delete_state`) using `sqlx::query()` runtime variant
- [x] T018 [US1] Create `crates/mister-smith-persistence/src/hybrid/mod.rs` with submodule declarations (`manager`, `router`)
- [x] T019 [US1] Create `crates/mister-smith-persistence/src/hybrid/router.rs` — `DataRouter` with `select_storage()` mapping `DataType` enum to `StorageLayer` enum per data-model.md storage-patterns, and `get_ttl()` per data type
- [x] T020 [US1] Create `crates/mister-smith-persistence/src/hybrid/manager.rs` — `HybridStateManager` with `write_state()` (KV first + dirty tracking), `read_state()` (KV first, SQL fallback + lazy hydration), and basic `flush_to_sql()` (batch upsert dirty keys to PG in a transaction) per research.md R4
- [x] T021 [US1] Create `crates/mister-smith-persistence/src/repository/mod.rs` — `Repository<T>` trait definition per contracts/repository.md with `save`, `find`, `update`, `delete` methods
- [x] T022 [US1] Create `crates/mister-smith-persistence/src/repository/agent.rs` — `AgentRepository` implementing `Repository<AgentRecord>` plus `save_state()`, `get_state()`, `get_all_state()`, `checkpoint()`, `hydrate()` methods per contracts/repository.md AgentRepository section
- [x] T023 [US1] Write unit tests in `crates/mister-smith-persistence/src/repository/agent.rs` (inline `#[cfg(test)]` module) — test `AgentRecord` serialization, `DataRouter` routing logic, `ConflictStrategy` variants, error mapping from sqlx/KV errors to `PersistenceError`
- [x] T024 [US1] Write env-gated integration test `crates/mister-smith-persistence/tests/postgres_tests.rs` — test PG pool creation, migration execution, agent registry CRUD, agent state CRUD (requires `DATABASE_URL`) per research.md R6 pattern
- [x] T025 [US1] Write env-gated integration test `crates/mister-smith-persistence/tests/kv_tests.rs` — test KV bucket creation, state save/get/update/delete, conflict resolution, TTL expiration, and watch event delivery (requires `NATS_URL`) per research.md R6 pattern

**Checkpoint**: Agent state can be saved, restored across restarts, and checkpointed. `cargo test -p mister-smith-persistence` passes unit tests; integration tests pass with `DATABASE_URL` and `NATS_URL` set.

---

## Phase 4: User Story 2 — Fast Distributed State Access (Priority: P2)

**Goal**: KV layer provides sub-millisecond reads with automatic SQL fallback, dirty-key flushing, and change watching

**Independent Test**: Write to KV, read within latency budget, let TTL expire, verify SQL fallback hydrates KV

### Implementation for User Story 2

- [x] T026 [US2] Create `crates/mister-smith-persistence/src/kv/watch.rs` — KV watcher wrapping `kv::Store::watch()` that emits `StateChange` events (key, operation, revision) as a tokio stream per contracts/kv-store.md
- [x] T027 [US2] Extend `HybridStateManager` in `crates/mister-smith-persistence/src/hybrid/manager.rs` — add dirty-key tracking with `Arc<Mutex<HashSet<String>>>`, configurable `flush_threshold` (count), and `flush_deadline` (time since oldest dirty key). Background flush task spawned via `tokio::spawn` with circuit breaker wrapping from `mister-smith-async` per research.md R4
- [x] T028 [US2] Implement flush safety in `crates/mister-smith-persistence/src/hybrid/manager.rs` — flush deadline = `min(configured_deadline, kv_ttl - safety_margin)` to prevent data loss from TTL expiring before flush per plan.md D6
- [x] T029 [US2] Implement graceful degradation in `crates/mister-smith-persistence/src/hybrid/manager.rs` — KV-unreachable: fall back to SQL with warning log; SQL-unreachable: continue KV writes with dirty tracking, retry flushes with backoff; both unreachable: return `PersistenceError::ConnectionFailed` per spec Edge Cases
- [x] T030 [US2] Write env-gated integration test `crates/mister-smith-persistence/tests/hybrid_tests.rs` — test write-through to KV, flush to SQL on threshold, read-from-KV path, SQL-fallback-on-KV-miss path, dirty tracking accuracy (requires `DATABASE_URL` + `NATS_URL`)

**Checkpoint**: KV provides fast reads, dirty state flushes to SQL, fallback paths work. Integration tests verify dual-store behavior.

---

## Phase 5: User Story 3 — Schema-Managed Relational Storage (Priority: P3)

**Goal**: Operators can run versioned migrations to create/upgrade schemas with rollback support and partition management

**Independent Test**: Run migrations on empty DB, verify tables/indexes exist, run subsequent migration, verify upgrade with no data loss

### Implementation for User Story 3

- [x] T031 [US3] Add rollback SQL files alongside migration files — `00001_initial_schema.down.sql`, `00002_indexes.down.sql`, `00003_partitions.down.sql`, `00004_audit_schema.down.sql` in `crates/mister-smith-persistence/migrations/`
- [x] T032 [US3] Extend `MigrationRunner` in `crates/mister-smith-persistence/src/postgres/migrations.rs` — add `revert()` method to rollback last migration, using down-migration SQL
- [x] T033 [US3] Add partition management SQL functions in `crates/mister-smith-persistence/migrations/00003_partitions.sql` — function to create next month's partition for messages and audit_log tables, function to check partition coverage per research.md R7
- [x] T034 [US3] Write env-gated integration test `crates/mister-smith-persistence/tests/migration_tests.rs` — test full migration cycle (empty → migrated → insert data → next migration → verify data preserved), test `status()` and `verify()` methods, test rollback (requires `DATABASE_URL`)

**Checkpoint**: Migrations run forward and backward. Partition management functions exist. Schema versioning is tracked and verifiable.

---

## Phase 6: User Story 4 — Task and Message Persistence (Priority: P4)

**Goal**: Tasks and messages are durably stored and queryable by agent, status, time range, priority, and correlation ID

**Independent Test**: Create tasks and messages via repository, query by various criteria, verify correctness and ordering

### Implementation for User Story 4

- [x] T035 [P] [US4] Add task query helpers to `crates/mister-smith-persistence/src/postgres/queries.rs` — `insert_task`, `find_task`, `update_task_status`, `find_tasks_by_agent`, `find_tasks_by_time_range`, `find_tasks_by_correlation`, `find_tasks_by_status_and_priority`
- [x] T036 [P] [US4] Add message query helpers to `crates/mister-smith-persistence/src/postgres/queries.rs` — `insert_message`, `find_message`, `update_message_status`, `find_messages_by_sender`, `find_messages_by_receiver`, `find_messages_by_correlation`, `find_messages_by_time_range`
- [x] T037 [US4] Create `crates/mister-smith-persistence/src/repository/task.rs` — `TaskRepository` implementing `Repository<TaskRecord>` plus `find_by_agent()`, `find_by_time_range()`, `find_by_correlation()` per contracts/repository.md TaskRepository section
- [x] T038 [US4] Create `crates/mister-smith-persistence/src/repository/message.rs` — `MessageRepository` implementing `Repository<MessageRecord>` plus `find_by_correlation()`, `find_by_sender()` per contracts/repository.md MessageRepository section
- [x] T039 [US4] Write unit tests (inline `#[cfg(test)]`) in task.rs and message.rs — test record serialization/deserialization, priority range validation (0-4), status enum mapping
- [x] T040 [US4] Extend `crates/mister-smith-persistence/tests/postgres_tests.rs` — add task CRUD tests, message CRUD tests, correlation query tests, time-range query tests (env-gated)

**Checkpoint**: Tasks and messages persist with full metadata. Queries by agent, status, time, correlation return correct results. Partitioned tables handle volume.

---

## Phase 7: User Story 5 — Remaining Repositories & Wiring (Priority: P5)

**Goal**: Complete all repository implementations, add transactional support, and wire audit persistence from Phase 5

**Independent Test**: CRUD through repository API, concurrent updates trigger conflict resolution, transactions atomically commit or rollback, audit events flow from ring buffer to PostgreSQL

### Implementation for User Story 5

- [x] T041 [US5] Create `crates/mister-smith-persistence/src/repository/audit.rs` — `AuditRepository` with `append()` and `append_batch()` for audit log persistence, plus `find_by_agent()` query per contracts/repository.md AuditRepository section
- [x] T042 [US5] Add configuration query helpers to `crates/mister-smith-persistence/src/postgres/queries.rs` — `upsert_config`, `get_config`, `get_config_by_environment`, `get_config_history` (no dedicated ConfigRepository — config access is low-frequency and doesn't need dual-store routing)
- [x] T043 [US5] Add transactional support to repository implementations — method `with_transaction()` on repositories that wraps operations in `sqlx::Transaction` for atomicity per spec FR-010
- [x] T044 [US5] Add audit persistence wiring — `AuditPersister` struct that periodically drains the Phase 5 `AuditLogger` in-memory ring buffer and batch-writes entries to `AuditRepository` via `append_batch()` per plan.md D7. Feature-gated behind `security` feature.
- [x] T045 [US5] Write env-gated integration test `crates/mister-smith-persistence/tests/repository_tests.rs` — test generic `Repository<T>` behavior: save/find/update/delete for agents, tasks, messages. Test OCC conflict on concurrent version updates. Test transaction commit and rollback. Test audit append and batch operations.

**Checkpoint**: All repositories functional. Transactions provide atomicity. Audit events flow from Phase 5 to PostgreSQL. OCC prevents lost updates.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Integration testing, workspace validation, documentation

- [x] T046 [P] Write cross-crate integration tests in `crates/mister-smith-integration-tests/tests/persistence_integration.rs` — test persistence + security audit wiring (e2e: security event → audit ring buffer → AuditPersister → PostgreSQL → query back), test persistence + events notification, test persistence health checks registered with monitoring
- [x] T047 [P] Add `mister-smith-persistence` dependency (workspace, optional, feature-gated) to `crates/mister-smith-integration-tests/Cargo.toml`
- [x] T048 [P] Write env-gated performance validation in `crates/mister-smith-persistence/tests/performance_tests.rs` — SC-002: benchmark KV read latency (assert <1ms local p95); SC-005: spawn 100 concurrent agent state read/write tasks and verify no deadlocks or corruption; SC-008: insert 10K+ task/message rows and verify query by agent/correlation completes within 100ms (requires `DATABASE_URL` + `NATS_URL`)
- [x] T049 Run `cargo clippy --workspace -- -D warnings` — fix any warnings introduced by the persistence crate
- [x] T050 Run `cargo doc -p mister-smith-persistence --no-deps` — ensure all public items have doc comments, no broken links
- [x] T051 Run `cargo test --workspace` — verify all tests pass (existing 717 + new persistence tests), no regressions
- [x] T052 Update `CLAUDE.md` — mark Phase 6 complete in Implementation Status table, add `mister-smith-persistence` to crate dependency tree, update test count
- [x] T053 Update `VERSION_REFERENCE.md` — add sqlx 0.8.6 and chrono 0.4 to the dependency matrix

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup (T001-T006) — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational (T007-T015) — MVP target
- **US2 (Phase 4)**: Depends on US1 (builds on HybridStateManager from T020)
- **US3 (Phase 5)**: Depends on Foundational only (migration files from T011, runner from T012)
- **US4 (Phase 6)**: Depends on Foundational only (PG pool from T010, migrations from T011)
- **US5 (Phase 7)**: Depends on US1 (repository trait from T021) and US4 (task/message repos)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

```
Phase 1: Setup
    ↓
Phase 2: Foundational
    ↓
    ├── Phase 3: US1 (State Recovery) ← MVP
    │       ↓
    │   Phase 4: US2 (Fast KV Access) — extends US1's HybridStateManager
    │
    ├── Phase 5: US3 (Migrations) — parallel with US1
    │
    ├── Phase 6: US4 (Tasks/Messages) — parallel with US1
    │       ↓
    └── Phase 7: US5 (Repository Layer) — needs US1 + US4
            ↓
        Phase 8: Polish
```

### Within Each User Story

- Models/types before service logic
- Service logic before repository implementation
- Repository before integration tests
- Unit tests alongside implementation (inline `#[cfg(test)]`)

### Parallel Opportunities

**Phase 1** (all parallel):
- T005 (config.rs) + T006 (error.rs)

**Phase 2** (partial parallel):
- T007 (core errors) + T008 (config types) — different crates, parallel
- T009 (postgres/mod.rs) + T013 (kv/mod.rs) + T015 (health.rs) — different modules, parallel
- T010 (pool.rs) and T014 (buckets.rs) — after their mod.rs files, parallel

**Phase 3 US1**:
- T016 (kv/state.rs) + T017 (queries.rs) — different modules, parallel
- T024 (postgres_tests.rs) + T025 (kv_tests.rs) — different test files, parallel

**Phase 6 US4**:
- T035 (task queries) + T036 (message queries) — same file but additive, parallel

**Phase 8**:
- T046 (integration tests) + T047 (Cargo.toml) + T048 (performance tests) — parallel
- T049 (clippy) + T050 (docs) — parallel

---

## Parallel Example: User Story 1

```bash
# After Phase 2 completes, launch these in parallel:
Agent 1: "T016 — Create KV StateManager in src/kv/state.rs"
Agent 2: "T017 — Create PG query helpers in src/postgres/queries.rs"

# After T016+T017 complete:
Agent 1: "T019 — Create DataRouter in src/hybrid/router.rs"
# Then T020 (HybridStateManager), T021 (Repository trait), T022 (AgentRepository)

# After implementation complete, tests in parallel:
Agent 1: "T024 — PostgreSQL integration tests"
Agent 2: "T025 — JetStream KV integration tests"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T006)
2. Complete Phase 2: Foundational (T007-T015)
3. Complete Phase 3: User Story 1 (T016-T025)
4. **STOP and VALIDATE**: Agent state persists and recovers across restarts
5. Run `cargo test --workspace` — no regressions

### Incremental Delivery

1. Setup + Foundational → Framework compiles with persistence crate
2. US1 (State Recovery) → Agents survive restarts (MVP!)
3. US2 (Fast KV Access) → Performance + graceful degradation
4. US3 (Migrations) + US4 (Tasks/Messages) → Full schema + queryable history (parallel)
5. US5 (Repository Layer) → Clean abstraction + audit wiring
6. Polish → Integration tests, docs, workspace validation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Env-gated tests use `DATABASE_URL` and `NATS_URL` — skip when not set
- All SQL in migration files, not in Rust string literals (compile-time embedding via `sqlx::migrate!()`)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
