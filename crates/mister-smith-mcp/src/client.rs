//! MCP client for connecting to external MCP servers.
//!
//! Provides tool discovery, caching, namespace prefixing, and tool invocation.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use mister_smith_core::DelegatedAction;
use rmcp::{
    model::{CallToolRequestParams, ClientInfo, Meta},
    service::{Peer, RoleClient, RunningService},
    transport::{StreamableHttpClientTransport, TokioChildProcess},
    ClientHandler, ServiceExt,
};
use tokio::process::Command;
use tokio::sync::{watch, RwLock};

use crate::config::{McpClientConfig, McpTransportType};
use crate::errors::McpError;
use crate::server::ToolCallRequest;

const TOOL_CALL_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const SMITH_CAPABILITY_META_KEY: &str = "mister_smith_capability";

/// Descriptor metadata for one discoverable external capability surface.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ExternalCapabilityDescriptor {
    /// Boundary family that owns the capability surface.
    pub boundary: String,
    /// Externally visible resource name.
    pub external_name: String,
    /// Stable descriptor bound to delegated authority.
    pub descriptor_id: String,
    /// Whether the surface rejects anonymous invocation.
    pub delegation_required: bool,
    /// Namespace within the external boundary.
    pub namespace: String,
    /// Boundary-local resource identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// Delegated action required for bounded discovery of the surface.
    pub discover_action: DelegatedAction,
    /// Delegated action required for execution of the surface.
    pub execute_action: DelegatedAction,
}

impl ExternalCapabilityDescriptor {
    pub(crate) fn into_meta_value(self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| serde_json::json!({}))
    }
}

pub(crate) fn capability_descriptor_from_meta(
    meta: Option<&Meta>,
) -> Option<ExternalCapabilityDescriptor> {
    meta.and_then(|meta| meta.get(SMITH_CAPABILITY_META_KEY))
        .cloned()
        .and_then(|value| serde_json::from_value(value).ok())
}

/// Represents a discovered MCP tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpTool {
    /// Tool name (without namespace prefix).
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON schema for tool parameters.
    pub input_schema: serde_json::Value,
    /// Optional bounded capability descriptor published by the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_descriptor: Option<ExternalCapabilityDescriptor>,
}

impl McpTool {
    pub(crate) fn capability_meta(&self) -> Option<Meta> {
        self.capability_descriptor.clone().map(|descriptor| {
            let mut meta = Meta::new();
            meta.0.insert(
                SMITH_CAPABILITY_META_KEY.to_string(),
                descriptor.into_meta_value(),
            );
            meta
        })
    }
}

#[derive(Clone)]
struct ToolListWatcher {
    tx: watch::Sender<u64>,
}

impl ToolListWatcher {
    fn new() -> (Self, watch::Receiver<u64>) {
        let (tx, rx) = watch::channel(0);
        (Self { tx }, rx)
    }
}

impl ClientHandler for ToolListWatcher {
    fn on_tool_list_changed(
        &self,
        _context: rmcp::service::NotificationContext<RoleClient>,
    ) -> impl Future<Output = ()> + Send + '_ {
        let tx = self.tx.clone();
        async move {
            let next = *tx.borrow() + 1;
            let _ = tx.send(next);
        }
    }

    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }
}

type RmcpSession = RunningService<RoleClient, ToolListWatcher>;

/// MCP client for connecting to and invoking tools on external MCP servers.
pub struct McpClient {
    config: McpClientConfig,
    /// Active rmcp session.
    session: Arc<RwLock<Option<RmcpSession>>>,
    /// Tool list change notifications from rmcp.
    tool_list_version: Arc<RwLock<Option<watch::Receiver<u64>>>>,
    /// Cached tool list, keyed by namespaced name.
    tool_cache: Arc<RwLock<HashMap<String, std::sync::Arc<McpTool>>>>,
    /// Whether the client is connected.
    connected: Arc<RwLock<bool>>,
}

impl McpClient {
    /// Create a new MCP client (not yet connected).
    pub fn new(config: McpClientConfig) -> Self {
        Self {
            config,
            session: Arc::new(RwLock::new(None)),
            tool_list_version: Arc::new(RwLock::new(None)),
            tool_cache: Arc::new(RwLock::new(HashMap::new())),
            connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Get the namespace for this client's tools.
    pub fn namespace(&self) -> &str {
        &self.config.namespace
    }

    /// Get the server name.
    pub fn server_name(&self) -> &str {
        &self.config.name
    }

    /// Check if the client is connected.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Connect to the MCP server and discover tools.
    pub async fn connect(&self) -> Result<(), McpError> {
        let (handler, version_rx) = ToolListWatcher::new();

        let session = match self.config.transport {
            McpTransportType::Stdio => {
                let command = self.config.command.as_deref().ok_or_else(|| {
                    McpError::ConnectionFailed("stdio transport requires command".into())
                })?;

                let mut parts = command.split_whitespace();
                let program = parts
                    .next()
                    .ok_or_else(|| McpError::ConnectionFailed("invalid command".into()))?;

                let mut cmd = Command::new(program);
                cmd.args(parts);
                let transport = TokioChildProcess::new(cmd)
                    .map_err(|e| McpError::ConnectionFailed(e.to_string()))?;
                handler
                    .serve(transport)
                    .await
                    .map_err(|e| McpError::ConnectionFailed(e.to_string()))?
            }
            McpTransportType::StreamableHttp => {
                let url = self.config.url.as_ref().ok_or_else(|| {
                    McpError::ConnectionFailed("streamable-http transport requires url".into())
                })?;
                let transport = StreamableHttpClientTransport::from_uri(url.clone());
                handler
                    .serve(transport)
                    .await
                    .map_err(|e| McpError::ConnectionFailed(e.to_string()))?
            }
        };

        {
            let mut session_lock = self.session.write().await;
            *session_lock = Some(session);
        }
        {
            let mut rx = self.tool_list_version.write().await;
            *rx = Some(version_rx);
        }
        *self.connected.write().await = true;

        self.discover_tools().await?;

        Ok(())
    }

    async fn peer(&self) -> Result<Peer<RoleClient>, McpError> {
        let session_lock = self.session.read().await;
        let session = session_lock
            .as_ref()
            .ok_or_else(|| McpError::SessionError("session not initialized".into()))?;
        Ok(session.peer().clone())
    }

    fn tool_allowed(&self, tool_name: &str) -> bool {
        if self.config.tool_filter.is_empty() {
            return true;
        }

        self.config
            .tool_filter
            .iter()
            .any(|pattern| wildcard_match(pattern, tool_name))
    }

    async fn handle_tools_changed_notification(&self) {
        let mut changed = false;
        {
            let mut version = self.tool_list_version.write().await;
            if let Some(rx) = version.as_mut() {
                changed = rx.has_changed().unwrap_or(false);
                if changed {
                    let _ = rx.borrow_and_update();
                }
            }
        }

        if changed {
            self.invalidate_cache().await;
        }
    }

    /// Discover available tools from the MCP server.
    pub async fn discover_tools(&self) -> Result<Vec<std::sync::Arc<McpTool>>, McpError> {
        if !*self.connected.read().await {
            return Err(McpError::ConnectionFailed(
                "not connected to MCP server".into(),
            ));
        }

        self.handle_tools_changed_notification().await;

        let cached = self.tool_cache.read().await;
        if !cached.is_empty() {
            return Ok(cached.values().cloned().collect());
        }
        drop(cached);

        let peer = self.peer().await?;
        let tools = peer
            .list_all_tools()
            .await
            .map_err(|e| McpError::SessionError(e.to_string()))?;

        let discovered: Vec<std::sync::Arc<McpTool>> = tools
            .into_iter()
            .filter(|tool| self.tool_allowed(tool.name.as_ref()))
            .map(|tool| {
                std::sync::Arc::new(McpTool {
                    name: tool.name.into_owned(),
                    description: tool.description.map(|d| d.into_owned()).unwrap_or_default(),
                    input_schema: serde_json::Value::Object((*tool.input_schema).clone()),
                    capability_descriptor: capability_descriptor_from_meta(tool.meta.as_ref()),
                })
            })
            .collect();

        let mut cache = self.tool_cache.write().await;
        cache.clear();
        for tool in &discovered {
            let namespace = &self.config.namespace;
            let name = &tool.name;
            let namespaced = format!("{namespace}.{name}");
            cache.insert(namespaced, std::sync::Arc::clone(tool));
        }

        Ok(discovered)
    }

    /// Get a tool by its namespaced name.
    pub async fn get_tool(
        &self,
        namespaced_name: &str,
    ) -> Result<std::sync::Arc<McpTool>, McpError> {
        let cache = self.tool_cache.read().await;
        cache
            .get(namespaced_name)
            .cloned()
            .ok_or_else(|| McpError::ToolNotFound(namespaced_name.to_string()))
    }

    /// Register a tool in the cache (used during discovery).
    pub async fn register_tool(&self, tool: std::sync::Arc<McpTool>) {
        let namespace = &self.config.namespace;
        let name = &tool.name;
        let namespaced = format!("{namespace}.{name}");
        let mut cache = self.tool_cache.write().await;
        cache.insert(namespaced, tool);
    }

    /// Invoke a tool by name with the given parameters.
    pub async fn call_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        self.call_tool_request(tool_name, ToolCallRequest::new(params))
            .await
    }

    /// Invoke a tool with explicit Smith transport context.
    pub async fn call_tool_request(
        &self,
        tool_name: &str,
        request: ToolCallRequest,
    ) -> Result<serde_json::Value, McpError> {
        if !*self.connected.read().await {
            return Err(McpError::ConnectionFailed("not connected".into()));
        }

        self.handle_tools_changed_notification().await;

        let namespace = &self.config.namespace;
        let namespaced = format!("{namespace}.{tool_name}");
        let _tool = self.get_tool(&namespaced).await?;

        let peer = self.peer().await?;
        let args = match request.into_wire_params() {
            serde_json::Value::Object(obj) => obj,
            _ => {
                return Err(McpError::SerializationError(
                    "tool params must be a JSON object".into(),
                ))
            }
        };

        let request = CallToolRequestParams::new(tool_name.to_string()).with_arguments(args);

        let result = tokio::time::timeout(TOOL_CALL_TIMEOUT, peer.call_tool(request)).await;

        let result = match result {
            Ok(inner) => inner.map_err(|e| McpError::ToolCallFailed(e.to_string()))?,
            Err(_) => {
                return Err(McpError::BridgeTimeout(format!(
                    "tool '{tool_name}' timed out after {TOOL_CALL_TIMEOUT:?}"
                )))
            }
        };

        if result.is_error.unwrap_or(false) {
            return Err(McpError::ToolCallFailed(
                serde_json::to_string(&result)
                    .unwrap_or_else(|_| "tool returned error".to_string()),
            ));
        }

        if let Some(structured) = result.structured_content {
            Ok(structured)
        } else {
            serde_json::to_value(result).map_err(|e| McpError::SerializationError(e.to_string()))
        }
    }

    /// Invalidate the tool cache (called on tools/list_changed notification).
    pub async fn invalidate_cache(&self) {
        let mut cache = self.tool_cache.write().await;
        cache.clear();
    }

    /// Disconnect from the MCP server.
    pub async fn disconnect(&self) -> Result<(), McpError> {
        let mut session = self.session.write().await;
        if let Some(mut running) = session.take() {
            let _ = running.close().await;
        }

        *self.connected.write().await = false;
        *self.tool_list_version.write().await = None;
        self.invalidate_cache().await;
        Ok(())
    }
}

fn wildcard_match(pattern: &str, input: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    let mut remaining = input;
    let mut first = true;

    for part in pattern.split('*') {
        if part.is_empty() {
            continue;
        }

        if first && !pattern.starts_with('*') {
            if !remaining.starts_with(part) {
                return false;
            }
            remaining = &remaining[part.len()..];
        } else if let Some(idx) = remaining.find(part) {
            remaining = &remaining[idx + part.len()..];
        } else {
            return false;
        }

        first = false;
    }

    pattern.ends_with('*')
        || remaining.is_empty()
        || pattern
            .split('*')
            .next_back()
            .is_some_and(|last| input.ends_with(last))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use crate::server::ToolCallRequest;
    use mister_smith_core::{
        AgentId, AuthorityPrincipal, CapabilityActionKind, DelegatedAction, DelegatedActionPolicy,
        DelegationScope, ExternalDelegationEnvelope,
    };
    use mister_smith_security::DelegationService;
    use rmcp::{model::*, serve_server, ServerHandler};

    fn test_config() -> McpClientConfig {
        McpClientConfig {
            name: "test-server".into(),
            transport: McpTransportType::Stdio,
            command: Some("echo".into()),
            url: None,
            tool_filter: Vec::new(),
            namespace: "test".into(),
        }
    }

    #[derive(Clone)]
    struct TestServer {
        tools: Arc<RwLock<Vec<Tool>>>,
        calls: Arc<RwLock<u64>>,
    }

    impl TestServer {
        fn new(tools: Vec<Tool>) -> Self {
            Self {
                tools: Arc::new(RwLock::new(tools)),
                calls: Arc::new(RwLock::new(0)),
            }
        }

        async fn call_count(&self) -> u64 {
            *self.calls.read().await
        }
    }

    impl ServerHandler for TestServer {
        fn get_info(&self) -> ServerInfo {
            ServerInfo::default()
        }

        fn list_tools(
            &self,
            _request: Option<PaginatedRequestParams>,
            _context: rmcp::service::RequestContext<rmcp::RoleServer>,
        ) -> impl Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send + '_ {
            let tools = self.tools.clone();
            async move {
                Ok(ListToolsResult {
                    next_cursor: None,
                    tools: tools.read().await.clone(),
                    meta: None,
                })
            }
        }

        fn call_tool(
            &self,
            request: CallToolRequestParams,
            _context: rmcp::service::RequestContext<rmcp::RoleServer>,
        ) -> impl Future<Output = Result<CallToolResult, rmcp::ErrorData>> + Send + '_ {
            let calls = self.calls.clone();
            async move {
                let mut guard = calls.write().await;
                *guard += 1;

                if request.name.as_ref() == "slow" {
                    tokio::time::sleep(Duration::from_secs(11)).await;
                }

                if request.name.as_ref() == "fail" {
                    return Ok(CallToolResult::structured_error(
                        serde_json::json!({"reason": "boom"}),
                    ));
                }

                Ok(CallToolResult::structured(serde_json::json!({
                    "name": request.name,
                    "args": request.arguments
                })))
            }
        }
    }

    fn mk_tool(name: &str) -> Tool {
        Tool::new(
            name.to_string(),
            format!("{name} description"),
            serde_json::Map::new(),
        )
    }

    fn sample_external_delegation() -> ExternalDelegationEnvelope {
        let service = DelegationService::new();
        let recipient = AgentId::from_uuid(uuid::Uuid::new_v4());
        let (capability, provenance) = service
            .issue_capability(
                AuthorityPrincipal::Policy("operator".to_string()),
                recipient,
                DelegationScope::InvokeTool,
                Some("tool:test.echo".to_string()),
                Duration::from_secs(300),
                None,
                None,
            )
            .expect("delegation should issue");

        ExternalDelegationEnvelope::new(capability, provenance).with_action(DelegatedAction {
            descriptor_id: "tool:test.echo".to_string(),
            action_id: "tool:test.echo#execute".to_string(),
            title: "execute test.echo".to_string(),
            description: "execute access for tool test.echo".to_string(),
            kind: CapabilityActionKind::Execute,
            policy: DelegatedActionPolicy {
                action: "execute".to_string(),
                resource: "tool".to_string(),
                scope: "test".to_string(),
                resource_id: Some("test.echo".to_string()),
            },
            required_scope: Some(DelegationScope::InvokeTool),
            revocation_key: "tool:test.echo#execute".to_string(),
        })
    }

    async fn connected_client_with_server(
        cfg: McpClientConfig,
        server: TestServer,
    ) -> Result<McpClient, McpError> {
        let (server_transport, client_transport) = tokio::io::duplex(4096);

        tokio::spawn(async move {
            if let Ok(running) = serve_server(server, server_transport).await {
                let _ = running.waiting().await;
            }
        });

        let (handler, version_rx) = ToolListWatcher::new();
        let session = handler
            .serve(client_transport)
            .await
            .map_err(|e| McpError::ConnectionFailed(e.to_string()))?;

        let client = McpClient::new(cfg);
        *client.connected.write().await = true;
        *client.session.write().await = Some(session);
        *client.tool_list_version.write().await = Some(version_rx);
        Ok(client)
    }

    #[tokio::test]
    async fn connection_failure_path() {
        let cfg = McpClientConfig {
            command: None,
            ..test_config()
        };

        let client = McpClient::new(cfg);
        let err = client.connect().await.unwrap_err();
        assert!(matches!(err, McpError::ConnectionFailed(_)));
    }

    #[tokio::test]
    async fn cache_miss_then_hit_behavior() {
        let server = TestServer::new(vec![mk_tool("echo")]);
        let client = connected_client_with_server(test_config(), server.clone())
            .await
            .unwrap();

        let first = client.discover_tools().await.unwrap();
        assert_eq!(first.len(), 1);

        let second = client.discover_tools().await.unwrap();
        assert_eq!(second.len(), 1);

        let result = client
            .call_tool("echo", serde_json::json!({"x": 1}))
            .await
            .unwrap();
        assert_eq!(result["name"], "echo");
        assert_eq!(server.call_count().await, 1);
    }

    #[tokio::test]
    async fn call_tool_request_wraps_delegation_context_on_wire() {
        let server = TestServer::new(vec![mk_tool("echo")]);
        let client = connected_client_with_server(test_config(), server)
            .await
            .unwrap();

        client.discover_tools().await.unwrap();

        let result = client
            .call_tool_request(
                "echo",
                ToolCallRequest::new(serde_json::json!({"x": 1}))
                    .with_delegation(sample_external_delegation()),
            )
            .await
            .unwrap();

        assert_eq!(result["args"]["params"]["x"], 1);
        assert_eq!(
            result["args"]["_mister_smith"]["delegation"]["action"]["descriptor_id"],
            "tool:test.echo"
        );
    }

    #[tokio::test]
    async fn tool_filter_application() {
        let cfg = McpClientConfig {
            tool_filter: vec!["read_*".into()],
            ..test_config()
        };
        let server = TestServer::new(vec![mk_tool("read_file"), mk_tool("write_file")]);
        let client = connected_client_with_server(cfg, server).await.unwrap();

        let tools = client.discover_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "read_file");
    }

    #[tokio::test]
    async fn discovery_preserves_mister_smith_capability_metadata() {
        let mut meta = Meta::new();
        meta.0.insert(
            SMITH_CAPABILITY_META_KEY.to_string(),
            ExternalCapabilityDescriptor {
                boundary: "mcp.tool".to_string(),
                external_name: "echo".to_string(),
                descriptor_id: "tool:echo".to_string(),
                delegation_required: false,
                namespace: "test".to_string(),
                resource_id: Some("echo".to_string()),
                discover_action: DelegatedAction {
                    descriptor_id: "tool:echo".to_string(),
                    action_id: "tool:echo#discover".to_string(),
                    title: "discover echo".to_string(),
                    description: "discover access for tool echo".to_string(),
                    kind: CapabilityActionKind::Discover,
                    policy: DelegatedActionPolicy {
                        action: "discover".to_string(),
                        resource: "tool".to_string(),
                        scope: "test".to_string(),
                        resource_id: Some("echo".to_string()),
                    },
                    required_scope: None,
                    revocation_key: "tool:echo#discover".to_string(),
                },
                execute_action: DelegatedAction {
                    descriptor_id: "tool:echo".to_string(),
                    action_id: "tool:echo#execute".to_string(),
                    title: "execute echo".to_string(),
                    description: "execute access for tool echo".to_string(),
                    kind: CapabilityActionKind::Execute,
                    policy: DelegatedActionPolicy {
                        action: "execute".to_string(),
                        resource: "tool".to_string(),
                        scope: "test".to_string(),
                        resource_id: Some("echo".to_string()),
                    },
                    required_scope: Some(DelegationScope::InvokeTool),
                    revocation_key: "tool:echo#execute".to_string(),
                },
            }
            .into_meta_value(),
        );
        let server = TestServer::new(vec![mk_tool("echo").with_meta(meta)]);
        let client = connected_client_with_server(test_config(), server)
            .await
            .unwrap();

        let tools = client.discover_tools().await.unwrap();
        let capability = tools[0]
            .capability_descriptor
            .as_ref()
            .expect("capability metadata should be preserved");
        assert_eq!(capability.boundary, "mcp.tool");
        assert_eq!(capability.external_name, "echo");
        assert_eq!(capability.descriptor_id, "tool:echo");
        assert!(!capability.delegation_required);
        assert_eq!(capability.discover_action.action_id, "tool:echo#discover");
        assert_eq!(capability.execute_action.action_id, "tool:echo#execute");
    }

    #[tokio::test]
    async fn namespace_prefixing() {
        let server = TestServer::new(vec![mk_tool("echo")]);
        let client = connected_client_with_server(test_config(), server)
            .await
            .unwrap();

        client.discover_tools().await.unwrap();
        assert!(client.get_tool("test.echo").await.is_ok());
    }

    #[tokio::test]
    async fn timeout_and_error_mapping() {
        let server = TestServer::new(vec![mk_tool("slow"), mk_tool("fail")]);
        let client = connected_client_with_server(test_config(), server)
            .await
            .unwrap();
        client.discover_tools().await.unwrap();

        let timeout = client.call_tool("slow", serde_json::json!({})).await;
        assert!(matches!(timeout, Err(McpError::BridgeTimeout(_))));

        let failed = client.call_tool("fail", serde_json::json!({})).await;
        assert!(matches!(failed, Err(McpError::ToolCallFailed(_))));
    }
}
