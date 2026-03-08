use futures::StreamExt;
use mister_smith_core::LlmError;
use mister_smith_llm::{
    ChatMessage, ChunkDelta, CompletionRequest, ContentBlock, MockProvider, ModelCapabilities,
    ModelProvider, StopReason,
};
use serde_json::json;

fn request_with_user(prompt: &str) -> CompletionRequest {
    CompletionRequest {
        messages: vec![ChatMessage::User {
            content: json!(prompt),
        }],
        system: None,
        tools: None,
        temperature: None,
        max_tokens: None,
        stop_sequences: None,
        metadata: json!({}),
        routing_hint: None,
    }
}

#[tokio::test]
async fn complete_returns_deterministic_text_response() {
    let provider = MockProvider::default();
    let response = provider
        .complete(CompletionRequest {
            metadata: json!({ "mock_response_text": "deterministic-answer" }),
            ..request_with_user("hello")
        })
        .await
        .unwrap();

    assert_eq!(response.model_id, "mock-default");
    assert_eq!(
        response.content,
        vec![ContentBlock::Text {
            text: "deterministic-answer".to_string(),
        }]
    );
    assert_eq!(response.stop_reason, StopReason::Completed);
    assert!(response.usage.input_tokens > 0);
    assert!(response.usage.output_tokens > 0);
    assert!(response.tool_calls.is_empty());
}

#[tokio::test]
async fn complete_returns_tool_call_when_requested_in_metadata() {
    let provider = MockProvider::default();
    let response = provider
        .complete(CompletionRequest {
            tools: Some(vec![]),
            metadata: json!({
                "mock_tool_call": {
                    "call_id": "call-7",
                    "name": "search",
                    "input": { "query": "phase 9" }
                }
            }),
            ..request_with_user("find phase 9 context")
        })
        .await
        .unwrap();

    assert_eq!(response.stop_reason, StopReason::ToolCall);
    assert_eq!(response.tool_calls.len(), 1);
    assert_eq!(response.tool_calls[0].call_id, "call-7");
    assert_eq!(response.tool_calls[0].name, "search");
    assert_eq!(response.tool_calls[0].input, json!({ "query": "phase 9" }));
}

#[tokio::test]
async fn stream_emits_ordered_text_and_terminal_stop() {
    let provider = MockProvider::default();
    let chunks = provider
        .stream(CompletionRequest {
            metadata: json!({ "mock_response_text": "stream-me" }),
            ..request_with_user("stream")
        })
        .collect::<Vec<_>>()
        .await;

    assert_eq!(chunks.len(), 2);
    assert_eq!(
        chunks[0].as_ref().unwrap().delta,
        ChunkDelta::Text {
            text: "stream-me".to_string(),
        }
    );
    assert_eq!(
        chunks[1].as_ref().unwrap().delta,
        ChunkDelta::Stop {
            reason: StopReason::Completed,
        }
    );
    assert_eq!(chunks[0].as_ref().unwrap().index, 0);
    assert_eq!(chunks[1].as_ref().unwrap().index, 1);
}

#[tokio::test]
async fn embed_is_deterministic() {
    let provider = MockProvider::default();
    let first = provider
        .embed(vec!["alpha".to_string(), "beta".to_string()])
        .await
        .unwrap();
    let second = provider
        .embed(vec!["alpha".to_string(), "beta".to_string()])
        .await
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(first.model_id, "mock-default");
    assert_eq!(first.embeddings.len(), 2);
    assert!(!first.embeddings[0].is_empty());
}

#[tokio::test]
async fn unsupported_capability_returns_typed_error() {
    let provider = MockProvider::default().with_capabilities(ModelCapabilities {
        completion: true,
        streaming: true,
        embeddings: false,
        tool_calling: true,
    });

    let error = provider.embed(vec!["alpha".to_string()]).await.unwrap_err();

    assert!(matches!(
        error,
        LlmError::UnsupportedCapability { capability, model }
        if capability == "embeddings" && model == "mock-default"
    ));
}
