//! Branch-local Guard intervention application.

use chrono::Utc;
use serde_json::json;

use mister_smith_core::{
    BranchRecoveryStrategy, BranchState, GuardDecision, GuardError, GuardTarget,
    InterventionRecord, InterventionRecordId, InterventionType, TaskId,
};

use crate::execution_graph::ExecutionGraph;
use crate::scheduler::TaskScheduler;

/// Applies typed Guard decisions to the current execution graph and scheduler state.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InterventionEngine;

impl InterventionEngine {
    /// Apply an intervention to the targeted branch, node, graph, or provider dependency.
    pub fn apply(
        &self,
        decision: &GuardDecision,
        scheduler: &TaskScheduler,
        graph: &mut ExecutionGraph,
    ) -> Result<InterventionRecord, GuardError> {
        match &decision.target_scope {
            GuardTarget::Branch(branch_id) => {
                let before_state = serde_json::to_value(
                    graph
                        .branch(branch_id)
                        .ok_or_else(|| {
                            GuardError::InvalidTarget(format!("unknown branch {branch_id}"))
                        })?
                        .clone(),
                )
                .unwrap_or_else(|_| json!({}));
                let task_ids = branch_task_ids(graph, branch_id)?;
                let mut abort_graph = false;

                match decision.intervention {
                    InterventionType::Retry | InterventionType::ContextRefresh => {
                        graph.state = mister_smith_core::GraphState::Running;
                        let branch = graph.branch_mut(branch_id).ok_or_else(|| {
                            GuardError::InvalidTarget(format!("unknown branch {branch_id}"))
                        })?;
                        branch.state = BranchState::Checkpointed;
                        branch.recovery_strategy = BranchRecoveryStrategy::Resume;
                        apply_scheduler_intervention(scheduler, &task_ids, decision.intervention)?;
                    }
                    InterventionType::Failover | InterventionType::Reassignment => {
                        graph.state = mister_smith_core::GraphState::Running;
                        let branch = graph.branch_mut(branch_id).ok_or_else(|| {
                            GuardError::InvalidTarget(format!("unknown branch {branch_id}"))
                        })?;
                        branch.state = BranchState::Reassigned;
                        branch.recovery_strategy = BranchRecoveryStrategy::Reassign;
                        apply_scheduler_intervention(
                            scheduler,
                            &task_ids,
                            InterventionType::Reassignment,
                        )?;
                    }
                    InterventionType::BranchIsolation => {
                        graph.state = mister_smith_core::GraphState::Running;
                        let branch = graph.branch_mut(branch_id).ok_or_else(|| {
                            GuardError::InvalidTarget(format!("unknown branch {branch_id}"))
                        })?;
                        branch.state = BranchState::Isolated;
                        branch.recovery_strategy = BranchRecoveryStrategy::Isolate;
                        apply_scheduler_intervention(
                            scheduler,
                            &task_ids,
                            InterventionType::BranchIsolation,
                        )?;
                    }
                    InterventionType::Escalation => {
                        let branch = graph.branch_mut(branch_id).ok_or_else(|| {
                            GuardError::InvalidTarget(format!("unknown branch {branch_id}"))
                        })?;
                        branch.state = BranchState::Failed;
                        branch.recovery_strategy = BranchRecoveryStrategy::Escalate;
                        apply_scheduler_intervention(
                            scheduler,
                            &task_ids,
                            InterventionType::Escalation,
                        )?;
                    }
                    InterventionType::Abort => {
                        let branch = graph.branch_mut(branch_id).ok_or_else(|| {
                            GuardError::InvalidTarget(format!("unknown branch {branch_id}"))
                        })?;
                        branch.state = BranchState::Failed;
                        abort_graph = true;
                        apply_scheduler_intervention(
                            scheduler,
                            &task_ids,
                            InterventionType::Abort,
                        )?;
                    }
                }

                let after_state = graph
                    .branch(branch_id)
                    .cloned()
                    .and_then(|branch| serde_json::to_value(branch).ok());
                if abort_graph {
                    graph.state = mister_smith_core::GraphState::Aborted;
                }
                Ok(InterventionRecord {
                    record_id: InterventionRecordId::new(),
                    decision_id: decision.decision_id,
                    before_state,
                    after_state,
                    rationale: rationale_for(decision),
                    emitted_at: Utc::now(),
                })
            }
            GuardTarget::Node(node_id) => {
                let task_id = TaskId::from_uuid(*node_id.as_ref());
                let before_state = scheduler
                    .get(&task_id)
                    .map(|task| serde_json::to_value(task).unwrap_or_else(|_| json!({})))
                    .unwrap_or_else(|| json!({"task_id": task_id.to_string()}));
                scheduler
                    .apply_intervention(&task_id, decision.intervention)
                    .map_err(|error| GuardError::InvalidTarget(error.to_string()))?;
                let after_state = scheduler
                    .get(&task_id)
                    .map(|task| serde_json::to_value(task).unwrap_or_else(|_| json!({})));
                Ok(InterventionRecord {
                    record_id: InterventionRecordId::new(),
                    decision_id: decision.decision_id,
                    before_state,
                    after_state,
                    rationale: rationale_for(decision),
                    emitted_at: Utc::now(),
                })
            }
            GuardTarget::Graph(graph_id) => {
                let before_state =
                    serde_json::to_value(graph.clone()).unwrap_or_else(|_| json!({}));
                if matches!(decision.intervention, InterventionType::Abort) {
                    graph.state = mister_smith_core::GraphState::Aborted;
                    let task_ids = graph
                        .nodes
                        .iter()
                        .map(|node| TaskId::from_uuid(*node.node_id.as_ref()))
                        .collect::<Vec<_>>();
                    apply_scheduler_intervention(scheduler, &task_ids, InterventionType::Abort)?;
                }
                let after_state = serde_json::to_value(graph.clone()).ok();
                Ok(InterventionRecord {
                    record_id: InterventionRecordId::new(),
                    decision_id: decision.decision_id,
                    before_state,
                    after_state,
                    rationale: format!("{} for graph {}", rationale_for(decision), graph_id),
                    emitted_at: Utc::now(),
                })
            }
            GuardTarget::Provider(provider) => Ok(provider_intervention_record(decision, provider)),
        }
    }

    /// Apply a provider-only intervention when no execution graph exists yet.
    pub fn apply_without_graph(
        &self,
        decision: &GuardDecision,
    ) -> Result<InterventionRecord, GuardError> {
        match &decision.target_scope {
            GuardTarget::Provider(provider) => Ok(provider_intervention_record(decision, provider)),
            target => Err(GuardError::InvalidTarget(format!(
                "graphless intervention is unsupported for target {target:?}"
            ))),
        }
    }
}

fn branch_task_ids(
    graph: &ExecutionGraph,
    branch_id: &mister_smith_core::ExecutionBranchId,
) -> Result<Vec<TaskId>, GuardError> {
    let branch = graph
        .branch(branch_id)
        .ok_or_else(|| GuardError::InvalidTarget(format!("unknown branch {branch_id}")))?;
    let scoped_node_ids = graph.recovery_node_ids(branch_id);
    if scoped_node_ids.is_empty() {
        return Ok(branch
            .node_ids
            .iter()
            .map(|node_id| TaskId::from_uuid(*node_id.as_ref()))
            .collect());
    }

    Ok(scoped_node_ids
        .into_iter()
        .map(|node_id| TaskId::from_uuid(*node_id.as_ref()))
        .collect())
}

fn apply_scheduler_intervention(
    scheduler: &TaskScheduler,
    task_ids: &[TaskId],
    intervention: InterventionType,
) -> Result<(), GuardError> {
    for task_id in task_ids {
        scheduler
            .apply_intervention(task_id, intervention)
            .map_err(|error| GuardError::InvalidTarget(error.to_string()))?;
    }
    Ok(())
}

fn rationale_for(decision: &GuardDecision) -> String {
    match decision.intervention {
        InterventionType::Retry => "applied retry for targeted recovery".to_string(),
        InterventionType::Failover => "applied failover for targeted recovery".to_string(),
        InterventionType::ContextRefresh => {
            "applied context refresh for semantic degradation".to_string()
        }
        InterventionType::BranchIsolation => {
            "applied branch isolation for targeted supervisory recovery".to_string()
        }
        InterventionType::Reassignment => "applied reassignment for targeted recovery".to_string(),
        InterventionType::Escalation => {
            "applied escalation under conservative fallback".to_string()
        }
        InterventionType::Abort => "applied abort to stop unsafe execution".to_string(),
    }
}

fn provider_intervention_record(decision: &GuardDecision, provider: &str) -> InterventionRecord {
    InterventionRecord {
        record_id: InterventionRecordId::new(),
        decision_id: decision.decision_id,
        before_state: json!({ "provider": provider }),
        after_state: Some(json!({
            "provider": provider,
            "intervention": format!("{:?}", decision.intervention)
        })),
        rationale: rationale_for(decision),
        emitted_at: Utc::now(),
    }
}
