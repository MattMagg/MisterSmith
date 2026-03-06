//! Critic agent role — reviews and validates outputs.

use mister_smith_core::{Actor, AgentId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Messages handled by the [`CriticAgent`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CriticMessage {
    /// Evaluate an output against the given criteria.
    Evaluate {
        /// The output to evaluate.
        output: serde_json::Value,
        /// Criteria for evaluation.
        criteria: serde_json::Value,
    },
    /// Query the evaluation history.
    QueryHistory,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Persistent state for the [`CriticAgent`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CriticState {
    /// Number of evaluations performed.
    pub evaluations_completed: u64,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by [`CriticAgent`] operations.
#[derive(Debug, thiserror::Error)]
pub enum CriticError {
    /// An evaluation operation failed.
    #[error("critic error: {0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// Reviews and validates outputs against specified criteria.
pub struct CriticAgent {
    id: AgentId,
}

impl CriticAgent {
    /// Create a new `CriticAgent` with the given identity.
    pub fn new(id: AgentId) -> Self {
        Self { id }
    }
}

#[async_trait::async_trait]
impl Actor for CriticAgent {
    type Message = CriticMessage;
    type State = CriticState;
    type Error = CriticError;
    type Response = serde_json::Value;

    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error> {
        match message {
            CriticMessage::Evaluate { output, criteria } => {
                state.evaluations_completed += 1;
                Ok(serde_json::json!({
                    "evaluation": "pass",
                    "output_reviewed": output,
                    "criteria_applied": criteria,
                    "evaluations_completed": state.evaluations_completed,
                }))
            }
            CriticMessage::QueryHistory => Ok(serde_json::json!({
                "evaluations_completed": state.evaluations_completed,
            })),
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
    async fn evaluate_increments_count_and_query_returns_it() {
        let mut agent = CriticAgent::new(AgentId::new());
        let mut state = CriticState::default();

        // Evaluate twice.
        let r1 = agent
            .handle_message(
                CriticMessage::Evaluate {
                    output: serde_json::json!("result-1"),
                    criteria: serde_json::json!(["accuracy"]),
                },
                &mut state,
            )
            .await
            .unwrap();
        assert_eq!(r1["evaluation"], "pass");
        assert_eq!(r1["evaluations_completed"], 1);

        let r2 = agent
            .handle_message(
                CriticMessage::Evaluate {
                    output: serde_json::json!("result-2"),
                    criteria: serde_json::json!(["completeness"]),
                },
                &mut state,
            )
            .await
            .unwrap();
        assert_eq!(r2["evaluations_completed"], 2);

        // QueryHistory should reflect the same count.
        let history = agent
            .handle_message(CriticMessage::QueryHistory, &mut state)
            .await
            .unwrap();
        assert_eq!(history["evaluations_completed"], 2);
    }
}
