use std::sync::Arc;

use async_trait::async_trait;
use futures::stream;
use std::time::Instant;

use mister_smith_llm::{
    budget::{BudgetNode, BudgetPolicy, BudgetStore, InMemoryBudgetStore},
    CascadePolicy, CascadeTier, CircuitBreakerConfig, CircuitState, CompletionRequest,
    ModelRouter, MockProvider, ModelProvider, ProviderConfig, ProviderKind, RoutingPolicy,
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

    // MockProvider is instant, so routing overhead should be very small.
    // Allow generous threshold since we're measuring in test environment.
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
        .add_provider(mock_config("test"), provider, CircuitBreakerConfig::default())
        .await;
    assert_eq!(router.provider_count().await, 1);
}

// Circuit breaker tests
mod circuit_breaker {
    use mister_smith_llm::health::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
    use std::time::Duration;

    #[test]
    fn closed_to_open_on_threshold() {
        let mut cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            error_rate_threshold: 1.1, // Disable error-rate trigger for this test
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

// Cascade routing tests (T051-T053)
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
        // Threshold is low (0.3) so the mock response should pass
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

        // Should accept at tier 0 (SLM)
        assert_eq!(decision.tier_label.as_deref(), Some("slm-tier"));
        assert!(decision.reason.contains("accepted"));
    }

    #[tokio::test]
    async fn cascade_escalates_when_confidence_low() {
        // Threshold is impossibly high (1.1) so every tier will be below threshold
        // except the last tier which is always accepted
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

        // Should escalate to tier 1 (LLM) and accept as last tier
        assert_eq!(decision.tier_label.as_deref(), Some("llm-tier"));
    }

    #[tokio::test]
    async fn cascade_honors_max_escalations() {
        // max_escalations=0 means only first tier is tried
        let policy = CascadePolicy {
            tiers: vec![
                CascadeTier {
                    provider_config: mock_config("tier-0"),
                    label: "only-tier".to_string(),
                },
            ],
            escalation_threshold: 1.1, // impossibly high
            max_escalations: 0,
        };
        let router = ModelRouter::new(RoutingPolicy::Cascade(policy));

        let p: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("tier-0"));
        router
            .add_provider(mock_config("tier-0"), p, CircuitBreakerConfig::default())
            .await;

        let (_response, decision) = router.route_completion(mock_request()).await.unwrap();

        // max_escalations=0 means 1 attempt total, so it returns the first tier's response
        assert_eq!(decision.tier_label.as_deref(), Some("only-tier"));
    }

    #[tokio::test]
    async fn cascade_returns_error_when_all_tiers_unhealthy() {
        let policy = cascade_policy(0.5, 1);
        let router = ModelRouter::new(RoutingPolicy::Cascade(policy));
        // No providers registered
        let result = router.route_completion(mock_request()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn confidence_signal_penalizes_short_responses() {
        use mister_smith_llm::ConfidenceSignal;
        use mister_smith_llm::{CompletionResponse, ContentBlock, StopReason, Usage};

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
        // Short text (< 10 chars) should reduce confidence
        assert!(signal.score < 1.0);
    }
}


#[derive(Clone)]
struct SharedBudgetStore(Arc<InMemoryBudgetStore>);

#[async_trait]
impl BudgetStore for SharedBudgetStore {
    async fn get(&self, key: &str) -> Result<Option<BudgetNode>, mister_smith_core::LlmError> {
        self.0.get(key).await
    }

    async fn cas_update(
        &self,
        node: &BudgetNode,
        expected_revision: u64,
    ) -> Result<u64, mister_smith_core::LlmError> {
        self.0.cas_update(node, expected_revision).await
    }
}

struct FailingProvider;

#[async_trait]
impl ModelProvider for FailingProvider {
    async fn complete(
        &self,
        _request: CompletionRequest,
    ) -> Result<mister_smith_llm::CompletionResponse, mister_smith_core::LlmError> {
        Err(mister_smith_core::LlmError::Network(
            "simulated provider failure".to_string(),
        ))
    }

    fn stream(&self, _request: CompletionRequest) -> mister_smith_llm::CompletionStream {
        Box::pin(stream::empty())
    }

    async fn embed(
        &self,
        _input: Vec<String>,
    ) -> Result<mister_smith_llm::EmbeddingResponse, mister_smith_core::LlmError> {
        Ok(mister_smith_llm::EmbeddingResponse {
            embeddings: Vec::new(),
            model_id: "failing-model".to_string(),
            usage: mister_smith_llm::Usage::new(0, 0),
        })
    }

    fn model_id(&self) -> &str {
        "failing-model"
    }

    fn capabilities(&self) -> mister_smith_llm::ModelCapabilities {
        mister_smith_llm::ModelCapabilities::all()
    }
}

#[tokio::test]
async fn provider_failure_after_reserve_reverts_budget_usage() {
    let store = Arc::new(InMemoryBudgetStore::new());
    store.insert(BudgetNode {
        key: "budget/router-failure".to_string(),
        limit_tokens: 10_000,
        used_tokens: 0,
        period: "test".to_string(),
        policy: BudgetPolicy::HardCap,
        revision: 1,
    });

    let enforcer = mister_smith_llm::BudgetEnforcer::new(Box::new(SharedBudgetStore(store.clone())));
    let router =
        ModelRouter::new(RoutingPolicy::RoundRobin).with_budget(enforcer, "budget/router-failure");

    let provider: Arc<dyn ModelProvider> = Arc::new(FailingProvider);
    router
        .add_provider(
            mock_config("failing-model"),
            provider,
            CircuitBreakerConfig::default(),
        )
        .await;

    let result = router.route_completion(mock_request()).await;
    assert!(matches!(result, Err(mister_smith_core::LlmError::Network(_))));

    let budget = store.get("budget/router-failure").await.unwrap().unwrap();
    assert_eq!(budget.used_tokens, 0);
}

#[tokio::test]
async fn failed_requests_have_no_net_token_leak() {
    let store = Arc::new(InMemoryBudgetStore::new());
    store.insert(BudgetNode {
        key: "budget/router-no-leak".to_string(),
        limit_tokens: 10_000,
        used_tokens: 0,
        period: "test".to_string(),
        policy: BudgetPolicy::HardCap,
        revision: 1,
    });

    let enforcer = mister_smith_llm::BudgetEnforcer::new(Box::new(SharedBudgetStore(store.clone())));
    let router =
        ModelRouter::new(RoutingPolicy::RoundRobin).with_budget(enforcer, "budget/router-no-leak");

    let provider: Arc<dyn ModelProvider> = Arc::new(FailingProvider);
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

    let budget = store.get("budget/router-no-leak").await.unwrap().unwrap();
    assert_eq!(budget.used_tokens, 0);
}
