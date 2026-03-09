//! Executor agent role — carries out planned actions.

use mister_smith_core::{Actor, AgentId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Messages handled by the [`ExecutorAgent`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutorMessage {
    /// Execute the given plan.
    ExecutePlan {
        /// The plan to execute (structured as JSON).
        plan: serde_json::Value,
    },
    /// Report that a plan step has completed.
    StepComplete {
        /// Identifier of the completed step.
        step_id: String,
        /// Result payload from the step.
        result: serde_json::Value,
    },
    /// Query the current execution progress.
    QueryProgress,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Persistent state for the [`ExecutorAgent`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutorState {
    /// Number of steps completed in the current plan.
    pub steps_completed: u64,
    /// Whether execution is currently in progress.
    pub executing: bool,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by [`ExecutorAgent`] operations.
#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    /// An execution operation failed.
    #[error("executor error: {0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// Carries out planned actions — plan execution, step tracking, and
/// progress queries.
pub struct ExecutorAgent {
    id: AgentId,
    #[cfg(feature = "llm")]
    router: Option<std::sync::Arc<mister_smith_llm::ModelRouter>>,
}

impl ExecutorAgent {
    /// Create a new `ExecutorAgent` with the given identity.
    pub fn new(id: AgentId) -> Self {
        Self {
            id,
            #[cfg(feature = "llm")]
            router: None,
        }
    }

    /// Create a new `ExecutorAgent` with an LLM [`ModelRouter`] for AI-powered execution strategy.
    #[cfg(feature = "llm")]
    pub fn with_router(id: AgentId, router: std::sync::Arc<mister_smith_llm::ModelRouter>) -> Self {
        Self {
            id,
            router: Some(router),
        }
    }
}

#[async_trait::async_trait]
impl Actor for ExecutorAgent {
    type Message = ExecutorMessage;
    type State = ExecutorState;
    type Error = ExecutorError;
    type Response = serde_json::Value;

    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error> {
        match message {
            ExecutorMessage::ExecutePlan { plan } => {
                state.executing = true;
                state.steps_completed = 0;

                // When the `llm` feature is enabled and a router is configured,
                // ask the model to analyze the plan and suggest an execution strategy.
                #[cfg(feature = "llm")]
                if let Some(router) = &self.router {
                    let result: Result<serde_json::Value, ExecutorError> = async {
                        use mister_smith_llm::{ChatMessage, CompletionRequest, ContentBlock};

                        let mut request = CompletionRequest::default();
                        request.system = Some(
                            "You are a task execution agent. Given a plan, analyze it and \
                             suggest an execution strategy. Return a JSON object with 'status', \
                             'strategy' (string), and 'estimated_steps' (number)."
                                .to_string(),
                        );
                        request.messages = vec![ChatMessage::User {
                            content: serde_json::json!({
                                "plan": plan,
                            }),
                        }];

                        let (response, _routing) = router
                            .route_completion(request)
                            .await
                            .map_err(|e| ExecutorError::Internal(e.to_string()))?;

                        // Extract text from the first text content block.
                        let text = response
                            .content
                            .iter()
                            .find_map(|block| match block {
                                ContentBlock::Text { text } => Some(text.as_str()),
                                _ => None,
                            })
                            .unwrap_or("");

                        let mut strategy: serde_json::Value = serde_json::from_str(text)
                            .unwrap_or_else(|_| {
                                serde_json::json!({
                                    "status": "executing",
                                    "raw_response": text,
                                })
                            });

                        // Ensure the plan is included in the response.
                        if let Some(obj) = strategy.as_object_mut() {
                            obj.entry("status".to_string())
                                .or_insert_with(|| serde_json::json!("executing"));
                            obj.insert("plan".to_string(), plan.clone());
                        }

                        Ok(strategy)
                    }
                    .await;

                    let strategy = result?;
                    return Ok(strategy);
                }

                // Stub implementation — deterministic response without an LLM.
                Ok(serde_json::json!({
                    "status": "executing",
                    "plan": plan,
                }))
            }
            ExecutorMessage::StepComplete { step_id, result } => {
                state.steps_completed += 1;
                Ok(serde_json::json!({
                    "step_id": step_id,
                    "steps_completed": state.steps_completed,
                    "result": result,
                }))
            }
            ExecutorMessage::QueryProgress => Ok(serde_json::json!({
                "executing": state.executing,
                "steps_completed": state.steps_completed,
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
    async fn executor_plan_step_progress_flow() {
        let mut agent = ExecutorAgent::new(AgentId::new());
        let mut state = ExecutorState::default();

        // 1. Start executing a plan.
        let plan = serde_json::json!({"steps": ["a", "b", "c"]});
        let resp = agent
            .handle_message(
                ExecutorMessage::ExecutePlan { plan: plan.clone() },
                &mut state,
            )
            .await
            .unwrap();
        assert_eq!(resp["status"], "executing");
        assert_eq!(resp["plan"], plan);
        assert!(state.executing);
        assert_eq!(state.steps_completed, 0);

        // 2. Complete two steps.
        let resp = agent
            .handle_message(
                ExecutorMessage::StepComplete {
                    step_id: "a".into(),
                    result: serde_json::json!("ok"),
                },
                &mut state,
            )
            .await
            .unwrap();
        assert_eq!(resp["step_id"], "a");
        assert_eq!(resp["steps_completed"], 1);
        assert_eq!(resp["result"], "ok");

        let _ = agent
            .handle_message(
                ExecutorMessage::StepComplete {
                    step_id: "b".into(),
                    result: serde_json::json!("done"),
                },
                &mut state,
            )
            .await
            .unwrap();
        assert_eq!(state.steps_completed, 2);

        // 3. Query progress.
        let resp = agent
            .handle_message(ExecutorMessage::QueryProgress, &mut state)
            .await
            .unwrap();
        assert_eq!(resp["executing"], true);
        assert_eq!(resp["steps_completed"], 2);
    }
}
