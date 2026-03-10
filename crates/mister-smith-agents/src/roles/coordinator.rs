//! Coordinator agent role — coordinates multi-agent workflows.

use crate::execution_graph::ExecutionGraph;
use crate::roles::planner::normalize_planner_output;
use crate::topology::{TopologyCompiler, TopologySignals};
use mister_smith_core::{Actor, AgentId, TaskId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Messages handled by the [`CoordinatorAgent`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinatorMessage {
    /// Submit a new task for coordinated execution.
    SubmitTask {
        /// The type/category of task to coordinate.
        task_type: String,
        /// Task input payload.
        input: serde_json::Value,
    },
    /// Report the result of a subtask back to the coordinator.
    SubtaskResult {
        /// Identifier of the completed subtask.
        task_id: TaskId,
        /// Result payload from the subtask.
        result: serde_json::Value,
    },
    /// Notify that a team member has failed.
    TeamMemberFailed(AgentId),
    /// Query the overall workflow progress.
    QueryProgress,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Persistent state for the [`CoordinatorAgent`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoordinatorState {
    /// IDs of tasks currently in flight.
    pub active_tasks: Vec<TaskId>,
    /// Number of subtask results received.
    pub results_received: u64,
    /// Most recently compiled execution graph for visibility and dispatch control.
    pub current_graph: Option<ExecutionGraph>,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by [`CoordinatorAgent`] operations.
#[derive(Debug, thiserror::Error)]
pub enum CoordinatorError {
    /// A coordination operation failed.
    #[error("coordinator error: {0}")]
    Internal(String),
    /// Planner output could not be normalized into a valid execution graph.
    #[error("coordinator topology error: {0}")]
    Topology(#[from] mister_smith_core::TopologyError),
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// Coordinates multi-agent workflows — task submission, subtask tracking,
/// failure notification, and progress queries.
pub struct CoordinatorAgent {
    id: AgentId,
}

impl CoordinatorAgent {
    /// Create a new `CoordinatorAgent` with the given identity.
    pub fn new(id: AgentId) -> Self {
        Self { id }
    }
}

#[async_trait::async_trait]
impl Actor for CoordinatorAgent {
    type Message = CoordinatorMessage;
    type State = CoordinatorState;
    type Error = CoordinatorError;
    type Response = serde_json::Value;

    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error> {
        match message {
            CoordinatorMessage::SubmitTask { task_type, input } => {
                let task_id = TaskId::new();
                let planner_payload = input
                    .get("planner_output")
                    .cloned()
                    .or_else(|| input.get("steps").map(|_| input.clone()))
                    .unwrap_or_else(|| {
                        serde_json::json!({
                            "goal": task_type,
                            "steps": [
                                {
                                    "id": task_id.to_string(),
                                    "step": 1,
                                    "action": task_type,
                                    "description": "submit task",
                                    "role": "worker",
                                    "input": input,
                                }
                            ],
                        })
                    });
                let planner_output = normalize_planner_output(&task_type, &input, planner_payload);
                let compiler = TopologyCompiler::default();
                let graph =
                    compiler.compile(task_id, &planner_output, &TopologySignals::default())?;
                state.active_tasks.push(task_id);
                state.current_graph = Some(graph.clone());
                Ok(serde_json::json!({
                    "task_id": task_id.to_string(),
                    "task_type": task_type,
                    "status": "submitted",
                    "graph_id": graph.graph_id.to_string(),
                    "topology_kind": format!("{:?}", graph.topology_plan.topology_kind),
                }))
            }
            CoordinatorMessage::SubtaskResult { task_id, result: _ } => {
                state.results_received += 1;
                Ok(serde_json::json!({
                    "received": task_id.to_string(),
                    "total_results": state.results_received
                }))
            }
            CoordinatorMessage::TeamMemberFailed(agent_id) => Ok(serde_json::json!({
                "failed_member": agent_id.to_string(),
                "active_tasks": state.active_tasks.len(),
                "current_topology": state
                    .current_graph
                    .as_ref()
                    .map(|graph| format!("{:?}", graph.topology_plan.topology_kind)),
            })),
            CoordinatorMessage::QueryProgress => Ok(serde_json::json!({
                "active_tasks": state.active_tasks.len(),
                "results_received": state.results_received,
                "graph_id": state.current_graph.as_ref().map(|graph| graph.graph_id.to_string()),
                "topology_kind": state
                    .current_graph
                    .as_ref()
                    .map(|graph| format!("{:?}", graph.topology_plan.topology_kind)),
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
    async fn submit_task_creates_and_tracks() {
        let mut agent = CoordinatorAgent::new(AgentId::new());
        let mut state = CoordinatorState::default();

        let resp = agent
            .handle_message(
                CoordinatorMessage::SubmitTask {
                    task_type: "analysis".into(),
                    input: serde_json::json!({"data": 1}),
                },
                &mut state,
            )
            .await
            .unwrap();

        assert_eq!(resp["task_type"], "analysis");
        assert_eq!(resp["status"], "submitted");
        assert!(resp["task_id"].is_string());
        assert_eq!(state.active_tasks.len(), 1);
    }

    #[tokio::test]
    async fn subtask_result_increments_counter() {
        let mut agent = CoordinatorAgent::new(AgentId::new());
        let mut state = CoordinatorState::default();
        let task_id = TaskId::new();

        let resp = agent
            .handle_message(
                CoordinatorMessage::SubtaskResult {
                    task_id,
                    result: serde_json::json!({"ok": true}),
                },
                &mut state,
            )
            .await
            .unwrap();

        assert_eq!(resp["received"], task_id.to_string());
        assert_eq!(resp["total_results"], 1);
        assert_eq!(state.results_received, 1);
    }

    #[tokio::test]
    async fn team_member_failed_reports_status() {
        let mut agent = CoordinatorAgent::new(AgentId::new());
        let mut state = CoordinatorState {
            active_tasks: vec![TaskId::new(), TaskId::new()],
            results_received: 0,
            current_graph: None,
        };
        let failed_id = AgentId::new();

        let resp = agent
            .handle_message(CoordinatorMessage::TeamMemberFailed(failed_id), &mut state)
            .await
            .unwrap();

        assert_eq!(resp["failed_member"], failed_id.to_string());
        assert_eq!(resp["active_tasks"], 2);
    }

    #[tokio::test]
    async fn query_progress_returns_counts() {
        let mut agent = CoordinatorAgent::new(AgentId::new());
        let mut state = CoordinatorState {
            active_tasks: vec![TaskId::new()],
            results_received: 5,
            current_graph: None,
        };

        let resp = agent
            .handle_message(CoordinatorMessage::QueryProgress, &mut state)
            .await
            .unwrap();

        assert_eq!(resp["active_tasks"], 1);
        assert_eq!(resp["results_received"], 5);
    }
}
