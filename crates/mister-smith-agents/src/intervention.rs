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
                let branch_index = graph
                    .branches
                    .iter()
                    .position(|branch| branch.branch_id == *branch_id)
                    .ok_or_else(|| {
                        GuardError::InvalidTarget(format!("unknown branch {branch_id}"))
                    })?;
                let before_state = serde_json::to_value(graph.branches[branch_index].clone())
                    .unwrap_or_else(|_| json!({}));
                let node_ids = graph.branches[branch_index].node_ids.clone();
                let mut abort_graph = false;

                match decision.intervention {
                    InterventionType::Retry | InterventionType::ContextRefresh => {
                        graph.state = mister_smith_core::GraphState::Running;
                        let branch = &mut graph.branches[branch_index];
                        branch.state = BranchState::Checkpointed;
                        branch.recovery_strategy = BranchRecoveryStrategy::Resume;
                        for node_id in &node_ids {
                            let task_id = TaskId::from_uuid(*node_id.as_ref());
                            let _ = scheduler.apply_intervention(&task_id, decision.intervention);
                        }
                    }
                    InterventionType::Failover | InterventionType::Reassignment => {
                        graph.state = mister_smith_core::GraphState::Running;
                        let branch = &mut graph.branches[branch_index];
                        branch.state = BranchState::Reassigned;
                        branch.recovery_strategy = BranchRecoveryStrategy::Reassign;
                        for node_id in &node_ids {
                            let task_id = TaskId::from_uuid(*node_id.as_ref());
                            let _ = scheduler
                                .apply_intervention(&task_id, InterventionType::Reassignment);
                        }
                    }
                    InterventionType::BranchIsolation => {
                        graph.state = mister_smith_core::GraphState::Running;
                        let branch = &mut graph.branches[branch_index];
                        branch.state = BranchState::Isolated;
                        branch.recovery_strategy = BranchRecoveryStrategy::Isolate;
                        for node_id in &node_ids {
                            let task_id = TaskId::from_uuid(*node_id.as_ref());
                            let _ = scheduler
                                .apply_intervention(&task_id, InterventionType::BranchIsolation);
                        }
                    }
                    InterventionType::Escalation => {
                        let branch = &mut graph.branches[branch_index];
                        branch.state = BranchState::Failed;
                        branch.recovery_strategy = BranchRecoveryStrategy::Escalate;
                    }
                    InterventionType::Abort => {
                        let branch = &mut graph.branches[branch_index];
                        branch.state = BranchState::Failed;
                        abort_graph = true;
                        for node_id in &node_ids {
                            let task_id = TaskId::from_uuid(*node_id.as_ref());
                            let _ = scheduler.apply_intervention(&task_id, InterventionType::Abort);
                        }
                    }
                }

                let after_state = serde_json::to_value(graph.branches[branch_index].clone()).ok();
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
            GuardTarget::Provider(provider) => Ok(InterventionRecord {
                record_id: InterventionRecordId::new(),
                decision_id: decision.decision_id,
                before_state: json!({ "provider": provider }),
                after_state: Some(json!({
                    "provider": provider,
                    "intervention": format!("{:?}", decision.intervention)
                })),
                rationale: rationale_for(decision),
                emitted_at: Utc::now(),
            }),
        }
    }
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
