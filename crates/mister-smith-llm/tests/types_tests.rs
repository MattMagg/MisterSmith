use mister_smith_llm::{
    ChatMessage, ChunkDelta, CompletionRequest, CompletionResponse, ContentBlock,
    ModelCapabilities, ProviderConfig, ProviderKind, StopReason, StreamChunk, ToolCall,
    ToolDefinition, ToolResult, Usage,
};
use serde_json::json;

#[test]
fn completion_request_roundtrip_preserves_tools_and_metadata() {
    let request = CompletionRequest {
        messages: vec![
            ChatMessage::System {
                content: "keep responses terse".to_string(),
            },
            ChatMessage::User {
                content: json!({ "prompt": "summarize this" }),
            },
            ChatMessage::Tool {
                result: ToolResult::success("call-1", json!({ "summary": "ok" })),
            },
        ],
        system: Some("planner".to_string()),
        tools: Some(vec![ToolDefinition {
            name: "search".to_string(),
            description: "Searches indexed content".to_string(),
            input_schema: json!({ "type": "object", "properties": { "query": { "type": "string" } } }),
        }]),
        temperature: Some(0.2),
        max_tokens: Some(256),
        stop_sequences: Some(vec!["END".to_string()]),
        metadata: json!({ "trace_id": "abc123" }),
    };

    let encoded = serde_json::to_string(&request).unwrap();
    let decoded: CompletionRequest = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, request);
}

#[test]
fn completion_response_roundtrip_preserves_tool_calls() {
    let response = CompletionResponse {
        content: vec![
            ContentBlock::Text {
                text: "thinking".to_string(),
            },
            ContentBlock::ToolUse {
                call_id: "call-1".to_string(),
                name: "search".to_string(),
                input: json!({ "query": "mister smith" }),
            },
        ],
        model_id: "mock-default".to_string(),
        usage: Usage::new(12, 8),
        stop_reason: StopReason::ToolCall,
        tool_calls: vec![ToolCall {
            call_id: "call-1".to_string(),
            name: "search".to_string(),
            input: json!({ "query": "mister smith" }),
        }],
    };

    let encoded = serde_json::to_string(&response).unwrap();
    let decoded: CompletionResponse = serde_json::from_str(&encoded).unwrap();

    assert_eq!(decoded, response);
}

#[test]
fn stream_chunk_and_config_defaults_match_phase9_contract() {
    let chunk = StreamChunk::stop(3, StopReason::Completed);
    assert_eq!(
        chunk,
        StreamChunk {
            index: 3,
            delta: ChunkDelta::Stop {
                reason: StopReason::Completed,
            },
        }
    );

    let config = ProviderConfig::default();
    assert_eq!(config.provider_kind, ProviderKind::Mock);
    assert_eq!(config.model_id, "mock-default");
    assert_eq!(config.timeout_ms, 30_000);
    assert_eq!(config.max_retries, 0);

    assert_eq!(ModelCapabilities::all().completion, true);
    assert_eq!(ModelCapabilities::all().streaming, true);
    assert_eq!(ModelCapabilities::all().embeddings, true);
    assert_eq!(ModelCapabilities::all().tool_calling, true);
}

#[test]
fn provider_config_roundtrip_preserves_openai_chatgpt_configuration() {
    let raw = json!({
        "provider_kind": "openai_chatgpt",
        "model_id": "gpt-5.4",
        "timeout_ms": 15_000,
        "max_retries": 1,
        "metadata": {
            "service_name": "mister-smith"
        }
    });

    let config: ProviderConfig = serde_json::from_value(raw.clone()).unwrap();
    let encoded = serde_json::to_value(config).unwrap();

    assert_eq!(encoded, raw);
}

#[test]
fn provider_config_validation_requires_api_key_for_usage_billed_openai() {
    let error = ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5".to_string(),
        ..ProviderConfig::default()
    }
    .validate()
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "LLM invalid request: provider 'openai' requires api_key_env"
    );
}

#[test]
fn provider_config_validation_accepts_chatgpt_provider_without_api_key_env() {
    ProviderConfig {
        provider_kind: ProviderKind::OpenAiChatGpt,
        model_id: "gpt-5".to_string(),
        ..ProviderConfig::default()
    }
    .validate()
    .unwrap();
}

#[test]
fn provider_kind_rejects_unknown_backend_names() {
    let error = serde_json::from_value::<ProviderConfig>(json!({
        "provider_kind": "openai_chatgpt_workspace",
        "model_id": "gpt-5"
    }))
    .unwrap_err();

    assert!(error.to_string().contains("unknown variant"));
}
