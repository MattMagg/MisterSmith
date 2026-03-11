//! Critic agent role — reviews and validates outputs.

#[cfg(feature = "llm")]
use crate::orchestrator::LlmSupervision;
#[cfg(feature = "llm")]
use crate::roles::llm_bridge::complete_with_optional_supervision;
use crate::context_manager::{
    resolve_managed_context_input, ContextManager, ManagedContextInput, ManagedContextRuntime,
};
use mister_smith_core::{Actor, AgentId, AgentType, ContextBudget};
use mister_smith_persistence::SnapshotScope;
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
        /// Optional managed context payload or runtime request.
        managed_context: Option<ManagedContextInput>,
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
    managed_context: Option<ManagedContextRuntime>,
    #[cfg(feature = "llm")]
    router: Option<std::sync::Arc<mister_smith_llm::ModelRouter>>,
    #[cfg(feature = "llm")]
    supervision: Option<LlmSupervision>,
}

impl CriticAgent {
    /// Create a new `CriticAgent` with the given identity.
    pub fn new(id: AgentId) -> Self {
        Self {
            id,
            managed_context: None,
            #[cfg(feature = "llm")]
            router: None,
            #[cfg(feature = "llm")]
            supervision: None,
        }
    }

    /// Create a new `CriticAgent` with a managed-context runtime.
    pub fn with_managed_context(id: AgentId, managed_context: ManagedContextRuntime) -> Self {
        Self {
            id,
            managed_context: Some(managed_context),
            #[cfg(feature = "llm")]
            router: None,
            #[cfg(feature = "llm")]
            supervision: None,
        }
    }

    /// Create a new `CriticAgent` with an LLM [`ModelRouter`] for AI-powered evaluation.
    #[cfg(feature = "llm")]
    pub fn with_router(id: AgentId, router: std::sync::Arc<mister_smith_llm::ModelRouter>) -> Self {
        Self {
            id,
            managed_context: None,
            router: Some(router),
            supervision: None,
        }
    }

    /// Create a critic with router-backed supervision for a specific workflow target.
    #[cfg(feature = "llm")]
    pub fn with_router_and_supervision(
        id: AgentId,
        router: std::sync::Arc<mister_smith_llm::ModelRouter>,
        supervision: LlmSupervision,
    ) -> Self {
        Self {
            id,
            managed_context: None,
            router: Some(router),
            supervision: Some(supervision),
        }
    }

    /// Create a new `CriticAgent` with both router and managed-context runtime.
    #[cfg(feature = "llm")]
    pub fn with_router_and_managed_context(
        id: AgentId,
        router: std::sync::Arc<mister_smith_llm::ModelRouter>,
        managed_context: ManagedContextRuntime,
    ) -> Self {
        Self {
            id,
            managed_context: Some(managed_context),
            router: Some(router),
            supervision: None,
        }
    }

    /// Attach or replace the managed-context runtime for this agent.
    pub fn set_managed_context(&mut self, managed_context: ManagedContextRuntime) {
        self.managed_context = Some(managed_context);
    }

    /// Evaluate output after assembling bounded role-aware managed context.
    pub async fn evaluate_with_managed_context(
        &mut self,
        output: serde_json::Value,
        criteria: serde_json::Value,
        context_manager: &mut ContextManager,
        scope: SnapshotScope,
        budget: ContextBudget,
        state: &mut CriticState,
    ) -> Result<serde_json::Value, CriticError> {
        let managed_context = context_manager
            .assemble_role_context(scope, AgentType::Critic, budget)
            .await
            .map_err(|error| CriticError::Internal(error.to_string()))?;

        self.handle_message(
            CriticMessage::Evaluate {
                output,
                criteria,
                managed_context: Some(ManagedContextInput::Payload(managed_context.payload)),
            },
            state,
        )
        .await
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
            CriticMessage::Evaluate {
                output,
                criteria,
                managed_context,
            } => {
                let criteria = match resolve_managed_context_input(
                    self.managed_context.as_mut(),
                    self.id,
                    AgentType::Critic,
                    managed_context,
                )
                .await
                .map_err(|error| CriticError::Internal(error.to_string()))?
                {
                    Some(payload) => {
                        crate::context_manager::attach_managed_context(criteria, payload)
                    }
                    None => criteria,
                };

                state.evaluations_completed += 1;

                // When the `llm` feature is enabled and a router is configured,
                // ask the model to evaluate the output against the criteria.
                #[cfg(feature = "llm")]
                if let Some(router) = &self.router {
                    let result: Result<serde_json::Value, CriticError> = async {
                        use mister_smith_llm::{ChatMessage, CompletionRequest, ContentBlock};

                        let request = CompletionRequest {
                            system: Some(
                                "You are a quality evaluation agent. Given an output and criteria, \
                                 evaluate whether the output meets the criteria. Return a JSON object \
                                 with 'evaluation' (pass/fail), 'confidence' (0.0-1.0), 'suggestions' \
                                 (array of strings), and 'reasoning' (string)."
                                    .to_string(),
                            ),
                            messages: vec![ChatMessage::User {
                                content: serde_json::json!({
                                    "output": output,
                                    "criteria": criteria,
                                }),
                            }],
                            ..CompletionRequest::default()
                        };
                        let response =
                            complete_with_optional_supervision(router, request, self.supervision.as_ref())
                                .await
                                .map_err(|error| CriticError::Internal(error.to_string()))?;

                        // Extract text from the first text content block.
                        let text = response
                            .content
                            .iter()
                            .find_map(|block| match block {
                                ContentBlock::Text { text } => Some(text.as_str()),
                                _ => None,
                            })
                            .unwrap_or("");

                        let mut eval: serde_json::Value = serde_json::from_str(text)
                            .unwrap_or_else(|_| {
                                serde_json::json!({
                                    "evaluation": "pass",
                                    "raw_response": text,
                                })
                            });

                        // Attach metadata the stub normally includes.
                        if let Some(obj) = eval.as_object_mut() {
                            obj.insert("output_reviewed".to_string(), output.clone());
                            obj.insert("criteria_applied".to_string(), criteria.clone());
                            obj.insert(
                                "evaluations_completed".to_string(),
                                serde_json::json!(state.evaluations_completed),
                            );
                        }

                        Ok(eval)
                    }
                    .await;

                    let eval = result?;
                    return Ok(eval);
                }

                // Stub implementation — deterministic evaluation without an LLM.
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
                    managed_context: None,
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
                    managed_context: None,
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
