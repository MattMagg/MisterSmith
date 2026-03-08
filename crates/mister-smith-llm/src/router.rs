use std::sync::Arc;

use async_trait::async_trait;
use mister_smith_core::LlmError;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::budget::BudgetEnforcer;
use crate::config::ProviderConfig;
use crate::health::{CircuitBreaker, CircuitBreakerConfig, HealthStatus};
use crate::model_event::ModelEvent;
use crate::provider::{CompletionStream, ModelProvider};
use crate::types::{
    ChatMessage, CompletionRequest, CompletionResponse, EmbeddingResponse, ModelCapabilities,
};

/// Routing policy for provider selection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum RoutingPolicy {
    /// Rotate across healthy providers.
    #[default]
    RoundRobin,
    /// Route to cheapest healthy provider meeting capability requirements.
    CostOptimized,
    /// Route based on model capability matching.
    CapabilityMatched,
    /// Multi-tier escalation (SLM-default, LLM-fallback).
    Cascade(CascadePolicy),
}


/// Caller-provided routing preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutingHint {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_tier: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_cost_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_capabilities: Vec<String>,
}

/// Multi-tier routing configuration for SLM-default / LLM-fallback.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadePolicy {
    pub tiers: Vec<CascadeTier>,
    pub escalation_threshold: f32,
    #[serde(default = "default_max_escalations")]
    pub max_escalations: u32,
}

fn default_max_escalations() -> u32 {
    1
}

/// A single tier within a CascadePolicy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeTier {
    pub provider_config: ProviderConfig,
    pub label: String,
}

/// Routing confidence signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceSignal {
    pub score: f32,
    pub source: String,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl ConfidenceSignal {
    /// Heuristic confidence based on response properties.
    pub fn from_response(response: &CompletionResponse) -> Self {
        let mut score: f32 = 1.0;

        // Penalize if stopped due to max tokens (likely truncated)
        if response.stop_reason == crate::types::StopReason::MaxTokens {
            score -= 0.3;
        }

        // Penalize very short responses (may indicate failure)
        let total_text_len: usize = response
            .content
            .iter()
            .map(|block| match block {
                crate::types::ContentBlock::Text { text } => text.len(),
                _ => 0,
            })
            .sum();
        if total_text_len < 10 {
            score -= 0.2;
        }

        // Penalize content filter
        if response.stop_reason == crate::types::StopReason::ContentFilter {
            score -= 0.5;
        }

        Self {
            score: score.clamp(0.0, 1.0),
            source: "heuristic".to_string(),
            metadata: serde_json::json!({}),
        }
    }
}

/// A routing decision record for observability.
#[derive(Debug, Clone)]
pub struct RoutingDecision {
    pub provider_id: String,
    pub model_id: String,
    pub tier_label: Option<String>,
    pub reason: String,
    pub estimated_cost_tokens: Option<u64>,
}

/// Sink interface for router-emitted model events.
pub trait ModelEventSink: Send + Sync {
    fn publish(&self, event: ModelEvent);
}

/// Registered provider entry with its circuit breaker.
struct ProviderEntry {
    config: ProviderConfig,
    provider: Arc<dyn ModelProvider>,
    circuit_breaker: CircuitBreaker,
}

/// Data-plane router that selects a provider per-request based on policy, health, and budget.
pub struct ModelRouter {
    providers: RwLock<Vec<ProviderEntry>>,
    routing_policy: RoutingPolicy,
    budget_enforcer: Option<BudgetEnforcer>,
    budget_root: Option<String>,
    round_robin_counter: std::sync::atomic::AtomicUsize,
    model_event_sink: Option<Arc<dyn ModelEventSink>>,
}

impl ModelRouter {
    /// Create a new router with the given policy.
    pub fn new(routing_policy: RoutingPolicy) -> Self {
        Self {
            providers: RwLock::new(Vec::new()),
            routing_policy,
            budget_enforcer: None,
            budget_root: None,
            round_robin_counter: std::sync::atomic::AtomicUsize::new(0),
            model_event_sink: None,
        }
    }

    /// Set the budget enforcer.
    pub fn with_budget(mut self, enforcer: BudgetEnforcer, root_key: impl Into<String>) -> Self {
        self.budget_enforcer = Some(enforcer);
        self.budget_root = Some(root_key.into());
        self
    }

    /// Set an optional sink to receive routing model events.
    pub fn with_model_event_sink(mut self, sink: Arc<dyn ModelEventSink>) -> Self {
        self.model_event_sink = Some(sink);
        self
    }

    fn emit_routing_event(&self, model_id: String, tier: String, reason: String) {
        if let Some(sink) = &self.model_event_sink {
            sink.publish(ModelEvent::RoutingDecision {
                model_id,
                tier,
                reason,
            });
        }
    }

    /// Register a provider.
    pub async fn add_provider(
        &self,
        config: ProviderConfig,
        provider: Arc<dyn ModelProvider>,
        cb_config: CircuitBreakerConfig,
    ) {
        let mut providers = self.providers.write().await;
        providers.push(ProviderEntry {
            config,
            provider,
            circuit_breaker: CircuitBreaker::new(cb_config),
        });
    }

    /// Get health status for all providers.
    pub async fn health_table(&self) -> Vec<HealthStatus> {
        let providers = self.providers.read().await;
        providers
            .iter()
            .map(|entry| entry.circuit_breaker.health_status(&entry.config.model_id))
            .collect()
    }

    /// Select a healthy provider based on routing policy.
    async fn select_provider(&self, _hint: Option<&RoutingHint>) -> Result<usize, LlmError> {
        let mut providers = self.providers.write().await;

        // First, transition any expired Open circuits to HalfOpen
        for entry in providers.iter_mut() {
            entry.circuit_breaker.maybe_transition_to_half_open();
        }

        let healthy_indices: Vec<usize> = providers
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                entry.circuit_breaker.is_allowed() && !entry.circuit_breaker.is_rate_limited()
            })
            .map(|(i, _)| i)
            .collect();

        if healthy_indices.is_empty() {
            return Err(LlmError::NoHealthyProvider(
                "All providers are unhealthy or rate-limited".to_string(),
            ));
        }

        match &self.routing_policy {
            RoutingPolicy::RoundRobin => {
                let idx = self
                    .round_robin_counter
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(healthy_indices[idx % healthy_indices.len()])
            }
            RoutingPolicy::CostOptimized => {
                // For now, prefer providers earlier in the list (assumed cheaper)
                Ok(healthy_indices[0])
            }
            RoutingPolicy::CapabilityMatched => Ok(healthy_indices[0]),
            RoutingPolicy::Cascade(_) => {
                // Cascade routing is handled separately in route_cascade
                Ok(healthy_indices[0])
            }
        }
    }

    /// Estimate token cost for a request.
    fn estimate_tokens(request: &CompletionRequest) -> u64 {
        let input_estimate: u64 = request
            .messages
            .iter()
            .map(|msg| match msg {
                ChatMessage::System { content } => content.len() as u64 / 4,
                ChatMessage::User { content } | ChatMessage::Assistant { content } => {
                    content.to_string().len() as u64 / 4
                }
                ChatMessage::Tool { result } => {
                    result
                        .output
                        .as_ref()
                        .map(|v| v.to_string().len())
                        .unwrap_or(0) as u64
                        / 4
                }
            })
            .sum();

        let output_estimate = request.max_tokens.unwrap_or(1024) as u64;
        input_estimate + output_estimate
    }

    /// Route a completion request through the data plane.
    pub async fn route_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<(CompletionResponse, RoutingDecision), LlmError> {
        // Delegate to cascade routing when policy is Cascade
        if let RoutingPolicy::Cascade(ref policy) = self.routing_policy {
            let policy = policy.clone();
            return self.route_cascade(request, &policy).await;
        }

        let estimated = Self::estimate_tokens(&request);

        // Budget check
        let reservation =
            if let (Some(enforcer), Some(root)) = (&self.budget_enforcer, &self.budget_root) {
                Some(enforcer.reserve(root, estimated).await?)
            } else {
                None
            };

        let idx = self.select_provider(None).await?;
        let providers = self.providers.read().await;
        let entry = &providers[idx];

        let decision = RoutingDecision {
            provider_id: entry.config.model_id.clone(),
            model_id: entry.provider.model_id().to_string(),
            tier_label: None,
            reason: format!("Selected via {:?}", self.routing_policy),
            estimated_cost_tokens: Some(estimated),
        };

        let start = std::time::Instant::now();
        let result = entry.provider.complete(request).await;
        let latency_ms = start.elapsed().as_millis() as u64;

        drop(providers);

        match result {
            Ok(response) => {
                // Record success
                let mut providers = self.providers.write().await;
                providers[idx].circuit_breaker.record_success(latency_ms);
                drop(providers);

                // Reconcile budget
                if let Some(reservation) = &reservation {
                    if let Some(enforcer) = &self.budget_enforcer {
                        let _ = enforcer
                            .reconcile(reservation, response.usage.total_tokens)
                            .await;
                    }
                }

                Ok((response, decision))
            }
            Err(err) => {
                let retry_after = match &err {
                    LlmError::RateLimited { retry_after_secs } => *retry_after_secs,
                    _ => None,
                };
                let mut providers = self.providers.write().await;
                providers[idx].circuit_breaker.record_failure(retry_after);
                Err(err)
            }
        }
    }

    /// Route a completion via cascade (SLM-default / LLM-fallback).
    ///
    /// Attempts providers in registration order (cheapest first). Each response
    /// is scored with [`ConfidenceSignal`]; if the score meets the escalation
    /// threshold or all tiers are exhausted, the response is returned.
    pub async fn route_cascade(
        &self,
        request: CompletionRequest,
        policy: &CascadePolicy,
    ) -> Result<(CompletionResponse, RoutingDecision), LlmError> {
        let provider_count = self.providers.read().await.len();
        let max_attempts = provider_count.min(policy.max_escalations as usize + 1);

        for tier_idx in 0..max_attempts {
            // Read provider info under a short-lived lock
            let (provider, config_model_id, model_id, allowed) = {
                let providers = self.providers.read().await;
                if tier_idx >= providers.len() {
                    break;
                }
                let entry = &providers[tier_idx];
                let allowed =
                    entry.circuit_breaker.is_allowed() && !entry.circuit_breaker.is_rate_limited();
                (
                    entry.provider.clone(),
                    entry.config.model_id.clone(),
                    entry.provider.model_id().to_string(),
                    allowed,
                )
            };

            if !allowed {
                continue;
            }

            let tier_label = policy
                .tiers
                .get(tier_idx)
                .map(|t| t.label.clone())
                .unwrap_or_else(|| format!("tier-{tier_idx}"));

            let start = std::time::Instant::now();
            let result = provider.complete(request.clone()).await;
            let latency_ms = start.elapsed().as_millis() as u64;

            match result {
                Ok(response) => {
                    {
                        let mut providers = self.providers.write().await;
                        if tier_idx < providers.len() {
                            providers[tier_idx].circuit_breaker.record_success(latency_ms);
                        }
                    }

                    let confidence = ConfidenceSignal::from_response(&response);
                    let is_last_tier = tier_idx + 1 >= max_attempts;

                    let decision = RoutingDecision {
                        provider_id: config_model_id,
                        model_id: model_id.clone(),
                        tier_label: Some(tier_label.clone()),
                        reason: if confidence.score >= policy.escalation_threshold || is_last_tier {
                            format!(
                                "Cascade accepted at tier '{}' (confidence: {:.2})",
                                tier_label, confidence.score
                            )
                        } else {
                            format!(
                                "Cascade escalating from tier '{}' (confidence: {:.2} < threshold: {:.2})",
                                tier_label, confidence.score, policy.escalation_threshold
                            )
                        },
                        estimated_cost_tokens: None,
                    };

                    let accepted = confidence.score >= policy.escalation_threshold || is_last_tier;
                    let reason = if accepted {
                        format!(
                            "accepted (confidence: {:.2}, threshold: {:.2}, last_tier: {})",
                            confidence.score, policy.escalation_threshold, is_last_tier
                        )
                    } else {
                        format!(
                            "escalated (confidence: {:.2} < threshold: {:.2})",
                            confidence.score, policy.escalation_threshold
                        )
                    };
                    self.emit_routing_event(model_id.clone(), tier_label.clone(), reason);

                    if accepted {
                        return Ok((response, decision));
                    }
                    // Below threshold — escalate to next tier
                }
                Err(err) => {
                    self.emit_routing_event(
                        model_id,
                        tier_label,
                        format!("failed ({err})"),
                    );
                    let retry_after = match &err {
                        LlmError::RateLimited { retry_after_secs } => *retry_after_secs,
                        _ => None,
                    };
                    let mut providers = self.providers.write().await;
                    if tier_idx < providers.len() {
                        providers[tier_idx].circuit_breaker.record_failure(retry_after);
                    }
                    // Continue to next tier on error
                }
            }
        }

        Err(LlmError::NoHealthyProvider(
            "Cascade exhausted all tiers".to_string(),
        ))
    }

    /// Get the number of registered providers.
    pub async fn provider_count(&self) -> usize {
        self.providers.read().await.len()
    }
}

/// ModelRouter also implements ModelProvider for seamless integration.
#[async_trait]
impl ModelProvider for ModelRouter {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        let (response, _decision) = self.route_completion(request).await?;
        Ok(response)
    }

    fn stream(&self, request: CompletionRequest) -> CompletionStream {
        // For streaming, select provider synchronously and delegate
        // This is a simplified version - full async selection would need a different approach
        let providers = self.providers.try_read();
        match providers {
            Ok(providers) => {
                let healthy: Vec<usize> = providers
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.circuit_breaker.is_allowed())
                    .map(|(i, _)| i)
                    .collect();

                if let Some(&idx) = healthy.first() {
                    providers[idx].provider.stream(request)
                } else {
                    Box::pin(futures::stream::once(async {
                        Err(LlmError::NoHealthyProvider(
                            "No healthy providers for streaming".to_string(),
                        ))
                    }))
                }
            }
            Err(_) => Box::pin(futures::stream::once(async {
                Err(LlmError::Network(
                    "Router lock contention during stream setup".to_string(),
                ))
            })),
        }
    }

    async fn embed(&self, input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        let idx = self.select_provider(None).await?;
        let providers = self.providers.read().await;
        providers[idx].provider.embed(input).await
    }

    fn model_id(&self) -> &str {
        "model-router"
    }

    fn capabilities(&self) -> ModelCapabilities {
        ModelCapabilities::all()
    }
}
