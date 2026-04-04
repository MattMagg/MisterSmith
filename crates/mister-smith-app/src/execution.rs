//! Runtime-backed HTTP task execution for one real local workflow path.

use std::collections::BTreeMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream::kv::{self, Operation as KvOperation, Store as KvStore};
use async_nats::jetstream::stream::RetentionPolicy;
use async_trait::async_trait;
use chrono::Utc;
use mister_smith_agents::agent::spawn_supervised;
use mister_smith_agents::config::TaskState;
use mister_smith_agents::orchestrator::{LlmSupervision, LlmSupervisionConfig};
use mister_smith_agents::roles::executor::{ExecutorAgent, ExecutorMessage, ExecutorState};
use mister_smith_agents::roles::planner::PlannerAgent;
use mister_smith_agents::roles::planner::{normalize_planner_output, PlannerMessage, PlannerState};
use mister_smith_agents::scheduler::{
    ArrayAggregator, IdentityDecomposer, TaskAssignment, TaskScheduler,
};
use mister_smith_agents::{
    AgentConfig, BranchCheckpoint, ExecutionGraph, GuardContext, Orchestrator, ProfileAssessment,
    ToolBus, TopologyCompiler, TopologySignals,
};
use mister_smith_config::{
    FrameworkConfig, RuntimeProviderTier, RuntimeRoutingPolicy, RuntimeRoutingProfile,
};
use mister_smith_core::{
    AgentId, AgentType, BranchState, CoordinatorRuntimeProofView, DurableWorkflowEventKind,
    DurableWorkflowLifecycleState, DurableWorkflowLifecycleVerb, EffectBoundaryIntentState,
    EffectBoundaryOutcomeState, EscalationPolicy, ExecutionBranchId, ExecutionNodeId,
    ExternalDelegationEnvelope, FailureContextCheckpoint, GraphState, GuardTarget,
    HandoffClarificationRequest, HealthState, HistoryCompactionMode, LifecycleDecisionOutcome,
    LlmError, NodeState, ProfileFingerprint, RepairDirective, RepairDirectiveAction,
    SemanticSignal, SemanticSignalKind, StepBudgetPressureLevel, StepBudgetPressureSummary,
    StepDifficultyAssessment, StepDifficultyBucket, StepEvaluationRecord, StepPolicyAction,
    StepPolicyConfidenceLabel, StepPolicyDecision, StepPolicyInputRefs,
    StepPolicyProofBoundaryRef, StepPolicySummaryView, SupervisionEvidenceView,
    SupervisionStrategy, TaskId, Tool, ToolCapabilities, ToolError, ToolId, ToolSchema,
    VerifierVerdict, PACKET_023_ORCHESTRATION_ONLY,
};
use mister_smith_events::{AutonomyStatusView, EventBus, StepRoutingDecisionSummary};
use mister_smith_http::server::{
    TaskExecutionService, TaskLifecycleView, TaskListRequest, TaskStatusView,
    TaskSubmissionRequest, TaskSubmissionResponse, TaskSummaryView,
};
use mister_smith_http::websocket::{broadcast_event, WsEvent};
use mister_smith_llm::{
    BudgetEnforcer, BudgetNode, BudgetStore, CascadePolicy, CascadeTier, CircuitBreakerConfig,
    ClaudeSubscriptionProvider, MockProvider, ModelProvider, ModelRouter, OpenAiChatGptProvider,
    ProviderConfig, ProviderKind, RoutingPolicy,
};
use mister_smith_nats::{JetStreamConfig, JetStreamManager, NatsTransport};
use mister_smith_persistence::postgres::migrations::MigrationRunner;
use mister_smith_persistence::postgres::queries::{self, TaskRecord};
use mister_smith_persistence::repository::task::TaskRepository;
use mister_smith_persistence::{
    effect_boundary_records,
    kv::buckets::{KvBucketManager, AGENT_STATE},
    latest_history_compaction, lifecycle_decision_history, merge_effect_boundary_metadata,
    merge_history_compaction_metadata, merge_lifecycle_decision_metadata,
    merge_workflow_history_metadata, workflow_history, EffectBoundaryRecord,
    HistoryCompactionRecord, KvConfig, LifecycleDecisionRecord, PostgresConnection,
    ProfileFingerprintStore, Repository, WorkflowHistoryEventRecord,
};
use mister_smith_supervision::SupervisedSystem;
use mister_smith_transport::{MessageEnvelope, MessagePriority};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio::sync::broadcast;
use tokio::task::JoinSet;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::auth;

const PROVIDER_KIND: ProviderKind = ProviderKind::OpenAiChatGpt;
pub(crate) const PROVIDER_KIND_NAME: &str = "openai_chatgpt";
pub(crate) const MODEL_ID: &str = "gpt-5.4";
const WORKFLOW_STREAM: &str = "mister_smith_workflows";
const WORKFLOW_SUBJECT_PATTERN: &str = "workflow.>";
const RUNTIME_BUDGET_BUCKET: &str = "runtime_budget";
const AUTONOMY_STATUS_METADATA_KEY: &str = "autonomy_status";
const ACCEPTED_TASK_INGRESS_METADATA_KEY: &str = "accepted_task_ingress";
const TASK_INGRESS_REQUEST_SURFACE: &str = "POST /api/v1/tasks";
const WORKFLOW_TOOL_NAMESPACE: &str = "workflow";
const WORKFLOW_EXECUTE_STEP_TOOL: &str = "execute_step";
const DEFAULT_RUNTIME_CASCADE_THRESHOLD: f32 = 0.5;
const HISTORY_COMPACTION_THRESHOLD: usize = 12;
const HISTORY_COMPACTION_REPLAY_TAIL: usize = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeLlmSelection {
    provider_kind: ProviderKind,
    provider_kind_name: String,
    model_id: String,
}

#[derive(Debug, Clone, PartialEq)]
struct RuntimeEnvelopeProvenance {
    provider_kind: String,
    model_id: String,
    runtime_execution_mode: Value,
}

#[derive(Debug, Clone)]
struct RuntimeBootstrapPlan {
    active_selection: RuntimeLlmSelection,
    registered_providers: Vec<RuntimeLlmSelection>,
    routing_policy: RoutingPolicy,
    budget_root: Option<String>,
}

#[derive(Debug, Clone)]
struct RuntimeSupervisionPlan {
    target: GuardTarget,
    assessment: ProfileAssessment,
    checkpoints: Vec<BranchCheckpoint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeExecutionModeContext {
    routing_policy: String,
    registered_provider_count: usize,
    budget_root: Option<String>,
}

#[derive(Debug, Clone)]
struct GraphTransitionSnapshot {
    step_key: String,
    branch_id: ExecutionBranchId,
    node_state: NodeState,
    branch_state: BranchState,
    graph_state: GraphState,
}

#[derive(Debug, Clone)]
struct JetStreamBudgetStore {
    store: KvStore,
}

impl Default for RuntimeLlmSelection {
    fn default() -> Self {
        Self {
            provider_kind: PROVIDER_KIND,
            provider_kind_name: PROVIDER_KIND_NAME.to_string(),
            model_id: MODEL_ID.to_string(),
        }
    }
}

impl RuntimeLlmSelection {
    fn from_config(config: &FrameworkConfig) -> Self {
        Self {
            provider_kind: config.llm.provider_kind,
            provider_kind_name: config.llm.provider_kind.as_str().to_string(),
            model_id: config.llm.model_id.clone(),
        }
    }

    fn from_runtime_provider_tier(tier: &RuntimeProviderTier) -> Self {
        Self {
            provider_kind: tier.provider_kind,
            provider_kind_name: tier.provider_kind.as_str().to_string(),
            model_id: tier.model_id.clone(),
        }
    }

    fn provider_config(&self) -> ProviderConfig {
        ProviderConfig {
            provider_kind: self.provider_kind,
            model_id: self.model_id.clone(),
            timeout_ms: 120_000,
            ..ProviderConfig::default()
        }
    }
}

impl Default for RuntimeExecutionModeContext {
    fn default() -> Self {
        Self {
            routing_policy: "round_robin".to_string(),
            registered_provider_count: 1,
            budget_root: None,
        }
    }
}

impl RuntimeBootstrapPlan {
    fn from_config(config: &FrameworkConfig) -> Self {
        let fallback_selection = RuntimeLlmSelection::from_config(config);
        let Some(profile) = config.llm.runtime_routing_profile.as_ref() else {
            return Self {
                active_selection: fallback_selection.clone(),
                registered_providers: vec![fallback_selection],
                routing_policy: RoutingPolicy::RoundRobin,
                budget_root: None,
            };
        };

        if profile.tiers.is_empty() {
            return Self {
                active_selection: fallback_selection.clone(),
                registered_providers: vec![fallback_selection],
                routing_policy: RoutingPolicy::RoundRobin,
                budget_root: None,
            };
        }

        let registered_providers: Vec<RuntimeLlmSelection> = profile
            .tiers
            .iter()
            .map(RuntimeLlmSelection::from_runtime_provider_tier)
            .collect();
        let active_selection = registered_providers
            .first()
            .cloned()
            .unwrap_or_else(|| fallback_selection.clone());

        Self {
            active_selection,
            registered_providers,
            routing_policy: runtime_routing_policy_from_profile(profile),
            budget_root: Some(profile.budget_root.clone()),
        }
    }
}

fn runtime_routing_policy_from_profile(profile: &RuntimeRoutingProfile) -> RoutingPolicy {
    match profile.policy {
        RuntimeRoutingPolicy::Cascade => RoutingPolicy::Cascade(CascadePolicy {
            tiers: profile
                .tiers
                .iter()
                .map(|tier| CascadeTier {
                    provider_config: RuntimeLlmSelection::from_runtime_provider_tier(tier)
                        .provider_config(),
                    label: tier.label.clone(),
                })
                .collect(),
            escalation_threshold: DEFAULT_RUNTIME_CASCADE_THRESHOLD,
            max_escalations: profile.tiers.len().saturating_sub(1) as u32,
        }),
    }
}

fn runtime_routing_policy_name(policy: &RoutingPolicy) -> &'static str {
    match policy {
        RoutingPolicy::RoundRobin => "round_robin",
        RoutingPolicy::CostOptimized => "cost_optimized",
        RoutingPolicy::CapabilityMatched => "capability_matched",
        RoutingPolicy::Cascade(_) => "cascade",
        _ => "other",
    }
}

impl JetStreamBudgetStore {
    fn new(store: KvStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl BudgetStore for JetStreamBudgetStore {
    async fn get(&self, key: &str) -> Result<Option<BudgetNode>, LlmError> {
        let entry = self.store.entry(key).await.map_err(|error| {
            LlmError::InvalidRequest(format!(
                "failed to read JetStream budget key '{key}': {error}"
            ))
        })?;

        match entry {
            Some(entry) if entry.operation == KvOperation::Put => {
                let mut node: BudgetNode =
                    serde_json::from_slice(&entry.value).map_err(|error| {
                        LlmError::InvalidRequest(format!(
                            "failed to decode JetStream budget key '{key}': {error}"
                        ))
                    })?;
                node.key = entry.key;
                node.revision = entry.revision;
                Ok(Some(node))
            }
            Some(_) | None => Ok(None),
        }
    }

    async fn cas_update(&self, node: &BudgetNode, expected_revision: u64) -> Result<u64, LlmError> {
        let mut persisted = node.clone();
        persisted.revision = 0;
        let payload = serde_json::to_vec(&persisted).map_err(|error| {
            LlmError::InvalidRequest(format!(
                "failed to encode JetStream budget key '{}': {error}",
                node.key
            ))
        })?;

        let revision = if expected_revision == 0 {
            self.store
                .create(node.key.as_str(), payload.into())
                .await
                .map_err(|error| {
                    LlmError::InvalidRequest(format!(
                        "failed to create JetStream budget key '{}': {error}",
                        node.key
                    ))
                })?
        } else {
            self.store
                .update(node.key.as_str(), payload.into(), expected_revision)
                .await
                .map_err(|error| {
                    LlmError::InvalidRequest(format!(
                        "failed to CAS-update JetStream budget key '{}': {error}",
                        node.key
                    ))
                })?
        };

        Ok(revision)
    }
}

struct PreparedDecisionExecution {
    tasks: Vec<PreparedTaskExecution>,
}

struct PreparedTaskExecution {
    task: TaskAssignment,
    worker_id: AgentId,
    execution_input: Value,
}

#[derive(Debug, Clone)]
struct CompletedTaskExecution {
    task_id: TaskId,
    worker_id: AgentId,
    branch_id: Value,
    action: String,
    result: Value,
    step_evaluation: Option<StepEvaluationRecord>,
}

#[derive(Debug)]
struct FailedTaskExecution {
    completed_steps: Vec<CompletedTaskExecution>,
    task_id: TaskId,
    worker_id: AgentId,
    branch_id: Value,
    action: String,
    message: String,
    result: Option<Value>,
    step_evaluation: Option<StepEvaluationRecord>,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
struct InjectedVerifierPolicy {
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    verdict: Option<VerifierVerdict>,
    #[serde(default)]
    confidence: Option<f32>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    failure_code: Option<String>,
    #[serde(default)]
    checkpoint_ref: Option<String>,
    #[serde(default)]
    repair_directive: Option<RepairDirective>,
    #[serde(default)]
    clarification_request: Option<HandoffClarificationRequest>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct LocalRepairState {
    clarification_request: Option<HandoffClarificationRequest>,
    failure_context_checkpoint: Option<FailureContextCheckpoint>,
}

struct StepExecutionContext<'a> {
    workflow_id: TaskId,
    task: &'a TaskAssignment,
    worker_id: AgentId,
    branch_id: Value,
    action: String,
    repair_state: &'a LocalRepairState,
}

enum StepExecutionDisposition {
    Completed(CompletedTaskExecution),
    Continue(LocalRepairTransition),
    Failed(FailedTaskExecution),
}

struct LocalRepairTransition {
    repair_action: RepairDirectiveAction,
    repair_state: LocalRepairState,
    step_evaluation: StepEvaluationRecord,
    result: Value,
}

#[derive(Clone)]
struct WorkflowStepTool {
    id: ToolId,
}

#[async_trait]
impl Tool for WorkflowStepTool {
    async fn execute(&self, params: Value) -> Result<Value, ToolError> {
        let mut object = params.as_object().cloned().ok_or_else(|| {
            ToolError::ParameterValidationFailed(
                "workflow.execute_step expects an object payload".to_string(),
            )
        })?;

        object.insert("status".to_string(), json!("completed"));
        object.insert("execution_boundary".to_string(), json!("tool_bus"));
        object.insert(
            "tool_name".to_string(),
            json!(format!(
                "{WORKFLOW_TOOL_NAMESPACE}.{WORKFLOW_EXECUTE_STEP_TOOL}"
            )),
        );

        Ok(Value::Object(object))
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

fn register_runtime_tools(tool_bus: &ToolBus, agent_id: AgentId) {
    tool_bus.register_native_tool(
        WORKFLOW_EXECUTE_STEP_TOOL,
        WORKFLOW_TOOL_NAMESPACE,
        agent_id,
        "Execute one prepared workflow step through the runtime tool boundary",
        json!({"type": "object"}),
        json!({"type": "object"}),
        Arc::new(WorkflowStepTool { id: ToolId::new() }),
    );
}

fn runtime_execution_mode_with_context(
    selection: &RuntimeLlmSelection,
    context: &RuntimeExecutionModeContext,
) -> Value {
    json!({
        "workflow_runner": "tokio_task",
        "planner_lifecycle": "supervised_actor",
        "executor_lifecycle": "supervised_actor",
        "execution_boundary": "tool_bus",
        "tool_name": format!("{WORKFLOW_TOOL_NAMESPACE}.{WORKFLOW_EXECUTE_STEP_TOOL}"),
        "provider_kind": selection.provider_kind_name,
        "model_id": selection.model_id,
        "routing_policy": context.routing_policy,
        "registered_provider_count": context.registered_provider_count,
        "budget_root": context.budget_root.as_deref().unwrap_or("disabled"),
    })
}

fn runtime_execution_mode(selection: &RuntimeLlmSelection) -> Value {
    runtime_execution_mode_with_context(selection, &RuntimeExecutionModeContext::default())
}

fn render_unsupported_runtime_provider(selection: &RuntimeLlmSelection) -> String {
    format!(
        "provider '{}' is not supported by the current mister-smith-app binary. \
Choose one of: openai_chatgpt, claude_subscription, mock.",
        selection.provider_kind_name
    )
}

fn render_claude_subscription_auth_status(
    creds: &mister_smith_llm::ClaudeOAuthCredentials,
) -> Result<(), String> {
    if creds.is_expired() {
        Err(auth::render_claude_subscription_status(creds))
    } else {
        Ok(())
    }
}

fn runtime_envelope_provenance(
    metadata: &Value,
    fallback: &RuntimeLlmSelection,
) -> RuntimeEnvelopeProvenance {
    let provider_kind = metadata
        .get("provider_kind")
        .and_then(Value::as_str)
        .unwrap_or(fallback.provider_kind_name.as_str())
        .to_string();
    let model_id = metadata
        .get("model_id")
        .and_then(Value::as_str)
        .unwrap_or(fallback.model_id.as_str())
        .to_string();
    let runtime_execution_mode = metadata
        .get("runtime_execution_mode")
        .cloned()
        .unwrap_or_else(|| runtime_execution_mode(fallback));

    RuntimeEnvelopeProvenance {
        provider_kind,
        model_id,
        runtime_execution_mode,
    }
}

fn build_runtime_provider(
    selection: &RuntimeLlmSelection,
) -> Result<Arc<dyn ModelProvider>, String> {
    let provider_config = selection.provider_config();

    match selection.provider_kind {
        ProviderKind::OpenAiChatGpt => Ok(Arc::new(
            OpenAiChatGptProvider::new(provider_config)
                .map_err(|error| format!("provider initialization failed: {error}"))?,
        )),
        ProviderKind::ClaudeSubscription => Ok(Arc::new(
            ClaudeSubscriptionProvider::new(provider_config)
                .map_err(|error| format!("provider initialization failed: {error}"))?,
        )),
        ProviderKind::Mock => Ok(Arc::new(MockProvider::new(selection.model_id.clone()))),
        ProviderKind::OpenAi | ProviderKind::Anthropic => {
            Err(render_unsupported_runtime_provider(selection))
        }
    }
}

async fn verify_runtime_provider_auth(selection: &RuntimeLlmSelection) -> Result<(), String> {
    match selection.provider_kind {
        ProviderKind::OpenAiChatGpt => {
            let auth_status = auth::openai_chatgpt_status()
                .await
                .map_err(|error| format!("ChatGPT auth check failed: {error}"))?;
            if auth_status.is_chatgpt_session() {
                Ok(())
            } else {
                Err(auth::render_openai_chatgpt_status(&auth_status))
            }
        }
        ProviderKind::ClaudeSubscription => match auth::claude_subscription_status() {
            Ok(creds) => render_claude_subscription_auth_status(&creds),
            Err(LlmError::Authentication(message))
                if message.starts_with("No Claude subscription credentials found") =>
            {
                Err(auth::render_claude_subscription_missing())
            }
            Err(error) => Err(error.to_string()),
        },
        ProviderKind::Mock => Ok(()),
        ProviderKind::OpenAi | ProviderKind::Anthropic => {
            Err(render_unsupported_runtime_provider(selection))
        }
    }
}

async fn verify_runtime_router_auth(plan: &RuntimeBootstrapPlan) -> Result<(), String> {
    for selection in &plan.registered_providers {
        verify_runtime_provider_auth(selection).await?;
    }

    Ok(())
}

async fn ensure_runtime_budget_kv_store(
    jetstream: &JetStreamManager,
    bucket: &str,
) -> Result<KvStore, String> {
    jetstream
        .context()
        .create_or_update_key_value(kv::Config {
            bucket: bucket.to_string(),
            description: "Runtime budget state for the task-path router".to_string(),
            history: 1,
            ..Default::default()
        })
        .await
        .map_err(|error| {
            format!("JetStream budget bucket '{bucket}' initialization failed: {error}")
        })
}

async fn runtime_profile_fingerprint_store(
    jetstream: &JetStreamManager,
) -> Result<ProfileFingerprintStore, String> {
    let mut manager = KvBucketManager::new(jetstream.context().clone(), KvConfig::default());
    manager
        .initialize_buckets()
        .await
        .map_err(|error| format!("JetStream fingerprint bucket initialization failed: {error}"))?;
    let store = manager
        .bucket(AGENT_STATE)
        .map_err(|error| format!("JetStream fingerprint bucket lookup failed: {error}"))?
        .clone();
    Ok(ProfileFingerprintStore::new(store))
}

async fn build_runtime_budget_enforcer_with_bucket(
    jetstream: &JetStreamManager,
    bucket: &str,
    budget_root: &str,
) -> Result<BudgetEnforcer, String> {
    let store = JetStreamBudgetStore::new(ensure_runtime_budget_kv_store(jetstream, bucket).await?);
    if store
        .get(budget_root)
        .await
        .map_err(|error| format!("JetStream budget preflight failed: {error}"))?
        .is_none()
    {
        return Err(format!(
            "JetStream budget key '{budget_root}' is missing from bucket '{bucket}'"
        ));
    }

    Ok(BudgetEnforcer::new(Box::new(store)))
}

async fn build_runtime_budget_enforcer(
    jetstream: &JetStreamManager,
    budget_root: &str,
) -> Result<BudgetEnforcer, String> {
    build_runtime_budget_enforcer_with_bucket(jetstream, RUNTIME_BUDGET_BUCKET, budget_root).await
}

async fn register_runtime_router_providers(
    router: &ModelRouter,
    selections: &[RuntimeLlmSelection],
) -> Result<(), String> {
    for selection in selections {
        let provider_config = selection.provider_config();
        let provider = build_runtime_provider(selection)?;
        router
            .add_provider(provider_config, provider, CircuitBreakerConfig::default())
            .await;
    }

    Ok(())
}

fn runtime_supervision_strategy() -> SupervisionStrategy {
    SupervisionStrategy {
        max_failures: 32,
        failure_window: Duration::from_secs(60),
        escalation_policy: EscalationPolicy::LogAndIgnore,
        ..Default::default()
    }
}

fn persist_step_routing_history(metadata: &mut Value, history: &[StepRoutingDecisionSummary]) {
    if history.is_empty() {
        return;
    }

    if let Ok(serialized) = serde_json::to_value(history) {
        put_metadata(metadata, "step_routing_history", serialized);
    }
}

fn append_durable_workflow_history_event(
    metadata: &mut Value,
    workflow_id: TaskId,
    event_kind: DurableWorkflowEventKind,
    branch_id: Option<ExecutionBranchId>,
    node_id: Option<ExecutionNodeId>,
    lifecycle_state: Option<DurableWorkflowLifecycleState>,
    payload: Value,
) {
    let replay_position = workflow_history(metadata)
        .ok()
        .and_then(|history| {
            history
                .last()
                .map(|event| event.replay_position.saturating_add(1))
        })
        .unwrap_or(1);
    let history_event = WorkflowHistoryEventRecord {
        workflow_id,
        event_id: Uuid::new_v4(),
        replay_position,
        event_kind,
        recorded_at: Utc::now(),
        actor_agent_id: None,
        source: Some("runtime_task_service".to_string()),
        branch_id,
        node_id,
        lifecycle_state,
        effect_boundary_id: None,
        compaction_id: None,
        parent_event_id: None,
        payload,
    };
    merge_workflow_history_metadata(metadata, &[history_event])
        .expect("workflow history metadata merge should succeed");
    maybe_record_history_compaction(metadata, workflow_id);
}

fn append_effect_boundary_history_event(
    metadata: &mut Value,
    workflow_id: TaskId,
    event_kind: DurableWorkflowEventKind,
    effect_boundary_id: Uuid,
    payload: Value,
) {
    let replay_position = workflow_history(metadata)
        .ok()
        .and_then(|history| {
            history
                .last()
                .map(|event| event.replay_position.saturating_add(1))
        })
        .unwrap_or(1);
    let history_event = WorkflowHistoryEventRecord {
        workflow_id,
        event_id: Uuid::new_v4(),
        replay_position,
        event_kind,
        recorded_at: Utc::now(),
        actor_agent_id: None,
        source: Some("runtime_task_service".to_string()),
        branch_id: None,
        node_id: None,
        lifecycle_state: None,
        effect_boundary_id: Some(effect_boundary_id),
        compaction_id: None,
        parent_event_id: None,
        payload,
    };
    merge_workflow_history_metadata(metadata, &[history_event])
        .expect("workflow history metadata merge should succeed");
    maybe_record_history_compaction(metadata, workflow_id);
}

pub(crate) fn effect_boundary_idempotency_key(subject: &str, payload: &Value) -> String {
    let stable_suffix = payload
        .get("task_id")
        .or_else(|| payload.get("workflow_id"))
        .or_else(|| payload.get("step_id"))
        .map(|value| match value {
            Value::String(text) => text.clone(),
            other => other.to_string(),
        })
        .unwrap_or_else(|| "workflow".to_string());

    format!("{subject}:{stable_suffix}")
}

fn effect_boundary_uuid(workflow_id: TaskId, idempotency_key: &str) -> Uuid {
    let _ = (workflow_id, idempotency_key);
    Uuid::new_v4()
}

pub(crate) fn effect_boundary_projection(
    metadata: &Value,
    subject: &str,
    payload: &Value,
) -> Option<EffectBoundaryRecord> {
    let idempotency_key = effect_boundary_idempotency_key(subject, payload);
    effect_boundary_records(metadata)
        .ok()?
        .into_iter()
        .find(|record| record.idempotency_key == idempotency_key)
}

pub(crate) fn should_skip_effect_boundary_publish(
    metadata: &Value,
    subject: &str,
    payload: &Value,
) -> bool {
    effect_boundary_projection(metadata, subject, payload)
        .is_some_and(|record| record.outcome_state == EffectBoundaryOutcomeState::Completed)
}

fn merge_effect_boundary_record(
    metadata: &mut Value,
    record: EffectBoundaryRecord,
) -> Result<(), String> {
    merge_effect_boundary_metadata(metadata, std::slice::from_ref(&record))
        .map_err(|error| format!("failed to merge effect boundary metadata: {error}"))
}

fn lifecycle_state_for_graph_state(graph_state: GraphState) -> DurableWorkflowLifecycleState {
    match graph_state {
        GraphState::Pending | GraphState::Running | GraphState::Checkpointed => {
            DurableWorkflowLifecycleState::Active
        }
        GraphState::Completed => DurableWorkflowLifecycleState::Completed,
        GraphState::Failed | GraphState::Aborted => DurableWorkflowLifecycleState::Failed,
    }
}

fn workflow_lifecycle_state_from_metadata(
    metadata: &Value,
    status: &str,
) -> DurableWorkflowLifecycleState {
    let decision_state_and_time = lifecycle_decision_history(metadata)
        .ok()
        .and_then(|history| {
            history
                .last()
                .map(|decision| (decision.resulting_state, decision.decided_at))
        });

    let workflow_state_and_time = workflow_history(metadata).ok().and_then(|history| {
        history.iter().rev().find_map(|event| {
            event
                .lifecycle_state
                .map(|state| (state, event.recorded_at))
        })
    });

    match (decision_state_and_time, workflow_state_and_time) {
        (Some((decision_state, decision_time)), Some((workflow_state, workflow_time))) => {
            if workflow_time > decision_time || workflow_state.is_terminal() {
                workflow_state
            } else {
                decision_state
            }
        }
        (Some((decision_state, _)), None) => decision_state,
        (None, Some((workflow_state, _))) => workflow_state,
        (None, None) => DurableWorkflowLifecycleState::from_task_status(status),
    }
}

fn workflow_lifecycle_state_for_record(record: &TaskRecord) -> DurableWorkflowLifecycleState {
    workflow_lifecycle_state_from_metadata(&record.metadata, &record.status)
}

#[derive(Debug, Clone)]
struct LifecycleTransitionDecision {
    outcome: LifecycleDecisionOutcome,
    resulting_state: DurableWorkflowLifecycleState,
    persisted_status: String,
    note: Option<String>,
}

fn lifecycle_transition_decision(
    current_state: DurableWorkflowLifecycleState,
    current_status: &str,
    verb: DurableWorkflowLifecycleVerb,
) -> LifecycleTransitionDecision {
    let current_status = current_status.to_string();

    match verb {
        DurableWorkflowLifecycleVerb::Pause => {
            if current_state == DurableWorkflowLifecycleState::Paused {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Noop,
                    resulting_state: DurableWorkflowLifecycleState::Paused,
                    persisted_status: current_status,
                    note: Some("workflow is already paused".to_string()),
                }
            } else if current_state.is_terminal() {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Noop,
                    resulting_state: current_state,
                    persisted_status: current_status,
                    note: Some(format!(
                        "workflow is already terminal ({})",
                        current_state.as_str()
                    )),
                }
            } else {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Deferred,
                    resulting_state: current_state,
                    persisted_status: current_status,
                    note: Some(
                        "durable pause command recorded; live pause control is not implemented in packet 022"
                            .to_string(),
                    ),
                }
            }
        }
        DurableWorkflowLifecycleVerb::Resume => {
            if current_state == DurableWorkflowLifecycleState::Paused {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Deferred,
                    resulting_state: current_state,
                    persisted_status: current_status,
                    note: Some(
                        "durable resume command recorded; replay-backed resume is not implemented in packet 022"
                            .to_string(),
                    ),
                }
            } else if current_state == DurableWorkflowLifecycleState::Active {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Noop,
                    resulting_state: DurableWorkflowLifecycleState::Active,
                    persisted_status: current_status,
                    note: Some("workflow is already active".to_string()),
                }
            } else if current_state.is_terminal() {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Noop,
                    resulting_state: current_state,
                    persisted_status: current_status,
                    note: Some(format!(
                        "workflow is already terminal ({})",
                        current_state.as_str()
                    )),
                }
            } else {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Deferred,
                    resulting_state: current_state,
                    persisted_status: current_status,
                    note: Some(format!(
                        "workflow cannot resume while in {}",
                        current_state.as_str()
                    )),
                }
            }
        }
        DurableWorkflowLifecycleVerb::Cancel => {
            if current_state == DurableWorkflowLifecycleState::Cancelled {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Noop,
                    resulting_state: DurableWorkflowLifecycleState::Cancelled,
                    persisted_status: "cancelled".to_string(),
                    note: Some("workflow is already cancelled".to_string()),
                }
            } else if current_state.is_terminal() {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Noop,
                    resulting_state: current_state,
                    persisted_status: current_status,
                    note: Some(format!(
                        "workflow is already terminal ({})",
                        current_state.as_str()
                    )),
                }
            } else {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Deferred,
                    resulting_state: current_state,
                    persisted_status: current_status,
                    note: Some(
                        "durable cancel command recorded; live cancellation control is not implemented in packet 022"
                            .to_string(),
                    ),
                }
            }
        }
        DurableWorkflowLifecycleVerb::Terminate => {
            if current_state == DurableWorkflowLifecycleState::Terminated {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Noop,
                    resulting_state: DurableWorkflowLifecycleState::Terminated,
                    persisted_status: current_status,
                    note: Some("workflow is already terminated".to_string()),
                }
            } else if current_state.is_terminal() {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Noop,
                    resulting_state: current_state,
                    persisted_status: current_status,
                    note: Some(format!(
                        "workflow is already terminal ({})",
                        current_state.as_str()
                    )),
                }
            } else {
                LifecycleTransitionDecision {
                    outcome: LifecycleDecisionOutcome::Deferred,
                    resulting_state: current_state,
                    persisted_status: current_status,
                    note: Some(
                        "durable terminate command recorded; live termination control is not implemented in packet 022"
                            .to_string(),
                    ),
                }
            }
        }
    }
}

fn latest_branch_states_from_history(history: &[WorkflowHistoryEventRecord]) -> Vec<Value> {
    let mut branch_states = BTreeMap::<String, Value>::new();
    for event in history {
        let Some(branch_key) = event
            .branch_id
            .map(|id| id.to_string())
            .or_else(|| {
                event
                    .payload
                    .get("branch_id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .or_else(|| {
                event
                    .payload
                    .get("branch_anchor_step_key")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
        else {
            continue;
        };

        let branch_state = event
            .payload
            .get("branch_state")
            .cloned()
            .unwrap_or(Value::Null);
        let recovery_strategy = event
            .payload
            .get("recovery_strategy")
            .cloned()
            .unwrap_or(Value::Null);
        let assigned_agent_ids = event
            .payload
            .get("assigned_agent_ids")
            .cloned()
            .unwrap_or_else(|| json!([]));

        branch_states.insert(
            branch_key.clone(),
            json!({
                "branch_key": branch_key,
                "branch_state": branch_state,
                "recovery_strategy": recovery_strategy,
                "assigned_agent_ids": assigned_agent_ids,
            }),
        );
    }

    branch_states.into_values().collect()
}

fn latest_node_states_from_history(history: &[WorkflowHistoryEventRecord]) -> Vec<Value> {
    let mut node_states = BTreeMap::<String, Value>::new();
    for event in history {
        let Some(step_key) = event
            .payload
            .get("step_key")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .or_else(|| {
                event.node_id.map(|id| id.to_string()).or_else(|| {
                    event
                        .payload
                        .get("node_id")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                })
            })
        else {
            continue;
        };

        let node_state = event
            .payload
            .get("node_state")
            .cloned()
            .unwrap_or(Value::Null);
        let branch_id = event
            .branch_id
            .map(|id| id.to_string())
            .or_else(|| {
                event
                    .payload
                    .get("branch_id")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .or_else(|| {
                event
                    .payload
                    .get("branch_anchor_step_key")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            });

        node_states.insert(
            step_key.clone(),
            json!({
                "step_key": step_key,
                "node_state": node_state,
                "branch_id": branch_id,
            }),
        );
    }

    node_states.into_values().collect()
}

fn maybe_record_history_compaction(metadata: &mut Value, workflow_id: TaskId) {
    let Ok(history) = workflow_history(metadata) else {
        return;
    };
    if history.len() <= HISTORY_COMPACTION_THRESHOLD
        || history.len() <= HISTORY_COMPACTION_REPLAY_TAIL
    {
        return;
    }

    let source_end_index = history
        .len()
        .saturating_sub(HISTORY_COMPACTION_REPLAY_TAIL + 1);
    let Some(source_end_event) = history.get(source_end_index) else {
        return;
    };
    let compacted_history = &history[..=source_end_index];
    let lifecycle_state = compacted_history
        .iter()
        .rev()
        .find_map(|event| event.lifecycle_state)
        .unwrap_or_else(|| workflow_lifecycle_state_from_metadata(metadata, "running"));
    let branch_states = latest_branch_states_from_history(compacted_history);
    let node_states = latest_node_states_from_history(compacted_history);
    let graph_state = compacted_history
        .iter()
        .rev()
        .find_map(|event| {
            event
                .payload
                .get("graph_state")
                .cloned()
                .and_then(|raw| serde_json::from_value::<GraphState>(raw).ok())
        })
        .unwrap_or(match lifecycle_state {
            DurableWorkflowLifecycleState::Completed => GraphState::Completed,
            DurableWorkflowLifecycleState::Cancelled => GraphState::Aborted,
            DurableWorkflowLifecycleState::Terminated | DurableWorkflowLifecycleState::Failed => {
                GraphState::Failed
            }
            DurableWorkflowLifecycleState::Active
            | DurableWorkflowLifecycleState::Paused
            | DurableWorkflowLifecycleState::Cancelling => GraphState::Running,
        });

    let snapshot_event_id = Uuid::new_v4();
    let replay_position = history
        .last()
        .map(|event| event.replay_position.saturating_add(1))
        .unwrap_or(1);

    // Find earliest tail event position that should not be compacted
    let tail_events = &history[(source_end_index + 1)..];
    let earliest_tail_position = tail_events
        .iter()
        .filter(|event| event.event_kind == DurableWorkflowEventKind::HistoryCompacted)
        .map(|event| event.replay_position)
        .min();

    let replay_start_position = earliest_tail_position.unwrap_or(replay_position);

    let note = format!(
        "compacted replay positions {}-{} into snapshot {}, replay starts at {}",
        history
            .first()
            .map(|event| event.replay_position)
            .unwrap_or(1),
        source_end_event.replay_position,
        replay_position,
        replay_start_position
    );
    let compaction_id = Uuid::new_v4();
    let compaction = HistoryCompactionRecord {
        workflow_id,
        compaction_id,
        mode: HistoryCompactionMode::ReplayPointer,
        source_replay_start: history
            .first()
            .map(|event| event.replay_position)
            .unwrap_or(1),
        source_replay_end: source_end_event.replay_position,
        replay_start_position,
        replacement_event_id: Some(snapshot_event_id),
        preserved_lineage_note: note.clone(),
        recorded_at: Utc::now(),
    };
    let snapshot_event = WorkflowHistoryEventRecord {
        workflow_id,
        event_id: snapshot_event_id,
        replay_position,
        event_kind: DurableWorkflowEventKind::HistoryCompacted,
        recorded_at: compaction.recorded_at,
        actor_agent_id: None,
        source: Some("runtime_task_service.compaction".to_string()),
        branch_id: None,
        node_id: None,
        lifecycle_state: Some(lifecycle_state),
        effect_boundary_id: None,
        compaction_id: Some(compaction_id),
        parent_event_id: history.last().map(|event| event.event_id),
        payload: json!({
            "graph_state": graph_state,
            "lifecycle_state": lifecycle_state,
            "branch_states": branch_states,
            "node_states": node_states,
            "source_replay_start": compaction.source_replay_start,
            "source_replay_end": compaction.source_replay_end,
            "replay_start_position": replay_position,
            "preserved_lineage_note": note,
        }),
    };

    merge_history_compaction_metadata(metadata, std::slice::from_ref(&compaction))
        .expect("history compaction metadata merge should succeed");
    merge_workflow_history_metadata(metadata, std::slice::from_ref(&snapshot_event))
        .expect("workflow history metadata merge should succeed");
}

fn retain_replay_history_after_compaction(
    history: &mut Vec<WorkflowHistoryEventRecord>,
    compaction: &HistoryCompactionRecord,
) {
    history.retain(|event| event.replay_position > compaction.source_replay_end);
}

struct TerminalResultViews {
    aggregated_result: Value,
    final_result: Value,
    task_result: Value,
}

#[derive(Debug, Clone)]
struct StepPolicyEvaluationCandidate {
    record: StepEvaluationRecord,
    plan_index: usize,
    repair_attempt_count: u32,
}

fn execution_plan_step_indices_for_policy(execution_plan: &Value) -> BTreeMap<String, usize> {
    execution_plan
        .get("steps")
        .and_then(Value::as_array)
        .map(|steps| {
            steps
                .iter()
                .enumerate()
                .filter_map(|(index, step)| {
                    step.get("id")
                        .and_then(Value::as_str)
                        .map(|step_id| (step_id.to_string(), index + 1))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn step_policy_repair_attempt_count(record: &StepEvaluationRecord) -> u32 {
    record
        .failure_context_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.attempt_count)
        .or_else(|| {
            record
                .clarification_request
                .as_ref()
                .map(|request| request.attempt_count)
        })
        .unwrap_or(0)
}

fn terminal_step_evaluation_for_policy(
    canonical_result: &mister_smith_core::UnifiedResultEnvelope,
) -> Option<StepEvaluationRecord> {
    let step_indices = execution_plan_step_indices_for_policy(&canonical_result.execution_plan);
    canonical_result
        .step_results
        .iter()
        .filter_map(|step_result| {
            let record = step_result
                .get("step_evaluation")
                .cloned()
                .and_then(|value| serde_json::from_value::<StepEvaluationRecord>(value).ok())?;
            Some(StepPolicyEvaluationCandidate {
                plan_index: step_indices.get(&record.step_id).copied().unwrap_or(0),
                repair_attempt_count: step_policy_repair_attempt_count(&record),
                record,
            })
        })
        .max_by_key(|candidate| (candidate.plan_index, candidate.repair_attempt_count))
        .map(|candidate| candidate.record)
}

fn latest_step_routing_for_policy(
    metadata: &Value,
    step_id: &str,
) -> Option<StepRoutingDecisionSummary> {
    let history = metadata
        .get("step_routing_history")
        .cloned()
        .and_then(|value| serde_json::from_value::<Vec<StepRoutingDecisionSummary>>(value).ok())?;

    history
        .iter()
        .rev()
        .find(|entry| entry.step_id == step_id)
        .cloned()
}

fn step_policy_confidence_label(
    step_evaluation: &StepEvaluationRecord,
    routing: Option<&StepRoutingDecisionSummary>,
    supervision_evidence: Option<&SupervisionEvidenceView>,
) -> StepPolicyConfidenceLabel {
    if routing.is_some() && supervision_evidence.is_some() {
        return StepPolicyConfidenceLabel::Deterministic;
    }

    if step_evaluation.confidence.is_some() || routing.is_some() || supervision_evidence.is_some() {
        return StepPolicyConfidenceLabel::ModerateConfidence;
    }

    StepPolicyConfidenceLabel::LowConfidence
}

fn runtime_truth_ref_for_policy(
    canonical_result: &mister_smith_core::UnifiedResultEnvelope,
) -> String {
    if canonical_result.proof_outcome
        == mister_smith_core::ProofOutcomeClassification::FailedBeforeGraph
    {
        "packet-023:orchestration_substrate_completion".to_string()
    } else {
        "packet-023:placeholder_or_simulated_step_completion".to_string()
    }
}

fn difficulty_bucket_label(bucket: StepDifficultyBucket) -> &'static str {
    match bucket {
        StepDifficultyBucket::Low => "low",
        StepDifficultyBucket::Moderate => "moderate",
        StepDifficultyBucket::High => "high",
        StepDifficultyBucket::Critical => "critical",
    }
}

fn pressure_level_label(level: StepBudgetPressureLevel) -> &'static str {
    match level {
        StepBudgetPressureLevel::None => "none",
        StepBudgetPressureLevel::Watch => "watch",
        StepBudgetPressureLevel::Softcap => "softcap",
        StepBudgetPressureLevel::HardStop => "hard_stop",
    }
}

fn step_budget_pressure_for_policy(
    canonical_result: &mister_smith_core::UnifiedResultEnvelope,
    step_evaluation: &StepEvaluationRecord,
    step_id: &str,
    routing: Option<&StepRoutingDecisionSummary>,
) -> Option<StepBudgetPressureSummary> {
    let budget_root = canonical_result
        .runtime_execution_mode
        .get("budget_root")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty() && *value != "disabled")
        .map(ToOwned::to_owned);
    let triggered_budget_checkpoint = routing
        .map(|entry| {
            entry
                .triggered_checkpoints
                .iter()
                .any(|checkpoint| checkpoint == "budget_policy")
        })
        .unwrap_or(false);

    if budget_root.is_none() && !triggered_budget_checkpoint {
        return None;
    }

    let pressure_level = if triggered_budget_checkpoint
        && (step_evaluation.verdict == VerifierVerdict::Rejected
            || step_policy_repair_attempt_count(step_evaluation) >= 2)
    {
        StepBudgetPressureLevel::HardStop
    } else if triggered_budget_checkpoint {
        StepBudgetPressureLevel::Softcap
    } else {
        StepBudgetPressureLevel::Watch
    };

    Some(StepBudgetPressureSummary {
        workflow_id: canonical_result.workflow_id,
        step_id: step_id.to_string(),
        pressure_level,
        pressure_source: if triggered_budget_checkpoint {
            "step_routing_history".to_string()
        } else {
            "runtime_execution_mode".to_string()
        },
        policy_hint: "prefer_local_correction_before_escalation".to_string(),
        budget_root,
        note: Some(if triggered_budget_checkpoint {
            if pressure_level == StepBudgetPressureLevel::HardStop {
                "budget-policy checkpoint exhausted the bounded local correction window".to_string()
            } else {
                "budget-policy checkpoint shaped the current step".to_string()
            }
        } else {
            "budget-aware routing profile is active".to_string()
        }),
    })
}

fn step_difficulty_assessment_for_policy(
    canonical_result: &mister_smith_core::UnifiedResultEnvelope,
    step_evaluation: &StepEvaluationRecord,
    routing: Option<&StepRoutingDecisionSummary>,
    supervision_evidence: Option<&SupervisionEvidenceView>,
) -> StepDifficultyAssessment {
    let mut risk_points = 0u8;
    let mut reason_codes = Vec::new();

    match step_evaluation.verdict {
        VerifierVerdict::Accepted => {}
        VerifierVerdict::Rejected => {
            risk_points += 3;
            reason_codes.push("verifier_rejected".to_string());
        }
    }

    if let Some(confidence) = step_evaluation.confidence {
        if confidence < 0.55 {
            risk_points += 2;
            reason_codes.push("weak_verifier_confidence".to_string());
        } else if confidence < 0.8 {
            risk_points += 1;
            reason_codes.push("moderate_verifier_confidence".to_string());
        }
    }

    if step_evaluation.clarification_request.is_some() {
        risk_points += 2;
        reason_codes.push("missing_context".to_string());
    }

    if step_policy_repair_attempt_count(step_evaluation) >= 2 {
        risk_points += 1;
        reason_codes.push("repeated_local_repair".to_string());
    }

    if let Some(routing) = routing {
        if routing.action_changed {
            risk_points += 1;
            reason_codes.push("unstable_recent_step_history".to_string());
        }
        if let Some(confidence_score) = routing.confidence_score {
            if confidence_score < 0.6 {
                risk_points += 1;
                reason_codes.push("weak_routing_confidence".to_string());
            }
        }
        if routing
            .triggered_checkpoints
            .iter()
            .any(|checkpoint| checkpoint == "budget_policy")
        {
            reason_codes.push("budget_policy_checkpoint".to_string());
        }
    }

    if let Some(supervision_evidence) = supervision_evidence {
        if let Some(profile_snapshot) = supervision_evidence.profile_snapshot.as_ref() {
            match profile_snapshot.health_state {
                HealthState::Healthy => {}
                HealthState::Degraded => {
                    risk_points += 1;
                    reason_codes.push("degraded_supervision_health".to_string());
                }
                HealthState::Unhealthy => {
                    risk_points += 2;
                    reason_codes.push("unhealthy_supervision_health".to_string());
                }
                HealthState::Unknown => {
                    reason_codes.push("unknown_supervision_health".to_string());
                }
            }
        }
        if supervision_evidence
            .decision_basis
            .as_deref()
            .map(|basis| basis.contains("conservative") || basis.contains("fallback"))
            .unwrap_or(false)
        {
            risk_points += 1;
            reason_codes.push("conservative_supervision_decision".to_string());
        }
    }

    if reason_codes.is_empty() {
        reason_codes.push("stable_verifier_accept".to_string());
    }

    let difficulty_bucket = match risk_points {
        0 | 1 => StepDifficultyBucket::Low,
        2 | 3 => StepDifficultyBucket::Moderate,
        4 | 5 => StepDifficultyBucket::High,
        _ => StepDifficultyBucket::Critical,
    };

    StepDifficultyAssessment {
        workflow_id: canonical_result.workflow_id,
        step_id: step_evaluation.step_id.clone(),
        difficulty_bucket,
        confidence_label: step_policy_confidence_label(
            step_evaluation,
            routing,
            supervision_evidence,
        ),
        reason_codes,
        verifier_ref: Some(format!("packet-020:{}", step_evaluation.step_id)),
        routing_ref: routing.map(|entry| format!("step-routing:{}", entry.step_id)),
        supervision_ref: supervision_evidence
            .as_ref()
            .map(|_| format!("packet-021:{}", step_evaluation.step_id)),
        grounding_status_ref: Some(runtime_truth_ref_for_policy(canonical_result)),
    }
}

fn step_policy_decision_for_assessment(
    assessment: &StepDifficultyAssessment,
    budget_pressure: Option<&StepBudgetPressureSummary>,
    step_evaluation: &StepEvaluationRecord,
) -> StepPolicyDecision {
    let hard_stop_budget = budget_pressure
        .map(|summary| summary.pressure_level == StepBudgetPressureLevel::HardStop)
        .unwrap_or(false);
    let chosen_action = match assessment.difficulty_bucket {
        StepDifficultyBucket::Low => {
            if hard_stop_budget {
                StepPolicyAction::Escalate
            } else {
                StepPolicyAction::Keep
            }
        }
        StepDifficultyBucket::Moderate => {
            if hard_stop_budget {
                StepPolicyAction::Escalate
            } else if step_evaluation.clarification_request.is_some() {
                StepPolicyAction::Clarify
            } else {
                StepPolicyAction::Retry
            }
        }
        StepDifficultyBucket::High => {
            if hard_stop_budget {
                StepPolicyAction::Downgrade
            } else if step_evaluation.clarification_request.is_some() {
                StepPolicyAction::Clarify
            } else {
                StepPolicyAction::Retry
            }
        }
        StepDifficultyBucket::Critical => {
            if step_evaluation.clarification_request.is_some() && !hard_stop_budget {
                StepPolicyAction::Clarify
            } else if hard_stop_budget || step_policy_repair_attempt_count(step_evaluation) >= 2 {
                StepPolicyAction::Escalate
            } else {
                StepPolicyAction::Retry
            }
        }
    };

    let mut action_reason_parts = vec![format!(
        "difficulty={}",
        difficulty_bucket_label(assessment.difficulty_bucket)
    )];
    if let Some(budget_pressure) = budget_pressure {
        action_reason_parts.push(format!(
            "budget={}",
            pressure_level_label(budget_pressure.pressure_level)
        ));
    }
    if step_evaluation.clarification_request.is_some() {
        action_reason_parts.push("missing_context".to_string());
    }

    StepPolicyDecision {
        workflow_id: assessment.workflow_id,
        step_id: assessment.step_id.clone(),
        chosen_action,
        action_reason: action_reason_parts.join(" "),
        difficulty_ref: Some(format!("assessment:{}", assessment.step_id)),
        budget_ref: budget_pressure.map(|_| format!("budget:{}", assessment.step_id)),
        repair_lineage_ref: step_evaluation
            .failure_context_checkpoint
            .as_ref()
            .and_then(|checkpoint| checkpoint.checkpoint_ref.clone())
            .or_else(|| step_evaluation.checkpoint_ref.clone()),
        requires_operator_attention: matches!(chosen_action, StepPolicyAction::Escalate),
    }
}

fn build_step_policy_summary(
    canonical_result: &mister_smith_core::UnifiedResultEnvelope,
    metadata: &Value,
    supervision_evidence: Option<&SupervisionEvidenceView>,
) -> Option<StepPolicySummaryView> {
    let step_evaluation = terminal_step_evaluation_for_policy(canonical_result)?;
    let routing = latest_step_routing_for_policy(metadata, &step_evaluation.step_id);
    let budget_pressure = step_budget_pressure_for_policy(
        canonical_result,
        &step_evaluation,
        &step_evaluation.step_id,
        routing.as_ref(),
    );
    let assessment = step_difficulty_assessment_for_policy(
        canonical_result,
        &step_evaluation,
        routing.as_ref(),
        supervision_evidence,
    );
    let policy_decision = step_policy_decision_for_assessment(
        &assessment,
        budget_pressure.as_ref(),
        &step_evaluation,
    );

    Some(StepPolicySummaryView {
        difficulty_assessment: assessment,
        budget_pressure,
        policy_decision,
        input_refs: StepPolicyInputRefs {
            latest_step_evaluation: Some(format!("packet-020:{}", step_evaluation.step_id)),
            latest_step_routing: routing
                .as_ref()
                .map(|entry| format!("step-routing:{}", entry.step_id)),
            supervision_evidence: supervision_evidence
                .as_ref()
                .map(|_| format!("packet-021:{}", step_evaluation.step_id)),
            runtime_truth: Some(runtime_truth_ref_for_policy(canonical_result)),
            boundary_evidence: None,
        },
        proof_boundary_ref: StepPolicyProofBoundaryRef {
            owner_packet: "023".to_string(),
            task_proof: PACKET_023_ORCHESTRATION_ONLY.to_string(),
        },
        display_note:
            "placeholder orchestration proof only; local correction preferred before escalation"
                .to_string(),
    })
}

fn terminal_result_views(
    status: &str,
    mut canonical_result: mister_smith_core::UnifiedResultEnvelope,
    supervision_evidence: Option<SupervisionEvidenceView>,
    metadata: &Value,
    coordinator_runtime_proof: Option<CoordinatorRuntimeProofView>,
) -> Result<TerminalResultViews, String> {
    canonical_result.coordinator_runtime_proof = coordinator_runtime_proof;
    let aggregated_result = canonical_result.aggregated_result.clone();
    let step_policy =
        build_step_policy_summary(&canonical_result, metadata, supervision_evidence.as_ref());
    let task_result = serde_json::to_value(crate::autonomy::build_task_result_view(
        status,
        canonical_result.clone(),
        supervision_evidence,
        step_policy,
    ))
    .map_err(|error| format!("failed to serialize task result view: {error}"))?;
    let final_result = serde_json::to_value(canonical_result)
        .map_err(|error| format!("failed to serialize canonical result: {error}"))?;

    Ok(TerminalResultViews {
        aggregated_result,
        final_result,
        task_result,
    })
}

#[derive(Clone)]
pub(crate) struct RuntimeTaskService {
    pool: PgPool,
    repository: Arc<TaskRepository>,
    jetstream: Arc<JetStreamManager>,
    event_tx: broadcast::Sender<WsEvent>,
    router: Arc<ModelRouter>,
    orchestrator: Arc<Orchestrator>,
    supervised_system: Arc<SupervisedSystem>,
    runtime_supervisor_id: AgentId,
    tool_bus: Arc<ToolBus>,
    boot_started_at: chrono::DateTime<Utc>,
    default_coordinator_id: AgentId,
    worker_ids: Vec<AgentId>,
    runtime_llm: RuntimeLlmSelection,
    runtime_execution_mode_context: RuntimeExecutionModeContext,
}

impl RuntimeTaskService {
    pub(crate) async fn bootstrap(
        config: &FrameworkConfig,
        event_bus: Arc<EventBus>,
        nats_transport: Option<Arc<NatsTransport>>,
        supervised_system: Arc<SupervisedSystem>,
        event_tx: broadcast::Sender<WsEvent>,
    ) -> Result<Arc<Self>, String> {
        let boot_started_at = Utc::now();
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL must be set for runtime task execution".to_string())?;
        let postgres = PostgresConnection::connect(&database_url)
            .await
            .map_err(|error| format!("PostgreSQL connection failed: {error}"))?;
        let pool = postgres.pool().clone();
        let migrations = MigrationRunner::new(pool.clone());
        migrations
            .run()
            .await
            .map_err(|error| format!("PostgreSQL migration failed: {error}"))?;
        let verified = migrations
            .verify()
            .await
            .map_err(|error| format!("PostgreSQL migration verification failed: {error}"))?;
        if !verified {
            return Err("PostgreSQL migrations did not verify cleanly".to_string());
        }

        let runtime_bootstrap = RuntimeBootstrapPlan::from_config(config);
        verify_runtime_router_auth(&runtime_bootstrap).await?;

        let nats_transport = nats_transport
            .ok_or_else(|| "NATS transport must be configured for runtime execution".to_string())?;
        let nats_client = nats_transport
            .inner_client()
            .await
            .map_err(|error| format!("NATS client unavailable: {error}"))?;
        let jetstream = Arc::new(JetStreamManager::new(
            nats_client,
            JetStreamConfig::default(),
        ));
        jetstream
            .create_stream(
                WORKFLOW_STREAM,
                vec![WORKFLOW_SUBJECT_PATTERN.to_string()],
                RetentionPolicy::Limits,
            )
            .await
            .map_err(|error| format!("JetStream stream initialization failed: {error}"))?;

        let routing_policy_name = runtime_routing_policy_name(&runtime_bootstrap.routing_policy);
        let registered_provider_count = runtime_bootstrap.registered_providers.len();
        let budget_root = runtime_bootstrap.budget_root.clone();
        let runtime_execution_mode_context = RuntimeExecutionModeContext {
            routing_policy: routing_policy_name.to_string(),
            registered_provider_count,
            budget_root: budget_root.clone(),
        };
        let router = if let Some(budget_root) = budget_root.as_ref() {
            let budget_enforcer =
                build_runtime_budget_enforcer(jetstream.as_ref(), budget_root).await?;
            Arc::new(
                ModelRouter::new(runtime_bootstrap.routing_policy.clone())
                    .with_budget(budget_enforcer, budget_root.clone()),
            )
        } else {
            Arc::new(ModelRouter::new(runtime_bootstrap.routing_policy.clone()))
        };
        register_runtime_router_providers(router.as_ref(), &runtime_bootstrap.registered_providers)
            .await?;

        let scheduler = Arc::new(TaskScheduler::new());
        let orchestrator = Arc::new(
            Orchestrator::new(
                Arc::new(IdentityDecomposer),
                Arc::new(ArrayAggregator),
                scheduler,
            )
            .with_event_bus(event_bus),
        );
        let runtime_supervisor_id = supervised_system
            .create_supervisor(runtime_supervision_strategy())
            .await;
        let tool_bus = Arc::new(ToolBus::new());
        register_runtime_tools(tool_bus.as_ref(), runtime_supervisor_id);

        info!(
            provider_kind = runtime_bootstrap
                .active_selection
                .provider_kind_name
                .as_str(),
            model_id = runtime_bootstrap.active_selection.model_id.as_str(),
            routing_policy = routing_policy_name,
            registered_provider_count,
            budget_root = budget_root.as_deref().unwrap_or("disabled"),
            stream = WORKFLOW_STREAM,
            "Runtime task execution service ready"
        );

        Ok(Arc::new(Self {
            pool: pool.clone(),
            repository: Arc::new(TaskRepository::new(pool)),
            jetstream,
            event_tx,
            router,
            orchestrator,
            supervised_system,
            runtime_supervisor_id,
            tool_bus,
            boot_started_at,
            default_coordinator_id: AgentId::new(),
            worker_ids: vec![AgentId::new(), AgentId::new()],
            runtime_llm: runtime_bootstrap.active_selection,
            runtime_execution_mode_context,
        }))
    }

    pub(crate) fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub(crate) fn provider_kind_name(&self) -> &str {
        &self.runtime_llm.provider_kind_name
    }

    pub(crate) fn model_id(&self) -> &str {
        &self.runtime_llm.model_id
    }

    fn runtime_execution_mode(&self) -> Value {
        runtime_execution_mode_with_context(&self.runtime_llm, &self.runtime_execution_mode_context)
    }

    pub(crate) async fn autonomy_status(&self, workflow_id: TaskId) -> Option<AutonomyStatusView> {
        if let Some(mut view) = self.orchestrator.autonomy_status(&workflow_id) {
            if let Ok(Some(record)) = queries::find_task(&self.pool, *workflow_id.as_ref()).await {
                crate::autonomy::enrich_lifecycle_state(&mut view, &record.metadata);
                crate::autonomy::enrich_session_linkage(&mut view, &record.metadata);
                crate::autonomy::enrich_step_routing_history(&mut view, &record.metadata);
                crate::autonomy::enrich_result_preview(
                    &mut view,
                    &record.metadata,
                    record.result.as_ref(),
                );
            }
            return Some(view);
        }

        let record = queries::find_task(&self.pool, *workflow_id.as_ref())
            .await
            .ok()
            .flatten()?;
        self.restore_execution_graph_from_record(&record);
        if let Some(mut view) = self.orchestrator.autonomy_status(&workflow_id) {
            crate::autonomy::enrich_lifecycle_state(&mut view, &record.metadata);
            crate::autonomy::enrich_session_linkage(&mut view, &record.metadata);
            crate::autonomy::enrich_step_routing_history(&mut view, &record.metadata);
            crate::autonomy::enrich_result_preview(
                &mut view,
                &record.metadata,
                record.result.as_ref(),
            );
            return Some(view);
        }
        recover_persisted_autonomy_status(&record)
    }

    pub(crate) fn autonomy_workflows(&self) -> Vec<TaskId> {
        self.orchestrator.autonomy_workflow_ids()
    }

    pub(crate) async fn persisted_autonomy_workflows(&self) -> Vec<TaskId> {
        queries::list_workflows_with_persisted_autonomy_status(&self.pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(TaskId::from_uuid)
            .collect()
    }

    fn restore_execution_graph_from_record(&self, record: &TaskRecord) -> bool {
        let workflow_id = TaskId::from_uuid(record.task_id);
        if self.orchestrator.execution_graph(&workflow_id).is_some() {
            return false;
        }

        let Some(execution_plan) = record.metadata.get("execution_plan") else {
            return false;
        };
        let Ok(mut history) = workflow_history(&record.metadata) else {
            return false;
        };
        if history.is_empty() {
            return false;
        }
        if let Ok(Some(compaction)) = latest_history_compaction(&record.metadata) {
            retain_replay_history_after_compaction(&mut history, &compaction);
        }
        if history.is_empty() {
            return false;
        }

        let Ok(graph) = compile_execution_plan(workflow_id, execution_plan) else {
            return false;
        };
        let Ok(graph) = self
            .orchestrator
            .replay_execution_graph_from_history(graph, &history)
        else {
            return false;
        };

        self.orchestrator.register_execution_graph(graph);
        true
    }

    pub(crate) async fn recover_orphaned_workflow(
        &self,
        workflow_id: TaskId,
    ) -> Result<Option<TaskRecord>, String> {
        let Some(record) = queries::find_task(&self.pool, *workflow_id.as_ref())
            .await
            .map_err(|error| format!("failed to load workflow record {workflow_id}: {error}"))?
        else {
            return Ok(None);
        };

        self.restore_execution_graph_from_record(&record);

        if !should_recover_orphaned_workflow(
            &record,
            self.boot_started_at,
            self.orchestrator.execution_graph(&workflow_id).is_some(),
        ) {
            return Ok(Some(record));
        }

        let failed_at = Utc::now();
        let mut metadata = record.metadata.clone();
        let message = "workflow interrupted by runtime restart before session sync";
        put_metadata(
            &mut metadata,
            "restart_recovery",
            json!({
                "reason": message,
                "recovered_at": failed_at,
            }),
        );
        put_metadata(&mut metadata, "failure", json!({ "message": message }));
        mark_persisted_autonomy_status_failed(&mut metadata);

        let aggregated_result = json!({
            "error": message,
            "recovered_after_restart": true,
        });
        let envelope_provenance = runtime_envelope_provenance(&metadata, &self.runtime_llm);
        let final_result = crate::autonomy::build_canonical_result_envelope(
            crate::autonomy::CanonicalResultEnvelopeInput {
                workflow_id,
                provider_kind: envelope_provenance.provider_kind.as_str(),
                model_id: envelope_provenance.model_id.as_str(),
                description: metadata
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or(message),
                runtime_execution_mode: envelope_provenance.runtime_execution_mode,
                planner_output: metadata
                    .get("planner_output")
                    .cloned()
                    .unwrap_or(Value::Null),
                execution_plan: metadata
                    .get("execution_plan")
                    .cloned()
                    .unwrap_or(Value::Null),
                step_results: metadata
                    .get("step_results")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default(),
                aggregated_result: aggregated_result.clone(),
                status: "failed",
            },
        );
        let result_views = terminal_result_views(
            "failed",
            final_result.clone(),
            self.current_supervision_evidence(workflow_id, &metadata, None),
            &metadata,
            self.orchestrator.coordinator_runtime_proof_view(&workflow_id),
        )
        .unwrap_or(TerminalResultViews {
            aggregated_result: aggregated_result.clone(),
            final_result: Value::Null,
            task_result: Value::Null,
        });
        put_metadata(
            &mut metadata,
            "aggregated_result",
            result_views.aggregated_result.clone(),
        );
        put_metadata(
            &mut metadata,
            "final_result",
            result_views.final_result.clone(),
        );
        append_durable_workflow_history_event(
            &mut metadata,
            workflow_id,
            DurableWorkflowEventKind::LifecycleChanged,
            None,
            None,
            Some(DurableWorkflowLifecycleState::Failed),
            json!({
                "graph_state": GraphState::Failed,
                "error": message,
            }),
        );
        self.capture_autonomy_status_metadata(
            workflow_id,
            &mut metadata,
            Some(&result_views.task_result),
        );

        self.update_root_record(
            workflow_id,
            "failed",
            metadata,
            Some(result_views.task_result),
            record.started_at.or(Some(record.created_at)),
            Some(failed_at),
        )
        .await?;

        queries::find_task(&self.pool, *workflow_id.as_ref())
            .await
            .map_err(|error| format!("failed to reload workflow record {workflow_id}: {error}"))
    }

    async fn run_workflow(
        &self,
        workflow_id: TaskId,
        request: TaskSubmissionRequest,
    ) -> Result<(), String> {
        let coordinator_id = coordinator_id_for_request(&request, self.default_coordinator_id);
        self.orchestrator
            .register_workflow_coordinator(&workflow_id, coordinator_id);
        let mut metadata = initial_metadata(
            &request,
            coordinator_id,
            &self.worker_ids,
            "running",
            &self.runtime_llm,
        );
        put_metadata(
            &mut metadata,
            "runtime_execution_mode",
            self.runtime_execution_mode(),
        );
        self.update_root_record(
            workflow_id,
            "running",
            metadata.clone(),
            None,
            Some(Utc::now()),
            None,
        )
        .await?;
        self.publish_event(
            coordinator_id,
            "workflow.started",
            "workflow.started",
            workflow_id,
            json!({
                "workflow_id": workflow_id,
                "provider_kind": self.provider_kind_name(),
                "model_id": self.model_id(),
                "description": request.description,
            }),
        )
        .await?;

        let planning_context = json!({
            "submission_path": if request.conversation.is_some() { "session" } else { "http" },
            "provider_kind": self.provider_kind_name(),
            "model_id": self.model_id(),
            "external_delegation": request.delegation,
            "conversation": request.conversation.as_ref().map(|conversation| {
                json!({
                    "session_id": conversation.session_id,
                    "turn_index": conversation.turn_index,
                    "coordinator_agent_id": conversation.coordinator_agent_id,
                    "retained_context": conversation.retained_context,
                })
            }).unwrap_or(Value::Null),
            "execution_contract": {
                "return_json_only": true,
                "workflow_policy": {
                    "prefer_smallest_workflow": true,
                    "default_shape": "sequential",
                    "parallelize_when": "the task has clearly independent subproblems or the user explicitly asks for parallel work",
                    "merge_when": "multiple branch outputs must be combined",
                    "merge_role": "coordinator",
                    "preserve_requested_output_shape": true
                },
                "schema": {
                    "goal": "string",
                    "topology_hint": "optional string",
                    "steps": [
                        {
                            "id": "string",
                            "step": 1,
                            "action": "string",
                            "description": "string",
                            "role": "optional string",
                            "branch": "optional string",
                            "depends_on": "optional array of step ids"
                        }
                    ]
                }
            }
        });

        let planner_output = self
            .execute_plan_with_supervised_planner(
                workflow_id,
                request.description.clone(),
                planning_context.clone(),
            )
            .await?;
        let planner_step_routing_history = self.orchestrator.step_routing_history(&workflow_id);
        let execution_plan = normalize_runtime_plan(
            &request.description,
            &planning_context,
            planner_output.clone(),
        );
        persist_step_routing_history(&mut metadata, &planner_step_routing_history);
        put_metadata(&mut metadata, "planner_output", planner_output.clone());
        put_metadata(&mut metadata, "execution_plan", execution_plan.clone());
        let mut graph = match compile_execution_plan(workflow_id, &execution_plan) {
            Ok(graph) => graph,
            Err(error) => {
                self.update_task_metadata(workflow_id, metadata.clone())
                    .await?;
                return Err(error);
            }
        };
        self.update_task_metadata(workflow_id, metadata.clone())
            .await?;
        self.publish_event(
            coordinator_id,
            "workflow.planned",
            "workflow.planned",
            workflow_id,
            json!({
                "workflow_id": workflow_id,
                "provider_kind": self.provider_kind_name(),
                "model_id": self.model_id(),
                "planner_output": planner_output,
                "execution_plan": execution_plan,
                "runtime_execution_mode": self.runtime_execution_mode(),
            }),
        )
        .await?;
        graph.state = GraphState::Running;
        self.orchestrator.register_execution_graph(graph.clone());
        append_durable_workflow_history_event(
            &mut metadata,
            workflow_id,
            DurableWorkflowEventKind::LifecycleChanged,
            None,
            None,
            Some(DurableWorkflowLifecycleState::Active),
            json!({
                "graph_state": GraphState::Running,
                "node_count": graph.nodes.len(),
                "branch_count": graph.branches.len(),
            }),
        );
        self.capture_autonomy_status_metadata(workflow_id, &mut metadata, None);
        self.update_task_metadata(workflow_id, metadata.clone())
            .await?;

        for node in &graph.nodes {
            self.orchestrator
                .scheduler()
                .submit(task_assignment_for_node(
                    workflow_id,
                    node,
                    &request.description,
                ));
        }

        let mut step_results = BTreeMap::new();
        let mut routing_history = Vec::new();

        while !self.orchestrator.all_subtasks_completed(&workflow_id) {
            let decisions = self
                .orchestrator
                .route_ready_branches(&workflow_id, &self.worker_ids)
                .map_err(|error| format!("branch routing failed: {error}"))?;
            if decisions.is_empty() {
                return Err(format!(
                    "no routable branches remained for workflow {workflow_id}"
                ));
            }

            let adaptive_team_plan = self.orchestrator.adaptive_team_plan(&workflow_id);
            if let Some(team_plan) = adaptive_team_plan.as_ref() {
                put_metadata(
                    &mut metadata,
                    "active_worker_ids",
                    json!(team_plan.worker_ids),
                );
                put_metadata(
                    &mut metadata,
                    "team_sizing",
                    json!(team_plan.sizing_decision),
                );
            }

            let decision_payload = decisions
                .iter()
                .map(|decision| {
                    json!({
                        "branch_id": decision.branch_id,
                        "selected_agent": decision.selected_agent,
                        "task_ids": decision.task_ids,
                        "rationale": decision.rationale,
                    })
                })
                .collect::<Vec<_>>();
            routing_history.extend(decision_payload.clone());
            put_metadata(&mut metadata, "routing_history", json!(routing_history));
            self.capture_autonomy_status_metadata(workflow_id, &mut metadata, None);
            self.update_task_metadata(workflow_id, metadata.clone())
                .await?;
            self.publish_event(
                coordinator_id,
                "workflow.routed",
                "workflow.routed",
                workflow_id,
                json!({
                    "workflow_id": workflow_id,
                    "routing": decision_payload,
                    "active_worker_ids": adaptive_team_plan
                        .as_ref()
                        .map(|team_plan| json!(team_plan.worker_ids))
                        .unwrap_or(Value::Null),
                    "team_sizing": adaptive_team_plan
                        .as_ref()
                        .map(|team_plan| json!(team_plan.sizing_decision))
                        .unwrap_or(Value::Null),
                }),
            )
            .await?;

            let mut prepared_decisions = Vec::new();
            for decision in decisions {
                let worker_id = decision.selected_agent.ok_or_else(|| {
                    format!("branch {} has no selected worker", decision.branch_id)
                })?;
                let mut prepared_tasks = Vec::new();
                for task_id in decision.task_ids {
                    self.orchestrator
                        .scheduler()
                        .start(&task_id)
                        .map_err(|error| {
                            format!("failed to start scheduled task {task_id}: {error}")
                        })?;
                    if let Some(transition) =
                        self.transition_graph(workflow_id, task_id, NodeState::Running)
                    {
                        append_durable_workflow_history_event(
                            &mut metadata,
                            workflow_id,
                            DurableWorkflowEventKind::NodeStateChanged,
                            Some(transition.branch_id),
                            Some(ExecutionNodeId::from_uuid(*task_id.as_ref())),
                            Some(lifecycle_state_for_graph_state(transition.graph_state)),
                            json!({
                                "step_key": transition.step_key,
                                "node_state": transition.node_state,
                                "branch_state": transition.branch_state,
                                "graph_state": transition.graph_state,
                            }),
                        );
                    }

                    let task = self.orchestrator.scheduler().get(&task_id).ok_or_else(|| {
                        format!("scheduled task {task_id} disappeared before execution")
                    })?;
                    let execution_input = execution_input_for_task(
                        &graph,
                        &task,
                        &request.description,
                        worker_id,
                        &step_results,
                        &self.runtime_llm,
                    );
                    prepared_tasks.push(PreparedTaskExecution {
                        task,
                        worker_id,
                        execution_input,
                    });
                }

                if !prepared_tasks.is_empty() {
                    prepared_decisions.push(PreparedDecisionExecution {
                        tasks: prepared_tasks,
                    });
                }
            }

            self.capture_autonomy_status_metadata(workflow_id, &mut metadata, None);
            self.update_task_metadata(workflow_id, metadata.clone())
                .await?;

            let mut join_set = JoinSet::new();
            for prepared in prepared_decisions {
                let service = self.clone();
                join_set.spawn(async move {
                    service
                        .execute_prepared_decision(workflow_id, prepared)
                        .await
                });
            }

            let mut failures = Vec::new();
            while let Some(joined) = join_set.join_next().await {
                match joined {
                    Ok(Ok(completed_steps)) => {
                        for completed_step in completed_steps {
                            self.record_completed_step(
                                workflow_id,
                                coordinator_id,
                                &mut metadata,
                                &mut step_results,
                                completed_step,
                            )
                            .await?;
                        }
                    }
                    Ok(Err(failed_step)) => {
                        let FailedTaskExecution {
                            completed_steps,
                            task_id,
                            worker_id,
                            branch_id,
                            action,
                            message,
                            result,
                            step_evaluation,
                        } = failed_step;
                        for completed_step in completed_steps {
                            self.record_completed_step(
                                workflow_id,
                                coordinator_id,
                                &mut metadata,
                                &mut step_results,
                                completed_step,
                            )
                            .await?;
                        }
                        self.record_failed_step(
                            workflow_id,
                            coordinator_id,
                            &mut metadata,
                            &mut step_results,
                            FailedTaskExecution {
                                completed_steps: Vec::new(),
                                task_id,
                                worker_id,
                                branch_id,
                                action,
                                message: message.clone(),
                                result,
                                step_evaluation,
                            },
                        )
                        .await?;
                        failures.push(message);
                    }
                    Err(error) => {
                        failures.push(format!(
                            "parallel branch execution join failed for workflow {workflow_id}: {error}"
                        ));
                    }
                }
            }

            if let Some(failure) = failures.into_iter().next() {
                return Err(failure);
            }

            graph = self
                .orchestrator
                .execution_graph(&workflow_id)
                .ok_or_else(|| format!("execution graph {workflow_id} missing after routing"))?;
        }

        let aggregated_result = self
            .orchestrator
            .aggregate(&workflow_id)
            .await
            .map_err(|error| format!("result aggregation failed: {error}"))?;
        let final_result = crate::autonomy::build_canonical_result_envelope(
            crate::autonomy::CanonicalResultEnvelopeInput {
                workflow_id,
                provider_kind: self.provider_kind_name(),
                model_id: self.model_id(),
                description: &request.description,
                runtime_execution_mode: self.runtime_execution_mode(),
                planner_output: metadata
                    .get("planner_output")
                    .cloned()
                    .unwrap_or(Value::Null),
                execution_plan: metadata
                    .get("execution_plan")
                    .cloned()
                    .unwrap_or(Value::Null),
                step_results: step_results.values().cloned().collect::<Vec<_>>(),
                aggregated_result: aggregated_result.clone(),
                status: "completed",
            },
        );
        let result_views = terminal_result_views(
            "completed",
            final_result,
            self.current_supervision_evidence(workflow_id, &metadata, None),
            &metadata,
            self.orchestrator.coordinator_runtime_proof_view(&workflow_id),
        )?;

        put_metadata(
            &mut metadata,
            "aggregated_result",
            result_views.aggregated_result.clone(),
        );
        put_metadata(
            &mut metadata,
            "final_result",
            result_views.final_result.clone(),
        );
        self.transition_workflow_complete(workflow_id);
        append_durable_workflow_history_event(
            &mut metadata,
            workflow_id,
            DurableWorkflowEventKind::LifecycleChanged,
            None,
            None,
            Some(DurableWorkflowLifecycleState::Completed),
            json!({
                "graph_state": GraphState::Completed,
            }),
        );
        self.capture_autonomy_status_metadata(
            workflow_id,
            &mut metadata,
            Some(&result_views.task_result),
        );
        self.update_root_record(
            workflow_id,
            "completed",
            metadata.clone(),
            Some(result_views.task_result),
            Some(Utc::now()),
            Some(Utc::now()),
        )
        .await?;
        if let Err(error) = self
            .publish_effect_boundary_event(
                coordinator_id,
                workflow_id,
                &mut metadata,
                "workflow.completed",
                "workflow.completed",
                result_views.final_result,
            )
            .await
        {
            warn!(
                workflow_id = %workflow_id,
                error = %error,
                "effect boundary event publish failed after workflow completion; workflow state preserved"
            );
        }

        info!(
            workflow_id = %workflow_id,
            provider_kind = self.provider_kind_name(),
            model_id = self.model_id(),
            "Workflow completed"
        );
        Ok(())
    }

    async fn execute_plan_with_supervised_planner(
        &self,
        workflow_id: TaskId,
        goal: String,
        context: Value,
    ) -> Result<Value, String> {
        let planner_id = AgentId::new();
        let planner_config = AgentConfig::for_type(AgentType::Planner);
        let planner_timeout = planner_config.task_timeout;
        let router = self.router.clone();
        let orchestrator = self.orchestrator.clone();
        let runtime_llm = self.runtime_llm.clone();

        let runtime = spawn_supervised(
            self.supervised_system.as_ref(),
            self.runtime_supervisor_id,
            move || {
                let supervision = LlmSupervision::new(
                    orchestrator.clone(),
                    workflow_id,
                    LlmSupervisionConfig::new(GuardTarget::Provider(
                        runtime_llm.provider_kind_name.clone(),
                    )),
                );
                (
                    PlannerAgent::with_router_and_supervision(
                        planner_id,
                        router.clone(),
                        supervision,
                    ),
                    PlannerState::default(),
                )
            },
            planner_config,
        )
        .await
        .map_err(|error| {
            format!("failed to spawn supervised planner for {workflow_id}: {error}")
        })?;

        let result = runtime
            .ask(
                PlannerMessage::PlanGoal {
                    goal,
                    context,
                    managed_context: None,
                },
                planner_timeout,
            )
            .await
            .map_err(|error| format!("planner execution failed: {error}"));
        let _ = runtime.stop().await;
        result
    }

    async fn execute_prepared_decision(
        &self,
        workflow_id: TaskId,
        prepared: PreparedDecisionExecution,
    ) -> Result<Vec<CompletedTaskExecution>, FailedTaskExecution> {
        let mut completed_steps = Vec::new();
        let first_task = prepared.tasks.first().map(|task| FailedTaskExecution {
            completed_steps: Vec::new(),
            task_id: task.task.task_id,
            worker_id: task.worker_id,
            branch_id: task
                .task
                .input
                .get("branch_id")
                .cloned()
                .unwrap_or(Value::Null),
            action: task.task.task_type.clone(),
            message: String::new(),
            result: None,
            step_evaluation: None,
        });
        let executor_runtime_id = AgentId::new();
        let executor_config = AgentConfig::for_type(AgentType::Executor);
        let executor_timeout = executor_config.task_timeout;
        let tool_bus = self.tool_bus.clone();
        let runtime = spawn_supervised(
            self.supervised_system.as_ref(),
            self.runtime_supervisor_id,
            move || {
                (
                    ExecutorAgent::with_tool_bus(
                        executor_runtime_id,
                        tool_bus.clone(),
                        WORKFLOW_TOOL_NAMESPACE,
                        WORKFLOW_EXECUTE_STEP_TOOL,
                    ),
                    ExecutorState::default(),
                )
            },
            executor_config,
        )
        .await
        .map_err(|error| {
            let mut failed = first_task.unwrap_or(FailedTaskExecution {
                completed_steps: Vec::new(),
                task_id: TaskId::new(),
                worker_id: AgentId::new(),
                branch_id: Value::Null,
                action: "unknown".to_string(),
                message: String::new(),
                result: None,
                step_evaluation: None,
            });
            failed.message =
                format!("failed to spawn supervised executor for workflow {workflow_id}: {error}");
            failed
        })?;

        for prepared_task in prepared.tasks {
            let task_id = prepared_task.task.task_id;
            let worker_id = prepared_task.worker_id;
            let branch_id = prepared_task
                .task
                .input
                .get("branch_id")
                .cloned()
                .unwrap_or(Value::Null);
            let action = prepared_task.task.task_type.clone();

            info!(
                workflow_id = %workflow_id,
                task_id = %task_id,
                worker_id = %worker_id,
                provider_kind = self.provider_kind_name(),
                model_id = self.model_id(),
                "Executing workflow step"
            );

            let verifier_policies = verifier_policies_from_task_input(&prepared_task.task)
                .map_err(|message| FailedTaskExecution {
                    completed_steps: completed_steps.clone(),
                    task_id,
                    worker_id,
                    branch_id: branch_id.clone(),
                    action: action.clone(),
                    message,
                    result: None,
                    step_evaluation: None,
                })?;
            let mut active_policy_index = 0usize;
            let mut repair_state = LocalRepairState::default();
            let mut execution_input = prepared_task.execution_input;

            loop {
                let result = match runtime
                    .ask(
                        ExecutorMessage::ExecutePlan {
                            plan: execution_input.clone(),
                            managed_context: None,
                        },
                        executor_timeout,
                    )
                    .await
                {
                    Ok(result) => result,
                    Err(error) => {
                        let _ = runtime.stop().await;
                        return Err(FailedTaskExecution {
                            completed_steps,
                            task_id,
                            worker_id,
                            branch_id: branch_id.clone(),
                            action: action.clone(),
                            message: format!("worker {worker_id} failed task {task_id}: {error}"),
                            result: None,
                            step_evaluation: None,
                        });
                    }
                };

                let context = StepExecutionContext {
                    workflow_id,
                    task: &prepared_task.task,
                    worker_id,
                    branch_id: branch_id.clone(),
                    action: action.clone(),
                    repair_state: &repair_state,
                };

                match step_execution_disposition(
                    &context,
                    result,
                    verifier_policies.get(active_policy_index).cloned(),
                )
                .map_err(|failed_step| *failed_step)?
                {
                    StepExecutionDisposition::Completed(completed_step) => {
                        if let Err(error) = self
                            .supervise_completed_runtime_step(
                                workflow_id,
                                &prepared_task.task,
                                &completed_step,
                            )
                            .await
                        {
                            let _ = runtime.stop().await;
                            return Err(FailedTaskExecution {
                                completed_steps,
                                task_id,
                                worker_id,
                                branch_id: branch_id.clone(),
                                action: action.clone(),
                                message: format!(
                                    "runtime supervision failed for completed step {task_id}: {error}"
                                ),
                                result: Some(completed_step.result.clone()),
                                step_evaluation: completed_step.step_evaluation.clone(),
                            });
                        }
                        completed_steps.push(completed_step);
                        break;
                    }
                    StepExecutionDisposition::Failed(mut failed_step) => {
                        if let Err(error) = self
                            .supervise_failed_runtime_step(
                                workflow_id,
                                &prepared_task.task,
                                &failed_step,
                            )
                            .await
                        {
                            if failed_step.message.is_empty() {
                                failed_step.message = error;
                            } else {
                                failed_step.message = format!("{} ({error})", failed_step.message);
                            }
                        }
                        failed_step.completed_steps = completed_steps;
                        let _ = runtime.stop().await;
                        return Err(failed_step);
                    }
                    StepExecutionDisposition::Continue(transition) => {
                        if verifier_policies.get(active_policy_index + 1).is_none() {
                            let supervision_error = self
                                .supervise_runtime_transition(
                                    workflow_id,
                                    &prepared_task.task,
                                    &transition,
                                )
                                .await
                                .err();
                            let mut failed_step = repair_failure(
                                &prepared_task.task,
                                worker_id,
                                branch_id.clone(),
                                action.clone(),
                                transition.result,
                                transition.step_evaluation,
                                Some("no follow-up verifier policy available for local repair"),
                            );
                            if let Some(error) = supervision_error {
                                if failed_step.message.is_empty() {
                                    failed_step.message = error;
                                } else {
                                    failed_step.message =
                                        format!("{} ({error})", failed_step.message);
                                }
                            }
                            failed_step.completed_steps = completed_steps;
                            let _ = runtime.stop().await;
                            return Err(failed_step);
                        }

                        if let Err(error) = self
                            .supervise_runtime_transition(
                                workflow_id,
                                &prepared_task.task,
                                &transition,
                            )
                            .await
                        {
                            let suffix = format!("runtime supervision failed: {error}");
                            let mut failed_step = repair_failure(
                                &prepared_task.task,
                                worker_id,
                                branch_id.clone(),
                                action.clone(),
                                transition.result,
                                transition.step_evaluation,
                                Some(&suffix),
                            );
                            failed_step.completed_steps = completed_steps;
                            let _ = runtime.stop().await;
                            return Err(failed_step);
                        }

                        repair_state = transition.repair_state;
                        active_policy_index += 1;
                        apply_local_repair_context(
                            &mut execution_input,
                            transition.repair_action,
                            &repair_state,
                        );
                    }
                }
            }
        }

        let _ = runtime.stop().await;
        Ok(completed_steps)
    }

    async fn save_root_record(
        &self,
        request: &TaskSubmissionRequest,
        workflow_id: TaskId,
        coordinator_id: AgentId,
    ) -> Result<(), String> {
        let record = TaskRecord {
            task_id: *workflow_id.as_ref(),
            task_type: "workflow".to_string(),
            agent_id: None,
            payload: json!({
                "description": request.description,
                "agent_type": request.agent_type,
                "priority": request.priority,
                "provider_kind": self.provider_kind_name(),
                "model_id": self.model_id(),
                "external_delegation": request.delegation,
            }),
            result: None,
            metadata: initial_metadata(
                request,
                coordinator_id,
                &self.worker_ids,
                "queued",
                &self.runtime_llm,
            ),
            status: "queued".to_string(),
            priority: priority_rank(request.priority.as_deref()),
            correlation_id: Some(*workflow_id.as_ref()),
            parent_task_id: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            expires_at: None,
        };
        self.repository
            .save(&record)
            .await
            .map_err(|error| format!("failed to persist root workflow record: {error}"))?;
        Ok(())
    }

    pub(crate) async fn prepare_workflow(
        &self,
        workflow_id: TaskId,
        request: &TaskSubmissionRequest,
    ) -> Result<(), String> {
        if request.description.trim().is_empty() {
            return Err("task description must not be empty".to_string());
        }

        let coordinator_id = coordinator_id_for_request(request, self.default_coordinator_id);
        self.save_root_record(request, workflow_id, coordinator_id)
            .await
    }

    pub(crate) fn launch_workflow(
        &self,
        workflow_id: TaskId,
        request: TaskSubmissionRequest,
    ) -> Result<(), String> {
        self.spawn_workflow_runner(workflow_id, request)
    }

    pub(crate) async fn apply_task_lifecycle(
        &self,
        workflow_id: TaskId,
        verb: DurableWorkflowLifecycleVerb,
        reason: Option<String>,
    ) -> Result<Option<TaskLifecycleView>, String> {
        let Some(record) = self.find_task_record(workflow_id).await? else {
            return Ok(None);
        };

        let current_state = workflow_lifecycle_state_for_record(&record);
        let decision = lifecycle_transition_decision(current_state, &record.status, verb);
        let decided_at = Utc::now();
        let mut metadata = record.metadata.clone();
        let command_id = Uuid::new_v4();
        let lifecycle_record = LifecycleDecisionRecord {
            workflow_id,
            command_id,
            verb,
            requested_by_agent_id: None,
            source: Some("http_task_lifecycle".to_string()),
            requested_at: decided_at,
            reason,
            outcome: decision.outcome,
            resulting_state: decision.resulting_state,
            decided_at,
            note: decision.note.clone(),
        };

        merge_lifecycle_decision_metadata(&mut metadata, std::slice::from_ref(&lifecycle_record))
            .map_err(|error| format!("failed to merge lifecycle decision metadata: {error}"))?;
        append_durable_workflow_history_event(
            &mut metadata,
            workflow_id,
            DurableWorkflowEventKind::LifecycleChanged,
            None,
            None,
            Some(decision.resulting_state),
            json!({
                "command_id": command_id,
                "verb": verb,
                "outcome": decision.outcome,
                "status": decision.persisted_status,
                "note": decision.note,
            }),
        );
        self.capture_autonomy_status_metadata(workflow_id, &mut metadata, record.result.as_ref());

        let completed_at = if decision.resulting_state.is_terminal() {
            record.completed_at.or(Some(decided_at))
        } else {
            record.completed_at
        };
        // TODO: Replace whole-document write with DB-level JSONB merge to avoid clobbering concurrent updates
        // Current implementation performs read-modify-write which can race with concurrent step/effect updates
        self.update_root_record(
            workflow_id,
            &decision.persisted_status,
            metadata,
            record.result,
            record.started_at,
            completed_at,
        )
        .await?;

        Ok(Some(TaskLifecycleView {
            task_id: workflow_id,
            status: decision.persisted_status,
            lifecycle_state: decision.resulting_state,
            outcome: decision.outcome,
            note: decision.note,
        }))
    }

    pub(crate) async fn delete_workflow_record(&self, workflow_id: TaskId) -> Result<(), String> {
        sqlx::query(
            r#"
            DELETE FROM tasks.records
            WHERE task_id = $1
            "#,
        )
        .bind(*workflow_id.as_ref())
        .execute(&self.pool)
        .await
        .map_err(|error| format!("failed to delete workflow record {workflow_id}: {error}"))?;
        Ok(())
    }

    async fn find_task_record(&self, task_id: TaskId) -> Result<Option<TaskRecord>, String> {
        self.repository
            .find(task_id.as_ref())
            .await
            .map_err(|error| format!("failed to load task record {task_id}: {error}"))
    }

    async fn update_task_metadata(
        &self,
        workflow_id: TaskId,
        metadata: Value,
    ) -> Result<(), String> {
        queries::update_task_metadata(&self.pool, *workflow_id.as_ref(), metadata)
            .await
            .map_err(|error| {
                format!("failed to update workflow metadata {workflow_id}: {error}")
            })?;
        Ok(())
    }

    async fn update_root_record(
        &self,
        workflow_id: TaskId,
        status: &str,
        metadata: Value,
        result: Option<Value>,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        completed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), String> {
        sqlx::query_as::<_, TaskRecord>(
            r#"
            UPDATE tasks.records
            SET status = $2::task_status_type,
                metadata = $3,
                result = $4,
                started_at = COALESCE(started_at, $5),
                completed_at = $6
            WHERE task_id = $1
            RETURNING
                task_id, task_type, agent_id, payload, result,
                metadata, status::TEXT AS status, priority, correlation_id,
                parent_task_id, created_at, started_at,
                completed_at, expires_at
            "#,
        )
        .bind(*workflow_id.as_ref())
        .bind(status)
        .bind(metadata)
        .bind(result)
        .bind(started_at)
        .bind(completed_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|error| format!("failed to update workflow record {workflow_id}: {error}"))?;
        Ok(())
    }

    async fn fail_workflow(&self, workflow_id: TaskId, message: String) {
        let mut metadata = match self.find_task_record(workflow_id).await {
            Ok(Some(record)) => {
                let mut metadata = record.metadata;
                put_metadata(&mut metadata, "failure", json!({ "message": message }));
                metadata
            }
            _ => json!({ "failure": { "message": message } }),
        };
        let final_result = crate::autonomy::build_canonical_result_envelope(
            crate::autonomy::CanonicalResultEnvelopeInput {
                workflow_id,
                provider_kind: self.provider_kind_name(),
                model_id: self.model_id(),
                description: metadata
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or("workflow failed"),
                runtime_execution_mode: self.runtime_execution_mode(),
                planner_output: metadata
                    .get("planner_output")
                    .cloned()
                    .unwrap_or(Value::Null),
                execution_plan: metadata
                    .get("execution_plan")
                    .cloned()
                    .unwrap_or(Value::Null),
                step_results: metadata
                    .get("step_results")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default(),
                aggregated_result: json!({ "error": message }),
                status: "failed",
            },
        );
        let aggregated_result = final_result.aggregated_result.clone();
        let result_views = terminal_result_views(
            "failed",
            final_result,
            self.current_supervision_evidence(workflow_id, &metadata, None),
            &metadata,
            self.orchestrator.coordinator_runtime_proof_view(&workflow_id),
        )
        .unwrap_or(TerminalResultViews {
            aggregated_result: aggregated_result.clone(),
            final_result: Value::Null,
            task_result: Value::Null,
        });
        put_metadata(
            &mut metadata,
            "aggregated_result",
            result_views.aggregated_result.clone(),
        );
        put_metadata(
            &mut metadata,
            "final_result",
            result_views.final_result.clone(),
        );
        self.capture_autonomy_status_metadata(
            workflow_id,
            &mut metadata,
            Some(&result_views.task_result),
        );
        let coordinator_id =
            coordinator_id_from_metadata(&metadata).unwrap_or(self.default_coordinator_id);
        let _ = self
            .update_root_record(
                workflow_id,
                "failed",
                metadata.clone(),
                Some(result_views.task_result),
                Some(Utc::now()),
                Some(Utc::now()),
            )
            .await;
        let _ = self
            .publish_effect_boundary_event(
                coordinator_id,
                workflow_id,
                &mut metadata,
                "workflow.failed",
                "workflow.failed",
                json!({
                    "workflow_id": workflow_id,
                    "provider_kind": self.provider_kind_name(),
                    "model_id": self.model_id(),
                    "error": message,
                }),
            )
            .await;
    }

    async fn publish_event(
        &self,
        source_agent_id: AgentId,
        subject: &str,
        message_type: &str,
        workflow_id: TaskId,
        payload: Value,
    ) -> Result<(), String> {
        let envelope = MessageEnvelope::builder(message_type)
            .correlation_id(*workflow_id.as_ref())
            .source_agent_id(*source_agent_id.as_ref())
            .priority(MessagePriority::Normal)
            .payload_json(&payload)
            .map_err(|error| format!("failed to encode workflow event payload: {error}"))?
            .build()
            .map_err(|error| format!("failed to build workflow event envelope: {error}"))?;

        self.jetstream
            .publish_and_ack(subject, envelope)
            .await
            .map_err(|error| format!("failed to publish JetStream event on {subject}: {error}"))?;

        broadcast_event(
            &self.event_tx,
            WsEvent {
                event_type: subject.to_string(),
                payload,
                timestamp: Utc::now().to_rfc3339(),
            },
        );

        Ok(())
    }

    async fn publish_effect_boundary_event(
        &self,
        source_agent_id: AgentId,
        workflow_id: TaskId,
        metadata: &mut Value,
        subject: &str,
        message_type: &str,
        payload: Value,
    ) -> Result<(), String> {
        if should_skip_effect_boundary_publish(metadata, subject, &payload) {
            return Ok(());
        }

        let idempotency_key = effect_boundary_idempotency_key(subject, &payload);
        let existing = effect_boundary_projection(metadata, subject, &payload);
        let effect_boundary_id = existing
            .as_ref()
            .map(|record| record.effect_boundary_id)
            .unwrap_or_else(|| effect_boundary_uuid(workflow_id, &idempotency_key));
        let intent_recorded_at = existing
            .as_ref()
            .map(|record| record.intent_recorded_at)
            .unwrap_or_else(Utc::now);

        if existing.is_none() {
            merge_effect_boundary_record(
                metadata,
                EffectBoundaryRecord {
                    workflow_id,
                    effect_boundary_id,
                    idempotency_key: idempotency_key.clone(),
                    intent_state: EffectBoundaryIntentState::Recorded,
                    outcome_state: EffectBoundaryOutcomeState::CompletionUnknown,
                    intent_recorded_at,
                    outcome_recorded_at: None,
                    history_event_id: None,
                    note: Some(format!(
                        "effect intent recorded for {subject}; awaiting durable completion"
                    )),
                },
            )?;
            append_effect_boundary_history_event(
                metadata,
                workflow_id,
                DurableWorkflowEventKind::EffectIntentRecorded,
                effect_boundary_id,
                json!({
                    "subject": subject,
                    "message_type": message_type,
                    "idempotency_key": idempotency_key,
                    "intent_state": EffectBoundaryIntentState::Recorded,
                    "outcome_state": EffectBoundaryOutcomeState::CompletionUnknown,
                }),
            );
            self.update_task_metadata(workflow_id, metadata.clone())
                .await?;
        }

        match self
            .publish_event(source_agent_id, subject, message_type, workflow_id, payload)
            .await
        {
            Ok(()) => {
                merge_effect_boundary_record(
                    metadata,
                    EffectBoundaryRecord {
                        workflow_id,
                        effect_boundary_id,
                        idempotency_key: idempotency_key.clone(),
                        intent_state: EffectBoundaryIntentState::Recorded,
                        outcome_state: EffectBoundaryOutcomeState::Completed,
                        intent_recorded_at,
                        outcome_recorded_at: Some(Utc::now()),
                        history_event_id: None,
                        note: Some(format!("effect completion recorded for {subject}")),
                    },
                )?;
                append_effect_boundary_history_event(
                    metadata,
                    workflow_id,
                    DurableWorkflowEventKind::EffectOutcomeRecorded,
                    effect_boundary_id,
                    json!({
                        "subject": subject,
                        "message_type": message_type,
                        "idempotency_key": idempotency_key,
                        "outcome_state": EffectBoundaryOutcomeState::Completed,
                    }),
                );
                self.update_task_metadata(workflow_id, metadata.clone())
                    .await?;
                Ok(())
            }
            Err(error) => {
                merge_effect_boundary_record(
                    metadata,
                    EffectBoundaryRecord {
                        workflow_id,
                        effect_boundary_id,
                        idempotency_key,
                        intent_state: EffectBoundaryIntentState::Recorded,
                        outcome_state: EffectBoundaryOutcomeState::CompletionUnknown,
                        intent_recorded_at,
                        outcome_recorded_at: None,
                        history_event_id: None,
                        note: Some(format!(
                            "effect completion still unknown for {subject}: {error}"
                        )),
                    },
                )?;
                self.update_task_metadata(workflow_id, metadata.clone())
                    .await?;
                Err(error)
            }
        }
    }

    async fn record_completed_step(
        &self,
        workflow_id: TaskId,
        coordinator_id: AgentId,
        metadata: &mut Value,
        step_results: &mut BTreeMap<String, Value>,
        completed_step: CompletedTaskExecution,
    ) -> Result<(), String> {
        self.orchestrator
            .scheduler()
            .complete(&completed_step.task_id, completed_step.result.clone())
            .map_err(|error| {
                format!(
                    "failed to complete scheduled task {}: {error}",
                    completed_step.task_id
                )
            })?;
        if let Some(transition) =
            self.transition_graph(workflow_id, completed_step.task_id, NodeState::Completed)
        {
            append_durable_workflow_history_event(
                metadata,
                workflow_id,
                DurableWorkflowEventKind::NodeStateChanged,
                Some(transition.branch_id),
                Some(ExecutionNodeId::from_uuid(*completed_step.task_id.as_ref())),
                Some(lifecycle_state_for_graph_state(transition.graph_state)),
                json!({
                    "step_key": transition.step_key,
                    "node_state": transition.node_state,
                    "branch_state": transition.branch_state,
                    "graph_state": transition.graph_state,
                }),
            );
        }

        let step_result = build_step_result_payload(
            completed_step.task_id,
            completed_step.worker_id,
            completed_step.branch_id.clone(),
            completed_step.action.clone(),
            Some(completed_step.result.clone()),
            completed_step.step_evaluation.clone(),
        );
        step_results.insert(completed_step.task_id.to_string(), step_result.clone());
        put_metadata(
            metadata,
            "step_results",
            json!(step_results.values().cloned().collect::<Vec<_>>()),
        );
        self.capture_autonomy_status_metadata(workflow_id, metadata, None);
        self.update_task_metadata(workflow_id, metadata.clone())
            .await?;
        if let Err(error) = self
            .publish_effect_boundary_event(
                coordinator_id,
                workflow_id,
                metadata,
                "workflow.step.completed",
                "workflow.step.completed",
                step_result,
            )
            .await
        {
            warn!(
                workflow_id = %workflow_id,
                error = %error,
                "effect boundary event publish failed after step completion; step state preserved"
            );
        }
        Ok(())
    }

    async fn record_failed_step(
        &self,
        workflow_id: TaskId,
        coordinator_id: AgentId,
        metadata: &mut Value,
        step_results: &mut BTreeMap<String, Value>,
        failed_step: FailedTaskExecution,
    ) -> Result<(), String> {
        if failed_step.step_evaluation.is_some() {
            let step_result = build_step_result_payload(
                failed_step.task_id,
                failed_step.worker_id,
                failed_step.branch_id.clone(),
                failed_step.action.clone(),
                failed_step.result.clone(),
                failed_step.step_evaluation.clone(),
            );
            step_results.insert(failed_step.task_id.to_string(), step_result);
            put_metadata(
                metadata,
                "step_results",
                json!(step_results.values().cloned().collect::<Vec<_>>()),
            );
        }

        self.orchestrator
            .scheduler()
            .fail(&failed_step.task_id, failed_step.message.clone())
            .map_err(|error| {
                format!(
                    "failed to mark scheduled task {} as failed: {error}",
                    failed_step.task_id
                )
            })?;
        if let Some(transition) =
            self.transition_graph(workflow_id, failed_step.task_id, NodeState::Failed)
        {
            append_durable_workflow_history_event(
                metadata,
                workflow_id,
                DurableWorkflowEventKind::NodeStateChanged,
                Some(transition.branch_id),
                Some(ExecutionNodeId::from_uuid(*failed_step.task_id.as_ref())),
                Some(lifecycle_state_for_graph_state(transition.graph_state)),
                json!({
                    "step_key": transition.step_key,
                    "node_state": transition.node_state,
                    "branch_state": transition.branch_state,
                    "graph_state": transition.graph_state,
                }),
            );
        }
        put_metadata(
            metadata,
            "last_step_failure",
            json!(failed_step.message.clone()),
        );
        self.capture_autonomy_status_metadata(workflow_id, metadata, None);
        self.update_task_metadata(workflow_id, metadata.clone())
            .await?;
        self.publish_event(
            coordinator_id,
            "workflow.step.failed",
            "workflow.step.failed",
            workflow_id,
            json!({
                "task_id": failed_step.task_id,
                "worker_id": failed_step.worker_id,
                "branch_id": failed_step.branch_id,
                "action": failed_step.action,
                "error": failed_step.message,
                "step_evaluation": failed_step.step_evaluation,
            }),
        )
        .await
    }

    fn transition_graph(
        &self,
        workflow_id: TaskId,
        task_id: TaskId,
        node_state: NodeState,
    ) -> Option<GraphTransitionSnapshot> {
        let mut graph = self.orchestrator.execution_graph(&workflow_id)?;

        let node_id = mister_smith_core::ExecutionNodeId::from_uuid(*task_id.as_ref());
        let transition = graph
            .nodes
            .iter_mut()
            .find(|node| node.node_id == node_id)
            .map(|node| {
                node.state = node_state;
                (node.step_key.clone(), node.branch_id)
            });

        let (step_key, branch_id) = transition?;

        if let Some(branch) = graph
            .branches
            .iter_mut()
            .find(|branch| branch.branch_id == branch_id)
        {
            let branch_nodes = graph
                .nodes
                .iter()
                .filter(|node| node.branch_id == branch_id)
                .collect::<Vec<_>>();
            branch.state = if branch_nodes
                .iter()
                .any(|node| node.state == NodeState::Failed)
            {
                BranchState::Failed
            } else if branch_nodes
                .iter()
                .all(|node| node.state == NodeState::Completed)
            {
                BranchState::Completed
            } else {
                BranchState::Running
            };
        }

        graph.state = if graph
            .nodes
            .iter()
            .any(|node| node.state == NodeState::Failed)
        {
            GraphState::Failed
        } else if graph
            .nodes
            .iter()
            .all(|node| node.state == NodeState::Completed)
        {
            GraphState::Completed
        } else {
            GraphState::Running
        };

        let snapshot = GraphTransitionSnapshot {
            step_key,
            branch_id,
            node_state,
            branch_state: graph
                .branches
                .iter()
                .find(|branch| branch.branch_id == branch_id)
                .map_or(BranchState::Pending, |branch| branch.state),
            graph_state: graph.state,
        };
        self.orchestrator.register_execution_graph(graph);
        Some(snapshot)
    }

    fn transition_workflow_complete(&self, workflow_id: TaskId) {
        let Some(mut graph) = self.orchestrator.execution_graph(&workflow_id) else {
            return;
        };

        for node in &mut graph.nodes {
            node.state = NodeState::Completed;
        }
        for branch in &mut graph.branches {
            branch.state = BranchState::Completed;
        }
        graph.state = GraphState::Completed;
        self.orchestrator.register_execution_graph(graph);
    }

    fn capture_autonomy_status_metadata(
        &self,
        workflow_id: TaskId,
        metadata: &mut Value,
        task_result: Option<&Value>,
    ) {
        let Some(mut view) = self.orchestrator.autonomy_status(&workflow_id).or_else(|| {
            persisted_or_synthesized_autonomy_status(workflow_id, metadata, task_result)
        }) else {
            return;
        };
        crate::autonomy::enrich_lifecycle_state(&mut view, metadata);
        crate::autonomy::enrich_session_linkage(&mut view, metadata);
        crate::autonomy::enrich_accepted_task_ingress_continuity(&mut view, metadata);
        crate::autonomy::enrich_step_routing_history(&mut view, metadata);
        crate::autonomy::enrich_result_preview(&mut view, metadata, task_result);
        persist_autonomy_status(metadata, &view);
    }

    async fn record_runtime_supervision(
        &self,
        workflow_id: TaskId,
        plan: Option<RuntimeSupervisionPlan>,
    ) -> Result<(), String> {
        let Some(plan) = plan else {
            return Ok(());
        };
        let RuntimeSupervisionPlan {
            target,
            assessment,
            checkpoints,
        } = plan;
        let current_fingerprint = self.load_current_runtime_fingerprint(&target).await?;
        let assessment = match current_fingerprint.as_ref() {
            Some(fingerprint) => assessment.with_fingerprint(fingerprint.clone()),
            None => assessment,
        };
        let snapshot = assessment.snapshot().cloned();

        let (decision, _record) = self
            .orchestrator
            .supervise(
                &workflow_id,
                GuardContext::new(target.clone())
                    .with_profile(assessment)
                    .with_checkpoints(checkpoints),
            )
            .await
            .map_err(|error| {
                format!("runtime supervision failed for workflow {workflow_id}: {error}")
            })?;
        if let Some(snapshot) = snapshot.as_ref() {
            let fingerprint = build_runtime_profile_fingerprint(
                workflow_id,
                &target,
                snapshot,
                &decision,
                current_fingerprint.as_ref(),
            );
            self.persist_runtime_profile_fingerprint(&fingerprint)
                .await?;
        }

        Ok(())
    }

    async fn supervise_runtime_transition(
        &self,
        workflow_id: TaskId,
        task: &TaskAssignment,
        transition: &LocalRepairTransition,
    ) -> Result<(), String> {
        let graph = self
            .orchestrator
            .execution_graph(&workflow_id)
            .ok_or_else(|| format!("execution graph missing for workflow {workflow_id}"))?;
        let plan = runtime_supervision_plan_for_repair_evaluation(
            &graph,
            task,
            &transition.step_evaluation,
        )?;
        self.record_runtime_supervision(workflow_id, plan).await
    }

    async fn supervise_completed_runtime_step(
        &self,
        workflow_id: TaskId,
        task: &TaskAssignment,
        completed_step: &CompletedTaskExecution,
    ) -> Result<(), String> {
        let Some(step_evaluation) = completed_step.step_evaluation.as_ref() else {
            return Ok(());
        };
        let graph = self
            .orchestrator
            .execution_graph(&workflow_id)
            .ok_or_else(|| format!("execution graph missing for workflow {workflow_id}"))?;
        let plan = runtime_supervision_plan_for_repair_evaluation(&graph, task, step_evaluation)?;
        self.record_runtime_supervision(workflow_id, plan).await
    }

    async fn supervise_failed_runtime_step(
        &self,
        workflow_id: TaskId,
        task: &TaskAssignment,
        failed_step: &FailedTaskExecution,
    ) -> Result<(), String> {
        let graph = self
            .orchestrator
            .execution_graph(&workflow_id)
            .ok_or_else(|| format!("execution graph missing for workflow {workflow_id}"))?;
        let plan = runtime_supervision_plan_for_failed_step(&graph, task, failed_step)?;
        self.record_runtime_supervision(workflow_id, plan).await
    }

    async fn load_current_runtime_fingerprint(
        &self,
        target: &GuardTarget,
    ) -> Result<Option<ProfileFingerprint>, String> {
        let target_kind = runtime_fingerprint_target_kind(target);
        let target_selector = runtime_fingerprint_target_selector(target);
        runtime_profile_fingerprint_store(self.jetstream.as_ref())
            .await?
            .current(target_kind, &target_selector)
            .await
            .map_err(|error| {
                format!(
                    "runtime fingerprint lookup failed for workflow target {target_kind}:{target_selector}: {error}"
                )
            })
    }

    async fn persist_runtime_profile_fingerprint(
        &self,
        fingerprint: &ProfileFingerprint,
    ) -> Result<(), String> {
        runtime_profile_fingerprint_store(self.jetstream.as_ref())
            .await?
            .save(fingerprint)
            .await
            .map(|_| ())
            .map_err(|error| {
                format!(
                    "runtime fingerprint persistence failed for {}:{}: {error}",
                    fingerprint.target_kind, fingerprint.target_selector
                )
            })
    }

    fn current_supervision_evidence(
        &self,
        workflow_id: TaskId,
        metadata: &Value,
        task_result: Option<&Value>,
    ) -> Option<SupervisionEvidenceView> {
        self.orchestrator
            .autonomy_status(&workflow_id)
            .or_else(|| {
                persisted_or_synthesized_autonomy_status(workflow_id, metadata, task_result)
            })
            .and_then(|view| view.supervision_evidence)
    }

    fn spawn_workflow_runner(
        &self,
        workflow_id: TaskId,
        request: TaskSubmissionRequest,
    ) -> Result<(), String> {
        let service = self.clone();
        tokio::spawn(async move {
            if let Err(error) = service.run_workflow(workflow_id, request).await {
                error!(workflow_id = %workflow_id, error = %error, "Workflow run failed");
                service.fail_workflow(workflow_id, error).await;
            }
        });

        Ok(())
    }
}

#[async_trait]
impl TaskExecutionService for RuntimeTaskService {
    async fn submit_task(
        &self,
        request: TaskSubmissionRequest,
    ) -> Result<TaskSubmissionResponse, String> {
        let workflow_id = TaskId::new();
        self.prepare_workflow(workflow_id, &request).await?;
        self.launch_workflow(workflow_id, request.clone())?;

        let coordinator_id = coordinator_id_for_request(&request, self.default_coordinator_id);

        Ok(TaskSubmissionResponse {
            task_id: workflow_id,
            assigned_agent_id: coordinator_id,
            status: "queued".to_string(),
        })
    }

    async fn get_task(&self, task_id: TaskId) -> Result<Option<TaskStatusView>, String> {
        let record = self.find_task_record(task_id).await?;
        Ok(record.map(|record| TaskStatusView {
            task_id: TaskId::from_uuid(record.task_id),
            status: record.status.clone(),
            lifecycle_state: workflow_lifecycle_state_for_record(&record),
            result: record.result,
        }))
    }

    async fn apply_task_lifecycle(
        &self,
        task_id: TaskId,
        verb: DurableWorkflowLifecycleVerb,
        reason: Option<String>,
    ) -> Result<Option<TaskLifecycleView>, String> {
        RuntimeTaskService::apply_task_lifecycle(self, task_id, verb, reason).await
    }

    async fn list_tasks(&self, request: TaskListRequest) -> Result<Vec<TaskSummaryView>, String> {
        let limit = i64::try_from(request.limit)
            .map_err(|_| format!("invalid task list limit {}", request.limit))?;
        let offset = i64::try_from(request.offset)
            .map_err(|_| format!("invalid task list offset {}", request.offset))?;
        let records = self
            .repository
            .list_root_workflows(request.status.as_deref(), limit, offset)
            .await
            .map_err(|error| format!("failed to list root workflows: {error}"))?;

        Ok(records
            .into_iter()
            .map(|record| build_task_summary_view(&record))
            .collect())
    }
}

pub(crate) fn build_task_summary_view(record: &TaskRecord) -> TaskSummaryView {
    let result_preview =
        recover_persisted_autonomy_status(record).and_then(|view| view.result_preview);
    let proof_outcome = result_preview
        .as_ref()
        .map(|preview| preview.proof_outcome)
        .or_else(|| task_proof_outcome(record.result.as_ref()));

    TaskSummaryView {
        task_id: TaskId::from_uuid(record.task_id),
        status: record.status.clone(),
        lifecycle_state: workflow_lifecycle_state_for_record(record),
        priority: record.priority,
        description: workflow_description(record),
        created_at: record.created_at,
        started_at: record.started_at,
        completed_at: record.completed_at,
        session_id: metadata_session_id(&record.metadata),
        turn_index: metadata_turn_index(&record.metadata),
        proof_outcome,
        result_preview,
    }
}

fn workflow_description(record: &TaskRecord) -> String {
    record
        .metadata
        .get("description")
        .and_then(Value::as_str)
        .or_else(|| record.payload.get("description").and_then(Value::as_str))
        .unwrap_or("workflow")
        .to_string()
}

fn metadata_session_id(metadata: &Value) -> Option<mister_smith_core::SessionId> {
    metadata
        .get("session_id")
        .and_then(Value::as_str)
        .and_then(|raw| Uuid::parse_str(raw).ok())
        .map(mister_smith_core::SessionId::from_uuid)
}

fn metadata_turn_index(metadata: &Value) -> Option<u32> {
    metadata
        .get("turn_index")
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
}

fn task_proof_outcome(
    result: Option<&Value>,
) -> Option<mister_smith_core::ProofOutcomeClassification> {
    result
        .and_then(|value| value.get("proof_outcome").cloned())
        .and_then(|raw| serde_json::from_value(raw).ok())
}

fn initial_metadata(
    request: &TaskSubmissionRequest,
    coordinator_id: AgentId,
    worker_ids: &[AgentId],
    status: &str,
    runtime_llm: &RuntimeLlmSelection,
) -> Value {
    let mut metadata = json!({
        "provider_kind": runtime_llm.provider_kind_name,
        "model_id": runtime_llm.model_id,
        "submission_path": if request.conversation.is_some() { "session" } else { "http" },
        "status": status,
        "description": request.description,
        "requested_agent_type": request.agent_type,
        "requested_priority": request.priority,
        "coordinator_agent_id": coordinator_id,
        "worker_agent_ids": worker_ids,
        "active_worker_ids": [],
        "planner_output": Value::Null,
        "execution_plan": Value::Null,
        "routing_history": [],
        "step_routing_history": [],
        "team_sizing": Value::Null,
        "step_results": [],
    });

    if let Some(conversation) = &request.conversation {
        put_metadata(&mut metadata, "session_id", json!(conversation.session_id));
        put_metadata(&mut metadata, "turn_index", json!(conversation.turn_index));
        put_metadata(
            &mut metadata,
            "retained_context",
            conversation.retained_context.clone(),
        );
    }

    if let Some(delegation) = &request.delegation {
        put_metadata(
            &mut metadata,
            "external_delegation",
            serde_json::to_value(delegation).unwrap_or(Value::Null),
        );

        if request.conversation.is_none() {
            put_metadata(
                &mut metadata,
                ACCEPTED_TASK_INGRESS_METADATA_KEY,
                accepted_task_ingress_metadata(delegation),
            );
        }
    }

    metadata
}

fn accepted_task_ingress_metadata(delegation: &ExternalDelegationEnvelope) -> Value {
    let action = delegation.action.as_ref();

    json!({
        "request_surface": TASK_INGRESS_REQUEST_SURFACE,
        "source_metadata_key": "external_delegation",
        "capability_id": delegation.capability.capability_id,
        "capability_descriptor_id": delegation.capability.descriptor_id,
        "action_descriptor_id": action.map(|action| action.descriptor_id.clone()),
        "action_id": action.map(|action| action.action_id.clone()),
        "action_title": action.map(|action| action.title.clone()),
        "scope": delegation.capability.scope,
        "required_scope": action.and_then(|action| action.required_scope),
        "policy_action": action.map(|action| action.policy.action.clone()),
        "policy_resource": action.map(|action| action.policy.resource.clone()),
        "policy_scope": action.map(|action| action.policy.scope.clone()),
        "policy_resource_id": action.and_then(|action| action.policy.resource_id.clone()),
        "revocation_state": delegation.capability.revocation_state,
        "chain_depth": delegation.provenance.links.len(),
    })
}

fn persist_autonomy_status(metadata: &mut Value, view: &AutonomyStatusView) {
    let Ok(value) = serde_json::to_value(view) else {
        return;
    };
    put_metadata(metadata, AUTONOMY_STATUS_METADATA_KEY, value);
}

fn persisted_or_synthesized_autonomy_status(
    workflow_id: TaskId,
    metadata: &Value,
    task_result: Option<&Value>,
) -> Option<AutonomyStatusView> {
    metadata
        .get(AUTONOMY_STATUS_METADATA_KEY)
        .cloned()
        .and_then(|raw| serde_json::from_value::<AutonomyStatusView>(raw).ok())
        .or_else(|| {
            crate::autonomy::synthesize_failed_before_graph_status(
                workflow_id,
                metadata,
                task_result,
            )
        })
}

fn recover_persisted_autonomy_status(record: &TaskRecord) -> Option<AutonomyStatusView> {
    let mut view = persisted_or_synthesized_autonomy_status(
        TaskId::from_uuid(record.task_id),
        &record.metadata,
        record.result.as_ref(),
    )?;
    crate::autonomy::enrich_lifecycle_state(&mut view, &record.metadata);
    crate::autonomy::enrich_session_linkage(&mut view, &record.metadata);
    crate::autonomy::enrich_accepted_task_ingress_continuity(&mut view, &record.metadata);
    crate::autonomy::enrich_step_routing_history(&mut view, &record.metadata);
    crate::autonomy::enrich_result_preview(&mut view, &record.metadata, record.result.as_ref());
    Some(view)
}

fn mark_persisted_autonomy_status_failed(metadata: &mut Value) {
    let Some(raw) = metadata.get(AUTONOMY_STATUS_METADATA_KEY).cloned() else {
        return;
    };
    let Ok(mut view) = serde_json::from_value::<AutonomyStatusView>(raw) else {
        return;
    };

    view.lifecycle_state = Some(DurableWorkflowLifecycleState::Failed);
    view.graph.state = GraphState::Failed;
    for branch in &mut view.branches {
        if !matches!(branch.state, BranchState::Completed | BranchState::Failed) {
            branch.state = BranchState::Failed;
        }
    }

    persist_autonomy_status(metadata, &view);
}

fn put_metadata(metadata: &mut Value, key: &str, value: Value) {
    if let Some(object) = metadata.as_object_mut() {
        object.insert(key.to_string(), value);
    }
}

fn priority_rank(priority: Option<&str>) -> i32 {
    match priority
        .unwrap_or("normal")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "0" | "urgent" => 0,
        "1" | "high" => 1,
        "2" | "normal" | "medium" => 2,
        "3" | "low" => 3,
        "4" | "background" => 4,
        _ => 2,
    }
}

fn should_recover_orphaned_workflow(
    record: &TaskRecord,
    boot_started_at: chrono::DateTime<Utc>,
    has_in_memory_graph: bool,
) -> bool {
    if has_in_memory_graph || !matches_inflight_status(&record.status) {
        return false;
    }

    record.started_at.unwrap_or(record.created_at) < boot_started_at
}

fn matches_inflight_status(status: &str) -> bool {
    matches!(
        status.trim().to_ascii_lowercase().as_str(),
        "queued" | "running"
    )
}

fn coordinator_id_for_request(request: &TaskSubmissionRequest, fallback: AgentId) -> AgentId {
    request
        .conversation
        .as_ref()
        .map(|conversation| conversation.coordinator_agent_id)
        .unwrap_or(fallback)
}

fn coordinator_id_from_metadata(metadata: &Value) -> Option<AgentId> {
    metadata
        .get("coordinator_agent_id")
        .and_then(Value::as_str)
        .and_then(|raw| Uuid::parse_str(raw).ok())
        .map(AgentId::from_uuid)
}

fn planner_output_supports_explicit_runtime_graph(steps: &[Value]) -> bool {
    steps.len() > 1
        && steps.iter().any(|raw_step| {
            raw_step.as_object().is_some_and(|step| {
                step.get("branch").and_then(Value::as_str).is_some()
                    || step
                        .get("depends_on")
                        .and_then(Value::as_array)
                        .map(|dependencies| !dependencies.is_empty())
                        .unwrap_or(false)
            })
        })
}

fn next_root_branch_label(index: usize) -> String {
    match index {
        1 => "branch-a".to_string(),
        2 => "branch-b".to_string(),
        _ => format!("branch-{index}"),
    }
}

fn next_join_branch_label(index: usize) -> String {
    match index {
        1 => "join".to_string(),
        _ => format!("join-{index}"),
    }
}

fn canonicalize_topology_hint(raw_hint: &str) -> String {
    let lowered = raw_hint.trim().to_ascii_lowercase();
    if lowered.starts_with("sequential") {
        "sequential".to_string()
    } else if lowered.starts_with("parallel") {
        "parallel".to_string()
    } else if lowered.starts_with("pipeline") {
        "pipeline".to_string()
    } else if lowered.starts_with("hierarchical") {
        "hierarchical".to_string()
    } else if lowered.starts_with("hybrid") {
        "hybrid".to_string()
    } else {
        raw_hint.to_string()
    }
}

fn compile_execution_plan(
    workflow_id: TaskId,
    execution_plan: &Value,
) -> Result<mister_smith_agents::ExecutionGraph, String> {
    let compiler = TopologyCompiler;
    compiler.compile(workflow_id, execution_plan, &TopologySignals::default())
        .map_err(|error| {
            format!(
                "workflow planning produced an invalid execution graph during topology compilation: {error}"
            )
        })
}

fn normalize_explicit_runtime_steps(goal: &str, steps: &[Value]) -> Vec<Value> {
    let mut runtime_steps = Vec::new();
    let mut branch_by_step_id = BTreeMap::<String, String>::new();
    let mut root_branch_index = 1usize;
    let mut join_branch_index = 1usize;

    for (index, raw_step) in steps.iter().enumerate() {
        let mut step = raw_step.as_object().cloned().unwrap_or_default();
        let step_id = step
            .get("id")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("step-{}", index + 1));
        let dependencies = step
            .get("depends_on")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        step.insert("id".to_string(), json!(step_id.clone()));
        step.insert("step".to_string(), json!(index + 1));
        step.entry("action".to_string())
            .or_insert_with(|| json!("execute"));
        step.entry("description".to_string())
            .or_insert_with(|| json!(goal));
        step.insert("depends_on".to_string(), Value::Array(dependencies.clone()));
        let is_merge_node = dependencies.len() > 1;

        let branch = if is_merge_node {
            let label = next_join_branch_label(join_branch_index);
            join_branch_index += 1;
            label
        } else {
            step.get("branch")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| {
                    if let Some(parent_step_id) = dependencies.first().and_then(Value::as_str) {
                        branch_by_step_id
                            .get(parent_step_id)
                            .cloned()
                            .unwrap_or_else(|| {
                                let label = next_root_branch_label(root_branch_index);
                                root_branch_index += 1;
                                label
                            })
                    } else {
                        let label = next_root_branch_label(root_branch_index);
                        root_branch_index += 1;
                        label
                    }
                })
        };

        if is_merge_node {
            step.insert("role".to_string(), json!("coordinator"));
        } else {
            step.entry("role".to_string())
                .or_insert_with(|| json!("worker"));
        }
        step.insert("branch".to_string(), json!(branch.clone()));
        branch_by_step_id.insert(step_id, branch);
        runtime_steps.push(Value::Object(step));
    }

    runtime_steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use mister_smith_core::{
        AuthorityPrincipal, BranchRecoveryStrategy, BranchState, CapabilityActionKind,
        CapabilityId, CoordinationPolicy, DelegatedAction, DelegatedActionPolicy, DelegationScope,
        ExecutionBranchId, ExecutionGraphId, ExternalDelegationEnvelope, GraphState,
        ProofOutcomeClassification, RevocationState, SessionId, TaskShapeClassification,
        TaskShapeKind, TopologyKind, TopologyRationale,
    };
    use mister_smith_events::{
        BranchSummary, ExecutionGraphSummary, ExternalCapabilityDecisionOutcome,
        ExternalCapabilityDecisionSummary, StepRoutingDecisionSummary, TopologyPlanSummary,
    };
    use mister_smith_security::DelegationService;

    fn sample_runtime_llm_selection(
        provider_kind: ProviderKind,
        model_id: &str,
    ) -> RuntimeLlmSelection {
        RuntimeLlmSelection {
            provider_kind,
            provider_kind_name: provider_kind.as_str().to_string(),
            model_id: model_id.to_string(),
        }
    }

    fn sample_autonomy_view() -> AutonomyStatusView {
        sample_autonomy_view_with_states(GraphState::Completed, BranchState::Completed)
    }

    fn sample_autonomy_view_with_states(
        graph_state: GraphState,
        branch_state: BranchState,
    ) -> AutonomyStatusView {
        let workflow_id = TaskId::new();
        let graph_id = ExecutionGraphId::new();
        let branch_id = ExecutionBranchId::new();
        AutonomyStatusView {
            session_id: None,
            turn_index: None,
            coordinator_agent_id: None,
            resume_provenance: None,
            lifecycle_state: Some(lifecycle_state_for_graph_state(graph_state)),
            graph: ExecutionGraphSummary {
                graph_id,
                workflow_id,
                state: graph_state,
                branch_count: 1,
                node_count: 3,
                active_topology: Some(TopologyKind::Sequential),
            },
            topology: TopologyPlanSummary {
                graph_id,
                topology_kind: TopologyKind::Sequential,
                parallelism_width: 1,
                task_shape: TaskShapeClassification {
                    kind: TaskShapeKind::StrictChain,
                    root_count: 1,
                    max_parallel_width: 1,
                    max_depth: 2,
                    has_join: false,
                    has_fanout: false,
                    structural_signals: vec!["roots:1".to_string()],
                },
                coordination_policy: CoordinationPolicy::Barrier,
                rationale: TopologyRationale {
                    dependency_shape: "single branch".to_string(),
                    operational_signals: vec!["restart-safe".to_string()],
                    selected_for: "preserve recovery context".to_string(),
                    fallback_reason: None,
                },
                fallback_topology: Some(TopologyKind::Sequential),
            },
            team_sizing: None,
            branches: vec![BranchSummary {
                branch_id,
                graph_id,
                state: branch_state,
                assigned_agents: vec![],
                checkpoint_id: None,
                recovery_strategy: BranchRecoveryStrategy::Resume,
            }],
            checkpoint_lineage: vec![],
            memory_pressure: vec![],
            routing_history: vec![],
            step_routing_history: vec![],
            result_preview: None,
            interventions: vec![],
            delegation_capabilities: vec![],
            delegation_alerts: vec![],
            delegation_records: vec![],
            subordinate_inbox: vec![],
            subagent_states: vec![],
            delegated_work_evidence: vec![],
            coordinator_decisions: vec![],
            coordinator_runtime_proof: None,
            external_capability_decisions: vec![],
            profiles: vec![],
            guard_decisions: vec![],
            runtime_truth: None,
            supervision_evidence: None,
            step_policy: None,
            conservative_reasons: vec!["restart-safe recovery".to_string()],
        }
    }

    fn sample_external_delegation() -> ExternalDelegationEnvelope {
        let service = DelegationService::new();
        let recipient = AgentId::from_uuid(uuid::Uuid::new_v4());
        let (capability, provenance) = service
            .issue_capability(
                AuthorityPrincipal::Policy("operator".to_string()),
                recipient,
                DelegationScope::InvokeTool,
                Some("tool:app.workflow".to_string()),
                std::time::Duration::from_secs(300),
                None,
                None,
            )
            .expect("delegation should issue");

        ExternalDelegationEnvelope::new(capability, provenance).with_action(DelegatedAction {
            descriptor_id: "tool:app.workflow".to_string(),
            action_id: "tool:app.workflow#execute".to_string(),
            title: "execute app.workflow".to_string(),
            description: "execute access for app.workflow".to_string(),
            kind: CapabilityActionKind::Execute,
            policy: DelegatedActionPolicy {
                action: "execute".to_string(),
                resource: "workflow".to_string(),
                scope: "app".to_string(),
                resource_id: Some("workflow.submit".to_string()),
            },
            required_scope: Some(DelegationScope::InvokeTool),
            revocation_key: "tool:app.workflow#execute".to_string(),
        })
    }

    fn sample_task_with_metadata(metadata: Value) -> TaskRecord {
        TaskRecord {
            task_id: Uuid::new_v4(),
            task_type: "workflow".to_string(),
            agent_id: None,
            payload: json!({ "description": "resume session" }),
            result: Some(json!({ "status": "completed" })),
            metadata,
            status: "completed".to_string(),
            priority: 1,
            correlation_id: None,
            parent_task_id: None,
            created_at: Utc::now(),
            started_at: Some(Utc::now()),
            completed_at: Some(Utc::now()),
            expires_at: None,
        }
    }

    fn sample_task_with_result(metadata: Value, result: Value) -> TaskRecord {
        let mut record = sample_task_with_metadata(metadata);
        record.result = Some(result);
        record
    }

    fn sample_step_policy_routing(
        step_id: &str,
        action_changed: bool,
        confidence_score: Option<f32>,
        triggered_checkpoints: &[&str],
    ) -> StepRoutingDecisionSummary {
        StepRoutingDecisionSummary {
            step_id: step_id.to_string(),
            step_index: Some(1),
            step_kind: Some("planner".to_string()),
            model_id: MODEL_ID.to_string(),
            tier: "llm-tier".to_string(),
            reason: "deterministic routing summary".to_string(),
            previous_step_id: None,
            previous_action: None,
            previous_tier: None,
            action: "continue".to_string(),
            action_changed,
            preferred_tier_after: Some("llm-tier".to_string()),
            estimated_cost_tokens: Some(128),
            confidence_score,
            triggered_checkpoints: triggered_checkpoints
                .iter()
                .map(|checkpoint| checkpoint.to_string())
                .collect(),
            change_rationale: vec![],
        }
    }

    fn sample_step_evaluation(
        workflow_id: TaskId,
        step_id: &str,
        verdict: VerifierVerdict,
        confidence: Option<f32>,
        clarification_attempt_count: Option<u32>,
        include_retry_repair: bool,
    ) -> StepEvaluationRecord {
        StepEvaluationRecord {
            workflow_id,
            step_id: step_id.to_string(),
            verdict,
            confidence,
            reason: "deterministic verifier outcome".to_string(),
            failure_code: (verdict == VerifierVerdict::Rejected)
                .then_some("policy_conflict".to_string()),
            checkpoint_ref: clarification_attempt_count.map(|count| format!("checkpoint-{count}")),
            repair_directive: include_retry_repair.then_some(RepairDirective {
                action: RepairDirectiveAction::RetryStep,
                issued_by: "runtime.verifier".to_string(),
                failure_context_ref: format!("planner:{step_id}/retry"),
                retry_budget_remaining: 1,
            }),
            clarification_request: clarification_attempt_count.map(|attempt_count| {
                HandoffClarificationRequest {
                    source_step_id: "collect-evidence".to_string(),
                    target_step_id: step_id.to_string(),
                    missing_constraints: vec!["required context".to_string()],
                    attempt_count,
                    expires_at: None,
                }
            }),
            failure_context_checkpoint: clarification_attempt_count.map(|attempt_count| {
                FailureContextCheckpoint {
                    failed_step_id: step_id.to_string(),
                    last_stable_step_id: Some("collect-evidence".to_string()),
                    checkpoint_ref: Some(format!("checkpoint-{attempt_count}")),
                    failure_context_ref: format!("planner:{step_id}/retry"),
                    failure_code: Some("policy_conflict".to_string()),
                    reason: "deterministic verifier outcome".to_string(),
                    attempt_count,
                }
            }),
        }
    }

    fn sample_supervision_evidence(decision_basis: &str) -> SupervisionEvidenceView {
        SupervisionEvidenceView {
            target_scope: mister_smith_core::SupervisionTargetScope {
                kind: mister_smith_core::SupervisionTargetKind::Branch,
                provider: None,
                graph_id: Some(ExecutionGraphId::new()),
                branch_id: Some(ExecutionBranchId::new()),
                node_id: None,
            },
            fingerprint_ref: None,
            profile_snapshot: None,
            guard_decision: None,
            intervention_record: None,
            decision_basis: Some(decision_basis.to_string()),
            repair_lineage_ref: None,
            proof_boundary: Some("deterministic-only".to_string()),
        }
    }

    #[test]
    fn terminal_result_views_preserve_proof_outcome_across_task_and_final_results() {
        let workflow_id = TaskId::new();
        let cases = [
            (
                "success",
                "completed",
                json!({
                    "steps": [{"id": "step-1"}, {"id": "step-2"}]
                }),
                vec![
                    json!({
                        "task_id": TaskId::new(),
                        "result": { "summary": "parallel branch alpha" }
                    }),
                    json!({
                        "task_id": TaskId::new(),
                        "result": { "summary": "parallel branch beta" }
                    }),
                ],
                ProofOutcomeClassification::GraphFormedAndCompleted,
            ),
            (
                "collapse",
                "completed",
                json!({
                    "steps": [{"id": "step-1"}]
                }),
                vec![json!({
                    "task_id": TaskId::new(),
                    "result": { "summary": "single sequential branch" }
                })],
                ProofOutcomeClassification::CollapsedToSequential,
            ),
            (
                "failure",
                "failed",
                json!({
                    "steps": [{"id": "step-1"}, {"id": "step-2"}]
                }),
                vec![json!({
                    "task_id": TaskId::new(),
                    "result": { "summary": "partial branch output" }
                })],
                ProofOutcomeClassification::FailedBeforeGraph,
            ),
        ];

        for (label, status, execution_plan, step_results, expected_outcome) in cases {
            let canonical_result = crate::autonomy::build_canonical_result_envelope(
                crate::autonomy::CanonicalResultEnvelopeInput {
                    workflow_id,
                    provider_kind: PROVIDER_KIND_NAME,
                    model_id: MODEL_ID,
                    description: "freeze the result contract",
                    runtime_execution_mode: json!({
                        "execution_boundary": "tool_bus",
                        "workflow_runner": "tokio_task"
                    }),
                    planner_output: json!({
                        "steps": execution_plan
                            .get("steps")
                            .and_then(Value::as_array)
                            .map(|steps| steps.len())
                    }),
                    execution_plan,
                    step_results,
                    aggregated_result: json!({
                        "summary": format!("{label} bounded answer preview")
                    }),
                    status,
                },
            );

            let result_views =
                terminal_result_views(status, canonical_result, None, &json!({}), None)
                .expect("canonical task and final result envelopes should serialize");

            assert_eq!(
                result_views.final_result["proof_outcome"],
                json!(expected_outcome.as_str()),
                "final_result should retain proof outcome for {label}"
            );
            assert_eq!(
                result_views.task_result["proof_outcome"],
                json!(expected_outcome.as_str()),
                "task.result wrapper should retain proof outcome for {label}"
            );
            assert_eq!(
                result_views.task_result["result"]["proof_outcome"],
                json!(expected_outcome.as_str()),
                "task.result canonical envelope should retain proof outcome for {label}"
            );
        }
    }

    #[test]
    fn build_step_policy_summary_keeps_stable_steps_low_risk() {
        let workflow_id = TaskId::new();
        let step_id = "draft-outline";
        let step_evaluation = sample_step_evaluation(
            workflow_id,
            step_id,
            VerifierVerdict::Accepted,
            Some(0.97),
            None,
            false,
        );
        let canonical_result = crate::autonomy::build_canonical_result_envelope(
            crate::autonomy::CanonicalResultEnvelopeInput {
                workflow_id,
                provider_kind: PROVIDER_KIND_NAME,
                model_id: MODEL_ID,
                description: "stable step",
                runtime_execution_mode: json!({
                    "execution_boundary": "tool_bus",
                    "workflow_runner": "tokio_task",
                    "budget_root": "disabled",
                }),
                planner_output: json!({ "steps": 1 }),
                execution_plan: json!({
                    "steps": [{ "id": step_id }]
                }),
                step_results: vec![json!({
                    "step_evaluation": serde_json::to_value(step_evaluation).expect("step evaluation should serialize"),
                    "result": { "summary": "stable step output" }
                })],
                aggregated_result: json!({ "summary": "stable step output" }),
                status: "completed",
            },
        );
        let metadata = json!({
            "step_routing_history": [
                sample_step_policy_routing(step_id, false, Some(0.94), &[])
            ]
        });

        let summary =
            build_step_policy_summary(&canonical_result, &metadata, None).expect("step policy");

        assert_eq!(
            summary.difficulty_assessment.difficulty_bucket,
            StepDifficultyBucket::Low
        );
        assert_eq!(
            summary.policy_decision.chosen_action,
            StepPolicyAction::Keep
        );
        assert_eq!(
            summary.difficulty_assessment.reason_codes,
            vec!["stable_verifier_accept".to_string()]
        );
    }

    #[test]
    fn build_step_policy_summary_prefers_clarify_for_missing_context() {
        let workflow_id = TaskId::new();
        let step_id = "planner.step.2";
        let step_evaluation = sample_step_evaluation(
            workflow_id,
            step_id,
            VerifierVerdict::Rejected,
            Some(0.41),
            Some(2),
            true,
        );
        let canonical_result = crate::autonomy::build_canonical_result_envelope(
            crate::autonomy::CanonicalResultEnvelopeInput {
                workflow_id,
                provider_kind: PROVIDER_KIND_NAME,
                model_id: MODEL_ID,
                description: "unstable step",
                runtime_execution_mode: json!({
                    "execution_boundary": "tool_bus",
                    "workflow_runner": "tokio_task",
                    "budget_root": "disabled",
                }),
                planner_output: json!({ "steps": 2 }),
                execution_plan: json!({
                    "steps": [{ "id": "collect-evidence" }, { "id": step_id }]
                }),
                step_results: vec![json!({
                    "step_evaluation": serde_json::to_value(step_evaluation).expect("step evaluation should serialize"),
                    "result": { "summary": "unstable step output" }
                })],
                aggregated_result: json!({ "summary": "unstable step output" }),
                status: "completed",
            },
        );
        let metadata = json!({
            "step_routing_history": [
                sample_step_policy_routing(step_id, true, Some(0.45), &[])
            ]
        });

        let summary = build_step_policy_summary(
            &canonical_result,
            &metadata,
            Some(&sample_supervision_evidence("conservative_fallback")),
        )
        .expect("step policy");

        assert!(matches!(
            summary.difficulty_assessment.difficulty_bucket,
            StepDifficultyBucket::High | StepDifficultyBucket::Critical
        ));
        assert_eq!(
            summary.policy_decision.chosen_action,
            StepPolicyAction::Clarify
        );
        assert!(summary
            .difficulty_assessment
            .reason_codes
            .contains(&"missing_context".to_string()));
        assert_eq!(summary.budget_pressure, None);
    }

    #[test]
    fn build_step_policy_summary_downgrades_under_hard_stop_budget_pressure() {
        let workflow_id = TaskId::new();
        let step_id = "planner.step.3";
        let step_evaluation = sample_step_evaluation(
            workflow_id,
            step_id,
            VerifierVerdict::Rejected,
            Some(0.74),
            None,
            true,
        );
        let canonical_result = crate::autonomy::build_canonical_result_envelope(
            crate::autonomy::CanonicalResultEnvelopeInput {
                workflow_id,
                provider_kind: PROVIDER_KIND_NAME,
                model_id: MODEL_ID,
                description: "budget constrained step",
                runtime_execution_mode: json!({
                    "execution_boundary": "tool_bus",
                    "workflow_runner": "tokio_task",
                    "budget_root": "runtime.task_path",
                }),
                planner_output: json!({ "steps": 2 }),
                execution_plan: json!({
                    "steps": [{ "id": "collect-evidence" }, { "id": step_id }]
                }),
                step_results: vec![json!({
                    "step_evaluation": serde_json::to_value(step_evaluation).expect("step evaluation should serialize"),
                    "result": { "summary": "budget constrained output" }
                })],
                aggregated_result: json!({ "summary": "budget constrained output" }),
                status: "completed",
            },
        );
        let metadata = json!({
            "step_routing_history": [
                sample_step_policy_routing(step_id, true, Some(0.68), &["budget_policy"])
            ]
        });

        let summary =
            build_step_policy_summary(&canonical_result, &metadata, None).expect("step policy");

        assert_eq!(
            summary
                .budget_pressure
                .as_ref()
                .map(|pressure| pressure.pressure_level),
            Some(StepBudgetPressureLevel::HardStop)
        );
        assert_eq!(
            summary.policy_decision.chosen_action,
            StepPolicyAction::Downgrade
        );
    }

    #[test]
    fn build_step_policy_summary_ignores_unrelated_step_routing_history() {
        let workflow_id = TaskId::new();
        let step_id = "draft-outline";
        let step_evaluation = sample_step_evaluation(
            workflow_id,
            step_id,
            VerifierVerdict::Accepted,
            Some(0.97),
            None,
            false,
        );
        let canonical_result = crate::autonomy::build_canonical_result_envelope(
            crate::autonomy::CanonicalResultEnvelopeInput {
                workflow_id,
                provider_kind: PROVIDER_KIND_NAME,
                model_id: MODEL_ID,
                description: "stable step with unrelated routing history",
                runtime_execution_mode: json!({
                    "execution_boundary": "tool_bus",
                    "workflow_runner": "tokio_task",
                    "budget_root": "disabled",
                }),
                planner_output: json!({ "steps": 2 }),
                execution_plan: json!({
                    "steps": [{ "id": "collect-evidence" }, { "id": step_id }]
                }),
                step_results: vec![json!({
                    "step_evaluation": serde_json::to_value(step_evaluation).expect("step evaluation should serialize"),
                    "result": { "summary": "stable step output" }
                })],
                aggregated_result: json!({ "summary": "stable step output" }),
                status: "completed",
            },
        );
        let metadata = json!({
            "step_routing_history": [
                sample_step_policy_routing("collect-evidence", true, Some(0.22), &["budget_policy"])
            ]
        });

        let summary =
            build_step_policy_summary(&canonical_result, &metadata, None).expect("step policy");

        assert_eq!(
            summary.difficulty_assessment.difficulty_bucket,
            StepDifficultyBucket::Low
        );
        assert_eq!(
            summary.policy_decision.chosen_action,
            StepPolicyAction::Keep
        );
        assert_eq!(summary.budget_pressure, None);
        assert_eq!(
            summary.difficulty_assessment.reason_codes,
            vec!["stable_verifier_accept".to_string()]
        );
    }

    #[test]
    fn build_step_policy_summary_escalates_moderate_step_under_hard_stop_budget_pressure() {
        let workflow_id = TaskId::new();
        let step_id = "planner.step.4";
        let step_evaluation = sample_step_evaluation(
            workflow_id,
            step_id,
            VerifierVerdict::Rejected,
            Some(0.91),
            None,
            true,
        );
        let canonical_result = crate::autonomy::build_canonical_result_envelope(
            crate::autonomy::CanonicalResultEnvelopeInput {
                workflow_id,
                provider_kind: PROVIDER_KIND_NAME,
                model_id: MODEL_ID,
                description: "moderate step under hard-stop budget pressure",
                runtime_execution_mode: json!({
                    "execution_boundary": "tool_bus",
                    "workflow_runner": "tokio_task",
                    "budget_root": "runtime.task_path",
                }),
                planner_output: json!({ "steps": 2 }),
                execution_plan: json!({
                    "steps": [{ "id": "collect-evidence" }, { "id": step_id }]
                }),
                step_results: vec![json!({
                    "step_evaluation": serde_json::to_value(step_evaluation).expect("step evaluation should serialize"),
                    "result": { "summary": "budget constrained output" }
                })],
                aggregated_result: json!({ "summary": "budget constrained output" }),
                status: "completed",
            },
        );
        let metadata = json!({
            "step_routing_history": [
                sample_step_policy_routing(step_id, false, Some(0.91), &["budget_policy"])
            ]
        });

        let summary =
            build_step_policy_summary(&canonical_result, &metadata, None).expect("step policy");

        assert_eq!(
            summary.difficulty_assessment.difficulty_bucket,
            StepDifficultyBucket::Moderate
        );
        assert_eq!(
            summary
                .budget_pressure
                .as_ref()
                .map(|pressure| pressure.pressure_level),
            Some(StepBudgetPressureLevel::HardStop)
        );
        assert_eq!(
            summary.policy_decision.chosen_action,
            StepPolicyAction::Escalate
        );
        assert!(summary.policy_decision.requires_operator_attention);
    }

    #[test]
    fn terminal_result_views_omit_step_policy_when_terminal_evaluation_is_missing() {
        let workflow_id = TaskId::new();
        let canonical_result = crate::autonomy::build_canonical_result_envelope(
            crate::autonomy::CanonicalResultEnvelopeInput {
                workflow_id,
                provider_kind: PROVIDER_KIND_NAME,
                model_id: MODEL_ID,
                description: "missing evaluation",
                runtime_execution_mode: json!({
                    "execution_boundary": "tool_bus",
                    "workflow_runner": "tokio_task",
                    "budget_root": "disabled",
                }),
                planner_output: json!({ "steps": 1 }),
                execution_plan: json!({
                    "steps": [{ "id": "draft-outline" }]
                }),
                step_results: vec![json!({
                    "result": { "summary": "no verifier output" }
                })],
                aggregated_result: json!({ "summary": "no verifier output" }),
                status: "completed",
            },
        );

        let result_views =
            terminal_result_views("completed", canonical_result, None, &json!({}), None)
            .expect("terminal result views");

        assert_eq!(result_views.task_result["step_policy"], Value::Null);
    }

    #[test]
    fn runtime_llm_selection_preserves_default_chatgpt_baseline() {
        let selection = RuntimeLlmSelection::from_config(&FrameworkConfig::default());

        assert_eq!(selection, RuntimeLlmSelection::default());
    }

    #[test]
    fn runtime_llm_selection_uses_framework_config_override() {
        let mut config = FrameworkConfig::default();
        config.llm.provider_kind = ProviderKind::Mock;
        config.llm.model_id = "mock-ops".to_string();

        let selection = RuntimeLlmSelection::from_config(&config);

        assert_eq!(
            selection,
            sample_runtime_llm_selection(ProviderKind::Mock, "mock-ops")
        );
        assert_eq!(
            selection.provider_config().provider_kind,
            ProviderKind::Mock
        );
        assert_eq!(selection.provider_config().model_id, "mock-ops");
    }

    #[test]
    fn runtime_bootstrap_plan_defaults_to_round_robin_single_provider() {
        let plan = RuntimeBootstrapPlan::from_config(&FrameworkConfig::default());

        assert_eq!(plan.active_selection, RuntimeLlmSelection::default());
        assert_eq!(
            plan.registered_providers,
            vec![RuntimeLlmSelection::default()]
        );
        assert!(matches!(plan.routing_policy, RoutingPolicy::RoundRobin));
    }

    #[test]
    fn runtime_bootstrap_plan_uses_profile_for_cascade_boot() {
        let mut config = FrameworkConfig::default();
        config.llm.provider_kind = ProviderKind::Mock;
        config.llm.model_id = "mock-fallback".to_string();
        config.llm.runtime_routing_profile = Some(RuntimeRoutingProfile {
            policy: RuntimeRoutingPolicy::Cascade,
            budget_root: "runtime.task_path".to_string(),
            tiers: vec![
                RuntimeProviderTier {
                    label: "primary".to_string(),
                    provider_kind: ProviderKind::OpenAiChatGpt,
                    model_id: "gpt-5.4".to_string(),
                    metadata: json!({}),
                },
                RuntimeProviderTier {
                    label: "fallback".to_string(),
                    provider_kind: ProviderKind::ClaudeSubscription,
                    model_id: "claude-sonnet".to_string(),
                    metadata: json!({}),
                },
            ],
        });

        let plan = RuntimeBootstrapPlan::from_config(&config);

        assert_eq!(
            plan.active_selection,
            sample_runtime_llm_selection(ProviderKind::OpenAiChatGpt, "gpt-5.4")
        );
        assert_eq!(plan.registered_providers.len(), 2);
        assert_eq!(plan.budget_root.as_deref(), Some("runtime.task_path"));

        match plan.routing_policy {
            RoutingPolicy::Cascade(policy) => {
                assert_eq!(policy.tiers.len(), 2);
                assert_eq!(
                    policy.escalation_threshold,
                    DEFAULT_RUNTIME_CASCADE_THRESHOLD
                );
                assert_eq!(policy.max_escalations, 1);
                assert_eq!(policy.tiers[0].label, "primary");
                assert_eq!(
                    policy.tiers[0].provider_config.provider_kind,
                    ProviderKind::OpenAiChatGpt
                );
                assert_eq!(policy.tiers[1].label, "fallback");
                assert_eq!(
                    policy.tiers[1].provider_config.provider_kind,
                    ProviderKind::ClaudeSubscription
                );
            }
            other => panic!("expected cascade routing policy, got {other:?}"),
        }
    }

    #[test]
    fn runtime_execution_mode_uses_selected_provider_and_model() {
        let selection = sample_runtime_llm_selection(ProviderKind::Mock, "mock-ops");

        let mode = runtime_execution_mode(&selection);

        assert_eq!(mode["provider_kind"], json!("mock"));
        assert_eq!(mode["model_id"], json!("mock-ops"));
        assert_eq!(mode["routing_policy"], json!("round_robin"));
        assert_eq!(mode["registered_provider_count"], json!(1));
        assert_eq!(mode["budget_root"], json!("disabled"));
    }

    #[test]
    fn runtime_execution_mode_with_context_surfaces_budget_routing_details() {
        let selection = sample_runtime_llm_selection(ProviderKind::Mock, "mock-ops");
        let context = RuntimeExecutionModeContext {
            routing_policy: "cascade".to_string(),
            registered_provider_count: 2,
            budget_root: Some("runtime.task_path".to_string()),
        };

        let mode = runtime_execution_mode_with_context(&selection, &context);

        assert_eq!(mode["provider_kind"], json!("mock"));
        assert_eq!(mode["model_id"], json!("mock-ops"));
        assert_eq!(mode["routing_policy"], json!("cascade"));
        assert_eq!(mode["registered_provider_count"], json!(2));
        assert_eq!(mode["budget_root"], json!("runtime.task_path"));
    }

    #[test]
    fn runtime_envelope_provenance_prefers_persisted_metadata_over_current_selection() {
        let metadata = json!({
            "provider_kind": "mock",
            "model_id": "mock-ops",
            "runtime_execution_mode": {
                "execution_boundary": "tool_bus",
                "provider_kind": "mock",
                "model_id": "mock-ops",
                "routing_policy": "cascade",
                "registered_provider_count": 2,
                "budget_root": "runtime.task_path"
            }
        });
        let fallback = RuntimeLlmSelection::default();

        let provenance = runtime_envelope_provenance(&metadata, &fallback);

        assert_eq!(provenance.provider_kind, "mock");
        assert_eq!(provenance.model_id, "mock-ops");
        assert_eq!(provenance.runtime_execution_mode["provider_kind"], "mock");
        assert_eq!(provenance.runtime_execution_mode["model_id"], "mock-ops");
        assert_eq!(
            provenance.runtime_execution_mode["routing_policy"],
            "cascade"
        );
        assert_eq!(
            provenance.runtime_execution_mode["registered_provider_count"],
            2
        );
        assert_eq!(
            provenance.runtime_execution_mode["budget_root"],
            "runtime.task_path"
        );
    }

    #[test]
    fn runtime_envelope_provenance_falls_back_to_current_selection_when_metadata_missing() {
        let fallback = sample_runtime_llm_selection(ProviderKind::Mock, "mock-ops");

        let provenance = runtime_envelope_provenance(&json!({}), &fallback);

        assert_eq!(provenance.provider_kind, "mock");
        assert_eq!(provenance.model_id, "mock-ops");
        assert_eq!(provenance.runtime_execution_mode["provider_kind"], "mock");
        assert_eq!(provenance.runtime_execution_mode["model_id"], "mock-ops");
        assert_eq!(
            provenance.runtime_execution_mode["routing_policy"],
            "round_robin"
        );
        assert_eq!(
            provenance.runtime_execution_mode["registered_provider_count"],
            1
        );
        assert_eq!(provenance.runtime_execution_mode["budget_root"], "disabled");
    }

    #[test]
    fn build_runtime_provider_supports_mock_selection() {
        let selection = sample_runtime_llm_selection(ProviderKind::Mock, "mock-ops");

        assert!(build_runtime_provider(&selection).is_ok());
    }

    #[test]
    fn build_runtime_provider_rejects_unshipped_provider_kinds() {
        for provider_kind in [ProviderKind::OpenAi, ProviderKind::Anthropic] {
            let selection = sample_runtime_llm_selection(provider_kind, "test-model");

            let error = match build_runtime_provider(&selection) {
                Ok(_) => panic!("unshipped provider kinds should fail explicitly"),
                Err(error) => error,
            };

            assert!(error.contains(provider_kind.as_str()));
            assert!(error.contains("not supported"));
        }
    }

    #[tokio::test]
    async fn register_runtime_router_providers_adds_every_registered_provider() {
        let router = ModelRouter::new(RoutingPolicy::RoundRobin);
        let selections = vec![
            sample_runtime_llm_selection(ProviderKind::Mock, "mock-primary"),
            sample_runtime_llm_selection(ProviderKind::Mock, "mock-fallback"),
        ];

        register_runtime_router_providers(&router, &selections)
            .await
            .expect("mock providers should register");

        assert_eq!(router.provider_count().await, 2);
    }

    #[tokio::test]
    #[ignore = "requires local NATS with JetStream on localhost:4222"]
    async fn jetstream_budget_store_round_trips_budget_nodes() {
        let client = async_nats::connect("nats://localhost:4222")
            .await
            .expect("local NATS with JetStream should be available");
        let jetstream = JetStreamManager::new(client, JetStreamConfig::default());
        let bucket = format!("runtimebudget{}", Uuid::new_v4().simple());
        let store = JetStreamBudgetStore::new(
            ensure_runtime_budget_kv_store(&jetstream, &bucket)
                .await
                .expect("budget bucket should initialize"),
        );
        let node = BudgetNode {
            key: "budget/test".to_string(),
            limit_tokens: 10_000,
            used_tokens: 0,
            period: "2026-03-daily".to_string(),
            policy: mister_smith_llm::BudgetPolicy::HardCap,
            revision: 0,
        };

        let created_revision = store
            .cas_update(&node, 0)
            .await
            .expect("initial CAS create should succeed");
        let created = store
            .get("budget/test")
            .await
            .expect("created budget node should load")
            .expect("created budget node should exist");
        assert_eq!(created.limit_tokens, 10_000);
        assert_eq!(created.used_tokens, 0);
        assert_eq!(created.revision, created_revision);

        let mut updated = created.clone();
        updated.used_tokens = 512;
        let updated_revision = store
            .cas_update(&updated, created.revision)
            .await
            .expect("CAS update should succeed");
        let fetched = store
            .get("budget/test")
            .await
            .expect("updated budget node should load")
            .expect("updated budget node should exist");
        assert_eq!(fetched.used_tokens, 512);
        assert_eq!(fetched.revision, updated_revision);

        jetstream
            .context()
            .delete_key_value(&bucket)
            .await
            .expect("test bucket should delete cleanly");
    }

    #[tokio::test]
    #[ignore = "requires local NATS with JetStream on localhost:4222"]
    async fn runtime_budget_enforcer_requires_existing_budget_root() {
        let client = async_nats::connect("nats://localhost:4222")
            .await
            .expect("local NATS with JetStream should be available");
        let jetstream = JetStreamManager::new(client, JetStreamConfig::default());
        let bucket = format!("runtimebudget{}", Uuid::new_v4().simple());

        let error =
            match build_runtime_budget_enforcer_with_bucket(&jetstream, &bucket, "budget/missing")
                .await
            {
                Ok(_) => panic!("missing budget roots should fail bootstrap preflight"),
                Err(error) => error,
            };

        assert!(error.contains("budget/missing"));
        assert!(error.contains(&bucket));

        jetstream
            .context()
            .delete_key_value(&bucket)
            .await
            .expect("test bucket should delete cleanly");
    }

    #[test]
    fn recover_persisted_autonomy_status_restores_session_linkage() {
        let session_id = SessionId::new();
        let coordinator_agent_id = AgentId::new();
        let view = sample_autonomy_view();
        let workflow_id = view.graph.workflow_id;
        let mut metadata = json!({
            "session_id": session_id,
            "turn_index": 2,
            "coordinator_agent_id": coordinator_agent_id,
        });
        persist_autonomy_status(&mut metadata, &view);
        let record = sample_task_with_metadata(metadata);

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("persisted autonomy status should round-trip");

        assert_eq!(recovered.graph.workflow_id, workflow_id);
        assert_eq!(recovered.session_id, Some(session_id));
        assert_eq!(recovered.turn_index, Some(2));
        assert_eq!(recovered.coordinator_agent_id, Some(coordinator_agent_id));
    }

    #[test]
    fn recover_persisted_autonomy_status_preserves_supervision_evidence_snapshot() {
        let mut view = sample_autonomy_view();
        let branch_id = view.branches[0].branch_id;
        let node_id = mister_smith_core::ExecutionNodeId::new();
        let fingerprint_ref = mister_smith_core::ProfileFingerprintRef {
            fingerprint_id: mister_smith_core::ProfileFingerprintId::new(),
            fingerprint_key: "branch-a".to_string(),
            confidence: 0.82,
            expires_at: Utc::now(),
        };
        let profile_snapshot = mister_smith_core::ProfileSnapshot {
            profile_id: mister_smith_core::ProfileSnapshotId::new(),
            target: mister_smith_core::ProfileTarget::Branch,
            health_state: mister_smith_core::HealthState::Degraded,
            latency_window: None,
            error_window: None,
            semantic_signals: vec![mister_smith_core::SemanticSignal {
                signal_kind: mister_smith_core::SemanticSignalKind::MissingContext,
                severity: 74,
                detail: "missing branch-local repair context".to_string(),
            }],
            fingerprint_ref: Some(fingerprint_ref.clone()),
            updated_at: Utc::now(),
        };
        let guard_decision = mister_smith_core::GuardDecision {
            decision_id: mister_smith_core::GuardDecisionId::new(),
            failure_class: mister_smith_core::FailureClass::Semantic,
            intervention: mister_smith_core::InterventionType::ContextRefresh,
            evidence: mister_smith_core::GuardEvidence {
                profile_id: Some(profile_snapshot.profile_id),
                decision_basis: mister_smith_core::SupervisionDecisionBasis::LiveSignalsOnly,
                signal_descriptions: vec!["missing branch-local repair context".to_string()],
                checkpoint_ids: vec![],
                notes: vec!["step boundary node evidence".to_string()],
            },
            target_scope: GuardTarget::Node(node_id),
            operator_visibility: true,
        };
        let intervention_record = mister_smith_core::InterventionRecord {
            record_id: mister_smith_core::InterventionRecordId::new(),
            decision_id: guard_decision.decision_id,
            before_state: json!({"state": "running"}),
            after_state: Some(json!({"state": "pending"})),
            rationale: "applied context refresh for semantic degradation".to_string(),
            emitted_at: Utc::now(),
        };
        view.supervision_evidence = Some(mister_smith_core::SupervisionEvidenceView {
            target_scope: mister_smith_core::SupervisionTargetScope {
                kind: mister_smith_core::SupervisionTargetKind::Node,
                provider: None,
                graph_id: Some(view.graph.graph_id),
                branch_id: Some(branch_id),
                node_id: Some(node_id),
            },
            fingerprint_ref: Some(fingerprint_ref),
            profile_snapshot: Some(profile_snapshot),
            guard_decision: Some(guard_decision),
            intervention_record: Some(intervention_record),
            decision_basis: Some("live_signals_only".to_string()),
            repair_lineage_ref: Some(mister_smith_core::RepairLineageRef {
                source: "packet-020".to_string(),
                checkpoint_ref: Some("checkpoint-clarify".to_string()),
            }),
            proof_boundary: Some("supported task path".to_string()),
        });

        let mut metadata = json!({});
        persist_autonomy_status(&mut metadata, &view);
        let record = sample_task_with_metadata(metadata);

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("persisted autonomy status should round-trip");
        let supervision = recovered
            .supervision_evidence
            .expect("supervision evidence should round-trip");

        assert_eq!(
            supervision.target_scope.kind,
            mister_smith_core::SupervisionTargetKind::Node
        );
        assert_eq!(supervision.target_scope.branch_id, Some(branch_id));
        assert_eq!(supervision.target_scope.node_id, Some(node_id));
        assert_eq!(
            supervision.decision_basis.as_deref(),
            Some("live_signals_only")
        );
        assert_eq!(
            supervision
                .fingerprint_ref
                .as_ref()
                .map(|reference| reference.fingerprint_key.as_str()),
            Some("branch-a")
        );
        assert_eq!(
            supervision
                .repair_lineage_ref
                .as_ref()
                .map(|lineage| lineage.source.as_str()),
            Some("packet-020")
        );
    }

    #[test]
    fn initial_metadata_persists_external_delegation_context() {
        let delegation = sample_external_delegation();
        let request = TaskSubmissionRequest {
            description: "delegated workflow".to_string(),
            agent_type: None,
            priority: Some("high".to_string()),
            conversation: None,
            delegation: Some(delegation.clone()),
        };

        let metadata = initial_metadata(
            &request,
            AgentId::new(),
            &[],
            "queued",
            &RuntimeLlmSelection::default(),
        );

        assert_eq!(
            metadata["external_delegation"]["capability"]["descriptor_id"],
            "tool:app.workflow"
        );
        assert_eq!(
            metadata["external_delegation"]["action"]["revocation_key"],
            "tool:app.workflow#execute"
        );
    }

    #[test]
    fn initial_metadata_uses_selected_provider_and_model() {
        let request = TaskSubmissionRequest {
            description: "mock workflow".to_string(),
            agent_type: None,
            priority: Some("normal".to_string()),
            conversation: None,
            delegation: None,
        };

        let metadata = initial_metadata(
            &request,
            AgentId::new(),
            &[],
            "queued",
            &sample_runtime_llm_selection(ProviderKind::Mock, "mock-ops"),
        );

        assert_eq!(metadata["provider_kind"], json!("mock"));
        assert_eq!(metadata["model_id"], json!("mock-ops"));
    }

    #[test]
    fn initial_metadata_persists_accepted_task_ingress_continuity_for_delegated_http_submission() {
        let delegation = sample_external_delegation();
        let request = TaskSubmissionRequest {
            description: "delegated workflow".to_string(),
            agent_type: None,
            priority: Some("high".to_string()),
            conversation: None,
            delegation: Some(delegation.clone()),
        };

        let metadata = initial_metadata(
            &request,
            AgentId::new(),
            &[],
            "queued",
            &RuntimeLlmSelection::default(),
        );
        let accepted = metadata
            .get(ACCEPTED_TASK_INGRESS_METADATA_KEY)
            .expect("delegated HTTP submission should persist accepted task ingress continuity");

        assert_eq!(
            accepted["request_surface"],
            json!(TASK_INGRESS_REQUEST_SURFACE)
        );
        assert_eq!(
            accepted["source_metadata_key"],
            json!("external_delegation")
        );
        assert_eq!(
            accepted["capability_id"],
            json!(delegation.capability.capability_id)
        );
        assert_eq!(
            accepted["capability_descriptor_id"],
            json!(delegation.capability.descriptor_id)
        );
        assert_eq!(
            accepted["action_id"],
            json!(delegation
                .action
                .as_ref()
                .map(|action| action.action_id.clone()))
        );
        assert_eq!(
            accepted["policy_resource_id"],
            json!(delegation
                .action
                .as_ref()
                .and_then(|action| action.policy.resource_id.clone()))
        );
        assert_eq!(
            accepted["chain_depth"],
            json!(delegation.provenance.links.len())
        );
    }

    #[test]
    fn initial_metadata_keeps_accepted_task_ingress_scope_frozen_to_http_task_submissions() {
        let request = TaskSubmissionRequest {
            description: "delegated session turn".to_string(),
            agent_type: None,
            priority: Some("high".to_string()),
            conversation: Some(mister_smith_http::server::ConversationTurnContext {
                session_id: SessionId::new(),
                turn_index: 1,
                coordinator_agent_id: AgentId::new(),
                retained_context: json!({
                    "transcript_summary": []
                }),
            }),
            delegation: Some(sample_external_delegation()),
        };

        let metadata = initial_metadata(
            &request,
            AgentId::new(),
            &[],
            "queued",
            &RuntimeLlmSelection::default(),
        );

        assert!(
            metadata.get("external_delegation").is_some(),
            "raw delegation context should still persist for session continuity"
        );
        assert!(
            metadata.get(ACCEPTED_TASK_INGRESS_METADATA_KEY).is_none(),
            "accepted task ingress continuity must remain frozen to POST /api/v1/tasks"
        );
    }

    #[test]
    fn recover_persisted_autonomy_status_does_not_infer_allowed_external_capability_decision() {
        let view = sample_autonomy_view();
        let delegation = sample_external_delegation();
        let mut metadata = json!({
            "external_delegation": delegation,
        });
        persist_autonomy_status(&mut metadata, &view);
        let record = sample_task_with_metadata(metadata);

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("persisted autonomy status should still recover");

        assert!(
            recovered.external_capability_decisions.is_empty(),
            "metadata-only delegation context must not fabricate an allowed boundary decision"
        );
    }

    #[test]
    fn recover_persisted_autonomy_status_synthesizes_task_ingress_decision_from_frozen_metadata() {
        let view = sample_autonomy_view();
        let delegation = sample_external_delegation();
        let mut metadata = json!({
            "external_delegation": delegation.clone(),
            ACCEPTED_TASK_INGRESS_METADATA_KEY: accepted_task_ingress_metadata(&delegation),
        });
        persist_autonomy_status(&mut metadata, &view);
        let record = sample_task_with_metadata(metadata);

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("persisted autonomy status should synthesize task-ingress continuity");
        let summary = recovered
            .external_capability_decisions
            .first()
            .expect("accepted task ingress should surface as one boundary decision");

        assert_eq!(
            summary.boundary_surface,
            Some(mister_smith_events::ExternalCapabilityDecisionSurface::TaskIngress)
        );
        assert_eq!(summary.outcome, ExternalCapabilityDecisionOutcome::Allowed);
        assert_eq!(summary.branch_id, None);
        assert_eq!(
            summary.action_id.as_deref(),
            Some("tool:app.workflow#execute")
        );
        assert!(summary
            .rationale
            .iter()
            .any(|line| line.contains("POST /api/v1/tasks")));
        assert!(summary.rationale.iter().any(|line| {
            line.contains("accepted_task_ingress sourced from external_delegation")
        }));
    }

    #[test]
    fn recover_persisted_autonomy_status_preserves_allowed_external_capability_decision_snapshot() {
        let mut view = sample_autonomy_view();
        view.external_capability_decisions = vec![ExternalCapabilityDecisionSummary {
            boundary_surface: Some(mister_smith_events::ExternalCapabilityDecisionSurface::ToolBus),
            branch_id: Some(ExecutionBranchId::new()),
            capability_id: Some(CapabilityId::new()),
            capability_descriptor_id: Some("tool:app.workflow".to_string()),
            action_descriptor_id: Some("tool:app.workflow".to_string()),
            action_id: Some("tool:app.workflow#execute".to_string()),
            action_title: Some("execute app.workflow".to_string()),
            scope: Some(mister_smith_core::DelegationScope::InvokeTool),
            required_scope: Some(mister_smith_core::DelegationScope::InvokeTool),
            policy_action: Some("execute".to_string()),
            policy_resource: Some("workflow".to_string()),
            policy_scope: Some("app".to_string()),
            policy_resource_id: Some("app.workflow".to_string()),
            revocation_state: Some(RevocationState::Active),
            attestation_source: None,
            chain_depth: 1,
            outcome: ExternalCapabilityDecisionOutcome::Allowed,
            observed_at: Some(Utc::now()),
            rationale: vec![
                "descriptor 'tool:app.workflow' matched the requested external action".to_string(),
                "required scope InvokeTool matched capability scope InvokeTool".to_string(),
            ],
        }];
        let mut metadata = json!({});
        persist_autonomy_status(&mut metadata, &view);
        let record = sample_task_with_metadata(metadata);

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("persisted autonomy status should expose external capability decisions");
        let summary = recovered
            .external_capability_decisions
            .first()
            .expect("external capability decision should survive in the persisted snapshot");

        assert_eq!(summary.outcome, ExternalCapabilityDecisionOutcome::Allowed);
        assert_eq!(
            summary.capability_descriptor_id.as_deref(),
            Some("tool:app.workflow")
        );
        assert_eq!(
            summary.action_descriptor_id.as_deref(),
            Some("tool:app.workflow")
        );
        assert!(summary
            .rationale
            .iter()
            .any(|line| line.contains("matched the requested external action")));
        assert!(summary.rationale.iter().any(|line| {
            line.contains("required scope InvokeTool matched capability scope InvokeTool")
        }));
    }

    #[test]
    fn recover_persisted_autonomy_status_preserves_rejected_external_capability_decision_snapshot()
    {
        let mut view = sample_autonomy_view();
        view.external_capability_decisions = vec![ExternalCapabilityDecisionSummary {
            boundary_surface: Some(mister_smith_events::ExternalCapabilityDecisionSurface::ToolBus),
            branch_id: Some(ExecutionBranchId::new()),
            capability_id: Some(CapabilityId::new()),
            capability_descriptor_id: Some("tool:app.other".to_string()),
            action_descriptor_id: Some("tool:app.workflow".to_string()),
            action_id: Some("tool:app.workflow#execute".to_string()),
            action_title: Some("execute app.workflow".to_string()),
            scope: Some(mister_smith_core::DelegationScope::InvokeTool),
            required_scope: Some(mister_smith_core::DelegationScope::InvokeTool),
            policy_action: Some("execute".to_string()),
            policy_resource: Some("workflow".to_string()),
            policy_scope: Some("app".to_string()),
            policy_resource_id: Some("app.workflow".to_string()),
            revocation_state: Some(RevocationState::Active),
            attestation_source: None,
            chain_depth: 1,
            outcome: ExternalCapabilityDecisionOutcome::Rejected,
            observed_at: Some(Utc::now()),
            rationale: vec![
                "delegation descriptor 'tool:app.other' does not authorize action descriptor 'tool:app.workflow'".to_string(),
            ],
        }];
        let mut metadata = json!({});
        persist_autonomy_status(&mut metadata, &view);
        let record = sample_task_with_metadata(metadata);

        let recovered = recover_persisted_autonomy_status(&record).expect(
            "persisted autonomy status should preserve rejected external capability decisions",
        );
        let summary = recovered.external_capability_decisions.first().expect(
            "rejected external capability decision should survive in the persisted snapshot",
        );

        assert_eq!(summary.outcome, ExternalCapabilityDecisionOutcome::Rejected);
        assert!(summary.rationale.iter().any(|line| {
            line.contains("does not authorize action descriptor 'tool:app.workflow'")
        }));
    }

    #[test]
    fn recover_persisted_autonomy_status_does_not_infer_rejected_external_capability_decision() {
        let view = sample_autonomy_view();
        let mut delegation = sample_external_delegation();
        delegation.capability.descriptor_id = Some("tool:app.other".to_string());
        let mut metadata = json!({
            "external_delegation": delegation,
        });
        persist_autonomy_status(&mut metadata, &view);
        let record = sample_task_with_metadata(metadata);

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("persisted autonomy status should still recover");

        assert!(
            recovered.external_capability_decisions.is_empty(),
            "metadata-only delegation context must not fabricate a rejected boundary decision"
        );
    }

    #[test]
    fn recover_persisted_autonomy_status_surfaces_restart_resume_provenance() {
        let resumed_from_workflow_id = TaskId::new();
        let view = sample_autonomy_view();
        let mut metadata = json!({
            "turn_index": 2,
            "restart_recovery": {
                "reason": "workflow interrupted by runtime restart before session sync",
                "recovered_at": "2026-03-17T21:00:00Z"
            },
            "retained_context": {
                "latest_workflow_id": resumed_from_workflow_id,
                "transcript_summary": [
                    {
                        "turn_index": 1,
                        "workflow_id": resumed_from_workflow_id,
                        "assistant_result": {
                            "workflow_id": resumed_from_workflow_id,
                            "turn_index": 1,
                            "status": "failed",
                            "assistant_result": {
                                "recovered_after_restart": true
                            },
                            "preview": "workflow interrupted by runtime restart before session sync",
                            "provenance": {
                                "runtime_execution_mode": {
                                    "execution_boundary": "tool_bus"
                                },
                                "source_fields": [
                                    "metadata.final_result",
                                    "metadata.aggregated_result"
                                ]
                            }
                        }
                    }
                ]
            }
        });
        persist_autonomy_status(&mut metadata, &view);
        let record = sample_task_with_metadata(metadata);

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("persisted autonomy status should expose restart and resume provenance");
        let provenance = recovered
            .resume_provenance
            .expect("resume provenance should be attached to the recovered view");

        assert!(provenance.recovered_after_restart);
        assert!(provenance.resumed_after_restart);
        assert_eq!(
            provenance.recovery_reason.as_deref(),
            Some("workflow interrupted by runtime restart before session sync")
        );
        assert_eq!(provenance.resumed_from_turn_index, Some(1));
        assert_eq!(
            provenance.resumed_from_workflow_id,
            Some(resumed_from_workflow_id)
        );
    }

    #[test]
    fn recover_persisted_autonomy_status_enriches_step_routing_history_from_metadata() {
        let view = sample_autonomy_view();
        let step_history = vec![StepRoutingDecisionSummary {
            step_id: "planner.step.2".to_string(),
            step_index: Some(2),
            step_kind: Some("planner".to_string()),
            model_id: "gpt-5.4".to_string(),
            tier: "llm-tier".to_string(),
            reason: "accepted at llm-tier after confidence review".to_string(),
            previous_step_id: Some("planner.step.1".to_string()),
            previous_action: Some("escalate".to_string()),
            previous_tier: Some("slm-tier".to_string()),
            action: "continue".to_string(),
            action_changed: true,
            preferred_tier_after: Some("llm-tier".to_string()),
            estimated_cost_tokens: Some(128),
            confidence_score: Some(0.92),
            triggered_checkpoints: vec![],
            change_rationale: vec!["action changed from escalate to continue".to_string()],
        }];
        let mut metadata = json!({
            "step_routing_history": step_history,
        });
        persist_autonomy_status(&mut metadata, &view);
        let record = sample_task_with_metadata(metadata);

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("persisted autonomy status should expose step routing history");

        assert_eq!(recovered.step_routing_history.len(), 1);
        assert_eq!(recovered.step_routing_history[0].action, "continue");
        assert!(recovered.step_routing_history[0].action_changed);
    }

    #[test]
    fn recover_persisted_autonomy_status_enriches_result_preview_from_task_result() {
        let mut view = sample_autonomy_view();
        view.graph.state = GraphState::Completed;
        let workflow_id = view.graph.workflow_id;
        let mut metadata = json!({
            "final_result": {
                "workflow_id": workflow_id,
                "provider_kind": PROVIDER_KIND_NAME,
                "model_id": MODEL_ID,
                "description": "freeze the result contract",
                "runtime_execution_mode": {
                    "execution_boundary": "tool_bus"
                },
                "planner_output": {
                    "steps": 1
                },
                "execution_plan": {
                    "steps": [{"id": "step-1"}]
                },
                "step_results": [
                    {
                        "task_id": TaskId::new(),
                        "result": {
                            "summary": "bounded answer preview"
                        }
                    }
                ],
                "aggregated_result": {
                    "summary": "bounded answer preview"
                },
            }
        });
        persist_autonomy_status(&mut metadata, &view);
        let record = sample_task_with_result(
            metadata,
            json!({
                "workflow_id": workflow_id,
                "status": "completed",
                "proof_outcome": "collapsed_to_sequential",
                "result": {
                    "workflow_id": workflow_id,
                    "provider_kind": PROVIDER_KIND_NAME,
                    "model_id": MODEL_ID,
                    "description": "freeze the result contract",
                    "runtime_execution_mode": {
                        "execution_boundary": "tool_bus"
                    },
                    "planner_output": {
                        "steps": 1
                    },
                    "execution_plan": {
                        "steps": [{"id": "step-1"}]
                    },
                    "step_results": [
                        {
                            "task_id": TaskId::new(),
                            "result": {
                                "summary": "bounded answer preview"
                            }
                        }
                    ],
                    "aggregated_result": {
                        "summary": "bounded answer preview"
                    },
                    "proof_outcome": "collapsed_to_sequential"
                }
            }),
        );

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("persisted autonomy status should expose the operator result preview");

        let preview = recovered
            .result_preview
            .expect("result preview should derive from the canonical task result");
        assert_eq!(preview.payload_location, "task.result");
        assert_eq!(
            preview.proof_outcome,
            mister_smith_core::ProofOutcomeClassification::CollapsedToSequential
        );
        assert_eq!(
            preview.preview_text.as_deref(),
            Some("bounded answer preview")
        );
    }

    #[test]
    fn recover_persisted_autonomy_status_preserves_fresher_snapshot_history() {
        let mut view = sample_autonomy_view();
        view.step_routing_history = vec![StepRoutingDecisionSummary {
            step_id: "planner.step.3".to_string(),
            step_index: Some(3),
            step_kind: Some("planner".to_string()),
            model_id: "gpt-5.4".to_string(),
            tier: "llm-tier".to_string(),
            reason: "continued after live snapshot publication".to_string(),
            previous_step_id: Some("planner.step.2".to_string()),
            previous_action: Some("continue".to_string()),
            previous_tier: Some("llm-tier".to_string()),
            action: "continue".to_string(),
            action_changed: false,
            preferred_tier_after: Some("llm-tier".to_string()),
            estimated_cost_tokens: Some(96),
            confidence_score: Some(0.95),
            triggered_checkpoints: vec![],
            change_rationale: vec!["live snapshot already includes the latest step".to_string()],
        }];

        let mut metadata = json!({
            "step_routing_history": [
                {
                    "step_id": "planner.step.1",
                    "step_index": 1,
                    "step_kind": "planner",
                    "model_id": "gpt-5.4",
                    "tier": "slm-tier",
                    "reason": "stale planning-time metadata",
                    "previous_step_id": null,
                    "previous_action": null,
                    "previous_tier": null,
                    "action": "escalate",
                    "action_changed": false,
                    "preferred_tier_after": "slm-tier",
                    "estimated_cost_tokens": 64,
                    "confidence_score": 0.40,
                    "triggered_checkpoints": ["confidence_review"],
                    "change_rationale": ["initial metadata only"]
                }
            ]
        });
        persist_autonomy_status(&mut metadata, &view);
        let record = sample_task_with_metadata(metadata);

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("persisted autonomy status should preserve fresher live history");

        assert_eq!(recovered.step_routing_history.len(), 1);
        assert_eq!(recovered.step_routing_history[0].step_id, "planner.step.3");
        assert_eq!(
            recovered.step_routing_history[0].reason,
            "continued after live snapshot publication"
        );
    }

    #[test]
    fn recover_persisted_autonomy_status_rejects_invalid_payloads() {
        let record = sample_task_with_metadata(json!({
            AUTONOMY_STATUS_METADATA_KEY: "not-an-object"
        }));

        assert!(recover_persisted_autonomy_status(&record).is_none());
    }

    #[test]
    fn recover_persisted_autonomy_status_synthesizes_failed_before_graph_without_snapshot() {
        let workflow_id = TaskId::new();
        let session_id = SessionId::new();
        let coordinator_agent_id = AgentId::new();
        let execution_plan = json!({
            "steps": [
                {
                    "id": "facts",
                    "step": 1,
                    "action": "analyze",
                    "role": "worker",
                    "branch": "branch-a",
                    "depends_on": []
                },
                {
                    "id": "hypotheses",
                    "step": 2,
                    "action": "analyze",
                    "role": "worker",
                    "branch": "branch-b",
                    "depends_on": []
                },
                {
                    "id": "actions",
                    "step": 3,
                    "action": "analyze",
                    "role": "worker",
                    "branch": "branch-c",
                    "depends_on": []
                },
                {
                    "id": "join",
                    "step": 4,
                    "action": "summarize",
                    "role": "coordinator",
                    "branch": "join",
                    "depends_on": ["facts", "hypotheses", "actions"]
                }
            ]
        });
        let canonical_result = crate::autonomy::build_canonical_result_envelope(
            crate::autonomy::CanonicalResultEnvelopeInput {
                workflow_id,
                provider_kind: PROVIDER_KIND_NAME,
                model_id: MODEL_ID,
                description: "packet 015 failed before graph publication",
                runtime_execution_mode: json!({
                    "execution_boundary": "tool_bus",
                    "workflow_runner": "tokio_task",
                    "planner_lifecycle": "supervised_actor",
                    "executor_lifecycle": "supervised_actor",
                }),
                planner_output: json!({
                    "goal": "incident packet",
                    "steps": 4,
                }),
                execution_plan: execution_plan.clone(),
                step_results: vec![],
                aggregated_result: json!({
                    "error": "execution graph compile failed: Unsupported topology contract: unsupported planner role 'joiner'",
                }),
                status: "failed",
            },
        );
        let task_result = serde_json::to_value(crate::autonomy::build_task_result_view(
            "failed",
            canonical_result.clone(),
            None,
            None,
        ))
        .expect("task result should serialize");
        let mut record = sample_task_with_result(
            json!({
                "session_id": session_id,
                "turn_index": 2,
                "coordinator_agent_id": coordinator_agent_id,
                "final_result": serde_json::to_value(canonical_result).expect("final result should serialize"),
                "execution_plan": execution_plan,
            }),
            task_result,
        );
        record.task_id = *workflow_id.as_ref();

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("failed-before-graph workflows should synthesize a bounded autonomy status");

        assert_eq!(recovered.graph.workflow_id, workflow_id);
        assert_eq!(recovered.graph.state, GraphState::Failed);
        assert_eq!(recovered.graph.branch_count, 3);
        assert_eq!(recovered.graph.node_count, 4);
        assert_eq!(recovered.topology.topology_kind, TopologyKind::Hybrid);
        assert_eq!(recovered.topology.parallelism_width, 3);
        assert_eq!(
            recovered.topology.task_shape.kind,
            TaskShapeKind::FanoutJoin
        );
        assert_eq!(recovered.session_id, Some(session_id));
        assert_eq!(recovered.turn_index, Some(2));
        assert_eq!(recovered.coordinator_agent_id, Some(coordinator_agent_id));
        assert!(recovered
            .conservative_reasons
            .iter()
            .any(|line| line.contains("workflow failed before graph publication")));

        let preview = recovered
            .result_preview
            .expect("synthesized view should still expose the canonical result preview");
        assert_eq!(preview.payload_location, "task.result");
        assert_eq!(
            preview.proof_outcome,
            ProofOutcomeClassification::FailedBeforeGraph
        );
        assert_eq!(
            preview.preview_text.as_deref(),
            Some(
                "execution graph compile failed: Unsupported topology contract: unsupported planner role 'joiner'"
            )
        );
    }

    #[test]
    fn recover_persisted_autonomy_status_falls_back_to_metadata_final_result_when_task_result_mismatches(
    ) {
        let workflow_id = TaskId::new();
        let session_id = SessionId::new();
        let coordinator_agent_id = AgentId::new();
        let execution_plan = json!({
            "steps": [
                {"id": "root", "depends_on": []},
                {"id": "branch-a", "depends_on": ["root"]},
                {"id": "branch-b", "depends_on": ["root"]},
                {"id": "join", "depends_on": ["branch-a", "branch-b"]},
            ]
        });
        let canonical_result = crate::autonomy::build_canonical_result_envelope(
            crate::autonomy::CanonicalResultEnvelopeInput {
                workflow_id,
                provider_kind: "openai_chatgpt",
                model_id: "gpt-5.4",
                description: "packet 015 failed before graph publication",
                runtime_execution_mode: json!({
                    "execution_boundary": "tool_bus",
                    "workflow_runner": "tokio_task",
                    "planner_lifecycle": "supervised_actor",
                    "executor_lifecycle": "supervised_actor",
                }),
                planner_output: json!({
                    "goal": "incident packet",
                    "steps": 4,
                }),
                execution_plan: execution_plan.clone(),
                step_results: vec![],
                aggregated_result: json!({
                    "error": "planner execution failed: Ask operation timed out",
                }),
                status: "failed",
            },
        );
        let mismatched_task_result = serde_json::to_value(crate::autonomy::build_task_result_view(
            "failed",
            crate::autonomy::build_canonical_result_envelope(
                crate::autonomy::CanonicalResultEnvelopeInput {
                    workflow_id: TaskId::new(),
                    provider_kind: "openai_chatgpt",
                    model_id: "gpt-5.4",
                    description: "wrong workflow",
                    runtime_execution_mode: json!({}),
                    planner_output: Value::Null,
                    execution_plan: Value::Null,
                    step_results: vec![],
                    aggregated_result: json!({
                        "error": "wrong workflow",
                    }),
                    status: "failed",
                },
            ),
            None,
            None,
        ))
        .expect("task result should serialize");
        let mut record = sample_task_with_result(
            json!({
                "session_id": session_id,
                "turn_index": 2,
                "coordinator_agent_id": coordinator_agent_id,
                "final_result": serde_json::to_value(canonical_result).expect("final result should serialize"),
                "execution_plan": execution_plan,
            }),
            mismatched_task_result,
        );
        record.task_id = *workflow_id.as_ref();

        let recovered = recover_persisted_autonomy_status(&record)
            .expect("metadata.final_result should recover failed-before-graph parity");

        assert_eq!(recovered.graph.workflow_id, workflow_id);
        assert_eq!(recovered.graph.state, GraphState::Failed);
        let preview = recovered
            .result_preview
            .expect("recovered view should expose a synthesized result preview");
        assert_eq!(preview.payload_location, "metadata.final_result");
        assert_eq!(
            preview.proof_outcome,
            ProofOutcomeClassification::FailedBeforeGraph
        );
        assert_eq!(
            preview.preview_text.as_deref(),
            Some("planner execution failed: Ask operation timed out")
        );
    }

    #[test]
    fn mark_persisted_autonomy_status_failed_rewrites_running_projection() {
        let mut metadata = json!({});
        let view = sample_autonomy_view_with_states(GraphState::Running, BranchState::Running);
        persist_autonomy_status(&mut metadata, &view);

        mark_persisted_autonomy_status_failed(&mut metadata);

        let record = sample_task_with_metadata(metadata);
        let recovered = recover_persisted_autonomy_status(&record)
            .expect("rewritten autonomy snapshot should still deserialize");

        assert_eq!(recovered.graph.state, GraphState::Failed);
        assert_eq!(recovered.branches.len(), 1);
        assert_eq!(recovered.branches[0].state, BranchState::Failed);
    }

    #[test]
    fn should_recover_orphaned_workflow_when_previous_process_left_it_running() {
        let mut record = sample_task_with_metadata(json!({}));
        record.status = "running".to_string();
        record.created_at = Utc::now() - chrono::Duration::minutes(5);
        record.started_at = Some(Utc::now() - chrono::Duration::minutes(4));

        assert!(should_recover_orphaned_workflow(&record, Utc::now(), false,));
    }

    #[test]
    fn should_not_recover_current_process_workflow_without_graph_yet() {
        let mut record = sample_task_with_metadata(json!({}));
        record.status = "queued".to_string();
        record.created_at = Utc::now();
        record.started_at = None;

        assert!(!should_recover_orphaned_workflow(
            &record,
            Utc::now() - chrono::Duration::seconds(1),
            false,
        ));
    }

    #[test]
    fn should_not_recover_workflow_that_is_still_live_in_memory() {
        let mut record = sample_task_with_metadata(json!({}));
        record.status = "running".to_string();
        record.created_at = Utc::now() - chrono::Duration::minutes(5);
        record.started_at = Some(Utc::now() - chrono::Duration::minutes(4));

        assert!(!should_recover_orphaned_workflow(&record, Utc::now(), true,));
    }

    #[test]
    fn retain_replay_history_after_compaction_keeps_tail_and_snapshot_events() {
        let workflow_id = TaskId::new();
        let mut history = vec![
            WorkflowHistoryEventRecord {
                workflow_id,
                event_id: Uuid::new_v4(),
                replay_position: 8,
                event_kind: DurableWorkflowEventKind::NodeStateChanged,
                recorded_at: Utc::now(),
                actor_agent_id: None,
                source: Some("test".to_string()),
                branch_id: None,
                node_id: None,
                lifecycle_state: None,
                effect_boundary_id: None,
                compaction_id: None,
                parent_event_id: None,
                payload: json!({}),
            },
            WorkflowHistoryEventRecord {
                workflow_id,
                event_id: Uuid::new_v4(),
                replay_position: 9,
                event_kind: DurableWorkflowEventKind::NodeStateChanged,
                recorded_at: Utc::now(),
                actor_agent_id: None,
                source: Some("test".to_string()),
                branch_id: None,
                node_id: None,
                lifecycle_state: None,
                effect_boundary_id: None,
                compaction_id: None,
                parent_event_id: None,
                payload: json!({}),
            },
            WorkflowHistoryEventRecord {
                workflow_id,
                event_id: Uuid::new_v4(),
                replay_position: 10,
                event_kind: DurableWorkflowEventKind::HistoryCompacted,
                recorded_at: Utc::now(),
                actor_agent_id: None,
                source: Some("test".to_string()),
                branch_id: None,
                node_id: None,
                lifecycle_state: None,
                effect_boundary_id: None,
                compaction_id: None,
                parent_event_id: None,
                payload: json!({}),
            },
        ];
        let compaction = HistoryCompactionRecord {
            workflow_id,
            compaction_id: Uuid::new_v4(),
            mode: HistoryCompactionMode::ReplayPointer,
            source_replay_start: 1,
            source_replay_end: 8,
            replay_start_position: 10,
            replacement_event_id: None,
            preserved_lineage_note: "test".to_string(),
            recorded_at: Utc::now(),
        };

        retain_replay_history_after_compaction(&mut history, &compaction);

        assert_eq!(
            history
                .iter()
                .map(|event| event.replay_position)
                .collect::<Vec<_>>(),
            vec![9, 10]
        );
    }
}

fn normalize_runtime_plan(goal: &str, context: &Value, raw_plan: Value) -> Value {
    let mut plan = normalize_planner_output(goal, context, raw_plan);
    let steps = plan
        .get("steps")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let preserve_explicit_graph = planner_output_supports_explicit_runtime_graph(&steps);

    let runtime_steps = if preserve_explicit_graph {
        normalize_explicit_runtime_steps(goal, &steps)
    } else {
        let mut runtime_steps = Vec::new();
        let mut previous_step_id: Option<String> = None;

        for (index, raw_step) in steps.iter().enumerate() {
            let mut step = raw_step.as_object().cloned().unwrap_or_default();
            let step_id = step
                .get("id")
                .and_then(Value::as_str)
                .map(ToString::to_string)
                .unwrap_or_else(|| format!("step-{}", index + 1));
            step.insert("id".to_string(), json!(step_id.clone()));
            step.insert("step".to_string(), json!(index + 1));
            step.entry("action".to_string())
                .or_insert_with(|| json!("execute"));
            step.entry("description".to_string())
                .or_insert_with(|| json!(goal));

            if index == 0 {
                step.insert("role".to_string(), json!("worker"));
                step.insert("branch".to_string(), json!("branch-a"));
                step.insert("depends_on".to_string(), json!([]));
            } else {
                step.entry("role".to_string())
                    .or_insert_with(|| json!("worker"));
                step.insert("branch".to_string(), json!("branch-a"));
                let deps = previous_step_id.iter().cloned().collect::<Vec<String>>();
                step.insert("depends_on".to_string(), json!(deps));
            }

            previous_step_id = Some(step_id.clone());
            runtime_steps.push(Value::Object(step));
        }

        runtime_steps
    };

    let topology_hint = if preserve_explicit_graph {
        plan.get("topology_hint")
            .and_then(Value::as_str)
            .map(canonicalize_topology_hint)
    } else {
        Some("sequential".to_string())
    };

    if let Some(object) = plan.as_object_mut() {
        object.insert("goal".to_string(), json!(goal));
        object.insert("context".to_string(), context.clone());
        object.insert("steps".to_string(), Value::Array(runtime_steps.clone()));
        if let Some(topology_hint) = topology_hint {
            object.insert("topology_hint".to_string(), json!(topology_hint));
        } else {
            object.remove("topology_hint");
        }
        object.insert(
            "runtime_normalized".to_string(),
            json!(runtime_steps.len() >= 2),
        );
    }

    plan
}

fn task_assignment_for_node(
    workflow_id: TaskId,
    node: &mister_smith_agents::ExecutionNode,
    goal: &str,
) -> TaskAssignment {
    TaskAssignment {
        task_id: TaskId::from_uuid(*node.node_id.as_ref()),
        task_type: node.action.clone(),
        priority: 128,
        deadline: None,
        input: json!({
            "workflow_id": workflow_id,
            "goal": goal,
            "step_id": node.step_key,
            "description": node.description,
            "action": node.action,
            "branch_id": node.branch_id,
            "dependencies": node.dependencies,
            "planner_metadata": node.metadata,
        }),
        output: None,
        state: TaskState::Pending,
        assigned_to: None,
        parent_task_id: Some(workflow_id),
        team_id: None,
        message_id: Uuid::new_v4(),
        created_at: Utc::now(),
        assigned_at: None,
        completed_at: None,
        error_message: None,
    }
}

fn runtime_supervision_target(
    graph: &ExecutionGraph,
    task: &TaskAssignment,
    prefer_branch: bool,
) -> Result<GuardTarget, String> {
    let node_id = ExecutionNodeId::from_uuid(*task.task_id.as_ref());
    let node = graph
        .nodes
        .iter()
        .find(|node| node.node_id == node_id)
        .ok_or_else(|| {
            format!(
                "workflow step '{}' is missing from execution graph {}",
                step_identifier(task),
                graph.graph_id
            )
        })?;

    Ok(if prefer_branch {
        GuardTarget::Branch(node.branch_id)
    } else {
        GuardTarget::Node(node.node_id)
    })
}

fn runtime_checkpoints_for_target(
    graph: &ExecutionGraph,
    target: &GuardTarget,
) -> Vec<BranchCheckpoint> {
    let branch_id = match target {
        GuardTarget::Branch(branch_id) => Some(*branch_id),
        GuardTarget::Node(node_id) => graph
            .nodes
            .iter()
            .find(|node| node.node_id == *node_id)
            .map(|node| node.branch_id),
        GuardTarget::Graph(_) | GuardTarget::Provider(_) => None,
    };

    branch_id
        .and_then(|branch_id| graph.latest_checkpoint(&branch_id).cloned())
        .into_iter()
        .collect()
}

fn runtime_fingerprint_target_kind(target: &GuardTarget) -> &'static str {
    match target {
        GuardTarget::Node(_) => "agent",
        GuardTarget::Branch(_) => "branch",
        GuardTarget::Graph(_) => "topology",
        GuardTarget::Provider(_) => "provider",
    }
}

fn runtime_fingerprint_target_selector(target: &GuardTarget) -> String {
    match target {
        GuardTarget::Node(node_id) => node_id.to_string(),
        GuardTarget::Branch(branch_id) => branch_id.to_string(),
        GuardTarget::Graph(graph_id) => graph_id.to_string(),
        GuardTarget::Provider(provider) => provider.clone(),
    }
}

fn runtime_health_label(health_state: mister_smith_core::HealthState) -> &'static str {
    match health_state {
        mister_smith_core::HealthState::Healthy => "healthy",
        mister_smith_core::HealthState::Degraded => "degraded",
        mister_smith_core::HealthState::Unhealthy => "unhealthy",
        mister_smith_core::HealthState::Unknown => "unknown",
    }
}

fn runtime_signal_kind_label(signal_kind: SemanticSignalKind) -> &'static str {
    match signal_kind {
        SemanticSignalKind::Stalled => "stalled",
        SemanticSignalKind::Repetitive => "repetitive",
        SemanticSignalKind::LowConfidence => "low_confidence",
        SemanticSignalKind::MissingContext => "missing_context",
        SemanticSignalKind::PolicyConflict => "policy_conflict",
    }
}

fn runtime_intervention_label(intervention: mister_smith_core::InterventionType) -> &'static str {
    match intervention {
        mister_smith_core::InterventionType::Retry => "retry",
        mister_smith_core::InterventionType::Failover => "failover",
        mister_smith_core::InterventionType::ContextRefresh => "context_refresh",
        mister_smith_core::InterventionType::BranchIsolation => "branch_isolation",
        mister_smith_core::InterventionType::Reassignment => "reassignment",
        mister_smith_core::InterventionType::Escalation => "escalation",
        mister_smith_core::InterventionType::Abort => "abort",
    }
}

fn runtime_fingerprint_confidence(
    snapshot: &mister_smith_core::ProfileSnapshot,
    existing: Option<&ProfileFingerprint>,
) -> f32 {
    let live = snapshot
        .semantic_signals
        .iter()
        .map(|signal| f32::from(signal.severity) / 100.0)
        .fold(0.55, f32::max)
        .min(0.95);
    existing
        .map(|fingerprint| fingerprint.confidence.max(live))
        .unwrap_or(live)
}

fn build_runtime_profile_fingerprint(
    workflow_id: TaskId,
    target: &GuardTarget,
    snapshot: &mister_smith_core::ProfileSnapshot,
    decision: &mister_smith_core::GuardDecision,
    existing: Option<&ProfileFingerprint>,
) -> ProfileFingerprint {
    let target_kind = runtime_fingerprint_target_kind(target).to_string();
    let target_selector = runtime_fingerprint_target_selector(target);
    let updated_at = Utc::now();
    let dominant_failure_modes = snapshot
        .semantic_signals
        .iter()
        .map(|signal| runtime_signal_kind_label(signal.signal_kind).to_string())
        .collect::<Vec<_>>();
    let source_refs = std::iter::once(format!("workflow:{workflow_id}"))
        .chain(
            decision
                .evidence
                .checkpoint_ids
                .iter()
                .map(|checkpoint_id| format!("checkpoint:{checkpoint_id}")),
        )
        .collect::<Vec<_>>();

    ProfileFingerprint {
        fingerprint_id: existing
            .map(|fingerprint| fingerprint.fingerprint_id)
            .unwrap_or_default(),
        target_kind,
        target_selector,
        source_refs,
        summary_payload: json!({
            "health_state": runtime_health_label(snapshot.health_state),
            "signal_kinds": snapshot
                .semantic_signals
                .iter()
                .map(|signal| runtime_signal_kind_label(signal.signal_kind))
                .collect::<Vec<_>>(),
            "decision_basis": decision.evidence.decision_basis.as_str(),
            "intervention": runtime_intervention_label(decision.intervention),
        }),
        dominant_failure_modes,
        preferred_interventions: vec![decision.intervention],
        confidence: runtime_fingerprint_confidence(snapshot, existing),
        expires_at: updated_at + chrono::Duration::hours(6),
        updated_at,
    }
}

fn runtime_supervision_plan(
    graph: &ExecutionGraph,
    task: &TaskAssignment,
    signal: SemanticSignal,
    notes: Vec<String>,
    prefer_branch: bool,
) -> Result<RuntimeSupervisionPlan, String> {
    let target = runtime_supervision_target(graph, task, prefer_branch)?;
    let checkpoints = runtime_checkpoints_for_target(graph, &target);
    let assessment = ProfileAssessment::from_supervisory_signals(&target, vec![signal], notes);

    Ok(RuntimeSupervisionPlan {
        target,
        assessment,
        checkpoints,
    })
}

fn runtime_supervision_plan_for_repair_evaluation(
    graph: &ExecutionGraph,
    task: &TaskAssignment,
    step_evaluation: &StepEvaluationRecord,
) -> Result<Option<RuntimeSupervisionPlan>, String> {
    let Some(repair_directive) = step_evaluation.repair_directive.as_ref() else {
        return Ok(None);
    };

    let step_id = step_identifier(task);
    let mut notes = vec![format!(
        "supported task-path local repair triggered for step '{step_id}': {}",
        step_evaluation.reason
    )];
    if let Some(checkpoint_ref) = step_evaluation.checkpoint_ref.as_ref() {
        notes.push(format!("repair checkpoint hint: {checkpoint_ref}"));
    }

    let (signal_kind, severity, detail, prefer_branch) = match repair_directive.action {
        RepairDirectiveAction::ClarifyHandoff => {
            let missing = step_evaluation
                .clarification_request
                .as_ref()
                .map(|request| request.missing_constraints.join(", "))
                .filter(|constraints| !constraints.is_empty())
                .unwrap_or_else(|| step_evaluation.reason.clone());
            notes.push("preferred node-scoped clarification before wider escalation".to_string());
            (SemanticSignalKind::MissingContext, 72, missing, false)
        }
        RepairDirectiveAction::RetryStep => {
            notes.push("preferred node-scoped retry before widening repair scope".to_string());
            (
                SemanticSignalKind::Stalled,
                78,
                step_evaluation.reason.clone(),
                false,
            )
        }
        RepairDirectiveAction::ReplanFromCheckpoint => {
            notes.push(
                "preferred node-scoped checkpoint replan before widening repair scope".to_string(),
            );
            (
                SemanticSignalKind::Stalled,
                82,
                step_evaluation.reason.clone(),
                false,
            )
        }
        RepairDirectiveAction::Stop => {
            notes.push("repair loop stopped and requires branch-scoped escalation".to_string());
            (
                SemanticSignalKind::PolicyConflict,
                95,
                step_evaluation.reason.clone(),
                true,
            )
        }
    };

    runtime_supervision_plan(
        graph,
        task,
        SemanticSignal {
            signal_kind,
            severity,
            detail,
        },
        notes,
        prefer_branch,
    )
    .map(Some)
}

fn runtime_supervision_plan_for_failed_step(
    graph: &ExecutionGraph,
    task: &TaskAssignment,
    failed_step: &FailedTaskExecution,
) -> Result<Option<RuntimeSupervisionPlan>, String> {
    let Some(step_evaluation) = failed_step.step_evaluation.as_ref() else {
        return Ok(None);
    };

    let step_id = step_identifier(task);
    let detail = if failed_step.message.trim().is_empty() {
        step_evaluation.reason.clone()
    } else {
        failed_step.message.clone()
    };
    let mut notes = vec![format!(
        "supported task-path failure requires branch-scoped intervention for step '{step_id}'"
    )];
    notes.push(format!("failure detail: {detail}"));
    if let Some(failure_code) = step_evaluation.failure_code.as_ref() {
        notes.push(format!("failure code: {failure_code}"));
    }
    if let Some(checkpoint_ref) = step_evaluation.checkpoint_ref.as_ref() {
        notes.push(format!("latest checkpoint ref: {checkpoint_ref}"));
    }

    runtime_supervision_plan(
        graph,
        task,
        SemanticSignal {
            signal_kind: SemanticSignalKind::PolicyConflict,
            severity: 95,
            detail,
        },
        notes,
        true,
    )
    .map(Some)
}

fn execution_input_for_task(
    graph: &ExecutionGraph,
    task: &TaskAssignment,
    goal: &str,
    worker_id: AgentId,
    step_results: &BTreeMap<String, Value>,
    runtime_llm: &RuntimeLlmSelection,
) -> Value {
    let node_id = ExecutionNodeId::from_uuid(*task.task_id.as_ref());
    let dependency_results = graph
        .nodes
        .iter()
        .find(|node| node.node_id == node_id)
        .map(|node| {
            node.dependencies
                .iter()
                .filter_map(|dependency| {
                    let key = TaskId::from_uuid(*dependency.as_ref()).to_string();
                    step_results.get(&key).cloned()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    json!({
        "workflow_goal": goal,
        "provider_kind": runtime_llm.provider_kind_name,
        "model_id": runtime_llm.model_id,
        "worker_id": worker_id,
        "task": task.input,
        "dependency_results": dependency_results,
    })
}

fn step_identifier(task: &TaskAssignment) -> String {
    task.input
        .get("id")
        .and_then(Value::as_str)
        .or_else(|| task.input.get("step_id").and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| task.task_id.to_string())
}

fn last_stable_step_identifier(task: &TaskAssignment) -> Option<String> {
    task.input
        .get("dependencies")
        .and_then(Value::as_array)
        .and_then(|dependencies| dependencies.last())
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn runtime_repair_action(task: &TaskAssignment) -> Option<RepairDirectiveAction> {
    let action = task
        .input
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or(task.task_type.as_str())
        .trim()
        .to_ascii_lowercase();
    let description = task
        .input
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();

    if action.contains("clarify")
        || action.contains("missing_context")
        || action.contains("handoff_context")
        || (action.contains("resolve")
            && (description.contains("missing required context")
                || description.contains("request the minimum clarification")
                || description.contains("bounded local repair")))
    {
        return Some(RepairDirectiveAction::ClarifyHandoff);
    }
    if action.contains("replan") {
        return Some(RepairDirectiveAction::ReplanFromCheckpoint);
    }
    if action.contains("retry") || action.contains("repair") {
        return Some(RepairDirectiveAction::RetryStep);
    }

    None
}

fn runtime_repair_reason(repair_action: RepairDirectiveAction) -> &'static str {
    match repair_action {
        RepairDirectiveAction::ClarifyHandoff => {
            "runtime executed planner-directed clarification step on description-only ingress"
        }
        RepairDirectiveAction::RetryStep => {
            "runtime executed planner-directed retry step on description-only ingress"
        }
        RepairDirectiveAction::ReplanFromCheckpoint => {
            "runtime executed planner-directed replan step on description-only ingress"
        }
        RepairDirectiveAction::Stop => {
            "runtime executed planner-directed stop step on description-only ingress"
        }
    }
}

fn runtime_repair_step_evaluation(
    workflow_id: TaskId,
    task: &TaskAssignment,
    repair_state: &LocalRepairState,
) -> Option<StepEvaluationRecord> {
    let repair_action = runtime_repair_action(task)?;
    if repair_action == RepairDirectiveAction::Stop {
        return None;
    }

    let step_id = step_identifier(task);
    let last_stable_step_id = repair_state
        .failure_context_checkpoint
        .as_ref()
        .and_then(|checkpoint| checkpoint.last_stable_step_id.clone())
        .or_else(|| last_stable_step_identifier(task));
    let checkpoint_ref = repair_state
        .failure_context_checkpoint
        .as_ref()
        .and_then(|checkpoint| checkpoint.checkpoint_ref.clone())
        .or_else(|| {
            last_stable_step_id
                .as_ref()
                .map(|step_id| format!("planner-step:{step_id}"))
        });
    let failure_context_ref = repair_state
        .failure_context_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.failure_context_ref.clone())
        .unwrap_or_else(|| format!("planner:{}/{}", step_id, repair_action.as_str()));
    let attempt_count = match repair_action {
        RepairDirectiveAction::ClarifyHandoff => repair_state
            .clarification_request
            .as_ref()
            .map(|request| request.attempt_count + 1)
            .unwrap_or(1),
        RepairDirectiveAction::RetryStep | RepairDirectiveAction::ReplanFromCheckpoint => {
            repair_state
                .failure_context_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.attempt_count + 1)
                .unwrap_or(1)
        }
        RepairDirectiveAction::Stop => 0,
    };
    let reason = runtime_repair_reason(repair_action).to_string();
    let failure_code = repair_state
        .failure_context_checkpoint
        .as_ref()
        .and_then(|checkpoint| checkpoint.failure_code.clone());
    let clarification_request = if repair_action == RepairDirectiveAction::ClarifyHandoff {
        Some(HandoffClarificationRequest {
            source_step_id: repair_state
                .clarification_request
                .as_ref()
                .map(|request| request.source_step_id.clone())
                .or_else(|| last_stable_step_id.clone())
                .unwrap_or_else(|| step_id.clone()),
            target_step_id: repair_state
                .clarification_request
                .as_ref()
                .map(|request| request.target_step_id.clone())
                .unwrap_or_else(|| step_id.clone()),
            missing_constraints: repair_state
                .clarification_request
                .as_ref()
                .map(|request| request.missing_constraints.clone())
                .filter(|constraints| !constraints.is_empty())
                .unwrap_or_else(|| vec!["required context".to_string()]),
            attempt_count,
            expires_at: repair_state
                .clarification_request
                .as_ref()
                .and_then(|request| request.expires_at),
        })
    } else {
        repair_state.clarification_request.clone()
    };

    Some(StepEvaluationRecord {
        workflow_id,
        step_id: step_id.clone(),
        verdict: VerifierVerdict::Accepted,
        confidence: None,
        reason: reason.clone(),
        failure_code: failure_code.clone(),
        checkpoint_ref: checkpoint_ref.clone(),
        repair_directive: Some(RepairDirective {
            action: repair_action,
            issued_by: "runtime.planner_repair".to_string(),
            failure_context_ref: failure_context_ref.clone(),
            retry_budget_remaining: 0,
        }),
        clarification_request,
        failure_context_checkpoint: Some(FailureContextCheckpoint {
            failed_step_id: repair_state
                .failure_context_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.failed_step_id.clone())
                .unwrap_or_else(|| step_id.clone()),
            last_stable_step_id,
            checkpoint_ref,
            failure_context_ref,
            failure_code,
            reason,
            attempt_count,
        }),
    })
}

fn parse_injected_verifier_policy(
    task: &TaskAssignment,
    policy: &Value,
    label: &str,
) -> Result<Option<InjectedVerifierPolicy>, String> {
    let enabled = policy
        .get("enabled")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            format!(
                "workflow step '{}' has invalid {label}.enabled",
                step_identifier(task)
            )
        })?;

    if !enabled {
        return Ok(None);
    }

    serde_json::from_value::<InjectedVerifierPolicy>(policy.clone())
        .map(Some)
        .map_err(|error| {
            format!(
                "workflow step '{}' has invalid {label}: {error}",
                step_identifier(task)
            )
        })
}

fn verifier_policy_from_task_input(
    task: &TaskAssignment,
) -> Result<Option<InjectedVerifierPolicy>, String> {
    let Some(policy) = task.input.get("verifier_policy") else {
        return Ok(None);
    };

    parse_injected_verifier_policy(task, policy, "verifier_policy")
}

fn verifier_policies_from_task_input(
    task: &TaskAssignment,
) -> Result<Vec<InjectedVerifierPolicy>, String> {
    if let Some(sequence) = task.input.get("verifier_policy_sequence") {
        let entries = sequence.as_array().ok_or_else(|| {
            format!(
                "workflow step '{}' has invalid verifier_policy_sequence",
                step_identifier(task)
            )
        })?;
        let mut parsed = Vec::new();
        for (index, policy) in entries.iter().enumerate() {
            if let Some(policy) = parse_injected_verifier_policy(
                task,
                policy,
                &format!("verifier_policy_sequence[{index}]"),
            )? {
                parsed.push(policy);
            }
        }
        return Ok(parsed);
    }

    Ok(verifier_policy_from_task_input(task)?.into_iter().collect())
}

fn build_step_evaluation_record(
    workflow_id: TaskId,
    task: &TaskAssignment,
    policy: InjectedVerifierPolicy,
    repair_state: &LocalRepairState,
) -> Result<StepEvaluationRecord, String> {
    let step_id = step_identifier(task);
    let verdict = policy.verdict.ok_or_else(|| {
        format!("workflow step '{step_id}' enabled verifier_policy missing verdict")
    })?;
    let reason = policy
        .reason
        .map(|reason| reason.trim().to_string())
        .filter(|reason| !reason.is_empty())
        .ok_or_else(|| {
            format!("workflow step '{step_id}' enabled verifier_policy missing reason")
        })?;

    let repair_directive = match verdict {
        VerifierVerdict::Accepted => {
            if policy.repair_directive.is_some() {
                return Err(format!(
                    "workflow step '{step_id}' accepted verifier_policy must not include repair_directive"
                ));
            }
            None
        }
        VerifierVerdict::Rejected => Some(policy.repair_directive.ok_or_else(|| {
            format!("workflow step '{step_id}' rejected verifier_policy missing repair_directive")
        })?),
    };

    let clarification_request = match repair_directive.as_ref().map(|directive| directive.action) {
        Some(RepairDirectiveAction::ClarifyHandoff) => {
            let request = repair_state
                .clarification_request
                .clone()
                .or(policy.clarification_request)
                .ok_or_else(|| {
                    format!(
                        "workflow step '{step_id}' clarify_handoff repair missing clarification_request"
                    )
                })?;
            Some(HandoffClarificationRequest {
                attempt_count: request.attempt_count + 1,
                ..request
            })
        }
        _ => repair_state.clarification_request.clone(),
    };

    let failure_context_checkpoint = if verdict == VerifierVerdict::Rejected {
        let failure_context_ref = repair_directive
            .as_ref()
            .map(|directive| directive.failure_context_ref.clone())
            .ok_or_else(|| {
                format!(
                    "workflow step '{step_id}' rejected verifier_policy missing failure_context_ref"
                )
            })?;
        Some(FailureContextCheckpoint {
            failed_step_id: step_id.clone(),
            last_stable_step_id: last_stable_step_identifier(task),
            checkpoint_ref: policy.checkpoint_ref.clone(),
            failure_context_ref,
            failure_code: policy.failure_code.clone(),
            reason: reason.clone(),
            attempt_count: repair_state
                .failure_context_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.attempt_count + 1)
                .unwrap_or(1),
        })
    } else {
        repair_state.failure_context_checkpoint.clone()
    };

    Ok(StepEvaluationRecord {
        workflow_id,
        step_id,
        verdict,
        confidence: policy.confidence,
        reason,
        failure_code: policy.failure_code,
        checkpoint_ref: policy.checkpoint_ref,
        repair_directive,
        clarification_request,
        failure_context_checkpoint,
    })
}

fn repair_state_from_evaluation(step_evaluation: &StepEvaluationRecord) -> LocalRepairState {
    LocalRepairState {
        clarification_request: step_evaluation.clarification_request.clone(),
        failure_context_checkpoint: step_evaluation.failure_context_checkpoint.clone(),
    }
}

fn repair_budget_exhausted(step_evaluation: &StepEvaluationRecord) -> bool {
    let Some(repair_directive) = step_evaluation.repair_directive.as_ref() else {
        return false;
    };

    match repair_directive.action {
        RepairDirectiveAction::ClarifyHandoff => step_evaluation
            .clarification_request
            .as_ref()
            .map(|request| request.attempt_count > repair_directive.retry_budget_remaining)
            .unwrap_or(true),
        RepairDirectiveAction::RetryStep | RepairDirectiveAction::ReplanFromCheckpoint => {
            step_evaluation
                .failure_context_checkpoint
                .as_ref()
                .map(|checkpoint| {
                    checkpoint.attempt_count > repair_directive.retry_budget_remaining
                })
                .unwrap_or(true)
        }
        RepairDirectiveAction::Stop => false,
    }
}

fn apply_local_repair_context(
    execution_input: &mut Value,
    repair_action: RepairDirectiveAction,
    repair_state: &LocalRepairState,
) {
    let Some(task) = execution_input
        .get_mut("task")
        .and_then(Value::as_object_mut)
    else {
        return;
    };

    task.insert("repair_action".to_string(), json!(repair_action.as_str()));

    if let Some(clarification_request) = repair_state.clarification_request.as_ref() {
        task.insert(
            "clarification_request".to_string(),
            serde_json::to_value(clarification_request)
                .expect("clarification request should serialize"),
        );
    } else {
        task.remove("clarification_request");
    }

    if let Some(failure_context_checkpoint) = repair_state.failure_context_checkpoint.as_ref() {
        task.insert(
            "failure_context_checkpoint".to_string(),
            serde_json::to_value(failure_context_checkpoint)
                .expect("failure context checkpoint should serialize"),
        );
        task.insert(
            "repair_attempt_count".to_string(),
            json!(failure_context_checkpoint.attempt_count),
        );
    } else {
        task.remove("failure_context_checkpoint");
        task.remove("repair_attempt_count");
    }
}

fn rejected_step_message(step_evaluation: &StepEvaluationRecord) -> String {
    let repair_action = step_evaluation
        .repair_directive
        .as_ref()
        .map(|directive| directive.action)
        .unwrap_or(RepairDirectiveAction::Stop);

    match repair_action {
        RepairDirectiveAction::ClarifyHandoff => {
            let missing = step_evaluation
                .clarification_request
                .as_ref()
                .map(|request| request.missing_constraints.join(", "))
                .filter(|missing| !missing.is_empty())
                .unwrap_or_else(|| "missing clarification details".to_string());
            format!(
                "workflow step '{}' requested clarification: {} (repair={})",
                step_evaluation.step_id,
                missing,
                repair_action.as_str()
            )
        }
        RepairDirectiveAction::ReplanFromCheckpoint => {
            let checkpoint_ref = step_evaluation
                .failure_context_checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.checkpoint_ref.as_deref())
                .unwrap_or("unknown");
            format!(
                "workflow step '{}' rejected by verifier: {} (repair={} checkpoint={checkpoint_ref})",
                step_evaluation.step_id,
                step_evaluation.reason,
                repair_action.as_str()
            )
        }
        _ => format!(
            "workflow step '{}' rejected by verifier: {} (repair={})",
            step_evaluation.step_id,
            step_evaluation.reason,
            repair_action.as_str()
        ),
    }
}

fn repair_failure(
    task: &TaskAssignment,
    worker_id: AgentId,
    branch_id: Value,
    action: String,
    result: Value,
    step_evaluation: StepEvaluationRecord,
    suffix: Option<&str>,
) -> FailedTaskExecution {
    let mut message = rejected_step_message(&step_evaluation);
    if let Some(suffix) = suffix {
        message.push_str(&format!(" ({suffix})"));
    }

    FailedTaskExecution {
        completed_steps: Vec::new(),
        task_id: task.task_id,
        worker_id,
        branch_id,
        action,
        message,
        result: Some(result),
        step_evaluation: Some(step_evaluation),
    }
}

fn step_execution_disposition(
    context: &StepExecutionContext<'_>,
    result: Value,
    policy: Option<InjectedVerifierPolicy>,
) -> Result<StepExecutionDisposition, Box<FailedTaskExecution>> {
    let Some(policy) = policy else {
        let step_evaluation =
            runtime_repair_step_evaluation(context.workflow_id, context.task, context.repair_state);
        return Ok(StepExecutionDisposition::Completed(
            CompletedTaskExecution {
                task_id: context.task.task_id,
                worker_id: context.worker_id,
                branch_id: context.branch_id.clone(),
                action: context.action.clone(),
                result,
                step_evaluation,
            },
        ));
    };

    let step_evaluation = build_step_evaluation_record(
        context.workflow_id,
        context.task,
        policy,
        context.repair_state,
    )
    .map_err(|message| {
        Box::new(FailedTaskExecution {
            completed_steps: Vec::new(),
            task_id: context.task.task_id,
            worker_id: context.worker_id,
            branch_id: context.branch_id.clone(),
            action: context.action.clone(),
            message,
            result: Some(result.clone()),
            step_evaluation: None,
        })
    })?;

    if step_evaluation.verdict == VerifierVerdict::Rejected {
        let repair_action = step_evaluation
            .repair_directive
            .as_ref()
            .map(|directive| directive.action)
            .unwrap_or(RepairDirectiveAction::Stop);

        if repair_action == RepairDirectiveAction::Stop {
            return Ok(StepExecutionDisposition::Failed(repair_failure(
                context.task,
                context.worker_id,
                context.branch_id.clone(),
                context.action.clone(),
                result,
                step_evaluation,
                None,
            )));
        }

        if repair_budget_exhausted(&step_evaluation) {
            return Ok(StepExecutionDisposition::Failed(repair_failure(
                context.task,
                context.worker_id,
                context.branch_id.clone(),
                context.action.clone(),
                result,
                step_evaluation,
                Some("local repair budget exhausted"),
            )));
        }

        return Ok(StepExecutionDisposition::Continue(LocalRepairTransition {
            repair_action,
            repair_state: repair_state_from_evaluation(&step_evaluation),
            step_evaluation,
            result,
        }));
    }

    Ok(StepExecutionDisposition::Completed(
        CompletedTaskExecution {
            task_id: context.task.task_id,
            worker_id: context.worker_id,
            branch_id: context.branch_id.clone(),
            action: context.action.clone(),
            result,
            step_evaluation: Some(step_evaluation),
        },
    ))
}

#[cfg(test)]
fn gate_step_execution_result(
    workflow_id: TaskId,
    task: &TaskAssignment,
    worker_id: AgentId,
    result: Value,
) -> Result<CompletedTaskExecution, Box<FailedTaskExecution>> {
    let branch_id = task.input.get("branch_id").cloned().unwrap_or(Value::Null);
    let action = task.task_type.clone();
    let policy = verifier_policy_from_task_input(task).map_err(|message| {
        Box::new(FailedTaskExecution {
            completed_steps: Vec::new(),
            task_id: task.task_id,
            worker_id,
            branch_id: branch_id.clone(),
            action: action.clone(),
            message,
            result: Some(result.clone()),
            step_evaluation: None,
        })
    })?;
    let repair_state = LocalRepairState::default();
    let context = StepExecutionContext {
        workflow_id,
        task,
        worker_id,
        branch_id: branch_id.clone(),
        action: action.clone(),
        repair_state: &repair_state,
    };

    match step_execution_disposition(&context, result, policy) {
        Ok(StepExecutionDisposition::Completed(completed_step)) => Ok(completed_step),
        Ok(StepExecutionDisposition::Failed(failed_step)) => Err(Box::new(failed_step)),
        Ok(StepExecutionDisposition::Continue(transition)) => Err(Box::new(repair_failure(
            task,
            worker_id,
            branch_id,
            action,
            transition.result,
            transition.step_evaluation,
            Some("no follow-up verifier policy available for local repair"),
        ))),
        Err(failed_step) => Err(failed_step),
    }
}

fn build_step_result_payload(
    task_id: TaskId,
    worker_id: AgentId,
    branch_id: Value,
    action: String,
    result: Option<Value>,
    step_evaluation: Option<StepEvaluationRecord>,
) -> Value {
    let mut payload = serde_json::Map::from_iter([
        ("task_id".to_string(), json!(task_id)),
        ("worker_id".to_string(), json!(worker_id)),
        ("branch_id".to_string(), branch_id),
        ("action".to_string(), json!(action)),
    ]);

    if let Some(result) = result {
        payload.insert("result".to_string(), result);
    }

    if let Some(step_evaluation) = step_evaluation {
        payload.insert(
            "step_evaluation".to_string(),
            serde_json::to_value(step_evaluation).expect("step evaluation should serialize"),
        );
    }

    Value::Object(payload)
}

#[cfg(test)]
mod runtime_plan_tests {
    use super::*;

    fn simulated_step_result(execution_input: &Value) -> Value {
        let mut object = execution_input
            .as_object()
            .cloned()
            .expect("execution input should be an object");
        object.insert("status".to_string(), json!("completed"));
        object.insert("execution_boundary".to_string(), json!("tool_bus"));
        object.insert(
            "tool_name".to_string(),
            json!(format!(
                "{WORKFLOW_TOOL_NAMESPACE}.{WORKFLOW_EXECUTE_STEP_TOOL}"
            )),
        );
        Value::Object(object)
    }

    fn run_local_repair_sequence(
        workflow_id: TaskId,
        task: &TaskAssignment,
        worker_id: AgentId,
    ) -> Result<(CompletedTaskExecution, Value), FailedTaskExecution> {
        let branch_id = task.input.get("branch_id").cloned().unwrap_or(Value::Null);
        let action = task.task_type.clone();
        let verifier_policies = verifier_policies_from_task_input(task)
            .expect("verifier policies should parse for deterministic tests");
        let mut active_policy_index = 0usize;
        let mut repair_state = LocalRepairState::default();
        let mut execution_input = json!({
            "workflow_goal": "ship proof",
            "provider_kind": PROVIDER_KIND_NAME,
            "model_id": MODEL_ID,
            "worker_id": worker_id,
            "task": task.input.clone(),
            "dependency_results": [],
        });

        loop {
            let result = simulated_step_result(&execution_input);
            let context = StepExecutionContext {
                workflow_id,
                task,
                worker_id,
                branch_id: branch_id.clone(),
                action: action.clone(),
                repair_state: &repair_state,
            };
            match step_execution_disposition(
                &context,
                result,
                verifier_policies.get(active_policy_index).cloned(),
            )
            .expect("deterministic repair sequence should not fail parsing")
            {
                StepExecutionDisposition::Completed(completed) => {
                    return Ok((completed, execution_input));
                }
                StepExecutionDisposition::Failed(failed) => return Err(failed),
                StepExecutionDisposition::Continue(transition) => {
                    if verifier_policies.get(active_policy_index + 1).is_none() {
                        return Err(repair_failure(
                            task,
                            worker_id,
                            branch_id.clone(),
                            action.clone(),
                            transition.result,
                            transition.step_evaluation,
                            Some("no follow-up verifier policy available for local repair"),
                        ));
                    }

                    repair_state = transition.repair_state;
                    active_policy_index += 1;
                    apply_local_repair_context(
                        &mut execution_input,
                        transition.repair_action,
                        &repair_state,
                    );
                }
            }
        }
    }

    fn runtime_supervision_graph() -> (ExecutionGraph, TaskAssignment, TaskId, GuardTarget) {
        let workflow_id = TaskId::new();
        let graph = TopologyCompiler::default()
            .compile(
                workflow_id,
                &json!({
                    "goal": "runtime-supervision",
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
                        }
                    ]
                }),
                &TopologySignals::default(),
            )
            .expect("graph should compile");
        let node = graph
            .nodes
            .iter()
            .find(|node| node.step_key == "branch-a")
            .cloned()
            .expect("branch node should exist");
        let task = task_assignment_for_node(workflow_id, &node, "ship proof");

        (
            graph,
            task,
            workflow_id,
            GuardTarget::Branch(node.branch_id),
        )
    }

    #[test]
    fn runtime_supervision_strategy_avoids_root_shutdown_budget() {
        let strategy = runtime_supervision_strategy();

        assert_eq!(strategy.max_failures, 32);
        assert_eq!(strategy.failure_window, Duration::from_secs(60));
        assert_eq!(strategy.escalation_policy, EscalationPolicy::LogAndIgnore);
    }

    #[test]
    fn normalize_runtime_plan_reindexes_duplicate_numeric_steps() {
        let plan = normalize_runtime_plan(
            "ship proof",
            &json!({}),
            json!({
                "steps": [
                    {
                        "id": "plan-a",
                        "step": 1,
                        "action": "analyze",
                        "description": "worker a"
                    },
                    {
                        "id": "plan-b",
                        "step": 1,
                        "action": "analyze",
                        "description": "worker b"
                    }
                ]
            }),
        );

        let steps = plan["steps"].as_array().expect("normalized steps array");
        let numeric_steps: Vec<u64> = steps
            .iter()
            .filter_map(|step| step["step"].as_u64())
            .collect();

        assert_eq!(numeric_steps, vec![1, 2]);
        assert_eq!(steps[0]["branch"], json!("branch-a"));
        assert_eq!(steps[0]["depends_on"], json!([]));
        assert_eq!(steps[1]["branch"], json!("branch-a"));
        assert_eq!(steps[1]["depends_on"], json!(["plan-a"]));
        assert_eq!(plan["topology_hint"], json!("sequential"));
        assert_eq!(plan["runtime_normalized"], json!(true));
    }

    #[test]
    fn normalize_runtime_plan_defaults_plain_multistep_plans_to_sequential_chain() {
        let plan = normalize_runtime_plan(
            "ship proof",
            &json!({}),
            json!({
                "steps": [
                    {
                        "id": "step-1",
                        "step": 4,
                        "action": "inspect",
                        "description": "inspect runtime"
                    },
                    {
                        "id": "step-2",
                        "step": 7,
                        "action": "compare",
                        "description": "compare operator evidence"
                    },
                    {
                        "id": "step-3",
                        "step": 9,
                        "action": "answer",
                        "description": "return final answer"
                    }
                ]
            }),
        );

        let steps = plan["steps"].as_array().expect("normalized steps array");
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0]["depends_on"], json!([]));
        assert_eq!(steps[1]["depends_on"], json!(["step-1"]));
        assert_eq!(steps[2]["depends_on"], json!(["step-2"]));
        assert_eq!(steps[0]["branch"], json!("branch-a"));
        assert_eq!(steps[1]["branch"], json!("branch-a"));
        assert_eq!(steps[2]["branch"], json!("branch-a"));
        assert_eq!(plan["topology_hint"], json!("sequential"));
        assert_eq!(plan["runtime_normalized"], json!(true));
    }

    #[test]
    fn normalize_runtime_plan_preserves_explicit_harder_workload_graph() {
        let plan = normalize_runtime_plan(
            "ship proof",
            &json!({}),
            json!({
                "topology_hint": "hybrid",
                "steps": [
                    {
                        "id": "draft-outline",
                        "step": 7,
                        "action": "outline",
                        "description": "draft outline",
                        "role": "worker",
                        "branch": "outline"
                    },
                    {
                        "id": "collect-evidence",
                        "step": 7,
                        "action": "collect",
                        "description": "collect evidence",
                        "role": "worker",
                        "branch": "research"
                    },
                    {
                        "id": "synthesize",
                        "step": 9,
                        "action": "synthesize",
                        "description": "merge both branches",
                        "depends_on": ["draft-outline", "collect-evidence"]
                    },
                    {
                        "id": "fact-check",
                        "step": 11,
                        "action": "fact-check",
                        "description": "fact check merged draft",
                        "depends_on": ["synthesize"]
                    }
                ]
            }),
        );

        let steps = plan["steps"].as_array().expect("normalized steps array");
        let numeric_steps: Vec<u64> = steps
            .iter()
            .filter_map(|step| step["step"].as_u64())
            .collect();
        let branches: Vec<&str> = steps
            .iter()
            .map(|step| step["branch"].as_str().expect("branch label"))
            .collect();

        assert_eq!(numeric_steps, vec![1, 2, 3, 4]);
        assert_eq!(branches, vec!["outline", "research", "join", "join"]);
        assert_eq!(
            steps[2]["depends_on"],
            json!(["draft-outline", "collect-evidence"])
        );
        assert_eq!(steps[3]["depends_on"], json!(["synthesize"]));
        assert_eq!(steps[2]["role"], json!("coordinator"));
        assert_eq!(steps[3]["role"], json!("worker"));
        assert_eq!(plan["topology_hint"], json!("hybrid"));
        assert_eq!(plan["runtime_normalized"], json!(true));
    }

    #[test]
    fn normalize_runtime_plan_coerces_explicit_join_merge_role() {
        let plan = normalize_runtime_plan(
            "ship proof",
            &json!({}),
            json!({
                "topology_hint": "hybrid",
                "steps": [
                    {
                        "id": "branch-a",
                        "step": 1,
                        "action": "inspect-a",
                        "description": "inspect a",
                        "role": "worker",
                        "branch": "branch-a"
                    },
                    {
                        "id": "branch-b",
                        "step": 2,
                        "action": "inspect-b",
                        "description": "inspect b",
                        "role": "worker",
                        "branch": "branch-b"
                    },
                    {
                        "id": "join",
                        "step": 3,
                        "action": "synthesize",
                        "description": "merge both branches",
                        "role": "worker",
                        "branch": "join",
                        "depends_on": ["branch-a", "branch-b"]
                    }
                ]
            }),
        );

        let steps = plan["steps"].as_array().expect("normalized steps array");
        assert_eq!(steps[2]["role"], json!("coordinator"));
        assert_eq!(steps[2]["branch"], json!("join"));
    }

    #[test]
    fn normalize_runtime_plan_coerces_multidependency_non_join_branch() {
        let plan = normalize_runtime_plan(
            "ship proof",
            &json!({}),
            json!({
                "topology_hint": "hybrid",
                "steps": [
                    {
                        "id": "branch-a",
                        "step": 1,
                        "action": "inspect-a",
                        "description": "inspect a",
                        "role": "worker",
                        "branch": "branch-a"
                    },
                    {
                        "id": "branch-b",
                        "step": 2,
                        "action": "inspect-b",
                        "description": "inspect b",
                        "role": "worker",
                        "branch": "branch-b"
                    },
                    {
                        "id": "join",
                        "step": 3,
                        "action": "synthesize",
                        "description": "merge both branches",
                        "role": "worker",
                        "branch": "branch-a",
                        "depends_on": ["branch-a", "branch-b"]
                    }
                ]
            }),
        );

        let steps = plan["steps"].as_array().expect("normalized steps array");
        assert_eq!(steps[2]["role"], json!("coordinator"));
        assert_eq!(steps[2]["branch"], json!("join"));
    }

    #[test]
    fn normalize_runtime_plan_coerces_unsupported_merge_role_by_structure() {
        let plan = normalize_runtime_plan(
            "ship proof",
            &json!({}),
            json!({
                "topology_hint": "hybrid",
                "steps": [
                    {
                        "id": "branch-a",
                        "step": 1,
                        "action": "inspect-a",
                        "description": "inspect a",
                        "role": "worker",
                        "branch": "branch-a"
                    },
                    {
                        "id": "branch-b",
                        "step": 2,
                        "action": "inspect-b",
                        "description": "inspect b",
                        "role": "worker",
                        "branch": "branch-b"
                    },
                    {
                        "id": "join",
                        "step": 3,
                        "action": "synthesize",
                        "description": "merge both branches",
                        "role": "joiner",
                        "branch": "join",
                        "depends_on": ["branch-a", "branch-b"]
                    }
                ]
            }),
        );

        let steps = plan["steps"].as_array().expect("normalized steps array");
        assert_eq!(steps[2]["role"], json!("coordinator"));
        assert_eq!(steps[2]["branch"], json!("join"));
    }

    #[test]
    fn normalize_runtime_plan_then_compile_accepts_merge_step_with_join_role_hint() {
        let plan = normalize_runtime_plan(
            "ship proof",
            &json!({}),
            json!({
                "topology_hint": "hybrid",
                "steps": [
                    {
                        "id": "branch-a",
                        "step": 1,
                        "action": "inspect-a",
                        "description": "inspect a",
                        "role": "worker",
                        "branch": "branch-a"
                    },
                    {
                        "id": "branch-b",
                        "step": 2,
                        "action": "inspect-b",
                        "description": "inspect b",
                        "role": "worker",
                        "branch": "branch-b"
                    },
                    {
                        "id": "join",
                        "step": 3,
                        "action": "join",
                        "description": "merge both branches",
                        "role": "join",
                        "branch": "join",
                        "depends_on": ["branch-a", "branch-b"]
                    }
                ]
            }),
        );

        compile_execution_plan(TaskId::new(), &plan)
            .expect("normalized merge step should compile with coordinator semantics");
    }

    #[test]
    fn normalize_runtime_plan_canonicalizes_supported_topology_hint_aliases() {
        let plan = normalize_runtime_plan(
            "ship proof",
            &json!({}),
            json!({
                "topology_hint": "sequential-single-branch",
                "steps": [
                    {
                        "id": "step-1",
                        "step": 1,
                        "action": "inspect",
                        "description": "inspect",
                        "role": "worker",
                        "branch": "branch-a",
                        "depends_on": []
                    },
                    {
                        "id": "step-2",
                        "step": 2,
                        "action": "summarize",
                        "description": "summarize",
                        "role": "worker",
                        "branch": "branch-a",
                        "depends_on": ["step-1"]
                    }
                ]
            }),
        );

        assert_eq!(plan["topology_hint"], json!("sequential"));
    }

    #[test]
    fn normalize_runtime_plan_preserves_one_shot_runtime_compatibility() {
        let plan = normalize_runtime_plan(
            "ship proof",
            &json!({}),
            json!({
                "steps": [
                    {
                        "id": "single-step",
                        "step": 4,
                        "action": "answer",
                        "description": "one bounded answer"
                    }
                ]
            }),
        );

        let steps = plan["steps"].as_array().expect("normalized steps array");
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0]["id"], json!("single-step"));
        assert_eq!(steps[0]["step"], json!(1));
        assert_eq!(steps[0]["branch"], json!("branch-a"));
        assert_eq!(steps[0]["depends_on"], json!([]));
        assert_eq!(plan["topology_hint"], json!("sequential"));
        assert_eq!(plan["runtime_normalized"], json!(false));
    }

    #[test]
    fn compile_execution_plan_returns_clear_topology_compile_error() {
        let error = compile_execution_plan(
            TaskId::new(),
            &json!({
                "goal": "invalid-plan",
                "steps": [
                    {
                        "id": "branch-a",
                        "step": 1,
                        "action": "inspect-a",
                        "description": "inspect a",
                        "role": "worker",
                        "branch": "branch-a"
                    },
                    {
                        "id": "branch-b",
                        "step": 2,
                        "action": "inspect-b",
                        "description": "inspect b",
                        "role": "worker",
                        "branch": "branch-b"
                    },
                    {
                        "id": "join",
                        "step": 3,
                        "action": "join",
                        "description": "merge both branches",
                        "role": "join",
                        "branch": "join",
                        "depends_on": ["branch-a", "branch-b"]
                    }
                ]
            }),
        )
        .expect_err("unsupported planner role should fail before planning publication");

        assert_eq!(
            error,
            "workflow planning produced an invalid execution graph during topology compilation: Unsupported topology contract: unsupported planner role 'join'"
        );
    }

    #[test]
    fn normalize_runtime_plan_keeps_single_explicit_branch_step_sequential() {
        let plan = normalize_runtime_plan(
            "ship proof",
            &json!({}),
            json!({
                "topology_hint": "parallel",
                "steps": [
                    {
                        "id": "single-step",
                        "step": 4,
                        "action": "answer",
                        "description": "one bounded answer",
                        "branch": "research",
                        "depends_on": ["earlier-step"]
                    }
                ]
            }),
        );

        let steps = plan["steps"].as_array().expect("normalized steps array");
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0]["id"], json!("single-step"));
        assert_eq!(steps[0]["step"], json!(1));
        assert_eq!(steps[0]["branch"], json!("branch-a"));
        assert_eq!(steps[0]["depends_on"], json!([]));
        assert_eq!(plan["topology_hint"], json!("sequential"));
        assert_eq!(plan["runtime_normalized"], json!(false));
    }

    #[test]
    fn persist_step_routing_history_writes_non_empty_history() {
        let mut metadata = json!({});
        let history = vec![StepRoutingDecisionSummary {
            step_id: "planner.step.1".to_string(),
            step_index: Some(1),
            step_kind: Some("planner".to_string()),
            model_id: "gpt-5.4".to_string(),
            tier: "llm-tier".to_string(),
            reason: "selected direct routing".to_string(),
            previous_step_id: None,
            previous_action: None,
            previous_tier: None,
            action: "continue".to_string(),
            action_changed: false,
            preferred_tier_after: Some("llm-tier".to_string()),
            estimated_cost_tokens: Some(96),
            confidence_score: Some(0.95),
            triggered_checkpoints: vec![],
            change_rationale: vec!["initial routing decision".to_string()],
        }];

        persist_step_routing_history(&mut metadata, &history);

        assert_eq!(metadata["step_routing_history"], json!(history));
    }

    #[test]
    fn append_durable_workflow_history_event_tracks_monotonic_replay_positions() {
        let workflow_id = TaskId::new();
        let branch_id = ExecutionBranchId::new();
        let node_id = ExecutionNodeId::new();
        let mut metadata = json!({});

        append_durable_workflow_history_event(
            &mut metadata,
            workflow_id,
            DurableWorkflowEventKind::LifecycleChanged,
            None,
            None,
            Some(DurableWorkflowLifecycleState::Active),
            json!({
                "graph_state": GraphState::Running,
            }),
        );
        append_durable_workflow_history_event(
            &mut metadata,
            workflow_id,
            DurableWorkflowEventKind::NodeStateChanged,
            Some(branch_id),
            Some(node_id),
            Some(DurableWorkflowLifecycleState::Active),
            json!({
                "step_key": "draft-outline",
                "node_state": NodeState::Running,
                "branch_state": BranchState::Running,
                "graph_state": GraphState::Running,
            }),
        );

        let history = workflow_history(&metadata).expect("history should deserialize");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].replay_position, 1);
        assert_eq!(history[1].replay_position, 2);
        assert_eq!(
            history[0].event_kind,
            DurableWorkflowEventKind::LifecycleChanged
        );
        assert_eq!(
            history[1].event_kind,
            DurableWorkflowEventKind::NodeStateChanged
        );
        assert_eq!(
            history[1].payload["step_key"],
            json!("draft-outline"),
            "stable step keys should be persisted for replay"
        );
    }

    #[test]
    fn verifier_gate_accepts_enabled_policy_and_records_step_evaluation() {
        let workflow_id = TaskId::new();
        let worker_id = AgentId::new();
        let task = TaskAssignment::new(
            "analyze",
            json!({
                "id": "draft-outline",
                "branch_id": "branch-a",
                "description": "draft the outline",
                "verifier_policy": {
                    "enabled": true,
                    "verdict": "accepted",
                    "confidence": 0.91,
                    "reason": "handoff satisfied the verifier contract",
                    "failure_code": null,
                    "checkpoint_ref": "checkpoint-1",
                    "repair_directive": null
                }
            }),
        );
        let result = json!({
            "summary": "accepted outline"
        });

        let completed = gate_step_execution_result(workflow_id, &task, worker_id, result.clone())
            .expect("accepted verifier policy should keep the step completed");

        assert_eq!(completed.result, result);
        let step_evaluation = completed
            .step_evaluation
            .clone()
            .expect("accepted steps should record verifier evaluation");
        assert_eq!(step_evaluation.verdict, VerifierVerdict::Accepted);
        assert_eq!(
            step_evaluation.reason,
            "handoff satisfied the verifier contract"
        );

        let payload = build_step_result_payload(
            completed.task_id,
            completed.worker_id,
            completed.branch_id,
            completed.action,
            Some(completed.result),
            Some(step_evaluation),
        );
        assert_eq!(payload["step_evaluation"]["verdict"], json!("accepted"));
        assert_eq!(
            payload["step_evaluation"]["checkpoint_ref"],
            json!("checkpoint-1")
        );
    }

    #[test]
    fn verifier_gate_rejects_enabled_policy_and_records_repair_directive() {
        let workflow_id = TaskId::new();
        let worker_id = AgentId::new();
        let task = TaskAssignment::new(
            "analyze",
            json!({
                "id": "draft-outline",
                "branch_id": "branch-a",
                "description": "draft the outline",
                "verifier_policy": {
                    "enabled": true,
                    "verdict": "rejected",
                    "confidence": 0.23,
                    "reason": "missing bounded retry context",
                    "failure_code": "missing_context",
                    "checkpoint_ref": "checkpoint-2",
                    "repair_directive": {
                        "action": "retry_step",
                        "issued_by": "verifier.runtime",
                        "failure_context_ref": "draft-outline/missing-context",
                        "retry_budget_remaining": 1
                    }
                }
            }),
        );
        let result = json!({
            "summary": "rejected outline"
        });

        let failed = gate_step_execution_result(workflow_id, &task, worker_id, result.clone())
            .expect_err("rejected verifier policy should stop downstream progression");

        let step_evaluation = failed
            .step_evaluation
            .clone()
            .expect("rejected steps should record verifier evaluation");
        assert_eq!(step_evaluation.verdict, VerifierVerdict::Rejected);
        assert_eq!(
            step_evaluation
                .repair_directive
                .as_ref()
                .map(|directive| directive.action.as_str()),
            Some("retry_step")
        );

        let payload = build_step_result_payload(
            failed.task_id,
            failed.worker_id,
            failed.branch_id.clone(),
            failed.action.clone(),
            failed.result.clone(),
            failed.step_evaluation.clone(),
        );
        assert_eq!(payload["step_evaluation"]["verdict"], json!("rejected"));
        assert_eq!(
            payload["step_evaluation"]["repair_directive"]["action"],
            json!("retry_step")
        );
        assert_eq!(
            payload["step_evaluation"]["failure_context_checkpoint"]["checkpoint_ref"],
            json!("checkpoint-2")
        );
        assert!(failed.message.contains("repair=retry_step"));
    }

    #[test]
    fn verifier_gate_clarifies_handoff_before_accepting_follow_up() {
        let workflow_id = TaskId::new();
        let worker_id = AgentId::new();
        let task = TaskAssignment::new(
            "analyze",
            json!({
                "id": "draft-outline",
                "branch_id": "branch-a",
                "description": "draft the outline",
                "verifier_policy_sequence": [
                    {
                        "enabled": true,
                        "verdict": "rejected",
                        "reason": "missing budget ceiling and target audience",
                        "failure_code": "missing_constraints",
                        "checkpoint_ref": "checkpoint-clarify",
                        "repair_directive": {
                            "action": "clarify_handoff",
                            "issued_by": "verifier.runtime",
                            "failure_context_ref": "draft-outline/clarify",
                            "retry_budget_remaining": 1
                        },
                        "clarification_request": {
                            "source_step_id": "draft-outline",
                            "target_step_id": "write-brief",
                            "missing_constraints": ["budget ceiling", "target audience"],
                            "attempt_count": 0,
                            "expires_at": "2026-03-27T12:00:00Z"
                        }
                    },
                    {
                        "enabled": true,
                        "verdict": "accepted",
                        "confidence": 0.91,
                        "reason": "clarified handoff is sufficient",
                        "checkpoint_ref": "checkpoint-clarify"
                    }
                ]
            }),
        );

        let (completed, execution_input) = run_local_repair_sequence(workflow_id, &task, worker_id)
            .expect("clarification follow-up should complete");
        let step_evaluation = completed
            .step_evaluation
            .expect("clarification repair should preserve final step evaluation");

        assert_eq!(step_evaluation.verdict, VerifierVerdict::Accepted);
        assert_eq!(
            step_evaluation
                .clarification_request
                .as_ref()
                .map(|request| request.attempt_count),
            Some(1)
        );
        assert_eq!(
            step_evaluation
                .failure_context_checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.checkpoint_ref.as_deref()),
            Some("checkpoint-clarify")
        );
        assert_eq!(
            execution_input["task"]["repair_action"],
            json!("clarify_handoff")
        );
        assert_eq!(
            execution_input["task"]["clarification_request"]["missing_constraints"],
            json!(["budget ceiling", "target audience"])
        );
    }

    #[test]
    fn verifier_gate_stops_when_retry_budget_is_exhausted() {
        let workflow_id = TaskId::new();
        let worker_id = AgentId::new();
        let task = TaskAssignment::new(
            "analyze",
            json!({
                "id": "draft-outline",
                "branch_id": "branch-a",
                "description": "draft the outline",
                "verifier_policy_sequence": [
                    {
                        "enabled": true,
                        "verdict": "rejected",
                        "reason": "first retry requested",
                        "failure_code": "retryable_context_gap",
                        "checkpoint_ref": "checkpoint-retry",
                        "repair_directive": {
                            "action": "retry_step",
                            "issued_by": "verifier.runtime",
                            "failure_context_ref": "draft-outline/retry-1",
                            "retry_budget_remaining": 1
                        }
                    },
                    {
                        "enabled": true,
                        "verdict": "rejected",
                        "reason": "retry still missing context",
                        "failure_code": "retryable_context_gap",
                        "checkpoint_ref": "checkpoint-retry",
                        "repair_directive": {
                            "action": "retry_step",
                            "issued_by": "verifier.runtime",
                            "failure_context_ref": "draft-outline/retry-2",
                            "retry_budget_remaining": 1
                        }
                    }
                ]
            }),
        );

        let failed = run_local_repair_sequence(workflow_id, &task, worker_id)
            .expect_err("retry budget exhaustion should stop local repair");

        assert!(failed.message.contains("local repair budget exhausted"));
        assert_eq!(
            failed
                .step_evaluation
                .as_ref()
                .and_then(|evaluation| evaluation.failure_context_checkpoint.as_ref())
                .map(|checkpoint| checkpoint.attempt_count),
            Some(2)
        );
    }

    #[test]
    fn verifier_gate_replans_from_checkpoint_with_preserved_failure_context() {
        let workflow_id = TaskId::new();
        let worker_id = AgentId::new();
        let task = TaskAssignment::new(
            "analyze",
            json!({
                "id": "draft-outline",
                "branch_id": "branch-a",
                "description": "draft the outline",
                "verifier_policy_sequence": [
                    {
                        "enabled": true,
                        "verdict": "rejected",
                        "reason": "resume from stable checkpoint",
                        "failure_code": "checkpoint_resume",
                        "checkpoint_ref": "checkpoint-replan",
                        "repair_directive": {
                            "action": "replan_from_checkpoint",
                            "issued_by": "verifier.runtime",
                            "failure_context_ref": "draft-outline/replan",
                            "retry_budget_remaining": 1
                        }
                    },
                    {
                        "enabled": true,
                        "verdict": "accepted",
                        "confidence": 0.84,
                        "reason": "checkpoint-scoped replan succeeded",
                        "checkpoint_ref": "checkpoint-replan"
                    }
                ]
            }),
        );

        let (completed, execution_input) = run_local_repair_sequence(workflow_id, &task, worker_id)
            .expect("checkpoint-scoped replan should complete");
        let step_evaluation = completed
            .step_evaluation
            .expect("checkpoint repair should preserve final step evaluation");

        assert_eq!(step_evaluation.verdict, VerifierVerdict::Accepted);
        assert_eq!(
            step_evaluation
                .failure_context_checkpoint
                .as_ref()
                .and_then(|checkpoint| checkpoint.checkpoint_ref.as_deref()),
            Some("checkpoint-replan")
        );
        assert_eq!(
            execution_input["task"]["repair_action"],
            json!("replan_from_checkpoint")
        );
        assert_eq!(
            execution_input["task"]["failure_context_checkpoint"]["failure_context_ref"],
            json!("draft-outline/replan")
        );
    }

    #[test]
    fn description_only_repair_step_records_runtime_generated_step_evaluation() {
        let workflow_id = TaskId::new();
        let worker_id = AgentId::new();
        let task = TaskAssignment::new(
            "resolve_missing_handoff_context",
            json!({
                "id": "s2",
                "step_id": "s2",
                "branch_id": "branch-a",
                "action": "resolve_missing_handoff_context",
                "description": "Request the minimum clarification needed or perform bounded local repair if any worker handoff is missing required context.",
                "dependencies": ["s1"]
            }),
        );
        let result = json!({
            "summary": "clarified context"
        });

        let completed = gate_step_execution_result(workflow_id, &task, worker_id, result.clone())
            .expect("description-only repair step should still complete");

        assert_eq!(completed.result, result);
        let step_evaluation = completed
            .step_evaluation
            .clone()
            .expect("planner repair step should emit a runtime-generated evaluation");
        assert_eq!(step_evaluation.verdict, VerifierVerdict::Accepted);
        assert_eq!(
            step_evaluation
                .repair_directive
                .as_ref()
                .map(|directive| directive.action.as_str()),
            Some("clarify_handoff")
        );
        assert_eq!(
            step_evaluation
                .repair_directive
                .as_ref()
                .map(|directive| directive.issued_by.as_str()),
            Some("runtime.planner_repair")
        );
        assert_eq!(
            step_evaluation
                .clarification_request
                .as_ref()
                .map(|request| request.attempt_count),
            Some(1)
        );
        assert_eq!(
            step_evaluation.checkpoint_ref.as_deref(),
            Some("planner-step:s1")
        );

        let payload = build_step_result_payload(
            completed.task_id,
            completed.worker_id,
            completed.branch_id,
            completed.action,
            Some(completed.result),
            Some(step_evaluation),
        );
        assert_eq!(payload["step_evaluation"]["verdict"], json!("accepted"));
        assert_eq!(
            payload["step_evaluation"]["repair_directive"]["issued_by"],
            json!("runtime.planner_repair")
        );
        assert_eq!(
            payload["step_evaluation"]["failure_context_checkpoint"]["last_stable_step_id"],
            json!("s1")
        );
    }

    #[test]
    fn verifier_gate_preserves_current_behavior_when_policy_disabled() {
        let workflow_id = TaskId::new();
        let worker_id = AgentId::new();
        let task = TaskAssignment::new(
            "analyze",
            json!({
                "id": "draft-outline",
                "branch_id": "branch-a",
                "description": "draft the outline",
                "verifier_policy": {
                    "enabled": false,
                    "verdict": "invalid",
                    "repair_directive": "ignore me"
                }
            }),
        );
        let result = json!({
            "summary": "current behavior preserved"
        });

        let completed = gate_step_execution_result(workflow_id, &task, worker_id, result.clone())
            .expect("disabled verifier policy should preserve the current happy path");

        assert_eq!(completed.result, result);
        assert!(
            completed.step_evaluation.is_none(),
            "disabled verifier policy must not annotate the step"
        );

        let payload = build_step_result_payload(
            completed.task_id,
            completed.worker_id,
            completed.branch_id,
            completed.action,
            Some(completed.result),
            completed.step_evaluation,
        );
        assert!(payload.get("step_evaluation").is_none());
    }

    #[test]
    fn runtime_supervision_plan_targets_node_for_local_clarification() {
        let (graph, mut task, workflow_id, branch_target) = runtime_supervision_graph();
        let worker_id = AgentId::new();
        let node_target = GuardTarget::Node(ExecutionNodeId::from_uuid(*task.task_id.as_ref()));
        let branch_id = match branch_target {
            GuardTarget::Branch(branch_id) => branch_id,
            _ => panic!("expected branch target"),
        };
        task.input
            .as_object_mut()
            .expect("task input should be mutable")
            .insert(
                "verifier_policy_sequence".to_string(),
                json!([
                    {
                        "enabled": true,
                        "verdict": "rejected",
                        "reason": "missing branch-local repair context",
                        "failure_code": "missing_context",
                        "checkpoint_ref": "checkpoint-clarify",
                        "repair_directive": {
                            "action": "clarify_handoff",
                            "issued_by": "verifier.runtime",
                            "failure_context_ref": "branch-a/clarify",
                            "retry_budget_remaining": 1
                        },
                        "clarification_request": {
                            "source_step_id": "branch-a",
                            "target_step_id": "branch-a",
                            "missing_constraints": ["repair context"],
                            "attempt_count": 0,
                            "expires_at": "2026-03-27T12:00:00Z"
                        }
                    },
                    {
                        "enabled": true,
                        "verdict": "accepted",
                        "reason": "clarified"
                    }
                ]),
            );

        let verifier_policies =
            verifier_policies_from_task_input(&task).expect("policies should parse");
        let repair_state = LocalRepairState::default();
        let context = StepExecutionContext {
            workflow_id,
            task: &task,
            worker_id,
            branch_id: json!(branch_id),
            action: task.task_type.clone(),
            repair_state: &repair_state,
        };
        let transition = match step_execution_disposition(
            &context,
            json!({"summary": "clarify"}),
            verifier_policies.first().cloned(),
        )
        .expect("clarify policy should evaluate")
        {
            StepExecutionDisposition::Continue(transition) => transition,
            _ => panic!("expected local repair transition"),
        };

        let plan = runtime_supervision_plan_for_repair_evaluation(
            &graph,
            &task,
            &transition.step_evaluation,
        )
        .expect("plan should build")
        .expect("transition should produce supervision");

        assert_eq!(plan.target, node_target);
        assert!(plan.checkpoints.is_empty());
        let snapshot = plan
            .assessment
            .snapshot()
            .expect("profile snapshot should exist");
        assert_eq!(snapshot.semantic_signals.len(), 1);
        assert_eq!(
            snapshot.semantic_signals[0].signal_kind,
            SemanticSignalKind::MissingContext
        );
    }

    #[test]
    fn runtime_supervision_plan_targets_node_for_description_only_repair_step() {
        let (graph, mut task, workflow_id, _) = runtime_supervision_graph();
        task.input
            .as_object_mut()
            .expect("task input should be mutable")
            .insert(
                "action".to_string(),
                json!("resolve_missing_handoff_context"),
            );
        task.input
            .as_object_mut()
            .expect("task input should be mutable")
            .insert(
                "description".to_string(),
                json!(
                    "Request the minimum clarification needed or perform bounded local repair if any worker handoff is missing required context."
                ),
            );
        task.input
            .as_object_mut()
            .expect("task input should be mutable")
            .insert("dependencies".to_string(), json!(["branch-a"]));
        let step_evaluation =
            runtime_repair_step_evaluation(workflow_id, &task, &LocalRepairState::default())
                .expect("planner repair step should emit a runtime-generated evaluation");

        let plan = runtime_supervision_plan_for_repair_evaluation(&graph, &task, &step_evaluation)
            .expect("plan should build")
            .expect("repair evaluation should produce supervision");

        assert!(matches!(plan.target, GuardTarget::Node(_)));
        let snapshot = plan
            .assessment
            .snapshot()
            .expect("profile snapshot should exist");
        assert_eq!(snapshot.semantic_signals.len(), 1);
        assert_eq!(
            snapshot.semantic_signals[0].signal_kind,
            SemanticSignalKind::MissingContext
        );
    }

    #[test]
    fn runtime_supervision_plan_targets_branch_for_failed_step() {
        let (graph, mut task, workflow_id, branch_target) = runtime_supervision_graph();
        let worker_id = AgentId::new();
        let branch_id = match branch_target {
            GuardTarget::Branch(branch_id) => branch_id,
            _ => panic!("expected branch target"),
        };
        task.input
            .as_object_mut()
            .expect("task input should be mutable")
            .insert(
                "verifier_policy".to_string(),
                json!({
                    "enabled": true,
                    "verdict": "rejected",
                    "reason": "policy conflict requires escalation",
                    "failure_code": "policy_conflict",
                    "checkpoint_ref": "checkpoint-stop",
                    "repair_directive": {
                        "action": "stop",
                        "issued_by": "verifier.runtime",
                        "failure_context_ref": "branch-a/stop",
                        "retry_budget_remaining": 0
                    }
                }),
            );

        let repair_state = LocalRepairState::default();
        let context = StepExecutionContext {
            workflow_id,
            task: &task,
            worker_id,
            branch_id: json!(branch_id),
            action: task.task_type.clone(),
            repair_state: &repair_state,
        };
        let failed_step = match step_execution_disposition(
            &context,
            json!({"summary": "stop"}),
            verifier_policy_from_task_input(&task).expect("policy should parse"),
        )
        .expect("stop policy should evaluate")
        {
            StepExecutionDisposition::Failed(failed) => failed,
            _ => panic!("expected failed step"),
        };

        let plan = runtime_supervision_plan_for_failed_step(&graph, &task, &failed_step)
            .expect("plan should build")
            .expect("failed step should produce supervision");

        assert_eq!(plan.target, GuardTarget::Branch(branch_id));
        let snapshot = plan
            .assessment
            .snapshot()
            .expect("profile snapshot should exist");
        assert_eq!(snapshot.semantic_signals.len(), 1);
        assert_eq!(
            snapshot.semantic_signals[0].signal_kind,
            SemanticSignalKind::PolicyConflict
        );
    }
}

#[cfg(test)]
mod workflow_step_tool_tests {
    use super::*;

    #[tokio::test]
    async fn workflow_step_tool_marks_payload_as_tool_bus_completed() {
        let tool = WorkflowStepTool { id: ToolId::new() };
        let payload = json!({
            "workflow_goal": "ship proof",
            "task": {
                "step_id": "step-1",
                "action": "analyze",
            },
        });

        let result = tool.execute(payload.clone()).await.unwrap();

        assert_eq!(result["status"], "completed");
        assert_eq!(result["execution_boundary"], "tool_bus");
        assert_eq!(
            result["tool_name"],
            format!("{WORKFLOW_TOOL_NAMESPACE}.{WORKFLOW_EXECUTE_STEP_TOOL}")
        );
        assert_eq!(result["task"], payload["task"]);
    }
}
