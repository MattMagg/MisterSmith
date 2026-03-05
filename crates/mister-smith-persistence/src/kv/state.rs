//! Typed state operations with conflict resolution on a single KV bucket.
//!
//! [`StateManager`] wraps an async-nats JetStream [`kv::Store`] and provides
//! typed get/save/update/delete operations with JSON serialization and
//! configurable conflict resolution via [`ConflictStrategy`].
//!
//! All values are serialized to JSON bytes before writing and deserialized
//! from JSON bytes on read. The [`update`](StateManager::update) method always
//! uses compare-and-swap (CAS) regardless of the configured conflict strategy.

use async_nats::jetstream::kv;
use bytes::Bytes;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, warn};

use crate::error::{from_kv_error, from_kv_version_error, PersistenceError};

/// Conflict resolution strategy for save operations.
///
/// Controls what happens when [`StateManager::save`] is called for a key
/// that already exists in the bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Unconditionally overwrite existing values.
    LastWriteWins,
    /// Same as `LastWriteWins` — the KV server records creation timestamps,
    /// so ordering is implicit. Provided as a semantic marker for callers
    /// that reason about wall-clock ordering.
    Timestamp,
    /// Reject the write if the key already exists, returning
    /// [`PersistenceError::DuplicateKey`].
    Reject,
}

/// The kind of mutation that produced a [`StateChange`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// A value was written (created or updated).
    Put,
    /// A value was soft-deleted (tombstone marker).
    Delete,
    /// A value was permanently removed (all revisions erased).
    Purge,
}

/// Describes a single state mutation observed on a KV bucket.
///
/// Used by the watch layer (T026) to surface change notifications.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateChange {
    /// The key that was modified.
    pub key: String,
    /// The kind of operation that occurred.
    pub operation: Operation,
    /// The revision number assigned by the KV server after this operation.
    pub revision: u64,
}

/// Typed state operations backed by a JetStream KV bucket.
///
/// Wraps a single [`kv::Store`] and provides ergonomic typed access with
/// JSON serialization and configurable conflict resolution on writes.
///
/// # Conflict resolution
///
/// The [`ConflictStrategy`] only affects [`save`](Self::save):
///
/// | Strategy | Behavior |
/// |----------|----------|
/// | `LastWriteWins` | Unconditional `put` — overwrites any existing value |
/// | `Timestamp` | Same as `LastWriteWins` (server tracks ordering) |
/// | `Reject` | Checks for existence first; returns `DuplicateKey` if the key is present |
///
/// [`update`](Self::update) always uses compare-and-swap regardless of strategy.
pub struct StateManager {
    store: kv::Store,
    strategy: ConflictStrategy,
}

impl StateManager {
    /// Create a new `StateManager` over the given KV bucket.
    pub fn new(store: kv::Store, strategy: ConflictStrategy) -> Self {
        Self { store, strategy }
    }

    /// Persist a value under `key`, respecting the configured [`ConflictStrategy`].
    ///
    /// Returns the revision number assigned by the KV server on success.
    ///
    /// # Errors
    ///
    /// - [`PersistenceError::DuplicateKey`] if the strategy is `Reject` and the key exists.
    /// - [`PersistenceError::SerializationFailed`] if JSON serialization fails.
    /// - Other `PersistenceError` variants on transport or server errors.
    pub async fn save<T: Serialize + Send>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<u64, PersistenceError> {
        let json = serde_json::to_vec(value)
            .map_err(|e| PersistenceError::SerializationFailed(e.to_string()))?;

        // Reject strategy: check for existing key before writing.
        if self.strategy == ConflictStrategy::Reject {
            match self.store.entry(key).await {
                Ok(Some(entry)) if entry.operation == kv::Operation::Put => {
                    debug!(key = %key, "Reject strategy: key already exists at revision {}", entry.revision);
                    return Err(PersistenceError::DuplicateKey(format!(
                        "Key '{key}' already exists (revision {})",
                        entry.revision,
                    )));
                }
                Ok(_) => {
                    // Key does not exist or has been deleted — safe to write.
                }
                Err(err) => {
                    warn!(key = %key, error = %err, "Failed to check key existence");
                    return Err(from_kv_error(err));
                }
            }
        }

        let revision = self
            .store
            .put(key, Bytes::from(json))
            .await
            .map_err(from_kv_error)?;

        debug!(key = %key, revision = revision, "State saved");
        Ok(revision)
    }

    /// Read and deserialize the value at `key`.
    ///
    /// Returns `Ok(None)` if the key does not exist or has been deleted.
    ///
    /// # Errors
    ///
    /// - [`PersistenceError::SerializationFailed`] if JSON deserialization fails.
    /// - Other `PersistenceError` variants on transport or server errors.
    pub async fn get<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, PersistenceError> {
        let maybe_bytes = self.store.get(key).await.map_err(from_kv_error)?;

        match maybe_bytes {
            Some(bytes) => {
                let value: T = serde_json::from_slice(&bytes)
                    .map_err(|e| PersistenceError::SerializationFailed(e.to_string()))?;
                debug!(key = %key, "State retrieved");
                Ok(Some(value))
            }
            None => {
                debug!(key = %key, "Key not found");
                Ok(None)
            }
        }
    }

    /// Update a key using compare-and-swap (optimistic concurrency control).
    ///
    /// The write succeeds only if the current server-side revision matches
    /// `expected_revision`. This is enforced regardless of the configured
    /// [`ConflictStrategy`].
    ///
    /// Returns the new revision number on success.
    ///
    /// # Errors
    ///
    /// - [`PersistenceError::VersionConflict`] if the server-side revision differs.
    /// - [`PersistenceError::SerializationFailed`] if JSON serialization fails.
    /// - Other `PersistenceError` variants on transport or server errors.
    pub async fn update<T: Serialize + Send>(
        &self,
        key: &str,
        value: &T,
        expected_revision: u64,
    ) -> Result<u64, PersistenceError> {
        let json = serde_json::to_vec(value)
            .map_err(|e| PersistenceError::SerializationFailed(e.to_string()))?;

        let revision = self
            .store
            .update(key, Bytes::from(json), expected_revision)
            .await
            .map_err(|e| from_kv_version_error(key, expected_revision, e))?;

        debug!(key = %key, old_revision = expected_revision, new_revision = revision, "State updated (CAS)");
        Ok(revision)
    }

    /// Delete a key from the bucket (soft delete — sets a tombstone marker).
    ///
    /// # Errors
    ///
    /// - `PersistenceError` variants on transport or server errors.
    pub async fn delete(&self, key: &str) -> Result<(), PersistenceError> {
        self.store.delete(key).await.map_err(from_kv_error)?;
        debug!(key = %key, "State deleted");
        Ok(())
    }

    /// Returns a reference to the underlying KV store.
    ///
    /// Useful for callers that need direct access for advanced operations
    /// (e.g. watch, purge, status) not exposed by `StateManager`.
    pub fn store(&self) -> &kv::Store {
        &self.store
    }

    /// Returns the configured conflict strategy.
    pub fn strategy(&self) -> ConflictStrategy {
        self.strategy
    }

    // Watch support is implemented in the watch module (T026).
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conflict_strategy_variants() {
        // Verify all three variants exist and are distinct.
        let lww = ConflictStrategy::LastWriteWins;
        let ts = ConflictStrategy::Timestamp;
        let reject = ConflictStrategy::Reject;

        assert_ne!(lww, ts);
        assert_ne!(lww, reject);
        assert_ne!(ts, reject);

        // Verify Copy + Clone.
        let copied = lww;
        assert_eq!(copied, ConflictStrategy::LastWriteWins);
    }

    #[test]
    fn operation_variants() {
        // Verify all three variants exist and are distinct.
        let put = Operation::Put;
        let delete = Operation::Delete;
        let purge = Operation::Purge;

        assert_ne!(put, delete);
        assert_ne!(put, purge);
        assert_ne!(delete, purge);

        // Verify Copy + Clone.
        let copied = put;
        assert_eq!(copied, Operation::Put);
    }

    #[test]
    fn state_change_construction() {
        let change = StateChange {
            key: "agent.foo.state".to_string(),
            operation: Operation::Put,
            revision: 42,
        };

        assert_eq!(change.key, "agent.foo.state");
        assert_eq!(change.operation, Operation::Put);
        assert_eq!(change.revision, 42);
    }

    #[test]
    fn state_change_clone_and_eq() {
        let original = StateChange {
            key: "session.123".to_string(),
            operation: Operation::Delete,
            revision: 7,
        };
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn json_serialization_round_trip() {
        // Verify that values serialize to valid JSON and can be deserialized back.
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct AgentState {
            agent_id: String,
            status: String,
            version: u32,
        }

        let state = AgentState {
            agent_id: "agent-001".to_string(),
            status: "running".to_string(),
            version: 3,
        };

        // Serialize (same path as StateManager::save)
        let json_bytes = serde_json::to_vec(&state).expect("serialization should succeed");

        // Verify it's valid JSON
        let raw_value: serde_json::Value =
            serde_json::from_slice(&json_bytes).expect("should be valid JSON");
        assert_eq!(raw_value["agent_id"], "agent-001");
        assert_eq!(raw_value["status"], "running");
        assert_eq!(raw_value["version"], 3);

        // Deserialize (same path as StateManager::get)
        let deserialized: AgentState =
            serde_json::from_slice(&json_bytes).expect("deserialization should succeed");
        assert_eq!(deserialized, state);
    }

    #[test]
    fn json_serialization_with_bytes() {
        // Verify the Bytes conversion used by StateManager works correctly.
        use bytes::Bytes;

        let value = vec![1u32, 2, 3, 4, 5];
        let json = serde_json::to_vec(&value).expect("serialization should succeed");
        let bytes = Bytes::from(json.clone());

        // Deserialize from Bytes (as StateManager::get does via store.get())
        let deserialized: Vec<u32> =
            serde_json::from_slice(&bytes).expect("deserialization from Bytes should succeed");
        assert_eq!(deserialized, value);
    }

    #[test]
    fn conflict_strategy_debug() {
        // Ensure Debug is derived — useful for logging.
        let formatted = format!("{:?}", ConflictStrategy::Reject);
        assert_eq!(formatted, "Reject");
    }

    #[test]
    fn operation_debug() {
        assert_eq!(format!("{:?}", Operation::Put), "Put");
        assert_eq!(format!("{:?}", Operation::Delete), "Delete");
        assert_eq!(format!("{:?}", Operation::Purge), "Purge");
    }

    #[test]
    fn state_change_debug() {
        let change = StateChange {
            key: "k".to_string(),
            operation: Operation::Purge,
            revision: 0,
        };
        let formatted = format!("{:?}", change);
        assert!(formatted.contains("Purge"));
        assert!(formatted.contains("revision: 0"));
    }
}
