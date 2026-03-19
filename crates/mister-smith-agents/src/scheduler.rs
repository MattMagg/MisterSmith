use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use mister_smith_core::{AgentId, InterventionType, TaskId};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::config::TaskState;
use crate::errors::AgentSystemError;

/// A task submitted to the scheduling system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    pub task_id: TaskId,
    pub task_type: String,
    pub priority: u8,
    pub deadline: Option<DateTime<Utc>>,
    pub input: serde_json::Value,
    pub output: Option<serde_json::Value>,
    pub state: TaskState,
    pub assigned_to: Option<AgentId>,
    pub parent_task_id: Option<TaskId>,
    pub team_id: Option<uuid::Uuid>,
    pub message_id: uuid::Uuid,
    pub created_at: DateTime<Utc>,
    pub assigned_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl TaskAssignment {
    /// Create a new task assignment in Pending state.
    pub fn new(task_type: impl Into<String>, input: serde_json::Value) -> Self {
        Self {
            task_id: TaskId::new(),
            task_type: task_type.into(),
            priority: 128,
            deadline: None,
            input,
            output: None,
            state: TaskState::Pending,
            assigned_to: None,
            parent_task_id: None,
            team_id: None,
            message_id: uuid::Uuid::new_v4(),
            created_at: Utc::now(),
            assigned_at: None,
            completed_at: None,
            error_message: None,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn with_parent(mut self, parent_id: TaskId) -> Self {
        self.parent_task_id = Some(parent_id);
        self
    }
}

/// Pluggable trait for decomposing a task into subtasks.
#[async_trait::async_trait]
pub trait TaskDecomposer: Send + Sync {
    async fn decompose(
        &self,
        task: &TaskAssignment,
    ) -> Result<Vec<TaskAssignment>, AgentSystemError>;
}

/// Pluggable trait for aggregating subtask results.
#[async_trait::async_trait]
pub trait ResultAggregator: Send + Sync {
    async fn aggregate(
        &self,
        results: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, AgentSystemError>;
}

/// Default decomposer — returns the task as-is (no decomposition).
pub struct IdentityDecomposer;

#[async_trait::async_trait]
impl TaskDecomposer for IdentityDecomposer {
    async fn decompose(
        &self,
        task: &TaskAssignment,
    ) -> Result<Vec<TaskAssignment>, AgentSystemError> {
        Ok(vec![task.clone()])
    }
}

/// Default aggregator — collects results into a JSON array.
pub struct ArrayAggregator;

#[async_trait::async_trait]
impl ResultAggregator for ArrayAggregator {
    async fn aggregate(
        &self,
        results: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value, AgentSystemError> {
        Ok(serde_json::Value::Array(results))
    }
}

/// Tracks active tasks and manages state transitions.
pub struct TaskScheduler {
    tasks: Arc<DashMap<TaskId, TaskAssignment>>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(DashMap::new()),
        }
    }

    /// Submit a new task (transitions to Pending).
    pub fn submit(&self, task: TaskAssignment) -> TaskId {
        let id = task.task_id;
        self.tasks.insert(id, task);
        id
    }

    /// Assign a task to an agent (Pending → Assigned).
    pub fn assign(&self, task_id: &TaskId, agent_id: AgentId) -> Result<(), AgentSystemError> {
        self.assign_with_team(task_id, agent_id, None)
    }

    /// Assign a task to an agent and retain the adaptive team that owns it.
    pub fn assign_to_team(
        &self,
        task_id: &TaskId,
        agent_id: AgentId,
        team_id: Uuid,
    ) -> Result<(), AgentSystemError> {
        self.assign_with_team(task_id, agent_id, Some(team_id))
    }

    fn assign_with_team(
        &self,
        task_id: &TaskId,
        agent_id: AgentId,
        team_id: Option<Uuid>,
    ) -> Result<(), AgentSystemError> {
        let mut entry = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| AgentSystemError::SchedulingError("Task not found".into()))?;

        if entry.state != TaskState::Pending {
            return Err(AgentSystemError::SchedulingError(format!(
                "Cannot assign task in {:?} state",
                entry.state
            )));
        }

        entry.state = TaskState::Assigned;
        entry.assigned_to = Some(agent_id);
        entry.team_id = team_id;
        entry.assigned_at = Some(Utc::now());
        Ok(())
    }

    /// Mark a task as running (Assigned → Running).
    pub fn start(&self, task_id: &TaskId) -> Result<(), AgentSystemError> {
        let mut entry = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| AgentSystemError::SchedulingError("Task not found".into()))?;

        if entry.state != TaskState::Assigned {
            return Err(AgentSystemError::SchedulingError(format!(
                "Cannot start task in {:?} state",
                entry.state
            )));
        }

        entry.state = TaskState::Running;
        Ok(())
    }

    /// Complete a task with a result (Running → Completed).
    pub fn complete(
        &self,
        task_id: &TaskId,
        result: serde_json::Value,
    ) -> Result<(), AgentSystemError> {
        let mut entry = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| AgentSystemError::SchedulingError("Task not found".into()))?;

        if entry.state != TaskState::Running {
            return Err(AgentSystemError::SchedulingError(format!(
                "Cannot complete task in {:?} state",
                entry.state
            )));
        }

        entry.state = TaskState::Completed;
        entry.output = Some(result);
        entry.completed_at = Some(Utc::now());
        Ok(())
    }

    /// Fail a task (Running → Failed).
    pub fn fail(&self, task_id: &TaskId, error: impl Into<String>) -> Result<(), AgentSystemError> {
        let mut entry = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| AgentSystemError::SchedulingError("Task not found".into()))?;

        entry.state = TaskState::Failed;
        entry.error_message = Some(error.into());
        entry.completed_at = Some(Utc::now());
        Ok(())
    }

    /// Mark a task as timed out.
    pub fn timeout(&self, task_id: &TaskId) -> Result<(), AgentSystemError> {
        let mut entry = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| AgentSystemError::SchedulingError("Task not found".into()))?;

        entry.state = TaskState::TimedOut;
        entry.completed_at = Some(Utc::now());
        Ok(())
    }

    /// Cancel a task without widening the failure scope to sibling work.
    pub fn cancel(&self, task_id: &TaskId) -> Result<(), AgentSystemError> {
        let mut entry = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| AgentSystemError::SchedulingError("Task not found".into()))?;

        entry.state = TaskState::Cancelled;
        entry.completed_at = Some(Utc::now());
        Ok(())
    }

    /// Reset a task to Pending for reassignment.
    pub fn reset(&self, task_id: &TaskId) -> Result<(), AgentSystemError> {
        let mut entry = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| AgentSystemError::SchedulingError("Task not found".into()))?;

        entry.state = TaskState::Pending;
        entry.assigned_to = None;
        entry.team_id = None;
        entry.assigned_at = None;
        entry.output = None;
        entry.completed_at = None;
        entry.error_message = None;
        Ok(())
    }

    /// Apply a typed Guard intervention to a single scheduled task.
    pub fn apply_intervention(
        &self,
        task_id: &TaskId,
        intervention: InterventionType,
    ) -> Result<(), AgentSystemError> {
        match intervention {
            InterventionType::Retry
            | InterventionType::Failover
            | InterventionType::ContextRefresh
            | InterventionType::Reassignment => self.reset(task_id),
            InterventionType::BranchIsolation | InterventionType::Escalation => {
                self.cancel(task_id)
            }
            InterventionType::Abort => self.fail(task_id, "aborted by guard intervention"),
        }
    }

    /// Get a task by ID.
    pub fn get(&self, task_id: &TaskId) -> Option<TaskAssignment> {
        self.tasks.get(task_id).map(|e| e.value().clone())
    }

    /// Get all tasks for a parent task (subtasks).
    pub fn subtasks(&self, parent_id: &TaskId) -> Vec<TaskAssignment> {
        self.tasks
            .iter()
            .filter_map(|e| {
                if e.value().parent_task_id.as_ref() == Some(parent_id) {
                    Some(e.value().clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all subtasks for a parent task in a specific state without unnecessary allocations.
    pub fn subtasks_in_state(&self, parent_id: &TaskId, state: TaskState) -> Vec<TaskAssignment> {
        self.tasks
            .iter()
            .filter_map(|e| {
                if e.value().parent_task_id.as_ref() == Some(parent_id) && e.value().state == state
                {
                    Some(e.value().clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all subtasks for a parent task in any of the specified states without unnecessary allocations.
    pub fn subtasks_in_states(
        &self,
        parent_id: &TaskId,
        states: &[TaskState],
    ) -> Vec<TaskAssignment> {
        self.tasks
            .iter()
            .filter_map(|e| {
                if e.value().parent_task_id.as_ref() == Some(parent_id)
                    && states.contains(&e.value().state)
                {
                    Some(e.value().clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get completed subtask outputs without cloning full task records.
    pub fn completed_subtask_outputs(&self, parent_id: &TaskId) -> Vec<serde_json::Value> {
        self.tasks
            .iter()
            .filter_map(|e| {
                let task = e.value();
                if task.parent_task_id.as_ref() == Some(parent_id)
                    && task.state == TaskState::Completed
                {
                    task.output.clone()
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if all subtasks for a given parent task are complete without allocating a Vec.
    pub fn all_subtasks_completed(&self, parent_id: &TaskId) -> bool {
        let mut has_subtasks = false;
        let mut all_completed = true;

        for entry in self.tasks.iter() {
            let task = entry.value();
            if task.parent_task_id.as_ref() == Some(parent_id) {
                has_subtasks = true;
                if task.state != TaskState::Completed {
                    all_completed = false;
                    break;
                }
            }
        }

        has_subtasks && all_completed
    }

    /// Get count of tracked tasks.
    pub fn count(&self) -> usize {
        self.tasks.len()
    }

    /// Get all tasks in a specific state.
    pub fn tasks_in_state(&self, state: TaskState) -> Vec<TaskAssignment> {
        self.tasks
            .iter()
            .filter_map(|e| {
                if e.value().state == state {
                    Some(e.value().clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all tracked tasks.
    pub fn all_tasks(&self) -> Vec<TaskAssignment> {
        self.tasks.iter().map(|e| e.value().clone()).collect()
    }

    /// Calculate the number of assigned/running tasks per worker without cloning task payloads.
    pub fn worker_loads(
        &self,
        worker_ids: &[AgentId],
    ) -> std::collections::HashMap<AgentId, usize> {
        let mut loads = worker_ids
            .iter()
            .copied()
            .map(|worker_id| (worker_id, 0_usize))
            .collect::<std::collections::HashMap<_, _>>();

        for entry in self.tasks.iter() {
            let task = entry.value();
            if let Some(agent_id) = task.assigned_to {
                if matches!(task.state, TaskState::Assigned | TaskState::Running) {
                    if let Some(load) = loads.get_mut(&agent_id) {
                        *load += 1;
                    }
                }
            }
        }

        loads
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Background task that monitors deadlines and marks expired tasks as TimedOut.
pub struct DeadlineMonitor {
    scheduler: Arc<TaskScheduler>,
    check_interval: Duration,
    handle: Option<JoinHandle<()>>,
    stop_tx: Option<watch::Sender<bool>>,
}

impl DeadlineMonitor {
    pub fn new(scheduler: Arc<TaskScheduler>, check_interval: Duration) -> Self {
        Self {
            scheduler,
            check_interval,
            handle: None,
            stop_tx: None,
        }
    }

    /// Start the deadline monitor background task.
    pub fn start(&mut self) {
        let scheduler = self.scheduler.clone();
        let check_interval = self.check_interval;
        let (stop_tx, mut stop_rx) = watch::channel(false);
        self.stop_tx = Some(stop_tx);

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(check_interval);

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        let now = Utc::now();
                        // Check running and assigned tasks for deadline expiry
                        let active: Vec<(TaskId, chrono::DateTime<Utc>)> = scheduler
                            .tasks
                            .iter()
                            .filter_map(|e| {
                                let task = e.value();
                                if matches!(task.state, TaskState::Running | TaskState::Assigned) {
                                    task.deadline.map(|deadline| (task.task_id, deadline))
                                } else {
                                    None
                                }
                            })
                            .collect();

                        for (task_id, deadline) in active {
                            if now > deadline {
                                let _ = scheduler.timeout(&task_id);
                            }
                        }
                    }
                    _ = stop_rx.changed() => {
                        if *stop_rx.borrow() {
                            break;
                        }
                    }
                }
            }
        });

        self.handle = Some(handle);
    }

    /// Stop the deadline monitor.
    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(true);
        }
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }

    pub fn is_running(&self) -> bool {
        self.handle
            .as_ref()
            .map(|h| !h.is_finished())
            .unwrap_or(false)
    }
}

impl Drop for DeadlineMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_lifecycle() {
        let scheduler = TaskScheduler::new();
        let task = TaskAssignment::new("analysis", serde_json::json!({"data": "test"}));
        let task_id = task.task_id;
        let agent_id = AgentId::new();

        scheduler.submit(task);
        assert_eq!(scheduler.get(&task_id).unwrap().state, TaskState::Pending);

        scheduler.assign(&task_id, agent_id).unwrap();
        assert_eq!(scheduler.get(&task_id).unwrap().state, TaskState::Assigned);

        scheduler.start(&task_id).unwrap();
        assert_eq!(scheduler.get(&task_id).unwrap().state, TaskState::Running);

        scheduler
            .complete(&task_id, serde_json::json!({"result": "done"}))
            .unwrap();
        let completed = scheduler.get(&task_id).unwrap();
        assert_eq!(completed.state, TaskState::Completed);
        assert!(completed.output.is_some());
        assert!(completed.completed_at.is_some());
    }

    #[test]
    fn test_task_failure() {
        let scheduler = TaskScheduler::new();
        let task = TaskAssignment::new("analysis", serde_json::json!({}));
        let task_id = task.task_id;
        let agent_id = AgentId::new();

        scheduler.submit(task);
        scheduler.assign(&task_id, agent_id).unwrap();
        scheduler.start(&task_id).unwrap();
        scheduler.fail(&task_id, "something broke").unwrap();

        let failed = scheduler.get(&task_id).unwrap();
        assert_eq!(failed.state, TaskState::Failed);
        assert_eq!(failed.error_message.as_deref(), Some("something broke"));
    }

    #[test]
    fn test_task_reset() {
        let scheduler = TaskScheduler::new();
        let task = TaskAssignment::new("analysis", serde_json::json!({}));
        let task_id = task.task_id;
        let agent_id = AgentId::new();

        scheduler.submit(task);
        scheduler.assign(&task_id, agent_id).unwrap();
        scheduler.start(&task_id).unwrap();
        scheduler.fail(&task_id, "transient error").unwrap();

        // Reset for retry
        scheduler.reset(&task_id).unwrap();
        let reset = scheduler.get(&task_id).unwrap();
        assert_eq!(reset.state, TaskState::Pending);
        assert!(reset.assigned_to.is_none());
        assert!(reset.team_id.is_none());
    }

    #[test]
    fn test_assign_to_team_records_membership() {
        let scheduler = TaskScheduler::new();
        let task = TaskAssignment::new("analysis", serde_json::json!({}));
        let task_id = task.task_id;
        let agent_id = AgentId::new();
        let team_id = Uuid::new_v4();

        scheduler.submit(task);
        scheduler
            .assign_to_team(&task_id, agent_id, team_id)
            .unwrap();

        let assigned = scheduler.get(&task_id).unwrap();
        assert_eq!(assigned.assigned_to, Some(agent_id));
        assert_eq!(assigned.team_id, Some(team_id));
    }

    #[tokio::test]
    async fn test_identity_decomposer() {
        let decomposer = IdentityDecomposer;
        let task = TaskAssignment::new("test", serde_json::json!({}));
        let subtasks = decomposer.decompose(&task).await.unwrap();
        assert_eq!(subtasks.len(), 1);
    }

    #[tokio::test]
    async fn test_array_aggregator() {
        let agg = ArrayAggregator;
        let results = vec![serde_json::json!({"a": 1}), serde_json::json!({"b": 2})];
        let combined = agg.aggregate(results).await.unwrap();
        assert!(combined.is_array());
        assert_eq!(combined.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_completed_subtask_outputs_filters_to_completed_children() {
        let scheduler = TaskScheduler::new();
        let parent_id = TaskId::new();
        let agent_id = AgentId::new();

        let completed_child =
            TaskAssignment::new("child", serde_json::json!({})).with_parent(parent_id);
        let completed_child_id = completed_child.task_id;
        scheduler.submit(completed_child);
        scheduler.assign(&completed_child_id, agent_id).unwrap();
        scheduler.start(&completed_child_id).unwrap();
        scheduler
            .complete(&completed_child_id, serde_json::json!({"result": "done"}))
            .unwrap();

        let pending_child =
            TaskAssignment::new("child", serde_json::json!({})).with_parent(parent_id);
        scheduler.submit(pending_child);

        let other_parent_child =
            TaskAssignment::new("child", serde_json::json!({})).with_parent(TaskId::new());
        let other_parent_child_id = other_parent_child.task_id;
        scheduler.submit(other_parent_child);
        scheduler.assign(&other_parent_child_id, agent_id).unwrap();
        scheduler.start(&other_parent_child_id).unwrap();
        scheduler
            .complete(
                &other_parent_child_id,
                serde_json::json!({"result": "other"}),
            )
            .unwrap();

        let outputs = scheduler.completed_subtask_outputs(&parent_id);

        assert_eq!(outputs, vec![serde_json::json!({"result": "done"})]);
    }
}
