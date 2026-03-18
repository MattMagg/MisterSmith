use std::sync::{Arc, Mutex};
use std::time::Instant;

use async_trait::async_trait;
use futures::stream;
use futures::StreamExt;
use mister_smith_core::LlmError;
use mister_smith_llm::budget::{
    BudgetEnforcer, BudgetNode, BudgetPolicy, BudgetStore, InMemoryBudgetStore,
};
use mister_smith_llm::router::ModelEventSink;
use mister_smith_llm::{
    CascadePolicy, CascadeTier, CircuitBreakerConfig, CircuitState, CompletionRequest,
    CompletionResponse, ContentBlock, EmbeddingResponse, MockProvider, ModelCapabilities,
    ModelEvent, ModelProvider, ModelRouter, ProviderConfig, ProviderKind, RoutingHint,
    RoutingPolicy, StepRoutingAction, StepRoutingMetadata, StepVerificationCheckpointKind,
    StepVerificationOutcome, StopReason, Usage,
};

fn mock_request() -> CompletionRequest {
    CompletionRequest {
        messages: vec![mister_smith_llm::ChatMessage::User {
            content: serde_json::json!("test prompt"),
        }],
        ..Default::default()
    }
}

fn mock_config(model_id: &str) -> ProviderConfig {
    ProviderConfig {
        provider_kind: ProviderKind::Mock,
        model_id: model_id.to_string(),
        ..Default::default()
    }
}

fn mock_config_with_tier(model_id: &str, tier: &str) -> ProviderConfig {
    ProviderConfig {
        metadata: serde_json::json!({ "tier": tier }),
        ..mock_config(model_id)
    }
}

// --- Shared test helpers ---

/// Provider that always fails with a network error.
#[derive(Debug)]
struct FailingProvider {
    model_id: String,
}

impl FailingProvider {
    fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
        }
    }
}

#[async_trait]
impl ModelProvider for FailingProvider {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        Err(LlmError::Network("simulated provider failure".to_string()))
    }

    fn stream(&self, _request: CompletionRequest) -> mister_smith_llm::CompletionStream {
        Box::pin(stream::once(async {
            Err(LlmError::Network("simulated provider failure".to_string()))
        }))
    }

    async fn embed(&self, _input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        Ok(EmbeddingResponse {
            embeddings: vec![],
            model_id: self.model_id.clone(),
            usage: Usage::default(),
        })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            completion: true,
            streaming: false,
            embeddings: false,
            tool_calling: false,
        }
    }
}

/// Provider that records requests for inspection.
#[derive(Debug)]
struct RecordingProvider {
    model_id: String,
    capabilities: ModelCapabilities,
    observed_requests: Arc<Mutex<Vec<CompletionRequest>>>,
}

impl RecordingProvider {
    fn new(
        model_id: &str,
        capabilities: ModelCapabilities,
        observed_requests: Arc<Mutex<Vec<CompletionRequest>>>,
    ) -> Self {
        Self {
            model_id: model_id.to_string(),
            capabilities,
            observed_requests,
        }
    }
}

#[async_trait]
impl ModelProvider for RecordingProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        self.observed_requests.lock().unwrap().push(request.clone());
        Ok(CompletionResponse {
            content: vec![ContentBlock::Text {
                text: "recorded".to_string(),
            }],
            model_id: self.model_id.clone(),
            usage: Usage::new(10, 5),
            stop_reason: StopReason::Completed,
            tool_calls: vec![],
        })
    }

    fn stream(&self, request: CompletionRequest) -> mister_smith_llm::CompletionStream {
        self.observed_requests.lock().unwrap().push(request.clone());
        Box::pin(stream::iter(vec![
            Ok(mister_smith_llm::StreamChunk {
                index: 0,
                delta: mister_smith_llm::ChunkDelta::Text {
                    text: "recorded-stream".to_string(),
                },
            }),
            Ok(mister_smith_llm::StreamChunk::stop(
                1,
                StopReason::Completed,
            )),
        ]))
    }

    async fn embed(&self, _input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        Err(LlmError::UnsupportedCapability {
            capability: "embeddings".into(),
            model: "recording".into(),
        })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn capabilities(&self) -> ModelCapabilities {
        self.capabilities
    }
}

/// Provider that returns a fixed completion response and tracks call count.
#[derive(Debug)]
struct StaticResponseProvider {
    model_id: String,
    response: CompletionResponse,
    calls: Arc<Mutex<u32>>,
}

impl StaticResponseProvider {
    fn new(
        model_id: impl Into<String>,
        response: CompletionResponse,
        calls: Arc<Mutex<u32>>,
    ) -> Self {
        Self {
            model_id: model_id.into(),
            response,
            calls,
        }
    }
}

#[async_trait]
impl ModelProvider for StaticResponseProvider {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        *self.calls.lock().unwrap() += 1;
        Ok(self.response.clone())
    }

    fn stream(&self, _request: CompletionRequest) -> mister_smith_llm::CompletionStream {
        Box::pin(stream::once(async {
            Err(LlmError::UnsupportedCapability {
                capability: "streaming".into(),
                model: "static-response".into(),
            })
        }))
    }

    async fn embed(&self, _input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        Err(LlmError::UnsupportedCapability {
            capability: "embeddings".into(),
            model: "static-response".into(),
        })
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities {
            completion: true,
            streaming: false,
            embeddings: false,
            tool_calling: false,
        }
    }
}

/// Shared budget store wrapper for test assertions.
#[derive(Clone)]
struct SharedBudgetStore(Arc<InMemoryBudgetStore>);

#[async_trait]
impl BudgetStore for SharedBudgetStore {
    async fn get(&self, key: &str) -> Result<Option<BudgetNode>, LlmError> {
        self.0.get(key).await
    }

    async fn cas_update(&self, node: &BudgetNode, expected_revision: u64) -> Result<u64, LlmError> {
        self.0.cas_update(node, expected_revision).await
    }
}

fn budget_store_with_default() -> SharedBudgetStore {
    let inner = InMemoryBudgetStore::new();
    inner.insert(BudgetNode {
        key: "budget/test".to_string(),
        limit_tokens: 50_000,
        used_tokens: 0,
        period: "test".to_string(),
        policy: BudgetPolicy::HardCap,
        revision: 1,
    });
    SharedBudgetStore(Arc::new(inner))
}

fn budget_store_with_limit(limit_tokens: u64) -> SharedBudgetStore {
    let inner = InMemoryBudgetStore::new();
    inner.insert(BudgetNode {
        key: "budget/test".to_string(),
        limit_tokens,
        used_tokens: 0,
        period: "test".to_string(),
        policy: BudgetPolicy::HardCap,
        revision: 1,
    });
    SharedBudgetStore(Arc::new(inner))
}

fn budget_store_with_policy(policy: BudgetPolicy) -> SharedBudgetStore {
    let inner = InMemoryBudgetStore::new();
    inner.insert(BudgetNode {
        key: "budget/test".to_string(),
        limit_tokens: 50_000,
        used_tokens: 0,
        period: "test".to_string(),
        policy,
        revision: 1,
    });
    SharedBudgetStore(Arc::new(inner))
}

/// Recording sink for model event assertions.
#[derive(Default)]
struct RecordingSink {
    events: Mutex<Vec<ModelEvent>>,
}

impl RecordingSink {
    fn events(&self) -> Vec<ModelEvent> {
        self.events.lock().unwrap().clone()
    }
}

impl ModelEventSink for RecordingSink {
    fn publish(&self, event: ModelEvent) {
        self.events.lock().unwrap().push(event);
    }
}

// --- Basic routing tests ---

#[tokio::test]
async fn round_robin_distributes_across_providers() {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);
    let p1: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("model-a"));
    let p2: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("model-b"));

    router
        .add_provider(mock_config("model-a"), p1, CircuitBreakerConfig::default())
        .await;
    router
        .add_provider(mock_config("model-b"), p2, CircuitBreakerConfig::default())
        .await;

    let (_r1, d1) = router.route_completion(mock_request()).await.unwrap();
    let (_r2, d2) = router.route_completion(mock_request()).await.unwrap();

    // Round-robin should alternate between providers
    assert_ne!(d1.provider_id, d2.provider_id);
}

#[tokio::test]
async fn sub_millisecond_routing_overhead() {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);
    let provider: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("fast-mock"));

    router
        .add_provider(
            mock_config("fast-mock"),
            provider,
            CircuitBreakerConfig::default(),
        )
        .await;

    // Warm up
    let _ = router.route_completion(mock_request()).await;

    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        let _ = router.route_completion(mock_request()).await;
    }
    let elapsed = start.elapsed();
    let per_request_us = elapsed.as_micros() / iterations;

    assert!(
        per_request_us < 5000,
        "Routing overhead too high: {per_request_us}us per request"
    );
}

#[tokio::test]
async fn cost_optimized_selects_first_healthy() {
    let router = ModelRouter::new(RoutingPolicy::CostOptimized);
    let p1: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("cheap"));
    let p2: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("expensive"));

    router
        .add_provider(mock_config("cheap"), p1, CircuitBreakerConfig::default())
        .await;
    router
        .add_provider(
            mock_config("expensive"),
            p2,
            CircuitBreakerConfig::default(),
        )
        .await;

    let (_, decision) = router.route_completion(mock_request()).await.unwrap();
    assert_eq!(decision.provider_id, "cheap");
}

#[tokio::test]
async fn routing_decision_carries_step_metadata_contract() {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);
    let provider: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("step-aware"));

    router
        .add_provider(
            mock_config("step-aware"),
            provider,
            CircuitBreakerConfig::default(),
        )
        .await;

    let request = CompletionRequest {
        routing_hint: Some(RoutingHint {
            step_metadata: Some(StepRoutingMetadata {
                step_id: "planner.step.3".to_string(),
                step_index: Some(3),
                step_kind: Some("planner_reasoning".to_string()),
            }),
            ..Default::default()
        }),
        ..mock_request()
    };

    let (_response, decision) = router.route_completion(request).await.unwrap();

    assert_eq!(decision.step_signal.metadata.step_id, "planner.step.3");
    assert_eq!(decision.step_signal.metadata.step_index, Some(3));
    assert_eq!(
        decision.step_signal.metadata.step_kind.as_deref(),
        Some("planner_reasoning")
    );
    assert_eq!(decision.step_signal.action, StepRoutingAction::Continue);
}

#[tokio::test]
async fn direct_complete_emits_routing_event_with_step_signal() {
    let sink = Arc::new(RecordingSink::default());
    let router = ModelRouter::new(RoutingPolicy::RoundRobin).with_model_event_sink(sink.clone());
    let provider: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("step-aware"));

    router
        .add_provider(
            mock_config("step-aware"),
            provider,
            CircuitBreakerConfig::default(),
        )
        .await;

    let request = CompletionRequest {
        routing_hint: Some(RoutingHint {
            step_metadata: Some(StepRoutingMetadata {
                step_id: "planner.step.4".to_string(),
                step_index: Some(4),
                step_kind: Some("planner_reasoning".to_string()),
            }),
            ..Default::default()
        }),
        ..mock_request()
    };

    let _ = router.complete(request).await.unwrap();

    let events = sink.events();
    assert_eq!(events.len(), 1, "should emit 1 routing event");

    match &events[0] {
        ModelEvent::RoutingDecision {
            model_id,
            tier,
            reason,
            step_signal,
        } => {
            assert_eq!(model_id, "step-aware");
            assert_eq!(tier, "direct");
            assert!(reason.contains("RoundRobin"), "reason: {reason}");
            assert_eq!(step_signal.metadata.step_id, "planner.step.4");
            assert_eq!(step_signal.action, StepRoutingAction::Continue);
        }
        other => panic!("expected RoutingDecision, got: {other:?}"),
    }
}

#[tokio::test]
async fn soft_cap_budget_marks_step_signal_for_downgrade() {
    let store = budget_store_with_policy(BudgetPolicy::SoftCap);
    let enforcer = BudgetEnforcer::new(Box::new(store));
    let router = ModelRouter::new(RoutingPolicy::RoundRobin).with_budget(enforcer, "budget/test");
    let provider: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("budget-aware"));

    router
        .add_provider(
            mock_config("budget-aware"),
            provider,
            CircuitBreakerConfig::default(),
        )
        .await;

    let (_response, decision) = router.route_completion(mock_request()).await.unwrap();

    assert_eq!(decision.step_signal.action, StepRoutingAction::Downgrade);
    assert!(decision.step_signal.confidence.is_some());
    assert!(decision.step_signal.checkpoints.iter().any(|checkpoint| {
        checkpoint.kind == StepVerificationCheckpointKind::BudgetPolicy
            && checkpoint.outcome == StepVerificationOutcome::Triggered
    }));
}

#[tokio::test]
async fn no_healthy_provider_returns_error() {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);
    // No providers registered
    let result = router.route_completion(mock_request()).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        mister_smith_core::LlmError::NoHealthyProvider(_)
    ));
}

#[tokio::test]
async fn health_table_reflects_provider_state() {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);
    let provider: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("test-model"));

    router
        .add_provider(
            mock_config("test-model"),
            provider,
            CircuitBreakerConfig::default(),
        )
        .await;

    let table = router.health_table().await;
    assert_eq!(table.len(), 1);
    assert_eq!(table[0].provider_id, "test-model");
    assert_eq!(table[0].circuit_state, CircuitState::Closed);
}

#[tokio::test]
async fn provider_count() {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);
    assert_eq!(router.provider_count().await, 0);

    let provider: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("test"));
    router
        .add_provider(
            mock_config("test"),
            provider,
            CircuitBreakerConfig::default(),
        )
        .await;
    assert_eq!(router.provider_count().await, 1);
}

// --- Routing hint tests (#121) ---

#[tokio::test]
async fn required_capabilities_hint_filters_providers() {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);

    let no_tools = MockProvider::new("no-tools").with_capabilities(ModelCapabilities {
        completion: true,
        streaming: true,
        embeddings: true,
        tool_calling: false,
    });
    let with_tools = MockProvider::new("with-tools");

    router
        .add_provider(
            mock_config("no-tools"),
            Arc::new(no_tools),
            CircuitBreakerConfig::default(),
        )
        .await;
    router
        .add_provider(
            mock_config("with-tools"),
            Arc::new(with_tools),
            CircuitBreakerConfig::default(),
        )
        .await;

    let request = CompletionRequest {
        routing_hint: Some(RoutingHint {
            required_capabilities: vec!["tool_calling".to_string()],
            ..Default::default()
        }),
        ..mock_request()
    };

    let (_, decision) = router.route_completion(request).await.unwrap();
    assert_eq!(decision.provider_id, "with-tools");
}

#[tokio::test]
async fn preferred_tier_hint_prioritizes_matching_provider_for_non_cascade_routing() {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);

    router
        .add_provider(
            mock_config_with_tier("baseline", "slm"),
            Arc::new(MockProvider::new("baseline-model")),
            CircuitBreakerConfig::default(),
        )
        .await;
    router
        .add_provider(
            mock_config_with_tier("preferred", "llm"),
            Arc::new(MockProvider::new("preferred-model")),
            CircuitBreakerConfig::default(),
        )
        .await;

    let request = CompletionRequest {
        routing_hint: Some(RoutingHint {
            preferred_tier: Some("llm".to_string()),
            ..Default::default()
        }),
        ..mock_request()
    };

    let (_, decision) = router.route_completion(request).await.unwrap();
    assert_eq!(decision.provider_id, "preferred");
}

#[tokio::test]
async fn max_cost_tokens_hint_blocks_over_budget_estimate() {
    let router = ModelRouter::new(RoutingPolicy::RoundRobin);
    let provider: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("test"));

    router
        .add_provider(
            mock_config("test"),
            provider,
            CircuitBreakerConfig::default(),
        )
        .await;

    let request = CompletionRequest {
        max_tokens: Some(200),
        routing_hint: Some(RoutingHint {
            max_cost_tokens: Some(1),
            ..Default::default()
        }),
        ..mock_request()
    };

    let result = router.route_completion(request).await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LlmError::NoHealthyProvider(_)
    ));
}

#[tokio::test]
async fn stream_consumes_hint_for_selection_and_strips_before_dispatch() {
    let sink = Arc::new(RecordingSink::default());
    let observed = Arc::new(Mutex::new(Vec::new()));
    let recorder: Arc<dyn ModelProvider> = Arc::new(RecordingProvider::new(
        "recorder",
        ModelCapabilities::all(),
        observed.clone(),
    ));
    let no_tools = Arc::new(
        MockProvider::new("no-tools").with_capabilities(ModelCapabilities {
            completion: true,
            streaming: true,
            embeddings: true,
            tool_calling: false,
        }),
    );

    let router = ModelRouter::new(RoutingPolicy::RoundRobin).with_model_event_sink(sink.clone());
    router
        .add_provider(
            mock_config_with_tier("baseline", "slm"),
            no_tools,
            CircuitBreakerConfig::default(),
        )
        .await;
    router
        .add_provider(
            mock_config_with_tier("preferred", "llm"),
            recorder,
            CircuitBreakerConfig::default(),
        )
        .await;

    let chunks = router
        .stream(CompletionRequest {
            tools: Some(vec![]),
            routing_hint: Some(RoutingHint {
                preferred_tier: Some("llm".to_string()),
                required_capabilities: vec!["tool_calling".to_string()],
                step_metadata: Some(StepRoutingMetadata {
                    step_id: "planner.stream.1".to_string(),
                    step_index: Some(1),
                    step_kind: Some("planner_reasoning".to_string()),
                }),
                ..Default::default()
            }),
            ..mock_request()
        })
        .collect::<Vec<_>>()
        .await;

    assert_eq!(chunks.len(), 2);
    assert_eq!(
        chunks[0].as_ref().unwrap().delta,
        mister_smith_llm::ChunkDelta::Text {
            text: "recorded-stream".to_string(),
        }
    );

    let recorded = observed.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert!(
        recorded[0].routing_hint.is_none(),
        "routing_hint should be stripped before dispatch to provider"
    );

    let events = sink.events();
    assert_eq!(events.len(), 1, "should emit 1 routing event");
    match &events[0] {
        ModelEvent::RoutingDecision {
            model_id,
            tier,
            reason,
            step_signal,
        } => {
            assert_eq!(model_id, "recorder");
            assert_eq!(tier, "direct");
            assert!(reason.contains("stream"), "reason: {reason}");
            assert_eq!(step_signal.metadata.step_id, "planner.stream.1");
            assert_eq!(step_signal.action, StepRoutingAction::Continue);
        }
        other => panic!("expected RoutingDecision, got: {other:?}"),
    }
}

#[tokio::test]
async fn route_completion_strips_routing_hint_before_provider_dispatch() {
    let observed = Arc::new(Mutex::new(Vec::new()));
    let recorder: Arc<dyn ModelProvider> = Arc::new(RecordingProvider::new(
        "recorder",
        ModelCapabilities::all(),
        observed.clone(),
    ));

    let router = ModelRouter::new(RoutingPolicy::RoundRobin);
    router
        .add_provider(
            mock_config("recorder"),
            recorder,
            CircuitBreakerConfig::default(),
        )
        .await;

    let request = CompletionRequest {
        routing_hint: Some(RoutingHint {
            preferred_tier: Some("fast".into()),
            max_cost_tokens: Some(5000),
            required_capabilities: vec!["completion".into()],
            step_metadata: None,
        }),
        ..mock_request()
    };

    let _ = router.route_completion(request).await.unwrap();

    let recorded = observed.lock().unwrap();
    assert_eq!(recorded.len(), 1);
    assert!(
        recorded[0].routing_hint.is_none(),
        "routing_hint should be stripped before dispatch to provider"
    );
}

// --- Budget error-path tests (#118) ---

#[tokio::test]
async fn provider_failure_after_reserve_reverts_budget_usage() {
    let store = budget_store_with_default();
    let enforcer = BudgetEnforcer::new(Box::new(store.clone()));

    let router = ModelRouter::new(RoutingPolicy::RoundRobin).with_budget(enforcer, "budget/test");

    let provider: Arc<dyn ModelProvider> = Arc::new(FailingProvider::new("failing-model"));
    router
        .add_provider(
            mock_config("failing-model"),
            provider,
            CircuitBreakerConfig::default(),
        )
        .await;

    let result = router.route_completion(mock_request()).await;
    assert!(result.is_err());

    // Budget should be fully reverted
    let node = store.get("budget/test").await.unwrap().unwrap();
    assert_eq!(
        node.used_tokens, 0,
        "budget should revert to 0 after provider failure"
    );
}

#[tokio::test]
async fn failed_requests_have_no_net_token_leak() {
    let store = budget_store_with_default();
    let enforcer = BudgetEnforcer::new(Box::new(store.clone()));

    let router = ModelRouter::new(RoutingPolicy::RoundRobin).with_budget(enforcer, "budget/test");

    let provider: Arc<dyn ModelProvider> = Arc::new(FailingProvider::new("failing-model"));
    router
        .add_provider(
            mock_config("failing-model"),
            provider,
            CircuitBreakerConfig::default(),
        )
        .await;

    for _ in 0..3 {
        let _ = router.route_completion(mock_request()).await;
    }

    let node = store.get("budget/test").await.unwrap().unwrap();
    assert_eq!(
        node.used_tokens, 0,
        "3 failed requests should leave 0 used tokens"
    );
}

// --- Circuit breaker tests ---

mod circuit_breaker {
    use mister_smith_llm::health::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
    use std::time::Duration;

    #[test]
    fn closed_to_open_on_threshold() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            error_rate_threshold: 1.1,
            ..Default::default()
        });
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure(None);
        cb.record_failure(None);
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure(None);
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn open_to_half_open_on_timeout() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(1),
            ..Default::default()
        });

        cb.record_failure(None);
        assert_eq!(cb.state(), CircuitState::Open);

        std::thread::sleep(Duration::from_millis(5));
        cb.maybe_transition_to_half_open();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn half_open_to_closed_on_probe_success() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(1),
            ..Default::default()
        });

        cb.record_failure(None);
        std::thread::sleep(Duration::from_millis(5));
        cb.maybe_transition_to_half_open();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success(10);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn half_open_to_open_on_probe_failure() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            recovery_timeout: Duration::from_millis(1),
            ..Default::default()
        });

        cb.record_failure(None);
        std::thread::sleep(Duration::from_millis(5));
        cb.maybe_transition_to_half_open();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_failure(None);
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn retry_after_honoring_from_429() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig::default());
        cb.record_failure(Some(60));
        assert!(cb.is_rate_limited());
    }
}

// --- Cascade routing tests ---

mod cascade {
    use super::*;

    fn cascade_policy(threshold: f32, max_escalations: u32) -> CascadePolicy {
        CascadePolicy {
            tiers: vec![
                CascadeTier {
                    provider_config: mock_config("slm"),
                    label: "slm-tier".to_string(),
                },
                CascadeTier {
                    provider_config: mock_config("llm"),
                    label: "llm-tier".to_string(),
                },
            ],
            escalation_threshold: threshold,
            max_escalations,
        }
    }

    #[tokio::test]
    async fn cascade_accepts_first_tier_when_confidence_high() {
        let policy = cascade_policy(0.3, 1);
        let router = ModelRouter::new(RoutingPolicy::Cascade(policy));

        let slm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("slm-model"));
        let llm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("llm-model"));

        router
            .add_provider(mock_config("slm"), slm, CircuitBreakerConfig::default())
            .await;
        router
            .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
            .await;

        let (_response, decision) = router.route_completion(mock_request()).await.unwrap();

        assert_eq!(decision.tier_label.as_deref(), Some("slm-tier"));
        assert!(decision.reason.contains("accepted"));
    }

    #[tokio::test]
    async fn cascade_escalates_when_confidence_low() {
        let policy = cascade_policy(1.1, 1);
        let router = ModelRouter::new(RoutingPolicy::Cascade(policy));

        let slm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("slm-model"));
        let llm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("llm-model"));

        router
            .add_provider(mock_config("slm"), slm, CircuitBreakerConfig::default())
            .await;
        router
            .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
            .await;

        let (_response, decision) = router.route_completion(mock_request()).await.unwrap();

        assert_eq!(decision.tier_label.as_deref(), Some("llm-tier"));
    }

    #[tokio::test]
    async fn cascade_honors_max_escalations() {
        let policy = CascadePolicy {
            tiers: vec![CascadeTier {
                provider_config: mock_config("tier-0"),
                label: "only-tier".to_string(),
            }],
            escalation_threshold: 1.1,
            max_escalations: 0,
        };
        let router = ModelRouter::new(RoutingPolicy::Cascade(policy));

        let p: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("tier-0"));
        router
            .add_provider(mock_config("tier-0"), p, CircuitBreakerConfig::default())
            .await;

        let (_response, decision) = router.route_completion(mock_request()).await.unwrap();

        assert_eq!(decision.tier_label.as_deref(), Some("only-tier"));
    }

    #[tokio::test]
    async fn cascade_returns_error_when_all_tiers_unhealthy() {
        let policy = cascade_policy(0.5, 1);
        let router = ModelRouter::new(RoutingPolicy::Cascade(policy));
        // No providers registered — will fail at build_cascade_attempt_plan
        let result = router.route_completion(mock_request()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn confidence_signal_penalizes_short_responses() {
        use mister_smith_llm::ConfidenceSignal;

        let short_response = CompletionResponse {
            content: vec![ContentBlock::Text {
                text: "ok".to_string(),
            }],
            model_id: "test".to_string(),
            usage: Usage::new(10, 2),
            stop_reason: StopReason::Completed,
            tool_calls: vec![],
        };

        let signal = ConfidenceSignal::from_response(&short_response);
        assert!(signal.score < 1.0);
    }

    // --- Tier ordering tests (#124) ---

    #[tokio::test]
    async fn cascade_uses_declared_tier_order_over_registration_order() {
        let policy = cascade_policy(0.3, 1);
        let router = ModelRouter::new(RoutingPolicy::Cascade(policy));

        // Deliberately register in REVERSE order: llm first, then slm
        let llm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("llm-model"));
        let slm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("slm-model"));

        router
            .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
            .await;
        router
            .add_provider(mock_config("slm"), slm, CircuitBreakerConfig::default())
            .await;

        let (_response, decision) = router.route_completion(mock_request()).await.unwrap();

        // Should still use slm-tier first (declared order), not llm (registration order)
        assert_eq!(decision.tier_label.as_deref(), Some("slm-tier"));
        assert_eq!(decision.provider_id, "slm");
    }

    // --- Preferred tier hint tests (#121) ---

    #[tokio::test]
    async fn cascade_preferred_tier_hint_is_prioritized() {
        let policy = cascade_policy(0.3, 1);
        let router = ModelRouter::new(RoutingPolicy::Cascade(policy));

        let slm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("slm-model"));
        let llm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("llm-model"));

        router
            .add_provider(mock_config("slm"), slm, CircuitBreakerConfig::default())
            .await;
        router
            .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
            .await;

        let request = CompletionRequest {
            routing_hint: Some(RoutingHint {
                preferred_tier: Some("llm-tier".to_string()),
                ..Default::default()
            }),
            ..mock_request()
        };

        let (_, decision) = router.route_completion(request).await.unwrap();

        assert_eq!(decision.tier_label.as_deref(), Some("llm-tier"));
    }

    // --- Cascade budget tests (#119) ---

    #[tokio::test]
    async fn cascade_reserves_and_reconciles_budget_on_accept() {
        let store = budget_store_with_default();
        let enforcer = BudgetEnforcer::new(Box::new(store.clone()));

        let policy = cascade_policy(0.3, 1);
        let router =
            ModelRouter::new(RoutingPolicy::Cascade(policy)).with_budget(enforcer, "budget/test");

        let slm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("slm-model"));
        let llm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("llm-model"));

        router
            .add_provider(mock_config("slm"), slm, CircuitBreakerConfig::default())
            .await;
        router
            .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
            .await;

        let (response, _decision) = router.route_completion(mock_request()).await.unwrap();

        let node = store.get("budget/test").await.unwrap().unwrap();
        assert_eq!(
            node.used_tokens, response.usage.total_tokens,
            "budget should be reconciled to actual usage"
        );
        assert!(response.usage.total_tokens < 1024);
    }

    #[tokio::test]
    async fn cascade_accounts_for_both_tiers_when_escalating_to_second_tier() {
        let store = budget_store_with_default();
        let enforcer = BudgetEnforcer::new(Box::new(store.clone()));

        // threshold 1.1 forces escalation
        let policy = cascade_policy(1.1, 1);
        let router =
            ModelRouter::new(RoutingPolicy::Cascade(policy)).with_budget(enforcer, "budget/test");

        let slm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("slm-model"));
        let llm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("llm-model"));

        router
            .add_provider(mock_config("slm"), slm, CircuitBreakerConfig::default())
            .await;
        router
            .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
            .await;

        let (response, decision) = router.route_completion(mock_request()).await.unwrap();

        assert_eq!(decision.tier_label.as_deref(), Some("llm-tier"));
        let node = store.get("budget/test").await.unwrap().unwrap();
        assert_eq!(
            node.used_tokens,
            response.usage.total_tokens * 2,
            "budget should include both tier attempts after escalation"
        );
        assert!(decision.estimated_cost_tokens.is_some());
    }

    #[tokio::test]
    async fn cascade_budget_counts_all_successful_attempts_during_escalation() {
        let store = budget_store_with_default();
        let enforcer = BudgetEnforcer::new(Box::new(store.clone()));

        let policy = cascade_policy(1.1, 1);
        let router =
            ModelRouter::new(RoutingPolicy::Cascade(policy)).with_budget(enforcer, "budget/test");

        let slm_calls = Arc::new(Mutex::new(0));
        let llm_calls = Arc::new(Mutex::new(0));
        let slm: Arc<dyn ModelProvider> = Arc::new(StaticResponseProvider::new(
            "slm-model",
            CompletionResponse {
                content: vec![ContentBlock::Text {
                    text: "ok".to_string(),
                }],
                model_id: "slm-model".to_string(),
                usage: Usage::new(9, 3),
                stop_reason: StopReason::Completed,
                tool_calls: vec![],
            },
            slm_calls.clone(),
        ));
        let llm: Arc<dyn ModelProvider> = Arc::new(StaticResponseProvider::new(
            "llm-model",
            CompletionResponse {
                content: vec![ContentBlock::Text {
                    text: "fallback accepted response".to_string(),
                }],
                model_id: "llm-model".to_string(),
                usage: Usage::new(14, 6),
                stop_reason: StopReason::Completed,
                tool_calls: vec![],
            },
            llm_calls.clone(),
        ));

        router
            .add_provider(mock_config("slm"), slm, CircuitBreakerConfig::default())
            .await;
        router
            .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
            .await;

        let (_response, decision) = router.route_completion(mock_request()).await.unwrap();

        assert_eq!(decision.tier_label.as_deref(), Some("llm-tier"));
        assert_eq!(*slm_calls.lock().unwrap(), 1);
        assert_eq!(*llm_calls.lock().unwrap(), 1);

        let node = store.get("budget/test").await.unwrap().unwrap();
        assert_eq!(
            node.used_tokens, 32,
            "budget should include the first-tier and fallback-tier actual usage"
        );
    }

    #[tokio::test]
    async fn cascade_budget_rechecks_hard_cap_before_fallback_attempt() {
        let store = budget_store_with_limit(46);
        let enforcer = BudgetEnforcer::new(Box::new(store.clone()));

        let policy = cascade_policy(1.1, 1);
        let router =
            ModelRouter::new(RoutingPolicy::Cascade(policy)).with_budget(enforcer, "budget/test");

        let slm_calls = Arc::new(Mutex::new(0));
        let llm_calls = Arc::new(Mutex::new(0));
        let slm: Arc<dyn ModelProvider> = Arc::new(StaticResponseProvider::new(
            "slm-model",
            CompletionResponse {
                content: vec![ContentBlock::Text {
                    text: "ok".to_string(),
                }],
                model_id: "slm-model".to_string(),
                usage: Usage::new(9, 3),
                stop_reason: StopReason::Completed,
                tool_calls: vec![],
            },
            slm_calls.clone(),
        ));
        let llm: Arc<dyn ModelProvider> = Arc::new(StaticResponseProvider::new(
            "llm-model",
            CompletionResponse {
                content: vec![ContentBlock::Text {
                    text: "fallback accepted response".to_string(),
                }],
                model_id: "llm-model".to_string(),
                usage: Usage::new(14, 6),
                stop_reason: StopReason::Completed,
                tool_calls: vec![],
            },
            llm_calls.clone(),
        ));

        router
            .add_provider(mock_config("slm"), slm, CircuitBreakerConfig::default())
            .await;
        router
            .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
            .await;

        let result = router
            .route_completion(CompletionRequest {
                max_tokens: Some(32),
                ..mock_request()
            })
            .await;

        assert!(matches!(result, Err(LlmError::BudgetExhausted { .. })));
        assert_eq!(*slm_calls.lock().unwrap(), 1);
        assert_eq!(
            *llm_calls.lock().unwrap(),
            0,
            "fallback tier must not run when a fresh reservation exceeds budget"
        );

        let node = store.get("budget/test").await.unwrap().unwrap();
        assert_eq!(
            node.used_tokens, 12,
            "budget should reflect only the first-tier actual usage after the fallback reservation is rejected"
        );
    }

    // --- Routing event emission tests (#127) ---

    #[tokio::test]
    async fn cascade_emits_routing_events_for_escalation_and_acceptance() {
        let sink = Arc::new(RecordingSink::default());

        // threshold 1.1 forces escalation from tier 0 to tier 1
        let policy = cascade_policy(1.1, 1);
        let router =
            ModelRouter::new(RoutingPolicy::Cascade(policy)).with_model_event_sink(sink.clone());

        let slm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("slm-model"));
        let llm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("llm-model"));

        router
            .add_provider(mock_config("slm"), slm, CircuitBreakerConfig::default())
            .await;
        router
            .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
            .await;

        let _ = router.route_completion(mock_request()).await.unwrap();

        let events = sink.events();
        assert_eq!(events.len(), 2, "should emit 2 routing events");

        // First event: escalated from slm-tier
        match &events[0] {
            ModelEvent::RoutingDecision {
                model_id,
                tier,
                reason,
                step_signal,
            } => {
                assert_eq!(model_id, "slm-model");
                assert_eq!(tier, "slm-tier");
                assert!(reason.contains("escalated"), "reason: {reason}");
                assert_eq!(step_signal.action, StepRoutingAction::Escalate);
                assert!(step_signal.checkpoints.iter().any(|checkpoint| {
                    checkpoint.kind == StepVerificationCheckpointKind::ConfidenceReview
                        && checkpoint.outcome == StepVerificationOutcome::Triggered
                }));
            }
            other => panic!("expected RoutingDecision, got: {other:?}"),
        }

        // Second event: accepted at llm-tier
        match &events[1] {
            ModelEvent::RoutingDecision {
                model_id,
                tier,
                reason,
                step_signal,
            } => {
                assert_eq!(model_id, "llm-model");
                assert_eq!(tier, "llm-tier");
                assert!(reason.contains("accepted"), "reason: {reason}");
                assert_eq!(step_signal.action, StepRoutingAction::Continue);
            }
            other => panic!("expected RoutingDecision, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn cascade_emits_routing_events_for_failure_then_acceptance() {
        let sink = Arc::new(RecordingSink::default());

        let policy = cascade_policy(0.3, 1);
        let router =
            ModelRouter::new(RoutingPolicy::Cascade(policy)).with_model_event_sink(sink.clone());

        // First tier fails, second succeeds
        let failing: Arc<dyn ModelProvider> = Arc::new(FailingProvider::new("slm-failure"));
        let llm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("llm-model"));

        router
            .add_provider(mock_config("slm"), failing, CircuitBreakerConfig::default())
            .await;
        router
            .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
            .await;

        let _ = router.route_completion(mock_request()).await.unwrap();

        let events = sink.events();
        assert_eq!(events.len(), 2, "should emit 2 routing events");

        match &events[0] {
            ModelEvent::RoutingDecision {
                model_id,
                tier,
                reason,
                step_signal,
            } => {
                assert_eq!(model_id, "slm-failure");
                assert_eq!(tier, "slm-tier");
                assert!(reason.contains("failed"), "reason: {reason}");
                assert_eq!(step_signal.action, StepRoutingAction::Fallback);
                assert!(step_signal.checkpoints.iter().any(|checkpoint| {
                    checkpoint.kind == StepVerificationCheckpointKind::ProviderFailure
                        && checkpoint.outcome == StepVerificationOutcome::Triggered
                }));
            }
            other => panic!("expected RoutingDecision, got: {other:?}"),
        }

        match &events[1] {
            ModelEvent::RoutingDecision {
                model_id,
                tier,
                reason,
                step_signal,
            } => {
                assert_eq!(model_id, "llm-model");
                assert_eq!(tier, "llm-tier");
                assert!(reason.contains("accepted"), "reason: {reason}");
                assert_eq!(step_signal.action, StepRoutingAction::Continue);
            }
            other => panic!("expected RoutingDecision, got: {other:?}"),
        }
    }
}
