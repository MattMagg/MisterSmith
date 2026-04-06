mod common;

use axum::{routing::{get, post}, Json, Router};
use serde_json::json;
use std::process::Stdio;
use tokio::process::Command;

#[tokio::test(flavor = "multi_thread")]
async fn no_arg_entry_renders_recent_first_home() {
    let session_id = "11111111-1111-1111-1111-111111111111";
    let app = Router::new()
        .route(
            "/api/v1/health",
            get(|| async { Json(json!({ "status": "healthy", "components": [] })) }),
        )
        .route(
            "/api/v1/sessions",
            get(move || async move {
                Json(json!([
                    {
                        "title": "finish packet 030 shell",
                        "session_id": session_id,
                        "status": "active",
                        "coordinator_agent_id": "22222222-2222-2222-2222-222222222222",
                        "provider_kind": "openai_chatgpt",
                        "model_id": "gpt-5.4",
                        "active_workflow_id": null,
                        "last_completed_workflow_id": "33333333-3333-3333-3333-333333333333",
                        "turn_count": 4,
                        "updated_at": "2026-04-05T12:00:00Z",
                        "ended_at": null,
                        "last_preview": "tighten the CLI shell contract"
                    }
                ]))
            }),
        );
    let (base_url, handle) = common::spawn_mock_server(app).await;

    let output = Command::new(common::binary_path())
        .arg("--base-url")
        .arg(&base_url)
        .stdin(Stdio::null())
        .output()
        .await
        .expect("binary should run");

    handle.abort();

    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Mister Smith CLI shell"));
    assert!(stdout.contains("resume_last: 11111111-1111-1111-1111-111111111111"));
    assert!(stdout.contains("finish packet 030 shell"));
    assert!(stdout.contains("actions:"));
}

#[tokio::test(flavor = "multi_thread")]
async fn direct_prompt_entry_starts_a_new_session() {
    let app = Router::new().route(
        "/api/v1/sessions",
        post(|| async {
            Json(json!({
                "session_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "workflow_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                "coordinator_agent_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
                "turn_index": 1,
                "status": "queued"
            }))
        }),
    );
    let (base_url, handle) = common::spawn_mock_server(app).await;

    let output = Command::new(common::binary_path())
        .arg("--base-url")
        .arg(&base_url)
        .arg("start")
        .arg("a")
        .arg("fresh")
        .arg("session")
        .stdin(Stdio::null())
        .output()
        .await
        .expect("binary should run");

    handle.abort();

    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("session_id: aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"));
    assert!(stdout.contains("workflow_id: bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"));
    assert!(stdout.contains("turn_index: 1"));
}
