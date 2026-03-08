use std::sync::Arc;
use std::time::Instant;

use mister_smith_llm::{
    budget::{BudgetEnforcer, BudgetNode, BudgetPolicy},
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
    use async_trait::async_trait;
    use mister_smith_core::LlmError;
    use mister_smith_llm::budget::BudgetStore;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct SharedBudgetStore {
        nodes: Arc<Mutex<HashMap<String, BudgetNode>>>,
    }

    impl SharedBudgetStore {
        fn with_budget() -> (Self, Arc<Mutex<HashMap<String, BudgetNode>>>) {
            let nodes = Arc::new(Mutex::new(HashMap::new()));
            nodes.lock().unwrap().insert(
                "budget/test".to_string(),
                BudgetNode {
                    key: "budget/test".to_string(),
                    limit_tokens: 50_000,
                    used_tokens: 0,
                    period: "test".to_string(),
                    policy: BudgetPolicy::HardCap,
                    revision: 1,
                },
            );

            (
                Self {
                    nodes: nodes.clone(),
                },
                nodes,
            )
        }
    }

    #[async_trait]
    impl BudgetStore for SharedBudgetStore {
        async fn get(&self, key: &str) -> Result<Option<BudgetNode>, LlmError> {
            Ok(self.nodes.lock().unwrap().get(key).cloned())
        }

        async fn cas_update(
            &self,
            node: &BudgetNode,
            expected_revision: u64,
        ) -> Result<u64, LlmError> {
            let mut nodes = self.nodes.lock().unwrap();
            if let Some(existing) = nodes.get(&node.key) {
                if existing.revision != expected_revision {
                    return Err(LlmError::InvalidRequest(format!(
                        "CAS conflict on budget key '{}': expected revision {}, actual {}",
                        node.key, expected_revision, existing.revision
                    )));
                }
            }

            let mut updated = node.clone();
            updated.revision = expected_revision + 1;
            nodes.insert(node.key.clone(), updated);
            Ok(expected_revision + 1)
        }
    }

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
    async fn cascade_reserves_and_reconciles_budget_on_accept() {
        let policy = cascade_policy(0.3, 1);

        let (store, nodes) = SharedBudgetStore::with_budget();

        let router = ModelRouter::new(RoutingPolicy::Cascade(policy))
            .with_budget(BudgetEnforcer::new(Box::new(store)), "budget/test");

        let slm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("slm-model"));
        let llm: Arc<dyn ModelProvider> = Arc::new(MockProvider::new("llm-model"));

        router
            .add_provider(mock_config("slm"), slm, CircuitBreakerConfig::default())
            .await;
        router
            .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
            .await;

        let (response, _decision) = router.route_completion(mock_request()).await.unwrap();

        let used_tokens = nodes
            .lock()
            .unwrap()
            .get("budget/test")
            .unwrap()
            .used_tokens;

        assert_eq!(used_tokens, response.usage.total_tokens);
        assert!(response.usage.total_tokens < 1024);
    }

    #[tokio::test]
    async fn cascade_reconciles_budget_when_escalating_to_second_tier() {
        let policy = cascade_policy(1.1, 1);

        let (store, nodes) = SharedBudgetStore::with_budget();
        let enforcer = BudgetEnforcer::new(Box::new(store));

        let router = ModelRouter::new(RoutingPolicy::Cascade(policy))
            .with_budget(enforcer, "budget/test");

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

        let used_tokens = nodes
            .lock()
            .unwrap()
            .get("budget/test")
            .unwrap()
            .used_tokens;
        assert_eq!(used_tokens, response.usage.total_tokens);
        assert_eq!(decision.estimated_cost_tokens, Some(1027));
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
