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

use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
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

#[tokio::test(flavor = "multi_thread")]
async fn inspect_session_for_cli_surfaces_http_status_errors() {
    let app = Router::new().route(
        "/api/v1/sessions/11111111-1111-1111-1111-111111111111",
        get(|| async {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "missing session" })),
            )
        }),
    );
    let (base_url, handle) = spawn_mock_server(app).await;
    let session_id = conversation::parse_session_id("11111111-1111-1111-1111-111111111111")
        .expect("session id should parse");

    let error = conversation::inspect_session_for_cli(
        &base_url,
        &mister_smith_config::FrameworkConfig::default(),
        session_id,
    )
    .await
    .expect_err("status failure should not fall back to direct store");

    handle.abort();

    assert!(matches!(
        error,
        conversation::ConversationClientError::HttpStatus(StatusCode::NOT_FOUND, _)
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn update_session_control_for_cli_surfaces_http_status_errors() {
    let app = Router::new().route(
        "/api/v1/sessions/11111111-1111-1111-1111-111111111111/controls",
        post(|| async {
            (
                StatusCode::FORBIDDEN,
                Json(json!({ "error": "controls are forbidden" })),
            )
        }),
    );
    let (base_url, handle) = spawn_mock_server(app).await;
    let session_id = conversation::parse_session_id("11111111-1111-1111-1111-111111111111")
        .expect("session id should parse");

    let error = conversation::update_session_control_for_cli(
        &base_url,
        &mister_smith_config::FrameworkConfig::default(),
        session_id,
        mister_smith_http::server::ConversationSessionControlUpdateRequest {
            status_view: Some("detail".to_string()),
            ..Default::default()
        },
    )
    .await
    .expect_err("status failure should not fall back to direct store");

    handle.abort();

    assert!(matches!(
        error,
        conversation::ConversationClientError::HttpStatus(StatusCode::FORBIDDEN, _)
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn build_startup_home_surfaces_session_discovery_status_errors() {
    let app = Router::new()
        .route(
            "/api/v1/health",
            get(|| async { Json(json!({ "status": "healthy", "components": [] })) }),
        )
        .route(
            "/api/v1/sessions",
            get(|| async {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "session list denied" })),
                )
            }),
        );
    let (base_url, handle) = spawn_mock_server(app).await;

    let home = conversation::build_startup_home(
        &base_url,
        &mister_smith_config::FrameworkConfig::default(),
        "config loaded from defaults".to_string(),
        8,
    )
    .await;

    handle.abort();

    assert!(home.runtime_available);
    assert_eq!(home.session_source, "runtime_api_error");
    assert!(home.recent_sessions.is_empty());
    assert!(home
        .startup_warnings
        .iter()
        .any(|notice| notice.notice_kind == "session_discovery_failed"
            && notice.summary.contains("runtime returned 401")));
    assert!(!home
        .startup_warnings
        .iter()
        .any(|notice| notice.notice_kind == "no_recent_sessions"));
}

#[test]
fn render_session_surfaces_control_state_and_support_notices() {
    let view = conversation::ConversationCliSessionView {
        title: "live control session".to_string(),
        session_id: "11111111-1111-1111-1111-111111111111".to_string(),
        status: "active".to_string(),
        coordinator_agent_id: "22222222-2222-2222-2222-222222222222".to_string(),
        provider_kind: "openai_chatgpt".to_string(),
        model_id: "gpt-5.4".to_string(),
        active_workflow_id: None,
        last_completed_workflow_id: Some("33333333-3333-3333-3333-333333333333".to_string()),
        turn_count: 2,
        last_assistant_result: None,
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
        }],
        ended_at: None,
    };

    let rendered = conversation::render_session(&view);

    assert!(rendered.contains("title: live control session"));
    assert!(rendered.contains("selected_model: gpt-5.4-mini"));
    assert!(rendered.contains("permission_mode: review"));
    assert!(rendered.contains("[warning] Shell control changes are stored with this session"));
}
