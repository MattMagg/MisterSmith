//! Tool-calling bridge tests (requires llm feature).
//!
//! Run with: cargo test -p mister-smith-agents --features llm

#![cfg(feature = "llm")]

use mister_smith_agents::tool_bus::ToolBus;
use mister_smith_core::AgentId;
use mister_smith_llm::ToolCall;

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

    // Find the analyzer definition
    let analyzer = defs.iter().find(|d| d.name == "data.analyzer").unwrap();
    assert_eq!(analyzer.description, "Analyzes data");
    assert!(analyzer.input_schema.is_object());

    // Find the formatter definition
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
    let call = ToolCall {
        call_id: "call-1".into(),
        name: "no-namespace".into(), // Missing namespace.name format
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
async fn execute_tool_call_tool_not_found() {
    let bus = ToolBus::new();
    let call = ToolCall {
        call_id: "call-1".into(),
        name: "ns.nonexistent".into(),
        input: serde_json::json!({}),
    };

    let result = bus.execute_tool_call(None, &call).await;
    // execute_tool_call returns Ok with error field when invoke fails
    assert!(result.is_ok());
    let tool_result = result.unwrap();
    assert!(tool_result.error.is_some());
    assert!(tool_result.output.is_none());
    assert_eq!(tool_result.call_id, "call-1");
}
