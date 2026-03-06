//! MCP session management with lazy initialization and reconnection.
//!
//! Manages client connections on-demand: connects lazily on first tool call,
//! reconnects with backoff on failure, and cleans up on shutdown.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::client::McpClient;
use crate::config::{McpClientConfig, McpConfig};
use crate::errors::McpError;

/// Default max retry attempts for reconnection.
const DEFAULT_MAX_RETRIES: u32 = 5;
/// Default initial backoff duration.
const DEFAULT_INITIAL_BACKOFF: Duration = Duration::from_secs(1);
/// Backoff multiplier for exponential backoff.
const BACKOFF_MULTIPLIER: f64 = 2.0;
/// Maximum backoff cap.
const MAX_BACKOFF: Duration = Duration::from_secs(60);

/// Session state for a single MCP client connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Not yet connected (lazy initialization pending).
    Idle,
    /// Attempting to connect.
    Connecting,
    /// Successfully connected and ready for tool calls.
    Connected,
    /// Disconnected, may reconnect.
    Disconnected,
    /// Permanently failed after max retries.
    Failed,
}

/// Tracks per-client session metadata.
struct SessionInfo {
    client: McpClient,
    state: SessionState,
    _retry_count: u32,
    _max_retries: u32,
}

/// Manages MCP client sessions with lazy connect and reconnection.
pub struct McpSessionManager {
    /// Sessions keyed by server name.
    sessions: Arc<RwLock<HashMap<String, SessionInfo>>>,
    /// Pending configs for lazy initialization.
    pending_configs: Arc<RwLock<Vec<McpClientConfig>>>,
}

impl McpSessionManager {
    /// Create a new session manager from MCP configuration.
    pub fn new(config: &McpConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            pending_configs: Arc::new(RwLock::new(config.clients.clone())),
        }
    }

    /// Create with no configured clients (for testing).
    pub fn empty() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            pending_configs: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get or lazily create a session for the named server.
    pub async fn get_or_connect(&self, server_name: &str) -> Result<(), McpError> {
        // Check if already connected.
        {
            let sessions = self.sessions.read().await;
            if let Some(info) = sessions.get(server_name) {
                match info.state {
                    SessionState::Connected => return Ok(()),
                    SessionState::Failed => {
                        return Err(McpError::ConnectionFailed(format!(
                            "session for '{server_name}' permanently failed after max retries"
                        )));
                    }
                    _ => {}
                }
            }
        }

        // Find config for this server.
        let config = {
            let configs = self.pending_configs.read().await;
            configs
                .iter()
                .find(|c| c.name == server_name)
                .cloned()
                .ok_or_else(|| {
                    McpError::SessionError(format!("no configuration for server '{server_name}'"))
                })?
        };

        // Create and connect.
        self.connect_with_backoff(server_name, &config).await
    }

    /// Connect to a server with exponential backoff retries.
    async fn connect_with_backoff(
        &self,
        server_name: &str,
        config: &McpClientConfig,
    ) -> Result<(), McpError> {
        let client = McpClient::new(config.clone());
        let mut retry_count = 0u32;
        let mut backoff = DEFAULT_INITIAL_BACKOFF;

        loop {
            // Update state to connecting.
            {
                let mut sessions = self.sessions.write().await;
                sessions.insert(
                    server_name.to_string(),
                    SessionInfo {
                        client: McpClient::new(config.clone()),
                        state: SessionState::Connecting,
                        _retry_count: retry_count,
                        _max_retries: DEFAULT_MAX_RETRIES,
                    },
                );
            }

            match client.connect().await {
                Ok(()) => {
                    let mut sessions = self.sessions.write().await;
                    sessions.insert(
                        server_name.to_string(),
                        SessionInfo {
                            client: McpClient::new(config.clone()),
                            state: SessionState::Connected,
                            _retry_count: retry_count,
                            _max_retries: DEFAULT_MAX_RETRIES,
                        },
                    );
                    return Ok(());
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= DEFAULT_MAX_RETRIES {
                        let mut sessions = self.sessions.write().await;
                        sessions.insert(
                            server_name.to_string(),
                            SessionInfo {
                                client: McpClient::new(config.clone()),
                                state: SessionState::Failed,
                                _retry_count: retry_count,
                                _max_retries: DEFAULT_MAX_RETRIES,
                            },
                        );
                        return Err(McpError::ConnectionFailed(format!(
                            "failed after {retry_count} retries: {e}"
                        )));
                    }
                    tokio::time::sleep(backoff).await;
                    backoff = Duration::from_secs_f64(
                        (backoff.as_secs_f64() * BACKOFF_MULTIPLIER).min(MAX_BACKOFF.as_secs_f64()),
                    );
                }
            }
        }
    }

    /// Get the session state for a named server.
    pub async fn session_state(&self, server_name: &str) -> SessionState {
        let sessions = self.sessions.read().await;
        sessions
            .get(server_name)
            .map(|i| i.state)
            .unwrap_or(SessionState::Idle)
    }

    /// List all session states.
    pub async fn all_sessions(&self) -> Vec<(String, SessionState)> {
        let sessions = self.sessions.read().await;
        sessions
            .iter()
            .map(|(name, info)| (name.clone(), info.state))
            .collect()
    }

    /// Shutdown all sessions gracefully.
    pub async fn shutdown(&self) -> Result<(), McpError> {
        let mut sessions = self.sessions.write().await;
        for (name, info) in sessions.iter_mut() {
            if info.state == SessionState::Connected {
                if let Err(e) = info.client.disconnect().await {
                    tracing::warn!(server = %name, error = %e, "error disconnecting MCP session");
                }
                info.state = SessionState::Disconnected;
            }
        }
        sessions.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{McpConfig, McpTransportType};

    fn test_config() -> McpConfig {
        McpConfig {
            enabled: true,
            clients: vec![McpClientConfig {
                name: "test-server".into(),
                transport: McpTransportType::Stdio,
                command: Some("echo".into()),
                url: None,
                tool_filter: Vec::new(),
                namespace: "test".into(),
            }],
            servers: Vec::new(),
            nats_bridge_enabled: false,
            nats_bridge_prefix: "ms.mcp".into(),
        }
    }

    #[tokio::test]
    async fn initial_state_is_idle() {
        let mgr = McpSessionManager::new(&test_config());
        assert_eq!(mgr.session_state("test-server").await, SessionState::Idle);
    }

    #[tokio::test]
    #[ignore = "requires real MCP server; connect() is no longer a placeholder after rmcp integration"]
    async fn lazy_connect() {
        let mgr = McpSessionManager::new(&test_config());
        mgr.get_or_connect("test-server").await.unwrap();
        assert_eq!(
            mgr.session_state("test-server").await,
            SessionState::Connected
        );
    }

    #[tokio::test]
    async fn unknown_server_returns_error() {
        let mgr = McpSessionManager::new(&test_config());
        let result = mgr.get_or_connect("unknown").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "requires real MCP server; connect() is no longer a placeholder after rmcp integration"]
    async fn shutdown_clears_sessions() {
        let mgr = McpSessionManager::new(&test_config());
        mgr.get_or_connect("test-server").await.unwrap();
        mgr.shutdown().await.unwrap();
        assert_eq!(mgr.session_state("test-server").await, SessionState::Idle);
        assert!(mgr.all_sessions().await.is_empty());
    }

    #[tokio::test]
    async fn empty_manager() {
        let mgr = McpSessionManager::empty();
        assert!(mgr.all_sessions().await.is_empty());
    }
}
