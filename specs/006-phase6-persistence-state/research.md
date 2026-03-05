# Research: Phase 6 — Persistence & State

**Date**: 2026-03-05
**Branch**: `006-phase6-persistence-state`

## R1: sqlx 0.8 Migration API

**Decision**: Use `sqlx::migrate!("./migrations")` compile-time macro with plain SQL migration files.

**Rationale**: sqlx's built-in migration system embeds migrations at compile time, supports `up.sql` / `down.sql` per version, tracks applied migrations in a `_sqlx_migrations` table, and integrates with `cargo sqlx` CLI. No additional dependency needed.

**Alternatives considered**:
- **refinery**: Lighter, but adds a dependency for functionality sqlx already provides.
- **flyway**: JVM-based, doesn't fit a Rust-native workflow.
- **diesel migrations**: Would require diesel as a dependency alongside sqlx.

**Key API details (sqlx 0.8.6)**:
- `sqlx::migrate!("./migrations").run(&pool).await` — runs all pending migrations
- Migration files: `migrations/{version}_{description}.sql` (e.g., `00001_initial_schema.sql`)
- Reversible migrations: `migrations/{version}_{description}.up.sql` / `.down.sql`
- `sqlx::migrate!()` is compile-time — migrations are embedded in the binary
- The `_sqlx_migrations` table tracks version, description, checksum, execution time

## R2: sqlx Connection Pool Integration with Resource Trait

**Decision**: Wrap `sqlx::PgPool` in a `PostgresConnection` struct that implements the core `Resource` trait. Do not replace `ConnectionPool<R>` — use `PgPool`'s native pooling directly since sqlx already provides robust pool management.

**Rationale**: sqlx's `PgPool` already handles connection pooling, health checks, idle timeout, and max connections. Wrapping it in our `ConnectionPool<R>` would add a pool-over-pool layer with no benefit. Instead, `PostgresConnection` holds a `PgPool` and implements `Resource` for lifecycle management (acquire = create pool, release = close pool, health_check = `pool.acquire()` test).

**Alternatives considered**:
- **ConnectionPool<PgConnection>**: Would manage individual connections through our pool. Rejected because sqlx's pool is battle-tested and handles connection lifecycle better than a generic wrapper.
- **Direct PgPool usage**: Would skip `Resource` trait integration. Rejected because `ResourceManager` registration enables framework-wide health monitoring.

**Key API details**:
- `PgPoolOptions::new().max_connections(n).connect(&url).await` — creates pool
- `pool.acquire().await` — gets a connection from pool
- `pool.close().await` — graceful shutdown
- `sqlx::query!()` / `sqlx::query_as!()` — compile-time checked queries (requires `DATABASE_URL` at compile time; use `sqlx::query()` runtime variant for flexibility)

## R3: JetStream KV API (async-nats 0.46)

**Decision**: Use the existing `JetStreamManager` in `mister-smith-nats` to access `jetstream::Context`, then use its KV methods directly. New KV management code lives in `mister-smith-persistence/src/kv/`.

**Rationale**: The NATS crate already provides JetStream access. KV buckets are created/managed through `jetstream::Context::create_key_value()`. The persistence crate consumes this API — it doesn't need to reimplement NATS client management.

**Key API details (async-nats 0.46 KV)**:
- `context.create_key_value(kv::Config { bucket, max_age, num_replicas, ... })` — create bucket
- `store.put(key, value.into()).await` — write (returns revision)
- `store.get(key).await` — read (returns `Option<Bytes>`, NOT Entry)
- `store.entry(key).await` — read with metadata (returns `Option<Entry>` with revision, operation)
- `store.update(key, value, revision).await` — CAS update (optimistic concurrency)
- `store.delete(key).await` — soft delete
- `store.watch(key_pattern).await` — returns Stream of changes
- Config: `max_age` (not `ttl`), `num_replicas` (not `replicas`), `history` for version count
- Error types: `PutError`, `EntryError`, `UpdateError`, `DeleteError`, `WatchError`

## R4: Hybrid State Flush Strategy

**Decision**: Background flush task with dual triggers: (1) dirty-key count exceeds threshold, (2) time-since-oldest-dirty-key exceeds deadline. Flush runs as a tokio::spawn'd task with circuit breaker protection.

**Rationale**: Count-only thresholds delay flushing low-volume agents. Time-only thresholds flush too eagerly for high-volume agents. Dual triggers handle both patterns. The deadline is set to `kv_ttl - safety_margin` to prevent data loss from TTL expiration.

**Alternatives considered**:
- **Count-only threshold (50 keys)**: Simple but risks data loss for agents with slow state mutation. A single key changed once might never flush.
- **Periodic timer (every N seconds)**: Simple but wasteful when nothing is dirty.
- **Write-through (flush every write)**: Defeats the purpose of the KV layer.

## R5: PersistenceError Expansion

**Decision**: Expand `PersistenceError` in `mister-smith-core` from 3 to 8 variants, covering the error taxonomy from FR-015.

**Current variants** (core/src/error.rs:208-218):
- `DatabaseFailed(String)`
- `SerializationFailed(String)`
- `DataCorrupted(String)`

**New variants to add**:
- `NotFound(String)` — entity/key not found
- `DuplicateKey(String)` — unique constraint violation
- `VersionConflict { key: String, expected: u64, actual: u64 }` — OCC failure
- `ConnectionFailed(String)` — backend unreachable
- `TtlExpired(String)` — KV entry expired before read
- `MigrationFailed(String)` — schema migration error

**Conversion helpers** (in persistence crate, not core — orphan rule):
- `from_sqlx_error(sqlx::Error) -> PersistenceError` — maps sqlx errors to domain errors
- `from_kv_error(...)` — maps async-nats KV errors to domain errors

## R6: Env-Gated Test Strategy

**Decision**: Integration tests gated behind environment variables, `#[ignore]` by default. CI runs with services provisioned.

**Pattern** (matches Phase 4 NATS test pattern):
```rust
fn postgres_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

#[tokio::test]
#[ignore] // Requires PostgreSQL: DATABASE_URL=postgres://...
async fn test_migration_runs_cleanly() {
    let url = match postgres_url() {
        Some(url) => url,
        None => return, // Skip if no database
    };
    // ...
}
```

**Test tiers**:
1. **Unit tests** (no external deps): Mock-based repository tests, config parsing, error mapping, data routing logic
2. **Integration tests** (env-gated): PG migrations, KV bucket CRUD, hybrid flush, repository end-to-end
3. **Cross-crate tests** (in `mister-smith-integration-tests`): Persistence + security audit wiring, persistence + events notification

## R7: Partition Management Strategy

**Decision**: SQL-defined partition management functions for time-based tables. Partitions created in migration scripts. Runtime partition creation deferred to Phase 8 operations.

**Rationale**: Partition management (creating future partitions, dropping old ones) is an operational concern. Phase 6 defines the partitioned table structure and initial partitions. Automated partition rotation is Phase 8 scope.

**Initial partitions** (in migration):
- Messages: monthly partitions for current + next 3 months
- Audit log: monthly partitions for current + next 3 months
- Metrics: monthly partitions for current + next 3 months
