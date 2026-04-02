use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc;
use std::sync::Arc;

use dashmap::DashMap;
use mister_smith_core::{
    packet_023_placeholder_runtime_truth, AgentId, BranchRecoveryStrategy, BranchState,
    CheckpointId, DurableWorkflowLifecycleState, ExecutionBranchId, ExecutionNodeId, GraphState,
    GuardDecision, GuardTarget, HealthState, InterventionRecord, NodeState, ProfileSnapshot,
    RepairLineageRef, RunTraceRelationshipKind, RuntimeTruthView, SupervisionEvidenceView,
    SupervisionTargetKind, SupervisionTargetScope, TaskId, TeamSizingDecision,
};
use mister_smith_persistence::WorkflowHistoryEventRecord;
use serde_json::Value;
use tracing::{instrument, warn};
use uuid::Uuid;

use crate::branch_checkpoint::{
    checkpoint_from_replay_payload, BranchCheckpointCoordinator, BranchCheckpointStore,
    BranchRecoveryPlan,
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
use crate::team::{plan_adaptive_team, AdaptiveTeamPlan, AdaptiveTeamSizingInputs};
use crate::topology::{TopologyCompiler, TopologySignals};
use mister_smith_events::autonomy::{
    infer_result_preview_from_projection, lifecycle_state_for_graph_state,
};
use mister_smith_events::{
    AutonomyEvent, AutonomyEventEnvelope, AutonomyStatusView, BranchSummary, CapabilitySummary,
    CheckpointRecordSummary, ContextPressureSummary, Event, EventBus, ExecutionGraphSummary,
    RoutingDecisionSummary, StepRoutingDecisionSummary, TopologyPlanSummary,
};

#[cfg(feature = "llm")]
use mister_smith_llm::router::ConfidenceSignal;
#[cfg(feature = "llm")]
use mister_smith_llm::{
    CompletionResponse, ModelEvent, RoutingDecision, RoutingHint, StepRoutingAction,
    StepRoutingMetadata, StepRoutingSignal, StepVerificationCheckpoint,
    StepVerificationCheckpointKind, StreamMonitor, StreamMonitorConfig,
};
#[cfg(feature = "llm")]
use serde::{Deserialize, Serialize};
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
    adaptive_team_plans: DashMap<TaskId, AdaptiveTeamPlan>,
    workflow_coordinators: DashMap<TaskId, AgentId>,
    autonomy_events: DashMap<TaskId, Vec<AutonomyEvent>>,
    step_routing_histories: DashMap<TaskId, Vec<StepRoutingDecisionSummary>>,
    autonomy_event_tx: Option<mpsc::Sender<Event>>,
    monitor_states: DashMap<TaskId, MonitorState>,
    supervisor_states: DashMap<TaskId, SupervisorState>,
}

const AUTONOMY_EVENT_SOURCE: &str = "mister-smith-agents::orchestrator";

fn baseline_team_sizing_decision(
    graph: &ExecutionGraph,
    routing_history: &[RoutingDecisionSummary],
    conservative_reasons: &[String],
) -> TeamSizingDecision {
    let desired_workers = graph.topology_plan.parallelism_width.max(1);
    let assigned_worker_count = graph
        .branches
        .iter()
        .flat_map(|branch| branch.assigned_agents.iter().copied())
        .collect::<HashSet<_>>()
        .len();
    let uses_structural_parallelism_fallback = assigned_worker_count == 0
        && matches!(
            graph.state,
            GraphState::Pending | GraphState::Running | GraphState::Checkpointed
        );
    let available_workers = if uses_structural_parallelism_fallback {
        desired_workers
    } else {
        assigned_worker_count.max(1)
    };
    let selected_workers = desired_workers.min(available_workers);
    let branch_frontier_width = graph
        .topology_plan
        .task_shape
        .max_parallel_width
        .max(graph.branches.len())
        .max(1);
    let dependency_depth = routing_history
        .iter()
        .map(|decision| decision.dependency_depth)
        .max()
        .unwrap_or(graph.topology_plan.task_shape.max_depth);
    let budget_pressure = routing_history
        .iter()
        .map(|decision| decision.budget_pressure)
        .max();
    let conservative_mode = !conservative_reasons.is_empty();
    let cap_reason = (selected_workers < desired_workers)
        .then_some("available worker pool smaller than structural width".to_string());

    let mut rationale_lines = vec![
        format!(
            "task shape {} with frontier width {}",
            graph.topology_plan.task_shape.kind.as_str(),
            branch_frontier_width
        ),
        format!(
            "topology {:?} requested {} workers",
            graph.topology_plan.topology_kind, desired_workers
        )
        .to_ascii_lowercase(),
    ];
    if uses_structural_parallelism_fallback {
        rationale_lines.push(format!(
            "selected {} workers from structural parallelism because no live worker assignments are recorded yet for this non-terminal graph",
            selected_workers
        ));
    } else if assigned_worker_count == 0 {
        rationale_lines.push(
            "selected 1 worker because this terminal graph no longer has live worker assignments to preserve"
                .to_string(),
        );
    } else {
        rationale_lines.push(format!(
            "selected {} workers because {} workers are currently available",
            selected_workers, available_workers
        ));
    }
    if let Some(pressure) = budget_pressure {
        rationale_lines.push(format!("latest routing budget pressure {}", pressure));
    }
    if conservative_mode {
        rationale_lines.extend(
            conservative_reasons
                .iter()
                .map(|reason| format!("conservative posture: {}", reason)),
        );
    }

    TeamSizingDecision {
        workflow_id: graph.workflow_id,
        graph_id: graph.graph_id,
        decision_phase: "initial".to_string(),
        desired_workers,
        selected_workers,
        available_workers,
        branch_frontier_width,
        dependency_depth,
        conservative_mode,
        budget_pressure,
        cap_reason,
        rationale_lines,
        decided_at: chrono::Utc::now(),
    }
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
            adaptive_team_plans: DashMap::new(),
            workflow_coordinators: DashMap::new(),
            autonomy_events: DashMap::new(),
            step_routing_histories: DashMap::new(),
            autonomy_event_tx: None,
            monitor_states: DashMap::new(),
            supervisor_states: DashMap::new(),
        }
    }

    /// Forward typed autonomy events into the shared event bus projection.
    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        let (tx, rx) = mpsc::channel::<Event>();
        std::thread::Builder::new()
            .name("mister-smith-autonomy-events".to_string())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        warn!(%error, "Failed to build autonomy event publisher runtime");
                        return;
                    }
                };

                while let Ok(event) = rx.recv() {
                    if let Err(error) = runtime.block_on(event_bus.publish(event)) {
                        warn!(%error, "Failed to publish autonomy event to shared event bus");
                    }
                }
            })
            .expect("autonomy event publisher thread should spawn");
        self.autonomy_event_tx = Some(tx);
        self
    }

    /// Remember the workflow coordinator so adaptive team plans can retain a stable owner.
    pub fn register_workflow_coordinator(&self, workflow_id: &TaskId, coordinator_id: AgentId) {
        self.workflow_coordinators
            .insert(*workflow_id, coordinator_id);
    }

    /// Return the latest adaptive team plan for a workflow, when one has been materialized.
    pub fn adaptive_team_plan(&self, workflow_id: &TaskId) -> Option<AdaptiveTeamPlan> {
        self.adaptive_team_plans
            .get(workflow_id)
            .map(|entry| entry.value().clone())
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

    /// Restore a replayed execution graph snapshot without emitting fresh autonomy events.
    pub fn restore_execution_graph_snapshot(&self, graph: ExecutionGraph) {
        self.execution_graphs.insert(graph.workflow_id, graph);
    }

    /// Rebuild a graph from accepted durable workflow history.
    pub fn replay_execution_graph(
        &self,
        graph: ExecutionGraph,
        history: &[WorkflowHistoryEventRecord],
    ) -> Result<ExecutionGraph, AgentSystemError> {
        self.replay_execution_graph_from_history(graph, history)
    }

    /// Register a precompiled execution graph for later supervision or inspection.
    pub fn register_execution_graph(&self, graph: ExecutionGraph) {
        let workflow_id = graph.workflow_id;
        let graph_id = graph.graph_id;
        let mut initial_events = vec![
            AutonomyEvent::GraphUpdated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: ExecutionGraphSummary {
                    graph_id,
                    workflow_id,
                    state: graph.state,
                    branch_count: graph.branches.len(),
                    node_count: graph.nodes.len(),
                    active_topology: Some(graph.topology_plan.topology_kind),
                },
                operator_visible: true,
            }),
            AutonomyEvent::TopologySelected(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: None,
                payload: TopologyPlanSummary {
                    graph_id,
                    topology_kind: graph.topology_plan.topology_kind,
                    parallelism_width: graph.topology_plan.parallelism_width,
                    task_shape: graph.topology_plan.task_shape.clone(),
                    coordination_policy: graph.topology_plan.coordination_policy,
                    rationale: graph.topology_plan.rationale.clone(),
                    fallback_topology: graph.topology_plan.fallback_topology,
                },
                operator_visible: true,
            }),
        ];
        initial_events.extend(graph.branches.iter().map(|branch| {
            AutonomyEvent::BranchUpdated(AutonomyEventEnvelope {
                workflow_id,
                graph_id: Some(graph_id),
                branch_id: Some(branch.branch_id),
                payload: BranchSummary {
                    branch_id: branch.branch_id,
                    graph_id: branch.graph_id,
                    state: branch.state,
                    assigned_agents: branch.assigned_agents.clone(),
                    checkpoint_id: graph
                        .latest_checkpoint(&branch.branch_id)
                        .map(|checkpoint| checkpoint.checkpoint_id),
                    recovery_strategy: branch.recovery_strategy,
                },
                operator_visible: true,
            })
        }));
        initial_events.extend(graph.branches.iter().filter_map(|branch| {
            graph
                .latest_checkpoint(&branch.branch_id)
                .map(|checkpoint| {
                    AutonomyEvent::CheckpointRecorded(AutonomyEventEnvelope {
                        workflow_id,
                        graph_id: Some(graph_id),
                        branch_id: Some(branch.branch_id),
                        payload: CheckpointRecordSummary {
                            graph_id,
                            branch_id: branch.branch_id,
                            checkpoint_id: checkpoint.checkpoint_id,
                            captured_at: checkpoint.created_at,
                            memory_snapshot_id: checkpoint.memory_snapshot_id,
                            completed_nodes: checkpoint.completed_nodes.clone(),
                            pending_nodes: checkpoint.pending_nodes.clone(),
                            recovery_strategy: branch.recovery_strategy,
                            failure_context: checkpoint.failure_context.clone(),
                        },
                        operator_visible: true,
                    })
                })
        }));

        self.execution_graphs.insert(workflow_id, graph);
        for event in initial_events {
            self.record_autonomy_event(&workflow_id, event);
        }
    }

    /// Rebuild graph and checkpoint state from accepted durable workflow history.
    pub fn replay_execution_graph_from_history(
        &self,
        mut graph: ExecutionGraph,
        history: &[WorkflowHistoryEventRecord],
    ) -> Result<ExecutionGraph, AgentSystemError> {
        let mut ordered_history = history.to_vec();
        ordered_history.sort_by(|left, right| {
            left.replay_position
                .cmp(&right.replay_position)
                .then_with(|| left.recorded_at.cmp(&right.recorded_at))
                .then_with(|| left.event_id.cmp(&right.event_id))
        });

        for event in &ordered_history {
            match event.event_kind {
                mister_smith_core::DurableWorkflowEventKind::LifecycleChanged => {
                    apply_lifecycle_history_event(&mut graph, event)?
                }
                mister_smith_core::DurableWorkflowEventKind::BranchStateChanged => {
                    apply_branch_history_event(&mut graph, event)?
                }
                mister_smith_core::DurableWorkflowEventKind::NodeStateChanged => {
                    apply_node_history_event(&mut graph, event)?
                }
                mister_smith_core::DurableWorkflowEventKind::EffectIntentRecorded
                | mister_smith_core::DurableWorkflowEventKind::EffectOutcomeRecorded => {}
                mister_smith_core::DurableWorkflowEventKind::HistoryCompacted => {
                    apply_compaction_history_event(&mut graph, event)?
                }
            }
        }

        recompute_replayed_graph_state(&mut graph);
        Ok(graph)
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
                    graph_id: graph.graph_id,
                    branch_id: checkpoint.branch_id,
                    checkpoint_id: checkpoint.checkpoint_id,
                    captured_at: checkpoint.created_at,
                    memory_snapshot_id: checkpoint.memory_snapshot_id,
                    completed_nodes: checkpoint.completed_nodes,
                    pending_nodes: checkpoint.pending_nodes,
                    recovery_strategy: branch.recovery_strategy,
                    failure_context: checkpoint.failure_context,
                },
                operator_visible: true,
            }),
        );
        self.record_autonomy_event(
            workflow_id,
            AutonomyEvent::BranchUpdated(AutonomyEventEnvelope {
                workflow_id: *workflow_id,
                graph_id: Some(graph.graph_id),
                branch_id: Some(branch.branch_id),
                payload: BranchSummary {
                    branch_id: branch.branch_id,
                    graph_id: branch.graph_id,
                    state: branch.state,
                    assigned_agents: branch.assigned_agents.clone(),
                    checkpoint_id: Some(checkpoint.checkpoint_id),
                    recovery_strategy: branch.recovery_strategy,
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
        let branch_id = graph
            .as_ref()
            .and_then(|graph| assessment.target_branch_id(graph));

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
            None,
        )
        .await
    }

    /// Plan branch-local resume from the latest durable checkpoint using delegated authority.
    pub async fn resume_branch_with_delegation<S: BranchCheckpointStore + ?Sized>(
        &self,
        workflow_id: &TaskId,
        store: &S,
        branch_id: ExecutionBranchId,
        assigned_agent: Option<AgentId>,
        capability: &CapabilitySummary,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        self.recover_branch(
            workflow_id,
            store,
            branch_id,
            assigned_agent,
            RecoveryAction::Resume,
            Some(capability),
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
            None,
        )
        .await
    }

    /// Plan branch reassignment from the latest durable checkpoint using delegated authority.
    pub async fn reassign_branch_with_delegation<S: BranchCheckpointStore + ?Sized>(
        &self,
        workflow_id: &TaskId,
        store: &S,
        branch_id: ExecutionBranchId,
        assigned_agent: AgentId,
        capability: &CapabilitySummary,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        self.recover_branch(
            workflow_id,
            store,
            branch_id,
            Some(assigned_agent),
            RecoveryAction::Reassign,
            Some(capability),
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
            let profile = if profile.target().is_some() {
                profile
            } else {
                profile.with_target(context.target().clone())
            };
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
        let graph_id = self
            .execution_graph(workflow_id)
            .map(|graph| graph.graph_id);
        let branch_id = branch_id_for_target(
            self.execution_graph(workflow_id).as_ref(),
            &decision.target_scope,
        );
        self.record_autonomy_event(
            workflow_id,
            AutonomyEvent::GuardDecisionEvaluated(AutonomyEventEnvelope {
                workflow_id: *workflow_id,
                graph_id,
                branch_id,
                payload: decision.clone(),
                operator_visible: true,
            }),
        );
        self.record_autonomy_event(
            workflow_id,
            AutonomyEvent::InterventionRecorded(AutonomyEventEnvelope {
                workflow_id: *workflow_id,
                graph_id,
                branch_id,
                payload: record.clone(),
                operator_visible: true,
            }),
        );
        if let Some(branch_id) = branch_id {
            if let Some(graph) = self.execution_graph(workflow_id) {
                if let Some(branch) = graph.branch(&branch_id) {
                    self.record_autonomy_event(
                        workflow_id,
                        AutonomyEvent::BranchUpdated(AutonomyEventEnvelope {
                            workflow_id: *workflow_id,
                            graph_id: Some(graph.graph_id),
                            branch_id: Some(branch_id),
                            payload: BranchSummary {
                                branch_id,
                                graph_id: branch.graph_id,
                                state: branch.state,
                                assigned_agents: branch.assigned_agents.clone(),
                                checkpoint_id: graph
                                    .latest_checkpoint(&branch_id)
                                    .map(|checkpoint| checkpoint.checkpoint_id),
                                recovery_strategy: branch.recovery_strategy,
                            },
                            operator_visible: true,
                        }),
                    );
                }
            }
        }

        Ok((decision, record))
    }

    /// Build the operator-visible autonomy status for a workflow.
    pub fn autonomy_status(&self, workflow_id: &TaskId) -> Option<AutonomyStatusView> {
        let graph = self.execution_graph(workflow_id)?;
        let profile_assessments = self
            .profiles
            .get(workflow_id)
            .map(|profiles| profiles.value().clone())
            .unwrap_or_default();
        let profiles = profile_assessments
            .iter()
            .filter_map(ProfileAssessment::snapshot)
            .cloned()
            .collect::<Vec<ProfileSnapshot>>();
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
        let routing_history = self
            .autonomy_events(workflow_id)
            .into_iter()
            .filter_map(|event| match event {
                AutonomyEvent::RoutingDecisionRecorded(envelope) => Some(envelope.payload),
                _ => None,
            })
            .collect::<Vec<_>>();
        let step_routing_history = self
            .step_routing_histories
            .get(workflow_id)
            .map(|history| history.value().clone())
            .unwrap_or_default();
        let mut delegation_capabilities = HashMap::new();
        for capability in self
            .autonomy_events(workflow_id)
            .into_iter()
            .filter_map(|event| match event {
                AutonomyEvent::DelegationUpdated(envelope) => Some(envelope.payload),
                _ => None,
            })
        {
            delegation_capabilities.insert(capability.capability_id, capability);
        }
        let mut delegation_capabilities = delegation_capabilities.into_values().collect::<Vec<_>>();
        delegation_capabilities.sort_by(|left, right| {
            left.expires_at.cmp(&right.expires_at).then_with(|| {
                left.capability_id
                    .to_string()
                    .cmp(&right.capability_id.to_string())
            })
        });
        let delegation_alerts = delegation_capabilities
            .iter()
            .filter_map(CapabilitySummary::to_alert)
            .collect::<Vec<_>>();
        let mut external_capability_decisions = self
            .autonomy_events(workflow_id)
            .into_iter()
            .filter_map(|event| match event {
                AutonomyEvent::DelegationDecisionRecorded(envelope) => Some(envelope.payload),
                _ => None,
            })
            .collect::<Vec<_>>();
        external_capability_decisions.sort_by(|left, right| {
            left.observed_at
                .cmp(&right.observed_at)
                .then_with(|| {
                    left.branch_id
                        .map(|branch_id| branch_id.to_string())
                        .unwrap_or_else(|| "none".to_string())
                        .cmp(
                            &right
                                .branch_id
                                .map(|branch_id| branch_id.to_string())
                                .unwrap_or_else(|| "none".to_string()),
                        )
                })
                .then_with(|| {
                    left.capability_id
                        .map(|capability_id| capability_id.to_string())
                        .unwrap_or_else(|| "none".to_string())
                        .cmp(
                            &right
                                .capability_id
                                .map(|capability_id| capability_id.to_string())
                                .unwrap_or_else(|| "none".to_string()),
                        )
                })
                .then_with(|| left.action_id.cmp(&right.action_id))
                .then_with(|| left.action_descriptor_id.cmp(&right.action_descriptor_id))
        });
        let team_sizing = self
            .adaptive_team_plans
            .get(workflow_id)
            .map(|plan| plan.sizing_decision.clone())
            .unwrap_or_else(|| {
                baseline_team_sizing_decision(&graph, &routing_history, &conservative_reasons)
            });

        let graph_summary = ExecutionGraphSummary {
            graph_id: graph.graph_id,
            workflow_id: graph.workflow_id,
            state: graph.state,
            branch_count: graph.branches.len(),
            node_count: graph.nodes.len(),
            active_topology: Some(graph.topology_plan.topology_kind),
        };
        let topology_summary = TopologyPlanSummary {
            graph_id: graph.graph_id,
            topology_kind: graph.topology_plan.topology_kind,
            parallelism_width: graph.topology_plan.parallelism_width,
            task_shape: graph.topology_plan.task_shape.clone(),
            coordination_policy: graph.topology_plan.coordination_policy,
            rationale: graph.topology_plan.rationale.clone(),
            fallback_topology: graph.topology_plan.fallback_topology,
        };
        let branches = graph
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
            .collect::<Vec<_>>();
        let mut result_preview = infer_result_preview_from_projection(
            &graph_summary,
            &topology_summary,
            &branches,
            &routing_history,
        );
        let supervision_evidence = build_supervision_evidence_view(
            &graph,
            &profile_assessments,
            &guard_decisions,
            &interventions,
        );
        let runtime_truth = build_runtime_truth_view(
            &graph,
            &topology_summary,
            supervision_evidence.as_ref(),
            &step_routing_history,
            &guard_decisions,
        );
        if let Some(preview) = result_preview.as_mut() {
            if graph_summary.state == GraphState::Completed {
                preview.runtime_truth = Some(runtime_truth.clone());
            }
        }

        Some(AutonomyStatusView {
            session_id: None,
            turn_index: None,
            coordinator_agent_id: None,
            resume_provenance: None,
            lifecycle_state: Some(lifecycle_state_for_graph_state(graph_summary.state)),
            result_preview,
            graph: graph_summary,
            topology: topology_summary,
            team_sizing: Some(team_sizing),
            branches,
            checkpoint_lineage: graph
                .checkpoint_lineage
                .iter()
                .filter_map(|checkpoint| {
                    graph
                        .branch(&checkpoint.branch_id)
                        .map(|branch| CheckpointRecordSummary {
                            graph_id: graph.graph_id,
                            branch_id: checkpoint.branch_id,
                            checkpoint_id: checkpoint.checkpoint_id,
                            captured_at: checkpoint.created_at,
                            memory_snapshot_id: checkpoint.memory_snapshot_id,
                            completed_nodes: checkpoint.completed_nodes.clone(),
                            pending_nodes: checkpoint.pending_nodes.clone(),
                            recovery_strategy: branch.recovery_strategy,
                            failure_context: checkpoint.failure_context.clone(),
                        })
                })
                .collect(),
            memory_pressure: Vec::<ContextPressureSummary>::new(),
            routing_history,
            step_routing_history,
            interventions,
            delegation_capabilities,
            delegation_alerts,
            external_capability_decisions,
            profiles,
            guard_decisions,
            supervision_evidence,
            runtime_truth: Some(runtime_truth),
            conservative_reasons,
        })
    }

    /// Return the latest persisted step-routing history even before a graph exists.
    pub fn step_routing_history(&self, workflow_id: &TaskId) -> Vec<StepRoutingDecisionSummary> {
        self.step_routing_histories
            .get(workflow_id)
            .map(|history| history.value().clone())
            .unwrap_or_default()
    }

    /// List workflow IDs that currently have recorded autonomy state.
    pub fn autonomy_workflow_ids(&self) -> Vec<TaskId> {
        let mut workflow_ids = self
            .autonomy_events
            .iter()
            .map(|entry| *entry.key())
            .collect::<Vec<_>>();
        workflow_ids.sort_by_key(|workflow_id| workflow_id.to_string());
        workflow_ids.dedup();
        workflow_ids
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
        let results = self.scheduler.completed_subtask_outputs(parent_task_id);
        self.aggregator.aggregate(results).await
    }

    /// Check if all subtasks of a parent task are completed.
    pub fn all_subtasks_completed(&self, parent_task_id: &TaskId) -> bool {
        self.scheduler.all_subtasks_completed(parent_task_id)
    }

    /// Get subtasks that are pending and have all dependencies satisfied.
    pub fn ready_subtasks(&self, parent_task_id: &TaskId) -> Vec<TaskAssignment> {
        self.scheduler
            .subtasks_in_state(parent_task_id, TaskState::Pending)
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
        let conservative_reasons = self
            .conservative_reasons
            .get(workflow_id)
            .map(|notes| notes.value().clone())
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

                let profile = latest_branch_profile(&graph, &profiles, branch.branch_id);
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
                        selected_agent: None,
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

        if candidates.is_empty() {
            return Ok(Vec::new());
        }

        let mut worker_loads = self.scheduler.worker_loads(worker_ids);
        let prior_team_plan = self
            .adaptive_team_plans
            .get(workflow_id)
            .map(|plan| plan.value().clone());
        let frontier_budget_pressure = aggregate_frontier_budget_pressure(
            candidates
                .iter()
                .map(|candidate| candidate.decision.budget_pressure),
        );
        let frontier_dependency_depth = candidates
            .iter()
            .map(|candidate| candidate.decision.dependency_depth)
            .max()
            .unwrap_or(graph.topology_plan.task_shape.max_depth);
        let frontier_health_state = aggregate_frontier_health(
            candidates
                .iter()
                .map(|candidate| candidate.decision.health_state),
        );
        let coordinator_id = self
            .workflow_coordinators
            .get(workflow_id)
            .map(|entry| *entry.value())
            .or_else(|| prior_team_plan.as_ref().map(|plan| plan.coordinator_id))
            .unwrap_or(worker_ids[0]);
        let adaptive_team_plan = plan_adaptive_team(AdaptiveTeamSizingInputs {
            workflow_id: *workflow_id,
            graph_id: graph.graph_id,
            coordinator_id,
            topology_kind: graph.topology_plan.topology_kind,
            task_shape_kind: graph.topology_plan.task_shape.kind,
            decision_phase: if prior_team_plan.is_some() {
                "frontier_rebalance"
            } else {
                "initial"
            },
            structural_parallelism: graph.topology_plan.parallelism_width,
            branch_frontier_width: candidates.len(),
            dependency_depth: frontier_dependency_depth,
            available_worker_ids: worker_ids,
            worker_loads: &worker_loads,
            health_state: frontier_health_state,
            budget_pressure: frontier_budget_pressure,
            conservative_reasons: &conservative_reasons,
            existing_team_id: prior_team_plan.as_ref().map(|plan| plan.team_id),
        });
        let active_worker_ids = adaptive_team_plan.worker_ids.clone();
        self.adaptive_team_plans
            .insert(*workflow_id, adaptive_team_plan.clone());

        let mut decisions = Vec::new();
        for mut candidate in candidates.into_iter().take(active_worker_ids.len()) {
            let worker = select_worker(&active_worker_ids, &worker_loads);
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

                self.scheduler
                    .assign_to_team(task_id, worker, adaptive_team_plan.team_id)?;
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
                        graph_id: candidate.decision.graph_id,
                        branch_id: candidate.decision.branch_id,
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

            for decision in &decisions {
                if let Some(branch) = graph.branch(&decision.branch_id) {
                    self.record_autonomy_event(
                        workflow_id,
                        AutonomyEvent::BranchUpdated(AutonomyEventEnvelope {
                            workflow_id: *workflow_id,
                            graph_id: Some(graph.graph_id),
                            branch_id: Some(decision.branch_id),
                            payload: BranchSummary {
                                branch_id: branch.branch_id,
                                graph_id: branch.graph_id,
                                state: branch.state,
                                assigned_agents: branch.assigned_agents.clone(),
                                checkpoint_id: graph
                                    .latest_checkpoint(&branch.branch_id)
                                    .map(|checkpoint| checkpoint.checkpoint_id),
                                recovery_strategy: branch.recovery_strategy,
                            },
                            operator_visible: true,
                        }),
                    );
                }
            }

            drop(graph);
            self.record_status_update(workflow_id);
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
            .subtasks_in_states(parent_task_id, &[TaskState::Failed, TaskState::TimedOut])
    }

    pub fn validate_branch_delegation(
        &self,
        workflow_id: &TaskId,
        branch_id: ExecutionBranchId,
        capability: &CapabilitySummary,
    ) -> Result<(), AgentSystemError> {
        let graph = self.execution_graph(workflow_id).ok_or_else(|| {
            AgentSystemError::OrchestrationError(format!(
                "No execution graph found for workflow {workflow_id}"
            ))
        })?;

        let mut required_scopes = graph
            .nodes
            .iter()
            .filter(|node| node.branch_id == branch_id)
            .filter_map(|node| node.delegation_requirement);

        let Some(required_scope) = required_scopes.next() else {
            return Ok(());
        };

        if required_scopes.any(|scope| scope != required_scope) {
            return Err(AgentSystemError::PermissionDenied(format!(
                "delegation rejected for branch {branch_id}: branch contains multiple delegation scopes"
            )));
        }

        if capability.scope != required_scope
            || capability.rejection_reason.is_some()
            || capability.revocation_state != mister_smith_core::RevocationState::Active
        {
            return Err(AgentSystemError::PermissionDenied(format!(
                "delegation rejected for branch {branch_id}: {}",
                capability
                    .rejection_reason
                    .clone()
                    .unwrap_or_else(|| format!("{:?}", capability.revocation_state))
            )));
        }

        Ok(())
    }

    pub fn record_delegation_update(
        &self,
        workflow_id: &TaskId,
        branch_id: Option<ExecutionBranchId>,
        payload: CapabilitySummary,
    ) {
        let graph_id = self
            .execution_graph(workflow_id)
            .map(|graph| graph.graph_id);
        self.record_autonomy_event(
            workflow_id,
            AutonomyEvent::DelegationUpdated(AutonomyEventEnvelope {
                workflow_id: *workflow_id,
                graph_id,
                branch_id,
                payload,
                operator_visible: true,
            }),
        );
    }

    async fn recover_branch<S: BranchCheckpointStore + ?Sized>(
        &self,
        workflow_id: &TaskId,
        store: &S,
        branch_id: ExecutionBranchId,
        assigned_agent: Option<AgentId>,
        action: RecoveryAction,
        delegation: Option<&CapabilitySummary>,
    ) -> Result<BranchRecoveryPlan, AgentSystemError> {
        if let Some(capability) = delegation {
            if let Err(error) = self.validate_branch_delegation(workflow_id, branch_id, capability)
            {
                self.record_delegation_update(workflow_id, Some(branch_id), capability.clone());
                return Err(error);
            }
        }

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
                RecoveryAction::Resume => match delegation {
                    Some(capability) => {
                        self.branch_checkpoint_coordinator
                            .resume_branch_with_delegation(
                                store,
                                *workflow_id,
                                graph.value_mut(),
                                branch_id,
                                assigned_agent,
                                capability,
                            )
                            .await?
                    }
                    None => {
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
                },
                RecoveryAction::Reassign => {
                    let assigned_agent = assigned_agent.ok_or_else(|| {
                        AgentSystemError::OrchestrationError(
                            "branch reassignment requires a target agent".to_string(),
                        )
                    })?;
                    match delegation {
                        Some(capability) => {
                            self.branch_checkpoint_coordinator
                                .reassign_branch_with_delegation(
                                    store,
                                    *workflow_id,
                                    graph.value_mut(),
                                    branch_id,
                                    assigned_agent,
                                    capability,
                                )
                                .await?
                        }
                        None => {
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
                    }
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
        if let Some(capability) = delegation {
            self.record_delegation_update(workflow_id, Some(branch_id), capability.clone());
        }

        Ok(recovery)
    }

    fn record_autonomy_event(&self, workflow_id: &TaskId, event: AutonomyEvent) {
        self.autonomy_events
            .entry(*workflow_id)
            .or_default()
            .push(event.clone());

        if let Some(tx) = &self.autonomy_event_tx {
            if let Err(error) = tx.send(event.into_event(AUTONOMY_EVENT_SOURCE)) {
                warn!(%error, "Failed to queue autonomy event for shared event bus");
            }
        }
    }

    fn record_status_update(&self, workflow_id: &TaskId) {
        let Some(view) = self.autonomy_status(workflow_id) else {
            return;
        };

        self.record_autonomy_event(
            workflow_id,
            AutonomyEvent::StatusUpdated(Box::new(AutonomyEventEnvelope {
                workflow_id: *workflow_id,
                graph_id: Some(view.graph.graph_id),
                branch_id: None,
                payload: view,
                operator_visible: true,
            })),
        );
    }

    #[allow(dead_code)]
    fn update_step_routing_history(
        &self,
        workflow_id: &TaskId,
        history: &[StepRoutingDecisionSummary],
    ) {
        self.step_routing_histories
            .insert(*workflow_id, history.to_vec());
        self.record_status_update(workflow_id);
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

fn recompute_replayed_graph_state(graph: &mut ExecutionGraph) {
    let should_preserve_terminal_state = matches!(
        graph.state,
        GraphState::Completed | GraphState::Failed | GraphState::Aborted
    );
    let checkpointed_branches = graph
        .checkpoint_lineage
        .iter()
        .map(|checkpoint| checkpoint.branch_id)
        .collect::<HashSet<_>>();

    for branch in &mut graph.branches {
        let branch_nodes = graph
            .nodes
            .iter()
            .filter(|node| node.branch_id == branch.branch_id)
            .collect::<Vec<_>>();
        let has_checkpoint = checkpointed_branches.contains(&branch.branch_id);
        let has_active_nodes = branch_nodes
            .iter()
            .any(|node| matches!(node.state, NodeState::Running | NodeState::Checkpointed));
        let all_nodes_completed = !branch_nodes.is_empty()
            && branch_nodes
                .iter()
                .all(|node| node.state == NodeState::Completed);
        let any_node_failed = branch_nodes
            .iter()
            .any(|node| node.state == NodeState::Failed);

        branch.state = if any_node_failed {
            BranchState::Failed
        } else if all_nodes_completed {
            BranchState::Completed
        } else if matches!(
            branch.state,
            BranchState::Running | BranchState::Checkpointed | BranchState::Reassigned
        ) && (has_active_nodes || has_checkpoint)
        {
            branch.state
        } else if has_active_nodes {
            BranchState::Running
        } else {
            BranchState::Pending
        };
    }

    let any_failed_branch = graph
        .branches
        .iter()
        .any(|branch| branch.state == BranchState::Failed);
    let any_active_branch = graph.branches.iter().any(|branch| {
        matches!(
            branch.state,
            BranchState::Running | BranchState::Checkpointed | BranchState::Reassigned
        )
    });

    graph.state = if graph
        .nodes
        .iter()
        .any(|node| node.state == NodeState::Failed)
        || any_failed_branch
    {
        GraphState::Failed
    } else if should_preserve_terminal_state {
        graph.state
    } else if !graph.nodes.is_empty()
        && graph
            .nodes
            .iter()
            .all(|node| node.state == NodeState::Completed)
    {
        GraphState::Completed
    } else if matches!(graph.state, GraphState::Running | GraphState::Checkpointed)
        && any_active_branch
    {
        graph.state
    } else if graph
        .nodes
        .iter()
        .any(|node| matches!(node.state, NodeState::Running | NodeState::Checkpointed))
        || any_active_branch
    {
        GraphState::Running
    } else {
        GraphState::Pending
    };
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

fn apply_lifecycle_history_event(
    graph: &mut ExecutionGraph,
    event: &WorkflowHistoryEventRecord,
) -> Result<(), AgentSystemError> {
    if let Some(graph_state) =
        deserialize_history_value::<GraphState>(&event.payload, "graph_state")?
    {
        graph.state = graph_state;
    } else if let Some(lifecycle_state) = event.lifecycle_state.or(deserialize_history_value::<
        DurableWorkflowLifecycleState,
    >(
        &event.payload,
        "lifecycle_state",
    )?) {
        graph.state = match lifecycle_state {
            DurableWorkflowLifecycleState::Active
            | DurableWorkflowLifecycleState::Paused
            | DurableWorkflowLifecycleState::Cancelling => GraphState::Running,
            DurableWorkflowLifecycleState::Completed => GraphState::Completed,
            DurableWorkflowLifecycleState::Cancelled
            | DurableWorkflowLifecycleState::Terminated
            | DurableWorkflowLifecycleState::Failed => GraphState::Failed,
        };
    }

    Ok(())
}

fn apply_branch_history_event(
    graph: &mut ExecutionGraph,
    event: &WorkflowHistoryEventRecord,
) -> Result<(), AgentSystemError> {
    if let Some(checkpoint) = checkpoint_from_replay_payload(graph, &event.payload) {
        if graph.branch(&checkpoint.branch_id).is_some() {
            hydrate_checkpoint(graph, checkpoint);
        }
    }

    let Some(branch_id) = resolve_branch_id(graph, event)? else {
        return Err(AgentSystemError::OrchestrationError(format!(
            "branch replay failed: could not resolve branch_id for event {}",
            event.event_id
        )));
    };
    let Some(branch) = graph.branch_mut(&branch_id) else {
        return Err(AgentSystemError::OrchestrationError(format!(
            "branch replay failed: branch {} not found in graph for event {}",
            branch_id, event.event_id
        )));
    };

    if let Some(branch_state) =
        deserialize_history_value::<BranchState>(&event.payload, "branch_state")?
    {
        branch.state = branch_state;
    }
    if let Some(recovery_strategy) =
        deserialize_history_value::<BranchRecoveryStrategy>(&event.payload, "recovery_strategy")?
    {
        branch.recovery_strategy = recovery_strategy;
    }
    if let Some(assigned_agents) =
        deserialize_history_value::<Vec<AgentId>>(&event.payload, "assigned_agent_ids")?
    {
        branch.assigned_agents = assigned_agents;
    }

    Ok(())
}

fn apply_node_history_event(
    graph: &mut ExecutionGraph,
    event: &WorkflowHistoryEventRecord,
) -> Result<(), AgentSystemError> {
    let Some(node_index) = resolve_node_index(graph, event)? else {
        return Err(AgentSystemError::OrchestrationError(format!(
            "node replay failed: could not resolve node_index for event {}",
            event.event_id
        )));
    };
    let branch_id = graph.nodes[node_index].branch_id;

    if let Some(node_state) = deserialize_history_value::<NodeState>(&event.payload, "node_state")?
    {
        graph.nodes[node_index].state = node_state;
    }

    if let Some(branch) = graph.branch_mut(&branch_id) {
        if let Some(branch_state) =
            deserialize_history_value::<BranchState>(&event.payload, "branch_state")?
        {
            branch.state = branch_state;
        }
    }

    if let Some(graph_state) =
        deserialize_history_value::<GraphState>(&event.payload, "graph_state")?
    {
        graph.state = graph_state;
    }

    Ok(())
}

fn apply_compaction_history_event(
    graph: &mut ExecutionGraph,
    event: &WorkflowHistoryEventRecord,
) -> Result<(), AgentSystemError> {
    if let Some(graph_state) =
        deserialize_history_value::<GraphState>(&event.payload, "graph_state")?
    {
        graph.state = graph_state;
    }

    if let Some(branch_states) = event.payload.get("branch_states").and_then(Value::as_array) {
        for branch_snapshot in branch_states {
            let Some(branch_key) = branch_snapshot.get("branch_key").and_then(Value::as_str) else {
                return Err(AgentSystemError::OrchestrationError(format!(
                    "compaction replay failed: branch snapshot missing branch_key in event {}",
                    event.event_id
                )));
            };
            let Some(branch_id) = resolve_branch_key(graph, branch_key) else {
                return Err(AgentSystemError::OrchestrationError(format!(
                    "compaction replay failed: could not resolve branch_key '{}' for event {}",
                    branch_key, event.event_id
                )));
            };
            let Some(branch) = graph.branch_mut(&branch_id) else {
                return Err(AgentSystemError::OrchestrationError(format!(
                    "compaction replay failed: branch {} not found in graph for event {}",
                    branch_id, event.event_id
                )));
            };
            if let Some(branch_state) = branch_snapshot
                .get("branch_state")
                .cloned()
                .and_then(|raw| serde_json::from_value::<BranchState>(raw).ok())
            {
                branch.state = branch_state;
            }
            if let Some(recovery_strategy) = branch_snapshot
                .get("recovery_strategy")
                .cloned()
                .and_then(|raw| serde_json::from_value::<BranchRecoveryStrategy>(raw).ok())
            {
                branch.recovery_strategy = recovery_strategy;
            }
            if let Some(assigned_agents) = branch_snapshot
                .get("assigned_agent_ids")
                .cloned()
                .and_then(|raw| serde_json::from_value::<Vec<AgentId>>(raw).ok())
            {
                branch.assigned_agents = assigned_agents;
            }
        }
    }

    if let Some(node_states) = event.payload.get("node_states").and_then(Value::as_array) {
        for node_snapshot in node_states {
            let Some(step_key) = node_snapshot.get("step_key").and_then(Value::as_str) else {
                return Err(AgentSystemError::OrchestrationError(format!(
                    "compaction replay failed: node snapshot missing step_key in event {}",
                    event.event_id
                )));
            };
            let Some(node) = graph
                .nodes
                .iter_mut()
                .find(|node| node.step_key == step_key)
            else {
                return Err(AgentSystemError::OrchestrationError(format!(
                    "compaction replay failed: node with step_key '{}' not found in graph for event {}",
                    step_key, event.event_id
                )));
            };
            if let Some(node_state) = node_snapshot
                .get("node_state")
                .cloned()
                .and_then(|raw| serde_json::from_value::<NodeState>(raw).ok())
            {
                node.state = node_state;
            }
        }
    }

    Ok(())
}

fn resolve_branch_key(graph: &ExecutionGraph, branch_key: &str) -> Option<ExecutionBranchId> {
    Uuid::parse_str(branch_key)
        .ok()
        .map(ExecutionBranchId::from_uuid)
        .filter(|branch_id| graph.branch(branch_id).is_some())
        .or_else(|| {
            graph
                .nodes
                .iter()
                .find(|node| node.step_key == branch_key)
                .map(|node| node.branch_id)
        })
}

fn resolve_node_index(
    graph: &ExecutionGraph,
    event: &WorkflowHistoryEventRecord,
) -> Result<Option<usize>, AgentSystemError> {
    if let Some(node_id) = event.node_id {
        if let Some(index) = graph.nodes.iter().position(|node| node.node_id == node_id) {
            return Ok(Some(index));
        }
    }

    if let Some(step_id) = event
        .payload
        .get("step_id")
        .and_then(Value::as_str)
        .or_else(|| event.payload.get("step_key").and_then(Value::as_str))
    {
        return Ok(graph.nodes.iter().position(|node| node.step_key == step_id));
    }

    Ok(None)
}

fn resolve_branch_id(
    graph: &ExecutionGraph,
    event: &WorkflowHistoryEventRecord,
) -> Result<Option<ExecutionBranchId>, AgentSystemError> {
    if let Some(branch_id) = event.branch_id {
        if graph.branch(&branch_id).is_some() {
            return Ok(Some(branch_id));
        }
    }

    if let Some(step_id) = event
        .payload
        .get("step_id")
        .and_then(Value::as_str)
        .or_else(|| event.payload.get("step_key").and_then(Value::as_str))
        .or_else(|| {
            event
                .payload
                .get("branch_anchor_step_key")
                .and_then(Value::as_str)
        })
    {
        if let Some(node) = graph.nodes.iter().find(|node| node.step_key == step_id) {
            return Ok(Some(node.branch_id));
        }
    }

    Ok(None)
}

fn deserialize_history_value<T>(payload: &Value, key: &str) -> Result<Option<T>, AgentSystemError>
where
    T: serde::de::DeserializeOwned,
{
    payload
        .get(key)
        .cloned()
        .map(|value| {
            serde_json::from_value(value).map_err(|error| {
                AgentSystemError::OrchestrationError(format!(
                    "workflow history payload field '{key}' failed to deserialize: {error}"
                ))
            })
        })
        .transpose()
}

fn hydrate_checkpoint(
    graph: &mut ExecutionGraph,
    checkpoint: crate::execution_graph::BranchCheckpoint,
) {
    graph
        .checkpoint_lineage
        .retain(|existing| existing.checkpoint_id != checkpoint.checkpoint_id);
    graph.checkpoint_lineage.push(checkpoint);
    graph
        .checkpoint_lineage
        .sort_by_key(|entry| entry.created_at);
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

fn latest_branch_profile<'a>(
    graph: &ExecutionGraph,
    profiles: &'a [ProfileAssessment],
    branch_id: ExecutionBranchId,
) -> Option<&'a ProfileAssessment> {
    profiles
        .iter()
        .rev()
        .find(|assessment| assessment.target_branch_id(graph) == Some(branch_id))
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

fn aggregate_frontier_health<I>(health_states: I) -> HealthState
where
    I: IntoIterator<Item = HealthState>,
{
    let states = health_states.into_iter().collect::<Vec<_>>();
    if states.is_empty() {
        return HealthState::Unknown;
    }

    if states.contains(&HealthState::Unhealthy) {
        HealthState::Unhealthy
    } else if states.contains(&HealthState::Degraded) {
        HealthState::Degraded
    } else if states.contains(&HealthState::Unknown) {
        HealthState::Unknown
    } else {
        HealthState::Healthy
    }
}

fn aggregate_frontier_budget_pressure<I>(budget_pressures: I) -> Option<u8>
where
    I: IntoIterator<Item = u8>,
{
    let pressures = budget_pressures.into_iter().collect::<Vec<_>>();
    (!pressures.is_empty()).then(|| {
        let total = pressures
            .iter()
            .map(|pressure| u16::from(*pressure))
            .sum::<u16>();
        (total / pressures.len() as u16) as u8
    })
}

fn branch_id_for_target(
    graph: Option<&ExecutionGraph>,
    target: &GuardTarget,
) -> Option<ExecutionBranchId> {
    match target {
        GuardTarget::Branch(branch_id) => Some(*branch_id),
        GuardTarget::Node(node_id) => graph.and_then(|graph| {
            graph
                .nodes
                .iter()
                .find(|node| node.node_id == *node_id)
                .map(|node| node.branch_id)
        }),
        GuardTarget::Graph(_) | GuardTarget::Provider(_) => None,
    }
}

fn build_supervision_evidence_view(
    graph: &ExecutionGraph,
    profiles: &[ProfileAssessment],
    guard_decisions: &[GuardDecision],
    interventions: &[InterventionRecord],
) -> Option<SupervisionEvidenceView> {
    let intervention_record = interventions.last().cloned();
    let guard_decision = intervention_record
        .as_ref()
        .and_then(|record| {
            guard_decisions
                .iter()
                .rev()
                .find(|decision| decision.decision_id == record.decision_id)
        })
        .cloned()
        .or_else(|| guard_decisions.last().cloned());
    let profile = guard_decision
        .as_ref()
        .and_then(|decision| decision.evidence.profile_id)
        .and_then(|profile_id| {
            profiles.iter().rev().find(|assessment| {
                assessment
                    .snapshot()
                    .map(|snapshot| snapshot.profile_id == profile_id)
                    .unwrap_or(false)
            })
        })
        .or_else(|| {
            profiles
                .iter()
                .rev()
                .find(|assessment| assessment.snapshot().is_some())
        });
    let profile_snapshot = profile.and_then(ProfileAssessment::snapshot).cloned();

    let target_scope = guard_decision
        .as_ref()
        .map(|decision| {
            supervision_target_scope_from_guard_target(Some(graph), &decision.target_scope)
        })
        .or_else(|| {
            profile
                .and_then(ProfileAssessment::target)
                .map(|target| supervision_target_scope_from_guard_target(Some(graph), target))
        })
        .or_else(|| {
            profile_snapshot
                .as_ref()
                .map(supervision_target_scope_from_profile_snapshot)
        })?;
    let fingerprint_ref = profile_snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.fingerprint_ref.clone());
    let decision_basis = guard_decision
        .as_ref()
        .map(|decision| decision.evidence.decision_basis.as_str().to_string());
    let repair_lineage_ref =
        supervision_repair_lineage_ref_from_graph(graph, &target_scope, guard_decision.as_ref());

    Some(SupervisionEvidenceView {
        target_scope,
        fingerprint_ref,
        profile_snapshot,
        guard_decision,
        intervention_record,
        decision_basis,
        repair_lineage_ref,
        proof_boundary: Some(supported_task_path_proof_boundary()),
    })
}

fn build_runtime_truth_view(
    graph: &ExecutionGraph,
    topology: &TopologyPlanSummary,
    supervision_evidence: Option<&SupervisionEvidenceView>,
    step_routing_history: &[StepRoutingDecisionSummary],
    guard_decisions: &[GuardDecision],
) -> RuntimeTruthView {
    let branch_id = supervision_evidence.and_then(|evidence| evidence.target_scope.branch_id);
    let node_id = supervision_evidence.and_then(|evidence| evidence.target_scope.node_id);
    let mut relationships = vec![
        RunTraceRelationshipKind::Graph,
        RunTraceRelationshipKind::ToolBoundary,
    ];

    if !graph.branches.is_empty() {
        relationships.push(RunTraceRelationshipKind::Branch);
    }
    if node_id.is_some() || !graph.nodes.is_empty() {
        relationships.push(RunTraceRelationshipKind::Node);
    }
    if graph.branches.len() > 1 {
        relationships.push(RunTraceRelationshipKind::FanOut);
    }
    if topology.task_shape.has_join {
        relationships.push(RunTraceRelationshipKind::Join);
    }
    if supervision_evidence.is_some() {
        relationships.push(RunTraceRelationshipKind::Supervision);
    }
    if supervision_evidence
        .and_then(|evidence| evidence.repair_lineage_ref.as_ref())
        .is_some()
    {
        relationships.push(RunTraceRelationshipKind::Repair);
    }
    if step_routing_history.iter().any(|entry| {
        matches!(entry.action.as_str(), "retry" | "fallback")
            || matches!(entry.previous_action.as_deref(), Some("retry" | "fallback"))
    }) {
        relationships.push(RunTraceRelationshipKind::Retry);
    }
    if guard_decisions.iter().any(|decision| {
        decision
            .evidence
            .notes
            .iter()
            .any(|note| note.contains("handoff"))
    }) {
        relationships.push(RunTraceRelationshipKind::Handoff);
    }

    relationships.sort_by_key(|kind| *kind as u8);
    relationships.dedup();

    packet_023_placeholder_runtime_truth(
        graph.workflow_id,
        Some(graph.graph_id),
        branch_id,
        node_id,
        relationships,
        vec![],
    )
}

fn supervision_repair_lineage_ref_from_graph(
    graph: &ExecutionGraph,
    target_scope: &SupervisionTargetScope,
    guard_decision: Option<&GuardDecision>,
) -> Option<RepairLineageRef> {
    let checkpoint_ref = graph
        .checkpoint_lineage
        .iter()
        .filter(|checkpoint| checkpoint_matches_target(checkpoint.branch_id, target_scope))
        .max_by_key(|checkpoint| checkpoint.created_at)
        .map(|checkpoint| checkpoint.checkpoint_id.to_string())
        .or_else(|| {
            guard_decision.and_then(|decision| {
                decision
                    .evidence
                    .checkpoint_ids
                    .first()
                    .map(ToString::to_string)
            })
        })?;

    Some(RepairLineageRef {
        source: "packet-020".to_string(),
        checkpoint_ref: Some(checkpoint_ref),
    })
}

fn checkpoint_matches_target(
    checkpoint_branch_id: ExecutionBranchId,
    target_scope: &SupervisionTargetScope,
) -> bool {
    match target_scope.kind {
        SupervisionTargetKind::Provider => false,
        SupervisionTargetKind::Graph => true,
        SupervisionTargetKind::Branch | SupervisionTargetKind::Node => target_scope
            .branch_id
            .map(|branch_id| branch_id == checkpoint_branch_id)
            .unwrap_or(false),
    }
}

fn supported_task_path_proof_boundary() -> String {
    "supported task path".to_string()
}

fn supervision_target_scope_from_guard_target(
    graph: Option<&ExecutionGraph>,
    target: &GuardTarget,
) -> SupervisionTargetScope {
    let branch_id = branch_id_for_target(graph, target);
    match target {
        GuardTarget::Provider(provider) => SupervisionTargetScope {
            kind: SupervisionTargetKind::Provider,
            provider: Some(provider.clone()),
            graph_id: None,
            branch_id: None,
            node_id: None,
        },
        GuardTarget::Graph(graph_id) => SupervisionTargetScope {
            kind: SupervisionTargetKind::Graph,
            provider: None,
            graph_id: Some(*graph_id),
            branch_id,
            node_id: None,
        },
        GuardTarget::Branch(branch_id) => SupervisionTargetScope {
            kind: SupervisionTargetKind::Branch,
            provider: None,
            graph_id: graph.map(|graph| graph.graph_id),
            branch_id: Some(*branch_id),
            node_id: None,
        },
        GuardTarget::Node(node_id) => SupervisionTargetScope {
            kind: SupervisionTargetKind::Node,
            provider: None,
            graph_id: graph.map(|graph| graph.graph_id),
            branch_id,
            node_id: Some(*node_id),
        },
    }
}

fn supervision_target_scope_from_profile_snapshot(
    profile: &ProfileSnapshot,
) -> SupervisionTargetScope {
    match profile.target {
        mister_smith_core::ProfileTarget::Provider => SupervisionTargetScope {
            kind: SupervisionTargetKind::Provider,
            provider: None,
            graph_id: None,
            branch_id: None,
            node_id: None,
        },
        mister_smith_core::ProfileTarget::Topology => SupervisionTargetScope {
            kind: SupervisionTargetKind::Graph,
            provider: None,
            graph_id: None,
            branch_id: None,
            node_id: None,
        },
        mister_smith_core::ProfileTarget::Branch => SupervisionTargetScope {
            kind: SupervisionTargetKind::Branch,
            provider: None,
            graph_id: None,
            branch_id: None,
            node_id: None,
        },
        mister_smith_core::ProfileTarget::Agent => SupervisionTargetScope {
            kind: SupervisionTargetKind::Node,
            provider: None,
            graph_id: None,
            branch_id: None,
            node_id: None,
        },
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
    let profile = latest_branch_profile(graph, profiles, branch_id);
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
            graph_id: graph.graph_id,
            branch_id,
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

    pub fn sync_step_routing_history(&self, history: &[StepRoutingDecisionSummary]) {
        self.orchestrator
            .update_step_routing_history(&self.workflow_id, history);
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StepRoutingControl {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_preferred_tier: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification_checkpoints: Vec<StepVerificationCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_signal: Option<StepRoutingSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub history: Vec<StepRoutingDecisionSummary>,
}

#[cfg(feature = "llm")]
impl StepRoutingControl {
    pub fn request_hint(
        &self,
        step_id: impl Into<String>,
        step_index: Option<u32>,
        step_kind: impl Into<String>,
    ) -> RoutingHint {
        RoutingHint {
            preferred_tier: self.next_preferred_tier.clone(),
            step_metadata: Some(StepRoutingMetadata {
                step_id: step_id.into(),
                step_index,
                step_kind: Some(step_kind.into()),
            }),
            ..RoutingHint::default()
        }
    }

    pub fn verification_guidance(&self) -> Option<String> {
        if self.verification_checkpoints.is_empty() {
            return None;
        }

        let mut directives = Vec::new();
        if let Some(signal) = &self.last_signal {
            match signal.action {
                StepRoutingAction::Escalate => directives.push(
                    "Routing carryover: keep the stronger reasoning tier active until the step stabilizes."
                        .to_string(),
                ),
                StepRoutingAction::Downgrade => directives.push(
                    "Routing carryover: keep this step concise and avoid unnecessary optional breadth."
                        .to_string(),
                ),
                StepRoutingAction::Fallback => directives.push(
                    "Routing carryover: prefer resilient assumptions because the previous step required a fallback path."
                        .to_string(),
                ),
                StepRoutingAction::Continue => {}
            }
        }

        for checkpoint in &self.verification_checkpoints {
            if checkpoint.outcome != mister_smith_llm::StepVerificationOutcome::Triggered {
                continue;
            }
            match checkpoint.kind {
                StepVerificationCheckpointKind::ConfidenceReview => directives.push(
                    "Confidence review is active: perform an explicit self-check before finalizing."
                        .to_string(),
                ),
                StepVerificationCheckpointKind::BudgetPolicy => directives.push(
                    "Budget policy is active: stay compact and defer non-essential expansion."
                        .to_string(),
                ),
                StepVerificationCheckpointKind::ProviderFailure => directives.push(
                    "Provider failure fallback is active: restate critical assumptions and prefer robust output."
                        .to_string(),
                ),
                StepVerificationCheckpointKind::FinalTierGuard => directives.push(
                    "Final tier guard is active: include a compact verification pass before returning."
                        .to_string(),
                ),
            }
        }

        directives.sort();
        directives.dedup();
        Some(directives.join(" "))
    }

    pub fn apply_routing_decision(&mut self, decision: &RoutingDecision) {
        let previous_entry = self.history.last().cloned();
        let previous_preferred_tier = self.next_preferred_tier.clone();
        self.last_signal = Some(decision.carryover_signal.clone());
        self.verification_checkpoints = decision.carryover_signal.checkpoints.clone();

        match decision.carryover_signal.action {
            StepRoutingAction::Downgrade => {
                self.next_preferred_tier = None;
            }
            StepRoutingAction::Escalate | StepRoutingAction::Fallback => {
                self.next_preferred_tier = decision
                    .tier_label
                    .clone()
                    .or_else(|| Some(decision.provider_id.clone()));
            }
            StepRoutingAction::Continue => {
                if let Some(tier_label) = &decision.tier_label {
                    self.next_preferred_tier = Some(tier_label.clone());
                } else if self.next_preferred_tier.is_some() {
                    self.next_preferred_tier = Some(decision.provider_id.clone());
                }
            }
        }

        self.history.push(build_step_routing_summary(
            decision,
            previous_entry.as_ref(),
            previous_preferred_tier,
            self.next_preferred_tier.clone(),
        ));
    }
}

#[cfg(feature = "llm")]
fn step_routing_action_label(action: StepRoutingAction) -> &'static str {
    match action {
        StepRoutingAction::Continue => "continue",
        StepRoutingAction::Escalate => "escalate",
        StepRoutingAction::Downgrade => "downgrade",
        StepRoutingAction::Fallback => "fallback",
    }
}

#[cfg(feature = "llm")]
fn step_checkpoint_label(kind: StepVerificationCheckpointKind) -> &'static str {
    match kind {
        StepVerificationCheckpointKind::ConfidenceReview => "confidence_review",
        StepVerificationCheckpointKind::BudgetPolicy => "budget_policy",
        StepVerificationCheckpointKind::ProviderFailure => "provider_failure",
        StepVerificationCheckpointKind::FinalTierGuard => "final_tier_guard",
    }
}

#[cfg(feature = "llm")]
fn build_step_routing_summary(
    decision: &RoutingDecision,
    previous_entry: Option<&StepRoutingDecisionSummary>,
    preferred_tier_before: Option<String>,
    preferred_tier_after: Option<String>,
) -> StepRoutingDecisionSummary {
    let action = step_routing_action_label(decision.carryover_signal.action).to_string();
    let previous_action = previous_entry.map(|entry| entry.action.clone());
    let action_changed = previous_action
        .as_deref()
        .map(|prior| prior != action.as_str())
        .unwrap_or(false);
    let tier = decision
        .tier_label
        .clone()
        .unwrap_or_else(|| "direct".to_string());
    let triggered_checkpoints = decision
        .carryover_signal
        .checkpoints
        .iter()
        .filter(|checkpoint| {
            checkpoint.outcome == mister_smith_llm::StepVerificationOutcome::Triggered
        })
        .map(|checkpoint| step_checkpoint_label(checkpoint.kind).to_string())
        .collect::<Vec<_>>();

    let mut change_rationale = Vec::new();
    match previous_entry {
        Some(previous) => {
            change_rationale.push(format!(
                "previous step {} ended with action={} tier={}",
                previous.step_id, previous.action, previous.tier
            ));
            if action_changed {
                change_rationale.push(format!(
                    "action changed from {} to {}",
                    previous.action, action
                ));
            }
            if previous.tier != tier {
                change_rationale.push(format!("tier changed from {} to {}", previous.tier, tier));
            }
        }
        None => change_rationale
            .push("initial step routing decision for workflow-visible control state".to_string()),
    }

    if preferred_tier_before != preferred_tier_after {
        change_rationale.push(format!(
            "preferred tier updated from {} to {}",
            preferred_tier_before.as_deref().unwrap_or("none"),
            preferred_tier_after.as_deref().unwrap_or("none")
        ));
    }

    if !triggered_checkpoints.is_empty() {
        change_rationale.push(format!(
            "triggered checkpoints: {}",
            triggered_checkpoints.join(", ")
        ));
    }

    StepRoutingDecisionSummary {
        step_id: decision.carryover_signal.metadata.step_id.clone(),
        step_index: decision.carryover_signal.metadata.step_index,
        step_kind: decision.carryover_signal.metadata.step_kind.clone(),
        model_id: decision.model_id.clone(),
        tier,
        reason: decision.reason.clone(),
        previous_step_id: previous_entry.map(|entry| entry.step_id.clone()),
        previous_action,
        previous_tier: previous_entry.map(|entry| entry.tier.clone()),
        action,
        action_changed,
        preferred_tier_after,
        estimated_cost_tokens: decision.estimated_cost_tokens,
        confidence_score: decision
            .carryover_signal
            .confidence
            .as_ref()
            .map(|signal| signal.score),
        triggered_checkpoints,
        change_rationale,
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
    use mister_smith_core::{
        ExecutionEvidenceClass, FailureClass, GuardDecisionId, GuardEvidence, InterventionRecordId,
        InterventionType, ProfileSnapshotId, ProfileTarget, SemanticSignal, SemanticSignalKind,
        SupervisionDecisionBasis, SupervisionTargetKind, PACKET_023_GRAPH_EXECUTION_SUCCESS,
        PACKET_023_GROUNDED_TOOL_EXECUTION_MINIMAL, PACKET_023_ORCHESTRATION_ONLY,
        PACKET_023_SEMANTIC_COMPLETION_UNPROVEN,
    };
    use serde_json::json;

    fn semantic_signal(kind: SemanticSignalKind, severity: u8, detail: &str) -> SemanticSignal {
        SemanticSignal {
            signal_kind: kind,
            severity,
            detail: detail.to_string(),
        }
    }

    fn node_scoped_supervision_graph() -> (
        ExecutionGraph,
        Vec<TaskId>,
        ExecutionBranchId,
        ExecutionNodeId,
    ) {
        let graph = TopologyCompiler::default()
            .compile(
                TaskId::new(),
                &json!({
                    "goal": "guard-branch",
                    "steps": [
                        {
                            "id": "collect",
                            "step": 1,
                            "action": "collect",
                            "description": "Collect context"
                        },
                        {
                            "id": "branch-a",
                            "step": 2,
                            "action": "branch-a",
                            "description": "Branch A",
                            "depends_on": ["collect"],
                            "branch": "branch-a"
                        },
                        {
                            "id": "branch-b",
                            "step": 3,
                            "action": "branch-b",
                            "description": "Branch B",
                            "depends_on": ["collect"],
                            "branch": "branch-b"
                        }
                    ]
                }),
                &TopologySignals::default(),
            )
            .expect("graph should compile");

        let node_id = graph
            .nodes
            .iter()
            .find(|node| node.step_key == "branch-a")
            .map(|node| node.node_id)
            .expect("branch-a node should exist");
        let branch_id = graph
            .nodes
            .iter()
            .find(|node| node.node_id == node_id)
            .map(|node| node.branch_id)
            .expect("branch-a branch should exist");
        let task_ids = graph
            .nodes
            .iter()
            .map(|node| TaskId::from_uuid(*node.node_id.as_ref()))
            .collect::<Vec<_>>();

        (graph, task_ids, branch_id, node_id)
    }

    fn scheduler_for_graph(task_ids: &[TaskId]) -> Arc<TaskScheduler> {
        let scheduler = Arc::new(TaskScheduler::new());
        for task_id in task_ids {
            let mut task = TaskAssignment::new("analysis", json!({"branch": "guard"}));
            task.task_id = *task_id;
            scheduler.submit(task);
        }
        scheduler
    }

    #[test]
    fn build_runtime_truth_view_uses_packet_023_placeholder_boundary() {
        let (graph, _task_ids, branch_id, node_id) = node_scoped_supervision_graph();
        let topology = TopologyPlanSummary {
            graph_id: graph.graph_id,
            topology_kind: graph.topology_plan.topology_kind,
            parallelism_width: graph.topology_plan.parallelism_width,
            task_shape: graph.topology_plan.task_shape.clone(),
            coordination_policy: graph.topology_plan.coordination_policy,
            rationale: graph.topology_plan.rationale.clone(),
            fallback_topology: graph.topology_plan.fallback_topology,
        };
        let guard_decision = GuardDecision {
            decision_id: GuardDecisionId::new(),
            failure_class: FailureClass::Semantic,
            intervention: InterventionType::ContextRefresh,
            evidence: GuardEvidence {
                profile_id: None,
                decision_basis: SupervisionDecisionBasis::LiveSignalsOnly,
                signal_descriptions: vec!["handoff required".to_string()],
                checkpoint_ids: vec![],
                notes: vec!["handoff required before final answer".to_string()],
            },
            target_scope: GuardTarget::Node(node_id),
            operator_visibility: true,
        };
        let supervision_evidence = SupervisionEvidenceView {
            target_scope: SupervisionTargetScope {
                kind: SupervisionTargetKind::Node,
                provider: None,
                graph_id: Some(graph.graph_id),
                branch_id: Some(branch_id),
                node_id: Some(node_id),
            },
            fingerprint_ref: None,
            profile_snapshot: None,
            guard_decision: Some(guard_decision.clone()),
            intervention_record: None,
            decision_basis: Some(
                SupervisionDecisionBasis::LiveSignalsOnly
                    .as_str()
                    .to_string(),
            ),
            repair_lineage_ref: Some(RepairLineageRef {
                source: "packet-020".to_string(),
                checkpoint_ref: Some("checkpoint-1".to_string()),
            }),
            proof_boundary: Some("supported task path".to_string()),
        };

        let runtime_truth = build_runtime_truth_view(
            &graph,
            &topology,
            Some(&supervision_evidence),
            &[StepRoutingDecisionSummary {
                step_id: "branch-a.retry".to_string(),
                step_index: Some(2),
                step_kind: Some("worker".to_string()),
                model_id: "gpt-5.4".to_string(),
                tier: "llm-tier".to_string(),
                reason: "retry after verification".to_string(),
                previous_step_id: Some("branch-a.first".to_string()),
                previous_action: Some("retry".to_string()),
                previous_tier: Some("llm-tier".to_string()),
                action: "continue".to_string(),
                action_changed: true,
                preferred_tier_after: Some("llm-tier".to_string()),
                estimated_cost_tokens: Some(42),
                confidence_score: Some(0.71),
                triggered_checkpoints: vec![],
                change_rationale: vec!["retry preserved operator visibility".to_string()],
            }],
            &[guard_decision],
        );

        assert_eq!(
            runtime_truth.evidence_class,
            ExecutionEvidenceClass::PlaceholderOrSimulatedStepCompletion
        );
        assert_eq!(
            runtime_truth.proof_boundary.graph_execution,
            PACKET_023_GRAPH_EXECUTION_SUCCESS
        );
        assert_eq!(
            runtime_truth.proof_boundary.semantic_completion,
            PACKET_023_SEMANTIC_COMPLETION_UNPROVEN
        );
        assert_eq!(
            runtime_truth.proof_boundary.grounded_tool_execution,
            PACKET_023_GROUNDED_TOOL_EXECUTION_MINIMAL
        );
        assert_eq!(
            runtime_truth.proof_boundary.task_proof,
            PACKET_023_ORCHESTRATION_ONLY
        );
        assert_eq!(runtime_truth.run_trace.branch_id, Some(branch_id));
        assert_eq!(runtime_truth.run_trace.node_id, Some(node_id));
        assert!(runtime_truth
            .run_trace
            .relationships
            .contains(&RunTraceRelationshipKind::Repair));
        assert!(runtime_truth
            .run_trace
            .relationships
            .contains(&RunTraceRelationshipKind::Retry));
        assert!(runtime_truth
            .run_trace
            .relationships
            .contains(&RunTraceRelationshipKind::Handoff));
        assert!(runtime_truth
            .run_trace
            .relationships
            .contains(&RunTraceRelationshipKind::Supervision));
    }

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

    #[test]
    fn aggregate_frontier_health_tracks_worst_present_state() {
        assert_eq!(
            aggregate_frontier_health([
                HealthState::Healthy,
                HealthState::Unhealthy,
                HealthState::Unknown,
            ]),
            HealthState::Unhealthy
        );
        assert_eq!(
            aggregate_frontier_health([HealthState::Healthy, HealthState::Degraded]),
            HealthState::Degraded
        );
        assert_eq!(
            aggregate_frontier_health([HealthState::Healthy, HealthState::Unknown]),
            HealthState::Unknown
        );
    }

    #[test]
    fn autonomy_status_uses_structural_team_width_for_unassigned_harder_workload_graphs() {
        let scheduler = Arc::new(TaskScheduler::new());
        let orchestrator = Orchestrator::new(
            Arc::new(IdentityDecomposer),
            Arc::new(ArrayAggregator),
            scheduler,
        );
        let graph = TopologyCompiler::default()
            .compile(
                TaskId::new(),
                &json!({
                    "goal": "harder-proof",
                    "steps": [
                        {
                            "id": "root",
                            "step": 1,
                            "action": "root",
                            "description": "root"
                        },
                        {
                            "id": "alpha",
                            "step": 2,
                            "action": "alpha",
                            "description": "alpha",
                            "depends_on": ["root"],
                            "branch": "alpha"
                        },
                        {
                            "id": "beta",
                            "step": 3,
                            "action": "beta",
                            "description": "beta",
                            "depends_on": ["root"],
                            "branch": "beta"
                        },
                        {
                            "id": "gamma",
                            "step": 4,
                            "action": "gamma",
                            "description": "gamma",
                            "depends_on": ["root"],
                            "branch": "gamma"
                        }
                    ]
                }),
                &TopologySignals::default(),
            )
            .expect("harder workload graph should compile");
        let workflow_id = graph.workflow_id;
        orchestrator.register_execution_graph(graph);

        let team_sizing = orchestrator
            .autonomy_status(&workflow_id)
            .and_then(|status| status.team_sizing)
            .expect("autonomy status should expose baseline team sizing");

        assert_eq!(team_sizing.desired_workers, 3);
        assert_eq!(team_sizing.available_workers, 3);
        assert_eq!(team_sizing.selected_workers, 3);
        assert!(team_sizing
            .rationale_lines
            .iter()
            .any(|line| line.contains("non-terminal graph")));
    }

    #[test]
    fn autonomy_status_does_not_reuse_structural_team_width_for_unassigned_terminal_graphs() {
        for terminal_state in [
            mister_smith_core::GraphState::Completed,
            mister_smith_core::GraphState::Failed,
        ] {
            let scheduler = Arc::new(TaskScheduler::new());
            let orchestrator = Orchestrator::new(
                Arc::new(IdentityDecomposer),
                Arc::new(ArrayAggregator),
                scheduler,
            );
            let mut graph = TopologyCompiler::default()
                .compile(
                    TaskId::new(),
                    &json!({
                        "goal": "harder-proof",
                        "steps": [
                            {
                                "id": "root",
                                "step": 1,
                                "action": "root",
                                "description": "root"
                            },
                            {
                                "id": "alpha",
                                "step": 2,
                                "action": "alpha",
                                "description": "alpha",
                                "depends_on": ["root"],
                                "branch": "alpha"
                            },
                            {
                                "id": "beta",
                                "step": 3,
                                "action": "beta",
                                "description": "beta",
                                "depends_on": ["root"],
                                "branch": "beta"
                            },
                            {
                                "id": "gamma",
                                "step": 4,
                                "action": "gamma",
                                "description": "gamma",
                                "depends_on": ["root"],
                                "branch": "gamma"
                            }
                        ]
                    }),
                    &TopologySignals::default(),
                )
                .expect("harder workload graph should compile");
            let workflow_id = graph.workflow_id;
            graph.state = terminal_state;
            orchestrator.register_execution_graph(graph);

            let team_sizing = orchestrator
                .autonomy_status(&workflow_id)
                .and_then(|status| status.team_sizing)
                .expect("autonomy status should expose baseline team sizing");

            assert_eq!(team_sizing.desired_workers, 3);
            assert_eq!(team_sizing.available_workers, 1);
            assert_eq!(team_sizing.selected_workers, 1);
            assert_eq!(
                team_sizing.cap_reason.as_deref(),
                Some("available worker pool smaller than structural width")
            );
            assert!(team_sizing
                .rationale_lines
                .iter()
                .any(|line| line.contains("terminal graph")));
        }
    }

    #[test]
    fn build_supervision_evidence_prefers_guard_linked_profile_snapshot() {
        let graph = TopologyCompiler::default()
            .compile(
                TaskId::new(),
                &json!({
                    "goal": "profile-join",
                    "steps": [
                        {
                            "id": "root",
                            "step": 1,
                            "action": "root",
                            "description": "root"
                        },
                        {
                            "id": "alpha",
                            "step": 2,
                            "action": "alpha",
                            "description": "alpha",
                            "depends_on": ["root"],
                            "branch": "alpha"
                        },
                        {
                            "id": "beta",
                            "step": 3,
                            "action": "beta",
                            "description": "beta",
                            "depends_on": ["root"],
                            "branch": "beta"
                        }
                    ]
                }),
                &TopologySignals::default(),
            )
            .expect("multi-branch graph should compile");
        let branch_a = graph.branches[0].branch_id;

        let primary_profile_id = mister_smith_core::ProfileSnapshotId::new();
        let other_profile_id = mister_smith_core::ProfileSnapshotId::new();
        let primary_assessment = ProfileAssessment::new(
            Some(ProfileSnapshot {
                profile_id: primary_profile_id,
                target: mister_smith_core::ProfileTarget::Branch,
                health_state: HealthState::Degraded,
                latency_window: None,
                error_window: None,
                semantic_signals: vec![],
                fingerprint_ref: Some(mister_smith_core::ProfileFingerprintRef {
                    fingerprint_id: mister_smith_core::ProfileFingerprintId::new(),
                    fingerprint_key: "branch-a".to_string(),
                    confidence: 0.8,
                    expires_at: chrono::Utc::now(),
                }),
                updated_at: chrono::Utc::now(),
            }),
            Vec::new(),
        );
        let other_assessment = ProfileAssessment::new(
            Some(ProfileSnapshot {
                profile_id: other_profile_id,
                target: mister_smith_core::ProfileTarget::Branch,
                health_state: HealthState::Healthy,
                latency_window: None,
                error_window: None,
                semantic_signals: vec![],
                fingerprint_ref: Some(mister_smith_core::ProfileFingerprintRef {
                    fingerprint_id: mister_smith_core::ProfileFingerprintId::new(),
                    fingerprint_key: "branch-b".to_string(),
                    confidence: 0.4,
                    expires_at: chrono::Utc::now(),
                }),
                updated_at: chrono::Utc::now(),
            }),
            Vec::new(),
        );
        let decision = GuardDecision {
            decision_id: GuardDecisionId::new(),
            failure_class: FailureClass::Semantic,
            intervention: InterventionType::ContextRefresh,
            evidence: GuardEvidence {
                profile_id: Some(primary_profile_id),
                decision_basis: SupervisionDecisionBasis::FingerprintReinforced,
                signal_descriptions: vec!["loop detected".to_string()],
                checkpoint_ids: vec![],
                notes: vec!["join against the linked profile".to_string()],
            },
            target_scope: GuardTarget::Branch(branch_a),
            operator_visibility: true,
        };
        let intervention = InterventionRecord {
            record_id: InterventionRecordId::new(),
            decision_id: decision.decision_id,
            before_state: serde_json::json!({"state": "running"}),
            after_state: Some(serde_json::json!({"state": "refreshed"})),
            rationale: "context refresh".to_string(),
            emitted_at: chrono::Utc::now(),
        };

        let evidence = build_supervision_evidence_view(
            &graph,
            &[primary_assessment, other_assessment],
            std::slice::from_ref(&decision),
            std::slice::from_ref(&intervention),
        )
        .expect("supervision evidence should be assembled");

        assert_eq!(
            evidence
                .profile_snapshot
                .as_ref()
                .map(|snapshot| snapshot.profile_id),
            Some(primary_profile_id)
        );
        assert_eq!(
            evidence
                .fingerprint_ref
                .as_ref()
                .map(|reference| reference.fingerprint_key.as_str()),
            Some("branch-a")
        );
        assert_eq!(evidence.target_scope.graph_id, Some(graph.graph_id));
        assert_eq!(evidence.target_scope.branch_id, Some(branch_a));
    }

    #[tokio::test]
    async fn supervise_records_node_profile_events_with_branch_context() {
        let (graph, task_ids, branch_id, node_id) = node_scoped_supervision_graph();
        let workflow_id = graph.workflow_id;
        let task_id = TaskId::from_uuid(*node_id.as_ref());
        let scheduler = scheduler_for_graph(&task_ids);
        let agent = AgentId::new();

        scheduler.assign(&task_id, agent).unwrap();
        scheduler.start(&task_id).unwrap();

        let orchestrator = Arc::new(Orchestrator::new(
            Arc::new(IdentityDecomposer),
            Arc::new(ArrayAggregator),
            scheduler.clone(),
        ));
        orchestrator.register_execution_graph(graph);

        let profile_id = ProfileSnapshotId::new();
        let assessment = ProfileAssessment::new(
            Some(ProfileSnapshot {
                profile_id,
                target: ProfileTarget::Agent,
                health_state: HealthState::Degraded,
                latency_window: None,
                error_window: None,
                semantic_signals: vec![semantic_signal(
                    SemanticSignalKind::MissingContext,
                    74,
                    "missing node-scoped repair context",
                )],
                fingerprint_ref: None,
                updated_at: chrono::Utc::now(),
            }),
            vec!["step boundary node evidence".to_string()],
        );

        let (decision, _record) = orchestrator
            .supervise(
                &workflow_id,
                GuardContext::new(GuardTarget::Node(node_id)).with_profile(assessment),
            )
            .await
            .expect("node-scoped supervision should succeed");

        assert_eq!(decision.target_scope, GuardTarget::Node(node_id));
        assert_eq!(decision.intervention, InterventionType::ContextRefresh);
        assert_eq!(
            scheduler.get(&task_id).expect("task should exist").state,
            TaskState::Pending
        );

        let profile_event = orchestrator
            .autonomy_events(&workflow_id)
            .into_iter()
            .find_map(|event| match event {
                AutonomyEvent::ProfileSnapshotRecorded(envelope) => Some(envelope),
                _ => None,
            })
            .expect("profile snapshot event should be recorded");
        assert_eq!(profile_event.branch_id, Some(branch_id));
        assert_eq!(profile_event.payload.profile_id, profile_id);

        let status = orchestrator
            .autonomy_status(&workflow_id)
            .expect("status should be available");
        let supervision_evidence = status
            .supervision_evidence
            .expect("supervision evidence should be projected");
        assert_eq!(
            supervision_evidence.target_scope.kind,
            SupervisionTargetKind::Node
        );
        assert_eq!(supervision_evidence.target_scope.branch_id, Some(branch_id));
        assert_eq!(supervision_evidence.target_scope.node_id, Some(node_id));
        assert_eq!(
            supervision_evidence.decision_basis.as_deref(),
            Some(SupervisionDecisionBasis::LiveSignalsOnly.as_str())
        );
        let runtime_truth = status
            .runtime_truth
            .expect("runtime truth should be projected");
        assert_eq!(
            runtime_truth.evidence_class.as_str(),
            "placeholder_or_simulated_step_completion"
        );
        assert_eq!(
            runtime_truth.proof_boundary.task_proof,
            "result is orchestration proof, not substantive task proof"
        );
        assert!(runtime_truth
            .run_trace
            .relationships
            .contains(&RunTraceRelationshipKind::Supervision));
        assert!(status.guard_decisions[0]
            .evidence
            .notes
            .iter()
            .any(|note| note.contains("node evidence")));
    }
}
