#[cfg(feature = "openai-chatgpt")]
use std::fs;
#[cfg(feature = "openai")]
use std::net::SocketAddr;
#[cfg(feature = "openai-chatgpt")]
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
#[cfg(feature = "openai-chatgpt")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(feature = "openai")]
use axum::{extract::State, routing::post, Json, Router};
use futures::StreamExt;
use mister_smith_core::LlmError;
#[cfg(feature = "openai-chatgpt")]
use mister_smith_llm::OpenAiChatGptProvider;
#[cfg(feature = "openai")]
use mister_smith_llm::OpenAiProvider;
#[cfg(feature = "openai")]
use mister_smith_llm::ToolCall;
use mister_smith_llm::{
    ChatMessage, ChunkDelta, CompletionRequest, ContentBlock, ModelProvider, ProviderConfig,
    ProviderKind, StopReason,
};
use serde_json::json;
#[cfg(feature = "openai")]
use tokio::net::TcpListener;
#[cfg(feature = "openai")]
use tokio::sync::Mutex as AsyncMutex;

fn completion_request(prompt: &str) -> CompletionRequest {
    CompletionRequest {
        messages: vec![ChatMessage::User {
            content: json!(prompt),
        }],
        ..CompletionRequest::default()
    }
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[cfg(feature = "openai")]
fn set_locked_env(name: &str, value: &str) -> std::sync::MutexGuard<'static, ()> {
    let guard = env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var(name, value);
    guard
}

#[cfg(feature = "openai-chatgpt")]
fn fake_codex_script_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("mister-smith-fake-codex-{unique}.py"))
}

#[cfg(feature = "openai-chatgpt")]
fn write_fake_codex_script(authenticated: bool, requires_openai_auth: bool) -> PathBuf {
    write_fake_codex_script_with_turn_notifications(
        authenticated,
        requires_openai_auth,
        r#"
        send({
            "jsonrpc": "2.0",
            "method": "item/agentMessage/delta",
            "params": {
                "threadId": "thread-1",
                "turnId": "turn-1",
                "itemId": "item-1",
                "delta": "hello from chatgpt"
            }
        })
        send({
            "jsonrpc": "2.0",
            "method": "turn/completed",
            "params": {
                "threadId": "thread-1",
                "turn": {
                    "id": "turn-1",
                    "items": [],
                    "status": "completed",
                    "error": None
                }
            }
        })
"#,
    )
}

#[cfg(feature = "openai-chatgpt")]
fn write_fake_codex_script_with_turn_notifications(
    authenticated: bool,
    requires_openai_auth: bool,
    turn_notifications: &str,
) -> PathBuf {
    let path = fake_codex_script_path();
    let authenticated = if authenticated { "True" } else { "False" };
    let requires_openai_auth = if requires_openai_auth {
        "True"
    } else {
        "False"
    };
    let script = format!(
        r#"#!/usr/bin/env python3
import json
import sys

AUTHENTICATED = {authenticated}

def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()

for raw in sys.stdin:
    if not raw.strip():
        continue

    message = json.loads(raw)
    method = message.get("method")

    if method == "initialize":
        send({{"jsonrpc": "2.0", "id": message["id"], "result": {{"userAgent": "fake-codex"}}}})
    elif method == "initialized":
        continue
    elif method == "account/read":
        account = {{"type": "chatgpt", "email": "ops@example.com", "planType": "team"}} if AUTHENTICATED else None
        send({{
            "jsonrpc": "2.0",
            "id": message["id"],
            "result": {{
                "account": account,
                "requiresOpenaiAuth": {requires_openai_auth}
            }}
        }})
    elif method == "thread/start":
        params = message.get("params", {{}})
        if params.get("sandbox") != "read-only":
            send({{
                "jsonrpc": "2.0",
                "id": message["id"],
                "error": {{
                    "code": -32602,
                    "message": "thread/start sandbox must be read-only"
                }}
            }})
            continue
        send({{
            "jsonrpc": "2.0",
            "id": message["id"],
            "result": {{
                "thread": {{
                    "id": "thread-1",
                    "preview": "",
                    "ephemeral": True,
                    "modelProvider": "openai",
                    "createdAt": 1,
                    "updatedAt": 1,
                    "status": "ready",
                    "path": None,
                    "cwd": ".",
                    "cliVersion": "test",
                    "source": "app-server",
                    "agentNickname": None,
                    "agentRole": None,
                    "gitInfo": None,
                    "name": None,
                    "turns": []
                }},
                "model": "gpt-5",
                "modelProvider": "openai",
                "cwd": ".",
                "approvalPolicy": "never",
                "sandbox": "danger-full-access",
                "reasoningEffort": None
            }}
        }})
    elif method == "turn/start":
        params = message.get("params", {{}})
        sandbox_policy = params.get("sandboxPolicy") or {{}}
        if sandbox_policy.get("type") != "readOnly" or sandbox_policy.get("networkAccess") is not False:
            send({{
                "jsonrpc": "2.0",
                "id": message["id"],
                "error": {{
                    "code": -32602,
                    "message": "turn/start sandboxPolicy must be readOnly with networkAccess=false"
                }}
            }})
            continue
        send({{
            "jsonrpc": "2.0",
            "id": message["id"],
            "result": {{
                "turn": {{
                    "id": "turn-1",
                    "items": [],
                    "status": "inProgress",
                    "error": None
                }}
            }}
        }})
{turn_notifications}
"#,
        turn_notifications = turn_notifications
    );

    fs::write(&path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
    }
    path
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_normalizes_responses_api_payload_from_stub_server() {
    async fn handler() -> Json<serde_json::Value> {
        Json(json!({
            "id": "resp_123",
            "model": "gpt-5-mini",
            "status": "completed",
            "output": [{
                "type": "message",
                "id": "msg_123",
                "role": "assistant",
                "content": [{
                    "type": "output_text",
                    "text": "stubbed answer",
                    "annotations": []
                }]
            }],
            "usage": {
                "input_tokens": 7,
                "output_tokens": 3,
                "total_tokens": 10
            }
        }))
    }

    let app = Router::new().route("/responses", post(handler));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider
        .complete(completion_request("hello"))
        .await
        .unwrap();

    assert_eq!(response.model_id, "gpt-5-mini");
    assert_eq!(
        response.content,
        vec![ContentBlock::Text {
            text: "stubbed answer".to_string(),
        }]
    );
    assert_eq!(response.stop_reason, StopReason::Completed);
    assert!(response.tool_calls.is_empty());
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_sets_minimal_reasoning_for_bounded_gpt5_requests() {
    async fn handler(
        State(captured_body): State<std::sync::Arc<AsyncMutex<Option<serde_json::Value>>>>,
        Json(body): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        *captured_body.lock().await = Some(body);

        Json(json!({
            "model": "gpt-5-mini",
            "status": "completed",
            "output": [{
                "type": "message",
                "role": "assistant",
                "content": [{
                    "type": "output_text",
                    "text": "ack",
                    "annotations": []
                }]
            }]
        }))
    }

    let captured_body = std::sync::Arc::new(AsyncMutex::new(None));
    let app = Router::new()
        .route("/responses", post(handler))
        .with_state(captured_body.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider
        .complete(CompletionRequest {
            max_tokens: Some(16),
            ..completion_request("Reply with ack.")
        })
        .await
        .unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::Text {
            text: "ack".to_string(),
        }]
    );

    let captured_body = captured_body.lock().await.clone().unwrap();
    assert_eq!(
        captured_body
            .get("reasoning")
            .and_then(|value| value.get("effort"))
            .and_then(serde_json::Value::as_str),
        Some("minimal")
    );
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_rejects_empty_incomplete_responses_without_visible_output() {
    async fn handler() -> Json<serde_json::Value> {
        Json(json!({
            "model": "gpt-5-mini",
            "status": "incomplete",
            "incomplete_details": {
                "reason": "max_output_tokens"
            },
            "output": [{
                "type": "reasoning",
                "summary": []
            }],
            "usage": {
                "input_tokens": 7,
                "output_tokens": 16,
                "total_tokens": 23
            }
        }))
    }

    let app = Router::new().route("/responses", post(handler));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        ..ProviderConfig::default()
    })
    .unwrap();

    let error = provider
        .complete(CompletionRequest {
            max_tokens: Some(16),
            ..completion_request("Reply with ack.")
        })
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        LlmError::InvalidRequest(message)
        if message.contains("no visible output")
            && message.contains("max_output_tokens")
    ));
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_preserves_content_filter_stops_without_visible_output() {
    async fn handler() -> Json<serde_json::Value> {
        Json(json!({
            "model": "gpt-5-mini",
            "status": "incomplete",
            "incomplete_details": {
                "reason": "content_filter"
            },
            "output": [],
            "usage": {
                "input_tokens": 7,
                "output_tokens": 0,
                "total_tokens": 7
            }
        }))
    }

    let app = Router::new().route("/responses", post(handler));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider
        .complete(completion_request("Reply with ack."))
        .await
        .unwrap();

    assert!(response.content.is_empty());
    assert_eq!(response.stop_reason, StopReason::ContentFilter);
    assert!(response.tool_calls.is_empty());
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_uses_embedding_model_override() {
    async fn handler(
        State(captured_model): State<std::sync::Arc<AsyncMutex<Option<String>>>>,
        Json(body): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        *captured_model.lock().await = body
            .get("model")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);

        Json(json!({
            "object": "list",
            "data": [{
                "object": "embedding",
                "embedding": [0.1, 0.2, 0.3],
                "index": 0
            }],
            "model": "text-embedding-3-small",
            "usage": {
                "prompt_tokens": 4,
                "total_tokens": 4
            }
        }))
    }

    let captured_model = std::sync::Arc::new(AsyncMutex::new(None));
    let app = Router::new()
        .route("/embeddings", post(handler))
        .with_state(captured_model.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        metadata: json!({
            "embedding_model_id": "text-embedding-3-small"
        }),
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider.embed(vec!["hello".to_string()]).await.unwrap();

    assert_eq!(response.model_id, "text-embedding-3-small");
    assert_eq!(response.embeddings, vec![vec![0.1, 0.2, 0.3]]);
    assert_eq!(response.usage.input_tokens, 4);
    assert_eq!(response.usage.output_tokens, 0);
    assert_eq!(response.usage.total_tokens, 4);
    assert_eq!(
        captured_model.lock().await.as_deref(),
        Some("text-embedding-3-small")
    );
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_normalizes_tool_calls_from_responses_api_payload() {
    async fn handler() -> Json<serde_json::Value> {
        Json(json!({
            "model": "gpt-5-mini",
            "status": "completed",
            "output": [{
                "type": "function_call",
                "id": "fc_123",
                "call_id": "call_123",
                "name": "search_docs",
                "arguments": "{\"query\":\"mister smith\"}",
                "status": "completed"
            }],
            "usage": {
                "input_tokens": 11,
                "output_tokens": 5,
                "total_tokens": 16
            }
        }))
    }

    let app = Router::new().route("/responses", post(handler));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider
        .complete(CompletionRequest {
            tools: Some(vec![mister_smith_llm::ToolDefinition {
                name: "search_docs".to_string(),
                description: "Search indexed docs".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }),
            }]),
            ..completion_request("search the docs")
        })
        .await
        .unwrap();

    assert_eq!(response.stop_reason, StopReason::ToolCall);
    assert_eq!(
        response.tool_calls,
        vec![ToolCall {
            call_id: "call_123".to_string(),
            name: "search_docs".to_string(),
            input: json!({ "query": "mister smith" }),
        }]
    );
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_streams_text_deltas_from_sse() {
    async fn handler() -> impl axum::response::IntoResponse {
        (
            [(axum::http::header::CONTENT_TYPE, "text/event-stream")],
            concat!(
                "event: response.output_text.delta\n",
                "data: {\"delta\":\"hello \"}\n\n",
                "event: response.output_text.delta\n",
                "data: {\"delta\":\"world\"}\n\n",
                "event: response.completed\n",
                "data: {\"type\":\"response.completed\"}\n\n"
            ),
        )
    }

    let app = Router::new().route("/responses", post(handler));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        ..ProviderConfig::default()
    })
    .unwrap();

    let chunks = provider
        .stream(completion_request("stream hello world"))
        .collect::<Vec<_>>()
        .await;

    assert_eq!(chunks.len(), 3);
    assert!(matches!(
        chunks[0].as_ref().unwrap().delta,
        ChunkDelta::Text { ref text } if text == "hello "
    ));
    assert!(matches!(
        chunks[1].as_ref().unwrap().delta,
        ChunkDelta::Text { ref text } if text == "world"
    ));
    assert!(matches!(
        chunks[2].as_ref().unwrap().delta,
        ChunkDelta::Stop { ref reason } if reason == &StopReason::Completed
    ));
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_retries_retryable_failures() {
    #[derive(Clone, Default)]
    struct Attempts {
        count: std::sync::Arc<AsyncMutex<u32>>,
    }

    async fn handler(
        State(attempts): State<Attempts>,
    ) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
        let mut count = attempts.count.lock().await;
        *count += 1;
        if *count == 1 {
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        } else {
            Ok(Json(json!({
                "model": "gpt-5-mini",
                "status": "completed",
                "output": [{
                    "type": "message",
                    "role": "assistant",
                    "content": [{
                        "type": "output_text",
                        "text": "retried answer",
                        "annotations": []
                    }]
                }],
                "usage": {
                    "input_tokens": 3,
                    "output_tokens": 2,
                    "total_tokens": 5
                }
            })))
        }
    }

    let attempts = Attempts::default();
    let app = Router::new()
        .route("/responses", post(handler))
        .with_state(attempts.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        max_retries: 1,
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider
        .complete(completion_request("retry me"))
        .await
        .unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::Text {
            text: "retried answer".to_string()
        }]
    );
    assert_eq!(*attempts.count.lock().await, 2);
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_honors_timeout_ms() {
    async fn handler() -> Json<serde_json::Value> {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Json(json!({
            "model": "gpt-5-mini",
            "status": "completed",
            "output": [{
                "type": "message",
                "role": "assistant",
                "content": [{
                    "type": "output_text",
                    "text": "late answer",
                    "annotations": []
                }]
            }]
        }))
    }

    let app = Router::new().route("/responses", post(handler));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        timeout_ms: 10,
        ..ProviderConfig::default()
    })
    .unwrap();

    let error = provider
        .complete(completion_request("timeout"))
        .await
        .unwrap_err();

    assert!(matches!(error, LlmError::Network(message) if message.contains("timed out")));
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_rejects_empty_api_key_value() {
    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "   ");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some("http://127.0.0.1:9".to_string()),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        ..ProviderConfig::default()
    })
    .unwrap();

    let error = provider
        .complete(completion_request("hello"))
        .await
        .unwrap_err();

    match error {
        LlmError::Authentication(message) => {
            assert!(
                message.contains("environment variable 'OPENAI_TEST_API_KEY' is empty"),
                "unexpected authentication message: {message}"
            );
        }
        other => panic!("unexpected error for empty OpenAI API key: {other:?}"),
    }

    std::env::remove_var("OPENAI_TEST_API_KEY");
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_maps_unauthorized_responses_to_authentication() {
    async fn handler() -> impl axum::response::IntoResponse {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": {
                    "message": "invalid api key"
                }
            })),
        )
    }

    let app = Router::new().route("/responses", post(handler));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        ..ProviderConfig::default()
    })
    .unwrap();

    let error = provider
        .complete(completion_request("hello"))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        LlmError::Authentication(message) if message == "invalid api key"
    ));
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_maps_forbidden_responses_to_authentication() {
    async fn handler() -> impl axum::response::IntoResponse {
        (
            axum::http::StatusCode::FORBIDDEN,
            Json(json!({
                "error": {
                    "message": "project does not allow this model"
                }
            })),
        )
    }

    let app = Router::new().route("/responses", post(handler));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        ..ProviderConfig::default()
    })
    .unwrap();

    let error = provider
        .complete(completion_request("hello"))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        LlmError::Authentication(message) if message == "project does not allow this model"
    ));
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_maps_rate_limits_with_retry_after() {
    async fn handler() -> axum::response::Response {
        axum::http::Response::builder()
            .status(axum::http::StatusCode::TOO_MANY_REQUESTS)
            .header(axum::http::header::RETRY_AFTER, "13")
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::from(
                r#"{"error":{"message":"rate limited"}}"#,
            ))
            .unwrap()
    }

    let app = Router::new().route("/responses", post(handler));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        ..ProviderConfig::default()
    })
    .unwrap();

    let error = provider
        .complete(completion_request("hello"))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        LlmError::RateLimited {
            retry_after_secs: Some(13)
        }
    ));
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_surfaces_non_json_server_errors() {
    async fn handler() -> impl axum::response::IntoResponse {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "backend exploded",
        )
    }

    let app = Router::new().route("/responses", post(handler));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        ..ProviderConfig::default()
    })
    .unwrap();

    let error = provider
        .complete(completion_request("hello"))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        LlmError::ProviderError {
            status: 500,
            ref message,
            retryable: true
        } if message == "backend exploded"
    ));
}

#[cfg(feature = "openai")]
#[tokio::test]
async fn openai_provider_retries_stream_setup_failures() {
    #[derive(Clone, Default)]
    struct Attempts {
        count: std::sync::Arc<AsyncMutex<u32>>,
    }

    async fn handler(State(attempts): State<Attempts>) -> axum::response::Response {
        let mut count = attempts.count.lock().await;
        *count += 1;
        if *count == 1 {
            axum::http::Response::builder()
                .status(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
                .header(axum::http::header::CONTENT_TYPE, "text/plain")
                .body(axum::body::Body::from("try again"))
                .unwrap()
        } else {
            axum::http::Response::builder()
                .status(axum::http::StatusCode::OK)
                .header(axum::http::header::CONTENT_TYPE, "text/event-stream")
                .body(axum::body::Body::from(concat!(
                    "event: response.output_text.delta\n",
                    "data: {\"delta\":\"retry \"}\n\n",
                    "event: response.output_text.delta\n",
                    "data: {\"delta\":\"worked\"}\n\n",
                    "event: response.completed\n",
                    "data: {\"type\":\"response.completed\"}\n\n"
                )))
                .unwrap()
        }
    }

    let attempts = Attempts::default();
    let app = Router::new()
        .route("/responses", post(handler))
        .with_state(attempts.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let _guard = set_locked_env("OPENAI_TEST_API_KEY", "test-key");

    let provider = OpenAiProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAi,
        model_id: "gpt-5-mini".to_string(),
        api_base_url: Some(format!("http://{address}")),
        api_key_env: Some("OPENAI_TEST_API_KEY".to_string()),
        max_retries: 1,
        ..ProviderConfig::default()
    })
    .unwrap();

    let chunks = provider
        .stream(completion_request("retry stream"))
        .collect::<Vec<_>>()
        .await;

    assert_eq!(*attempts.count.lock().await, 2);
    assert_eq!(chunks.len(), 3);
    assert!(matches!(
        chunks[0].as_ref().unwrap().delta,
        ChunkDelta::Text { ref text } if text == "retry "
    ));
    assert!(matches!(
        chunks[1].as_ref().unwrap().delta,
        ChunkDelta::Text { ref text } if text == "worked"
    ));
    assert!(matches!(
        chunks[2].as_ref().unwrap().delta,
        ChunkDelta::Stop { ref reason } if reason == &StopReason::Completed
    ));
}

#[cfg(feature = "openai-chatgpt")]
#[tokio::test]
async fn openai_chatgpt_provider_reports_unsupported_embeddings_and_tool_calling() {
    let provider = OpenAiChatGptProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAiChatGpt,
        model_id: "gpt-5".to_string(),
        ..ProviderConfig::default()
    })
    .unwrap();

    let capabilities = provider.capabilities();
    assert!(capabilities.completion);
    assert!(capabilities.streaming);
    assert!(!capabilities.embeddings);
    assert!(!capabilities.tool_calling);

    let error = provider.embed(vec!["hello".to_string()]).await.unwrap_err();

    assert!(matches!(
        error,
        LlmError::UnsupportedCapability { capability, model }
        if capability == "embeddings" && model == "gpt-5"
    ));
}

#[cfg(feature = "openai-chatgpt")]
#[tokio::test]
async fn openai_chatgpt_provider_requires_codex_login_before_completion() {
    let _guard = env_lock().lock().unwrap();
    let script_path = write_fake_codex_script(false, true);
    std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

    let provider = OpenAiChatGptProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAiChatGpt,
        model_id: "gpt-5".to_string(),
        ..ProviderConfig::default()
    })
    .unwrap();

    let error = provider
        .complete(completion_request("hello"))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        LlmError::Authentication(message)
        if message.contains("mister-smith auth openai-chatgpt login")
    ));

    std::env::remove_var("MISTER_SMITH_CODEX_BIN");
    let _ = fs::remove_file(script_path);
}

#[cfg(feature = "openai-chatgpt")]
#[tokio::test]
async fn openai_chatgpt_provider_normalizes_completion_from_codex_app_server() {
    let _guard = env_lock().lock().unwrap();
    let script_path = write_fake_codex_script(true, true);
    std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

    let provider = OpenAiChatGptProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAiChatGpt,
        model_id: "gpt-5".to_string(),
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider
        .complete(completion_request("hello"))
        .await
        .unwrap();

    assert_eq!(response.model_id, "gpt-5");
    assert_eq!(
        response.content,
        vec![ContentBlock::Text {
            text: "hello from chatgpt".to_string(),
        }]
    );
    assert_eq!(response.stop_reason, StopReason::Completed);
    assert!(response.tool_calls.is_empty());

    std::env::remove_var("MISTER_SMITH_CODEX_BIN");
    let _ = fs::remove_file(script_path);
}

#[cfg(feature = "openai-chatgpt")]
#[tokio::test]
async fn openai_chatgpt_provider_rejects_codex_api_key_auth_mode() {
    let _guard = env_lock().lock().unwrap();
    let script_path = fake_codex_script_path();
    let script = r#"#!/usr/bin/env python3
import json
import sys

def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()

for raw in sys.stdin:
    if not raw.strip():
        continue
    message = json.loads(raw)
    method = message.get("method")
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": message["id"], "result": {"userAgent": "fake-codex"}})
    elif method == "initialized":
        continue
    elif method == "account/read":
        send({
            "jsonrpc": "2.0",
            "id": message["id"],
            "result": {
                "account": {"type": "apiKey"},
                "requiresOpenaiAuth": True
            }
        })
"#;
    fs::write(&script_path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&script_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions).unwrap();
    }
    std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

    let provider = OpenAiChatGptProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAiChatGpt,
        model_id: "gpt-5".to_string(),
        ..ProviderConfig::default()
    })
    .unwrap();

    let error = provider
        .complete(completion_request("hello"))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        LlmError::Authentication(message)
        if message.contains("authenticated with an API key")
    ));

    std::env::remove_var("MISTER_SMITH_CODEX_BIN");
    let _ = fs::remove_file(script_path);
}

#[cfg(feature = "openai-chatgpt")]
#[tokio::test]
async fn openai_chatgpt_provider_accepts_authenticated_session_when_openai_auth_is_required() {
    let _guard = env_lock().lock().unwrap();
    let script_path = write_fake_codex_script(true, true);
    std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

    let provider = OpenAiChatGptProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAiChatGpt,
        model_id: "gpt-5".to_string(),
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider
        .complete(completion_request("hello"))
        .await
        .unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::Text {
            text: "hello from chatgpt".to_string()
        }]
    );

    std::env::remove_var("MISTER_SMITH_CODEX_BIN");
    let _ = fs::remove_file(script_path);
}

#[cfg(feature = "openai-chatgpt")]
#[tokio::test]
async fn openai_chatgpt_provider_honors_timeout_ms() {
    let _guard = env_lock().lock().unwrap();
    let script_path = write_fake_codex_script_with_turn_notifications(
        true,
        true,
        r#"
        import time
        time.sleep(0.2)
"#,
    );
    std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

    let provider = OpenAiChatGptProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAiChatGpt,
        model_id: "gpt-5".to_string(),
        timeout_ms: 25,
        ..ProviderConfig::default()
    })
    .unwrap();

    let error = provider
        .complete(completion_request("hello"))
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        LlmError::Network(message)
        if message.contains("Codex app-server request timed out after 25ms")
    ));

    std::env::remove_var("MISTER_SMITH_CODEX_BIN");
    let _ = fs::remove_file(script_path);
}

#[cfg(feature = "openai-chatgpt")]
#[tokio::test]
async fn openai_chatgpt_provider_uses_item_completed_as_authoritative_fallback_and_tracks_usage() {
    let _guard = env_lock().lock().unwrap();
    let script_path = write_fake_codex_script_with_turn_notifications(
        true,
        true,
        r#"
        send({
            "jsonrpc": "2.0",
            "method": "model/rerouted",
            "params": {
                "threadId": "thread-1",
                "turnId": "turn-1",
                "fromModel": "gpt-5",
                "toModel": "gpt-5-codex",
                "reason": "policy"
            }
        })
        send({
            "jsonrpc": "2.0",
            "method": "thread/tokenUsage/updated",
            "params": {
                "threadId": "thread-1",
                "turnId": "turn-1",
                "tokenUsage": {
                    "total": {
                        "totalTokens": 42,
                        "inputTokens": 30,
                        "cachedInputTokens": 2,
                        "outputTokens": 12,
                        "reasoningOutputTokens": 0
                    },
                    "last": {
                        "totalTokens": 42,
                        "inputTokens": 30,
                        "cachedInputTokens": 2,
                        "outputTokens": 12,
                        "reasoningOutputTokens": 0
                    },
                    "modelContextWindow": 200000
                }
            }
        })
        send({
            "jsonrpc": "2.0",
            "method": "item/completed",
            "params": {
                "threadId": "thread-1",
                "turnId": "turn-1",
                "item": {
                    "type": "agentMessage",
                    "id": "item-1",
                    "text": "final chatgpt answer",
                    "phase": None
                }
            }
        })
        send({
            "jsonrpc": "2.0",
            "method": "turn/completed",
            "params": {
                "threadId": "thread-1",
                "turn": {
                    "id": "turn-1",
                    "items": [],
                    "status": "completed",
                    "error": None
                }
            }
        })
"#,
    );
    std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

    let provider = OpenAiChatGptProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAiChatGpt,
        model_id: "gpt-5".to_string(),
        ..ProviderConfig::default()
    })
    .unwrap();

    let response = provider
        .complete(completion_request("hello"))
        .await
        .unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::Text {
            text: "final chatgpt answer".to_string(),
        }]
    );
    assert_eq!(response.model_id, "gpt-5-codex");
    assert_eq!(response.usage.input_tokens, 30);
    assert_eq!(response.usage.output_tokens, 12);
    assert_eq!(response.usage.total_tokens, 42);

    std::env::remove_var("MISTER_SMITH_CODEX_BIN");
    let _ = fs::remove_file(script_path);
}

#[cfg(feature = "openai-chatgpt")]
#[tokio::test]
async fn openai_chatgpt_provider_stream_emits_final_item_text_when_no_deltas_are_sent() {
    let _guard = env_lock().lock().unwrap();
    let script_path = write_fake_codex_script_with_turn_notifications(
        true,
        true,
        r#"
        send({
            "jsonrpc": "2.0",
            "method": "item/completed",
            "params": {
                "threadId": "thread-1",
                "turnId": "turn-1",
                "item": {
                    "type": "agentMessage",
                    "id": "item-1",
                    "text": "fallback streamed answer",
                    "phase": None
                }
            }
        })
        send({
            "jsonrpc": "2.0",
            "method": "turn/completed",
            "params": {
                "threadId": "thread-1",
                "turn": {
                    "id": "turn-1",
                    "items": [],
                    "status": "completed",
                    "error": None
                }
            }
        })
"#,
    );
    std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

    let provider = OpenAiChatGptProvider::new(ProviderConfig {
        provider_kind: ProviderKind::OpenAiChatGpt,
        model_id: "gpt-5".to_string(),
        ..ProviderConfig::default()
    })
    .unwrap();

    let chunks = provider
        .stream(completion_request("hello"))
        .collect::<Vec<_>>()
        .await;

    assert_eq!(chunks.len(), 2);
    assert!(matches!(
        chunks[0].as_ref().unwrap().delta,
        ChunkDelta::Text { ref text } if text == "fallback streamed answer"
    ));
    assert!(matches!(
        chunks[1].as_ref().unwrap().delta,
        ChunkDelta::Stop { ref reason } if reason == &StopReason::Completed
    ));

    std::env::remove_var("MISTER_SMITH_CODEX_BIN");
    let _ = fs::remove_file(script_path);
}
