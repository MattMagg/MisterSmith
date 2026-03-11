use std::sync::Arc;

use dashmap::DashMap;
use mister_smith_core::{AgentId, GuardDecision, InterventionRecord, ProfileSnapshot, TaskId};
use tracing::instrument;

use crate::config::TaskState;
use crate::errors::AgentSystemError;
use crate::execution_graph::ExecutionGraph;
use crate::guard::{Guard, GuardContext, GuardPolicy};
use crate::intervention::InterventionEngine;
use crate::profile::ProfileAssessment;
use crate::roles::monitor::{MonitorMessage, MonitorState};
use crate::roles::planner::planner_output_from_subtasks;
use crate::roles::supervisor::{SupervisorMessage, SupervisorState};
use crate::scheduler::{ResultAggregator, TaskAssignment, TaskDecomposer, TaskScheduler};
use crate::topology::{TopologyCompiler, TopologySignals};
use mister_smith_events::{
    AutonomyStatusView, BranchSummary, ContextPressureSummary, DelegationAlert,
    ExecutionGraphSummary, TopologyPlanSummary,
};

#[cfg(feature = "llm")]
use mister_smith_core::GuardTarget;
#[cfg(feature = "llm")]
use mister_smith_llm::router::ConfidenceSignal;
#[cfg(feature = "llm")]
use mister_smith_llm::{CompletionResponse, ModelEvent, StreamMonitor, StreamMonitorConfig};
#[cfg(feature = "llm")]
use tokio::sync::Mutex;

/// Orchestrator holds decomposer, aggregator, team, and scheduler
/// to manage the full lifecycle of a complex task.
pub struct Orchestrator {
    decomposer: Arc<dyn TaskDecomposer>,
    aggregator: Arc<dyn ResultAggregator>,
    scheduler: Arc<TaskScheduler>,
    topology_compiler: TopologyCompiler,
    topology_signals: TopologySignals,
    guard: Guard,
    intervention_engine: InterventionEngine,
    execution_graphs: DashMap<TaskId, ExecutionGraph>,
    guard_decisions: DashMap<TaskId, Vec<GuardDecision>>,
    interventions: DashMap<TaskId, Vec<InterventionRecord>>,
    profiles: DashMap<TaskId, Vec<ProfileAssessment>>,
    conservative_reasons: DashMap<TaskId, Vec<String>>,
    monitor_states: DashMap<TaskId, MonitorState>,
    supervisor_states: DashMap<TaskId, SupervisorState>,
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
            guard: Guard::new(GuardPolicy::default()),
            intervention_engine: InterventionEngine,
            execution_graphs: DashMap::new(),
            guard_decisions: DashMap::new(),
            interventions: DashMap::new(),
            profiles: DashMap::new(),
            conservative_reasons: DashMap::new(),
            monitor_states: DashMap::new(),
            supervisor_states: DashMap::new(),
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

    /// Register a precompiled execution graph for later supervision or inspection.
    pub fn register_execution_graph(&self, graph: ExecutionGraph) {
        self.execution_graphs.insert(graph.workflow_id, graph);
    }

    /// Evaluate and apply a typed Guard decision against an existing workflow.
    pub async fn supervise(
        &self,
        workflow_id: &TaskId,
        context: GuardContext,
    ) -> Result<(GuardDecision, InterventionRecord), AgentSystemError> {
        let decision = self.guard.evaluate(&context)?;
        let record = {
            let mut graph = self.execution_graphs.get_mut(workflow_id).ok_or_else(|| {
                AgentSystemError::OrchestrationError(format!(
                    "No execution graph found for workflow {workflow_id}"
                ))
            })?;
            self.intervention_engine
                .apply(&decision, &self.scheduler, graph.value_mut())?
        };

        self.guard_decisions
            .entry(*workflow_id)
            .or_default()
            .push(decision.clone());
        self.interventions
            .entry(*workflow_id)
            .or_default()
            .push(record.clone());
        if let Some(profile) = context.profile().cloned() {
            self.profiles.entry(*workflow_id).or_default().push(profile);
        }
        let conservative = decision
            .evidence
            .notes
            .iter()
            .filter(|note| note.contains("conservative fallback"))
            .cloned()
            .collect::<Vec<_>>();
        if !conservative.is_empty() {
            self.conservative_reasons
                .entry(*workflow_id)
                .or_default()
                .extend(conservative);
        }
        self.record_monitor_message(
            workflow_id,
            &MonitorMessage::GuardDecisionEvaluated(decision.clone()),
        );
        self.record_monitor_message(
            workflow_id,
            &MonitorMessage::InterventionApplied(record.clone()),
        );
        self.record_supervisor_message(
            workflow_id,
            &SupervisorMessage::RecordGuardDecision(decision.clone()),
        );
        self.record_supervisor_message(
            workflow_id,
            &SupervisorMessage::RecordIntervention(record.clone()),
        );

        Ok((decision, record))
    }

    /// Build the operator-visible autonomy status for a workflow.
    pub fn autonomy_status(&self, workflow_id: &TaskId) -> Option<AutonomyStatusView> {
        let graph = self.execution_graph(workflow_id)?;
        let profiles = self
            .profiles
            .get(workflow_id)
            .map(|profiles| {
                profiles
                    .iter()
                    .filter_map(ProfileAssessment::snapshot)
                    .cloned()
                    .collect::<Vec<ProfileSnapshot>>()
            })
            .unwrap_or_default();
        let guard_decisions = self
            .monitor_state(workflow_id)
            .map(|state| state.guard_decisions)
            .or_else(|| {
                self.guard_decisions
                    .get(workflow_id)
                    .map(|decisions| decisions.value().clone())
            })
            .unwrap_or_default();
        let interventions = self
            .monitor_state(workflow_id)
            .map(|state| state.interventions)
            .or_else(|| {
                self.interventions
                    .get(workflow_id)
                    .map(|records| records.value().clone())
            })
            .unwrap_or_default();
        let conservative_reasons = self
            .conservative_reasons
            .get(workflow_id)
            .map(|notes| notes.value().clone())
            .unwrap_or_default();

        Some(AutonomyStatusView {
            graph: ExecutionGraphSummary {
                graph_id: graph.graph_id,
                workflow_id: graph.workflow_id,
                state: graph.state,
                branch_count: graph.branches.len(),
                node_count: graph.nodes.len(),
                active_topology: Some(graph.topology_plan.topology_kind),
            },
            topology: TopologyPlanSummary {
                graph_id: graph.graph_id,
                topology_kind: graph.topology_plan.topology_kind,
                parallelism_width: graph.topology_plan.parallelism_width,
                coordination_policy: graph.topology_plan.coordination_policy,
                rationale: graph.topology_plan.rationale.clone(),
                fallback_topology: graph.topology_plan.fallback_topology,
            },
            branches: graph
                .branches
                .iter()
                .map(|branch| BranchSummary {
                    branch_id: branch.branch_id,
                    graph_id: branch.graph_id,
                    state: branch.state,
                    assigned_agents: branch.assigned_agents.clone(),
                    checkpoint_id: graph
                        .checkpoint_lineage
                        .iter()
                        .find(|checkpoint| checkpoint.branch_id == branch.branch_id)
                        .map(|checkpoint| checkpoint.checkpoint_id),
                    recovery_strategy: branch.recovery_strategy,
                })
                .collect(),
            memory_pressure: Vec::<ContextPressureSummary>::new(),
            interventions,
            delegation_alerts: Vec::<DelegationAlert>::new(),
            profiles,
            guard_decisions,
            conservative_reasons,
        })
    }

    /// Return the latest monitor state for a workflow, when available.
    pub fn monitor_state(&self, workflow_id: &TaskId) -> Option<MonitorState> {
        self.monitor_states
            .get(workflow_id)
            .map(|state| state.value().clone())
    }

    /// Return the latest supervisor state for a workflow, when available.
    pub fn supervisor_state(&self, workflow_id: &TaskId) -> Option<SupervisorState> {
        self.supervisor_states
            .get(workflow_id)
            .map(|state| state.value().clone())
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

    fn record_monitor_message(&self, workflow_id: &TaskId, message: &MonitorMessage) {
        let mut state = self.monitor_states.entry(*workflow_id).or_default();
        state.value_mut().apply(message);
    }

    fn record_supervisor_message(&self, workflow_id: &TaskId, message: &SupervisorMessage) {
        let mut state = self.supervisor_states.entry(*workflow_id).or_default();
        state.value_mut().apply(message);
    }

    #[cfg(feature = "llm")]
    fn checkpoints_for_target(
        &self,
        workflow_id: &TaskId,
        target: &GuardTarget,
    ) -> Vec<crate::execution_graph::BranchCheckpoint> {
        let Some(graph) = self.execution_graph(workflow_id) else {
            return Vec::new();
        };

        match target {
            GuardTarget::Branch(branch_id) => graph
                .checkpoint_lineage
                .into_iter()
                .filter(|checkpoint| checkpoint.branch_id == *branch_id)
                .collect(),
            GuardTarget::Node(node_id) => graph
                .nodes
                .iter()
                .find(|node| node.node_id == *node_id)
                .map(|node| {
                    graph
                        .checkpoint_lineage
                        .into_iter()
                        .filter(|checkpoint| checkpoint.branch_id == node.branch_id)
                        .collect()
                })
                .unwrap_or_default(),
            GuardTarget::Graph(_) => graph.checkpoint_lineage,
            GuardTarget::Provider(_) => Vec::new(),
        }
    }
}

#[cfg(feature = "llm")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmSupervisionConfig {
    target: GuardTarget,
    control_plane_fresh: bool,
    memory_metadata_available: bool,
}

#[cfg(feature = "llm")]
impl LlmSupervisionConfig {
    pub fn new(target: GuardTarget) -> Self {
        Self {
            target,
            control_plane_fresh: true,
            memory_metadata_available: true,
        }
    }

    pub fn with_control_plane_fresh(mut self, fresh: bool) -> Self {
        self.control_plane_fresh = fresh;
        self
    }

    pub fn with_memory_metadata_available(mut self, available: bool) -> Self {
        self.memory_metadata_available = available;
        self
    }
}

#[cfg(feature = "llm")]
#[derive(Clone)]
pub struct LlmSupervision {
    orchestrator: Arc<Orchestrator>,
    workflow_id: TaskId,
    config: LlmSupervisionConfig,
    stream_monitor: Arc<Mutex<StreamMonitor>>,
    request_id: String,
}

#[cfg(feature = "llm")]
impl LlmSupervision {
    pub fn new(
        orchestrator: Arc<Orchestrator>,
        workflow_id: TaskId,
        config: LlmSupervisionConfig,
    ) -> Self {
        Self {
            orchestrator,
            workflow_id,
            config,
            stream_monitor: Arc::new(Mutex::new(StreamMonitor::new(
                StreamMonitorConfig::default(),
            ))),
            request_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub async fn request_started(&self, model_id: &str) -> Result<(), AgentSystemError> {
        let _ = self
            .observe_model_event(&ModelEvent::StreamStarted {
                model_id: model_id.to_string(),
                request_id: self.request_id.clone(),
            })
            .await?;
        Ok(())
    }

    pub async fn completion_succeeded(
        &self,
        response: &CompletionResponse,
    ) -> Result<Option<(GuardDecision, InterventionRecord)>, AgentSystemError> {
        let mut latest = None;
        for block in &response.content {
            if let mister_smith_llm::ContentBlock::Text { text } = block {
                latest = self
                    .observe_model_event(&ModelEvent::TextCompleted {
                        full_text: text.clone(),
                    })
                    .await?
                    .or(latest);
            }
        }

        let confidence = ConfidenceSignal::from_response(response);
        latest = self
            .observe_model_event(&ModelEvent::TextAnnotation {
                annotation: serde_json::json!({ "confidence": confidence.score }),
            })
            .await?
            .or(latest);
        latest = self
            .observe_model_event(&ModelEvent::StreamCompleted {
                usage: response.usage,
                stop_reason: response.stop_reason.clone(),
            })
            .await?
            .or(latest);

        Ok(latest)
    }

    pub async fn completion_failed(
        &self,
        error: &mister_smith_core::LlmError,
    ) -> Result<Option<(GuardDecision, InterventionRecord)>, AgentSystemError> {
        let event = model_event_for_error(error);
        self.observe_model_event(&event).await
    }

    /// Feed a canonical model event through the stream monitor and trigger Guard
    /// supervision when the event yields degradation signals.
    pub async fn observe_model_event(
        &self,
        event: &ModelEvent,
    ) -> Result<Option<(GuardDecision, InterventionRecord)>, AgentSystemError> {
        let observation = {
            let mut monitor = self.stream_monitor.lock().await;
            monitor.observe(event)
        };

        if observation.degradation_signals.is_empty() {
            return Ok(None);
        }

        let notes = observation
            .step_boundaries
            .into_iter()
            .map(|boundary| format!("step boundary observed: {boundary:?}"))
            .collect::<Vec<_>>();
        let assessment = ProfileAssessment::from_supervisory_signals(
            &self.config.target,
            observation.degradation_signals,
            notes,
        );
        let context = GuardContext::new(self.config.target.clone())
            .with_profile(assessment)
            .with_checkpoints(
                self.orchestrator
                    .checkpoints_for_target(&self.workflow_id, &self.config.target),
            )
            .with_control_plane_fresh(self.config.control_plane_fresh)
            .with_memory_metadata_available(self.config.memory_metadata_available);

        self.orchestrator
            .supervise(&self.workflow_id, context)
            .await
            .map(Some)
    }
}

#[cfg(feature = "llm")]
fn model_event_for_error(error: &mister_smith_core::LlmError) -> ModelEvent {
    let (code, recoverable) = match error {
        mister_smith_core::LlmError::ProviderError { retryable, .. } => {
            ("provider_error", *retryable)
        }
        mister_smith_core::LlmError::RateLimited { .. } => ("rate_limited", true),
        mister_smith_core::LlmError::Serialization(_) => ("serialization", false),
        mister_smith_core::LlmError::Network(_) => ("network", true),
        mister_smith_core::LlmError::UnsupportedCapability { .. } => {
            ("unsupported_capability", false)
        }
        mister_smith_core::LlmError::InvalidRequest(_) => ("invalid_request", false),
        mister_smith_core::LlmError::Authentication(_) => ("authentication", false),
        mister_smith_core::LlmError::BudgetExhausted { .. } => ("budget_exhausted", false),
        mister_smith_core::LlmError::NoHealthyProvider(_) => ("no_healthy_provider", true),
    };

    ModelEvent::Error {
        code: code.to_string(),
        message: error.to_string(),
        recoverable,
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
