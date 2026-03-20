//! HTTP transport server lifecycle management.
//!
//! Provides [`AppState`] (shared state for all handlers) and the [`start`]
//! function that composes the router with middleware and starts the Axum server
//! with graceful shutdown.

use async_trait::async_trait;
use axum::middleware as axum_mw;
use axum::Router;
use chrono::{DateTime, Utc};
use mister_smith_core::{
    AgentId, AgentType, ExternalDelegationEnvelope, SessionId, SessionRetainedResultView,
    SessionStatus, TaskId,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::broadcast;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

use crate::config::HttpTransportConfig;
use crate::middleware::{
    rate_limit_middleware, request_id_middleware, security_headers_middleware, security_middleware,
    RateLimiter,
};
use crate::routes::{protected_api_router, public_router};
use crate::websocket::WsEvent;

/// Default broadcast channel capacity for WebSocket events.
const EVENT_CHANNEL_CAPACITY: usize = 1024;

/// Runtime health interface for the transport dependency backing this HTTP server.
pub trait TransportHealth: Send + Sync {
    /// Return true when the underlying transport is connected and serving traffic.
    fn is_connected(&self) -> bool;
}

/// Submission request passed from HTTP handlers into the runtime task service.
#[derive(Debug, Clone)]
pub struct TaskSubmissionRequest {
    /// Goal or description submitted by the operator.
    pub description: String,
    /// Optional requested agent type.
    pub agent_type: Option<AgentType>,
    /// Optional priority label.
    pub priority: Option<String>,
    /// Optional retained same-agent conversation context.
    pub conversation: Option<ConversationTurnContext>,
    /// Delegated authority preserved from the external transport boundary, when any.
    pub delegation: Option<ExternalDelegationEnvelope>,
}

/// Submission response returned by the runtime task service.
#[derive(Debug, Clone)]
pub struct TaskSubmissionResponse {
    /// Stable task and workflow identifier.
    pub task_id: TaskId,
    /// Coordinator agent selected for the run.
    pub assigned_agent_id: AgentId,
    /// Current root task status.
    pub status: String,
}

/// Point-in-time task view returned by the runtime task service.
#[derive(Debug, Clone)]
pub struct TaskStatusView {
    /// Stable task identifier.
    pub task_id: TaskId,
    /// Current persisted status.
    pub status: String,
    /// Final result payload when the task is terminal.
    pub result: Option<serde_json::Value>,
}

/// Runtime execution contract for task submission and lookup.
#[async_trait]
pub trait TaskExecutionService: Send + Sync {
    /// Submit a new task for runtime-backed execution.
    async fn submit_task(
        &self,
        request: TaskSubmissionRequest,
    ) -> Result<TaskSubmissionResponse, String>;

    /// Look up the latest task state by its stable identifier.
    async fn get_task(&self, task_id: TaskId) -> Result<Option<TaskStatusView>, String>;
}

/// Session context attached to a workflow submission.
#[derive(Debug, Clone)]
pub struct ConversationTurnContext {
    /// Stable conversation identifier.
    pub session_id: SessionId,
    /// Accepted turn index within the session.
    pub turn_index: u32,
    /// Stable coordinator reused across accepted turns in the session.
    pub coordinator_agent_id: AgentId,
    /// Persisted retained context assembled from prior turns.
    pub retained_context: serde_json::Value,
}

/// Create-session request passed from HTTP handlers into the runtime session service.
#[derive(Debug, Clone)]
pub struct ConversationCreateRequest {
    /// Operator message for the first turn.
    pub message: String,
    /// Optional priority label for the turn workflow.
    pub priority: Option<String>,
    /// Delegated authority preserved from the external transport boundary, when any.
    pub delegation: Option<ExternalDelegationEnvelope>,
}

/// Continue-session request passed from HTTP handlers into the runtime session service.
#[derive(Debug, Clone)]
pub struct ConversationContinueRequest {
    /// Session being continued.
    pub session_id: SessionId,
    /// Operator message for the next accepted turn.
    pub message: String,
    /// Optional priority label for the new turn workflow.
    pub priority: Option<String>,
    /// Delegated authority preserved from the external transport boundary, when any.
    pub delegation: Option<ExternalDelegationEnvelope>,
}

/// Accepted conversation turn returned by create and continue operations.
#[derive(Debug, Clone)]
pub struct ConversationTurnAccepted {
    /// Stable session identifier.
    pub session_id: SessionId,
    /// Root workflow created for the accepted turn.
    pub workflow_id: TaskId,
    /// Stable coordinator identity reused across the session.
    pub coordinator_agent_id: AgentId,
    /// 1-based accepted turn order.
    pub turn_index: u32,
    /// Current root workflow status.
    pub status: String,
}

/// Ordered summary of one persisted turn in a session.
#[derive(Debug, Clone)]
pub struct ConversationTurnSummaryView {
    /// 1-based accepted turn order.
    pub turn_index: u32,
    /// Root workflow for the turn.
    pub workflow_id: TaskId,
    /// Current turn status mirrored from the root workflow.
    pub status: String,
    /// Original operator message for the turn.
    pub user_message: String,
    /// Retained session-facing result projection for the turn, when available.
    pub assistant_result: Option<SessionRetainedResultView>,
    /// Restart and resume provenance derived from workflow metadata when available.
    pub resume_provenance: Option<ConversationResumeProvenanceView>,
}

/// Restart and resume provenance for one session turn.
#[derive(Debug, Clone)]
pub struct ConversationResumeProvenanceView {
    /// Workflow record was recovered after a runtime restart.
    pub recovered_after_restart: bool,
    /// Turn resumes after a prior workflow was restart-recovered.
    pub resumed_after_restart: bool,
    /// Timestamp recorded when the workflow was marked recovered.
    pub recovered_at: Option<DateTime<Utc>>,
    /// Human-readable recovery reason recorded in workflow metadata.
    pub recovery_reason: Option<String>,
    /// Prior workflow in the resumed turn lineage, when available.
    pub resumed_from_workflow_id: Option<TaskId>,
    /// Prior turn index in the resumed turn lineage, when available.
    pub resumed_from_turn_index: Option<u32>,
}

/// Operator-facing inspect view for a conversation session.
#[derive(Debug, Clone)]
pub struct ConversationSessionView {
    /// Stable session identifier.
    pub session_id: SessionId,
    /// Session lifecycle state.
    pub status: SessionStatus,
    /// Stable coordinator identity.
    pub coordinator_agent_id: AgentId,
    /// Provider currently attributed to the session.
    pub provider_kind: String,
    /// Model currently attributed to the session.
    pub model_id: String,
    /// Active root workflow when the session is busy.
    pub active_workflow_id: Option<TaskId>,
    /// Most recent completed or failed root workflow.
    pub last_completed_workflow_id: Option<TaskId>,
    /// Number of accepted turns.
    pub turn_count: u32,
    /// Most recent retained session-facing result projection.
    pub last_assistant_result: Option<SessionRetainedResultView>,
    /// Ordered turn summaries.
    pub turns: Vec<ConversationTurnSummaryView>,
    /// Logical close time when ended.
    pub ended_at: Option<DateTime<Utc>>,
}

/// Operator-facing response after logically ending a session.
#[derive(Debug, Clone)]
pub struct ConversationEndView {
    /// Stable session identifier.
    pub session_id: SessionId,
    /// Updated lifecycle state.
    pub status: SessionStatus,
    /// Time the session was ended.
    pub ended_at: DateTime<Utc>,
}

/// Typed errors for the conversation session surface.
#[derive(Debug, Clone, Error)]
pub enum ConversationServiceError {
    /// The request was syntactically or semantically invalid.
    #[error("{0}")]
    BadRequest(String),
    /// The requested session does not exist.
    #[error("session {session_id} not found")]
    NotFound {
        /// Missing session identifier.
        session_id: SessionId,
    },
    /// The session is already handling another active turn.
    #[error("session {session_id} is busy with workflow {active_workflow_id}")]
    SessionBusy {
        /// Busy session identifier.
        session_id: SessionId,
        /// Workflow currently occupying the session.
        active_workflow_id: TaskId,
    },
    /// The session has already been logically ended.
    #[error("session {session_id} has ended")]
    SessionEnded {
        /// Ended session identifier.
        session_id: SessionId,
    },
    /// An internal runtime or persistence error occurred.
    #[error("{0}")]
    Internal(String),
}

/// Runtime execution contract for durable conversation sessions.
#[async_trait]
pub trait ConversationSessionService: Send + Sync {
    /// Create a new session and accept its first turn.
    async fn create_session(
        &self,
        request: ConversationCreateRequest,
    ) -> Result<ConversationTurnAccepted, ConversationServiceError>;

    /// Accept a new turn in an existing session.
    async fn continue_session(
        &self,
        request: ConversationContinueRequest,
    ) -> Result<ConversationTurnAccepted, ConversationServiceError>;

    /// Inspect the durable state and ordered lineage of a session.
    async fn get_session(
        &self,
        session_id: SessionId,
    ) -> Result<Option<ConversationSessionView>, ConversationServiceError>;

    /// Logically end an idle session.
    async fn end_session(
        &self,
        session_id: SessionId,
    ) -> Result<ConversationEndView, ConversationServiceError>;
}

/// NATS transport health check implementation backed by an atomic connection flag.
#[derive(Debug, Default)]
pub struct NatsHealthCheck {
    connected: AtomicBool,
}

impl NatsHealthCheck {
    /// Build a new check with explicit initial connectivity.
    pub fn new(connected: bool) -> Self {
        Self {
            connected: AtomicBool::new(connected),
        }
    }

    /// Update the observed NATS connectivity.
    pub fn set_connected(&self, connected: bool) {
        self.connected.store(connected, Ordering::Relaxed);
    }
}

impl TransportHealth for NatsHealthCheck {
    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
}

/// Shared application state accessible by all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Broadcast sender for WebSocket events.
    pub event_tx: broadcast::Sender<WsEvent>,
    /// Transport health dependency used by readiness/liveness handlers.
    pub transport_health: Arc<dyn TransportHealth>,
    /// Optional runtime-backed task submission service.
    pub task_service: Option<Arc<dyn TaskExecutionService>>,
    /// Optional runtime-backed conversation session service.
    pub conversation_service: Option<Arc<dyn ConversationSessionService>>,
    /// Optional security layer for JWT authentication.
    #[cfg(feature = "security")]
    pub security: Option<Arc<mister_smith_security::middleware::SecurityLayer>>,
}

impl AppState {
    /// Create a new `AppState` with default settings.
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self {
            event_tx,
            transport_health: Arc::new(NatsHealthCheck::new(true)),
            task_service: None,
            conversation_service: None,
            #[cfg(feature = "security")]
            security: None,
        }
    }

    /// Create a new `AppState` with a custom event channel capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let (event_tx, _) = broadcast::channel(capacity);
        Self {
            event_tx,
            transport_health: Arc::new(NatsHealthCheck::new(true)),
            task_service: None,
            conversation_service: None,
            #[cfg(feature = "security")]
            security: None,
        }
    }

    /// Set a custom transport health checker implementation.
    pub fn with_transport_health(mut self, transport_health: Arc<dyn TransportHealth>) -> Self {
        self.transport_health = transport_health;
        self
    }

    /// Set the runtime-backed task execution service.
    pub fn with_task_service(mut self, task_service: Arc<dyn TaskExecutionService>) -> Self {
        self.task_service = Some(task_service);
        self
    }

    /// Set the runtime-backed conversation session service.
    pub fn with_conversation_service(
        mut self,
        conversation_service: Arc<dyn ConversationSessionService>,
    ) -> Self {
        self.conversation_service = Some(conversation_service);
        self
    }

    /// Set the security layer for JWT authentication enforcement.
    #[cfg(feature = "security")]
    pub fn with_security(
        mut self,
        security: Arc<mister_smith_security::middleware::SecurityLayer>,
    ) -> Self {
        self.security = Some(security);
        self
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the full application router with all routes and middleware.
pub fn build_router(config: &HttpTransportConfig, state: AppState) -> Router {
    let rate_limiter = Arc::new(RateLimiter::new(config.rate_limit_rps));

    // Axum executes layers in reverse declaration order (last = outermost = first).
    // Keep health public, but preserve the shared request ID, CORS, and rate-limit stack.
    // Rate limiting must remain outermost to block floods of unauthenticated requests.
    let router = public_router()
        .merge(protected_api_router().layer(axum_mw::from_fn(security_middleware)))
        .layer(axum_mw::from_fn(security_headers_middleware))
        .layer(axum_mw::from_fn(request_id_middleware));

    // Configure CORS based on allowed_origins.
    let router = if !config.allowed_origins.is_empty() {
        let allow_origin = if config.allowed_origins.contains(&"*".to_string()) {
            AllowOrigin::any()
        } else {
            let origins: Vec<_> = config
                .allowed_origins
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            AllowOrigin::list(origins)
        };

        let cors = CorsLayer::new()
            .allow_origin(allow_origin)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any);

        router.layer(cors)
    } else {
        router
    };

    let router = router
        .layer(axum_mw::from_fn(rate_limit_middleware))
        .layer(axum::Extension(rate_limiter));

    // Inject SecurityLayer into extensions when available.
    #[cfg(feature = "security")]
    let router = if let Some(ref security) = state.security {
        router.layer(axum::Extension(security.clone()))
    } else {
        router
    };

    router.with_state(state)
}

/// Start the HTTP transport server.
///
/// Binds to the configured address, composes all routes and middleware,
/// and runs until a shutdown signal is received.
pub async fn start(
    config: HttpTransportConfig,
    state: AppState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = build_router(&config, state);

    let listener = tokio::net::TcpListener::bind(&config.bind_address).await?;
    info!(address = %config.bind_address, "HTTP server listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    info!("HTTP server shut down");
    Ok(())
}

/// Wait for a shutdown signal (Ctrl+C).
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
    info!("Shutdown signal received");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::extract::ConnectInfo;
    use axum::http::{Request, StatusCode};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use tower::ServiceExt;

    #[cfg(feature = "security")]
    use std::time::Duration;

    #[cfg(feature = "security")]
    use mister_smith_core::{AuthorityPrincipal, DelegationScope};
    #[cfg(feature = "security")]
    use mister_smith_security::config::{AuditConfig, JwtConfig, KeySource, RbacConfig};
    #[cfg(feature = "security")]
    use mister_smith_security::jwt::AgentClaims;
    #[cfg(feature = "security")]
    use mister_smith_security::middleware::{SecurityLayer, SecurityLayerConfig};

    #[cfg(feature = "security")]
    fn test_security_layer() -> Arc<SecurityLayer> {
        Arc::new(
            SecurityLayer::new(SecurityLayerConfig {
                enabled: true,
                auth_enabled: true,
                authz_enabled: true,
                audit_enabled: true,
                tls_enabled: false,
                jwt_config: Some(JwtConfig {
                    algorithm: "HS256".to_string(),
                    access_token_ttl: Duration::from_secs(300),
                    refresh_token_ttl: Duration::from_secs(3_600),
                    issuer: None,
                    audience: Vec::new(),
                    delegation_chain_max_depth: 5,
                    key_source: KeySource::Hmac {
                        secret: b"http-server-test-secret-key-at-least-32-bytes!".to_vec(),
                    },
                }),
                rbac_config: Some(RbacConfig::default()),
                audit_config: Some(AuditConfig::default()),
                tls_config: None,
            })
            .expect("test security layer should initialize"),
        )
    }

    #[cfg(feature = "security")]
    fn delegation_test_security_layer() -> Arc<SecurityLayer> {
        Arc::new(
            SecurityLayer::new(SecurityLayerConfig {
                enabled: true,
                auth_enabled: true,
                authz_enabled: false,
                audit_enabled: true,
                tls_enabled: false,
                jwt_config: Some(JwtConfig {
                    algorithm: "HS256".to_string(),
                    access_token_ttl: Duration::from_secs(300),
                    refresh_token_ttl: Duration::from_secs(3_600),
                    issuer: Some("mister-smith-http-tests".to_string()),
                    audience: vec!["http-tests".to_string()],
                    delegation_chain_max_depth: 5,
                    key_source: KeySource::Hmac {
                        secret: b"http-server-delegation-test-secret-key-32-bytes!".to_vec(),
                    },
                }),
                rbac_config: Some(RbacConfig::default()),
                audit_config: Some(AuditConfig::default()),
                tls_config: None,
            })
            .expect("delegation security layer should initialize"),
        )
    }

    #[cfg(feature = "security")]
    fn http_delegation_descriptor(method: &str, route: &str) -> String {
        format!("http:{}:{route}", method.to_ascii_lowercase())
    }

    #[cfg(feature = "security")]
    fn delegated_bearer_token(
        security: &Arc<SecurityLayer>,
        method: &str,
        route: &str,
    ) -> (String, mister_smith_core::CapabilityId, String) {
        let recipient = AgentId::from_uuid(uuid::Uuid::new_v4());
        let delegation_service = security
            .delegation_service
            .as_ref()
            .expect("delegation service should be configured");
        let descriptor_id = http_delegation_descriptor(method, route);
        let permission = format!("{}:{route}:{route}", method.to_ascii_lowercase());
        let (capability, provenance) = delegation_service
            .issue_capability(
                AuthorityPrincipal::Policy("operator".to_string()),
                recipient,
                DelegationScope::InvokeTool,
                Some(descriptor_id.clone()),
                Duration::from_secs(300),
                None,
                None,
            )
            .expect("delegation should issue");

        let claims = AgentClaims {
            iss: Some("mister-smith-http-tests".to_string()),
            sub: recipient.to_string(),
            aud: vec!["http-tests".to_string()],
            agent_id: recipient.to_string(),
            agent_type: "worker".to_string(),
            permissions: vec![permission],
            delegation_capability: Some(capability.clone()),
            provenance_chain: Some(provenance),
            ..Default::default()
        };

        let token = security
            .jwt
            .as_ref()
            .expect("jwt manager should be configured")
            .generate_token_pair(&claims)
            .expect("token generation should succeed")
            .access_token;

        (
            token,
            capability.capability_id,
            format!("{descriptor_id}#execute"),
        )
    }

    #[test]
    fn app_state_default() {
        let state = AppState::default();
        // Verify we can subscribe (channel is functional).
        let _rx = state.event_tx.subscribe();
    }

    #[test]
    fn app_state_with_capacity() {
        let state = AppState::with_capacity(64);
        let _rx = state.event_tx.subscribe();
    }

    #[test]
    fn nats_health_check_tracks_connectivity() {
        let check = NatsHealthCheck::new(true);
        assert!(check.is_connected());
        check.set_connected(false);
        assert!(!check.is_connected());
    }

    #[test]
    fn build_router_does_not_panic() {
        let config = HttpTransportConfig::default();
        let state = AppState::new();
        let _router = build_router(&config, state);
    }

    #[tokio::test]
    async fn event_broadcast_through_state() {
        let state = AppState::new();
        let mut rx = state.event_tx.subscribe();

        let event = WsEvent {
            event_type: "test".to_string(),
            payload: serde_json::json!({"key": "value"}),
            timestamp: "2026-03-04T00:00:00Z".to_string(),
        };

        state.event_tx.send(event.clone()).unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.event_type, "test");
    }

    #[tokio::test]
    async fn build_router_rate_limits_repeated_requests() {
        let mut config = HttpTransportConfig::default();
        config.rate_limit_rps = 2;

        let app = build_router(&config, AppState::new());
        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40123);

        for _ in 0..2 {
            let request = Request::builder()
                .uri("/api/v1/health")
                .extension(ConnectInfo(client_addr))
                .body(Body::empty())
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        let request = Request::builder()
            .uri("/api/v1/health")
            .extension(ConnectInfo(client_addr))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn build_router_includes_cors_headers_when_configured() {
        let mut config = HttpTransportConfig::default();
        config.allowed_origins = vec!["*".to_string()];
        let app = build_router(&config, AppState::new());
        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40124);

        let request = Request::builder()
            .method("OPTIONS")
            .uri("/api/v1/health")
            .header("origin", "http://example.com")
            .header("access-control-request-method", "GET")
            .extension(ConnectInfo(client_addr))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response
                .headers()
                .get("access-control-allow-origin")
                .unwrap(),
            "*"
        );
    }

    #[tokio::test]
    async fn build_router_excludes_cors_headers_by_default() {
        let config = HttpTransportConfig::default();
        let app = build_router(&config, AppState::new());
        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40125);

        let request = Request::builder()
            .method("OPTIONS")
            .uri("/api/v1/health")
            .header("origin", "http://example.com")
            .header("access-control-request-method", "GET")
            .extension(ConnectInfo(client_addr))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Without CORS middleware, OPTIONS might 405 or 404 depending on router,
        // or just return OK but without CORS headers.
        // In Axum, `any(ws_handler)` at the end might catch it, or it might be a 405.
        // The main point is checking for the header.
        assert!(response
            .headers()
            .get("access-control-allow-origin")
            .is_none());
    }

    #[cfg(feature = "security")]
    #[tokio::test]
    async fn build_router_keeps_health_public_when_security_enabled() {
        let config = HttpTransportConfig::default();
        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40126);
        let app = build_router(
            &config,
            AppState::new().with_security(test_security_layer()),
        );

        let health_request = Request::builder()
            .uri("/api/v1/health")
            .extension(ConnectInfo(client_addr))
            .body(Body::empty())
            .unwrap();
        let health_response = app.clone().oneshot(health_request).await.unwrap();
        assert_eq!(health_response.status(), StatusCode::OK);

        let agents_request = Request::builder()
            .uri("/api/v1/agents")
            .extension(ConnectInfo(client_addr))
            .body(Body::empty())
            .unwrap();
        let agents_response = app.oneshot(agents_request).await.unwrap();
        assert_eq!(agents_response.status(), StatusCode::UNAUTHORIZED);
    }

    #[cfg(feature = "security")]
    #[tokio::test]
    async fn build_router_rejects_revoked_delegation_capability() {
        let config = HttpTransportConfig::default();
        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40127);
        let security = delegation_test_security_layer();
        let (token, capability_id, _) = delegated_bearer_token(&security, "GET", "/api/v1/agents");
        security
            .delegation_service
            .as_ref()
            .expect("delegation service should be configured")
            .revoke_capability(capability_id);

        let app = build_router(&config, AppState::new().with_security(security));

        let request = Request::builder()
            .uri("/api/v1/agents")
            .header("authorization", format!("Bearer {token}"))
            .extension(ConnectInfo(client_addr))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[cfg(feature = "security")]
    #[tokio::test]
    async fn build_router_rejects_delegation_bound_to_different_route() {
        let config = HttpTransportConfig::default();
        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40128);
        let security = delegation_test_security_layer();
        let (token, _, _) = delegated_bearer_token(&security, "POST", "/api/v1/tasks");

        let app = build_router(&config, AppState::new().with_security(security));

        let request = Request::builder()
            .uri("/api/v1/agents")
            .header("authorization", format!("Bearer {token}"))
            .extension(ConnectInfo(client_addr))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[cfg(feature = "security")]
    #[tokio::test]
    async fn build_router_rejects_revoked_delegation_action_for_route() {
        let config = HttpTransportConfig::default();
        let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40129);
        let security = delegation_test_security_layer();
        let (token, _, revocation_key) = delegated_bearer_token(&security, "GET", "/api/v1/agents");
        security
            .delegation_service
            .as_ref()
            .expect("delegation service should be configured")
            .revoke_action(&revocation_key);

        let app = build_router(&config, AppState::new().with_security(security));

        let request = Request::builder()
            .uri("/api/v1/agents")
            .header("authorization", format!("Bearer {token}"))
            .extension(ConnectInfo(client_addr))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
