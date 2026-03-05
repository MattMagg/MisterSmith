//! NATS bridge for routing MCP tool calls across distributed nodes.
//!
//! Enables distributed tool discovery and invocation by routing MCP
//! requests over NATS subjects. Handles `tools/list` aggregation
//! and `tools/call` forwarding with timeout.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::client::McpTool;
use crate::errors::McpError;
use mister_smith_transport::{MessageEnvelope, Subscription, Transport, TransportError};

/// Default timeout for bridge requests to remote nodes.
const DEFAULT_BRIDGE_TIMEOUT: Duration = Duration::from_secs(10);

/// Routes MCP tool calls to agents on remote nodes over NATS.
pub struct McpNatsBridge {
    /// Transport used for NATS-backed bridge messaging.
    transport: Arc<dyn Transport>,
    /// NATS subject prefix for MCP bridge messages.
    subject_prefix: String,
    /// Timeout for remote node requests.
    timeout: Duration,
    /// Whether the bridge is active.
    active: bool,
    /// Held subscriptions that keep bridge listeners alive.
    subscriptions: Vec<Subscription>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RemoteToolsResponse {
    tools: Vec<McpTool>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RemoteToolCallRequest {
    params: serde_json::Value,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct RemoteToolCallResponse {
    #[serde(default)]
    result: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<String>,
}

impl McpNatsBridge {
    /// Create a new NATS bridge with the given subject prefix.
    pub fn new(subject_prefix: &str, transport: Arc<dyn Transport>) -> Self {
        Self {
            transport,
            subject_prefix: subject_prefix.to_string(),
            timeout: DEFAULT_BRIDGE_TIMEOUT,
            active: false,
            subscriptions: Vec::new(),
        }
    }

    /// Set the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Get the subject prefix.
    pub fn subject_prefix(&self) -> &str {
        &self.subject_prefix
    }

    /// Get the NATS subject for tools/list requests.
    pub fn tools_list_subject(&self) -> String {
        format!("{}.tools.list", self.subject_prefix)
    }

    /// Get the NATS subject for tools/call requests.
    pub fn tools_call_subject(&self, tool_name: &str) -> String {
        format!("{}.tools.call.{tool_name}", self.subject_prefix)
    }

    /// Start the bridge (begin listening for MCP requests on NATS).
    pub async fn start(&mut self) -> Result<(), McpError> {
        let list_sub = self
            .transport
            .subscribe(&self.tools_list_subject())
            .await
            .map_err(|err| map_transport_error("subscribe tools/list", err, self.timeout))?;
        let call_sub = self
            .transport
            .subscribe(&format!("{}.tools.call.*", self.subject_prefix))
            .await
            .map_err(|err| map_transport_error("subscribe tools/call", err, self.timeout))?;

        self.subscriptions = vec![list_sub, call_sub];
        self.active = true;
        Ok(())
    }

    /// Stop the bridge.
    pub async fn stop(&mut self) -> Result<(), McpError> {
        self.active = false;
        self.subscriptions.clear();
        Ok(())
    }

    /// Check if the bridge is active.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Aggregate tool lists from all remote nodes via NATS request-reply.
    pub async fn discover_remote_tools(&self) -> Result<Vec<McpTool>, McpError> {
        if !self.active {
            return Err(McpError::ConnectionFailed("bridge not active".into()));
        }
        let reply_subject = format!(
            "{}.tools.list.reply.{}",
            self.subject_prefix,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or_default()
        );

        let mut reply_sub = self
            .transport
            .subscribe(&reply_subject)
            .await
            .map_err(|err| map_transport_error("subscribe discovery replies", err, self.timeout))?;

        let request = MessageEnvelope::builder("mcp.bridge.tools.list.request")
            .payload_json(&serde_json::json!({}))
            .map_err(|err| McpError::ToolCallFailed(format!("serialize discovery request: {err}")))?
            .header("reply_subject", &reply_subject)
            .build()
            .map_err(|err| McpError::ToolCallFailed(format!("build discovery request: {err}")))?;

        self.transport
            .publish(&self.tools_list_subject(), request)
            .await
            .map_err(|err| map_transport_error("publish discovery request", err, self.timeout))?;

        let deadline = Instant::now() + self.timeout;
        let mut by_name: HashMap<String, McpTool> = HashMap::new();

        loop {
            let now = Instant::now();
            if now >= deadline {
                break;
            }

            let remaining = deadline - now;
            match tokio::time::timeout(remaining, reply_sub.next()).await {
                Ok(Some(msg)) => {
                    let response: RemoteToolsResponse =
                        msg.envelope.payload_as_json().map_err(|err| {
                            McpError::ToolCallFailed(format!(
                                "deserialize discovery response: {err}"
                            ))
                        })?;
                    for tool in response.tools {
                        by_name.entry(tool.name.clone()).or_insert(tool);
                    }
                }
                Ok(None) | Err(_) => break,
            }
        }

        Ok(by_name.into_values().collect())
    }

    /// Forward a tool call to a remote node via NATS.
    pub async fn call_remote_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        if !self.active {
            return Err(McpError::ConnectionFailed("bridge not active".into()));
        }
        let request_payload = RemoteToolCallRequest { params };
        let request = MessageEnvelope::builder("mcp.bridge.tools.call.request")
            .payload_json(&request_payload)
            .map_err(|err| McpError::ToolCallFailed(format!("serialize tool call request: {err}")))?
            .build()
            .map_err(|err| McpError::ToolCallFailed(format!("build tool call request: {err}")))?;

        let response_envelope = self
            .transport
            .request(&self.tools_call_subject(tool_name), request, self.timeout)
            .await
            .map_err(|err| {
                map_transport_error(
                    &format!("request remote tool '{tool_name}'"),
                    err,
                    self.timeout,
                )
            })?;

        let response: RemoteToolCallResponse =
            response_envelope.payload_as_json().map_err(|err| {
                McpError::ToolCallFailed(format!(
                    "deserialize tool call response for '{tool_name}': {err}"
                ))
            })?;

        if let Some(error) = response.error {
            return Err(McpError::ToolCallFailed(format!(
                "remote tool '{tool_name}' failed: {error}"
            )));
        }

        response.result.ok_or_else(|| {
            McpError::ToolCallFailed(format!(
                "remote tool '{tool_name}' returned malformed response: missing result and error"
            ))
        })
    }
}

fn map_transport_error(operation: &str, err: TransportError, timeout: Duration) -> McpError {
    match err {
        TransportError::Timeout(inner) => McpError::BridgeTimeout(format!(
            "{operation} timed out after {}s: {inner}",
            timeout.as_secs()
        )),
        other => McpError::ConnectionFailed(format!("{operation} failed: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mister_smith_transport::InMemoryTransport;

    fn test_bridge(prefix: &str, transport: Arc<dyn Transport>) -> McpNatsBridge {
        McpNatsBridge::new(prefix, transport)
    }

    #[test]
    fn subject_construction() {
        let bridge = test_bridge("ms.mcp", Arc::new(InMemoryTransport::new()));
        assert_eq!(bridge.tools_list_subject(), "ms.mcp.tools.list");
        assert_eq!(
            bridge.tools_call_subject("fs.read_file"),
            "ms.mcp.tools.call.fs.read_file"
        );
    }

    #[test]
    fn custom_prefix() {
        let bridge = test_bridge("custom.prefix", Arc::new(InMemoryTransport::new()));
        assert_eq!(bridge.subject_prefix(), "custom.prefix");
        assert_eq!(bridge.tools_list_subject(), "custom.prefix.tools.list");
    }

    #[tokio::test]
    async fn bridge_lifecycle() {
        let mut bridge = test_bridge("ms.mcp", Arc::new(InMemoryTransport::new()));
        assert!(!bridge.is_active());

        bridge.start().await.unwrap();
        assert!(bridge.is_active());

        bridge.stop().await.unwrap();
        assert!(!bridge.is_active());
    }

    #[tokio::test]
    async fn discover_requires_active() {
        let bridge = test_bridge("ms.mcp", Arc::new(InMemoryTransport::new()));
        assert!(bridge.discover_remote_tools().await.is_err());
    }

    #[tokio::test]
    async fn call_requires_active() {
        let bridge = test_bridge("ms.mcp", Arc::new(InMemoryTransport::new()));
        let result = bridge
            .call_remote_tool("test.tool", serde_json::json!({}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn call_timeout_when_active() {
        let mut bridge = test_bridge("ms.mcp", Arc::new(InMemoryTransport::new()));
        bridge.start().await.unwrap();
        let result = bridge
            .call_remote_tool("test.tool", serde_json::json!({}))
            .await;
        assert!(matches!(result, Err(McpError::BridgeTimeout(_))));
    }

    #[tokio::test]
    async fn discover_remote_tools_aggregates_and_deduplicates() {
        let transport = Arc::new(InMemoryTransport::new());
        let mut responder_1 = transport.subscribe("ms.mcp.tools.list").await.unwrap();
        let mut responder_2 = transport.subscribe("ms.mcp.tools.list").await.unwrap();

        let t1 = transport.clone();
        tokio::spawn(async move {
            let msg = responder_1.next().await.unwrap();
            let reply_to = msg.envelope.headers.get("reply_subject").unwrap().clone();
            let response = MessageEnvelope::builder("mcp.bridge.tools.list.response")
                .payload_json(&RemoteToolsResponse {
                    tools: vec![
                        McpTool {
                            name: "echo".into(),
                            description: "Echo text".into(),
                            input_schema: serde_json::json!({"type": "object"}),
                        },
                        McpTool {
                            name: "sum".into(),
                            description: "Sum numbers".into(),
                            input_schema: serde_json::json!({"type": "object"}),
                        },
                    ],
                })
                .unwrap()
                .build()
                .unwrap();
            t1.publish(&reply_to, response).await.unwrap();
        });

        let t2 = transport.clone();
        tokio::spawn(async move {
            let msg = responder_2.next().await.unwrap();
            let reply_to = msg.envelope.headers.get("reply_subject").unwrap().clone();
            let response = MessageEnvelope::builder("mcp.bridge.tools.list.response")
                .payload_json(&RemoteToolsResponse {
                    tools: vec![
                        McpTool {
                            name: "echo".into(),
                            description: "Echo duplicate".into(),
                            input_schema: serde_json::json!({"type": "object"}),
                        },
                        McpTool {
                            name: "search".into(),
                            description: "Search docs".into(),
                            input_schema: serde_json::json!({"type": "object"}),
                        },
                    ],
                })
                .unwrap()
                .build()
                .unwrap();
            t2.publish(&reply_to, response).await.unwrap();
        });

        let mut bridge = test_bridge("ms.mcp", transport);
        bridge.start().await.unwrap();
        let tools = bridge.discover_remote_tools().await.unwrap();
        let names: std::collections::HashSet<_> = tools.into_iter().map(|t| t.name).collect();
        assert_eq!(names.len(), 3);
        assert!(names.contains("echo"));
        assert!(names.contains("sum"));
        assert!(names.contains("search"));
    }

    #[tokio::test]
    async fn call_remote_tool_success() {
        let transport = Arc::new(InMemoryTransport::new());
        let mut responder = transport
            .subscribe("ms.mcp.tools.call.test.tool")
            .await
            .unwrap();

        let t = transport.clone();
        tokio::spawn(async move {
            let msg = responder.next().await.unwrap();
            let mut response = MessageEnvelope::builder("mcp.bridge.tools.call.response")
                .payload_json(&RemoteToolCallResponse {
                    result: Some(serde_json::json!({"ok": true})),
                    error: None,
                })
                .unwrap()
                .build()
                .unwrap();
            response.correlation_id = msg.envelope.correlation_id;
            t.publish("ignored", response).await.unwrap();
        });

        let mut bridge = test_bridge("ms.mcp", transport);
        bridge.start().await.unwrap();

        let result = bridge
            .call_remote_tool("test.tool", serde_json::json!({"foo": "bar"}))
            .await
            .unwrap();
        assert_eq!(result, serde_json::json!({"ok": true}));
    }

    #[tokio::test]
    async fn malformed_response_handling() {
        let transport = Arc::new(InMemoryTransport::new());
        let mut responder = transport
            .subscribe("ms.mcp.tools.call.bad.tool")
            .await
            .unwrap();
        let t = transport.clone();

        tokio::spawn(async move {
            let msg = responder.next().await.unwrap();
            let mut response = MessageEnvelope::builder("mcp.bridge.tools.call.response")
                .payload_raw(b"not-json".to_vec())
                .build()
                .unwrap();
            response.correlation_id = msg.envelope.correlation_id;
            t.publish("ignored", response).await.unwrap();
        });

        let mut bridge = test_bridge("ms.mcp", transport);
        bridge.start().await.unwrap();
        let err = bridge
            .call_remote_tool("bad.tool", serde_json::json!({}))
            .await
            .unwrap_err();
        assert!(matches!(err, McpError::ToolCallFailed(_)));
    }

    #[test]
    fn custom_timeout() {
        let bridge = test_bridge("ms.mcp", Arc::new(InMemoryTransport::new()))
            .with_timeout(Duration::from_secs(30));
        assert_eq!(bridge.timeout, Duration::from_secs(30));
    }
}
