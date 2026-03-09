//! Tool-calling bridge tests (requires llm feature).
//!
//! Run with: cargo test -p mister-smith-agents --features llm

#![cfg(feature = "llm")]

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use mister_smith_agents::tool_bus::{ToolBus, ToolPrincipal};
use mister_smith_agents::AgentSystemError;
use mister_smith_core::{AgentId, Tool, ToolCapabilities, ToolError, ToolId, ToolSchema};
use mister_smith_llm::ToolCall as LlmToolCall;
use mister_smith_security::config::RbacConfig;
use mister_smith_security::jwt::AgentClaims;
use mister_smith_security::rbac::PolicyEngine;

#[derive(Clone)]
struct EchoTool {
    id: ToolId,
}

#[async_trait]
impl Tool for EchoTool {
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        Ok(serde_json::json!({ "echo": params }))
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

#[derive(Clone)]
struct SlowTool {
    id: ToolId,
    delay: Duration,
}

#[async_trait]
impl Tool for SlowTool {
    async fn execute(&self, _params: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        tokio::time::sleep(self.delay).await;
        Ok(serde_json::json!({ "status": "slow-ok" }))
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

fn claims(agent_id: AgentId, permissions: &[&str]) -> AgentClaims {
    let now = chrono::Utc::now().timestamp() as u64;
    AgentClaims {
        sub: agent_id.to_string(),
        exp: now + 3600,
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
        agent_id: agent_id.to_string(),
        agent_type: "worker".to_string(),
        permissions: permissions
            .iter()
            .map(|permission| permission.to_string())
            .collect(),
        token_use: "access".to_string(),
        ..Default::default()
    }
}

#[test]
fn to_tool_definitions_exports_registered_tools() {
    let bus = ToolBus::new();
    let agent_id = AgentId::new();

    bus.register(
        "analyzer",
        "data",
        agent_id,
        "Analyzes data",
        serde_json::json!({"type": "object", "properties": {"input": {"type": "string"}}}),
        serde_json::json!({}),
    );
    bus.register(
        "formatter",
        "text",
        agent_id,
        "Formats text",
        serde_json::json!({"type": "object"}),
        serde_json::json!({}),
    );

    let defs = bus.to_tool_definitions();
    assert_eq!(defs.len(), 2);

    let analyzer = defs.iter().find(|d| d.name == "data.analyzer").unwrap();
    assert_eq!(analyzer.description, "Analyzes data");
    assert!(analyzer.input_schema.is_object());

    let formatter = defs.iter().find(|d| d.name == "text.formatter").unwrap();
    assert_eq!(formatter.description, "Formats text");
}

#[test]
fn to_tool_definitions_empty_when_no_tools() {
    let bus = ToolBus::new();
    let defs = bus.to_tool_definitions();
    assert!(defs.is_empty());
}

#[test]
fn to_tool_definitions_preserves_input_schema() {
    let bus = ToolBus::new();
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "query": { "type": "string" },
            "limit": { "type": "integer" }
        },
        "required": ["query"]
    });

    bus.register(
        "search",
        "web",
        AgentId::new(),
        "Web search",
        schema.clone(),
        serde_json::json!({}),
    );

    let defs = bus.to_tool_definitions();
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].input_schema, schema);
}

#[tokio::test]
async fn execute_tool_call_invalid_format() {
    let bus = ToolBus::new();
    let call = LlmToolCall {
        call_id: "call-1".into(),
        name: "no-namespace".into(),
        input: serde_json::json!({}),
    };

    let result = bus.execute_tool_call(None, &call).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("namespace.name"),
        "error should mention required format: {err}"
    );
}

#[tokio::test]
async fn execute_tool_call_tool_not_found_returns_typed_error() {
    let bus = ToolBus::new();
    let call = LlmToolCall {
        call_id: "call-1".into(),
        name: "ns.nonexistent".into(),
        input: serde_json::json!({}),
    };

    let err = bus.execute_tool_call(None, &call).await.unwrap_err();
    assert!(matches!(
        err,
        AgentSystemError::Tool(ToolError::NotFound(_))
    ));
}

#[tokio::test]
async fn execute_tool_call_permission_denied_returns_typed_error() {
    let bus = ToolBus::with_security(
        Some(Arc::new(PolicyEngine::new(&RbacConfig::default()))),
        None,
    );
    let agent_id = AgentId::new();
    let principal = ToolPrincipal::new(agent_id, claims(agent_id, &[]));

    bus.register_native_tool(
        "echo",
        "data",
        agent_id,
        "Echoes payload",
        serde_json::json!({"type": "object"}),
        serde_json::json!({"type": "object"}),
        Arc::new(EchoTool { id: ToolId::new() }),
    );

    let call = LlmToolCall {
        call_id: "call-denied".into(),
        name: "data.echo".into(),
        input: serde_json::json!({"v": 1}),
    };

    let err = bus
        .execute_tool_call(Some(&principal), &call)
        .await
        .unwrap_err();
    assert!(matches!(err, AgentSystemError::PermissionDenied(_)));
}

#[tokio::test(start_paused = true)]
async fn execute_tool_call_timeout_returns_typed_error() {
    let bus = ToolBus::new();
    let agent_id = AgentId::new();

    bus.register_native_tool(
        "slow",
        "data",
        agent_id,
        "Sleeps before returning",
        serde_json::json!({"type": "object"}),
        serde_json::json!({"type": "object"}),
        Arc::new(SlowTool {
            id: ToolId::new(),
            delay: Duration::from_secs(3600),
        }),
    );

    let call = LlmToolCall {
        call_id: "call-timeout".into(),
        name: "data.slow".into(),
        input: serde_json::json!({}),
    };

    let pending = tokio::spawn({
        let bus = bus;
        async move { bus.execute_tool_call(None, &call).await }
    });

    tokio::time::advance(Duration::from_secs(31)).await;

    let err = pending.await.unwrap().unwrap_err();
    assert!(matches!(err, AgentSystemError::Timeout(_)));
}

#[tokio::test]
async fn execute_tool_call_tool_unavailable_returns_typed_error() {
    let bus = ToolBus::new();
    bus.register(
        "analyzer",
        "data",
        AgentId::new(),
        "Analyzer",
        serde_json::json!({"type": "object"}),
        serde_json::json!({"type": "object"}),
    );

    let call = LlmToolCall {
        call_id: "call-unavailable".into(),
        name: "data.analyzer".into(),
        input: serde_json::json!({}),
    };

    let err = bus.execute_tool_call(None, &call).await.unwrap_err();
    assert!(matches!(err, AgentSystemError::ToolUnavailable(_)));
}
