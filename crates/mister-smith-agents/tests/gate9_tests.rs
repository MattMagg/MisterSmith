//! Gate 9 validation tests — end-to-end LLM provider integration.
//!
//! Exercises: Planner → MockProvider via ModelRouter → ToolBus → Orchestrator flow.
//! Run with: `cargo test -p mister-smith-agents --features llm --test gate9_tests`
#![cfg(feature = "llm")]

use std::sync::Arc;

use async_trait::async_trait;
use futures::stream;
use mister_smith_agents::roles::critic::{CriticAgent, CriticMessage, CriticState};
use mister_smith_agents::roles::executor::{ExecutorAgent, ExecutorMessage, ExecutorState};
use mister_smith_agents::roles::planner::{PlannerAgent, PlannerMessage, PlannerState};
use mister_smith_agents::tool_bus::ToolBus;
use mister_smith_core::{Actor, AgentId, LlmError};
use mister_smith_llm::{
    CircuitBreakerConfig, CompletionRequest, CompletionResponse, CompletionStream,
    EmbeddingResponse, MockProvider, ModelCapabilities, ModelProvider, ModelRouter, ProviderConfig,
    ProviderKind, RoutingPolicy, ToolCall,
};

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
            },
            &mut state,
        )
        .await
        .unwrap();

    assert!(result.is_object());
    assert_eq!(state.evaluations_completed, 1);
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
            },
            &mut state,
        )
        .await
        .unwrap_err();

    assert!(err.to_string().starts_with("executor error:"));
}
