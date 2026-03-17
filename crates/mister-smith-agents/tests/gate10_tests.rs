use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;
use mister_smith_agents::config::TaskState;
use mister_smith_agents::orchestrator::Orchestrator;
use mister_smith_agents::scheduler::{
    ArrayAggregator, IdentityDecomposer, TaskAssignment, TaskScheduler,
};
use mister_smith_agents::{
    BranchCheckpoint, BranchCheckpointStore, BranchResumeMetadata, GuardContext, ProfileAssessment,
    TopologyCompiler, TopologySignals,
};
use mister_smith_core::{
    AgentId, AuthorityPrincipal, BranchState, CheckpointId, DelegationScope, ExecutionBranchId,
    FailureClass, GuardTarget, HealthState, InterventionType, MemorySnapshotId, NodeState,
    PersistenceError, ProfileSnapshot, ProfileSnapshotId, ProfileTarget, ProvenanceChain,
    ProvenanceLink, RevocationState, SemanticSignal, SemanticSignalKind, TaskId,
};
use mister_smith_events::{AutonomyEvent, CapabilitySummary, EventBus};
use serde_json::json;
use tokio::time::{timeout, Duration};
use uuid::Uuid;

fn mixed_dependency_plan() -> serde_json::Value {
    json!({
        "goal": "gate10-mixed-dependencies",
        "steps": [
            {
                "id": "root",
                "step": 1,
                "action": "root",
                "description": "root"
            },
            {
                "id": "left",
                "step": 2,
                "action": "left",
                "description": "left",
                "depends_on": ["root"],
                "branch": "left"
            },
            {
                "id": "right-1",
                "step": 3,
                "action": "right-1",
                "description": "right-1",
                "depends_on": ["root"],
                "branch": "right"
            },
            {
                "id": "right-2",
                "step": 4,
                "action": "right-2",
                "description": "right-2",
                "depends_on": ["right-1"],
                "branch": "right"
            },
            {
                "id": "join",
                "step": 5,
                "action": "join",
                "description": "join",
                "depends_on": ["left", "right-2"],
                "branch": "join"
            }
        ]
    })
}

fn cyclic_plan() -> serde_json::Value {
    json!({
        "goal": "gate10-invalid-graph",
        "steps": [
            {
                "id": "first",
                "step": 1,
                "action": "first",
                "description": "first",
                "depends_on": ["second"]
            },
            {
                "id": "second",
                "step": 2,
                "action": "second",
                "description": "second",
                "depends_on": ["first"]
            }
        ]
    })
}

fn frontier_rebalance_plan() -> serde_json::Value {
    json!({
        "goal": "gate10-frontier-rebalance",
        "steps": [
            {
                "id": "root",
                "step": 1,
                "action": "root",
                "description": "root"
            },
            {
                "id": "left",
                "step": 2,
                "action": "left",
                "description": "left",
                "depends_on": ["root"],
                "branch": "left"
            },
            {
                "id": "right",
                "step": 3,
                "action": "right",
                "description": "right",
                "depends_on": ["root"],
                "branch": "right"
            },
            {
                "id": "join",
                "step": 4,
                "action": "join",
                "description": "join",
                "depends_on": ["left", "right"],
                "branch": "join"
            }
        ]
    })
}

fn branch_id_for(
    graph: &mister_smith_agents::ExecutionGraph,
    step_key: &str,
) -> mister_smith_core::ExecutionBranchId {
    graph
        .nodes
        .iter()
        .find(|node| node.step_key == step_key)
        .map(|node| node.branch_id)
        .expect("expected branch to exist")
}

fn node_id_for(
    graph: &mister_smith_agents::ExecutionGraph,
    step_key: &str,
) -> mister_smith_core::ExecutionNodeId {
    graph
        .nodes
        .iter()
        .find(|node| node.step_key == step_key)
        .map(|node| node.node_id)
        .expect("expected node to exist")
}

fn submit_task(
    scheduler: &TaskScheduler,
    task_id: TaskId,
    task_type: &str,
    parent_task_id: TaskId,
    state: TaskState,
) {
    scheduler.submit(TaskAssignment {
        task_id,
        task_type: task_type.to_string(),
        priority: 128,
        deadline: None,
        input: json!({ "from": "gate10" }),
        output: None,
        state,
        assigned_to: None,
        parent_task_id: Some(parent_task_id),
        team_id: None,
        message_id: Uuid::new_v4(),
        created_at: Utc::now(),
        assigned_at: None,
        completed_at: (state == TaskState::Completed).then_some(Utc::now()),
        error_message: None,
    });
}

struct MixedDependencyFixture {
    scheduler: Arc<TaskScheduler>,
    orchestrator: Orchestrator,
    event_bus: Arc<EventBus>,
    workflow_id: TaskId,
    right_branch: ExecutionBranchId,
    checkpoint_id: CheckpointId,
}

fn checkpointed_fixture() -> MixedDependencyFixture {
    let scheduler = Arc::new(TaskScheduler::new());
    let event_bus = Arc::new(EventBus::default());
    let orchestrator = Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    )
    .with_event_bus(event_bus.clone());
    let mut graph = TopologyCompiler::default()
        .compile(
            TaskId::new(),
            &mixed_dependency_plan(),
            &TopologySignals::default(),
        )
        .expect("mixed dependency graph should compile");
    let workflow_id = graph.workflow_id;
    let root = node_id_for(&graph, "root");
    let left = node_id_for(&graph, "left");
    let right_1 = node_id_for(&graph, "right-1");
    let right_2 = node_id_for(&graph, "right-2");
    let join = node_id_for(&graph, "join");
    let root_branch = branch_id_for(&graph, "root");
    let left_branch = branch_id_for(&graph, "left");
    let right_branch = branch_id_for(&graph, "right-1");

    graph.nodes.iter_mut().for_each(|node| {
        if node.node_id == root || node.node_id == left || node.node_id == right_1 {
            node.state = NodeState::Completed;
        }
    });
    graph.branch_mut(&root_branch).unwrap().state = BranchState::Completed;
    graph.branch_mut(&left_branch).unwrap().state = BranchState::Completed;
    graph.branch_mut(&right_branch).unwrap().state = BranchState::Checkpointed;
    let checkpoint = BranchCheckpoint::new(
        right_branch,
        vec![right_1],
        vec![right_2],
        MemorySnapshotId::new(),
    );
    let checkpoint_id = checkpoint.checkpoint_id;
    graph.checkpoint_lineage.push(checkpoint);
    orchestrator.register_execution_graph(graph);

    submit_task(
        &scheduler,
        TaskId::from_uuid(*root.as_ref()),
        "root",
        workflow_id,
        TaskState::Completed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*left.as_ref()),
        "left",
        workflow_id,
        TaskState::Completed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*right_1.as_ref()),
        "right-1",
        workflow_id,
        TaskState::Completed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*right_2.as_ref()),
        "right-2",
        workflow_id,
        TaskState::Failed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*join.as_ref()),
        "join",
        workflow_id,
        TaskState::Pending,
    );

    MixedDependencyFixture {
        scheduler,
        orchestrator,
        event_bus,
        workflow_id,
        right_branch,
        checkpoint_id,
    }
}

#[derive(Default)]
struct InMemoryBranchCheckpointStore {
    checkpoints: Mutex<HashMap<(TaskId, ExecutionBranchId), Vec<BranchCheckpoint>>>,
    resumes: Mutex<HashMap<(TaskId, ExecutionBranchId), Vec<BranchResumeMetadata>>>,
}

impl InMemoryBranchCheckpointStore {
    fn latest_resume(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Option<BranchResumeMetadata> {
        self.resumes
            .lock()
            .unwrap()
            .get(&(workflow_id, branch_id))
            .and_then(|entries| entries.last().cloned())
    }
}

#[async_trait]
impl BranchCheckpointStore for InMemoryBranchCheckpointStore {
    async fn persist_branch_checkpoint(
        &self,
        workflow_id: TaskId,
        checkpoint: &BranchCheckpoint,
    ) -> Result<(), PersistenceError> {
        self.checkpoints
            .lock()
            .unwrap()
            .entry((workflow_id, checkpoint.branch_id))
            .or_default()
            .push(checkpoint.clone());
        Ok(())
    }

    async fn persist_branch_resume(
        &self,
        resume: &BranchResumeMetadata,
    ) -> Result<(), PersistenceError> {
        self.resumes
            .lock()
            .unwrap()
            .entry((resume.workflow_id, resume.branch_id))
            .or_default()
            .push(resume.clone());
        Ok(())
    }

    async fn latest_branch_checkpoint(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Result<Option<BranchCheckpoint>, PersistenceError> {
        Ok(self
            .checkpoints
            .lock()
            .unwrap()
            .get(&(workflow_id, branch_id))
            .and_then(|entries| entries.last().cloned()))
    }

    async fn branch_resume_history(
        &self,
        workflow_id: TaskId,
        branch_id: ExecutionBranchId,
    ) -> Result<Vec<BranchResumeMetadata>, PersistenceError> {
        Ok(self
            .resumes
            .lock()
            .unwrap()
            .get(&(workflow_id, branch_id))
            .cloned()
            .unwrap_or_default())
    }
}

fn delegated_fixture(required_scope: DelegationScope) -> MixedDependencyFixture {
    let fixture = checkpointed_fixture();
    let mut graph = fixture
        .orchestrator
        .execution_graph(&fixture.workflow_id)
        .expect("execution graph should exist");
    for node in graph
        .nodes
        .iter_mut()
        .filter(|node| node.branch_id == fixture.right_branch)
    {
        node.delegation_requirement = Some(required_scope);
    }
    fixture.orchestrator.register_execution_graph(graph);
    fixture
}

fn capability_summary(
    scope: DelegationScope,
    revocation_state: RevocationState,
    rejection_reason: Option<&str>,
) -> CapabilitySummary {
    let capability_id = mister_smith_core::CapabilityId::new();
    let recipient = AgentId::new();
    let expires_at = Utc::now() + chrono::Duration::minutes(30);
    let issuer = AuthorityPrincipal::Policy("operator".to_string());
    CapabilitySummary {
        capability_id,
        issuer: issuer.clone(),
        recipient,
        scope,
        parent_capability: None,
        expires_at,
        provenance: ProvenanceChain {
            root_issuer: issuer.clone(),
            terminal_capability: capability_id,
            links: vec![ProvenanceLink {
                issuer,
                recipient,
                capability_id,
                scope,
                expires_at,
            }],
        },
        revocation_state,
        rejection_reason: rejection_reason.map(str::to_string),
    }
}

fn assessed_profile(
    branch_id: ExecutionBranchId,
    health_state: HealthState,
    semantic_signals: Vec<SemanticSignal>,
    notes: Vec<&str>,
) -> ProfileAssessment {
    ProfileAssessment::new(
        Some(ProfileSnapshot {
            profile_id: ProfileSnapshotId::new(),
            target: ProfileTarget::Branch,
            health_state,
            latency_window: None,
            error_window: None,
            semantic_signals,
            updated_at: Utc::now(),
        }),
        notes.into_iter().map(str::to_string).collect(),
    )
    .with_target(GuardTarget::Branch(branch_id))
}

async fn wait_for_event_bus_status(
    event_bus: &EventBus,
    workflow_id: &TaskId,
    expected_checkpoints: usize,
    expected_routing_history: usize,
) -> mister_smith_events::AutonomyStatusView {
    timeout(Duration::from_millis(500), async {
        loop {
            if let Some(status) = event_bus.autonomy_status(workflow_id).await {
                if status.checkpoint_lineage.len() == expected_checkpoints
                    && status.routing_history.len() == expected_routing_history
                {
                    return status;
                }
            }

            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("shared event bus autonomy status should appear")
}

async fn wait_for_delegation_status(
    event_bus: &EventBus,
    workflow_id: &TaskId,
    expected_capabilities: usize,
    expected_alerts: usize,
) -> mister_smith_events::AutonomyStatusView {
    timeout(Duration::from_millis(500), async {
        loop {
            if let Some(status) = event_bus.autonomy_status(workflow_id).await {
                if status.delegation_capabilities.len() == expected_capabilities
                    && status.delegation_alerts.len() == expected_alerts
                {
                    return status;
                }
            }

            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("delegation status should become visible on the shared event bus")
}

#[tokio::test]
async fn gate10_mixed_dependency_resume_preserves_completed_branches() {
    let fixture = checkpointed_fixture();
    let graph = fixture
        .orchestrator
        .execution_graph(&fixture.workflow_id)
        .expect("execution graph should be registered");
    let left = node_id_for(&graph, "left");
    let right_1 = node_id_for(&graph, "right-1");
    let right_2 = node_id_for(&graph, "right-2");
    let join = node_id_for(&graph, "join");

    let worker = AgentId::new();
    let decisions = fixture
        .orchestrator
        .route_ready_branches(&fixture.workflow_id, &[worker])
        .expect("Gate 10 resume should route only the failed branch scope");

    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].branch_id, fixture.right_branch);
    assert_eq!(
        decisions[0].task_ids,
        vec![TaskId::from_uuid(*right_2.as_ref())]
    );
    assert_eq!(
        fixture
            .scheduler
            .get(&TaskId::from_uuid(*left.as_ref()))
            .unwrap()
            .state,
        TaskState::Completed
    );
    assert_eq!(
        fixture
            .scheduler
            .get(&TaskId::from_uuid(*right_1.as_ref()))
            .unwrap()
            .state,
        TaskState::Completed
    );
    assert_eq!(
        fixture
            .scheduler
            .get(&TaskId::from_uuid(*join.as_ref()))
            .unwrap()
            .state,
        TaskState::Pending
    );
    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("operator-visible autonomy status should exist");
    assert_eq!(status.checkpoint_lineage.len(), 1);
    assert_eq!(status.routing_history.len(), 1);
    assert_eq!(
        status.checkpoint_lineage[0].checkpoint_id,
        fixture.checkpoint_id
    );
    let team_sizing = status
        .team_sizing
        .as_ref()
        .expect("orchestrator should emit the frozen adaptive-team contract");
    assert_eq!(team_sizing.decision_phase, "initial");
    assert_eq!(team_sizing.branch_frontier_width, 1);
    assert_eq!(team_sizing.desired_workers, 1);
    assert!(status.routing_history[0]
        .rationale
        .iter()
        .any(|reason| reason.contains("checkpoint")));
    assert!(fixture
        .orchestrator
        .autonomy_events(&fixture.workflow_id)
        .iter()
        .any(|event| matches!(event, AutonomyEvent::RoutingDecisionRecorded(_))));

    let shared_status =
        wait_for_event_bus_status(fixture.event_bus.as_ref(), &fixture.workflow_id, 1, 1).await;
    assert_eq!(shared_status.checkpoint_lineage.len(), 1);
    assert_eq!(shared_status.routing_history.len(), 1);
    assert_eq!(
        shared_status.checkpoint_lineage[0].checkpoint_id,
        fixture.checkpoint_id
    );
}

#[test]
fn gate10_rejects_invalid_graph_before_dispatch() {
    let err = TopologyCompiler::default()
        .compile(TaskId::new(), &cyclic_plan(), &TopologySignals::default())
        .expect_err("invalid gate10 graph should be rejected before dispatch");

    assert!(matches!(
        err,
        mister_smith_core::TopologyError::CycleDetected { .. }
    ));
}

#[tokio::test]
async fn gate10_transient_retry_remains_operator_visible() {
    let fixture = checkpointed_fixture();
    let context = GuardContext::new(GuardTarget::Branch(fixture.right_branch))
        .with_profile(assessed_profile(
            fixture.right_branch,
            HealthState::Degraded,
            Vec::new(),
            vec!["transient provider timeout"],
        ))
        .with_checkpoints(
            fixture
                .orchestrator
                .execution_graph(&fixture.workflow_id)
                .unwrap()
                .checkpoint_lineage,
        );

    let (decision, _) = fixture
        .orchestrator
        .supervise(&fixture.workflow_id, context)
        .await
        .expect("transient supervision should succeed");

    assert_eq!(decision.failure_class, FailureClass::Transient);
    assert_eq!(decision.intervention, InterventionType::Retry);

    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("operator status should exist after transient retry");
    assert_eq!(status.guard_decisions.len(), 1);
    assert_eq!(status.interventions.len(), 1);
    assert_eq!(
        status.checkpoint_lineage[0].checkpoint_id,
        fixture.checkpoint_id
    );
}

#[tokio::test]
async fn gate10_streaming_retry_remains_operator_visible() {
    let fixture = checkpointed_fixture();
    let context = GuardContext::new(GuardTarget::Branch(fixture.right_branch))
        .with_profile(assessed_profile(
            fixture.right_branch,
            HealthState::Degraded,
            vec![SemanticSignal {
                signal_kind: SemanticSignalKind::Stalled,
                severity: 70,
                detail: "stream stalled before branch completion".to_string(),
            }],
            vec!["stream monitor observed stall"],
        ))
        .with_checkpoints(
            fixture
                .orchestrator
                .execution_graph(&fixture.workflow_id)
                .unwrap()
                .checkpoint_lineage,
        );

    let (decision, record) = fixture
        .orchestrator
        .supervise(&fixture.workflow_id, context)
        .await
        .expect("streaming supervision should succeed");

    assert_eq!(decision.failure_class, FailureClass::Streaming);
    assert_eq!(decision.intervention, InterventionType::Retry);
    assert!(record.rationale.contains("targeted recovery"));

    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("operator status should exist after streaming retry");
    assert!(status.guard_decisions[0]
        .evidence
        .signal_descriptions
        .iter()
        .any(|detail| detail.contains("stalled")));
}

#[tokio::test]
async fn gate10_semantic_branch_isolation_remains_operator_visible() {
    let fixture = checkpointed_fixture();
    let context = GuardContext::new(GuardTarget::Branch(fixture.right_branch))
        .with_profile(assessed_profile(
            fixture.right_branch,
            HealthState::Unhealthy,
            vec![SemanticSignal {
                signal_kind: SemanticSignalKind::Repetitive,
                severity: 95,
                detail: "branch entered repetitive low-value loop".to_string(),
            }],
            vec!["semantic degradation exceeded safe threshold"],
        ))
        .with_checkpoints(
            fixture
                .orchestrator
                .execution_graph(&fixture.workflow_id)
                .unwrap()
                .checkpoint_lineage,
        );

    let (decision, _) = fixture
        .orchestrator
        .supervise(&fixture.workflow_id, context)
        .await
        .expect("semantic supervision should succeed");

    assert_eq!(decision.failure_class, FailureClass::Semantic);
    assert_eq!(decision.intervention, InterventionType::BranchIsolation);

    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("operator status should exist after isolation");
    let branch = status
        .branches
        .iter()
        .find(|branch| branch.branch_id == fixture.right_branch)
        .expect("isolated branch should remain visible");
    assert_eq!(branch.state, BranchState::Isolated);
    assert_eq!(status.interventions.len(), 1);
}

#[tokio::test]
async fn gate10_missing_input_fallback_remains_operator_visible() {
    let fixture = checkpointed_fixture();
    let context = GuardContext::new(GuardTarget::Branch(fixture.right_branch))
        .with_control_plane_fresh(false)
        .with_memory_metadata_available(false)
        .with_checkpoints(
            fixture
                .orchestrator
                .execution_graph(&fixture.workflow_id)
                .unwrap()
                .checkpoint_lineage,
        );

    let (decision, record) = fixture
        .orchestrator
        .supervise(&fixture.workflow_id, context)
        .await
        .expect("missing-input fallback should escalate visibly");

    assert_eq!(decision.failure_class, FailureClass::Structural);
    assert_eq!(decision.intervention, InterventionType::Escalation);
    assert!(record.rationale.contains("conservative fallback"));

    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("operator status should exist after conservative fallback");
    assert!(status
        .conservative_reasons
        .iter()
        .any(|reason| reason.contains("control-plane state unavailable")));
    assert!(status
        .conservative_reasons
        .iter()
        .any(|reason| reason.contains("memory metadata unavailable")));
}

#[tokio::test]
async fn gate10_delegated_resume_checks_all_branch_node_requirements() {
    let fixture = checkpointed_fixture();
    let store = InMemoryBranchCheckpointStore::default();
    let mut graph = fixture
        .orchestrator
        .execution_graph(&fixture.workflow_id)
        .expect("execution graph should exist");
    let right_two = node_id_for(&graph, "right-2");
    graph
        .nodes
        .iter_mut()
        .find(|node| node.node_id == right_two)
        .expect("right-2 node should exist")
        .delegation_requirement = Some(DelegationScope::ManageBranch);
    fixture.orchestrator.register_execution_graph(graph);

    let err = fixture
        .orchestrator
        .resume_branch_with_delegation(
            &fixture.workflow_id,
            &store,
            fixture.right_branch,
            Some(AgentId::new()),
            &capability_summary(
                DelegationScope::ExecuteWorkflow,
                RevocationState::Active,
                None,
            ),
        )
        .await
        .expect_err("mixed branch requirements should reject the wrong delegated scope");

    assert!(matches!(
        err,
        mister_smith_agents::AgentSystemError::PermissionDenied(message)
            if message.contains("delegation rejected")
    ));
}

#[tokio::test]
async fn gate10_delegated_resume_reconstructs_checkpoint_provenance() {
    let fixture = delegated_fixture(DelegationScope::ManageBranch);
    let store = InMemoryBranchCheckpointStore::default();
    let assigned_agent = AgentId::new();
    let capability =
        capability_summary(DelegationScope::ManageBranch, RevocationState::Active, None);

    let recovery = fixture
        .orchestrator
        .resume_branch_with_delegation(
            &fixture.workflow_id,
            &store,
            fixture.right_branch,
            Some(assigned_agent),
            &capability,
        )
        .await
        .expect("delegated branch resume should succeed");

    assert_eq!(
        recovery.resume_metadata.delegation_capability_id,
        Some(capability.capability_id)
    );
    assert_eq!(
        recovery.resume_metadata.delegation_scope,
        Some(DelegationScope::ManageBranch)
    );
    assert_eq!(
        recovery.resume_metadata.delegation_chain_depth,
        Some(capability.provenance.links.len())
    );
    assert_eq!(
        store
            .latest_resume(fixture.workflow_id, fixture.right_branch)
            .expect("resume metadata should persist")
            .delegation_capability_id,
        Some(capability.capability_id)
    );
    assert_eq!(
        recovery
            .checkpoint
            .failure_context
            .as_ref()
            .and_then(|context| context.get("delegation_capability_id"))
            .cloned(),
        Some(json!(capability.capability_id))
    );
    assert_eq!(
        recovery
            .checkpoint
            .failure_context
            .as_ref()
            .and_then(|context| context.get("delegation_chain_depth"))
            .cloned(),
        Some(json!(capability.provenance.links.len()))
    );

    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("delegated resume should remain operator-visible");
    assert_eq!(status.delegation_capabilities.len(), 1);
    assert!(status.delegation_alerts.is_empty());
    assert_eq!(
        status.delegation_capabilities[0]
            .provenance
            .terminal_capability,
        capability.capability_id
    );

    let shared_status =
        wait_for_delegation_status(fixture.event_bus.as_ref(), &fixture.workflow_id, 1, 0).await;
    assert_eq!(shared_status.delegation_capabilities.len(), 1);
    assert_eq!(
        shared_status.delegation_capabilities[0].provenance.links[0].scope,
        DelegationScope::ManageBranch
    );

    let routing_status =
        wait_for_event_bus_status(fixture.event_bus.as_ref(), &fixture.workflow_id, 1, 1).await;
    assert_eq!(routing_status.routing_history.len(), 1);
}

#[tokio::test]
async fn gate10_delegated_resume_preserves_existing_failure_context() {
    let fixture = delegated_fixture(DelegationScope::ManageBranch);
    let store = InMemoryBranchCheckpointStore::default();
    let capability =
        capability_summary(DelegationScope::ManageBranch, RevocationState::Active, None);
    let mut graph = fixture
        .orchestrator
        .execution_graph(&fixture.workflow_id)
        .expect("execution graph should exist");
    graph
        .checkpoint_lineage
        .last_mut()
        .expect("checkpoint lineage should exist")
        .failure_context = Some(json!({
        "failure_class": "transient",
        "details": "provider timeout",
    }));
    fixture.orchestrator.register_execution_graph(graph);

    let recovery = fixture
        .orchestrator
        .resume_branch_with_delegation(
            &fixture.workflow_id,
            &store,
            fixture.right_branch,
            Some(AgentId::new()),
            &capability,
        )
        .await
        .expect("delegated branch resume should succeed");

    let context = recovery
        .checkpoint
        .failure_context
        .as_ref()
        .expect("delegated resume should preserve checkpoint context");
    assert_eq!(context.get("failure_class"), Some(&json!("transient")));
    assert_eq!(context.get("details"), Some(&json!("provider timeout")));
    assert_eq!(
        context.get("delegation_capability_id"),
        Some(&json!(capability.capability_id))
    );
}

#[tokio::test]
async fn gate10_rejected_delegated_resume_surfaces_operator_reason() {
    let fixture = delegated_fixture(DelegationScope::ManageBranch);
    let store = InMemoryBranchCheckpointStore::default();
    let capability = capability_summary(
        DelegationScope::ManageBranch,
        RevocationState::Revoked,
        Some("delegation revoked before branch resume"),
    );

    let error = fixture
        .orchestrator
        .resume_branch_with_delegation(
            &fixture.workflow_id,
            &store,
            fixture.right_branch,
            Some(AgentId::new()),
            &capability,
        )
        .await
        .expect_err("revoked delegation should be rejected");

    assert!(matches!(
        error,
        mister_smith_agents::AgentSystemError::PermissionDenied(_)
    ));

    let status = fixture
        .orchestrator
        .autonomy_status(&fixture.workflow_id)
        .expect("rejected delegation should still be visible to operators");
    let alert = status
        .delegation_alerts
        .iter()
        .find(|alert| alert.capability_id == Some(capability.capability_id))
        .expect("rejected delegation should produce an alert");
    assert_eq!(alert.chain_depth, capability.provenance.links.len());
    assert_eq!(
        alert.rejection_reason.as_deref(),
        Some("delegation revoked before branch resume")
    );

    let shared_status =
        wait_for_delegation_status(fixture.event_bus.as_ref(), &fixture.workflow_id, 1, 1).await;
    assert!(shared_status
        .delegation_alerts
        .iter()
        .any(|alert| alert.rejection_reason.as_deref()
            == Some("delegation revoked before branch resume")));
}

#[tokio::test]
async fn gate10_frontier_rebalance_keeps_completed_branches_closed() {
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    );
    let mut graph = TopologyCompiler::default()
        .compile(
            TaskId::new(),
            &frontier_rebalance_plan(),
            &TopologySignals::default(),
        )
        .expect("frontier rebalance graph should compile");
    let workflow_id = graph.workflow_id;
    let root = node_id_for(&graph, "root");
    let left = node_id_for(&graph, "left");
    let right = node_id_for(&graph, "right");
    let join = node_id_for(&graph, "join");
    let root_branch = branch_id_for(&graph, "root");
    let left_branch = branch_id_for(&graph, "left");
    let right_branch = branch_id_for(&graph, "right");
    let join_branch = branch_id_for(&graph, "join");

    graph
        .nodes
        .iter_mut()
        .find(|node| node.node_id == root)
        .expect("root node should exist")
        .state = NodeState::Completed;
    graph.branch_mut(&root_branch).unwrap().state = BranchState::Completed;
    orchestrator.register_execution_graph(graph.clone());

    submit_task(
        &scheduler,
        TaskId::from_uuid(*root.as_ref()),
        "root",
        workflow_id,
        TaskState::Completed,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*left.as_ref()),
        "left",
        workflow_id,
        TaskState::Pending,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*right.as_ref()),
        "right",
        workflow_id,
        TaskState::Pending,
    );
    submit_task(
        &scheduler,
        TaskId::from_uuid(*join.as_ref()),
        "join",
        workflow_id,
        TaskState::Pending,
    );

    let first_pass = orchestrator
        .route_ready_branches(&workflow_id, &[AgentId::new(), AgentId::new()])
        .expect("wide frontier should route");
    assert_eq!(first_pass.len(), 2);
    let first_team_plan = orchestrator
        .adaptive_team_plan(&workflow_id)
        .expect("initial frontier should materialize a team plan");
    assert_eq!(first_team_plan.sizing_decision.decision_phase, "initial");
    assert_eq!(first_team_plan.sizing_decision.selected_workers, 2);

    for decision in &first_pass {
        for task_id in &decision.task_ids {
            scheduler.start(task_id).unwrap();
            scheduler.complete(task_id, json!({"done": true})).unwrap();
        }
    }

    let mut advanced_graph = orchestrator
        .execution_graph(&workflow_id)
        .expect("execution graph should remain registered");
    for node in &mut advanced_graph.nodes {
        if node.node_id == left || node.node_id == right {
            node.state = NodeState::Completed;
        }
    }
    advanced_graph.branch_mut(&left_branch).unwrap().state = BranchState::Completed;
    advanced_graph.branch_mut(&right_branch).unwrap().state = BranchState::Completed;
    advanced_graph.branch_mut(&join_branch).unwrap().state = BranchState::Pending;
    orchestrator.register_execution_graph(advanced_graph);

    let second_pass = orchestrator
        .route_ready_branches(&workflow_id, &[AgentId::new(), AgentId::new()])
        .expect("join frontier should route");
    assert_eq!(second_pass.len(), 1);
    assert_eq!(second_pass[0].branch_id, join_branch);
    assert_eq!(
        scheduler
            .get(&TaskId::from_uuid(*left.as_ref()))
            .unwrap()
            .state,
        TaskState::Completed
    );
    assert_eq!(
        scheduler
            .get(&TaskId::from_uuid(*right.as_ref()))
            .unwrap()
            .state,
        TaskState::Completed
    );
    let second_team_plan = orchestrator
        .adaptive_team_plan(&workflow_id)
        .expect("rebalanced frontier should materialize a team plan");
    assert_eq!(
        second_team_plan.sizing_decision.decision_phase,
        "frontier_rebalance"
    );
    assert_eq!(second_team_plan.sizing_decision.selected_workers, 1);
}
