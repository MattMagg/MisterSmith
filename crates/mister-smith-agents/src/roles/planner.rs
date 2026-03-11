//! Planner agent role — creates execution plans from goals.

use crate::context_manager::{
    resolve_managed_context_input, ContextManager, ManagedContextInput, ManagedContextRuntime,
};
#[cfg(feature = "llm")]
use crate::orchestrator::LlmSupervision;
#[cfg(feature = "llm")]
use crate::roles::llm_bridge::complete_with_optional_supervision;
use crate::scheduler::TaskAssignment;
use mister_smith_core::{Actor, AgentId, AgentType, ContextBudget};
use mister_smith_persistence::SnapshotScope;
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
        /// Optional managed context payload or runtime request.
        managed_context: Option<ManagedContextInput>,
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
    managed_context: Option<ManagedContextRuntime>,
    #[cfg(feature = "llm")]
    router: Option<std::sync::Arc<mister_smith_llm::ModelRouter>>,
    #[cfg(feature = "llm")]
    supervision: Option<LlmSupervision>,
}

impl PlannerAgent {
    /// Create a new `PlannerAgent` with the given identity.
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

    /// Create a new `PlannerAgent` with a managed-context runtime.
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

    /// Create a new `PlannerAgent` with an LLM [`ModelRouter`] for AI-powered planning.
    #[cfg(feature = "llm")]
    pub fn with_router(id: AgentId, router: std::sync::Arc<mister_smith_llm::ModelRouter>) -> Self {
        Self {
            id,
            managed_context: None,
            router: Some(router),
            supervision: None,
        }
    }

    /// Create a planner with router-backed supervision for a specific workflow target.
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

    /// Create a new `PlannerAgent` with both router and managed-context runtime.
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

    /// Plan a goal after assembling bounded role-aware managed context.
    pub async fn plan_goal_with_managed_context(
        &mut self,
        goal: String,
        context: serde_json::Value,
        context_manager: &mut ContextManager,
        scope: SnapshotScope,
        budget: ContextBudget,
        state: &mut PlannerState,
    ) -> Result<serde_json::Value, PlannerError> {
        let managed_context = context_manager
            .assemble_role_context(scope, AgentType::Planner, budget)
            .await
            .map_err(|error| PlannerError::Internal(error.to_string()))?;

        self.handle_message(
            PlannerMessage::PlanGoal {
                goal,
                context,
                managed_context: Some(ManagedContextInput::Payload(managed_context.payload)),
            },
            state,
        )
        .await
    }
}

/// Normalize planner output into the minimum shape required by the Phase 10 control plane.
pub fn normalize_planner_output(
    goal: &str,
    context: &serde_json::Value,
    plan: serde_json::Value,
) -> serde_json::Value {
    let has_steps = plan
        .get("steps")
        .and_then(serde_json::Value::as_array)
        .map(|steps| !steps.is_empty())
        .unwrap_or(false);
    if has_steps {
        let mut object = plan.as_object().cloned().unwrap_or_default();
        object
            .entry("goal".to_string())
            .or_insert_with(|| serde_json::json!(goal));
        object
            .entry("context".to_string())
            .or_insert_with(|| context.clone());
        serde_json::Value::Object(object)
    } else {
        serde_json::json!({
            "goal": goal,
            "steps": [
                {
                    "id": "step-1",
                    "step": 1,
                    "action": "analyze",
                    "description": goal,
                    "role": "worker",
                }
            ],
            "context": context,
        })
    }
}

/// Synthesize planner-shaped output from decomposed scheduler tasks.
pub fn planner_output_from_subtasks(
    task: &TaskAssignment,
    subtasks: &[TaskAssignment],
) -> serde_json::Value {
    let sequential = subtasks.iter().all(|subtask| {
        subtask
            .input
            .get("step_index")
            .and_then(serde_json::Value::as_u64)
            .is_some()
    });
    let mut ordered = subtasks.to_vec();
    if sequential {
        ordered.sort_by_key(|subtask| {
            subtask
                .input
                .get("step_index")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(u64::MAX)
        });
    }

    let mut previous_step_id = None;
    let steps: Vec<serde_json::Value> = ordered
        .iter()
        .enumerate()
        .map(|(index, subtask)| {
            let step_id = subtask.task_id.to_string();
            let step_description = subtask
                .input
                .get("step")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(subtask.task_type.as_str());
            let depends_on = if sequential {
                previous_step_id
                    .iter()
                    .map(|step: &String| serde_json::Value::String(step.clone()))
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            previous_step_id = Some(step_id.clone());

            serde_json::json!({
                "id": step_id,
                "step": index + 1,
                "action": subtask.task_type,
                "description": step_description,
                "role": "worker",
                "depends_on": depends_on,
                "input": subtask.input,
            })
        })
        .collect();

    serde_json::json!({
        "goal": task.task_type,
        "steps": steps,
        "context": task.input,
    })
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
            PlannerMessage::PlanGoal {
                goal,
                context,
                managed_context,
            } => {
                let context = match resolve_managed_context_input(
                    self.managed_context.as_mut(),
                    self.id,
                    AgentType::Planner,
                    managed_context,
                )
                .await
                .map_err(|error| PlannerError::Internal(error.to_string()))?
                {
                    Some(payload) => {
                        crate::context_manager::attach_managed_context(context, payload)
                    }
                    None => context,
                };

                // When the `llm` feature is enabled and a router is configured,
                // ask the model to decompose the goal into concrete steps.
                #[cfg(feature = "llm")]
                if let Some(router) = &self.router {
                    let result: Result<serde_json::Value, PlannerError> = async {
                        use mister_smith_llm::{ChatMessage, CompletionRequest, ContentBlock};

                        let request = CompletionRequest {
                            system: Some(
                                "You are a task planning agent. Given a goal and context, \
                                 decompose it into concrete steps. Return a JSON object with \
                                 'goal', 'steps' (array of objects with 'step' number, 'action', \
                                 and 'description'), and 'context'."
                                    .to_string(),
                            ),
                            messages: vec![ChatMessage::User {
                                content: serde_json::json!({
                                    "goal": goal,
                                    "context": context,
                                }),
                            }],
                            ..CompletionRequest::default()
                        };
                        let response = complete_with_optional_supervision(
                            router,
                            request,
                            self.supervision.as_ref(),
                        )
                        .await
                        .map_err(|error| PlannerError::Internal(error.to_string()))?;

                        // Extract text from the first text content block.
                        let text = response
                            .content
                            .iter()
                            .find_map(|block| match block {
                                ContentBlock::Text { text } => Some(text.as_str()),
                                _ => None,
                            })
                            .unwrap_or("");

                        let plan: serde_json::Value = serde_json::from_str(text)
                            .map(|value| normalize_planner_output(&goal, &context, value))
                            .unwrap_or_else(|_| {
                                normalize_planner_output(
                                    &goal,
                                    &context,
                                    serde_json::json!({
                                        "goal": goal,
                                        "raw_response": text,
                                        "context": context,
                                    }),
                                )
                            });

                        Ok(plan)
                    }
                    .await;

                    let plan = result?;
                    state.current_plan = Some(plan.clone());
                    return Ok(plan);
                }

                // Stub implementation — deterministic plan without an LLM.
                let plan = normalize_planner_output(
                    &goal,
                    &context,
                    serde_json::json!({
                        "goal": goal,
                        "steps": [
                            {
                                "id": "step-1",
                                "step": 1,
                                "action": "analyze",
                                "description": goal,
                                "role": "worker",
                            }
                        ],
                        "context": context,
                    }),
                );
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
                    managed_context: None,
                },
                &mut state,
            )
            .await
            .unwrap();

        let expected = serde_json::json!({
            "goal": "deploy service",
            "steps": [{
                "id": "step-1",
                "step": 1,
                "action": "analyze",
                "description": "deploy service",
                "role": "worker",
            }],
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
