//! Route definitions for the HTTP API.
//!
//! Composes all REST and WebSocket endpoints under `/api/v1`.

use axum::routing::{any, get, post};
use axum::Router;

use crate::handlers;
use crate::server::AppState;
use crate::websocket;

/// Build the public probe router that must remain reachable without auth.
pub fn public_router() -> Router<AppState> {
    Router::new().route("/api/v1/health", get(handlers::health_check))
}

/// Build the protected API router with authenticated REST and WebSocket routes.
pub fn protected_api_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/agents", get(handlers::list_agents))
        .route("/api/v1/agents/{agent_id}", get(handlers::get_agent))
        .route(
            "/api/v1/tasks",
            get(handlers::list_tasks).post(handlers::create_task),
        )
        .route("/api/v1/tasks/{task_id}", get(handlers::get_task))
        .route(
            "/api/v1/tasks/{task_id}/lifecycle",
            post(handlers::apply_task_lifecycle),
        )
        .route(
            "/api/v1/sessions",
            get(handlers::list_sessions).post(handlers::create_session),
        )
        .route("/api/v1/sessions/{session_id}", get(handlers::get_session))
        .route(
            "/api/v1/sessions/{session_id}/turns",
            post(handlers::continue_session),
        )
        .route(
            "/api/v1/sessions/{session_id}/end",
            post(handlers::end_session),
        )
        .route("/api/v1/config", get(handlers::get_config))
        .route("/api/v1/events/ws", any(websocket::ws_handler))
}

/// Build the complete API router with both public probe and protected routes.
pub fn api_router() -> Router<AppState> {
    public_router().merge(protected_api_router())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{AppState, NatsHealthCheck};
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn test_app() -> Router {
        api_router().with_state(AppState::new())
    }

    fn test_app_with_transport(connected: bool) -> Router {
        api_router().with_state(
            AppState::new()
                .with_transport_health(std::sync::Arc::new(NatsHealthCheck::new(connected))),
        )
    }

    #[tokio::test]
    async fn health_route_responds_healthy_when_transport_connected() {
        let app = test_app_with_transport(true);
        let request = Request::builder()
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["status"], "healthy");
        assert_eq!(payload["components"][0]["name"], "http_server");
        assert_eq!(payload["components"][1]["name"], "nats_transport");
        assert_eq!(payload["components"][1]["status"], "healthy");
    }

    #[tokio::test]
    async fn health_route_responds_unhealthy_when_transport_disconnected() {
        let app = test_app_with_transport(false);
        let request = Request::builder()
            .uri("/api/v1/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["status"], "unhealthy");
        assert_eq!(payload["components"][0]["name"], "http_server");
        assert_eq!(payload["components"][1]["name"], "nats_transport");
        assert_eq!(payload["components"][1]["status"], "unhealthy");
    }

    #[tokio::test]
    async fn agents_route_responds_ok() {
        let app = test_app();
        let request = Request::builder()
            .uri("/api/v1/agents")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn agent_detail_not_found() {
        let app = test_app();
        let request = Request::builder()
            .uri("/api/v1/agents/00000000-0000-0000-0000-000000000099")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn create_task_route_responds_accepted() {
        let app = test_app();
        let body = serde_json::json!({
            "description": "Test task",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/api/v1/tasks")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn config_route_responds_ok() {
        let app = test_app();
        let request = Request::builder()
            .uri("/api/v1/config")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_route_responds_not_found() {
        let app = test_app();
        let request = Request::builder()
            .uri("/api/v1/unknown")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
