//! HTTP transport server lifecycle management.
//!
//! Provides [`AppState`] (shared state for all handlers) and the [`start`]
//! function that composes the router with middleware and starts the Axum server
//! with graceful shutdown.

use axum::middleware as axum_mw;
use axum::Router;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

use crate::config::HttpTransportConfig;
use crate::middleware::{
    rate_limit_middleware, request_id_middleware, security_middleware, RateLimiter,
};
use crate::routes::api_router;
use crate::websocket::WsEvent;

/// Default broadcast channel capacity for WebSocket events.
const EVENT_CHANNEL_CAPACITY: usize = 1024;

/// Runtime health interface for the transport dependency backing this HTTP server.
pub trait TransportHealth: Send + Sync {
    /// Return true when the underlying transport is connected and serving traffic.
    fn is_connected(&self) -> bool;
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
            #[cfg(feature = "security")]
            security: None,
        }
    }

    /// Set a custom transport health checker implementation.
    pub fn with_transport_health(mut self, transport_health: Arc<dyn TransportHealth>) -> Self {
        self.transport_health = transport_health;
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
    // Rate limiting must be outermost to block floods of unauthenticated requests.
    let router = api_router()
        .layer(axum_mw::from_fn(request_id_middleware))
        .layer(axum_mw::from_fn(security_middleware));

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
}
