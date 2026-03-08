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
    #[cfg(feature = "llm")]
    router: Option<std::sync::Arc<mister_smith_llm::ModelRouter>>,
}

impl PlannerAgent {
    /// Create a new `PlannerAgent` with the given identity.
    pub fn new(id: AgentId) -> Self {
        Self {
            id,
            #[cfg(feature = "llm")]
            router: None,
        }
    }

    /// Create a new `PlannerAgent` with an LLM [`ModelRouter`] for AI-powered planning.
    #[cfg(feature = "llm")]
    pub fn with_router(
        id: AgentId,
        router: std::sync::Arc<mister_smith_llm::ModelRouter>,
    ) -> Self {
        Self {
            id,
            router: Some(router),
        }
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
                // When the `llm` feature is enabled and a router is configured,
                // ask the model to decompose the goal into concrete steps.
                #[cfg(feature = "llm")]
                if let Some(router) = &self.router {
                    let result: Result<serde_json::Value, PlannerError> = async {
                        use mister_smith_llm::{ChatMessage, CompletionRequest, ContentBlock};

                        let mut request = CompletionRequest::default();
                        request.system = Some(
                            "You are a task planning agent. Given a goal and context, \
                             decompose it into concrete steps. Return a JSON object with \
                             'goal', 'steps' (array of objects with 'step' number, 'action', \
                             and 'description'), and 'context'."
                                .to_string(),
                        );
                        request.messages = vec![ChatMessage::User {
                            content: serde_json::json!({
                                "goal": goal,
                                "context": context,
                            }),
                        }];

                        let (response, _routing) = router
                            .route_completion(request)
                            .await
                            .map_err(|e| PlannerError::Internal(e.to_string()))?;

                        // Extract text from the first text content block.
                        let text = response
                            .content
                            .iter()
                            .find_map(|block| match block {
                                ContentBlock::Text { text } => Some(text.as_str()),
                                _ => None,
                            })
                            .unwrap_or("");

                        let plan: serde_json::Value =
                            serde_json::from_str(text).unwrap_or_else(|_| {
                                serde_json::json!({
                                    "goal": goal,
                                    "raw_response": text,
                                    "context": context,
                                })
                            });

                        Ok(plan)
                    }
                    .await;

                    let plan = result?;
                    state.current_plan = Some(plan.clone());
                    return Ok(plan);
                }

                // Stub implementation — deterministic plan without an LLM.
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
