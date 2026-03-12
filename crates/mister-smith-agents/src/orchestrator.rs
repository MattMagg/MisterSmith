use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use mister_smith_core::{
    AgentId, BranchRecoveryStrategy, BranchState, CheckpointId, ExecutionBranchId, ExecutionNodeId,
    GuardDecision, GuardTarget, HealthState, InterventionRecord, ProfileSnapshot, TaskId,
};
use tracing::instrument;

use crate::branch_checkpoint::{
    BranchCheckpointCoordinator, BranchCheckpointStore, BranchRecoveryPlan,
};
use crate::config::TaskState;
use crate::errors::AgentSystemError;
use crate::execution_graph::ExecutionGraph;
use crate::guard::{Guard, GuardContext, GuardPolicy};
use crate::intervention::InterventionEngine;
use crate::profile::ProfileAssessment;
use crate::roles::monitor::{MonitorMessage, MonitorState};
use crate::roles::planner::planner_output_from_subtasks;
use crate::roles::router::BranchRoutingDecision;
use crate::roles::supervisor::{SupervisorMessage, SupervisorState};
use crate::scheduler::{ResultAggregator, TaskAssignment, TaskDecomposer, TaskScheduler};
use crate::topology::{TopologyCompiler, TopologySignals};
use mister_smith_events::{
    AutonomyEvent, AutonomyEventEnvelope, AutonomyStatusView, BranchSummary,
    CheckpointRecordSummary, ContextPressureSummary, DelegationAlert, ExecutionGraphSummary,
    RoutingDecisionSummary, TopologyPlanSummary,
};

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
    branch_checkpoint_coordinator: BranchCheckpointCoordinator,
    execution_graphs: DashMap<TaskId, ExecutionGraph>,
    guard_decisions: DashMap<TaskId, Vec<GuardDecision>>,
    interventions: DashMap<TaskId, Vec<InterventionRecord>>,
    profiles: DashMap<TaskId, Vec<ProfileAssessment>>,
    conservative_reasons: DashMap<TaskId, Vec<String>>,
    autonomy_events: DashMap<TaskId, Vec<AutonomyEvent>>,
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
            branch_checkpoint_coordinator: BranchCheckpointCoordinator,
            execution_graphs: DashMap::new(),
            guard_decisions: DashMap::new(),
            interventions: DashMap::new(),
            profiles: DashMap::new(),
            conservative_reasons: DashMap::new(),
            autonomy_events: DashMap::new(),
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
        self.register_execution_graph(graph);

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
        let workflow_id = graph.workflow_id;
        let graph_id = graph.graph_id;
        let checkpoint_events = graph
            .branches
            .iter()
            .filter_map(|branch| {
                graph
                    .latest_checkpoint(&branch.branch_id)
                    .map(|checkpoint| {
                        AutonomyEvent::CheckpointRecorded(AutonomyEventEnvelope {
                            workflow_id,
                            graph_id: Some(graph_id),
                            branch_id: Some(branch.branch_id),
                            payload: CheckpointRecordSummary {
                                checkpoint_id: checkpoint.checkpoint_id,
                                memory_snapshot_id: checkpoint.memory_snapshot_id,
                                completed_nodes: checkpoint.completed_nodes.clone(),
                                pending_nodes: checkpoint.pending_nodes.clone(),
                                recovery_strategy: branch.recovery_strategy,
                                failure_context: checkpoint.failure_context.clone(),
                            },
                            operator_visible: true,
                        })
                    })
            })
            .collect::<Vec<_>>();

        self.execution_graphs.insert(workflow_id, graph);
        for event in checkpoint_events {
            self.record_autonomy_event(&workflow_id, event);
        }
    }

    /// Record a branch checkpoint after graph registration and emit a typed autonomy event.
    pub async fn record_branch_checkpoint<S: BranchCheckpointStore + ?Sized>(
        &self,
        workflow_id: &TaskId,
        store: &S,
        checkpoint: crate::execution_graph::BranchCheckpoint,
    ) -> Result<(), AgentSystemError> {
        let mut graph = self.execution_graphs.get_mut(workflow_id).ok_or_else(|| {
            AgentSystemError::OrchestrationError(format!(
                "No execution graph found for workflow {workflow_id}"
            ))
        })?;

        self.branch_checkpoint_coordinator
            .record_checkpoint(store, *workflow_id, graph.value_mut(), checkpoint.clone())
            .await?;

        let branch = graph.branch(&checkpoint.branch_id).ok_or_else(|| {
            AgentSystemError::OrchestrationError(format!(
                "No branch {} found after checkpoint record",
                checkpoint.branch_id
            ))
        })?;

        self.record_autonomy_event(
            workflow_id,
            AutonomyEvent::CheckpointRecorded(AutonomyEventEnvelope {
                workflow_id: *workflow_id,
                graph_id: Some(graph.graph_id),
                branch_id: Some(checkpoint.branch_id),
                payload: CheckpointRecordSummary {
                    checkpoint_id: checkpoint.checkpoint_id,
                    memory_snapshot_id: checkpoint.memory_snapshot_id,
                    completed_nodes: checkpoint.completed_nodes,
                    pending_nodes: checkpoint.pending_nodes,
                    recovery_strategy: branch.recovery_strategy,
                    failure_context: checkpoint.failure_context,
                },
                operator_visible: true,
            }),
        );

        Ok(())
    }

    /// Record a profile assessment for later routing and operator inspection.
    pub fn record_profile_assessment(&self, workflow_id: &TaskId, assessment: ProfileAssessment) {
        let graph = self.execution_graph(workflow_id);
        let graph_id = graph.as_ref().map(|graph| graph.graph_id);
        let branch_id = match assessment.target() {
            Some(GuardTarget::Branch(branch_id)) => Some(*branch_id),
            Some(GuardTarget::Node(node_id)) => graph
                .as_ref()
                .and_then(|graph| graph.nodes.iter().find(|node| node.node_id == *node_id))
                .map(|node| node.branch_id),
            _ => None,
        };

        if let Some(snapshot) = assessment.snapshot().cloned() {
            self.record_autonomy_event(
                workflow_id,
                AutonomyEvent::ProfileSnapshotRecorded(AutonomyEventEnvelope {
                    workflow_id: *workflow_id,
                    graph_id,
                    branch_id,
                    payload: snapshot,
                    operator_visible: true,
                }),
            );
        }

        self.profiles
            .entry(*workflow_id)
            .or_default()
            .push(assessment);
    }

    /// Plan branch-local resume from the latest durable checkpoint and record routing rationale.
    pub async fn resume_branch<S: BranchCheckpointStore + ?Sized>(
        &self,
        workflow_id: &TaskId,
        store: &S,
        branch_id: ExecutionBranchId,
        assigned_agent: Option<AgentId>,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        self.recover_branch(
            workflow_id,
            store,
            branch_id,
            assigned_agent,
            RecoveryAction::Resume,
        )
        .await
    }

    /// Plan branch reassignment from the latest durable checkpoint and record routing rationale.
    pub async fn reassign_branch<S: BranchCheckpointStore + ?Sized>(
        &self,
        workflow_id: &TaskId,
        store: &S,
        branch_id: ExecutionBranchId,
        assigned_agent: AgentId,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        self.recover_branch(
            workflow_id,
            store,
            branch_id,
            Some(assigned_agent),
            RecoveryAction::Reassign,
        )
        .await
    }

    /// Return recorded autonomy events for a workflow.
    pub fn autonomy_events(&self, workflow_id: &TaskId) -> Vec<AutonomyEvent> {
        self.autonomy_events
            .get(workflow_id)
            .map(|events| events.value().clone())
            .unwrap_or_default()
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
            self.record_profile_assessment(workflow_id, profile);
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
                        .latest_checkpoint(&branch.branch_id)
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
        self.scheduler.all_subtasks_completed(parent_task_id)
    }

    /// Get subtasks that are pending and have all dependencies satisfied.
    pub fn ready_subtasks(&self, parent_task_id: &TaskId) -> Vec<TaskAssignment> {
        self.scheduler
            .subtasks(parent_task_id)
            .into_iter()
            .filter(|t| t.state == TaskState::Pending)
            .collect()
    }

    /// Route ready branches using health, budget, dependency-depth, and profile signals.
    pub fn route_ready_branches(
        &self,
        workflow_id: &TaskId,
        worker_ids: &[AgentId],
    ) -> Result<Vec<BranchRoutingDecision>, AgentSystemError> {
        if worker_ids.is_empty() {
            return Err(AgentSystemError::OrchestrationError(
                "No workers available for branch routing".to_string(),
            ));
        }

        let graph = self.execution_graph(workflow_id).ok_or_else(|| {
            AgentSystemError::OrchestrationError(format!(
                "No execution graph found for workflow {workflow_id}"
            ))
        })?;
        graph.validate()?;

        let tasks_by_node = graph
            .nodes
            .iter()
            .filter_map(|node| {
                self.scheduler
                    .get(&task_id_for_node(node.node_id))
                    .map(|task| (node.node_id, task))
            })
            .collect::<HashMap<_, _>>();
        let depth_by_node = dependency_depths(&graph);
        let profiles = self
            .profiles
            .get(workflow_id)
            .map(|entries| entries.value().clone())
            .unwrap_or_default();

        struct Candidate {
            decision: BranchRoutingDecision,
            recovery_strategy: BranchRecoveryStrategy,
            checkpoint_id: Option<CheckpointId>,
        }

        let mut candidates = graph
            .branches
            .iter()
            .filter_map(|branch| {
                if branch.state == BranchState::Completed {
                    return None;
                }

                let recovery_node_ids = recovery_scope_node_ids(&graph, branch.branch_id);
                let ready_node_ids = ready_node_ids_for_branch(
                    &graph,
                    branch.branch_id,
                    &recovery_node_ids,
                    &tasks_by_node,
                );
                if ready_node_ids.is_empty() {
                    return None;
                }

                let profile = latest_branch_profile(&profiles, branch.branch_id);
                let health_state = profile
                    .and_then(|assessment| {
                        assessment.snapshot().map(|snapshot| snapshot.health_state)
                    })
                    .unwrap_or(HealthState::Unknown);
                let profile_id = profile.and_then(|assessment| {
                    assessment.snapshot().map(|snapshot| snapshot.profile_id)
                });
                let budget_pressure = budget_pressure_score(&graph, &recovery_node_ids);
                let dependency_depth = ready_node_ids
                    .iter()
                    .filter_map(|node_id| depth_by_node.get(node_id))
                    .copied()
                    .max()
                    .unwrap_or(0);
                let checkpoint_id = graph
                    .latest_checkpoint(&branch.branch_id)
                    .map(|checkpoint| checkpoint.checkpoint_id);
                let task_ids = ready_node_ids
                    .iter()
                    .copied()
                    .map(task_id_for_node)
                    .collect::<Vec<_>>();
                let rationale = routing_rationale(
                    budget_pressure,
                    dependency_depth,
                    profile,
                    recovery_node_ids.len(),
                    checkpoint_id,
                );

                Some(Candidate {
                    decision: BranchRoutingDecision {
                        workflow_id: *workflow_id,
                        graph_id: graph.graph_id,
                        branch_id: branch.branch_id,
                        task_ids,
                        recovery_node_ids,
                        recovery_strategy: branch.recovery_strategy,
                        checkpoint_id,
                        selected_agent: Some(worker_ids[0]),
                        health_state,
                        budget_pressure,
                        dependency_depth,
                        profile_id,
                        rationale,
                    },
                    recovery_strategy: branch.recovery_strategy,
                    checkpoint_id,
                })
            })
            .collect::<Vec<_>>();

        candidates.sort_by(|left, right| {
            health_priority(left.decision.health_state)
                .cmp(&health_priority(right.decision.health_state))
                .then_with(|| {
                    left.decision
                        .budget_pressure
                        .partial_cmp(&right.decision.budget_pressure)
                        .unwrap_or(Ordering::Equal)
                })
                .then_with(|| {
                    left.decision
                        .dependency_depth
                        .cmp(&right.decision.dependency_depth)
                })
                .then_with(|| {
                    left.decision
                        .branch_id
                        .to_string()
                        .cmp(&right.decision.branch_id.to_string())
                })
        });

        let mut worker_loads = self.scheduler.worker_loads(worker_ids);
        let mut decisions = Vec::new();
        for mut candidate in candidates {
            let worker = select_worker(worker_ids, &worker_loads);
            let mut assigned_task_ids = Vec::new();
            for task_id in &candidate.decision.task_ids {
                let Some(task) = self.scheduler.get(task_id) else {
                    continue;
                };

                match task.state {
                    TaskState::Completed | TaskState::Assigned | TaskState::Running => continue,
                    TaskState::Failed | TaskState::TimedOut | TaskState::Cancelled => {
                        self.scheduler.reset(task_id)?;
                    }
                    TaskState::Pending => {}
                }

                self.scheduler.assign(task_id, worker)?;
                assigned_task_ids.push(*task_id);
            }

            if assigned_task_ids.is_empty() {
                continue;
            }

            candidate.decision.selected_agent = Some(worker);
            candidate.decision.task_ids = assigned_task_ids.clone();
            *worker_loads.entry(worker).or_default() += assigned_task_ids.len();

            self.record_autonomy_event(
                workflow_id,
                AutonomyEvent::RoutingDecisionRecorded(AutonomyEventEnvelope {
                    workflow_id: *workflow_id,
                    graph_id: Some(candidate.decision.graph_id),
                    branch_id: Some(candidate.decision.branch_id),
                    payload: RoutingDecisionSummary {
                        selected_agent: worker,
                        task_ids: assigned_task_ids,
                        recovery_strategy: candidate.recovery_strategy,
                        checkpoint_id: candidate.checkpoint_id,
                        dependency_depth: candidate.decision.dependency_depth,
                        budget_pressure: candidate.decision.budget_pressure,
                        health_state: candidate.decision.health_state,
                        profile_id: candidate.decision.profile_id,
                        rationale: candidate.decision.rationale.clone(),
                    },
                    operator_visible: true,
                }),
            );

            decisions.push(candidate.decision);
        }

        if !decisions.is_empty() {
            let mut graph = self.execution_graphs.get_mut(workflow_id).ok_or_else(|| {
                AgentSystemError::OrchestrationError(format!(
                    "No execution graph found for workflow {workflow_id}"
                ))
            })?;

            for decision in &decisions {
                let has_checkpoint = graph.latest_checkpoint(&decision.branch_id).is_some();
                if let Some(branch) = graph.branch_mut(&decision.branch_id) {
                    if let Some(agent_id) = decision.selected_agent {
                        branch.assigned_agents = vec![agent_id];
                    }
                    branch.state = if has_checkpoint {
                        BranchState::Checkpointed
                    } else {
                        BranchState::Running
                    };
                }
            }
        }

        Ok(decisions)
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

        // 2. Route only the ready branch scope using resilience-aware signals.
        let routed = self.route_ready_branches(&task.task_id, worker_ids)?;
        if routed.is_empty() {
            return Err(AgentSystemError::OrchestrationError(
                "No ready branches available for routing".into(),
            ));
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

    async fn recover_branch<S: BranchCheckpointStore + ?Sized>(
        &self,
        workflow_id: &TaskId,
        store: &S,
        branch_id: ExecutionBranchId,
        assigned_agent: Option<AgentId>,
        action: RecoveryAction,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        let profiles = self
            .profiles
            .get(workflow_id)
            .map(|entries| entries.value().clone())
            .unwrap_or_default();
        let (recovery, routing_event) = {
            let mut graph = self.execution_graphs.get_mut(workflow_id).ok_or_else(|| {
                AgentSystemError::OrchestrationError(format!(
                    "No execution graph found for workflow {workflow_id}"
                ))
            })?;

            let recovery = match action {
                RecoveryAction::Resume => {
                    self.branch_checkpoint_coordinator
                        .resume_branch(
                            store,
                            *workflow_id,
                            graph.value_mut(),
                            branch_id,
                            assigned_agent,
                        )
                        .await?
                }
                RecoveryAction::Reassign => {
                    let assigned_agent = assigned_agent.ok_or_else(|| {
                        AgentSystemError::OrchestrationError(
                            "branch reassignment requires a target agent".to_string(),
                        )
                    })?;
                    self.branch_checkpoint_coordinator
                        .reassign_branch(
                            store,
                            *workflow_id,
                            graph.value_mut(),
                            branch_id,
                            assigned_agent,
                        )
                        .await?
                }
            };

            let selected_agent = recovery.resume_metadata.assigned_agent.or_else(|| {
                graph
                    .branch(&branch_id)
                    .and_then(|branch| branch.assigned_agents.first().copied())
            });
            let routing_event = selected_agent.map(|selected_agent| {
                recovery_routing_event(
                    *workflow_id,
                    &graph,
                    branch_id,
                    selected_agent,
                    &profiles,
                    &recovery,
                )
            });

            (recovery, routing_event)
        };

        if let Some(event) = routing_event {
            self.record_autonomy_event(workflow_id, event);
        }

        Ok(recovery)
    }

    fn record_autonomy_event(&self, workflow_id: &TaskId, event: AutonomyEvent) {
        self.autonomy_events
            .entry(*workflow_id)
            .or_default()
            .push(event);
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

fn task_id_for_node(node_id: ExecutionNodeId) -> TaskId {
    TaskId::from_uuid(*node_id.as_ref())
}

fn recovery_scope_node_ids(
    graph: &ExecutionGraph,
    branch_id: ExecutionBranchId,
) -> Vec<ExecutionNodeId> {
    if graph.latest_checkpoint(&branch_id).is_some() {
        graph.recovery_node_ids(&branch_id)
    } else {
        graph
            .branch(&branch_id)
            .map(|branch| branch.node_ids.clone())
            .unwrap_or_default()
    }
}

fn ready_node_ids_for_branch(
    graph: &ExecutionGraph,
    branch_id: ExecutionBranchId,
    recovery_node_ids: &[ExecutionNodeId],
    tasks_by_node: &HashMap<ExecutionNodeId, TaskAssignment>,
) -> Vec<ExecutionNodeId> {
    recovery_node_ids
        .iter()
        .copied()
        .filter(|node_id| {
            let Some(node) = graph.nodes.iter().find(|node| node.node_id == *node_id) else {
                return false;
            };
            if node.branch_id != branch_id {
                return false;
            }

            let Some(task) = tasks_by_node.get(node_id) else {
                return false;
            };
            if matches!(
                task.state,
                TaskState::Completed | TaskState::Assigned | TaskState::Running
            ) {
                return false;
            }

            node.dependencies
                .iter()
                .all(|dependency| node_completed(graph, tasks_by_node, *dependency))
        })
        .collect()
}

fn node_completed(
    graph: &ExecutionGraph,
    tasks_by_node: &HashMap<ExecutionNodeId, TaskAssignment>,
    node_id: ExecutionNodeId,
) -> bool {
    graph
        .nodes
        .iter()
        .find(|node| node.node_id == node_id)
        .map(|node| node.state == mister_smith_core::NodeState::Completed)
        .unwrap_or(false)
        || tasks_by_node
            .get(&node_id)
            .map(|task| task.state == TaskState::Completed)
            .unwrap_or(false)
}

fn dependency_depths(graph: &ExecutionGraph) -> HashMap<ExecutionNodeId, usize> {
    let node_lookup = graph
        .nodes
        .iter()
        .map(|node| (node.node_id, node))
        .collect::<HashMap<_, _>>();
    let mut memo = HashMap::new();
    for node in &graph.nodes {
        let _ = dependency_depth(node.node_id, &node_lookup, &mut memo);
    }
    memo
}

fn dependency_depth(
    node_id: ExecutionNodeId,
    node_lookup: &HashMap<ExecutionNodeId, &crate::execution_graph::ExecutionNode>,
    memo: &mut HashMap<ExecutionNodeId, usize>,
) -> usize {
    if let Some(depth) = memo.get(&node_id) {
        return *depth;
    }

    let depth = node_lookup
        .get(&node_id)
        .map(|node| {
            node.dependencies
                .iter()
                .copied()
                .map(|dependency| dependency_depth(dependency, node_lookup, memo) + 1)
                .max()
                .unwrap_or(0)
        })
        .unwrap_or(0);
    memo.insert(node_id, depth);
    depth
}

fn latest_branch_profile(
    profiles: &[ProfileAssessment],
    branch_id: ExecutionBranchId,
) -> Option<&ProfileAssessment> {
    profiles.iter().rev().find(|assessment| {
        matches!(
            assessment.target(),
            Some(GuardTarget::Branch(target_branch_id)) if *target_branch_id == branch_id
        )
    })
}

fn budget_pressure_score(graph: &ExecutionGraph, node_ids: &[ExecutionNodeId]) -> u8 {
    node_ids
        .iter()
        .filter_map(|node_id| {
            graph
                .nodes
                .iter()
                .find(|node| node.node_id == *node_id)
                .and_then(|node| {
                    (node.budget.max_units > 0).then_some(
                        ((node.budget.reserved_units as f32 / node.budget.max_units as f32) * 100.0)
                            .round()
                            .clamp(0.0, 100.0) as u8,
                    )
                })
        })
        .max()
        .unwrap_or(0)
}

fn routing_rationale(
    budget_pressure: u8,
    dependency_depth: usize,
    profile: Option<&ProfileAssessment>,
    recovery_scope_len: usize,
    checkpoint_id: Option<CheckpointId>,
) -> Vec<String> {
    let mut rationale = vec![
        format!("budget pressure {budget_pressure}% across routed scope"),
        format!("dependency depth {dependency_depth} across ready branch frontier"),
        format!("checkpoint recovery scope covers {recovery_scope_len} nodes"),
    ];

    if let Some(checkpoint_id) = checkpoint_id {
        rationale.push(format!(
            "checkpoint {checkpoint_id} limits replay to pending work"
        ));
    } else {
        rationale.push("checkpoint unavailable; routing original branch scope".to_string());
    }

    if let Some(profile) = profile {
        if let Some(snapshot) = profile.snapshot() {
            rationale.push(format!(
                "profile {} reports {:?} health",
                snapshot.profile_id, snapshot.health_state
            ));
        } else {
            rationale.push("profile present without snapshot payload".to_string());
        }
        rationale.extend(
            profile
                .notes()
                .iter()
                .map(|note| format!("profile note: {note}")),
        );
    } else {
        rationale.push("profile unavailable; defaulting to unknown health".to_string());
    }

    rationale
}

fn health_priority(health_state: HealthState) -> u8 {
    match health_state {
        HealthState::Healthy => 0,
        HealthState::Degraded => 1,
        HealthState::Unhealthy => 2,
        HealthState::Unknown => 3,
    }
}

fn select_worker(worker_ids: &[AgentId], worker_loads: &HashMap<AgentId, usize>) -> AgentId {
    worker_ids
        .iter()
        .copied()
        .min_by_key(|worker_id| worker_loads.get(worker_id).copied().unwrap_or(0))
        .unwrap_or(worker_ids[0])
}

fn recovery_routing_event(
    workflow_id: TaskId,
    graph: &ExecutionGraph,
    branch_id: ExecutionBranchId,
    selected_agent: AgentId,
    profiles: &[ProfileAssessment],
    recovery: &BranchRecoveryPlan,
) -> AutonomyEvent {
    let profile = latest_branch_profile(profiles, branch_id);
    let budget_pressure = budget_pressure_score(graph, &recovery.recovery_node_ids);
    let dependency_depths = dependency_depths(graph);
    let dependency_depth = recovery
        .recovery_node_ids
        .iter()
        .filter_map(|node_id| dependency_depths.get(node_id))
        .copied()
        .max()
        .unwrap_or(0);
    let health_state = profile
        .and_then(|assessment| assessment.snapshot().map(|snapshot| snapshot.health_state))
        .unwrap_or(HealthState::Unknown);
    let profile_id =
        profile.and_then(|assessment| assessment.snapshot().map(|snapshot| snapshot.profile_id));
    let mut rationale = routing_rationale(
        budget_pressure,
        dependency_depth,
        profile,
        recovery.recovery_node_ids.len(),
        Some(recovery.checkpoint.checkpoint_id),
    );
    rationale.extend(recovery.resume_metadata.notes.iter().cloned());

    AutonomyEvent::RoutingDecisionRecorded(AutonomyEventEnvelope {
        workflow_id,
        graph_id: Some(graph.graph_id),
        branch_id: Some(branch_id),
        payload: RoutingDecisionSummary {
            selected_agent,
            task_ids: recovery
                .recovery_node_ids
                .iter()
                .copied()
                .map(task_id_for_node)
                .collect(),
            recovery_strategy: recovery.resume_metadata.recovery_strategy,
            checkpoint_id: Some(recovery.checkpoint.checkpoint_id),
            dependency_depth,
            budget_pressure,
            health_state,
            profile_id,
            rationale,
        },
        operator_visible: true,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryAction {
    Resume,
    Reassign,
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
