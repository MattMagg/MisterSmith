use std::sync::Arc;

use async_trait::async_trait;
use mister_smith_core::LlmError;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::budget::{BudgetEnforcer, BudgetReservation};
use crate::config::ProviderConfig;
use crate::health::{CircuitBreaker, CircuitBreakerConfig, HealthStatus};
use crate::model_event::ModelEvent;
use crate::provider::{CompletionStream, ModelProvider};
use crate::types::{
    ChatMessage, CompletionRequest, CompletionResponse, EmbeddingResponse, ModelCapabilities,
    RoutingHint,
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

/// Cascade attempt binding a declared tier to a registered provider.
#[derive(Debug, Clone)]
struct CascadeAttempt {
    tier_order: usize,
    tier_label: String,
    provider_index: usize,
}

/// Budget reservation context for consistent reconciliation.
struct BudgetReservationContext<'a> {
    enforcer: &'a BudgetEnforcer,
    reservation: BudgetReservation,
}

impl<'a> BudgetReservationContext<'a> {
    async fn reconcile(self, actual_tokens: u64) -> Result<(), LlmError> {
        self.enforcer
            .reconcile(&self.reservation, actual_tokens)
            .await
    }
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

    /// Set the model event sink for routing observability.
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
    /// Filters by hint capabilities and cost estimate when provided.
    async fn select_provider(
        &self,
        hint: Option<&RoutingHint>,
        estimated_tokens: u64,
    ) -> Result<usize, LlmError> {
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

        // Filter by hint capabilities and cost
        let filtered_indices: Vec<usize> = if let Some(hint) = hint {
            healthy_indices
                .into_iter()
                .filter(|&i| {
                    Self::provider_matches_hint(
                        providers[i].provider.as_ref(),
                        Some(hint),
                        estimated_tokens,
                    )
                })
                .collect()
        } else {
            healthy_indices
        };

        if filtered_indices.is_empty() {
            return Err(LlmError::NoHealthyProvider(
                "No providers match routing hint requirements".to_string(),
            ));
        }

        match &self.routing_policy {
            RoutingPolicy::RoundRobin => {
                let idx = self
                    .round_robin_counter
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(filtered_indices[idx % filtered_indices.len()])
            }
            RoutingPolicy::CostOptimized => Ok(filtered_indices[0]),
            RoutingPolicy::CapabilityMatched => Ok(filtered_indices[0]),
            RoutingPolicy::Cascade(_) => Ok(filtered_indices[0]),
        }
    }

    fn provider_supports_capability(provider: &dyn ModelProvider, capability: &str) -> bool {
        let caps = provider.capabilities();
        match capability {
            "completion" => caps.completion,
            "streaming" => caps.streaming,
            "embeddings" => caps.embeddings,
            "tool_calling" => caps.tool_calling,
            _ => false,
        }
    }

    fn provider_matches_hint(
        provider: &dyn ModelProvider,
        hint: Option<&RoutingHint>,
        estimated_tokens: u64,
    ) -> bool {
        let Some(hint) = hint else { return true };
        for cap in &hint.required_capabilities {
            if !Self::provider_supports_capability(provider, cap) {
                return false;
            }
        }
        if let Some(max_cost) = hint.max_cost_tokens {
            if estimated_tokens > max_cost {
                return false;
            }
        }
        true
    }

    /// Clone the request with routing_hint stripped before dispatch to a provider.
    fn provider_request(request: &CompletionRequest) -> CompletionRequest {
        let mut clean = request.clone();
        clean.routing_hint = None;
        clean
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

    /// Reserve budget tokens. Returns None if no enforcer is configured.
    async fn reserve_budget(
        &self,
        estimated_tokens: u64,
    ) -> Result<Option<BudgetReservationContext<'_>>, LlmError> {
        if let (Some(enforcer), Some(root)) = (&self.budget_enforcer, &self.budget_root) {
            let reservation = enforcer.reserve(root, estimated_tokens).await?;
            Ok(Some(BudgetReservationContext {
                enforcer,
                reservation,
            }))
        } else {
            Ok(None)
        }
    }

    /// Build a tier-to-provider mapping from the cascade policy.
    fn build_cascade_attempt_plan(
        providers: &[ProviderEntry],
        policy: &CascadePolicy,
    ) -> Result<Vec<CascadeAttempt>, LlmError> {
        let mut plan = Vec::with_capacity(policy.tiers.len());
        for (order, tier) in policy.tiers.iter().enumerate() {
            let provider_index = providers
                .iter()
                .position(|entry| {
                    entry.config.model_id == tier.provider_config.model_id
                        && entry.config.provider_kind == tier.provider_config.provider_kind
                })
                .ok_or_else(|| {
                    LlmError::InvalidRequest(format!(
                        "Cascade tier '{}' has no matching registered provider \
                         (model_id: '{}', provider_kind: {:?})",
                        tier.label,
                        tier.provider_config.model_id,
                        tier.provider_config.provider_kind
                    ))
                })?;
            plan.push(CascadeAttempt {
                tier_order: order,
                tier_label: tier.label.clone(),
                provider_index,
            });
        }
        Ok(plan)
    }

    /// Route a completion request through the data plane.
    pub async fn route_completion(
        &self,
        request: CompletionRequest,
    ) -> Result<(CompletionResponse, RoutingDecision), LlmError> {
        let estimated = Self::estimate_tokens(&request);
        let hint = request.routing_hint.as_ref();

        // Budget reservation — covers both cascade and non-cascade paths
        let reservation = self.reserve_budget(estimated).await?;

        // Delegate to cascade routing when policy is Cascade
        if let RoutingPolicy::Cascade(ref policy) = self.routing_policy {
            let policy = policy.clone();
            let result = self.route_cascade(request, &policy, estimated).await;

            // Reconcile budget after cascade
            if let Some(ctx) = reservation {
                match &result {
                    Ok((response, _)) => ctx.reconcile(response.usage.total_tokens).await?,
                    Err(_) => ctx.reconcile(0).await?,
                }
            }
            return result;
        }

        // Non-cascade path
        let idx = match self.select_provider(hint, estimated).await {
            Ok(idx) => idx,
            Err(err) => {
                // Reconcile budget on selection failure
                if let Some(ctx) = reservation {
                    ctx.reconcile(0).await?;
                }
                return Err(err);
            }
        };

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
        let result = entry
            .provider
            .complete(Self::provider_request(&request))
            .await;
        let latency_ms = start.elapsed().as_millis() as u64;

        drop(providers);

        match result {
            Ok(response) => {
                // Record success
                let mut providers = self.providers.write().await;
                providers[idx].circuit_breaker.record_success(latency_ms);
                drop(providers);

                // Reconcile budget — propagate errors
                if let Some(ctx) = reservation {
                    ctx.reconcile(response.usage.total_tokens).await?;
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
                drop(providers);

                // Reconcile budget on provider failure
                if let Some(ctx) = reservation {
                    ctx.reconcile(0).await?;
                }

                Err(err)
            }
        }
    }

    /// Route a completion via cascade (SLM-default / LLM-fallback).
    ///
    /// Attempts providers in declared tier order (CascadePolicy::tiers), which is
    /// authoritative for cheapest-first escalation. Budget is managed by the caller
    /// (`route_completion`).
    async fn route_cascade(
        &self,
        request: CompletionRequest,
        policy: &CascadePolicy,
        estimated_tokens: u64,
    ) -> Result<(CompletionResponse, RoutingDecision), LlmError> {
        let hint = request.routing_hint.as_ref();

        let providers = self.providers.read().await;
        let attempt_plan = Self::build_cascade_attempt_plan(&providers, policy)?;
        drop(providers);

        // Build attempt indices, optionally reordering by preferred_tier hint
        let mut attempt_indices: Vec<usize> = (0..attempt_plan.len()).collect();
        if let Some(hint) = hint {
            if let Some(preferred) = &hint.preferred_tier {
                if let Some(pos) = attempt_plan.iter().position(|a| a.tier_label == *preferred) {
                    attempt_indices.retain(|&i| i != pos);
                    attempt_indices.insert(0, pos);
                }
            }
        }

        let max_attempts = attempt_indices
            .len()
            .min(policy.max_escalations as usize + 1);

        for (attempt_idx, plan_idx) in attempt_indices.into_iter().take(max_attempts).enumerate() {
            let attempt = &attempt_plan[plan_idx];
            let provider_index = attempt.provider_index;

            // Read provider info under a short-lived write lock (for half-open transition)
            let (provider, config_model_id, model_id, allowed) = {
                let mut providers = self.providers.write().await;
                if provider_index >= providers.len() {
                    break;
                }
                let entry = &mut providers[provider_index];
                entry.circuit_breaker.maybe_transition_to_half_open();
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

            // Check hint filtering
            if !Self::provider_matches_hint(provider.as_ref(), hint, estimated_tokens) {
                continue;
            }

            let tier_label = attempt.tier_label.clone();

            let start = std::time::Instant::now();
            let result = provider.complete(Self::provider_request(&request)).await;
            let latency_ms = start.elapsed().as_millis() as u64;

            match result {
                Ok(response) => {
                    {
                        let mut providers = self.providers.write().await;
                        if provider_index < providers.len() {
                            providers[provider_index]
                                .circuit_breaker
                                .record_success(latency_ms);
                        }
                    }

                    let confidence = ConfidenceSignal::from_response(&response);
                    let is_last_tier = attempt_idx + 1 >= max_attempts;

                    let decision = RoutingDecision {
                        provider_id: config_model_id,
                        model_id: model_id.clone(),
                        tier_label: Some(tier_label.clone()),
                        reason: if confidence.score >= policy.escalation_threshold || is_last_tier {
                            format!(
                                "Cascade accepted at tier '{}' (order {}, confidence: {:.2})",
                                tier_label, attempt.tier_order, confidence.score
                            )
                        } else {
                            format!(
                                "Cascade escalating from tier '{}' (order {}, confidence: {:.2} < threshold: {:.2})",
                                tier_label, attempt.tier_order, confidence.score, policy.escalation_threshold
                            )
                        },
                        estimated_cost_tokens: Some(estimated_tokens),
                    };

                    let accepted = confidence.score >= policy.escalation_threshold || is_last_tier;
                    let event_reason = if accepted {
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
                    self.emit_routing_event(model_id, tier_label, event_reason);

                    if accepted {
                        return Ok((response, decision));
                    }
                    // Below threshold — escalate to next tier
                }
                Err(err) => {
                    let retry_after = match &err {
                        LlmError::RateLimited { retry_after_secs } => *retry_after_secs,
                        _ => None,
                    };
                    let mut providers = self.providers.write().await;
                    if provider_index < providers.len() {
                        providers[provider_index]
                            .circuit_breaker
                            .record_failure(retry_after);
                    }
                    drop(providers);

                    self.emit_routing_event(model_id, tier_label, format!("failed ({err})"));

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
        let idx = self.select_provider(None, 0).await?;
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
