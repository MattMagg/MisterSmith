//! Memory agent role — manages persistent memory and context.

use std::collections::HashMap;

use mister_smith_core::{Actor, AgentId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Messages handled by the [`MemoryAgent`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryMessage {
    /// Store a key-value pair.
    Store {
        /// The key to store under.
        key: String,
        /// The value to store.
        value: serde_json::Value,
    },
    /// Retrieve a value by key.
    Retrieve(String),
    /// Search for entries matching a key prefix.
    Search {
        /// The prefix to search for.
        prefix: String,
    },
    /// Delete an entry by key.
    Delete(String),
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Persistent state for the [`MemoryAgent`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryState {
    /// Number of entries currently stored.
    pub entry_count: u64,
    /// Key-value entries managed by this agent.
    pub entries: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by [`MemoryAgent`] operations.
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    /// A memory operation failed.
    #[error("memory error: {0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// Manages persistent memory and context — storage, retrieval, search,
/// and deletion of key-value entries.
pub struct MemoryAgent {
    id: AgentId,
}

impl MemoryAgent {
    /// Create a new `MemoryAgent` with the given identity.
    pub fn new(id: AgentId) -> Self {
        Self { id }
    }
}

#[async_trait::async_trait]
impl Actor for MemoryAgent {
    type Message = MemoryMessage;
    type State = MemoryState;
    type Error = MemoryError;
    type Response = serde_json::Value;

    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error> {
        match message {
            MemoryMessage::Store { key, value } => {
                state.entries.insert(key.clone(), value);
                state.entry_count = state.entries.len() as u64;
                Ok(serde_json::json!({
                    "stored": key,
                    "entry_count": state.entry_count,
                }))
            }
            MemoryMessage::Retrieve(key) => {
                let value = state.entries.get(&key).cloned();
                Ok(serde_json::json!({
                    "key": key,
                    "value": value,
                }))
            }
            MemoryMessage::Search { prefix } => {
                let matches: serde_json::Map<String, serde_json::Value> = state
                    .entries
                    .iter()
                    .filter(|(k, _)| k.starts_with(&prefix))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                Ok(serde_json::json!({
                    "prefix": prefix,
                    "matches": matches,
                }))
            }
            MemoryMessage::Delete(key) => {
                let found = state.entries.remove(&key).is_some();
                state.entry_count = state.entries.len() as u64;
                Ok(serde_json::json!({
                    "deleted": key,
                    "found": found,
                }))
            }
        }
    }

    fn pre_start(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn post_stop(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn actor_id(&self) -> AgentId {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_store_retrieve_delete_flow() {
        let mut agent = MemoryAgent::new(AgentId::new());
        let mut state = MemoryState::default();

        // 1. Store a value.
        let resp = agent
            .handle_message(
                MemoryMessage::Store {
                    key: "user.name".into(),
                    value: serde_json::json!("Alice"),
                },
                &mut state,
            )
            .await
            .unwrap();
        assert_eq!(resp["stored"], "user.name");
        assert_eq!(resp["entry_count"], 1);
        assert_eq!(state.entry_count, 1);

        // 2. Retrieve the stored value.
        let resp = agent
            .handle_message(MemoryMessage::Retrieve("user.name".into()), &mut state)
            .await
            .unwrap();
        assert_eq!(resp["key"], "user.name");
        assert_eq!(resp["value"], "Alice");

        // 3. Retrieve a missing key returns null.
        let resp = agent
            .handle_message(MemoryMessage::Retrieve("missing".into()), &mut state)
            .await
            .unwrap();
        assert_eq!(resp["key"], "missing");
        assert!(resp["value"].is_null());

        // 4. Store a second entry and search by prefix.
        let _ = agent
            .handle_message(
                MemoryMessage::Store {
                    key: "user.email".into(),
                    value: serde_json::json!("alice@example.com"),
                },
                &mut state,
            )
            .await
            .unwrap();
        assert_eq!(state.entry_count, 2);

        let resp = agent
            .handle_message(
                MemoryMessage::Search {
                    prefix: "user.".into(),
                },
                &mut state,
            )
            .await
            .unwrap();
        assert_eq!(resp["prefix"], "user.");
        let matches = resp["matches"].as_object().unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches["user.name"], "Alice");
        assert_eq!(matches["user.email"], "alice@example.com");

        // 5. Delete the first entry.
        let resp = agent
            .handle_message(MemoryMessage::Delete("user.name".into()), &mut state)
            .await
            .unwrap();
        assert_eq!(resp["deleted"], "user.name");
        assert_eq!(resp["found"], true);
        assert_eq!(state.entry_count, 1);

        // 6. Retrieve deleted key returns null.
        let resp = agent
            .handle_message(MemoryMessage::Retrieve("user.name".into()), &mut state)
            .await
            .unwrap();
        assert!(resp["value"].is_null());

        // 7. Delete a non-existent key.
        let resp = agent
            .handle_message(MemoryMessage::Delete("nope".into()), &mut state)
            .await
            .unwrap();
        assert_eq!(resp["deleted"], "nope");
        assert_eq!(resp["found"], false);
        assert_eq!(state.entry_count, 1);
    }
}
