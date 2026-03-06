//! Worker agent role — performs assigned tasks.

use mister_smith_core::{Actor, AgentId, TaskId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

/// Messages handled by the [`WorkerAgent`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerMessage {
    /// Assign a task for execution.
    AssignTask {
        /// Identifier of the task to execute.
        task_id: TaskId,
        /// Task input payload.
        input: serde_json::Value,
    },
    /// Cancel a running task.
    CancelTask(TaskId),
    /// Query the worker's current status.
    QueryStatus,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Persistent state for the [`WorkerAgent`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkerState {
    /// The task currently being executed, if any.
    pub current_task: Option<TaskId>,
    /// Number of tasks completed since startup.
    pub tasks_completed: u64,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors produced by [`WorkerAgent`] operations.
#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    /// A task-execution operation failed.
    #[error("worker error: {0}")]
    Internal(String),
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

/// Performs assigned tasks — execution, cancellation, and status reporting.
pub struct WorkerAgent {
    id: AgentId,
}

impl WorkerAgent {
    /// Create a new `WorkerAgent` with the given identity.
    pub fn new(id: AgentId) -> Self {
        Self { id }
    }
}

#[async_trait::async_trait]
impl Actor for WorkerAgent {
    type Message = WorkerMessage;
    type State = WorkerState;
    type Error = WorkerError;
    type Response = serde_json::Value;

    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<Self::Response, Self::Error> {
        match message {
            WorkerMessage::AssignTask { task_id, input } => {
                let response = serde_json::json!({
                    "accepted": task_id.to_string(),
                    "input": input,
                });
                // Task completes immediately in this simple implementation.
                state.tasks_completed += 1;
                state.current_task = None;
                Ok(response)
            }
            WorkerMessage::CancelTask(task_id) => {
                if state.current_task == Some(task_id) {
                    state.current_task = None;
                    Ok(serde_json::json!({ "cancelled": task_id.to_string() }))
                } else {
                    Ok(serde_json::json!({ "error": "no such task" }))
                }
            }
            WorkerMessage::QueryStatus => {
                Ok(serde_json::json!({
                    "current_task": state.current_task.map(|t| t.to_string()),
                    "tasks_completed": state.tasks_completed,
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
    async fn assign_task_then_query_shows_completed() {
        let mut agent = WorkerAgent::new(AgentId::new());
        let mut state = WorkerState::default();

        let task_id = TaskId::new();
        let input = serde_json::json!({"key": "value"});

        // Assign a task — completes immediately.
        let resp = agent
            .handle_message(
                WorkerMessage::AssignTask {
                    task_id,
                    input: input.clone(),
                },
                &mut state,
            )
            .await
            .expect("AssignTask should succeed");

        assert_eq!(resp["accepted"], task_id.to_string());
        assert_eq!(resp["input"], input);

        // Query status — should reflect one completed task and no current task.
        let status = agent
            .handle_message(WorkerMessage::QueryStatus, &mut state)
            .await
            .expect("QueryStatus should succeed");

        assert_eq!(status["tasks_completed"], 1);
        assert!(status["current_task"].is_null());
    }

    #[tokio::test]
    async fn cancel_nonexistent_task_returns_error() {
        let mut agent = WorkerAgent::new(AgentId::new());
        let mut state = WorkerState::default();

        let resp = agent
            .handle_message(WorkerMessage::CancelTask(TaskId::new()), &mut state)
            .await
            .expect("CancelTask should succeed");

        assert_eq!(resp["error"], "no such task");
    }

    #[tokio::test]
    async fn cancel_current_task_clears_it() {
        let mut agent = WorkerAgent::new(AgentId::new());
        let mut state = WorkerState::default();

        let task_id = TaskId::new();
        state.current_task = Some(task_id);

        let resp = agent
            .handle_message(WorkerMessage::CancelTask(task_id), &mut state)
            .await
            .expect("CancelTask should succeed");

        assert_eq!(resp["cancelled"], task_id.to_string());
        assert!(state.current_task.is_none());
    }
}
