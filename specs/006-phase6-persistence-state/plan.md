# Implementation Plan: Phase 6 — Persistence & State

**Branch**: `006-phase6-persistence-state` | **Date**: 2026-03-05 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-phase6-persistence-state/spec.md`

## Summary

Phase 6 adds durable storage to the framework through a new `mister-smith-persistence` crate. It implements a dual-store architecture: PostgreSQL (via sqlx 0.8) for authoritative relational persistence, and JetStream KV (via existing async-nats 0.46) for fast distributed state. A repository abstraction routes reads/writes to the appropriate backend, with dirty-key tracking, configurable flush thresholds, state hydration on startup, and graceful degradation when a backend is unavailable.

## Technical Context

**Language/Version**: Rust, MSRV 1.88.0
**Primary Dependencies**: sqlx 0.8.6 (new), async-nats 0.46.0 (existing), tokio 1.49.0 (existing), serde 1.x (existing)
**Storage**: PostgreSQL 15+ (relational), JetStream KV (distributed ephemeral)
**Testing**: cargo test + env-gated integration tests (requires PostgreSQL and NATS)
**Target Platform**: Linux server (same as framework)
**Project Type**: Library crate (workspace member)
**Performance Goals**: KV reads <1ms local / <5ms distributed p95; SQL queries <100ms for 1M rows
**Constraints**: No blocking I/O on tokio runtime; all errors typed and propagated
**Scale/Scope**: 100 concurrent agents, 1M row tables, 8 partitioned high-volume tables

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Canonical Single Source | PASS | `PersistenceError` already defined in `mister-smith-core/src/error.rs:208`. New types will be added there, not redefined. |
| II. Spec-First Design | PASS | 7 spec files exist under `spec/data-management/`. This plan traces to them. |
| III. Phase-Gated Build Order | PASS | Phase 5 complete (717 tests). Gate 5 criteria satisfied. Phase 6 depends on Phases 2, 4, 5 — all complete. |
| IV. Model-Agnostic Architecture | PASS | Persistence is model-agnostic — stores agent state, not LLM-specific data. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Persistence integrates with supervision via health checks and graceful degradation — does not modify actor model. |
| VI. Evidence-Based Validation | PASS | Gate criteria will use `cargo test`, `cargo clippy`, migration execution, and health check verification. |
| VII. Explicit Dependency Management | PASS | sqlx added to workspace Cargo.toml; VERSION_REFERENCE.md updated. |

No violations. No complexity justifications needed.

## Project Structure

### Documentation (this feature)

```text
specs/006-phase6-persistence-state/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 research findings
├── data-model.md        # Phase 1 entity/relationship model
├── quickstart.md        # Phase 1 developer quickstart
├── contracts/           # Phase 1 public API contracts
│   ├── repository.md    # Repository trait contract
│   ├── kv-store.md      # KV store operations contract
│   └── migrations.md    # Migration system contract
└── tasks.md             # Phase 2 task breakdown (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/mister-smith-persistence/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Crate root: module declarations, re-exports
│   ├── error.rs            # PersistenceError re-export + sqlx conversion helpers
│   ├── config.rs           # PersistenceConfig, PostgresConfig, KvConfig
│   ├── postgres/
│   │   ├── mod.rs          # PostgreSQL module root
│   │   ├── pool.rs         # PostgresConnection implementing Resource trait
│   │   ├── migrations.rs   # sqlx migration runner + version tracking
│   │   └── queries.rs      # Prepared query helpers (agents, tasks, messages, audit)
│   ├── kv/
│   │   ├── mod.rs          # JetStream KV module root
│   │   ├── buckets.rs      # Bucket management (session, agent-state, cache tiers)
│   │   ├── state.rs        # StateManager (conflict resolution, OCC)
│   │   └── watch.rs        # KV watcher for change notifications
│   ├── hybrid/
│   │   ├── mod.rs          # Dual-store module root
│   │   ├── manager.rs      # HybridStateManager (dirty tracking, flush, hydration)
│   │   └── router.rs       # DataRouter (type-based storage selection)
│   ├── repository/
│   │   ├── mod.rs          # Repository trait + implementations
│   │   ├── agent.rs        # AgentRepository (registry, state, checkpoints)
│   │   ├── task.rs         # TaskRepository (task lifecycle CRUD)
│   │   ├── message.rs      # MessageRepository (message history, correlation queries)
│   │   └── audit.rs        # AuditRepository (audit log persistence from Phase 5 ring buffer)
│   └── health.rs           # Health check implementations for PG + KV
├── migrations/
│   ├── 00001_initial_schema.sql
│   ├── 00002_indexes.sql
│   ├── 00003_partitions.sql
│   └── 00004_audit_schema.sql
└── tests/
    ├── postgres_tests.rs    # Env-gated PostgreSQL integration tests
    ├── kv_tests.rs          # Env-gated JetStream KV tests
    ├── hybrid_tests.rs      # Dual-store behavior tests
    ├── repository_tests.rs  # Repository CRUD + concurrency tests
    └── migration_tests.rs   # Schema migration tests

# Modified existing crates:
crates/mister-smith-core/src/error.rs           # Expand PersistenceError variants
crates/mister-smith-config/src/types.rs          # Add PersistenceConfig to FrameworkConfig
Cargo.toml                                       # Add sqlx to workspace deps, new crate member
```

**Structure Decision**: Single new crate `mister-smith-persistence` following the established pattern (one crate per domain). PostgreSQL and KV are submodules, not separate crates, because they share the hybrid manager and repository layer. Feature flags gate PostgreSQL (`sqlx` feature) so consumers without a database can still use KV-only mode.

## Design Decisions

### D1: Single Crate vs Multiple Crates

**Decision**: Single `mister-smith-persistence` crate.
**Rationale**: The dual-store pattern requires tight coupling between PG and KV (dirty tracking, flush, hydration). Splitting into two crates would force the hybrid manager into a third crate. One crate with feature flags (`sqlx` for PG) keeps it simple while allowing KV-only builds.

### D2: sqlx Native Migrations (not refinery, not flyway)

**Decision**: Use `sqlx::migrate!()` macro.
**Rationale**: sqlx is already the database driver. Its migration framework is integrated (no additional dependency), supports compile-time verification, and the spec recommends it. Rollback via down-migration SQL files.

### D3: Env-Gated Integration Tests

**Decision**: PostgreSQL tests require `DATABASE_URL` env var; KV tests require `NATS_URL` env var. Tests are `#[ignore]` by default, enabled via env.
**Rationale**: Follows the pattern established for NATS tests in Phase 4. Developers without a running database can still run the full unit test suite.

### D4: PersistenceError Expansion in Core

**Decision**: Expand `PersistenceError` in `mister-smith-core/src/error.rs` with new variants (NotFound, DuplicateKey, VersionConflict, ConnectionFailed, TtlExpired, MigrationFailed).
**Rationale**: Constitution Principle I — canonical single source. The error enum lives in core; child crates re-export it.

### D5: Repository Pattern with Generic Trait

**Decision**: Define `Repository<T>` trait with `save`, `find`, `update`, `delete` methods. Concrete implementations per entity type.
**Rationale**: Spec FR-008 requires a storage-agnostic abstraction. The trait hides dual-store routing from consumers.

### D6: Flush Safety — Flush Before TTL

**Decision**: Dirty entries are tracked with a flush deadline set to `min(flush_threshold_time, kv_ttl - safety_margin)`. A background task flushes on whichever deadline comes first.
**Rationale**: Edge case from spec: "dirty entries must be flushed before TTL expiration." The safety margin prevents data loss from TTL expiration racing the flush.

### D7: Audit Log Persistence Wiring

**Decision**: Add an `AuditPersister` that periodically drains the Phase 5 `AuditLogger` in-memory ring buffer and batch-writes entries to PostgreSQL via the audit repository's `append_batch()`.
**Rationale**: Phase 5 audit logging uses an in-memory ring buffer (`parking_lot::RwLock`). This phase adds durable persistence by polling/draining that buffer without modifying the Phase 5 audit API.

## Dependency Changes

### New Workspace Dependencies (Cargo.toml)

```toml
sqlx = { version = "0.8", features = ["postgres", "runtime-tokio-rustls", "uuid", "json", "chrono", "migrate"] }
chrono = { version = "0.4", features = ["serde"] }
```

### New Crate (Cargo.toml members)

```toml
"crates/mister-smith-persistence"
```

### Crate Dependencies (mister-smith-persistence/Cargo.toml)

```toml
[dependencies]
mister-smith-core = { workspace = true }
mister-smith-config = { workspace = true }
mister-smith-resources = { workspace = true }
mister-smith-nats = { workspace = true }
mister-smith-security = { workspace = true, optional = true }
mister-smith-events = { workspace = true }
mister-smith-async = { workspace = true }

sqlx = { workspace = true, optional = true }
async-nats = { workspace = true }
chrono = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }
uuid = { workspace = true }
bytes = { workspace = true }

[features]
default = ["sqlx", "security"]
sqlx = ["dep:sqlx"]
security = ["dep:mister-smith-security"]

[dev-dependencies]
tokio = { workspace = true, features = ["test-util"] }
tempfile = "3"
```

## Integration Points

### Phase 2 Integration

- `PostgresConnection` (wrapping `PgPool`) registered with `ResourceManager` via `Resource` trait — no `ConnectionPool` wrapper (avoids pool-over-pool per research.md R2)
- `CircuitBreaker` from `mister-smith-async` wraps PG and KV operations
- `RetryPolicy` from `mister-smith-async` for transient failures
- `EventBus` from `mister-smith-events` for persistence change notifications

### Phase 4 Integration

- `NatsTransport::inner_client()` → `JetStreamManager` → KV bucket access
- Reuse existing `JetStreamConfig` for KV bucket configuration

### Phase 5 Integration

- `AuditLogger` events persisted to PostgreSQL audit table
- Credentials from `SecurityConfig` or environment variables for PG connection
- Security feature-gated — persistence works without security crate

## Complexity Tracking

No constitution violations. No complexity justifications needed.
