mod common;

use axum::{routing::get, Json, Router};
use serde_json::json;
use std::process::Stdio;
use tokio::process::Command;

#[tokio::test(flavor = "multi_thread")]
async fn resume_last_opens_the_most_recent_session() {
    let session_id = "11111111-1111-1111-1111-111111111111";
    let app = Router::new()
        .route(
            "/api/v1/sessions",
            get(move || async move {
                Json(json!([
                    {
                        "title": "most recent retained session",
                        "session_id": session_id,
                        "status": "active",
                        "coordinator_agent_id": "22222222-2222-2222-2222-222222222222",
                        "provider_kind": "openai_chatgpt",
                        "model_id": "gpt-5.4",
                        "active_workflow_id": null,
                        "last_completed_workflow_id": "33333333-3333-3333-3333-333333333333",
                        "turn_count": 3,
                        "updated_at": "2026-04-05T12:00:00Z",
                        "ended_at": null,
                        "last_preview": "resume here"
                    }
                ]))
            }),
        )
        .route(
            "/api/v1/sessions/11111111-1111-1111-1111-111111111111",
            get(|| async {
                Json(json!({
                    "title": "most recent retained session",
                    "session_id": "11111111-1111-1111-1111-111111111111",
                    "status": "active",
                    "coordinator_agent_id": "22222222-2222-2222-2222-222222222222",
                    "provider_kind": "openai_chatgpt",
                    "model_id": "gpt-5.4",
                    "active_workflow_id": null,
                    "last_completed_workflow_id": "33333333-3333-3333-3333-333333333333",
                    "turn_count": 3,
                    "last_assistant_result": null,
                    "turns": [],
                    "control_state": {
                        "selected_provider_kind": null,
                        "selected_model_id": null,
                        "permission_mode": "default",
                        "config_posture": "inline",
                        "status_view": "summary",
                        "mcp_posture": "support_only"
                    },
                    "support_notices": [],
                    "ended_at": null
                }))
            }),
        );
    let (base_url, handle) = common::spawn_mock_server(app).await;

    let output = Command::new(common::binary_path())
        .arg("--base-url")
        .arg(&base_url)
        .arg("resume")
        .arg("--last")
        .stdin(Stdio::null())
        .output()
        .await
        .expect("binary should run");

    handle.abort();

    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("session: most recent retained session"));
    assert!(stdout.contains("loop_state: ready"));
    assert!(stdout.contains("session_id: 11111111-1111-1111-1111-111111111111"));
}

#[tokio::test(flavor = "multi_thread")]
async fn sessions_list_renders_recent_rows() {
    let app = Router::new().route(
        "/api/v1/sessions",
        get(|| async {
            Json(json!([
                {
                    "title": "resume packet review",
                    "session_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "status": "active",
                    "coordinator_agent_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                    "provider_kind": "openai_chatgpt",
                    "model_id": "gpt-5.4",
                    "active_workflow_id": null,
                    "last_completed_workflow_id": null,
                    "turn_count": 2,
                    "updated_at": "2026-04-05T12:00:00Z",
                    "ended_at": null,
                    "last_preview": "check the final diff"
                }
            ]))
        }),
    );
    let (base_url, handle) = common::spawn_mock_server(app).await;

    let output = Command::new(common::binary_path())
        .arg("--base-url")
        .arg(&base_url)
        .arg("sessions")
        .arg("list")
        .stdin(Stdio::null())
        .output()
        .await
        .expect("binary should run");

    handle.abort();

    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("recent_sessions:"));
    assert!(stdout.contains("resume packet review"));
    assert!(stdout.contains("preview=check the final diff"));
}

#[tokio::test(flavor = "multi_thread")]
async fn sessions_open_renders_the_selected_session() {
    let app = Router::new().route(
        "/api/v1/sessions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        get(|| async {
            Json(json!({
                "title": "selected retained session",
                "session_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "status": "active",
                "coordinator_agent_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                "provider_kind": "openai_chatgpt",
                "model_id": "gpt-5.4",
                "active_workflow_id": null,
                "last_completed_workflow_id": "cccccccc-cccc-cccc-cccc-cccccccccccc",
                "turn_count": 5,
                "last_assistant_result": null,
                "turns": [],
                "control_state": {
                    "selected_provider_kind": null,
                    "selected_model_id": "gpt-5.4-mini",
                    "permission_mode": "review",
                    "config_posture": "inline",
                    "status_view": "summary",
                    "mcp_posture": "support_only"
                },
                "support_notices": [
                    {
                        "notice_kind": "runtime_unavailable",
                        "severity": "warning",
                        "summary": "Runtime is unavailable. This session is shown from durable storage only.",
                        "support_surface": "run"
                    }
                ],
                "ended_at": null
            }))
        }),
    );
    let (base_url, handle) = common::spawn_mock_server(app).await;

    let output = Command::new(common::binary_path())
        .arg("--base-url")
        .arg(&base_url)
        .arg("sessions")
        .arg("open")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .stdin(Stdio::null())
        .output()
        .await
        .expect("binary should run");

    handle.abort();

    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("session: selected retained session"));
    assert!(stdout.contains("current_turn:\n  none"));
    assert!(stdout.contains("selected_model: gpt-5.4-mini"));
    assert!(stdout.contains("permission_mode: review"));
}

#[tokio::test(flavor = "multi_thread")]
async fn resume_by_session_id_renders_the_selected_session() {
    let app = Router::new().route(
        "/api/v1/sessions/aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        get(|| async {
            Json(json!({
                "title": "resume specific retained session",
                "session_id": "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                "status": "active",
                "coordinator_agent_id": "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                "provider_kind": "openai_chatgpt",
                "model_id": "gpt-5.4",
                "active_workflow_id": null,
                "last_completed_workflow_id": null,
                "turn_count": 4,
                "last_assistant_result": null,
                "turns": [],
                "control_state": {
                    "selected_provider_kind": null,
                    "selected_model_id": null,
                    "permission_mode": "default",
                    "config_posture": "inline",
                    "status_view": "summary",
                    "mcp_posture": "support_only"
                },
                "support_notices": [],
                "ended_at": null
            }))
        }),
    );
    let (base_url, handle) = common::spawn_mock_server(app).await;

    let output = Command::new(common::binary_path())
        .arg("--base-url")
        .arg(&base_url)
        .arg("resume")
        .arg("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa")
        .stdin(Stdio::null())
        .output()
        .await
        .expect("binary should run");

    handle.abort();

    assert!(output.status.success(), "{:?}", output);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("session: resume specific retained session"));
    assert!(stdout.contains("loop_state: ready"));
    assert!(stdout.contains("session_id: aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"));
}
