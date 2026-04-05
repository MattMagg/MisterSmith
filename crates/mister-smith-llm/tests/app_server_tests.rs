use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use mister_smith_core::LlmError;
use mister_smith_llm::{
    AppServerAccountStatus, ChatMessage, CodexAppServerClient, CompletionRequest, ContentBlock,
    StopReason,
};
use serde_json::json;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn fake_codex_script_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("mister-smith-app-server-test-{unique}.py"))
}

fn write_fake_completion_script() -> PathBuf {
    let path = fake_codex_script_path();
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
                "account": {"type": "chatgpt", "email": "ops@example.com", "planType": "team"},
                "requiresOpenaiAuth": False
            }
        })
    elif method == "thread/start":
        send({
            "jsonrpc": "2.0",
            "id": message["id"],
            "result": {
                "thread": {"id": "thread-1", "status": "ready"}
            }
        })
    elif method == "turn/start":
        send({
            "jsonrpc": "2.0",
            "method": "mcpServer/startupStatus/updated",
            "params": {
                "server": "filesystem",
                "status": "ready"
            }
        })
        send({
            "jsonrpc": "2.0",
            "id": message["id"],
            "result": {
                "turn": {
                    "id": "turn-1",
                    "items": [],
                    "status": "inProgress",
                    "error": None
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
                    "text": "stubbed answer"
                }
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
                        "outputTokens": 12
                    }
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
"#;
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

#[test]
fn app_server_account_status_detects_authenticated_chatgpt_session() {
    let status = AppServerAccountStatus::from_account_read_payload(&json!({
        "account": {
            "type": "chatgpt",
            "email": "ops@example.com",
            "planType": "team"
        },
        "requiresOpenaiAuth": false
    }))
    .unwrap();

    assert_eq!(status.backend, "openai_chatgpt");
    assert_eq!(status.account_type.as_deref(), Some("chatgpt"));
    assert!(status.authenticated);
    assert_eq!(status.email.as_deref(), Some("ops@example.com"));
    assert_eq!(status.plan_type.as_deref(), Some("team"));
    assert!(!status.requires_openai_auth);
}

#[test]
fn app_server_account_status_detects_missing_chatgpt_login() {
    let status = AppServerAccountStatus::from_account_read_payload(&json!({
        "account": null,
        "requiresOpenaiAuth": true
    }))
    .unwrap();

    assert_eq!(status.backend, "openai_chatgpt");
    assert_eq!(status.account_type, None);
    assert!(!status.authenticated);
    assert_eq!(status.email, None);
    assert_eq!(status.plan_type, None);
    assert!(status.requires_openai_auth);
}

#[test]
fn app_server_account_status_accepts_api_key_auth_mode() {
    let status = AppServerAccountStatus::from_account_read_payload(&json!({
        "account": {
            "type": "apiKey"
        },
        "requiresOpenaiAuth": true
    }))
    .unwrap();

    assert_eq!(status.backend, "openai_chatgpt");
    assert_eq!(status.account_type.as_deref(), Some("apiKey"));
    assert!(status.authenticated);
    assert_eq!(status.email, None);
    assert_eq!(status.plan_type, None);
    assert!(status.requires_openai_auth);
}

#[test]
fn app_server_account_status_accepts_no_auth_required_mode() {
    let status = AppServerAccountStatus::from_account_read_payload(&json!({
        "account": null,
        "requiresOpenaiAuth": false
    }))
    .unwrap();

    assert_eq!(status.backend, "openai_chatgpt");
    assert_eq!(status.account_type, None);
    assert!(!status.authenticated);
    assert_eq!(status.email, None);
    assert_eq!(status.plan_type, None);
    assert!(!status.requires_openai_auth);
}

#[tokio::test]
async fn app_server_login_timeout_cancels_pending_browser_flow() {
    let _guard = env_lock().lock().unwrap();
    let script_path = fake_codex_script_path();
    let cancel_marker = std::env::temp_dir().join(format!(
        "mister-smith-login-cancelled-{}.txt",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let script = format!(
        r#"#!/usr/bin/env python3
import json
import pathlib
import sys

CANCEL_MARKER = pathlib.Path(r"{cancel_marker}")

def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()

for raw in sys.stdin:
    if not raw.strip():
        continue

    message = json.loads(raw)
    method = message.get("method")

    if method == "initialize":
        capabilities = message.get("params", {{}}).get("capabilities", {{}})
        if capabilities.get("experimentalApi") is True:
            send({{
                "jsonrpc": "2.0",
                "id": message["id"],
                "error": {{
                    "code": -32602,
                    "message": "experimentalApi must be omitted for stable clients"
                }}
            }})
            continue
        send({{"jsonrpc": "2.0", "id": message["id"], "result": {{"userAgent": "fake-codex"}}}})
    elif method == "initialized":
        continue
    elif method == "account/login/start":
        send({{
            "jsonrpc": "2.0",
            "id": message["id"],
            "result": {{
                "type": "chatgpt",
                "loginId": "login-timeout",
                "authUrl": "https://example.test/login"
            }}
        }})
    elif method == "account/login/cancel":
        login_id = message.get("params", {{}}).get("loginId")
        CANCEL_MARKER.write_text(login_id or "<missing>")
        send({{
            "jsonrpc": "2.0",
            "id": message["id"],
            "result": {{}}
        }})
"#,
        cancel_marker = cancel_marker.display()
    );
    fs::write(&script_path, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut permissions = fs::metadata(&script_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions).unwrap();
    }

    std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);
    std::env::set_var("MISTER_SMITH_OPENAI_CHATGPT_LOGIN_TIMEOUT_MS", "25");

    let mut client = CodexAppServerClient::connect().await.unwrap();
    let login_handle = client.start_chatgpt_login().await.unwrap();
    let error = client
        .wait_for_chatgpt_login(&login_handle)
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        LlmError::Authentication(message) if message.contains("timed out waiting for ChatGPT login")
    ));
    assert_eq!(
        fs::read_to_string(&cancel_marker).unwrap(),
        "login-timeout".to_string()
    );

    std::env::remove_var("MISTER_SMITH_CODEX_BIN");
    std::env::remove_var("MISTER_SMITH_OPENAI_CHATGPT_LOGIN_TIMEOUT_MS");
    let _ = fs::remove_file(script_path);
    let _ = fs::remove_file(cancel_marker);
}

#[tokio::test]
async fn app_server_run_completion_handles_notifications_before_turn_start_response() {
    let _guard = env_lock().lock().unwrap();
    let script_path = write_fake_completion_script();
    std::env::set_var("MISTER_SMITH_CODEX_BIN", &script_path);

    let mut client = CodexAppServerClient::connect().await.unwrap();
    let status = client.account_status(false).await.unwrap();
    assert!(status.is_chatgpt_session());

    let response = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        client.run_completion(
            "gpt-5.4",
            CompletionRequest {
                messages: vec![ChatMessage::User {
                    content: json!("Reply with the single word 'ack'."),
                }],
                ..CompletionRequest::default()
            },
            None,
        ),
    )
    .await
    .expect("run_completion should not hang when startup notifications arrive early")
    .unwrap();

    assert_eq!(
        response.content,
        vec![ContentBlock::Text {
            text: "stubbed answer".to_string()
        }]
    );
    assert_eq!(response.stop_reason, StopReason::Completed);
    assert_eq!(response.usage.total_tokens, 42);

    std::env::remove_var("MISTER_SMITH_CODEX_BIN");
    let _ = fs::remove_file(script_path);
}
