//! Supervisor agent role — manages child agent lifecycles.

use mister_smith_core::{Actor, AgentId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Messages handled by the [`SupervisorAgent`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupervisorMessage {
    /// Register a new child agent under this supervisor.
    RegisterChild(AgentId),
    /// Remove a child agent from supervision.
    RemoveChild(AgentId),
    /// Query the list of currently supervised children.
    QueryChildren,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Persistent state for the [`SupervisorAgent`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SupervisorState {
    /// IDs of agents currently supervised.
    pub children: Vec<AgentId>,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by [`SupervisorAgent`] operations.
#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    /// A child-management operation failed.
    #[error("supervisor error: {0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// Manages child agent lifecycles — registration, removal, and queries.
pub struct SupervisorAgent {
    id: AgentId,
}

impl SupervisorAgent {
    /// Create a new `SupervisorAgent` with the given identity.
    pub fn new(id: AgentId) -> Self {
        Self { id }
    }
}

#[async_trait::async_trait]
impl Actor for SupervisorAgent {
    type Message = SupervisorMessage;
    type State = SupervisorState;
    type Error = SupervisorError;
    type Response = serde_json::Value;

    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error> {
        match message {
            SupervisorMessage::RegisterChild(child_id) => {
                if !state.children.contains(&child_id) {
                    state.children.push(child_id);
                }
                Ok(serde_json::json!({ "registered": child_id.to_string() }))
            }
            SupervisorMessage::RemoveChild(child_id) => {
                state.children.retain(|id| id != &child_id);
                Ok(serde_json::json!({ "removed": child_id.to_string() }))
            }
            SupervisorMessage::QueryChildren => {
                let ids: Vec<String> = state.children.iter().map(|id| id.to_string()).collect();
                let count = ids.len();
                Ok(serde_json::json!({ "children": ids, "count": count }))
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
    async fn register_child_adds_to_state_and_returns_id() {
        let mut agent = SupervisorAgent::new(AgentId::new());
        let mut state = SupervisorState::default();
        let child_id = AgentId::new();

        let resp = agent
            .handle_message(SupervisorMessage::RegisterChild(child_id), &mut state)
            .await
            .unwrap();

        assert_eq!(resp["registered"], child_id.to_string());
        assert_eq!(state.children.len(), 1);
        assert_eq!(state.children[0], child_id);
    }

    #[tokio::test]
    async fn register_child_is_idempotent() {
        let mut agent = SupervisorAgent::new(AgentId::new());
        let mut state = SupervisorState::default();
        let child_id = AgentId::new();

        agent
            .handle_message(SupervisorMessage::RegisterChild(child_id), &mut state)
            .await
            .unwrap();
        agent
            .handle_message(SupervisorMessage::RegisterChild(child_id), &mut state)
            .await
            .unwrap();

        assert_eq!(state.children.len(), 1);
    }

    #[tokio::test]
    async fn remove_child_removes_from_state() {
        let mut agent = SupervisorAgent::new(AgentId::new());
        let mut state = SupervisorState::default();
        let child_id = AgentId::new();

        agent
            .handle_message(SupervisorMessage::RegisterChild(child_id), &mut state)
            .await
            .unwrap();
        let resp = agent
            .handle_message(SupervisorMessage::RemoveChild(child_id), &mut state)
            .await
            .unwrap();

        assert_eq!(resp["removed"], child_id.to_string());
        assert!(state.children.is_empty());
    }

    #[tokio::test]
    async fn remove_nonexistent_child_is_noop() {
        let mut agent = SupervisorAgent::new(AgentId::new());
        let mut state = SupervisorState::default();
        let child_id = AgentId::new();

        let resp = agent
            .handle_message(SupervisorMessage::RemoveChild(child_id), &mut state)
            .await
            .unwrap();

        assert_eq!(resp["removed"], child_id.to_string());
        assert!(state.children.is_empty());
    }

    #[tokio::test]
    async fn query_children_returns_ids_and_count() {
        let mut agent = SupervisorAgent::new(AgentId::new());
        let mut state = SupervisorState::default();
        let child_a = AgentId::new();
        let child_b = AgentId::new();

        agent
            .handle_message(SupervisorMessage::RegisterChild(child_a), &mut state)
            .await
            .unwrap();
        agent
            .handle_message(SupervisorMessage::RegisterChild(child_b), &mut state)
            .await
            .unwrap();

        let resp = agent
            .handle_message(SupervisorMessage::QueryChildren, &mut state)
            .await
            .unwrap();

        let children = resp["children"].as_array().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(resp["count"], 2);
        assert!(children.contains(&serde_json::json!(child_a.to_string())));
        assert!(children.contains(&serde_json::json!(child_b.to_string())));
    }

    #[tokio::test]
    async fn query_empty_children_returns_empty_list() {
        let mut agent = SupervisorAgent::new(AgentId::new());
        let mut state = SupervisorState::default();

        let resp = agent
            .handle_message(SupervisorMessage::QueryChildren, &mut state)
            .await
            .unwrap();

        assert_eq!(resp["children"].as_array().unwrap().len(), 0);
        assert_eq!(resp["count"], 0);
    }
}
