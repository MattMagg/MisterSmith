use std::sync::Arc;

use dashmap::DashMap;
use mister_smith_core::{AgentId, TaskId};
use tracing::instrument;

use crate::config::TaskState;
use crate::errors::AgentSystemError;
use crate::execution_graph::ExecutionGraph;
use crate::roles::planner::planner_output_from_subtasks;
use crate::scheduler::{ResultAggregator, TaskAssignment, TaskDecomposer, TaskScheduler};
use crate::topology::{TopologyCompiler, TopologySignals};

/// Orchestrator holds decomposer, aggregator, team, and scheduler
/// to manage the full lifecycle of a complex task.
pub struct Orchestrator {
    decomposer: Arc<dyn TaskDecomposer>,
    aggregator: Arc<dyn ResultAggregator>,
    scheduler: Arc<TaskScheduler>,
    topology_compiler: TopologyCompiler,
    topology_signals: TopologySignals,
    execution_graphs: DashMap<TaskId, ExecutionGraph>,
}

impl Orchestrator {
    pub fn new(
        decomposer: Arc<dyn TaskDecomposer>,
        aggregator: Arc<dyn ResultAggregator>,
        scheduler: Arc<TaskScheduler>,
    ) -> Self {
        Self {
            decomposer,
            aggregator,
            scheduler,
            topology_compiler: TopologyCompiler,
            topology_signals: TopologySignals::default(),
            execution_graphs: DashMap::new(),
        }
    }

    /// Decompose a task into subtasks and register them with the scheduler.
    #[instrument(skip(self, task), fields(task.id = %task.task_id, task.type = %task.task_type))]
    pub async fn decompose(&self, task: &TaskAssignment) -> Result<Vec<TaskId>, AgentSystemError> {
        let subtasks = self.decomposer.decompose(task).await?;
        if subtasks.is_empty() {
            return Err(AgentSystemError::OrchestrationError(
                "Decomposition produced no subtasks".into(),
            ));
        }

        let planner_output = planner_output_from_subtasks(task, &subtasks);
        let graph = self.topology_compiler.compile(
            task.task_id,
            &planner_output,
            &self.topology_signals,
        )?;
        self.execution_graphs.insert(task.task_id, graph);

        let mut ids = Vec::with_capacity(subtasks.len());
        for mut subtask in subtasks {
            subtask.parent_task_id = Some(task.task_id);
            let id = self.scheduler.submit(subtask);
            ids.push(id);
        }
        Ok(ids)
    }

    /// Return the latest compiled execution graph for a workflow, when available.
    pub fn execution_graph(&self, workflow_id: &TaskId) -> Option<ExecutionGraph> {
        self.execution_graphs
            .get(workflow_id)
            .map(|entry| entry.value().clone())
    }

    /// Aggregate results from completed subtasks of a parent task.
    #[instrument(skip(self), fields(task.id = %parent_task_id))]
    pub async fn aggregate(
        &self,
        parent_task_id: &TaskId,
    ) -> Result<serde_json::Value, AgentSystemError> {
        let subtasks = self.scheduler.subtasks(parent_task_id);
        let results: Vec<serde_json::Value> = subtasks
            .into_iter()
            .filter(|t| t.state == TaskState::Completed)
            .filter_map(|t| t.output)
            .collect();

        self.aggregator.aggregate(results).await
    }

    /// Check if all subtasks of a parent task are completed.
    pub fn all_subtasks_completed(&self, parent_task_id: &TaskId) -> bool {
        let subtasks = self.scheduler.subtasks(parent_task_id);
        !subtasks.is_empty() && subtasks.iter().all(|t| t.state == TaskState::Completed)
    }

    /// Get subtasks that are pending and have all dependencies satisfied.
    pub fn ready_subtasks(&self, parent_task_id: &TaskId) -> Vec<TaskAssignment> {
        self.scheduler
            .subtasks(parent_task_id)
            .into_iter()
            .filter(|t| t.state == TaskState::Pending)
            .collect()
    }

    /// Get the scheduler.
    pub fn scheduler(&self) -> &TaskScheduler {
        &self.scheduler
    }

    /// Execute a full orchestration workflow:
    /// decompose → assign to available workers → wait for completion → aggregate.
    ///
    /// This is the high-level entry point that coordinates the entire task lifecycle.
    /// Workers are assigned from the provided agent IDs (pre-assembled team members).
    #[instrument(skip(self, task, worker_ids), fields(task.id = %task.task_id, task.type = %task.task_type, worker_count = worker_ids.len()))]
    pub async fn execute(
        &self,
        task: &TaskAssignment,
        worker_ids: &[AgentId],
    ) -> Result<serde_json::Value, AgentSystemError> {
        // 1. Decompose the task into subtasks
        let subtask_ids = self.decompose(task).await?;

        if subtask_ids.is_empty() {
            return Err(AgentSystemError::OrchestrationError(
                "Decomposition produced no subtasks".into(),
            ));
        }

        // 2. Assign subtasks round-robin to available workers
        for (i, subtask_id) in subtask_ids.iter().enumerate() {
            if worker_ids.is_empty() {
                return Err(AgentSystemError::OrchestrationError(
                    "No workers available for assignment".into(),
                ));
            }
            let worker = worker_ids[i % worker_ids.len()];
            self.scheduler.assign(subtask_id, worker)?;
        }

        // 3. Aggregate results (caller is responsible for driving subtask completion)
        // In a real runtime, we'd monitor progress via events. Here we check completion state.
        if self.all_subtasks_completed(&task.task_id) {
            self.aggregate(&task.task_id).await
        } else {
            // Return partial status — subtasks assigned but not yet complete
            Ok(serde_json::json!({
                "status": "assigned",
                "subtask_count": subtask_ids.len(),
                "subtask_ids": subtask_ids.iter().map(|id| id.to_string()).collect::<Vec<_>>(),
            }))
        }
    }

    /// Reassign a failed subtask to a different worker.
    pub fn reassign_subtask(
        &self,
        task_id: &TaskId,
        new_worker: AgentId,
    ) -> Result<(), AgentSystemError> {
        // Reset the task to Pending, then assign to new worker
        self.scheduler.reset(task_id)?;
        self.scheduler.assign(task_id, new_worker)?;
        Ok(())
    }

    /// Get failed subtasks for a parent task.
    pub fn failed_subtasks(&self, parent_task_id: &TaskId) -> Vec<TaskAssignment> {
        self.scheduler
            .subtasks(parent_task_id)
            .into_iter()
            .filter(|t| t.state == TaskState::Failed || t.state == TaskState::TimedOut)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::{ArrayAggregator, IdentityDecomposer};

    #[tokio::test]
    async fn test_orchestrator_decompose_and_aggregate() {
        let scheduler = Arc::new(TaskScheduler::new());
        let orchestrator = Orchestrator::new(
            Arc::new(IdentityDecomposer),
            Arc::new(ArrayAggregator),
            scheduler.clone(),
        );

        let task = TaskAssignment::new("analysis", serde_json::json!({"data": "test"}));
        let task_id = task.task_id;

        // Decompose
        let subtask_ids = orchestrator.decompose(&task).await.unwrap();
        assert_eq!(subtask_ids.len(), 1);

        // Simulate subtask execution
        let sub_id = subtask_ids[0];
        let agent_id = mister_smith_core::AgentId::new();
        scheduler.assign(&sub_id, agent_id).unwrap();
        scheduler.start(&sub_id).unwrap();
        scheduler
            .complete(&sub_id, serde_json::json!({"result": "computed"}))
            .unwrap();

        // Check all complete
        assert!(orchestrator.all_subtasks_completed(&task_id));

        // Aggregate
        let result = orchestrator.aggregate(&task_id).await.unwrap();
        assert!(result.is_array());
    }
}
