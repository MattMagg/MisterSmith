//! MCP server for exposing agent tools to external MCP clients.
//!
//! Implements `tools/list` and `tools/call` handlers with namespace
//! filtering and permission checks.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

use mister_smith_core::{
    CapabilityActionKind, DelegatedAction, DelegatedActionPolicy, DelegationScope,
    ExternalDelegationEnvelope,
};
use mister_smith_security::DelegationService;
use rmcp::{
    model::{
        CallToolRequestParams, CallToolResult, ListToolsResult, PaginatedRequestParams, ServerInfo,
        Tool,
    },
    serve_server,
    transport::io::stdio,
    ServerHandler,
};

use crate::client::{ExternalCapabilityDescriptor, McpTool};
use crate::config::McpServerConfig;
use crate::errors::McpError;

/// Represents a registered agent tool exposed via MCP server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExposedTool {
    /// Tool name (namespace-scoped).
    pub name: String,
    /// Tool description.
    pub description: String,
    /// JSON schema for tool parameters.
    pub input_schema: serde_json::Value,
    /// Namespace this tool belongs to.
    pub namespace: String,
    /// Explicit delegated boundary action required to invoke the tool, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_boundary_action: Option<DelegatedAction>,
}

/// External capability catalog entry for one exposed MCP tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CapabilityCatalogEntry {
    /// Externally visible tool name.
    pub tool_name: String,
    /// Human-readable description.
    pub description: String,
    /// Tool input schema.
    pub input_schema: serde_json::Value,
    /// Capability descriptor published to MCP clients.
    pub capability_descriptor: ExternalCapabilityDescriptor,
    /// Exact delegated action that preserves policy at the call boundary.
    pub boundary_action: DelegatedAction,
    /// Whether the tool refuses anonymous invocation.
    pub delegation_required: bool,
}

/// Reserved metadata wrapper for Smith-specific MCP transport context.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct ToolCallContext {
    /// Delegated authority preserved across the MCP boundary, when any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation: Option<ExternalDelegationEnvelope>,
}

impl ToolCallContext {
    fn is_empty(&self) -> bool {
        self.delegation.is_none()
    }
}

/// Typed request delivered to MCP tool handlers.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ToolCallRequest {
    /// Original tool parameters supplied by the caller.
    pub params: serde_json::Value,
    /// Reserved Smith-specific transport context.
    #[serde(
        default,
        rename = "_mister_smith",
        skip_serializing_if = "ToolCallContext::is_empty"
    )]
    pub context: ToolCallContext,
}

impl ToolCallRequest {
    /// Create a tool call request from plain JSON parameters.
    #[must_use]
    pub fn new(params: serde_json::Value) -> Self {
        Self {
            params,
            context: ToolCallContext::default(),
        }
    }

    /// Attach a delegation envelope to the tool call request.
    #[must_use]
    pub fn with_delegation(mut self, delegation: ExternalDelegationEnvelope) -> Self {
        self.context.delegation = Some(delegation);
        self
    }

    /// Decode raw wire parameters into a typed request, preserving legacy raw params.
    pub fn from_wire_params(raw: serde_json::Value) -> Result<Self, McpError> {
        match raw {
            serde_json::Value::Object(map)
                if map.contains_key("_mister_smith") && map.contains_key("params") =>
            {
                serde_json::from_value(serde_json::Value::Object(map))
                    .map_err(|err| McpError::SerializationError(err.to_string()))
            }
            serde_json::Value::Object(map) => Ok(Self::new(serde_json::Value::Object(map))),
            _ => Err(McpError::SerializationError(
                "tool params must be a JSON object".into(),
            )),
        }
    }

    /// Encode the request for transport while preserving legacy raw params when possible.
    #[must_use]
    pub fn into_wire_params(self) -> serde_json::Value {
        if self.context.is_empty() {
            self.params
        } else {
            serde_json::to_value(self).unwrap_or_else(|_| serde_json::json!({}))
        }
    }
}

/// Handler result for tool invocations.
pub type ToolHandler = Arc<
    dyn Fn(
            ToolCallRequest,
        ) -> futures::future::BoxFuture<'static, Result<serde_json::Value, McpError>>
        + Send
        + Sync,
>;

/// MCP server that exposes agent tools to external clients.
pub struct McpServer {
    config: McpServerConfig,
    /// Registered tools keyed by namespaced name.
    tools: Arc<RwLock<HashMap<String, ExposedTool>>>,
    /// Tool handlers keyed by namespaced name.
    handlers: Arc<RwLock<HashMap<String, ToolHandler>>>,
    delegation_service: Option<Arc<DelegationService>>,
    /// Whether the server is running.
    running: Arc<RwLock<bool>>,
}

impl McpServer {
    /// Create a new MCP server with the given configuration.
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            tools: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            delegation_service: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Attach the local delegation validator used for inbound MCP requests.
    #[must_use]
    pub fn with_delegation_service(mut self, delegation_service: Arc<DelegationService>) -> Self {
        self.delegation_service = Some(delegation_service);
        self
    }

    /// Get the bind address.
    pub fn bind_address(&self) -> &str {
        &self.config.bind_address
    }

    /// Get the namespace views.
    pub fn namespace_views(&self) -> &[String] {
        &self.config.namespace_views
    }

    /// Register a tool to be exposed via MCP.
    pub async fn register_tool(&self, tool: ExposedTool, handler: ToolHandler) {
        let key = tool.external_name();
        let mut tools = self.tools.write().await;
        tools.insert(key.clone(), tool);
        let mut handlers = self.handlers.write().await;
        handlers.insert(key, handler);
    }

    /// Handle a `tools/list` request with namespace filtering.
    pub async fn handle_tools_list(
        &self,
        namespace_filter: Option<&str>,
    ) -> Result<Vec<McpTool>, McpError> {
        let tools = self.tools.read().await;
        let filtered: Vec<McpTool> = tools
            .values()
            .filter(|t| {
                if let Some(ns) = namespace_filter {
                    t.namespace == ns
                } else if self.config.namespace_views.is_empty() {
                    true
                } else {
                    self.config.namespace_views.contains(&t.namespace)
                }
            })
            .map(|t| McpTool {
                name: t.external_name(),
                description: t.description.clone(),
                input_schema: t.input_schema.clone(),
                capability_descriptor: Some(t.capability_descriptor()),
            })
            .collect();
        Ok(filtered)
    }

    /// Handle a `tools/call` request.
    pub async fn handle_tools_call(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        let request = ToolCallRequest::from_wire_params(params)?;
        let tool = self
            .tools
            .read()
            .await
            .get(tool_name)
            .cloned()
            .ok_or_else(|| McpError::ToolNotFound(tool_name.to_string()))?;
        let expected_action = tool.boundary_action();

        match request.context.delegation.as_ref() {
            Some(envelope) => {
                let delegation_service = self.delegation_service.as_ref().ok_or_else(|| {
                    McpError::ToolCallFailed(
                        "delegation envelope requires a configured delegation service".to_string(),
                    )
                })?;
                validate_delegation_for_boundary_action(
                    delegation_service,
                    envelope,
                    &expected_action,
                    tool_name,
                )?;
            }
            None if tool.required_boundary_action.is_some() => {
                return Err(McpError::ToolCallFailed(format!(
                    "delegation envelope required for MCP tool '{tool_name}'"
                )));
            }
            None => {}
        }

        let handlers = self.handlers.read().await;
        let handler = handlers
            .get(tool_name)
            .ok_or_else(|| McpError::ToolNotFound(tool_name.to_string()))?;

        let handler = Arc::clone(handler);
        drop(handlers);

        handler(request).await
    }

    /// Start the MCP server.
    pub async fn start(&self) -> Result<(), McpError> {
        let mut running = self.running.write().await;
        *running = true;
        Ok(())
    }

    /// Stop the MCP server.
    pub async fn stop(&self) -> Result<(), McpError> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    /// Check if the server is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Count registered tools.
    pub async fn tool_count(&self) -> usize {
        self.tools.read().await.len()
    }

    /// List registered external tool names.
    pub async fn registered_tool_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .tools
            .read()
            .await
            .values()
            .map(ExposedTool::external_name)
            .collect();
        names.sort();
        names
    }

    /// Return the current external capability catalog for registered tools.
    pub async fn capability_catalog(&self) -> Vec<CapabilityCatalogEntry> {
        let mut catalog: Vec<CapabilityCatalogEntry> = self
            .tools
            .read()
            .await
            .values()
            .map(ExposedTool::catalog_entry)
            .collect();
        catalog.sort_by(|left, right| left.tool_name.cmp(&right.tool_name));
        catalog
    }

    /// Serve the MCP server over stdio until the client disconnects.
    pub async fn serve_stdio(self: Arc<Self>) -> Result<(), McpError> {
        self.start().await?;

        let adapter = McpServerAdapter {
            server: self.clone(),
        };

        let running = serve_server(adapter, stdio())
            .await
            .map_err(|err| McpError::ConnectionFailed(err.to_string()))?;

        let wait_result = running.waiting().await;
        self.stop().await?;

        wait_result
            .map(|_| ())
            .map_err(|err| McpError::SessionError(err.to_string()))
    }
}

impl ExposedTool {
    fn external_name(&self) -> String {
        if self.namespace.trim().is_empty() {
            self.name.clone()
        } else {
            format!("{}.{}", self.namespace, self.name)
        }
    }

    fn boundary_action(&self) -> DelegatedAction {
        self.required_boundary_action.clone().unwrap_or_else(|| {
            tool_boundary_action(
                &self.external_name(),
                &self.namespace,
                CapabilityActionKind::Execute,
            )
        })
    }

    fn capability_descriptor(&self) -> ExternalCapabilityDescriptor {
        let external_name = self.external_name();
        let boundary_action = self.boundary_action();

        ExternalCapabilityDescriptor {
            boundary: "mcp.tool".to_string(),
            external_name,
            descriptor_id: boundary_action.descriptor_id.clone(),
            action_id: boundary_action.action_id.clone(),
            required_scope: boundary_action
                .required_scope
                .map(|scope| format!("{scope:?}"))
                .unwrap_or_else(|| "none".to_string()),
            namespace: self.namespace.clone(),
            resource_id: Some(self.name.clone()),
            revocation_key: boundary_action.revocation_key.clone(),
        }
    }

    fn catalog_entry(&self) -> CapabilityCatalogEntry {
        CapabilityCatalogEntry {
            tool_name: self.external_name(),
            description: self.description.clone(),
            input_schema: self.input_schema.clone(),
            capability_descriptor: self.capability_descriptor(),
            boundary_action: self.boundary_action(),
            delegation_required: self.required_boundary_action.is_some(),
        }
    }
}

fn tool_descriptor_id(tool_name: &str) -> String {
    format!("tool:{tool_name}")
}

/// Build the canonical delegated boundary action for one MCP tool.
#[must_use]
pub fn tool_boundary_action(
    external_name: &str,
    namespace: &str,
    kind: CapabilityActionKind,
) -> DelegatedAction {
    let descriptor_id = tool_descriptor_id(external_name);
    let action_id = format!("{descriptor_id}#{}", kind.policy_action());
    let required_scope =
        matches!(kind, CapabilityActionKind::Execute).then_some(DelegationScope::InvokeTool);

    DelegatedAction {
        descriptor_id,
        action_id: action_id.clone(),
        title: format!("{} {external_name}", kind.policy_action()),
        description: format!(
            "{} access for MCP tool {external_name}",
            kind.policy_action()
        ),
        kind,
        policy: DelegatedActionPolicy {
            action: kind.policy_action().to_string(),
            resource: "tool".to_string(),
            scope: namespace.to_string(),
            resource_id: Some(external_name.to_string()),
        },
        required_scope,
        revocation_key: action_id,
    }
}

fn validate_delegation_for_boundary_action(
    delegation_service: &DelegationService,
    envelope: &ExternalDelegationEnvelope,
    expected_action: &DelegatedAction,
    tool_name: &str,
) -> Result<(), McpError> {
    let Some(action) = envelope.action.as_ref() else {
        return Err(McpError::ToolCallFailed(format!(
            "delegation action missing for MCP tool '{tool_name}'"
        )));
    };

    if action != expected_action {
        return Err(McpError::ToolCallFailed(format!(
            "delegation action '{}' does not authorize MCP tool '{}' with required action '{}'",
            action.action_id, tool_name, expected_action.action_id
        )));
    }

    delegation_service
        .validate_action(&envelope.capability, &envelope.provenance, expected_action)
        .map_err(|err| {
            McpError::ToolCallFailed(format!(
                "delegation envelope rejected at MCP boundary: {err}"
            ))
        })?;

    Ok(())
}

#[derive(Clone)]
pub(crate) struct McpServerAdapter {
    pub(crate) server: Arc<McpServer>,
}

impl ServerHandler for McpServerAdapter {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::default()
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send + '_ {
        let server = self.server.clone();
        async move {
            let tools = server
                .handle_tools_list(None)
                .await
                .map_err(|err| rmcp::ErrorData::internal_error(err.to_string(), None))?;

            let rendered = tools
                .into_iter()
                .map(|tool| {
                    let capability_meta = tool.capability_meta();
                    let schema = match tool.input_schema {
                        serde_json::Value::Object(map) => map,
                        _ => serde_json::Map::new(),
                    };
                    let rendered = Tool::new(tool.name, tool.description, schema);
                    match capability_meta {
                        Some(meta) => rendered.with_meta(meta),
                        None => rendered,
                    }
                })
                .collect();

            Ok(ListToolsResult {
                next_cursor: None,
                tools: rendered,
                meta: None,
            })
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl Future<Output = Result<CallToolResult, rmcp::ErrorData>> + Send + '_ {
        let server = self.server.clone();
        async move {
            let params = serde_json::Value::Object(request.arguments.unwrap_or_default());
            match server
                .handle_tools_call(request.name.as_ref(), params)
                .await
            {
                Ok(value) => Ok(CallToolResult::structured(value)),
                Err(err) => Ok(CallToolResult::structured_error(serde_json::json!({
                    "status": "error",
                    "summary": err.to_string(),
                    "blocking_issues": [err.to_string()],
                }))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use mister_smith_core::{
        AgentId, AuthorityPrincipal, CapabilityActionKind, DelegatedAction, DelegationScope,
    };
    use mister_smith_security::DelegationService;

    fn test_config() -> McpServerConfig {
        McpServerConfig {
            bind_address: "0.0.0.0:8090".into(),
            namespace_views: vec!["agent".into()],
        }
    }

    fn sample_external_delegation_for_action(
        action: DelegatedAction,
    ) -> (Arc<DelegationService>, ExternalDelegationEnvelope) {
        let service = Arc::new(DelegationService::new());
        let recipient = AgentId::from_uuid(uuid::Uuid::new_v4());
        let (capability, provenance) = service
            .issue_capability(
                AuthorityPrincipal::Policy("operator".to_string()),
                recipient,
                DelegationScope::InvokeTool,
                Some(action.descriptor_id.clone()),
                Duration::from_secs(300),
                None,
                None,
            )
            .expect("delegation should issue");

        (
            service,
            ExternalDelegationEnvelope::new(capability, provenance).with_action(action),
        )
    }

    fn sample_external_delegation(
        descriptor_id: &str,
    ) -> (Arc<DelegationService>, ExternalDelegationEnvelope) {
        let external_name = descriptor_id.trim_start_matches("tool:");
        let namespace = external_name.split('.').next().unwrap_or_default();
        sample_external_delegation_for_action(tool_boundary_action(
            external_name,
            namespace,
            CapabilityActionKind::Execute,
        ))
    }

    #[tokio::test]
    async fn register_and_list_tools() {
        let server = McpServer::new(test_config());

        let tool = ExposedTool {
            name: "greet".into(),
            description: "Say hello".into(),
            input_schema: serde_json::json!({"type": "object"}),
            namespace: "agent".into(),
            required_boundary_action: None,
        };

        let handler: ToolHandler =
            Arc::new(|_params| Box::pin(async { Ok(serde_json::json!({"message": "hello"})) }));

        server.register_tool(tool, handler).await;

        let tools = server.handle_tools_list(None).await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "agent.greet");
        let capability = tools[0]
            .capability_descriptor
            .as_ref()
            .expect("listed tools should publish bounded capability metadata");
        assert_eq!(capability.boundary, "mcp.tool");
        assert_eq!(capability.external_name, "agent.greet");
        assert_eq!(capability.descriptor_id, "tool:agent.greet");
        let meta = tools[0]
            .capability_meta()
            .expect("listed tool should reconstruct Smith capability meta");
        assert_eq!(
            meta.0["mister_smith_capability"]["descriptor_id"],
            serde_json::json!("tool:agent.greet")
        );
    }

    #[tokio::test]
    async fn namespace_filtering() {
        let server = McpServer::new(test_config());

        let tool1 = ExposedTool {
            name: "read".into(),
            description: "Read file".into(),
            input_schema: serde_json::json!({}),
            namespace: "agent".into(),
            required_boundary_action: None,
        };
        let tool2 = ExposedTool {
            name: "search".into(),
            description: "Search web".into(),
            input_schema: serde_json::json!({}),
            namespace: "external".into(),
            required_boundary_action: None,
        };

        let handler: ToolHandler = Arc::new(|_| Box::pin(async { Ok(serde_json::json!({})) }));

        server.register_tool(tool1, handler.clone()).await;
        server.register_tool(tool2, handler).await;

        // Default view only shows "agent" namespace per config.
        let tools = server.handle_tools_list(None).await.unwrap();
        assert_eq!(tools.len(), 1);

        // Explicit filter shows specific namespace.
        let tools = server.handle_tools_list(Some("external")).await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "external.search");
    }

    #[tokio::test]
    async fn call_registered_tool() {
        let server = McpServer::new(test_config());

        let tool = ExposedTool {
            name: "echo".into(),
            description: "Echo back".into(),
            input_schema: serde_json::json!({}),
            namespace: "agent".into(),
            required_boundary_action: None,
        };

        let handler: ToolHandler = Arc::new(|request| {
            Box::pin(async move { Ok(serde_json::json!({"echo": request.params})) })
        });

        server.register_tool(tool, handler).await;

        let result = server
            .handle_tools_call("agent.echo", serde_json::json!({"msg": "hi"}))
            .await
            .unwrap();
        assert_eq!(result["echo"]["msg"], "hi");
    }

    #[tokio::test]
    async fn wrapped_tool_call_preserves_delegation_context() {
        let (service, delegation) = sample_external_delegation("tool:agent.echo");
        let server = McpServer::new(test_config()).with_delegation_service(service);

        let tool = ExposedTool {
            name: "echo".into(),
            description: "Echo back".into(),
            input_schema: serde_json::json!({}),
            namespace: "agent".into(),
            required_boundary_action: None,
        };

        let handler: ToolHandler = Arc::new(|request| {
            Box::pin(async move {
                Ok(serde_json::json!({
                    "descriptor_id": request
                        .context
                        .delegation
                        .as_ref()
                        .and_then(ExternalDelegationEnvelope::descriptor_id),
                    "params": request.params,
                }))
            })
        });

        server.register_tool(tool, handler).await;

        let result = server
            .handle_tools_call(
                "agent.echo",
                ToolCallRequest::new(serde_json::json!({"msg": "hi"}))
                    .with_delegation(delegation)
                    .into_wire_params(),
            )
            .await
            .unwrap();

        assert_eq!(result["descriptor_id"], "tool:agent.echo");
        assert_eq!(result["params"]["msg"], "hi");
    }

    #[tokio::test]
    async fn revoked_delegation_is_rejected_before_handler_execution() {
        let (service, delegation) = sample_external_delegation("tool:agent.echo");
        service.revoke_action("tool:agent.echo#execute");
        let server = McpServer::new(test_config()).with_delegation_service(service);

        let tool = ExposedTool {
            name: "echo".into(),
            description: "Echo back".into(),
            input_schema: serde_json::json!({}),
            namespace: "agent".into(),
            required_boundary_action: None,
        };

        let handler: ToolHandler =
            Arc::new(|_| Box::pin(async move { Ok(serde_json::json!({"unexpected": true})) }));

        server.register_tool(tool, handler).await;

        let err = server
            .handle_tools_call(
                "agent.echo",
                ToolCallRequest::new(serde_json::json!({"msg": "hi"}))
                    .with_delegation(delegation)
                    .into_wire_params(),
            )
            .await
            .unwrap_err();

        assert!(
            matches!(err, McpError::ToolCallFailed(message) if message.contains("rejected at MCP boundary"))
        );
    }

    #[tokio::test]
    async fn mismatched_delegation_descriptor_is_rejected_before_handler_execution() {
        let (service, delegation) = sample_external_delegation("tool:agent.other");
        let server = McpServer::new(test_config()).with_delegation_service(service);

        let tool = ExposedTool {
            name: "echo".into(),
            description: "Echo back".into(),
            input_schema: serde_json::json!({}),
            namespace: "agent".into(),
            required_boundary_action: None,
        };

        let handler: ToolHandler =
            Arc::new(|_| Box::pin(async move { Ok(serde_json::json!({"unexpected": true})) }));

        server.register_tool(tool, handler).await;

        let err = server
            .handle_tools_call(
                "agent.echo",
                ToolCallRequest::new(serde_json::json!({"msg": "hi"}))
                    .with_delegation(delegation)
                    .into_wire_params(),
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            McpError::ToolCallFailed(message)
                if message.contains("does not authorize MCP tool 'agent.echo'")
        ));
    }

    #[tokio::test]
    async fn mismatched_delegation_action_is_rejected_before_handler_execution() {
        let discover_action =
            tool_boundary_action("agent.echo", "agent", CapabilityActionKind::Discover);
        let (service, delegation) = sample_external_delegation_for_action(discover_action);
        let server = McpServer::new(test_config()).with_delegation_service(service);

        let tool = ExposedTool {
            name: "echo".into(),
            description: "Echo back".into(),
            input_schema: serde_json::json!({}),
            namespace: "agent".into(),
            required_boundary_action: None,
        };

        let handler: ToolHandler =
            Arc::new(|_| Box::pin(async move { Ok(serde_json::json!({"unexpected": true})) }));

        server.register_tool(tool, handler).await;

        let err = server
            .handle_tools_call(
                "agent.echo",
                ToolCallRequest::new(serde_json::json!({"msg": "hi"}))
                    .with_delegation(delegation)
                    .into_wire_params(),
            )
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            McpError::ToolCallFailed(message)
                if message.contains("does not authorize MCP tool 'agent.echo' with required action")
        ));
    }

    #[tokio::test]
    async fn required_boundary_action_rejects_missing_delegation() {
        let server = McpServer::new(test_config())
            .with_delegation_service(Arc::new(DelegationService::new()));
        let tool = ExposedTool {
            name: "describe".into(),
            description: "Describe capabilities".into(),
            input_schema: serde_json::json!({}),
            namespace: "agent".into(),
            required_boundary_action: Some(tool_boundary_action(
                "agent.describe",
                "agent",
                CapabilityActionKind::Discover,
            )),
        };

        let handler: ToolHandler =
            Arc::new(|_| Box::pin(async move { Ok(serde_json::json!({"unexpected": true})) }));

        server.register_tool(tool, handler).await;

        let err = server
            .handle_tools_call("agent.describe", serde_json::json!({}))
            .await
            .unwrap_err();

        assert!(matches!(
            err,
            McpError::ToolCallFailed(message)
                if message.contains("delegation envelope required for MCP tool 'agent.describe'")
        ));
    }

    #[tokio::test]
    async fn required_boundary_action_accepts_matching_discover_delegation() {
        let action =
            tool_boundary_action("agent.describe", "agent", CapabilityActionKind::Discover);
        let (service, delegation) = sample_external_delegation_for_action(action.clone());
        let server = McpServer::new(test_config()).with_delegation_service(service);

        let tool = ExposedTool {
            name: "describe".into(),
            description: "Describe capabilities".into(),
            input_schema: serde_json::json!({}),
            namespace: "agent".into(),
            required_boundary_action: Some(action),
        };

        let handler: ToolHandler = Arc::new(|request| {
            Box::pin(async move {
                Ok(serde_json::json!({
                    "descriptor_id": request
                        .context
                        .delegation
                        .as_ref()
                        .and_then(ExternalDelegationEnvelope::descriptor_id),
                }))
            })
        });

        server.register_tool(tool, handler).await;

        let result = server
            .handle_tools_call(
                "agent.describe",
                ToolCallRequest::new(serde_json::json!({}))
                    .with_delegation(delegation)
                    .into_wire_params(),
            )
            .await
            .unwrap();

        assert_eq!(result["descriptor_id"], "tool:agent.describe");
    }

    #[tokio::test]
    async fn call_unknown_tool_returns_error() {
        let server = McpServer::new(test_config());
        let result = server
            .handle_tools_call("agent.missing", serde_json::json!({}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn server_lifecycle() {
        let server = McpServer::new(test_config());
        assert!(!server.is_running().await);
        server.start().await.unwrap();
        assert!(server.is_running().await);
        server.stop().await.unwrap();
        assert!(!server.is_running().await);
    }
}
