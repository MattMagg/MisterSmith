//! HTTP transport server lifecycle management.
//!
//! Provides [`AppState`] (shared state for all handlers) and the [`start`]
//! function that composes the router with middleware and starts the Axum server
//! with graceful shutdown.

use axum::middleware as axum_mw;
use axum::Router;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

use crate::config::HttpTransportConfig;
use crate::middleware::{request_id_middleware, security_middleware, RateLimiter};
use crate::routes::api_router;
use crate::websocket::WsEvent;

/// Default broadcast channel capacity for WebSocket events.
const EVENT_CHANNEL_CAPACITY: usize = 1024;

/// Shared application state accessible by all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Broadcast sender for WebSocket events.
    pub event_tx: broadcast::Sender<WsEvent>,
}

impl AppState {
    /// Create a new `AppState` with default settings.
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Self { event_tx }
    }

    /// Create a new `AppState` with a custom event channel capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let (event_tx, _) = broadcast::channel(capacity);
        Self { event_tx }
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

    api_router()
        .layer(axum_mw::from_fn(request_id_middleware))
        .layer(axum_mw::from_fn(security_middleware))
        .layer(axum::Extension(rate_limiter))
        .with_state(state)
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
}
