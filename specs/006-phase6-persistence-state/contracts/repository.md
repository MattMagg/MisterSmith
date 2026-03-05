# Contract: Repository Trait

## Public API

```rust
/// Generic repository for entity persistence.
/// Implementations route to KV, SQL, or both based on data type.
#[async_trait]
pub trait Repository<T: Send + Sync>: Send + Sync {
    /// Persist an entity. Returns the saved entity (may include generated fields).
    async fn save(&self, entity: &T) -> Result<T, PersistenceError>;

    /// Find an entity by its primary identifier.
    async fn find(&self, id: &Uuid) -> Result<Option<T>, PersistenceError>;

    /// Update an existing entity. Returns the updated entity.
    /// Fails with VersionConflict if the entity was modified concurrently.
    async fn update(&self, entity: &T) -> Result<T, PersistenceError>;

    /// Delete an entity by its primary identifier.
    /// Returns true if the entity existed and was deleted.
    async fn delete(&self, id: &Uuid) -> Result<bool, PersistenceError>;
}
```

## Concrete Repositories

### AgentRepository

```rust
impl AgentRepository {
    /// Create from dual-store manager.
    pub fn new(hybrid: Arc<HybridStateManager>, pool: PgPool) -> Self;

    /// Find agents by type.
    pub async fn find_by_type(&self, agent_type: &str) -> Result<Vec<AgentRecord>, PersistenceError>;

    /// Find agents by status.
    pub async fn find_by_status(&self, status: &str) -> Result<Vec<AgentRecord>, PersistenceError>;

    /// Save agent state key-value pair (routed through hybrid manager).
    pub async fn save_state(&self, agent_id: Uuid, key: &str, value: serde_json::Value) -> Result<(), PersistenceError>;

    /// Get agent state (KV first, SQL fallback).
    pub async fn get_state(&self, agent_id: Uuid, key: &str) -> Result<Option<serde_json::Value>, PersistenceError>;

    /// Get all state keys for an agent.
    pub async fn get_all_state(&self, agent_id: Uuid) -> Result<Vec<(String, serde_json::Value)>, PersistenceError>;

    /// Create a checkpoint of agent's current state.
    pub async fn checkpoint(&self, agent_id: Uuid) -> Result<Uuid, PersistenceError>;

    /// Hydrate agent state from SQL into KV on startup.
    pub async fn hydrate(&self, agent_id: Uuid) -> Result<usize, PersistenceError>;
}
```

### TaskRepository

```rust
impl TaskRepository {
    pub fn new(pool: PgPool) -> Self;

    /// Query tasks by agent and optional status filter.
    pub async fn find_by_agent(
        &self,
        agent_id: Uuid,
        status: Option<&str>,
    ) -> Result<Vec<TaskRecord>, PersistenceError>;

    /// Query tasks by time range.
    pub async fn find_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TaskRecord>, PersistenceError>;

    /// Query tasks by correlation ID (full conversation chain).
    pub async fn find_by_correlation(
        &self,
        correlation_id: Uuid,
    ) -> Result<Vec<TaskRecord>, PersistenceError>;
}
```

### MessageRepository

```rust
impl MessageRepository {
    pub fn new(pool: PgPool) -> Self;

    /// Query messages by correlation ID (conversation thread).
    pub async fn find_by_correlation(
        &self,
        correlation_id: Uuid,
    ) -> Result<Vec<MessageRecord>, PersistenceError>;

    /// Query messages sent by an agent in a time range.
    pub async fn find_by_sender(
        &self,
        agent_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<MessageRecord>, PersistenceError>;
}
```

### AuditRepository

```rust
impl AuditRepository {
    pub fn new(pool: PgPool) -> Self;

    /// Append an audit event (insert-only).
    pub async fn append(&self, entry: &AuditEntry) -> Result<(), PersistenceError>;

    /// Batch append audit events (used by flush from Phase 5 ring buffer).
    pub async fn append_batch(&self, entries: &[AuditEntry]) -> Result<usize, PersistenceError>;

    /// Query audit events by agent and time range.
    pub async fn find_by_agent(
        &self,
        agent_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<AuditEntry>, PersistenceError>;
}
```

## Behavioral Contract

1. **Read path**: KV first → SQL fallback → hydrate KV on miss
2. **Write path**: KV immediate → dirty tracking → async flush to SQL
3. **Concurrency**: Optimistic — version field incremented on each write; `VersionConflict` on stale version
4. **Transactions**: SQL-level atomicity for multi-entity operations via `sqlx::Transaction`
5. **Errors**: All methods return `Result<_, PersistenceError>` — never panic, never swallow
