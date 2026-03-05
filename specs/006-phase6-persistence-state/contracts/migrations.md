# Contract: Migration System

## Public API

```rust
/// Runs database migrations and reports status.
pub struct MigrationRunner {
    // Wraps sqlx migration infrastructure
}

impl MigrationRunner {
    /// Create from a PgPool.
    pub fn new(pool: PgPool) -> Self;

    /// Run all pending migrations. Returns count of applied migrations.
    pub async fn run(&self) -> Result<usize, PersistenceError>;

    /// Check current migration version without applying anything.
    pub async fn current_version(&self) -> Result<Option<i64>, PersistenceError>;

    /// List all migrations and their applied status.
    pub async fn status(&self) -> Result<Vec<MigrationStatus>, PersistenceError>;

    /// Verify all migrations have been applied (health check use).
    pub async fn verify(&self) -> Result<bool, PersistenceError>;
}

pub struct MigrationStatus {
    pub version: i64,
    pub description: String,
    pub applied: bool,
    pub applied_at: Option<DateTime<Utc>>,
    pub checksum: String,
}
```

## Migration File Layout

```
migrations/
├── 00001_initial_schema.sql     # Schemas, domain types, core tables
├── 00002_indexes.sql            # Performance indexes
├── 00003_partitions.sql         # Time-based partitioning setup
└── 00004_audit_schema.sql       # Audit log tables
```

## Behavioral Contract

1. **Idempotent**: Running migrations on an already-migrated database is a no-op
2. **Ordered**: Migrations execute in version order, never out of sequence
3. **Tracked**: Applied migrations recorded in `_sqlx_migrations` table with checksum
4. **Compile-time embedded**: Migrations baked into binary via `sqlx::migrate!()` macro
5. **Failure behavior**: If a migration fails, the database remains at its pre-migration state; `MigrationFailed` error returned with details
