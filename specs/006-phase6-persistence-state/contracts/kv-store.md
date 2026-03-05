# Contract: KV Store Operations

## Public API

### KvBucketManager

```rust
/// Manages JetStream KV bucket lifecycle.
pub struct KvBucketManager {
    // Constructed from JetStreamManager context
}

impl KvBucketManager {
    /// Create from a JetStream context.
    pub fn new(context: jetstream::Context, config: KvConfig) -> Self;

    /// Initialize all configured buckets (session, agent-state, cache).
    pub async fn initialize_buckets(&self) -> Result<(), PersistenceError>;

    /// Get a named bucket.
    pub fn bucket(&self, name: &str) -> Result<&kv::Store, PersistenceError>;

    /// Health check: verify all buckets are accessible.
    pub async fn health_check(&self) -> Result<HealthStatus, PersistenceError>;
}
```

### StateManager

```rust
/// Typed state operations with conflict resolution on a single KV bucket.
pub struct StateManager {
    // Wraps a kv::Store with conflict strategy
}

impl StateManager {
    pub fn new(store: kv::Store, strategy: ConflictStrategy) -> Self;

    /// Save state with optimistic concurrency.
    pub async fn save<T: Serialize + Send>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<u64, PersistenceError>;  // Returns revision

    /// Get state, deserializing to T.
    pub async fn get<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, PersistenceError>;

    /// CAS update: fails with VersionConflict if revision doesn't match.
    pub async fn update<T: Serialize + Send>(
        &self,
        key: &str,
        value: &T,
        expected_revision: u64,
    ) -> Result<u64, PersistenceError>;

    /// Delete a key.
    pub async fn delete(&self, key: &str) -> Result<(), PersistenceError>;

    /// Watch a key pattern for changes.
    pub async fn watch(
        &self,
        pattern: &str,
    ) -> Result<impl Stream<Item = StateChange>, PersistenceError>;
}

pub enum ConflictStrategy {
    LastWriteWins,
    Timestamp,
    Reject,
}

pub struct StateChange {
    pub key: String,
    pub operation: Operation,  // Put, Delete, Purge
    pub revision: u64,
}
```

## Behavioral Contract

1. **Bucket initialization**: Idempotent — calling `initialize_buckets()` on existing buckets is a no-op
2. **Serialization**: All values serialized as JSON bytes via serde
3. **Conflict resolution**: Applied on `save()` when key already exists; `update()` always uses CAS regardless of strategy
4. **Timeouts**: All operations have a configurable timeout (default 10s); returns `PersistenceError::ConnectionFailed` on timeout
5. **TTL**: Managed by bucket config (`max_age`), not per-key — all keys in a bucket share the same TTL
