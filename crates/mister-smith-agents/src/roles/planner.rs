//! Planner agent role — creates execution plans from goals.

use mister_smith_core::{Actor, AgentId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Messages handled by the [`PlannerAgent`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlannerMessage {
    /// Request a plan for the given goal and context.
    PlanGoal {
        /// High-level goal description.
        goal: String,
        /// Additional context for planning.
        context: serde_json::Value,
    },
    /// Query the current or most recent plan.
    QueryPlan,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Persistent state for the [`PlannerAgent`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlannerState {
    /// The most recently generated plan, if any.
    pub current_plan: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by [`PlannerAgent`] operations.
#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    /// A planning operation failed.
    #[error("planner error: {0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// Creates execution plans from high-level goals and contextual information.
pub struct PlannerAgent {
    id: AgentId,
}

impl PlannerAgent {
    /// Create a new `PlannerAgent` with the given identity.
    pub fn new(id: AgentId) -> Self {
        Self { id }
    }
}

#[async_trait::async_trait]
impl Actor for PlannerAgent {
    type Message = PlannerMessage;
    type State = PlannerState;
    type Error = PlannerError;
    type Response = serde_json::Value;

    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error> {
        match message {
            PlannerMessage::PlanGoal { goal, context } => {
                let plan = serde_json::json!({
                    "goal": goal,
                    "steps": [
                        {
                            "step": 1,
                            "action": "analyze",
                            "description": goal,
                        }
                    ],
                    "context": context,
                });
                state.current_plan = Some(plan.clone());
                Ok(plan)
            }
            PlannerMessage::QueryPlan => Ok(state
                .current_plan
                .clone()
                .unwrap_or_else(|| serde_json::json!({"error": "no plan available"}))),
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
    async fn plan_goal_stores_plan_and_query_returns_it() {
        let mut agent = PlannerAgent::new(AgentId::new());
        let mut state = PlannerState::default();

        // QueryPlan with no plan returns error message.
        let result = agent
            .handle_message(PlannerMessage::QueryPlan, &mut state)
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!({"error": "no plan available"}));

        // PlanGoal creates and stores a plan.
        let goal = "deploy service".to_string();
        let context = serde_json::json!({"env": "staging"});
        let plan = agent
            .handle_message(
                PlannerMessage::PlanGoal {
                    goal: goal.clone(),
                    context: context.clone(),
                },
                &mut state,
            )
            .await
            .unwrap();

        let expected = serde_json::json!({
            "goal": "deploy service",
            "steps": [{"step": 1, "action": "analyze", "description": "deploy service"}],
            "context": {"env": "staging"},
        });
        assert_eq!(plan, expected);
        assert_eq!(state.current_plan, Some(expected.clone()));

        // QueryPlan now returns the stored plan.
        let queried = agent
            .handle_message(PlannerMessage::QueryPlan, &mut state)
            .await
            .unwrap();
        assert_eq!(queried, expected);
    }
}
