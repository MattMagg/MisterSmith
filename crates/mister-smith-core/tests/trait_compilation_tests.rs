//! Verify all core traits are implementable with dummy structs.
//!
//! These tests confirm that the trait signatures compile correctly and
//! can be implemented by downstream crates.

use async_trait::async_trait;
use mister_smith_core::*;
use serde::de::DeserializeOwned;
use std::any::TypeId;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Mock Actor
// ---------------------------------------------------------------------------

struct MockActor {
    id: AgentId,
}

#[async_trait]
impl Actor for MockActor {
    type Message = String;
    type State = Vec<String>;
    type Error = ActorError;
    type Response = ();

    async fn handle_message(
        &mut self,
        message: Self::Message,
        state: &mut Self::State,
    ) -> Result<(), Self::Error> {
        state.push(message);
        Ok(())
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

// ---------------------------------------------------------------------------
// Mock Tool and Agent
// ---------------------------------------------------------------------------

struct MockTool {
    id: ToolId,
}

#[async_trait]
impl Tool for MockTool {
    async fn execute(&self, _params: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        Ok(serde_json::json!({"status": "ok"}))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema
    }

    fn capabilities(&self) -> ToolCapabilities {
        ToolCapabilities
    }

    fn tool_id(&self) -> ToolId {
        self.id
    }

    fn version(&self) -> semver::Version {
        semver::Version::new(0, 1, 0)
    }
}

struct MockAgent {
    tool: MockTool,
    ctx: String,
}

#[async_trait]
impl Tool for MockAgent {
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        self.tool.execute(params).await
    }

    fn schema(&self) -> ToolSchema {
        self.tool.schema()
    }

    fn capabilities(&self) -> ToolCapabilities {
        self.tool.capabilities()
    }

    fn tool_id(&self) -> ToolId {
        self.tool.tool_id()
    }

    fn version(&self) -> semver::Version {
        self.tool.version()
    }
}

#[async_trait]
impl Agent for MockAgent {
    type Context = String;
    type Error = ToolError;

    async fn process(&self, _message: serde_json::Value) -> Result<serde_json::Value, Self::Error> {
        Ok(serde_json::json!({"agent": "mock"}))
    }

    fn role(&self) -> AgentType {
        AgentType::Worker
    }

    fn context(&self) -> &Self::Context {
        &self.ctx
    }

    async fn initialize(&mut self, context: Self::Context) -> Result<(), Self::Error> {
        self.ctx = context;
        Ok(())
    }

    fn dependencies() -> Vec<TypeId> {
        vec![]
    }
}

// ---------------------------------------------------------------------------
// Mock Resource
// ---------------------------------------------------------------------------

struct MockResource {
    id: ResourceId,
}

#[async_trait]
impl Resource for MockResource {
    type Config = String;
    type Error = ResourceError;

    async fn acquire(_config: Self::Config) -> Result<Self, Self::Error> {
        Ok(MockResource {
            id: ResourceId::new(),
        })
    }

    async fn release(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn is_healthy(&self) -> bool {
        true
    }

    async fn health_check(&self) -> Result<HealthStatus, Self::Error> {
        Ok(HealthStatus::Healthy)
    }

    fn resource_id(&self) -> ResourceId {
        self.id
    }
}

// ---------------------------------------------------------------------------
// Mock Supervisor
// ---------------------------------------------------------------------------

struct MockSupervisor {
    id: AgentId,
    strategy: SupervisionStrategy,
}

#[async_trait]
impl Supervisor for MockSupervisor {
    type Child = String;
    type Error = SupervisionError;

    async fn supervise(&self, _children: Vec<Self::Child>) -> Result<(), Self::Error> {
        Ok(())
    }

    fn supervision_strategy(&self) -> &SupervisionStrategy {
        &self.strategy
    }

    fn restart_policy(&self) -> RestartPolicy {
        self.strategy.restart_policy
    }

    fn escalation_policy(&self) -> EscalationPolicy {
        self.strategy.escalation_policy
    }

    fn supervisor_id(&self) -> AgentId {
        self.id
    }
}

// ---------------------------------------------------------------------------
// Mock Transport
// ---------------------------------------------------------------------------

struct MockTransport {
    status: ConnectionStatus,
}

#[async_trait]
impl Transport for MockTransport {
    type Message = serde_json::Value;
    type Subscription = ();
    type ConnectionInfo = String;

    async fn send(&self, _destination: &str, _message: Self::Message) -> Result<(), NetworkError> {
        Ok(())
    }

    async fn broadcast(&self, _topic: &str, _message: Self::Message) -> Result<(), NetworkError> {
        Ok(())
    }

    async fn subscribe(&self, _pattern: &str) -> Result<Self::Subscription, NetworkError> {
        Ok(())
    }

    async fn request_response(
        &self,
        _destination: &str,
        _message: Self::Message,
        _timeout: Duration,
    ) -> Result<Self::Message, NetworkError> {
        Ok(serde_json::json!({}))
    }

    async fn connect(
        &mut self,
        _config: &TransportConfig,
    ) -> Result<Self::ConnectionInfo, NetworkError> {
        self.status = ConnectionStatus::Connected;
        Ok("connected".to_string())
    }

    async fn disconnect(&mut self) -> Result<(), NetworkError> {
        self.status = ConnectionStatus::Disconnected;
        Ok(())
    }

    fn connection_status(&self) -> ConnectionStatus {
        self.status
    }
}

// ---------------------------------------------------------------------------
// Mock EventPublisher
// ---------------------------------------------------------------------------

struct MockEventPublisher;

#[async_trait]
impl EventPublisher for MockEventPublisher {
    async fn publish(&self, _event: SystemEvent) -> Result<(), EventError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn actor_trait_compiles() {
    let actor = MockActor { id: AgentId::new() };
    assert_ne!(actor.actor_id().to_string(), "");
}

#[test]
fn tool_trait_compiles() {
    let tool = MockTool { id: ToolId::new() };
    assert_eq!(tool.version(), semver::Version::new(0, 1, 0));
}

#[test]
fn agent_extends_tool() {
    let agent = MockAgent {
        tool: MockTool { id: ToolId::new() },
        ctx: "test".to_string(),
    };
    assert_eq!(agent.role(), AgentType::Worker);
    assert_eq!(agent.context(), "test");
}

#[test]
fn supervisor_trait_compiles() {
    let supervisor = MockSupervisor {
        id: AgentId::new(),
        strategy: SupervisionStrategy::default(),
    };
    assert_eq!(supervisor.restart_policy(), RestartPolicy::OneForOne);
}

#[test]
fn resource_trait_compiles() {
    let resource = MockResource {
        id: ResourceId::new(),
    };
    assert!(resource.is_healthy());
    assert_ne!(resource.resource_id().to_string(), "");
}

#[test]
fn transport_trait_compiles() {
    let transport = MockTransport {
        status: ConnectionStatus::Disconnected,
    };
    assert_eq!(
        transport.connection_status(),
        ConnectionStatus::Disconnected
    );
}

#[test]
fn event_publisher_trait_compiles() {
    let _publisher = MockEventPublisher;
}

#[test]
fn trait_objects_are_object_safe() {
    // Verify Tool and EventPublisher can be used as trait objects
    fn _accepts_tool(_t: &dyn Tool) {}
    fn _accepts_publisher(_p: &dyn EventPublisher) {}

    // Verify generic Resource can be instantiated as trait object
    fn _accepts_resource(_r: Box<dyn Resource<Config = ResourceConfig, Error = ResourceError>>) {}
}

fn assert_autonomy_traits<T>()
where
    T: Clone + Send + Sync + std::fmt::Debug + serde::Serialize + DeserializeOwned + 'static,
{
}

fn assert_error_traits<T>()
where
    T: std::error::Error + Send + Sync + 'static,
{
}

#[test]
fn autonomy_types_compile_with_shared_trait_bounds() {
    assert_autonomy_traits::<ProofOutcomeClassification>();
    assert_autonomy_traits::<UnifiedResultEnvelope>();
    assert_autonomy_traits::<TaskResultView>();
    assert_autonomy_traits::<ResultProvenanceSummary>();
    assert_autonomy_traits::<SessionRetainedResultView>();
    assert_autonomy_traits::<OperatorResultPreview>();
    assert_autonomy_traits::<TaskShapeKind>();
    assert_autonomy_traits::<TaskShapeClassification>();
    assert_autonomy_traits::<TopologyRationale>();
    assert_autonomy_traits::<TopologyPlan>();
    assert_autonomy_traits::<TeamSizingDecision>();
    assert_autonomy_traits::<ContextBudget>();
    assert_autonomy_traits::<MetricWindow>();
    assert_autonomy_traits::<SemanticSignal>();
    assert_autonomy_traits::<ProfileFingerprint>();
    assert_autonomy_traits::<ProfileFingerprintRef>();
    assert_autonomy_traits::<ProfileSnapshot>();
    assert_autonomy_traits::<GuardEvidence>();
    assert_autonomy_traits::<GuardDecision>();
    assert_autonomy_traits::<InterventionRecord>();
    assert_autonomy_traits::<RepairLineageRef>();
    assert_autonomy_traits::<SupervisionEvidenceView>();
    assert_autonomy_traits::<DelegationCapability>();
    assert_autonomy_traits::<ProvenanceLink>();
    assert_autonomy_traits::<ProvenanceChain>();
}

#[test]
fn autonomy_ids_enums_and_errors_are_available() {
    let workflow_id = TaskId::new();
    let graph_id = ExecutionGraphId::new();
    let budget = ContextBudget {
        budget_id: ContextBudgetId::new(),
        scope: BudgetScope::Branch,
        max_units: 2048,
        reserved_units: 256,
        policy: BudgetPolicy::Summarize,
    };
    let plan = TopologyPlan {
        topology_kind: TopologyKind::Hybrid,
        parallelism_width: 4,
        task_shape: TaskShapeClassification {
            kind: TaskShapeKind::FanoutJoin,
            root_count: 1,
            max_parallel_width: 4,
            max_depth: 3,
            has_join: true,
            has_fanout: true,
            structural_signals: vec![
                "roots:1".to_string(),
                "max_parallel_width:4".to_string(),
                "max_depth:3".to_string(),
            ],
        },
        rationale: TopologyRationale {
            dependency_shape: "mixed dependency graph".to_string(),
            operational_signals: vec!["budget.pressure".to_string()],
            selected_for: "preserve concurrency without violating dependencies".to_string(),
            fallback_reason: Some(
                "degrade to sequential when supervision signals are stale".to_string(),
            ),
        },
        coordination_policy: CoordinationPolicy::Mixed,
        fallback_topology: Some(TopologyKind::Sequential),
    };
    let team_sizing = TeamSizingDecision {
        workflow_id,
        graph_id,
        decision_phase: "initial".to_string(),
        desired_workers: 6,
        selected_workers: 4,
        available_workers: 4,
        branch_frontier_width: 4,
        dependency_depth: 3,
        conservative_mode: true,
        budget_pressure: Some(72),
        cap_reason: Some("available worker cap".to_string()),
        rationale_lines: vec![
            "frontier width justified wider fanout".to_string(),
            "available workers capped the active team".to_string(),
        ],
        decided_at: chrono::Utc::now(),
    };
    let decision = GuardDecision {
        decision_id: GuardDecisionId::new(),
        failure_class: FailureClass::Streaming,
        intervention: InterventionType::BranchIsolation,
        evidence: GuardEvidence {
            profile_id: None,
            decision_basis: SupervisionDecisionBasis::ConservativeFallback,
            signal_descriptions: vec!["stream stalled".to_string()],
            checkpoint_ids: vec![],
            notes: vec!["operator-visible conservative intervention".to_string()],
        },
        target_scope: GuardTarget::Graph(graph_id),
        operator_visibility: true,
    };
    let fingerprint_ref = ProfileFingerprintRef {
        fingerprint_id: ProfileFingerprintId::new(),
        fingerprint_key: "executor:branch".to_string(),
        confidence: 0.81,
        expires_at: chrono::Utc::now(),
    };
    let _fingerprint = ProfileFingerprint {
        fingerprint_id: fingerprint_ref.fingerprint_id,
        target_kind: "executor".to_string(),
        target_selector: "branch".to_string(),
        source_refs: vec!["fixture://bounded-packet-021".to_string()],
        summary_payload: serde_json::json!({"health": "degraded"}),
        dominant_failure_modes: vec!["context_stall".to_string()],
        preferred_interventions: vec![InterventionType::ContextRefresh],
        confidence: 0.81,
        expires_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    let supervision_evidence = SupervisionEvidenceView {
        target_scope: SupervisionTargetScope {
            kind: SupervisionTargetKind::Graph,
            provider: None,
            graph_id: Some(graph_id),
            branch_id: None,
            node_id: None,
        },
        fingerprint_ref: Some(fingerprint_ref.clone()),
        profile_snapshot: Some(ProfileSnapshot {
            profile_id: ProfileSnapshotId::new(),
            target: ProfileTarget::Provider,
            health_state: HealthState::Degraded,
            latency_window: None,
            error_window: None,
            semantic_signals: vec![],
            fingerprint_ref: Some(fingerprint_ref),
            updated_at: chrono::Utc::now(),
        }),
        guard_decision: Some(decision.clone()),
        intervention_record: None,
        decision_basis: Some(
            SupervisionDecisionBasis::ConservativeFallback
                .as_str()
                .to_string(),
        ),
        repair_lineage_ref: Some(RepairLineageRef {
            source: "packet-020".to_string(),
            checkpoint_ref: Some("checkpoint-1".to_string()),
        }),
        proof_boundary: Some("deterministic-only".to_string()),
    };
    let canonical_result = UnifiedResultEnvelope {
        workflow_id,
        provider_kind: "openai_chatgpt".to_string(),
        model_id: "gpt-5.4".to_string(),
        description: "freeze the result contract".to_string(),
        runtime_execution_mode: serde_json::json!({"execution_boundary": "tool_bus"}),
        planner_output: serde_json::json!({"steps": 2}),
        execution_plan: serde_json::json!({"topology_hint": "hybrid"}),
        step_results: vec![serde_json::json!({"status": "completed"})],
        aggregated_result: serde_json::json!({"summary": "bounded final payload"}),
        proof_outcome: ProofOutcomeClassification::GraphFormedAndCompleted,
    };
    let provenance = ResultProvenanceSummary {
        runtime_execution_mode: canonical_result.runtime_execution_mode.clone(),
        graph_state: Some("completed".to_string()),
        graph_id: Some(graph_id.to_string()),
        source_fields: vec![
            "metadata.final_result".to_string(),
            "metadata.aggregated_result".to_string(),
        ],
    };
    let runtime_truth = packet_023_placeholder_runtime_truth(
        workflow_id,
        Some(graph_id),
        Some(ExecutionBranchId::new()),
        Some(ExecutionNodeId::new()),
        vec![
            RunTraceRelationshipKind::Graph,
            RunTraceRelationshipKind::Branch,
            RunTraceRelationshipKind::Node,
            RunTraceRelationshipKind::ToolBoundary,
            RunTraceRelationshipKind::Repair,
            RunTraceRelationshipKind::Supervision,
        ],
        vec![],
    );
    let task_result = TaskResultView {
        workflow_id,
        status: "completed".to_string(),
        proof_outcome: canonical_result.proof_outcome,
        orchestration_quality: None,
        runtime_truth: Some(runtime_truth.clone()),
        supervision_evidence: Some(supervision_evidence.clone()),
        result: canonical_result.clone(),
    };
    let session_result = SessionRetainedResultView {
        workflow_id,
        turn_index: 3,
        status: "completed".to_string(),
        assistant_result: serde_json::json!({
            "preview": "bounded answer preview",
            "aggregated_result": canonical_result.aggregated_result.clone(),
        }),
        preview: Some("bounded answer preview".to_string()),
        runtime_truth: Some(runtime_truth.clone()),
        provenance: provenance.clone(),
    };
    let operator_preview = OperatorResultPreview {
        workflow_id,
        proof_outcome: canonical_result.proof_outcome,
        preview_text: Some("bounded answer preview".to_string()),
        payload_location: "task.result".to_string(),
        orchestration_quality: None,
        runtime_truth: Some(runtime_truth),
        provenance_lines: vec![
            "canonical result stored in metadata.final_result".to_string(),
            "aggregated payload nested under metadata.aggregated_result".to_string(),
        ],
    };
    let topology_error = TopologyError::CycleDetected {
        graph_id: Some(graph_id),
        message: "cycle detected in execution graph".to_string(),
    };
    let autonomy_error: AutonomyError = topology_error.into();
    let system_error: SystemError = autonomy_error.into();

    assert_ne!(graph_id.to_string(), "");
    assert_eq!(budget.scope, BudgetScope::Branch);
    assert_eq!(plan.topology_kind, TopologyKind::Hybrid);
    assert_eq!(
        canonical_result.proof_outcome.as_str(),
        "graph_formed_and_completed"
    );
    assert_eq!(task_result.workflow_id, workflow_id);
    assert_eq!(session_result.turn_index, 3);
    assert_eq!(session_result.provenance.source_fields.len(), 2);
    assert_eq!(operator_preview.payload_location, "task.result");
    assert_eq!(team_sizing.workflow_id, workflow_id);
    assert!(team_sizing.selected_workers <= team_sizing.desired_workers);
    assert!(team_sizing.selected_workers <= team_sizing.available_workers);
    assert_eq!(decision.intervention, InterventionType::BranchIsolation);
    assert!(matches!(
        system_error,
        SystemError::Autonomy(AutonomyError::Topology(TopologyError::CycleDetected { .. }))
    ));
}

#[test]
fn profile_fingerprint_serializes_as_structured_summary_without_transcript_payload() {
    let fingerprint = ProfileFingerprint {
        fingerprint_id: ProfileFingerprintId::new(),
        target_kind: "branch".to_string(),
        target_selector: "branch-a".to_string(),
        source_refs: vec![
            "workflow:packet-021".to_string(),
            "checkpoint:checkpoint-1".to_string(),
        ],
        summary_payload: serde_json::json!({
            "health_state": "degraded",
            "signal_kinds": ["missing_context"],
            "decision_basis": "live_signals_only",
        }),
        dominant_failure_modes: vec!["missing_context".to_string()],
        preferred_interventions: vec![InterventionType::ContextRefresh],
        confidence: 0.82,
        updated_at: chrono::Utc::now(),
        expires_at: chrono::Utc::now() + chrono::Duration::hours(6),
    };

    let encoded = serde_json::to_value(&fingerprint).expect("fingerprint should serialize");
    assert_eq!(encoded["target_kind"], "branch");
    assert!(encoded["summary_payload"].is_object());
    assert_eq!(
        encoded["summary_payload"]["signal_kinds"][0],
        serde_json::json!("missing_context")
    );
    assert!(encoded["summary_payload"]["raw_transcript"].is_null());
}

#[test]
fn autonomy_error_types_compile_as_standard_errors() {
    assert_error_traits::<TopologyError>();
    assert_error_traits::<MemoryError>();
    assert_error_traits::<GuardError>();
    assert_error_traits::<DelegationError>();
    assert_error_traits::<AutonomyError>();
}
