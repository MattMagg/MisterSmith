//! WebSocket event streaming endpoint.
//!
//! Provides real-time event streaming over WebSocket at `GET /api/v1/events/ws`.
//! Supports event filtering via query parameters, keepalive pings, and
//! client subscribe/unsubscribe messages.

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::Response;
use futures::stream::SplitSink;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, warn};

use crate::server::AppState;

/// Default keepalive ping interval.
const DEFAULT_KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

/// Query parameters for WebSocket event filtering.
#[derive(Debug, Deserialize, Default)]
pub struct WsQuery {
    /// Comma-separated list of event types to filter on.
    #[serde(default)]
    pub filter: Option<String>,
}

/// Event broadcast message sent to WebSocket clients.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEvent {
    /// Event type identifier (e.g., "agent_status", "task_progress").
    pub event_type: String,
    /// Event payload.
    pub payload: serde_json::Value,
    /// Timestamp as ISO 8601.
    pub timestamp: String,
}

/// Client-to-server WebSocket control message.
#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
enum ClientMessage {
    /// Subscribe to specific event types.
    #[serde(rename = "subscribe")]
    Subscribe { event_types: Vec<String> },
    /// Unsubscribe from specific event types.
    #[serde(rename = "unsubscribe")]
    Unsubscribe { event_types: Vec<String> },
}

/// WebSocket upgrade handler, routed with `any()` for HTTP/1.1 + HTTP/2 compatibility.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
) -> Response {
    let initial_filters = query
        .filter
        .map(|f| f.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    ws.on_upgrade(move |socket| handle_ws_connection(socket, state, initial_filters))
}

/// Handle an individual WebSocket connection.
async fn handle_ws_connection(
    socket: WebSocket,
    state: AppState,
    initial_filters: HashSet<String>,
) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(tokio::sync::Mutex::new(sender));

    let mut event_rx = state.event_tx.subscribe();
    let mut filters = initial_filters;

    // Track whether we've received a pong for the current ping cycle.
    let pong_received = Arc::new(std::sync::atomic::AtomicBool::new(true));

    // Spawn the keepalive ping task.
    let ping_sender = Arc::clone(&sender);
    let ping_pong = Arc::clone(&pong_received);
    let keepalive_handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(DEFAULT_KEEPALIVE_INTERVAL).await;

            // Check if pong was received for the previous ping.
            if !ping_pong.load(std::sync::atomic::Ordering::SeqCst) {
                debug!("WebSocket pong timeout — closing connection");
                let mut s = ping_sender.lock().await;
                let _ = s.send(Message::Close(None)).await;
                return;
            }

            // Send ping and reset pong flag.
            ping_pong.store(false, std::sync::atomic::Ordering::SeqCst);
            let mut s = ping_sender.lock().await;
            if s.send(Message::Ping(axum::body::Bytes::from_static(b"keepalive")))
                .await
                .is_err()
            {
                return;
            }
        }
    });

    // Spawn the event broadcast task.
    let event_sender = Arc::clone(&sender);
    let event_handle = tokio::spawn({
        let filters = filters.clone();
        async move {
            let filters = std::sync::Mutex::new(filters);
            loop {
                match event_rx.recv().await {
                    Ok(event) => {
                        let current_filters = filters.lock().unwrap().clone();
                        // Apply filter: if filters are empty, send all events.
                        if !current_filters.is_empty()
                            && !current_filters.contains(&event.event_type)
                        {
                            continue;
                        }

                        let json = match serde_json::to_string(&event) {
                            Ok(j) => j,
                            Err(e) => {
                                warn!(error = %e, "Failed to serialize WebSocket event");
                                continue;
                            }
                        };

                        let mut s = event_sender.lock().await;
                        if s.send(Message::text(json)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        warn!(skipped = n, "WebSocket client lagged behind broadcast");
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    });

    // Process incoming client messages.
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(text.as_str()) {
                    match client_msg {
                        ClientMessage::Subscribe { event_types } => {
                            debug!(?event_types, "Client subscribed to events");
                            filters.extend(event_types);
                        }
                        ClientMessage::Unsubscribe { event_types } => {
                            debug!(?event_types, "Client unsubscribed from events");
                            for et in &event_types {
                                filters.remove(et);
                            }
                        }
                    }
                }
            }
            Ok(Message::Pong(_)) => {
                pong_received.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            Ok(Message::Close(_)) => {
                debug!("WebSocket client sent close frame");
                break;
            }
            Err(e) => {
                debug!(error = %e, "WebSocket receive error");
                break;
            }
            _ => {}
        }
    }

    // Cleanup: abort background tasks.
    keepalive_handle.abort();
    event_handle.abort();

    // Send close frame if possible.
    let mut s = sender.lock().await;
    let _ = close_connection(&mut s).await;

    debug!("WebSocket connection closed");
}

/// Send a close frame to the WebSocket client.
async fn close_connection(sender: &mut SplitSink<WebSocket, Message>) -> Result<(), axum::Error> {
    sender.send(Message::Close(None)).await
}

/// Broadcast an event to all connected WebSocket clients.
///
/// This is a convenience function for publishing events from other parts
/// of the system.
pub fn broadcast_event(tx: &broadcast::Sender<WsEvent>, event: WsEvent) {
    // Ignore send errors (no receivers connected).
    let _ = tx.send(event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ws_event_serialization() {
        let event = WsEvent {
            event_type: "agent_status".to_string(),
            payload: serde_json::json!({"agent_id": "abc", "status": "idle"}),
            timestamp: "2026-03-04T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: WsEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_type, "agent_status");
    }

    #[test]
    fn client_subscribe_message_parsing() {
        let json = r#"{"action":"subscribe","event_types":["agent_status","task_progress"]}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Subscribe { event_types } => {
                assert_eq!(event_types.len(), 2);
                assert!(event_types.contains(&"agent_status".to_string()));
                assert!(event_types.contains(&"task_progress".to_string()));
            }
            _ => panic!("Expected Subscribe message"),
        }
    }

    #[test]
    fn client_unsubscribe_message_parsing() {
        let json = r#"{"action":"unsubscribe","event_types":["task_progress"]}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();
        match msg {
            ClientMessage::Unsubscribe { event_types } => {
                assert_eq!(event_types.len(), 1);
                assert_eq!(event_types[0], "task_progress");
            }
            _ => panic!("Expected Unsubscribe message"),
        }
    }

    #[test]
    fn ws_query_parsing_with_filter() {
        let query: WsQuery =
            serde_json::from_str(r#"{"filter": "agent_status,task_progress"}"#).unwrap();
        assert_eq!(query.filter.as_deref(), Some("agent_status,task_progress"));
    }

    #[test]
    fn ws_query_parsing_without_filter() {
        let query: WsQuery = serde_json::from_str(r#"{}"#).unwrap();
        assert!(query.filter.is_none());
    }

    #[test]
    fn broadcast_event_with_no_receivers() {
        let (tx, _) = broadcast::channel::<WsEvent>(16);
        // Should not panic even with no receivers.
        broadcast_event(
            &tx,
            WsEvent {
                event_type: "test".to_string(),
                payload: serde_json::json!({}),
                timestamp: "2026-01-01T00:00:00Z".to_string(),
            },
        );
    }

    #[test]
    fn broadcast_event_received() {
        let (tx, mut rx) = broadcast::channel::<WsEvent>(16);
        let event = WsEvent {
            event_type: "task_complete".to_string(),
            payload: serde_json::json!({"task_id": "123"}),
            timestamp: "2026-03-04T12:00:00Z".to_string(),
        };
        broadcast_event(&tx, event);
        let received = rx.try_recv().unwrap();
        assert_eq!(received.event_type, "task_complete");
    }

    #[test]
    fn filter_parsing() {
        let filter_str = "agent_status,task_progress,system_event";
        let filters: HashSet<String> = filter_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        assert_eq!(filters.len(), 3);
        assert!(filters.contains("agent_status"));
        assert!(filters.contains("task_progress"));
        assert!(filters.contains("system_event"));
    }
}
