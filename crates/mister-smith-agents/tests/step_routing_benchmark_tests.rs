#![cfg(feature = "llm")]

use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mister_smith_agents::orchestrator::StepRoutingControl;
use mister_smith_core::LlmError;
use mister_smith_core::ProofOutcomeClassification;
use mister_smith_events::StepRoutingDecisionSummary;
use mister_smith_llm::{
    CascadePolicy, CascadeTier, CircuitBreakerConfig, CompletionRequest, CompletionResponse,
    ContentBlock, EmbeddingResponse, ModelCapabilities, ModelProvider, ModelRouter, ProviderConfig,
    ProviderKind, RoutingPolicy, StopReason, Usage,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BenchmarkStrategy {
    AdaptiveCarryover,
    StatelessBaseline,
}

impl BenchmarkStrategy {
    const fn label(self) -> &'static str {
        match self {
            Self::AdaptiveCarryover => "adaptive_carryover",
            Self::StatelessBaseline => "stateless_baseline",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkloadClass {
    ConfidenceEscalation,
    ProviderFailureFallback,
}

impl WorkloadClass {
    const fn label(self) -> &'static str {
        match self {
            Self::ConfidenceEscalation => "confidence_escalation_bundle",
            Self::ProviderFailureFallback => "provider_failure_bundle",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct HarnessResult {
    workload_class: &'static str,
    strategy: &'static str,
    provider_calls: usize,
    triggered_checkpoints: usize,
    action_changes: usize,
    step_history: Vec<StepRoutingDecisionSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvaluationHarnessRun {
    workload_class: &'static str,
    completed: bool,
    graph_formed: bool,
    branch_count: Option<usize>,
    result_preview: Option<&'static str>,
    evidence_note_path: &'static str,
}

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
        let model_id = self.model_id.clone();
        Box::pin(futures::stream::once(async {
            Err(LlmError::UnsupportedCapability {
                capability: "streaming".into(),
                model: model_id,
            })
        }))
    }

    async fn embed(&self, _input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        Err(LlmError::UnsupportedCapability {
            capability: "embeddings".into(),
            model: self.model_id.clone(),
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

#[derive(Debug)]
struct FailingProvider {
    model_id: String,
    calls: Arc<Mutex<u32>>,
}

impl FailingProvider {
    fn new(model_id: impl Into<String>, calls: Arc<Mutex<u32>>) -> Self {
        Self {
            model_id: model_id.into(),
            calls,
        }
    }
}

#[async_trait]
impl ModelProvider for FailingProvider {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse, LlmError> {
        *self.calls.lock().unwrap() += 1;
        Err(LlmError::Network("simulated provider failure".to_string()))
    }

    fn stream(&self, _request: CompletionRequest) -> mister_smith_llm::CompletionStream {
        Box::pin(futures::stream::once(async {
            Err(LlmError::Network("simulated provider failure".to_string()))
        }))
    }

    async fn embed(&self, _input: Vec<String>) -> Result<EmbeddingResponse, LlmError> {
        Err(LlmError::UnsupportedCapability {
            capability: "embeddings".into(),
            model: self.model_id.clone(),
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

fn mock_config(model_id: &str) -> ProviderConfig {
    ProviderConfig {
        provider_kind: ProviderKind::Mock,
        model_id: model_id.to_string(),
        ..Default::default()
    }
}

fn cascade_policy(threshold: f32) -> CascadePolicy {
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
        max_escalations: 1,
    }
}

fn low_confidence_response(model_id: &str) -> CompletionResponse {
    CompletionResponse {
        content: vec![ContentBlock::Text {
            text: "ok".to_string(),
        }],
        model_id: model_id.to_string(),
        usage: Usage::new(24, 8),
        stop_reason: StopReason::MaxTokens,
        tool_calls: vec![],
    }
}

fn high_confidence_response(model_id: &str) -> CompletionResponse {
    CompletionResponse {
        content: vec![ContentBlock::Text {
            text: "confident result with enough detail".to_string(),
        }],
        model_id: model_id.to_string(),
        usage: Usage::new(24, 8),
        stop_reason: StopReason::Completed,
        tool_calls: vec![],
    }
}

async fn build_router(
    workload: WorkloadClass,
    slm_calls: Arc<Mutex<u32>>,
    llm_calls: Arc<Mutex<u32>>,
) -> ModelRouter {
    let threshold = match workload {
        WorkloadClass::ConfidenceEscalation => 0.6,
        WorkloadClass::ProviderFailureFallback => 0.3,
    };
    let router = ModelRouter::new(RoutingPolicy::Cascade(cascade_policy(threshold)));

    let slm: Arc<dyn ModelProvider> = match workload {
        WorkloadClass::ConfidenceEscalation => Arc::new(StaticResponseProvider::new(
            "slm-model",
            low_confidence_response("slm-model"),
            slm_calls,
        )),
        WorkloadClass::ProviderFailureFallback => {
            Arc::new(FailingProvider::new("slm-model", slm_calls))
        }
    };
    let llm: Arc<dyn ModelProvider> = Arc::new(StaticResponseProvider::new(
        "llm-model",
        high_confidence_response("llm-model"),
        llm_calls,
    ));

    router
        .add_provider(mock_config("slm"), slm, CircuitBreakerConfig::default())
        .await;
    router
        .add_provider(mock_config("llm"), llm, CircuitBreakerConfig::default())
        .await;

    router
}

async fn run_bundle(workload: WorkloadClass, strategy: BenchmarkStrategy) -> HarnessResult {
    let slm_calls = Arc::new(Mutex::new(0));
    let llm_calls = Arc::new(Mutex::new(0));
    let router = build_router(workload, slm_calls.clone(), llm_calls.clone()).await;
    let mut control = StepRoutingControl::default();
    let mut step_history = Vec::new();

    for step_index in 1..=2u32 {
        let step_id = match workload {
            WorkloadClass::ConfidenceEscalation => format!("planner.step.{step_index}"),
            WorkloadClass::ProviderFailureFallback => format!("critic.step.{step_index}"),
        };
        let step_kind = match workload {
            WorkloadClass::ConfidenceEscalation => "planner",
            WorkloadClass::ProviderFailureFallback => "critic",
        };
        let routing_hint = match strategy {
            BenchmarkStrategy::AdaptiveCarryover => {
                control.request_hint(step_id.clone(), Some(step_index), step_kind)
            }
            BenchmarkStrategy::StatelessBaseline => StepRoutingControl::default().request_hint(
                step_id.clone(),
                Some(step_index),
                step_kind,
            ),
        };
        let request = CompletionRequest {
            system: Some(format!("benchmark {step_id}")),
            messages: vec![mister_smith_llm::ChatMessage::User {
                content: serde_json::json!({
                    "workload_class": workload.label(),
                    "step_id": step_id,
                }),
            }],
            routing_hint: Some(routing_hint),
            ..CompletionRequest::default()
        };

        let (_response, decision) = router.route_completion(request).await.unwrap();
        match strategy {
            BenchmarkStrategy::AdaptiveCarryover => {
                control.apply_routing_decision(&decision);
                step_history.push(control.history.last().cloned().unwrap());
            }
            BenchmarkStrategy::StatelessBaseline => {
                let mut per_step_control = StepRoutingControl::default();
                per_step_control.apply_routing_decision(&decision);
                step_history.push(per_step_control.history.last().cloned().unwrap());
            }
        }
    }

    let provider_calls =
        usize::try_from(*slm_calls.lock().unwrap() + *llm_calls.lock().unwrap()).unwrap();

    HarnessResult {
        workload_class: workload.label(),
        strategy: strategy.label(),
        provider_calls,
        triggered_checkpoints: step_history
            .iter()
            .map(|summary| summary.triggered_checkpoints.len())
            .sum(),
        action_changes: step_history
            .iter()
            .filter(|summary| summary.action_changed)
            .count(),
        step_history,
    }
}

fn classify_evaluation_run(run: &EvaluationHarnessRun) -> ProofOutcomeClassification {
    if !run.completed || !run.graph_formed {
        ProofOutcomeClassification::FailedBeforeGraph
    } else if run.branch_count.unwrap_or(0) <= 1 {
        ProofOutcomeClassification::CollapsedToSequential
    } else {
        ProofOutcomeClassification::GraphFormedAndCompleted
    }
}

#[tokio::test]
async fn step_routing_benchmark_harness_records_confidence_bundle_improvement() {
    let baseline = run_bundle(
        WorkloadClass::ConfidenceEscalation,
        BenchmarkStrategy::StatelessBaseline,
    )
    .await;
    let adaptive = run_bundle(
        WorkloadClass::ConfidenceEscalation,
        BenchmarkStrategy::AdaptiveCarryover,
    )
    .await;

    assert_eq!(baseline.provider_calls, 4);
    assert_eq!(adaptive.provider_calls, 3);
    assert_eq!(baseline.triggered_checkpoints, 2);
    assert_eq!(adaptive.triggered_checkpoints, 1);
    assert_eq!(baseline.action_changes, 0);
    assert_eq!(adaptive.action_changes, 1);
    assert_eq!(adaptive.step_history[0].action, "escalate");
    assert_eq!(adaptive.step_history[1].action, "continue");
    assert!(adaptive.step_history[1]
        .change_rationale
        .iter()
        .any(|line| line.contains("action changed from escalate to continue")));
}

#[tokio::test]
async fn step_routing_benchmark_harness_records_provider_failure_bundle_match() {
    let baseline = run_bundle(
        WorkloadClass::ProviderFailureFallback,
        BenchmarkStrategy::StatelessBaseline,
    )
    .await;
    let adaptive = run_bundle(
        WorkloadClass::ProviderFailureFallback,
        BenchmarkStrategy::AdaptiveCarryover,
    )
    .await;

    assert_eq!(baseline.provider_calls, 3);
    assert_eq!(adaptive.provider_calls, 3);
    assert_eq!(baseline.triggered_checkpoints, 1);
    assert_eq!(adaptive.triggered_checkpoints, 1);
    assert_eq!(baseline.action_changes, 0);
    assert_eq!(adaptive.action_changes, 1);
    assert_eq!(adaptive.step_history[0].action, "fallback");
    assert_eq!(adaptive.step_history[1].action, "continue");
    assert!(adaptive.step_history[1]
        .change_rationale
        .iter()
        .any(|line| line.contains("action changed from fallback to continue")));
}

#[tokio::test]
async fn step_routing_benchmark_harness_is_repeatable() {
    let first = run_bundle(
        WorkloadClass::ConfidenceEscalation,
        BenchmarkStrategy::AdaptiveCarryover,
    )
    .await;
    let second = run_bundle(
        WorkloadClass::ConfidenceEscalation,
        BenchmarkStrategy::AdaptiveCarryover,
    )
    .await;

    assert_eq!(first, second);
}

#[test]
fn proof_matrix_harness_replays_success_collapse_and_failure_visible_cases() {
    let expected_labels = ProofOutcomeClassification::ALL.map(ProofOutcomeClassification::as_str);
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert_eq!(
        expected_labels,
        [
            "graph_formed_and_completed",
            "collapsed_to_sequential",
            "failed_before_graph",
        ]
    );

    let cases = [
        (
            EvaluationHarnessRun {
                workload_class: "short_multi_agent_result_evaluation",
                completed: true,
                graph_formed: true,
                branch_count: Some(3),
                result_preview: Some("workflow completed with a real multi-branch graph"),
                evidence_note_path: "docs/plans/2026-03-19-short-multi-agent-result-evaluation.md",
            },
            ProofOutcomeClassification::GraphFormedAndCompleted,
        ),
        (
            EvaluationHarnessRun {
                workload_class: "framework_stress_trimmed_benchmark",
                completed: true,
                graph_formed: true,
                branch_count: Some(1),
                result_preview: Some("completed with a sequential execution path"),
                evidence_note_path: "docs/plans/2026-03-19-framework-comparison-stress-test.md",
            },
            ProofOutcomeClassification::CollapsedToSequential,
        ),
        (
            EvaluationHarnessRun {
                workload_class: "framework_stress_heavy_benchmark",
                completed: false,
                graph_formed: false,
                branch_count: None,
                result_preview: Some("workflow failed before graph formation"),
                evidence_note_path: "docs/plans/2026-03-19-framework-comparison-stress-test.md",
            },
            ProofOutcomeClassification::FailedBeforeGraph,
        ),
    ];

    for (run, expected) in cases {
        assert!(
            repo_root.join(run.evidence_note_path).is_file(),
            "missing evidence note for {}",
            run.workload_class
        );
        assert_eq!(
            classify_evaluation_run(&run),
            expected,
            "unexpected proof outcome for {}",
            run.workload_class
        );
        assert!(
            run.result_preview.is_some(),
            "expected bounded preview for {}",
            run.workload_class
        );
    }
}
