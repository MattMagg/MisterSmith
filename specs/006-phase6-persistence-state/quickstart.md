# Quickstart: Phase 6 — Persistence & State

## Prerequisites

- Rust 1.88.0+ (MSRV)
- PostgreSQL 15+ running locally or via Docker
- NATS server with JetStream enabled (reuse existing Phase 4 setup)

### Docker Quick Setup

```bash
# PostgreSQL
docker run -d --name mister-smith-pg \
  -e POSTGRES_DB=mister_smith \
  -e POSTGRES_USER=mister_smith \
  -e POSTGRES_PASSWORD=dev_password \
  -p 5432:5432 \
  postgres:15

# NATS (if not already running)
docker start NATS  # Or: docker run -d --name NATS -p 4222:4222 nats:latest -js
```

## Build

```bash
# Build just the persistence crate
cargo build -p mister-smith-persistence

# Build full workspace
cargo build --workspace
```

## Run Migrations

```bash
# Set database URL
export DATABASE_URL="postgres://mister_smith:dev_password@localhost:5432/mister_smith"

# Run migrations (via sqlx CLI)
cargo sqlx migrate run --source crates/mister-smith-persistence/migrations

# Check migration status
cargo sqlx migrate info --source crates/mister-smith-persistence/migrations
```

## Run Tests

```bash
# Unit tests only (no external deps)
cargo test -p mister-smith-persistence

# Integration tests (requires PostgreSQL + NATS)
export DATABASE_URL="postgres://mister_smith:dev_password@localhost:5432/mister_smith"
export NATS_URL="nats://localhost:4222"
cargo test -p mister-smith-persistence -- --ignored

# Full workspace
cargo test --workspace
```

## Usage Example

```rust
use mister_smith_persistence::{
    PostgresConnection, KvBucketManager, HybridStateManager,
    AgentRepository, MigrationRunner,
};

// 1. Connect to PostgreSQL
let pool = PostgresConnection::connect("postgres://...").await?;

// 2. Run migrations
let runner = MigrationRunner::new(pool.clone());
runner.run().await?;

// 3. Initialize KV buckets
let nats_client = async_nats::connect("nats://localhost:4222").await?;
let js_context = async_nats::jetstream::new(nats_client);
let kv_manager = KvBucketManager::new(js_context, config.kv);
kv_manager.initialize_buckets().await?;

// 4. Create hybrid state manager
let hybrid = HybridStateManager::new(
    kv_manager.bucket("AGENT_STATE")?.clone(),
    pool.clone(),
    config.flush,
);

// 5. Use repositories
let agent_repo = AgentRepository::new(Arc::new(hybrid), pool);
agent_repo.save(&agent_record).await?;
let found = agent_repo.find(&agent_id).await?;
```

## Configuration

Add to your TOML config:

```toml
[persistence]
enabled = true

[persistence.postgres]
url = "postgres://user:pass@localhost:5432/mister_smith"
max_connections = 10
min_connections = 2
connect_timeout_secs = 30
idle_timeout_secs = 600

[persistence.kv]
enabled = true
# Bucket TTLs in seconds
session_ttl = 3600    # 1 hour
agent_state_ttl = 1800 # 30 minutes
cache_ttl = 300       # 5 minutes

[persistence.flush]
threshold = 50         # Flush after N dirty keys
deadline_secs = 60     # Or flush after N seconds
safety_margin_secs = 300 # Flush before TTL - this margin
```

## Key Files

| File | Purpose |
|------|---------|
| `crates/mister-smith-persistence/src/lib.rs` | Crate root, public API |
| `crates/mister-smith-persistence/src/postgres/pool.rs` | PostgreSQL connection + Resource impl |
| `crates/mister-smith-persistence/src/kv/buckets.rs` | JetStream KV bucket management |
| `crates/mister-smith-persistence/src/hybrid/manager.rs` | Dual-store orchestration |
| `crates/mister-smith-persistence/src/repository/` | Entity-specific repositories |
| `crates/mister-smith-persistence/migrations/` | SQL migration files |
