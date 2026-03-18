//! Gate 9 validation tests — end-to-end LLM provider integration.
//!
//! Exercises: Planner → MockProvider via ModelRouter → ToolBus → Orchestrator flow.
//! Run with: `cargo test -p mister-smith-agents --features llm --test gate9_tests`
#![cfg(feature = "llm")]

use std::sync::Arc;

use async_trait::async_trait;
use futures::stream;
use mister_smith_agents::orchestrator::{LlmSupervision, LlmSupervisionConfig};
use mister_smith_agents::roles::critic::{CriticAgent, CriticMessage, CriticState};
use mister_smith_agents::roles::executor::{ExecutorAgent, ExecutorMessage, ExecutorState};
use mister_smith_agents::roles::planner::{PlannerAgent, PlannerMessage, PlannerState};
use mister_smith_agents::scheduler::{
    ArrayAggregator, IdentityDecomposer, TaskAssignment, TaskScheduler,
};
use mister_smith_agents::tool_bus::ToolBus;
use mister_smith_agents::topology::{TopologyCompiler, TopologySignals};
use mister_smith_agents::Orchestrator;
use mister_smith_core::{
    Actor, AgentId, FailureClass, GuardTarget, InterventionType, LlmError, TaskId,
};
use mister_smith_llm::budget::{BudgetEnforcer, BudgetNode, BudgetPolicy, InMemoryBudgetStore};
use mister_smith_llm::{
    CascadePolicy, CascadeTier, CircuitBreakerConfig, CompletionRequest, CompletionResponse,
    CompletionStream, ContentBlock, EmbeddingResponse, MockProvider, ModelCapabilities,
    ModelProvider, ModelRouter, ProviderConfig, ProviderKind, RoutingPolicy, StopReason,
    StreamChunk, ToolCall, Usage,
};
use std::sync::Mutex;

async fn mock_router() -> Arc<ModelRouter> {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);
    let provider: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("mock-gate9"));
    let config = ProviderConfig {
        provider_kind: ProviderKind::Mock,
        model_id: "mock-gate9".to_string(),
        ..Default::default()
    };
    router
        .add_provider(config, provider, CircuitBreakerConfig::default())
        .await;
    Arc::new(router)
}

fn empty_router() -> Arc<ModelRouter> {
    Arc::new(ModelRouter::new(RoutingPolicy::RoundRobin))
}

#[derive(Debug)]
struct AlwaysFailProvider {
    model_id: String,
}

#[async_trait]
impl ModelProvider for AlwaysFailProvider {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        Err(LlmError::Network("provider completion failure".to_string()))
    }

    fn stream(&self, _request: CompletionRequest) -> CompletionStream {
        Box::pin(stream::once(async {
            Err(LlmError::Network("provider stream failure".to_string()))
        }))
    }

    async fn embed(&self, _input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        Err(LlmError::Network("provider embedding failure".to_string()))
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities::all()
    }
}

async fn failing_provider_router() -> Arc<ModelRouter> {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);
    let provider: Arc<dyn ModelProvider> = Arc::new(AlwaysFailProvider {
        model_id: "always-fail".to_string(),
    });
    let config = ProviderConfig {
        provider_kind: ProviderKind::Mock,
        model_id: "always-fail".to_string(),
        ..Default::default()
    };
    router
        .add_provider(config, provider, CircuitBreakerConfig::default())
        .await;
    Arc::new(router)
}

#[derive(Debug)]
struct RepetitiveStreamProvider {
    model_id: String,
}

#[async_trait]
impl ModelProvider for RepetitiveStreamProvider {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        Ok(CompletionResponse {
            content: vec![mister_smith_llm::ContentBlock::Text {
                text: r#"{"status":"executing","strategy":"baseline","estimated_steps":1}"#
                    .to_string(),
            }],
            model_id: self.model_id.clone(),
            usage: mister_smith_llm::Usage::new(8, 8),
            stop_reason: StopReason::Completed,
            tool_calls: Vec::new(),
        })
    }

    fn stream(&self, _request: CompletionRequest) -> CompletionStream {
        Box::pin(stream::iter(vec![
            Ok(StreamChunk {
                index: 0,
                delta: mister_smith_llm::ChunkDelta::Text {
                    text: "repeat".to_string(),
                },
            }),
            Ok(StreamChunk {
                index: 1,
                delta: mister_smith_llm::ChunkDelta::Text {
                    text: "repeat".to_string(),
                },
            }),
            Ok(StreamChunk {
                index: 2,
                delta: mister_smith_llm::ChunkDelta::Text {
                    text: "repeat".to_string(),
                },
            }),
            Ok(StreamChunk::stop(3, StopReason::Completed)),
        ]))
    }

    async fn embed(&self, _input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        Err(LlmError::UnsupportedCapability {
            capability: "embeddings".to_string(),
            model: self.model_id.clone(),
        })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities::all()
    }
}

async fn repetitive_stream_router() -> Arc<ModelRouter> {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);
    let provider: Arc<dyn ModelProvider> = Arc::new(RepetitiveStreamProvider {
        model_id: "repetitive-stream".to_string(),
    });
    let config = ProviderConfig {
        provider_kind: ProviderKind::Mock,
        model_id: "repetitive-stream".to_string(),
        ..Default::default()
    };
    router
        .add_provider(config, provider, CircuitBreakerConfig::default())
        .await;
    Arc::new(router)
}

fn mock_config_with_tier(model_id: &str, tier: &str) -> ProviderConfig {
    ProviderConfig {
        provider_kind: ProviderKind::Mock,
        model_id: model_id.to_string(),
        metadata: serde_json::json!({ "tier": tier }),
        ..Default::default()
    }
}

fn cascade_policy(threshold: f32, max_escalations: u32) -> CascadePolicy {
    CascadePolicy {
        tiers: vec![
            CascadeTier {
                provider_config: mock_config_with_tier("slm", "slm-tier"),
                label: "slm-tier".to_string(),
            },
            CascadeTier {
                provider_config: mock_config_with_tier("llm", "llm-tier"),
                label: "llm-tier".to_string(),
            },
        ],
        escalation_threshold: threshold,
        max_escalations,
    }
}

fn soft_cap_budget_store() -> InMemoryBudgetStore {
    let store = InMemoryBudgetStore::default();
    store.insert(BudgetNode {
        key: "budget/agents".to_string(),
        limit_tokens: 10_000,
        used_tokens: 0,
        period: "session".to_string(),
        policy: BudgetPolicy::SoftCap,
        revision: 0,
    });
    store
}

fn planner_response(text: &str) -> CompletionResponse {
    CompletionResponse {
        content: vec![ContentBlock::Text {
            text: text.to_string(),
        }],
        model_id: "scenario".to_string(),
        usage: Usage::new(16, 12),
        stop_reason: StopReason::Completed,
        tool_calls: Vec::new(),
    }
}

fn critic_response(text: &str) -> CompletionResponse {
    CompletionResponse {
        content: vec![ContentBlock::Text {
            text: text.to_string(),
        }],
        model_id: "scenario".to_string(),
        usage: Usage::new(14, 10),
        stop_reason: StopReason::Completed,
        tool_calls: Vec::new(),
    }
}

#[derive(Debug)]
struct RecordingResponseProvider {
    model_id: String,
    response: CompletionResponse,
    observed_requests: Arc<Mutex<Vec<CompletionRequest>>>,
}

impl RecordingResponseProvider {
    fn new(
        model_id: impl Into<String>,
        response: CompletionResponse,
        observed_requests: Arc<Mutex<Vec<CompletionRequest>>>,
    ) -> Self {
        Self {
            model_id: model_id.into(),
            response,
            observed_requests,
        }
    }
}

#[async_trait]
impl ModelProvider for RecordingResponseProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        self.observed_requests.lock().unwrap().push(request);
        Ok(self.response.clone())
    }

    fn stream(&self, _request: CompletionRequest) -> CompletionStream {
        let text = self
            .response
            .content
            .iter()
            .find_map(|block| match block {
                ContentBlock::Text { text } => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_default();
        Box::pin(stream::iter(vec![
            Ok(StreamChunk {
                index: 0,
                delta: mister_smith_llm::ChunkDelta::Text { text },
            }),
            Ok(StreamChunk::stop(1, self.response.stop_reason.clone())),
        ]))
    }

    async fn embed(&self, _input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        Err(LlmError::UnsupportedCapability {
            capability: "embeddings".to_string(),
            model: self.model_id.clone(),
        })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities::all()
    }
}

#[derive(Debug)]
struct RecordingFailingProvider {
    model_id: String,
    observed_requests: Arc<Mutex<Vec<CompletionRequest>>>,
}

impl RecordingFailingProvider {
    fn new(
        model_id: impl Into<String>,
        observed_requests: Arc<Mutex<Vec<CompletionRequest>>>,
    ) -> Self {
        Self {
            model_id: model_id.into(),
            observed_requests,
        }
    }
}

#[async_trait]
impl ModelProvider for RecordingFailingProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        self.observed_requests.lock().unwrap().push(request);
        Err(LlmError::Network("simulated provider failure".to_string()))
    }

    fn stream(&self, request: CompletionRequest) -> CompletionStream {
        self.observed_requests.lock().unwrap().push(request);
        Box::pin(stream::once(async {
            Err(LlmError::Network("simulated provider failure".to_string()))
        }))
    }

    async fn embed(&self, _input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        Err(LlmError::UnsupportedCapability {
            capability: "embeddings".to_string(),
            model: self.model_id.clone(),
        })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities::all()
    }
}

fn supervision_graph(
    workflow_id: TaskId,
) -> (
    mister_smith_agents::ExecutionGraph,
    mister_smith_core::ExecutionBranchId,
    TaskId,
) {
    let compiler = TopologyCompiler::default();
    let graph = compiler
        .compile(
            workflow_id,
            &serde_json::json!({
                "goal": "gate9-supervision",
                "steps": [
                    {
                        "id": "branch-a",
                        "step": 1,
                        "action": "analyze",
                        "description": "Analyze branch",
                        "branch": "branch-a",
                    }
                ]
            }),
            &TopologySignals::default(),
        )
        .expect("graph should compile");
    let node = graph.nodes[0].node_id;
    let branch_id = graph.nodes[0].branch_id;
    (graph, branch_id, TaskId::from_uuid(*node.as_ref()))
}

#[tokio::test]
async fn planner_produces_plan_via_mock_provider() {
    let router = mock_router().await;
    let mut planner = PlannerAgent::with_router(AgentId::new(), router);
    let mut state = PlannerState::default();

    let result = planner
        .handle_message(
            PlannerMessage::PlanGoal {
                goal: "deploy service".into(),
                context: serde_json::json!({"env": "staging"}),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap();

    assert!(result.is_object());
    assert!(state.current_plan.is_some());
}

#[tokio::test]
async fn critic_evaluates_via_mock_provider() {
    let router = mock_router().await;
    let mut critic = CriticAgent::with_router(AgentId::new(), router);
    let mut state = CriticState::default();

    let result = critic
        .handle_message(
            CriticMessage::Evaluate {
                output: serde_json::json!("some output"),
                criteria: serde_json::json!(["accuracy", "completeness"]),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap();

    assert!(result.is_object());
    assert_eq!(state.evaluations_completed, 1);
}

#[tokio::test]
async fn planner_carries_forward_escalated_tier_to_the_next_step() {
    let slm_requests = Arc::new(Mutex::new(Vec::new()));
    let llm_requests = Arc::new(Mutex::new(Vec::new()));

    let router = ModelRouter::new(RoutingPolicy::Cascade(cascade_policy(0.9, 1)));
    let slm: Arc<dyn ModelProvider> = Arc::new(RecordingResponseProvider::new(
        "slm-model",
        planner_response("{}"),
        slm_requests.clone(),
    ));
    let llm: Arc<dyn ModelProvider> = Arc::new(RecordingResponseProvider::new(
        "llm-model",
        planner_response(
            r#"{"goal":"deploy service","steps":[{"step":1,"action":"analyze","description":"deploy service"}],"context":{"env":"staging"}}"#,
        ),
        llm_requests.clone(),
    ));

    router
        .add_provider(
            mock_config_with_tier("slm", "slm-tier"),
            slm,
            CircuitBreakerConfig::default(),
        )
        .await;
    router
        .add_provider(
            mock_config_with_tier("llm", "llm-tier"),
            llm,
            CircuitBreakerConfig::default(),
        )
        .await;

    let mut planner = PlannerAgent::with_router(AgentId::new(), Arc::new(router));
    let mut state = PlannerState::default();

    planner
        .handle_message(
            PlannerMessage::PlanGoal {
                goal: "deploy service".into(),
                context: serde_json::json!({"env": "staging"}),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap();
    planner
        .handle_message(
            PlannerMessage::PlanGoal {
                goal: "verify deploy".into(),
                context: serde_json::json!({"env": "staging"}),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap();

    assert_eq!(
        slm_requests.lock().unwrap().len(),
        1,
        "the second planner step should not restart from the low-confidence SLM tier"
    );
    let llm_requests = llm_requests.lock().unwrap();
    assert_eq!(llm_requests.len(), 2);
    assert!(
        llm_requests[0]
            .system
            .as_deref()
            .unwrap()
            .contains("task planning agent"),
        "baseline planner prompt should remain intact"
    );
    let second_system = llm_requests[1].system.as_deref().unwrap();
    assert!(
        second_system.contains("Routing carryover: keep the stronger reasoning tier active"),
        "planner escalation carryover should persist the action hint into the next step: {second_system}"
    );
    assert!(
        second_system.contains("Confidence review is active"),
        "planner escalation carryover should preserve the triggered confidence checkpoint: {second_system}"
    );
}

#[tokio::test]
async fn critic_carries_forward_fallback_tier_to_the_next_step() {
    let slm_requests = Arc::new(Mutex::new(Vec::new()));
    let llm_requests = Arc::new(Mutex::new(Vec::new()));

    let router = ModelRouter::new(RoutingPolicy::Cascade(cascade_policy(0.9, 1)));
    let slm: Arc<dyn ModelProvider> = Arc::new(RecordingFailingProvider::new(
        "slm-model",
        slm_requests.clone(),
    ));
    let llm: Arc<dyn ModelProvider> = Arc::new(RecordingResponseProvider::new(
        "llm-model",
        critic_response(
            r#"{"evaluation":"pass","confidence":0.99,"suggestions":["none"],"reasoning":"fallback accepted"}"#,
        ),
        llm_requests.clone(),
    ));

    router
        .add_provider(
            mock_config_with_tier("slm", "slm-tier"),
            slm,
            CircuitBreakerConfig::default(),
        )
        .await;
    router
        .add_provider(
            mock_config_with_tier("llm", "llm-tier"),
            llm,
            CircuitBreakerConfig::default(),
        )
        .await;

    let mut critic = CriticAgent::with_router(AgentId::new(), Arc::new(router));
    let mut state = CriticState::default();

    critic
        .handle_message(
            CriticMessage::Evaluate {
                output: serde_json::json!("draft-1"),
                criteria: serde_json::json!(["accuracy"]),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap();
    critic
        .handle_message(
            CriticMessage::Evaluate {
                output: serde_json::json!("draft-2"),
                criteria: serde_json::json!(["accuracy"]),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap();

    assert_eq!(
        slm_requests.lock().unwrap().len(),
        1,
        "fallback carryover should prevent the second critic step from retrying the failed tier first"
    );
    let llm_requests = llm_requests.lock().unwrap();
    assert_eq!(llm_requests.len(), 2);
    let second_system = llm_requests[1].system.as_deref().unwrap();
    assert!(
        second_system.contains("Routing carryover: prefer resilient assumptions"),
        "critic fallback carryover should persist the fallback action hint into the next step: {second_system}"
    );
    assert!(
        second_system.contains("Provider failure fallback is active"),
        "critic fallback carryover should preserve the provider failure checkpoint: {second_system}"
    );
}

#[tokio::test]
async fn critic_downgrade_resets_preferred_tier_and_updates_verification_guidance() {
    let slm_requests = Arc::new(Mutex::new(Vec::new()));
    let llm_requests = Arc::new(Mutex::new(Vec::new()));
    let budget_store = soft_cap_budget_store();
    let budget_enforcer = BudgetEnforcer::new(Box::new(budget_store));

    let router = ModelRouter::new(RoutingPolicy::Cascade(cascade_policy(0.9, 1)))
        .with_budget(budget_enforcer, "budget/agents");
    let slm: Arc<dyn ModelProvider> = Arc::new(RecordingResponseProvider::new(
        "slm-model",
        critic_response("{}"),
        slm_requests.clone(),
    ));
    let llm: Arc<dyn ModelProvider> = Arc::new(RecordingResponseProvider::new(
        "llm-model",
        critic_response(
            r#"{"evaluation":"pass","confidence":0.99,"suggestions":["none"],"reasoning":"budget accepted"}"#,
        ),
        llm_requests.clone(),
    ));

    router
        .add_provider(
            mock_config_with_tier("slm", "slm-tier"),
            slm,
            CircuitBreakerConfig::default(),
        )
        .await;
    router
        .add_provider(
            mock_config_with_tier("llm", "llm-tier"),
            llm,
            CircuitBreakerConfig::default(),
        )
        .await;

    let mut critic = CriticAgent::with_router(AgentId::new(), Arc::new(router));
    let mut state = CriticState::default();

    critic
        .handle_message(
            CriticMessage::Evaluate {
                output: serde_json::json!("draft-1"),
                criteria: serde_json::json!(["accuracy"]),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap();
    critic
        .handle_message(
            CriticMessage::Evaluate {
                output: serde_json::json!("draft-2"),
                criteria: serde_json::json!(["accuracy"]),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap();

    let slm_requests = slm_requests.lock().unwrap();
    assert_eq!(
        slm_requests.len(),
        2,
        "a downgrade should clear the prior llm-tier preference so the next critic step starts from the baseline tier again"
    );
    let second_system = slm_requests[1].system.as_deref().unwrap();
    assert!(
        second_system.contains("Budget policy is active"),
        "downgrade guidance should be carried into the next critic step: {second_system}"
    );
    assert_eq!(llm_requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn executor_analyzes_plan_via_mock_provider() {
    let router = mock_router().await;
    let mut executor = ExecutorAgent::with_router(AgentId::new(), router);
    let mut state = ExecutorState::default();

    let result = executor
        .handle_message(
            ExecutorMessage::ExecutePlan {
                plan: serde_json::json!({"steps": ["a", "b"]}),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap();

    assert!(result.is_object());
    assert!(state.executing);
}

#[tokio::test]
async fn tool_bus_round_trip_with_mock_provider() {
    let bus = ToolBus::new();
    let agent_id = AgentId::new();

    bus.register(
        "analyzer",
        "data",
        agent_id,
        "Analyzes data",
        serde_json::json!({"type": "object"}),
        serde_json::json!({}),
    );

    let defs = bus.to_tool_definitions();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "data.analyzer");

    let call = ToolCall {
        call_id: "call-gate9".into(),
        name: "data.analyzer".into(),
        input: serde_json::json!({"query": "test"}),
    };
    let result = bus.execute_tool_call_provider_result(None, &call).await;
    // No backend registered, so provider adapter returns a failure payload.
    assert!(result.error.is_some());
    assert_eq!(result.call_id, "call-gate9");
}

#[tokio::test]
async fn planner_falls_back_to_stub_without_router() {
    let mut planner = PlannerAgent::new(AgentId::new());
    let mut state = PlannerState::default();

    let result = planner
        .handle_message(
            PlannerMessage::PlanGoal {
                goal: "test goal".into(),
                context: serde_json::json!({}),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap();

    assert_eq!(result["goal"], "test goal");
    assert!(result["steps"].is_array());
}

#[tokio::test]
async fn planner_returns_error_when_router_has_no_providers() {
    let mut planner = PlannerAgent::with_router(AgentId::new(), empty_router());
    let mut state = PlannerState::default();

    let err = planner
        .handle_message(
            PlannerMessage::PlanGoal {
                goal: "no providers".into(),
                context: serde_json::json!({}),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap_err();

    assert!(err.to_string().starts_with("planner error:"));
    assert!(state.current_plan.is_none());
}

#[tokio::test]
async fn critic_returns_error_when_router_provider_fails() {
    let router = failing_provider_router().await;
    let mut critic = CriticAgent::with_router(AgentId::new(), router);
    let mut state = CriticState::default();

    let err = critic
        .handle_message(
            CriticMessage::Evaluate {
                output: serde_json::json!("bad output"),
                criteria: serde_json::json!(["accuracy"]),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap_err();

    assert!(err.to_string().starts_with("critic error:"));
    assert_eq!(state.evaluations_completed, 1);
}

#[tokio::test]
async fn executor_returns_error_when_router_has_no_providers() {
    let mut executor = ExecutorAgent::with_router(AgentId::new(), empty_router());
    let mut state = ExecutorState::default();

    let err = executor
        .handle_message(
            ExecutorMessage::ExecutePlan {
                plan: serde_json::json!({"steps": ["x"]}),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap_err();

    assert!(err.to_string().starts_with("executor error:"));
}

#[tokio::test]
async fn executor_router_failure_triggers_guard_supervision_and_forwarded_states() {
    let router = empty_router();
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Arc::new(Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    ));
    let workflow_id = TaskId::new();
    let (graph, branch_id, task_id) = supervision_graph(workflow_id);
    orchestrator.register_execution_graph(graph);

    let mut task = TaskAssignment::new("execute-plan", serde_json::json!({"branch": "a"}));
    task.task_id = task_id;
    scheduler.submit(task);
    scheduler.assign(&task_id, AgentId::new()).unwrap();
    scheduler.start(&task_id).unwrap();

    let supervision = LlmSupervision::new(
        orchestrator.clone(),
        workflow_id,
        LlmSupervisionConfig::new(GuardTarget::Branch(branch_id)),
    );
    let mut executor =
        ExecutorAgent::with_router_and_supervision(AgentId::new(), router, supervision);
    let mut state = ExecutorState::default();

    let err = executor
        .handle_message(
            ExecutorMessage::ExecutePlan {
                plan: serde_json::json!({"steps": ["analyze"]}),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .unwrap_err();

    assert!(err.to_string().starts_with("executor error:"));

    let status = orchestrator
        .autonomy_status(&workflow_id)
        .expect("autonomy status should exist");
    assert_eq!(status.guard_decisions.len(), 1);
    assert_eq!(status.interventions.len(), 1);
    assert_eq!(
        status.guard_decisions[0].failure_class,
        FailureClass::Streaming
    );

    let monitor_state = orchestrator
        .monitor_state(&workflow_id)
        .expect("monitor state should be forwarded");
    assert_eq!(monitor_state.guard_decisions.len(), 1);
    assert_eq!(monitor_state.interventions.len(), 1);

    let supervisor_state = orchestrator
        .supervisor_state(&workflow_id)
        .expect("supervisor state should be forwarded");
    assert_eq!(supervisor_state.guard_decisions.len(), 1);
    assert_eq!(supervisor_state.interventions.len(), 1);

    let branch = orchestrator
        .execution_graph(&workflow_id)
        .expect("graph should remain registered")
        .branch(&branch_id)
        .expect("branch should exist")
        .clone();
    assert_eq!(branch.state, mister_smith_core::BranchState::Checkpointed);
    assert_eq!(
        scheduler.get(&task_id).expect("task should exist").state,
        mister_smith_agents::config::TaskState::Pending
    );
}

#[tokio::test]
async fn executor_stream_repetition_triggers_guard_supervision_and_forwarded_states() {
    let router = repetitive_stream_router().await;
    let scheduler = Arc::new(TaskScheduler::new());
    let orchestrator = Arc::new(Orchestrator::new(
        Arc::new(IdentityDecomposer),
        Arc::new(ArrayAggregator),
        scheduler.clone(),
    ));
    let workflow_id = TaskId::new();
    let (graph, branch_id, task_id) = supervision_graph(workflow_id);
    orchestrator.register_execution_graph(graph);

    let mut task = TaskAssignment::new("execute-plan", serde_json::json!({"branch": "a"}));
    task.task_id = task_id;
    scheduler.submit(task);
    scheduler.assign(&task_id, AgentId::new()).unwrap();
    scheduler.start(&task_id).unwrap();

    let supervision = LlmSupervision::new(
        orchestrator.clone(),
        workflow_id,
        LlmSupervisionConfig::new(GuardTarget::Branch(branch_id)),
    );
    let mut executor =
        ExecutorAgent::with_router_and_supervision(AgentId::new(), router, supervision);
    let mut state = ExecutorState::default();

    let result = executor
        .handle_message(
            ExecutorMessage::ExecutePlan {
                plan: serde_json::json!({"steps": ["analyze"]}),
                managed_context: None,
            },
            &mut state,
        )
        .await
        .expect("streamed execution should still succeed");

    assert_eq!(result["status"], "executing");

    let status = orchestrator
        .autonomy_status(&workflow_id)
        .expect("autonomy status should exist");
    assert_eq!(status.guard_decisions.len(), 1);
    assert_eq!(status.interventions.len(), 1);
    assert_eq!(
        status.guard_decisions[0].failure_class,
        FailureClass::Semantic
    );
    assert_eq!(
        status.guard_decisions[0].intervention,
        InterventionType::ContextRefresh
    );

    let monitor_state = orchestrator
        .monitor_state(&workflow_id)
        .expect("monitor state should be forwarded");
    assert_eq!(monitor_state.guard_decisions.len(), 1);
    assert_eq!(monitor_state.interventions.len(), 1);

    let supervisor_state = orchestrator
        .supervisor_state(&workflow_id)
        .expect("supervisor state should be forwarded");
    assert_eq!(supervisor_state.guard_decisions.len(), 1);
    assert_eq!(supervisor_state.interventions.len(), 1);

    let branch = orchestrator
        .execution_graph(&workflow_id)
        .expect("graph should remain registered")
        .branch(&branch_id)
        .expect("branch should exist")
        .clone();
    assert_eq!(branch.state, mister_smith_core::BranchState::Checkpointed);
    assert_eq!(
        scheduler.get(&task_id).expect("task should exist").state,
        mister_smith_agents::config::TaskState::Pending
    );
}
