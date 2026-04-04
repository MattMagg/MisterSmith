//! EventBus: in-process pub/sub with broadcast, filtering, and dead letter handling.
//!
//! The [`EventBus`] is the central event distribution mechanism. It implements
//! the core [`EventPublisher`] trait, allowing
//! any component that depends on `mister-smith-core` to publish events without
//! depending on this crate directly.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;
use tokio::sync::{broadcast, RwLock};
use tracing;

use mister_smith_core::{
    packet_023_placeholder_runtime_truth, CheckpointId, ContextBudgetId,
    CoordinatorDelegationRecord, CoordinatorMergeDecision, CoordinatorRuntimeProofView,
    CoordinatorSubordinateInboxRecord, DelegatedWorkEvidenceRef, EventPublisher,
    ExecutionBranchId, ExecutionGraphId, GraphState, GuardDecision, GuardDecisionId, GuardTarget,
    InterventionRecord, InterventionRecordId, OperatorResultPreview, ProfileSnapshot,
    ProfileSnapshotId, ProfileTarget, RepairLineageRef, RunTraceRelationshipKind, RuntimeTruthView,
    StepPolicySummaryView, SubagentStateRecord, SupervisionEvidenceView,
    SupervisionTargetKind, SupervisionTargetScope, SystemEvent, TaskId, TeamSizingDecision,
};

use crate::autonomy::{
    infer_result_preview_from_projection, merge_operator_result_preview, AutonomyEvent,
    AutonomyStatusView, BranchSummary, CapabilitySummary, CheckpointRecordSummary,
    ContextPressureSummary, DelegationAlert, ExecutionGraphSummary,
    ExternalCapabilityDecisionSummary, ResumeProvenanceSummary, RoutingDecisionSummary,
    StepRoutingDecisionSummary, TopologyPlanSummary,
};
use crate::dead_letter::DeadLetterQueue;
use crate::error::EventBusError;
use crate::handler::{EventFilter, EventHandler};
use crate::store::EventStore;
use crate::types::Event;

/// Default broadcast channel capacity.
const DEFAULT_BROADCAST_CAPACITY: usize = 10_000;

#[derive(Debug, Clone, Default)]
struct AutonomyStatusAccumulator {
    session_id: Option<mister_smith_core::SessionId>,
    turn_index: Option<u32>,
    coordinator_agent_id: Option<mister_smith_core::AgentId>,
    resume_provenance: Option<ResumeProvenanceSummary>,
    lifecycle_state: Option<mister_smith_core::DurableWorkflowLifecycleState>,
    result_preview: Option<OperatorResultPreview>,
    graph: Option<ExecutionGraphSummary>,
    topology: Option<TopologyPlanSummary>,
    team_sizing: Option<TeamSizingDecision>,
    branches: HashMap<ExecutionBranchId, BranchSummary>,
    checkpoint_lineage: HashMap<CheckpointId, CheckpointRecordSummary>,
    memory_pressure: HashMap<ContextBudgetId, ContextPressureSummary>,
    routing_history: Vec<RoutingDecisionSummary>,
    step_routing_history: Vec<StepRoutingDecisionSummary>,
    interventions: HashMap<InterventionRecordId, InterventionRecord>,
    delegation_capabilities: HashMap<mister_smith_core::CapabilityId, CapabilitySummary>,
    delegation_alerts: HashMap<String, DelegationAlert>,
    delegation_records: Vec<CoordinatorDelegationRecord>,
    subordinate_inbox: Vec<CoordinatorSubordinateInboxRecord>,
    subagent_states: Vec<SubagentStateRecord>,
    delegated_work_evidence: Vec<DelegatedWorkEvidenceRef>,
    coordinator_decisions: Vec<CoordinatorMergeDecision>,
    coordinator_runtime_proof: Option<CoordinatorRuntimeProofView>,
    external_capability_decisions: Vec<ExternalCapabilityDecisionSummary>,
    profiles: HashMap<ProfileSnapshotId, ProfileSnapshot>,
    guard_decisions: HashMap<GuardDecisionId, GuardDecision>,
    guard_decision_branch_ids: HashMap<GuardDecisionId, Option<ExecutionBranchId>>,
    runtime_truth: Option<RuntimeTruthView>,
    supervision_evidence: Option<SupervisionEvidenceView>,
    step_policy: Option<StepPolicySummaryView>,
    conservative_reasons: Vec<String>,
    latest_profile_id: Option<ProfileSnapshotId>,
    latest_guard_decision_id: Option<GuardDecisionId>,
    latest_intervention_id: Option<InterventionRecordId>,
}

impl AutonomyStatusAccumulator {
    fn apply(&mut self, event: AutonomyEvent) {
        match event {
            AutonomyEvent::GraphUpdated(envelope) => {
                self.graph = Some(envelope.payload);
            }
            AutonomyEvent::TopologySelected(envelope) => {
                if let Some(graph) = self.graph.as_mut() {
                    graph.active_topology = Some(envelope.payload.topology_kind);
                }
                self.topology = Some(envelope.payload);
            }
            AutonomyEvent::BranchUpdated(envelope) => {
                self.branches
                    .insert(envelope.payload.branch_id, envelope.payload);
            }
            AutonomyEvent::ContextPressureObserved(envelope) => {
                self.memory_pressure
                    .insert(envelope.payload.budget_id, envelope.payload);
            }
            AutonomyEvent::ProfileSnapshotRecorded(envelope) => {
                self.latest_profile_id = Some(envelope.payload.profile_id);
                self.profiles
                    .insert(envelope.payload.profile_id, envelope.payload);
            }
            AutonomyEvent::GuardDecisionEvaluated(envelope) => {
                let decision_id = envelope.payload.decision_id;
                self.latest_guard_decision_id = Some(decision_id);
                self.push_conservative_reasons(
                    envelope
                        .payload
                        .evidence
                        .notes
                        .iter()
                        .filter(|note| note.contains("conservative fallback"))
                        .cloned(),
                );
                self.guard_decision_branch_ids
                    .insert(decision_id, envelope.branch_id);
                self.guard_decisions.insert(decision_id, envelope.payload);
            }
            AutonomyEvent::InterventionRecorded(envelope) => {
                self.latest_intervention_id = Some(envelope.payload.record_id);
                self.interventions
                    .insert(envelope.payload.record_id, envelope.payload);
            }
            AutonomyEvent::CheckpointRecorded(envelope) => {
                if let Some(branch) = self.branches.get_mut(&envelope.payload.branch_id) {
                    branch.checkpoint_id = Some(envelope.payload.checkpoint_id);
                    branch.recovery_strategy = envelope.payload.recovery_strategy;
                }
                self.checkpoint_lineage
                    .insert(envelope.payload.checkpoint_id, envelope.payload);
            }
            AutonomyEvent::RoutingDecisionRecorded(envelope) => {
                if !self.routing_history.contains(&envelope.payload) {
                    self.routing_history.push(envelope.payload);
                }
            }
            AutonomyEvent::DelegationUpdated(envelope) => {
                self.update_delegation(envelope.payload);
            }
            AutonomyEvent::DelegationDecisionRecorded(envelope) => {
                self.update_external_capability_decision(envelope.payload);
            }
            AutonomyEvent::StatusUpdated(envelope) => {
                *self = Self::from_view(envelope.payload.clone());
            }
        }
    }

    fn view(&self) -> Option<AutonomyStatusView> {
        let graph = self.graph.clone()?;
        let topology = self.topology.clone()?;

        let mut branches = self.branches.values().cloned().collect::<Vec<_>>();
        branches.sort_by_key(|branch| branch.branch_id.to_string());

        let mut checkpoint_lineage = self
            .checkpoint_lineage
            .values()
            .cloned()
            .collect::<Vec<_>>();
        checkpoint_lineage.sort_by(|left, right| {
            left.captured_at.cmp(&right.captured_at).then_with(|| {
                left.checkpoint_id
                    .to_string()
                    .cmp(&right.checkpoint_id.to_string())
            })
        });

        let mut memory_pressure = self.memory_pressure.values().cloned().collect::<Vec<_>>();
        memory_pressure.sort_by_key(|pressure| pressure.budget_id.to_string());

        let mut interventions = self.interventions.values().cloned().collect::<Vec<_>>();
        interventions.sort_by(|left, right| left.emitted_at.cmp(&right.emitted_at));

        let mut delegation_capabilities = self
            .delegation_capabilities
            .values()
            .cloned()
            .collect::<Vec<_>>();
        delegation_capabilities.sort_by(|left, right| {
            left.expires_at.cmp(&right.expires_at).then_with(|| {
                left.capability_id
                    .to_string()
                    .cmp(&right.capability_id.to_string())
            })
        });

        let mut delegation_alerts = self.delegation_alerts.values().cloned().collect::<Vec<_>>();
        delegation_alerts.sort_by_key(|alert| alert.message.clone());

        let mut external_capability_decisions = self.external_capability_decisions.clone();
        external_capability_decisions.sort_by(|left, right| {
            left.observed_at.cmp(&right.observed_at).then_with(|| {
                external_capability_decision_key(left).cmp(&external_capability_decision_key(right))
            })
        });

        let mut profiles = self.profiles.values().cloned().collect::<Vec<_>>();
        profiles.sort_by(|left, right| left.updated_at.cmp(&right.updated_at));

        let mut guard_decisions = self.guard_decisions.values().cloned().collect::<Vec<_>>();
        guard_decisions.sort_by_key(|decision| decision.decision_id.to_string());

        let team_sizing = self.team_sizing.clone().or_else(|| {
            infer_team_sizing_from_projection(
                &graph,
                &topology,
                &branches,
                &self.routing_history,
                &self.conservative_reasons,
            )
        });
        let inferred_result_preview = infer_result_preview_from_projection(
            &graph,
            &topology,
            &branches,
            &self.routing_history,
            self.step_policy.as_ref(),
        );
        let mut result_preview = match (
            self.result_preview.as_ref(),
            inferred_result_preview.as_ref(),
        ) {
            (Some(preview), Some(inferred)) => {
                Some(merge_operator_result_preview(preview, inferred))
            }
            (Some(preview), None) => Some(preview.clone()),
            (None, Some(inferred)) => Some(inferred.clone()),
            (None, None) => None,
        };

        let supervision_evidence = self
            .synthesize_supervision_evidence()
            .map(|evidence| {
                if let Some(preserved) = self.supervision_evidence.as_ref() {
                    merge_supervision_evidence(evidence, preserved)
                } else {
                    evidence
                }
            })
            .or_else(|| self.supervision_evidence.clone());
        let synthesized_runtime_truth = synthesize_runtime_truth(
            &graph,
            &topology,
            &branches,
            &self.step_routing_history,
            supervision_evidence.as_ref(),
            &guard_decisions,
        );
        let runtime_truth = self
            .runtime_truth
            .clone()
            .or(Some(synthesized_runtime_truth.clone()));
        let step_policy = self.step_policy.clone();
        if let Some(preview) = result_preview.as_mut() {
            if preview.runtime_truth.is_none() && graph.state == GraphState::Completed {
                preview.runtime_truth = Some(synthesized_runtime_truth.clone());
            }
            if preview.step_policy.is_none() {
                preview.step_policy = step_policy.clone();
            }
        }

        Some(AutonomyStatusView {
            session_id: self.session_id,
            turn_index: self.turn_index,
            coordinator_agent_id: self.coordinator_agent_id,
            resume_provenance: self.resume_provenance.clone(),
            lifecycle_state: self.lifecycle_state.or(Some(
                crate::autonomy::lifecycle_state_for_graph_state(graph.state),
            )),
            result_preview,
            graph,
            topology,
            team_sizing,
            branches,
            checkpoint_lineage,
            memory_pressure,
            routing_history: self.routing_history.clone(),
            step_routing_history: self.step_routing_history.clone(),
            interventions,
            delegation_capabilities,
            delegation_alerts,
            delegation_records: self.delegation_records.clone(),
            subordinate_inbox: self.subordinate_inbox.clone(),
            subagent_states: self.subagent_states.clone(),
            delegated_work_evidence: self.delegated_work_evidence.clone(),
            coordinator_decisions: self.coordinator_decisions.clone(),
            coordinator_runtime_proof: self.coordinator_runtime_proof.clone(),
            external_capability_decisions,
            profiles,
            guard_decisions,
            supervision_evidence,
            runtime_truth,
            step_policy,
            conservative_reasons: self.conservative_reasons.clone(),
        })
    }

    fn from_view(view: AutonomyStatusView) -> Self {
        let supervision_evidence = view.supervision_evidence.clone();
        let runtime_truth = view.runtime_truth.clone();
        let step_policy = view.step_policy.clone();
        let delegation_records = view.delegation_records.clone();
        let subordinate_inbox = view.subordinate_inbox.clone();
        let subagent_states = view.subagent_states.clone();
        let delegated_work_evidence = view.delegated_work_evidence.clone();
        let coordinator_decisions = view.coordinator_decisions.clone();
        let coordinator_runtime_proof = view.coordinator_runtime_proof.clone();
        let mut accumulator = Self {
            session_id: view.session_id,
            turn_index: view.turn_index,
            coordinator_agent_id: view.coordinator_agent_id,
            resume_provenance: view.resume_provenance,
            lifecycle_state: view.lifecycle_state,
            result_preview: view.result_preview,
            graph: Some(view.graph),
            topology: Some(view.topology),
            team_sizing: view.team_sizing,
            runtime_truth,
            supervision_evidence,
            step_policy,
            delegation_records,
            subordinate_inbox,
            subagent_states,
            delegated_work_evidence,
            coordinator_decisions,
            coordinator_runtime_proof,
            ..Self::default()
        };

        for branch in view.branches {
            accumulator.branches.insert(branch.branch_id, branch);
        }
        for checkpoint in view.checkpoint_lineage {
            accumulator
                .checkpoint_lineage
                .insert(checkpoint.checkpoint_id, checkpoint);
        }
        for pressure in view.memory_pressure {
            accumulator
                .memory_pressure
                .insert(pressure.budget_id, pressure);
        }
        accumulator.routing_history = view.routing_history;
        accumulator.step_routing_history = view.step_routing_history;
        for record in view.interventions {
            accumulator.interventions.insert(record.record_id, record);
        }
        for capability in view.delegation_capabilities {
            accumulator
                .delegation_capabilities
                .insert(capability.capability_id, capability);
        }
        for alert in view.delegation_alerts {
            accumulator
                .delegation_alerts
                .insert(delegation_alert_key(&alert), alert);
        }
        accumulator.external_capability_decisions = view.external_capability_decisions;
        for profile in view.profiles {
            accumulator.profiles.insert(profile.profile_id, profile);
        }
        for decision in view.guard_decisions {
            accumulator
                .guard_decisions
                .insert(decision.decision_id, decision);
        }
        if let Some(evidence) = accumulator.supervision_evidence.clone() {
            if let Some(profile) = evidence.profile_snapshot.clone() {
                accumulator.latest_profile_id = Some(profile.profile_id);
                accumulator
                    .profiles
                    .entry(profile.profile_id)
                    .or_insert(profile);
            }
            if let Some(decision) = evidence.guard_decision.clone() {
                accumulator.latest_guard_decision_id = Some(decision.decision_id);
                accumulator
                    .guard_decisions
                    .entry(decision.decision_id)
                    .or_insert(decision);
            }
            if let Some(record) = evidence.intervention_record.clone() {
                accumulator.latest_intervention_id = Some(record.record_id);
                accumulator
                    .interventions
                    .entry(record.record_id)
                    .or_insert(record);
            }
            accumulator.latest_profile_id = evidence
                .profile_snapshot
                .as_ref()
                .map(|profile| profile.profile_id);
            accumulator.latest_guard_decision_id = evidence
                .guard_decision
                .as_ref()
                .map(|decision| decision.decision_id);
            accumulator.latest_intervention_id = evidence
                .intervention_record
                .as_ref()
                .map(|record| record.record_id);
        }
        accumulator.push_conservative_reasons(view.conservative_reasons);

        accumulator
    }

    fn synthesize_supervision_evidence(&self) -> Option<SupervisionEvidenceView> {
        let intervention_record = self
            .latest_intervention_id
            .and_then(|record_id| self.interventions.get(&record_id))
            .cloned()
            .or_else(|| {
                self.interventions
                    .values()
                    .max_by_key(|record| record.emitted_at)
                    .cloned()
            });
        let guard_decision = intervention_record
            .as_ref()
            .and_then(|record| self.guard_decisions.get(&record.decision_id))
            .cloned()
            .or_else(|| {
                self.latest_guard_decision_id
                    .and_then(|decision_id| self.guard_decisions.get(&decision_id))
                    .cloned()
            })
            .or_else(|| self.guard_decisions.values().next().cloned());
        let profile_snapshot = guard_decision
            .as_ref()
            .and_then(|decision| decision.evidence.profile_id)
            .and_then(|profile_id| self.profiles.get(&profile_id))
            .cloned()
            .or_else(|| {
                self.latest_profile_id
                    .and_then(|profile_id| self.profiles.get(&profile_id))
                    .cloned()
            })
            .or_else(|| {
                self.profiles
                    .values()
                    .max_by_key(|profile| profile.updated_at)
                    .cloned()
            });

        let target_scope = guard_decision
            .as_ref()
            .map(|decision| {
                let branch_hint = self
                    .guard_decision_branch_ids
                    .get(&decision.decision_id)
                    .copied()
                    .flatten();
                supervision_target_scope_from_guard_target(
                    self.graph.as_ref().map(|graph| graph.graph_id),
                    &decision.target_scope,
                    branch_hint,
                )
            })
            .or_else(|| {
                profile_snapshot
                    .as_ref()
                    .map(supervision_target_scope_from_profile)
            })?;

        let fingerprint_ref = profile_snapshot
            .as_ref()
            .and_then(|profile| profile.fingerprint_ref.clone());
        let decision_basis = guard_decision
            .as_ref()
            .map(|decision| decision.evidence.decision_basis.as_str().to_string());
        let repair_lineage_ref = repair_lineage_ref_from_checkpoint_lineage(
            self.checkpoint_lineage.values(),
            &target_scope,
        )
        .or_else(|| {
            guard_decision.as_ref().and_then(|decision| {
                decision
                    .evidence
                    .checkpoint_ids
                    .first()
                    .map(|checkpoint_id| RepairLineageRef {
                        source: "packet-020".to_string(),
                        checkpoint_ref: Some(checkpoint_id.to_string()),
                    })
            })
        });

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

    fn update_delegation(&mut self, capability: CapabilitySummary) {
        self.delegation_capabilities
            .insert(capability.capability_id, capability.clone());
        let key = delegation_capability_key(&capability);
        if let Some(alert) = capability.to_alert() {
            self.delegation_alerts.insert(key, alert);
        } else {
            self.delegation_alerts.remove(&key);
        }
    }

    fn update_external_capability_decision(&mut self, decision: ExternalCapabilityDecisionSummary) {
        if !self.external_capability_decisions.contains(&decision) {
            self.external_capability_decisions.push(decision);
        }
    }

    fn push_conservative_reasons<I>(&mut self, reasons: I)
    where
        I: IntoIterator<Item = String>,
    {
        for reason in reasons {
            if !self.conservative_reasons.contains(&reason) {
                self.conservative_reasons.push(reason);
            }
        }
    }
}

fn merge_supervision_evidence(
    mut synthesized: SupervisionEvidenceView,
    preserved: &SupervisionEvidenceView,
) -> SupervisionEvidenceView {
    synthesized.target_scope =
        merge_supervision_target_scope(synthesized.target_scope, &preserved.target_scope);
    if synthesized.fingerprint_ref.is_none() {
        synthesized.fingerprint_ref = preserved.fingerprint_ref.clone();
    }
    if synthesized.profile_snapshot.is_none() {
        synthesized.profile_snapshot = preserved.profile_snapshot.clone();
    }
    if synthesized.guard_decision.is_none() {
        synthesized.guard_decision = preserved.guard_decision.clone();
    }
    if synthesized.intervention_record.is_none() {
        synthesized.intervention_record = preserved.intervention_record.clone();
    }
    if synthesized.decision_basis.is_none() {
        synthesized.decision_basis = preserved.decision_basis.clone();
    }
    if synthesized.repair_lineage_ref.is_none() {
        synthesized.repair_lineage_ref = preserved.repair_lineage_ref.clone();
    }
    if synthesized.proof_boundary.is_none()
        || (synthesized.proof_boundary.as_deref() == Some("supported task path")
            && preserved.proof_boundary.is_some())
    {
        synthesized.proof_boundary = preserved.proof_boundary.clone();
    }
    synthesized
}

fn synthesize_runtime_truth(
    graph: &ExecutionGraphSummary,
    topology: &TopologyPlanSummary,
    branches: &[BranchSummary],
    step_routing_history: &[StepRoutingDecisionSummary],
    supervision_evidence: Option<&SupervisionEvidenceView>,
    guard_decisions: &[GuardDecision],
) -> RuntimeTruthView {
    let branch_id = supervision_evidence
        .and_then(|evidence| evidence.target_scope.branch_id)
        .or_else(|| branches.first().map(|branch| branch.branch_id));
    let node_id = supervision_evidence.and_then(|evidence| evidence.target_scope.node_id);
    let mut relationships = vec![
        RunTraceRelationshipKind::Graph,
        RunTraceRelationshipKind::ToolBoundary,
    ];

    if graph.branch_count > 0 {
        relationships.push(RunTraceRelationshipKind::Branch);
    }
    if node_id.is_some() || graph.node_count > 0 {
        relationships.push(RunTraceRelationshipKind::Node);
    }
    if graph.branch_count > 1 {
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
    if step_routing_history
        .iter()
        .any(|entry| matches!(entry.action.as_str(), "retry" | "fallback"))
    {
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

fn repair_lineage_ref_from_checkpoint_lineage<'a>(
    checkpoint_lineage: impl IntoIterator<Item = &'a CheckpointRecordSummary>,
    target_scope: &SupervisionTargetScope,
) -> Option<RepairLineageRef> {
    let checkpoint_ref = checkpoint_lineage
        .into_iter()
        .filter(|checkpoint| checkpoint_matches_target(checkpoint.branch_id, target_scope))
        .max_by_key(|checkpoint| checkpoint.captured_at)
        .map(|checkpoint| checkpoint.checkpoint_id.to_string())?;

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

fn merge_supervision_target_scope(
    mut synthesized: SupervisionTargetScope,
    preserved: &SupervisionTargetScope,
) -> SupervisionTargetScope {
    if synthesized.kind == preserved.kind {
        if synthesized.provider.is_none() {
            synthesized.provider = preserved.provider.clone();
        }
        if synthesized.graph_id.is_none() {
            synthesized.graph_id = preserved.graph_id;
        }
        if synthesized.branch_id.is_none() {
            synthesized.branch_id = preserved.branch_id;
        }
        if synthesized.node_id.is_none() {
            synthesized.node_id = preserved.node_id;
        }
    }
    synthesized
}

fn supervision_target_scope_from_guard_target(
    graph_id: Option<ExecutionGraphId>,
    target: &GuardTarget,
    branch_hint: Option<ExecutionBranchId>,
) -> SupervisionTargetScope {
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
            branch_id: None,
            node_id: None,
        },
        GuardTarget::Branch(branch_id) => SupervisionTargetScope {
            kind: SupervisionTargetKind::Branch,
            provider: None,
            graph_id,
            branch_id: Some(*branch_id),
            node_id: None,
        },
        GuardTarget::Node(node_id) => SupervisionTargetScope {
            kind: SupervisionTargetKind::Node,
            provider: None,
            graph_id,
            branch_id: branch_hint,
            node_id: Some(*node_id),
        },
    }
}

fn supervision_target_scope_from_profile(profile: &ProfileSnapshot) -> SupervisionTargetScope {
    match profile.target {
        ProfileTarget::Provider => SupervisionTargetScope {
            kind: SupervisionTargetKind::Provider,
            provider: None,
            graph_id: None,
            branch_id: None,
            node_id: None,
        },
        ProfileTarget::Topology => SupervisionTargetScope {
            kind: SupervisionTargetKind::Graph,
            provider: None,
            graph_id: None,
            branch_id: None,
            node_id: None,
        },
        ProfileTarget::Branch => SupervisionTargetScope {
            kind: SupervisionTargetKind::Branch,
            provider: None,
            graph_id: None,
            branch_id: None,
            node_id: None,
        },
        ProfileTarget::Agent => SupervisionTargetScope {
            kind: SupervisionTargetKind::Node,
            provider: None,
            graph_id: None,
            branch_id: None,
            node_id: None,
        },
    }
}

fn infer_team_sizing_from_projection(
    graph: &ExecutionGraphSummary,
    topology: &TopologyPlanSummary,
    branches: &[BranchSummary],
    routing_history: &[RoutingDecisionSummary],
    conservative_reasons: &[String],
) -> Option<TeamSizingDecision> {
    let desired_workers = topology.parallelism_width.max(1);
    let routed_agents = routing_history
        .iter()
        .map(|decision| decision.selected_agent)
        .collect::<HashSet<_>>();
    let observed_agents = if routed_agents.is_empty() {
        branches
            .iter()
            .flat_map(|branch| branch.assigned_agents.iter().cloned())
            .collect::<HashSet<_>>()
    } else {
        routed_agents
    };

    let observed_workers = observed_agents.len();
    if observed_workers == 0 {
        return None;
    }

    let selected_workers = observed_workers.min(desired_workers);
    let budget_pressure = routing_history
        .iter()
        .map(|decision| decision.budget_pressure)
        .max();
    let dependency_depth = routing_history
        .iter()
        .map(|decision| decision.dependency_depth)
        .max()
        .unwrap_or(topology.task_shape.max_depth);
    let cap_reason = (selected_workers < desired_workers).then(|| {
        conservative_reasons
            .first()
            .cloned()
            .or_else(|| topology.rationale.fallback_reason.clone())
            .unwrap_or_else(|| {
                format!(
                    "event projection observed {selected_workers} active worker(s) for desired width {desired_workers}"
                )
            })
    });

    let mut rationale_lines = vec![
        format!(
            "event projection derived desired width {desired_workers} from topology {:?} and task shape {}",
            topology.topology_kind,
            topology.task_shape.kind.as_str()
        ),
        format!(
            "event projection observed {observed_workers} active worker(s) across {} routed branch(es)",
            routing_history.len().max(1)
        ),
        format!("topology rationale: {}", topology.rationale.selected_for),
    ];
    if let Some(pressure) = budget_pressure {
        rationale_lines.push(format!("latest routed budget pressure {pressure}"));
    }
    if let Some(reason) = cap_reason.as_ref() {
        rationale_lines.push(reason.clone());
    }

    Some(TeamSizingDecision {
        workflow_id: graph.workflow_id,
        graph_id: graph.graph_id,
        decision_phase: "event_projection".to_string(),
        desired_workers,
        selected_workers,
        available_workers: observed_workers,
        branch_frontier_width: topology
            .task_shape
            .max_parallel_width
            .max(routing_history.len())
            .max(1),
        dependency_depth,
        conservative_mode: selected_workers < desired_workers || !conservative_reasons.is_empty(),
        budget_pressure,
        cap_reason,
        rationale_lines,
        decided_at: chrono::DateTime::<chrono::Utc>::from(SystemTime::UNIX_EPOCH),
    })
}

fn delegation_alert_key(alert: &DelegationAlert) -> String {
    format!(
        "{:?}:{}",
        alert.scope,
        alert
            .capability_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| alert.message.clone())
    )
}

fn delegation_capability_key(capability: &CapabilitySummary) -> String {
    format!("{:?}:{}", Some(capability.scope), capability.capability_id)
}

fn external_capability_decision_key(decision: &ExternalCapabilityDecisionSummary) -> String {
    format!(
        "{}:{}:{}:{}:{}:{}",
        decision
            .boundary_surface
            .map(|surface| format!("{surface:?}"))
            .unwrap_or_else(|| "none".to_string()),
        decision
            .branch_id
            .map(|branch_id| branch_id.to_string())
            .unwrap_or_else(|| "none".to_string()),
        decision
            .capability_id
            .map(|capability_id| capability_id.to_string())
            .unwrap_or_else(|| "none".to_string()),
        decision.action_id.as_deref().unwrap_or("none"),
        decision.action_descriptor_id.as_deref().unwrap_or("none"),
        match decision.outcome {
            crate::autonomy::ExternalCapabilityDecisionOutcome::Allowed => "allowed",
            crate::autonomy::ExternalCapabilityDecisionOutcome::Rejected => "rejected",
        }
    )
}

/// In-process event bus with handler dispatch, broadcast, and dead letter handling.
///
/// The event bus distributes events to registered handlers (with optional filtering)
/// and to broadcast subscribers. Events that fail all handler delivery are routed
/// to a dead letter queue for later inspection.
pub struct EventBus {
    handlers: RwLock<Vec<Arc<dyn EventHandler>>>,
    broadcast_tx: broadcast::Sender<Event>,
    event_store: Option<Arc<dyn EventStore>>,
    autonomy_state: RwLock<HashMap<TaskId, AutonomyStatusAccumulator>>,
    dead_letter: Arc<DeadLetterQueue>,
}

impl EventBus {
    /// Create a new event bus with the given broadcast channel capacity.
    pub fn new(broadcast_capacity: usize) -> Self {
        let (broadcast_tx, _) = broadcast::channel(broadcast_capacity);
        Self {
            handlers: RwLock::new(Vec::new()),
            broadcast_tx,
            event_store: None,
            autonomy_state: RwLock::new(HashMap::new()),
            dead_letter: Arc::new(DeadLetterQueue::default()),
        }
    }

    /// Attach an event store for persistence and replay.
    ///
    /// Consumes and returns `self` for builder-style chaining.
    pub fn with_event_store(mut self, store: Arc<dyn EventStore>) -> Self {
        self.event_store = Some(store);
        self
    }

    /// Publish an event to all matching handlers, broadcast subscribers, and the event store.
    pub async fn publish(&self, event: Event) -> Result<(), EventBusError> {
        // Persist to event store if configured.
        if let Some(ref store) = self.event_store {
            store.append(event.clone()).await.map_err(|e| {
                tracing::error!(event_id = %event.id, "Failed to persist event to store: {e}");
                e
            })?;
        }

        self.update_autonomy_projection(&event).await?;

        // Broadcast to subscribers (ignore send errors — no receivers is not an error).
        let _ = self.broadcast_tx.send(event.clone());

        // Dispatch to handlers.
        self.process_event(event).await;

        Ok(())
    }

    /// Register a handler to receive events.
    pub async fn subscribe(&self, handler: Arc<dyn EventHandler>) {
        self.handlers.write().await.push(handler);
    }

    /// Subscribe to the broadcast channel for all events.
    ///
    /// The returned receiver will see every published event regardless of handler filters.
    pub fn subscribe_broadcast(&self) -> broadcast::Receiver<Event> {
        self.broadcast_tx.subscribe()
    }

    /// Return the latest assembled autonomy status for a workflow, when available.
    pub async fn autonomy_status(&self, workflow_id: &TaskId) -> Option<AutonomyStatusView> {
        self.autonomy_state
            .read()
            .await
            .get(workflow_id)
            .and_then(AutonomyStatusAccumulator::view)
    }

    /// List workflow IDs that currently have an assembled autonomy projection.
    pub async fn autonomy_workflows(&self) -> Vec<TaskId> {
        let mut workflows = self
            .autonomy_state
            .read()
            .await
            .iter()
            .filter_map(|(workflow_id, state)| state.view().map(|_| *workflow_id))
            .collect::<Vec<_>>();
        workflows.sort_by_key(|workflow_id| workflow_id.to_string());
        workflows
    }

    /// Replay events from the event store within a time range, optionally filtered.
    ///
    /// Returns an error if no event store is configured.
    pub async fn replay_events(
        &self,
        from: SystemTime,
        to: SystemTime,
        filter: Option<EventFilter>,
    ) -> Result<Vec<Event>, EventBusError> {
        let store = self.event_store.as_ref().ok_or_else(|| {
            EventBusError::StoreFailed("no event store configured for replay".into())
        })?;

        let events = store.query(from, to).await?;

        match filter {
            Some(f) => Ok(events.into_iter().filter(|e| f.matches(e)).collect()),
            None => Ok(events),
        }
    }

    /// Returns a reference to the dead letter queue.
    pub fn dead_letter_queue(&self) -> &DeadLetterQueue {
        &self.dead_letter
    }

    /// Dispatch an event to all registered handlers, applying filters and
    /// routing failures to the dead letter queue.
    async fn process_event(&self, event: Event) {
        let handlers = {
            let handlers = self.handlers.read().await;

            if handlers.is_empty() {
                return;
            }

            handlers.iter().cloned().collect::<Vec<_>>()
        };

        let mut any_handled = false;
        let mut all_failed = true;

        for handler in handlers {
            // Apply handler filter.
            if let Some(filter) = handler.event_filter() {
                if !filter.matches(&event) {
                    continue;
                }
            }

            any_handled = true;

            match handler.handle_event(event.clone()).await {
                Ok(()) => {
                    all_failed = false;
                }
                Err(e) => {
                    tracing::warn!(
                        event_id = %event.id,
                        event_type = %event.event_type,
                        "Event handler failed: {e}"
                    );
                }
            }
        }

        // If at least one handler matched but all of them failed, dead-letter the event.
        if any_handled && all_failed {
            tracing::error!(
                event_id = %event.id,
                "All matching handlers failed; routing to dead letter queue"
            );
            self.dead_letter.enqueue(event);
        }
    }

    async fn update_autonomy_projection(&self, event: &Event) -> Result<(), EventBusError> {
        let Some(autonomy_event) = event.autonomy_event().map_err(|error| {
            EventBusError::HandlerFailed(format!(
                "failed to decode autonomy event payload: {error}"
            ))
        })?
        else {
            return Ok(());
        };

        let workflow_id = autonomy_event.workflow_id();
        let mut autonomy_state = self.autonomy_state.write().await;
        autonomy_state
            .entry(workflow_id)
            .or_default()
            .apply(autonomy_event);
        Ok(())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(DEFAULT_BROADCAST_CAPACITY)
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("dead_letter", &self.dead_letter)
            .field("has_event_store", &self.event_store.is_some())
            .finish_non_exhaustive()
    }
}

/// Implement the core [`EventPublisher`] trait so the event bus can be used
/// through the core crate's trait object interface.
///
/// Converts the minimal [`SystemEvent`] from core into the richer [`Event`] type.
#[async_trait]
impl EventPublisher for EventBus {
    async fn publish(
        &self,
        system_event: SystemEvent,
    ) -> Result<(), mister_smith_core::EventError> {
        let event = Event::new(
            "system",
            crate::types::EventType::Custom(system_event.event_type),
        );
        // Construct a full Event with the system event's payload.
        let event = Event {
            payload: system_event.payload,
            ..event
        };

        // Delegate to EventBus::publish, converting the error.
        EventBus::publish(self, event)
            .await
            .map_err(|e| -> mister_smith_core::EventError { e.into() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::InMemoryEventStore;
    use crate::types::{AgentEventType, EventType, SystemEventType};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    /// Test handler that counts invocations.
    struct CountingHandler {
        count: AtomicUsize,
        filter: Option<EventFilter>,
    }

    impl CountingHandler {
        fn new() -> Self {
            Self {
                count: AtomicUsize::new(0),
                filter: None,
            }
        }

        fn with_filter(filter: EventFilter) -> Self {
            Self {
                count: AtomicUsize::new(0),
                filter: Some(filter),
            }
        }

        fn count(&self) -> usize {
            self.count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl EventHandler for CountingHandler {
        async fn handle_event(&self, _event: Event) -> Result<(), EventBusError> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn event_filter(&self) -> Option<EventFilter> {
            self.filter.clone()
        }
    }

    struct ConcurrentSubscribeHandler {
        bus: Arc<EventBus>,
    }

    #[async_trait]
    impl EventHandler for ConcurrentSubscribeHandler {
        async fn handle_event(&self, _event: Event) -> Result<(), EventBusError> {
            self.bus.subscribe(Arc::new(CountingHandler::new())).await;
            Ok(())
        }
    }

    /// Handler that always fails.
    struct FailingHandler;

    #[async_trait]
    impl EventHandler for FailingHandler {
        async fn handle_event(&self, _event: Event) -> Result<(), EventBusError> {
            Err(EventBusError::HandlerFailed("intentional failure".into()))
        }
    }

    #[tokio::test]
    async fn publish_delivers_to_handler() {
        let bus = EventBus::default();
        let handler = Arc::new(CountingHandler::new());
        bus.subscribe(handler.clone()).await;

        let event = Event::new("test", EventType::System(SystemEventType::Started));
        bus.publish(event).await.unwrap();

        assert_eq!(handler.count(), 1);
    }

    #[tokio::test]
    async fn handler_filter_is_applied() {
        let bus = EventBus::default();

        let filter = EventFilter {
            event_types: Some(vec![EventType::System(SystemEventType::Started)]),
            ..Default::default()
        };
        let handler = Arc::new(CountingHandler::with_filter(filter));
        bus.subscribe(handler.clone()).await;

        // Matching event.
        let matching = Event::new("test", EventType::System(SystemEventType::Started));
        bus.publish(matching).await.unwrap();
        assert_eq!(handler.count(), 1);

        // Non-matching event.
        let non_matching = Event::new("test", EventType::System(SystemEventType::Stopped));
        bus.publish(non_matching).await.unwrap();
        assert_eq!(handler.count(), 1); // unchanged
    }

    #[tokio::test]
    async fn broadcast_delivers_all_events() {
        let bus = EventBus::default();
        let mut rx = bus.subscribe_broadcast();

        let event = Event::new("test", EventType::Agent(AgentEventType::Created));
        let event_id = event.id;
        bus.publish(event).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, event_id);
    }

    #[tokio::test]
    async fn handler_can_subscribe_during_dispatch_without_stall() {
        let bus = Arc::new(EventBus::default());
        bus.subscribe(Arc::new(ConcurrentSubscribeHandler {
            bus: Arc::clone(&bus),
        }))
        .await;

        let event = Event::new("test", EventType::Custom("concurrent-subscribe".into()));

        let publish_result =
            tokio::time::timeout(Duration::from_millis(250), bus.publish(event)).await;
        assert!(
            publish_result.is_ok(),
            "publish timed out due to lock contention"
        );
        assert!(publish_result.unwrap().is_ok());
    }

    #[tokio::test]
    async fn failing_handler_routes_to_dead_letter() {
        let bus = EventBus::default();
        bus.subscribe(Arc::new(FailingHandler)).await;

        let event = Event::new("test", EventType::Custom("fail".into()));
        bus.publish(event).await.unwrap();

        assert_eq!(bus.dead_letter_queue().len(), 1);
    }

    #[tokio::test]
    async fn partial_handler_failure_does_not_dead_letter() {
        let bus = EventBus::default();
        let success_handler = Arc::new(CountingHandler::new());
        bus.subscribe(success_handler.clone()).await;
        bus.subscribe(Arc::new(FailingHandler)).await;

        let event = Event::new("test", EventType::Custom("mixed".into()));
        bus.publish(event).await.unwrap();

        // One handler succeeded, so event should NOT be dead-lettered.
        assert_eq!(bus.dead_letter_queue().len(), 0);
        assert_eq!(success_handler.count(), 1);
    }

    #[tokio::test]
    async fn event_store_persists_events() {
        let store = Arc::new(InMemoryEventStore::new());
        let bus = EventBus::default().with_event_store(store.clone());

        let event = Event::new(
            "test",
            EventType::System(SystemEventType::ConfigurationChanged),
        );
        let event_id = event.id;
        bus.publish(event).await.unwrap();

        let found = store.get_by_id(event_id).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn replay_events_with_filter() {
        let store = Arc::new(InMemoryEventStore::new());
        let bus = EventBus::default().with_event_store(store);

        let before = SystemTime::now();
        tokio::time::sleep(Duration::from_millis(5)).await;

        bus.publish(Event::new(
            "agent-1",
            EventType::System(SystemEventType::Started),
        ))
        .await
        .unwrap();
        bus.publish(Event::new(
            "agent-2",
            EventType::Agent(AgentEventType::Created),
        ))
        .await
        .unwrap();

        tokio::time::sleep(Duration::from_millis(5)).await;
        let after = SystemTime::now();

        // No filter — get all.
        let all = bus.replay_events(before, after, None).await.unwrap();
        assert_eq!(all.len(), 2);

        // Filter by source.
        let filter = EventFilter {
            sources: Some(vec!["agent-1".into()]),
            ..Default::default()
        };
        let filtered = bus
            .replay_events(before, after, Some(filter))
            .await
            .unwrap();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source, "agent-1");
    }

    #[tokio::test]
    async fn replay_without_store_returns_error() {
        let bus = EventBus::default();
        let result = bus
            .replay_events(SystemTime::now(), SystemTime::now(), None)
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn publish_no_handlers_is_ok() {
        let bus = EventBus::default();
        let event = Event::new("test", EventType::Custom("orphan".into()));
        // Should not error even with no handlers.
        bus.publish(event).await.unwrap();
        assert_eq!(bus.dead_letter_queue().len(), 0);
    }

    #[tokio::test]
    async fn event_publisher_trait_impl() {
        let bus = EventBus::default();
        let handler = Arc::new(CountingHandler::new());
        bus.subscribe(handler.clone()).await;

        // Use the core trait interface.
        let publisher: &dyn EventPublisher = &bus;
        let system_event = SystemEvent {
            event_type: "test.event".into(),
            payload: serde_json::json!({"key": "value"}),
        };
        publisher.publish(system_event).await.unwrap();

        assert_eq!(handler.count(), 1);
    }

    #[test]
    fn merge_supervision_evidence_keeps_synthesized_supported_task_boundary_when_preserved_absent()
    {
        let synthesized = SupervisionEvidenceView {
            target_scope: SupervisionTargetScope {
                kind: SupervisionTargetKind::Graph,
                provider: None,
                graph_id: None,
                branch_id: None,
                node_id: None,
            },
            fingerprint_ref: None,
            profile_snapshot: None,
            guard_decision: None,
            intervention_record: None,
            decision_basis: None,
            repair_lineage_ref: None,
            proof_boundary: Some("supported task path".to_string()),
        };
        let preserved = SupervisionEvidenceView {
            target_scope: synthesized.target_scope.clone(),
            fingerprint_ref: None,
            profile_snapshot: None,
            guard_decision: None,
            intervention_record: None,
            decision_basis: None,
            repair_lineage_ref: None,
            proof_boundary: None,
        };

        let merged = merge_supervision_evidence(synthesized, &preserved);

        assert_eq!(
            merged.proof_boundary.as_deref(),
            Some("supported task path")
        );
    }

    #[test]
    fn merge_supervision_evidence_prefers_preserved_boundary_over_supported_task_path() {
        let synthesized = SupervisionEvidenceView {
            target_scope: SupervisionTargetScope {
                kind: SupervisionTargetKind::Graph,
                provider: None,
                graph_id: None,
                branch_id: None,
                node_id: None,
            },
            fingerprint_ref: None,
            profile_snapshot: None,
            guard_decision: None,
            intervention_record: None,
            decision_basis: None,
            repair_lineage_ref: None,
            proof_boundary: Some("supported task path".to_string()),
        };
        let preserved = SupervisionEvidenceView {
            target_scope: synthesized.target_scope.clone(),
            fingerprint_ref: None,
            profile_snapshot: None,
            guard_decision: None,
            intervention_record: None,
            decision_basis: None,
            repair_lineage_ref: None,
            proof_boundary: Some("explicit snapshot boundary".to_string()),
        };

        let merged = merge_supervision_evidence(synthesized, &preserved);

        assert_eq!(
            merged.proof_boundary.as_deref(),
            Some("explicit snapshot boundary")
        );
    }
}
