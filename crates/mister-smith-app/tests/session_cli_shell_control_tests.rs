#![allow(dead_code)]

#[path = "../src/auth.rs"]
mod auth;
#[path = "../src/autonomy.rs"]
mod autonomy;
#[path = "../src/conversation.rs"]
mod conversation;
#[path = "../src/execution.rs"]
mod execution;
#[path = "../src/observability.rs"]
mod observability;

use axum::{routing::post, Json, Router};
use mister_smith_core::DurableWorkflowLifecycleState;
use serde_json::json;
use tokio::net::TcpListener;

async fn spawn_mock_server(app: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{}", address), handle)
}

#[tokio::test(flavor = "multi_thread")]
async fn session_control_updates_round_trip_through_http_helper() {
    let app = Router::new().route(
        "/api/v1/sessions/11111111-1111-1111-1111-111111111111/controls",
        post(|| async {
            Json(json!({
                "selected_provider_kind": "openai_chatgpt",
                "selected_model_id": "gpt-5.4-mini",
                "permission_mode": "review",
                "config_posture": "inline",
                "status_view": "detail",
                "mcp_posture": "connected"
            }))
        }),
    );
    let (base_url, handle) = spawn_mock_server(app).await;
    let session_id = conversation::parse_session_id("11111111-1111-1111-1111-111111111111")
        .expect("session id should parse");

    let control = conversation::update_session_control_http(
        &base_url,
        session_id,
        mister_smith_http::server::ConversationSessionControlUpdateRequest {
            selected_provider_kind: Some("openai_chatgpt".to_string()),
            selected_model_id: Some("gpt-5.4-mini".to_string()),
            permission_mode: Some("review".to_string()),
            config_posture: Some("inline".to_string()),
            status_view: Some("detail".to_string()),
            mcp_posture: Some("connected".to_string()),
            clear_selected_provider_kind: false,
            clear_selected_model_id: false,
        },
    )
    .await
    .expect("control update should succeed");

    handle.abort();

    assert_eq!(control.selected_model_id.as_deref(), Some("gpt-5.4-mini"));
    assert_eq!(control.permission_mode, "review");
    assert_eq!(control.status_view, "detail");
    assert_eq!(control.mcp_posture, "connected");
}

#[test]
fn render_session_surfaces_control_state_and_support_notices() {
    let view = conversation::ConversationCliSessionView {
        title: "live control session".to_string(),
        session_id: "11111111-1111-1111-1111-111111111111".to_string(),
        status: "active".to_string(),
        loop_state: "ready".to_string(),
        coordinator_agent_id: "22222222-2222-2222-2222-222222222222".to_string(),
        provider_kind: "openai_chatgpt".to_string(),
        model_id: "gpt-5.4".to_string(),
        active_workflow_id: None,
        last_completed_workflow_id: Some("33333333-3333-3333-3333-333333333333".to_string()),
        turn_count: 2,
        last_assistant_result: None,
        current_turn_state: Some(conversation::ConversationCliCurrentTurnStateView {
            workflow_id: "33333333-3333-3333-3333-333333333333".to_string(),
            turn_index: 2,
            turn_status: "completed".to_string(),
            lifecycle_state: DurableWorkflowLifecycleState::Completed,
            result_preview: Some("control state captured".to_string()),
            proof_boundary_note: Some(
                "result is orchestration proof, not substantive task proof".to_string(),
            ),
            state_source: "retained_session".to_string(),
            next_action_hint: "send a follow-up turn or adjust the session controls".to_string(),
        }),
        turns: vec![conversation::ConversationCliTurnSummary {
            turn_index: 2,
            workflow_id: "33333333-3333-3333-3333-333333333333".to_string(),
            status: "completed".to_string(),
            lifecycle_state: DurableWorkflowLifecycleState::Completed,
            user_message: "show the control state".to_string(),
            assistant_result: None,
            resume_provenance: None,
        }],
        control_state: conversation::ConversationCliSessionControlState {
            selected_provider_kind: Some("openai_chatgpt".to_string()),
            selected_model_id: Some("gpt-5.4-mini".to_string()),
            permission_mode: "review".to_string(),
            config_posture: "inline".to_string(),
            status_view: "detail".to_string(),
            mcp_posture: "connected".to_string(),
        },
        support_notices: vec![conversation::ConversationCliSupportNoticeView {
            notice_kind: "session_shell_preferences".to_string(),
            severity: "warning".to_string(),
            summary: "Shell control changes are stored with this session, but runtime execution still follows the active runtime path.".to_string(),
            support_surface: Some("config".to_string()),
            blocks_live_turn: false,
            allowed_next_action:
                "keep working in this session or adjust the support posture".to_string(),
        }],
        next_action_hint: "send a follow-up turn or adjust the session controls".to_string(),
        ended_at: None,
    };

    let rendered = conversation::render_session(&view);

    assert!(rendered.contains("session: live control session"));
    assert!(rendered.contains("loop_state: ready"));
    assert!(rendered.contains("selected_model: gpt-5.4-mini"));
    assert!(rendered.contains("permission_mode: review"));
    assert!(rendered.contains("[warning] Shell control changes are stored with this session"));
}
