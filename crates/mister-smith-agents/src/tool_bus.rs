use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use mister_smith_core::{AgentId, Tool, ToolError};
use mister_smith_mcp::client::McpClient;
use mister_smith_mcp::errors::McpError;
use mister_smith_security::audit::{AuditEventType, AuditLogger, AuditOutcome, SecurityAuditEvent};
use mister_smith_security::jwt::AgentClaims;
use mister_smith_security::rbac::{AuthorizationRequest, PolicyDecision, PolicyEngine};
use serde::{Deserialize, Serialize};

use crate::errors::AgentSystemError;

type ToolKey = (String, String);

/// Authenticated caller context for tool discovery and invocation.
#[derive(Debug, Clone)]
pub struct ToolPrincipal {
    pub agent_id: AgentId,
    pub claims: AgentClaims,
}

impl ToolPrincipal {
    pub fn new(agent_id: AgentId, claims: AgentClaims) -> Self {
        Self { agent_id, claims }
    }
}

#[derive(Clone)]
enum ToolBackend {
    Native(Arc<dyn Tool>),
    Mcp(Arc<McpClient>),
}

/// Registry entry for a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEntry {
    pub name: String,
    pub namespace: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub agent_id: Option<AgentId>,
    pub mcp_session: Option<String>,
    pub registered_at: chrono::DateTime<chrono::Utc>,
    #[serde(with = "crate::config::humantime_serde")]
    pub timeout: Duration,
}

/// Metrics for a registered tool.
#[derive(Debug, Default, Clone)]
pub struct ToolMetrics {
    pub invocation_count: u64,
    pub error_count: u64,
    pub total_latency_ms: u64,
}

/// Central tool registry and invocation proxy.
pub struct ToolBus {
    tools: Arc<DashMap<ToolKey, ToolEntry>>,
    backends: Arc<DashMap<ToolKey, ToolBackend>>,
    metrics: Arc<DashMap<ToolKey, ToolMetrics>>,
    policy_engine: Option<Arc<PolicyEngine>>,
    audit_logger: Option<Arc<AuditLogger>>,
}

impl ToolBus {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(DashMap::new()),
            backends: Arc::new(DashMap::new()),
            metrics: Arc::new(DashMap::new()),
            policy_engine: None,
            audit_logger: None,
        }
    }

    pub fn with_security(
        policy_engine: Option<Arc<PolicyEngine>>,
        audit_logger: Option<Arc<AuditLogger>>,
    ) -> Self {
        Self {
            tools: Arc::new(DashMap::new()),
            backends: Arc::new(DashMap::new()),
            metrics: Arc::new(DashMap::new()),
            policy_engine,
            audit_logger,
        }
    }

    /// Register a native agent-backed tool entry without attaching an invocable backend.
    pub fn register(
        &self,
        name: impl Into<String>,
        namespace: impl Into<String>,
        agent_id: AgentId,
        description: impl Into<String>,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
    ) {
        self.insert_entry(
            name.into(),
            namespace.into(),
            Some(agent_id),
            None,
            description.into(),
            input_schema,
            output_schema,
            None,
        );
    }

    /// Register an invocable native tool.
    #[allow(clippy::too_many_arguments)]
    pub fn register_native_tool(
        &self,
        name: impl Into<String>,
        namespace: impl Into<String>,
        agent_id: AgentId,
        description: impl Into<String>,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
        tool: Arc<dyn Tool>,
    ) {
        self.insert_entry(
            name.into(),
            namespace.into(),
            Some(agent_id),
            None,
            description.into(),
            input_schema,
            output_schema,
            Some(ToolBackend::Native(tool)),
        );
    }

    /// Register an MCP-backed tool entry without attaching an invocable backend.
    pub fn register_mcp(
        &self,
        name: impl Into<String>,
        namespace: impl Into<String>,
        mcp_session: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
    ) {
        self.insert_entry(
            name.into(),
            namespace.into(),
            None,
            Some(mcp_session.into()),
            description.into(),
            input_schema,
            output_schema,
            None,
        );
    }

    /// Register an invocable MCP-backed tool.
    #[allow(clippy::too_many_arguments)]
    pub fn register_mcp_tool(
        &self,
        name: impl Into<String>,
        namespace: impl Into<String>,
        mcp_session: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
        client: Arc<McpClient>,
    ) {
        self.insert_entry(
            name.into(),
            namespace.into(),
            None,
            Some(mcp_session.into()),
            description.into(),
            input_schema,
            output_schema,
            Some(ToolBackend::Mcp(client)),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_entry(
        &self,
        name: String,
        namespace: String,
        agent_id: Option<AgentId>,
        mcp_session: Option<String>,
        description: String,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
        backend: Option<ToolBackend>,
    ) {
        let key = (namespace.clone(), name.clone());
        self.tools.insert(
            key.clone(),
            ToolEntry {
                name,
                namespace,
                description,
                input_schema,
                output_schema,
                agent_id,
                mcp_session,
                registered_at: chrono::Utc::now(),
                timeout: Duration::from_secs(30),
            },
        );

        if let Some(backend) = backend {
            self.backends.insert(key, backend);
        }
    }

    /// Deregister a tool.
    pub fn deregister(&self, namespace: &str, name: &str) -> bool {
        let key = (namespace.to_string(), name.to_string());
        self.backends.remove(&key);
        self.metrics.remove(&key);
        self.tools.remove(&key).is_some()
    }

    /// Discover tools matching a filter. Returns all matching tools when permissions allow it.
    pub fn discover(
        &self,
        principal: Option<&ToolPrincipal>,
        namespace_filter: Option<&str>,
    ) -> Result<Vec<ToolEntry>, AgentSystemError> {
        let mut visible = Vec::new();

        for entry in self.tools.iter() {
            let tool = entry.value();
            if namespace_filter
                .map(|namespace| tool.namespace != namespace)
                .unwrap_or(false)
            {
                continue;
            }

            let decision = self.evaluate_authorization(
                principal,
                "discover",
                &tool.namespace,
                Some(&tool.name),
            )?;
            if decision
                .as_ref()
                .map(|decision| decision.allowed)
                .unwrap_or(true)
            {
                visible.push(tool.clone());
            }
        }

        Ok(visible)
    }

    /// Look up a specific tool.
    pub fn find(&self, namespace: &str, name: &str) -> Option<ToolEntry> {
        let key = (namespace.to_string(), name.to_string());
        self.tools.get(&key).map(|entry| entry.clone())
    }

    /// Invoke a registered tool through the ToolBus execution boundary.
    pub async fn invoke(
        &self,
        principal: Option<&ToolPrincipal>,
        namespace: &str,
        name: &str,
        params: serde_json::Value,
        timeout: Option<Duration>,
    ) -> Result<serde_json::Value, AgentSystemError> {
        let key = (namespace.to_string(), name.to_string());
        let entry = self
            .tools
            .get(&key)
            .map(|tool| tool.clone())
            .ok_or_else(|| {
                AgentSystemError::Tool(ToolError::NotFound(format!("{namespace}.{name}")))
            })?;

        if let Some(decision) =
            self.evaluate_authorization(principal, "execute", namespace, Some(name))?
        {
            if !decision.allowed {
                self.record_audit_event(
                    principal,
                    namespace,
                    name,
                    AuditOutcome::Blocked,
                    "execute_tool",
                    Some(&decision.reason),
                    None,
                );
                return Err(AgentSystemError::PermissionDenied(decision.reason));
            }
        }

        let backend = self
            .backends
            .get(&key)
            .map(|backend| backend.clone())
            .ok_or_else(|| AgentSystemError::ToolUnavailable(format!("{namespace}.{name}")))?;

        let deadline = timeout.unwrap_or(entry.timeout);
        let started = Instant::now();
        let result = match backend {
            ToolBackend::Native(tool) => {
                match tokio::time::timeout(deadline, tool.execute(params)).await {
                    Ok(inner) => inner.map_err(AgentSystemError::from),
                    Err(_) => Err(AgentSystemError::Timeout(format!(
                        "tool '{namespace}.{name}' timed out after {deadline:?}"
                    ))),
                }
            }
            ToolBackend::Mcp(client) => {
                match tokio::time::timeout(deadline, client.call_tool(name, params)).await {
                    Ok(inner) => inner.map_err(Self::map_mcp_error),
                    Err(_) => Err(AgentSystemError::Timeout(format!(
                        "tool '{namespace}.{name}' timed out after {deadline:?}"
                    ))),
                }
            }
        };

        let latency = started.elapsed();
        self.record_invocation(namespace, name, latency, result.is_ok());

        match result {
            Ok(value) => {
                self.record_audit_event(
                    principal,
                    namespace,
                    name,
                    AuditOutcome::Success,
                    "invoke_tool",
                    None,
                    Some(latency),
                );
                Ok(value)
            }
            Err(err) => {
                self.record_audit_event(
                    principal,
                    namespace,
                    name,
                    AuditOutcome::Failure,
                    "invoke_tool",
                    Some(&err.to_string()),
                    Some(latency),
                );
                Err(err)
            }
        }
    }

    fn evaluate_authorization(
        &self,
        principal: Option<&ToolPrincipal>,
        action: &str,
        namespace: &str,
        name: Option<&str>,
    ) -> Result<Option<PolicyDecision>, AgentSystemError> {
        let Some(policy_engine) = &self.policy_engine else {
            return Ok(None);
        };

        let principal = principal.ok_or_else(|| {
            AgentSystemError::PermissionDenied(format!(
                "authenticated principal required for tool {action}"
            ))
        })?;

        let mut context = HashMap::new();
        context.insert("scope".to_string(), namespace.to_string());
        if let Some(name) = name {
            context.insert("tool_name".to_string(), name.to_string());
        }

        Ok(Some(policy_engine.evaluate(&AuthorizationRequest {
            principal: principal.claims.clone(),
            action: action.to_string(),
            resource: "tool".to_string(),
            resource_id: name.map(|name| format!("{namespace}.{name}")),
            context,
        })))
    }

    #[allow(clippy::too_many_arguments)]
    fn record_audit_event(
        &self,
        principal: Option<&ToolPrincipal>,
        namespace: &str,
        name: &str,
        outcome: AuditOutcome,
        action: &str,
        error: Option<&str>,
        latency: Option<Duration>,
    ) {
        let Some(audit_logger) = &self.audit_logger else {
            return;
        };

        let mut details = HashMap::new();
        details.insert("namespace".to_string(), namespace.to_string());
        details.insert("tool_name".to_string(), name.to_string());
        if let Some(error) = error {
            details.insert("error".to_string(), error.to_string());
        }
        if let Some(latency) = latency {
            details.insert("latency_ms".to_string(), latency.as_millis().to_string());
        }

        audit_logger.record(SecurityAuditEvent {
            event_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            event_type: AuditEventType::Authorization,
            principal: principal.map(|principal| principal.agent_id.to_string()),
            resource: Some(format!("tool:{namespace}.{name}")),
            action: Some(action.to_string()),
            outcome,
            details,
            source_ip: None,
            previous_hash: None,
        });
    }

    fn map_mcp_error(error: McpError) -> AgentSystemError {
        match error {
            McpError::ToolNotFound(message) => AgentSystemError::Tool(ToolError::NotFound(message)),
            McpError::BridgeTimeout(message) => AgentSystemError::Timeout(message),
            McpError::ConnectionFailed(message) | McpError::SessionError(message) => {
                AgentSystemError::ToolUnavailable(message)
            }
            McpError::SerializationError(message) => AgentSystemError::Serialization(message),
            McpError::ToolCallFailed(message) | McpError::NamespaceConflict(message) => {
                AgentSystemError::ToolBusError(message)
            }
        }
    }

    /// Record an invocation metric.
    pub fn record_invocation(&self, namespace: &str, name: &str, latency: Duration, success: bool) {
        let key = (namespace.to_string(), name.to_string());
        let mut metrics = self.metrics.entry(key).or_default();
        metrics.invocation_count += 1;
        metrics.total_latency_ms += latency.as_millis() as u64;
        if !success {
            metrics.error_count += 1;
        }
    }

    /// Get metrics for a tool.
    pub fn get_metrics(&self, namespace: &str, name: &str) -> Option<ToolMetrics> {
        let key = (namespace.to_string(), name.to_string());
        self.metrics.get(&key).map(|metrics| metrics.clone())
    }

    /// Get count of registered tools.
    pub fn count(&self) -> usize {
        self.tools.len()
    }

    /// Export registered tools as provider-neutral [`mister_smith_llm::ToolDefinition`]s
    /// for inclusion in LLM completion requests.
    #[cfg(feature = "llm")]
    pub fn to_tool_definitions(&self) -> Vec<mister_smith_llm::ToolDefinition> {
        self.tools
            .iter()
            .map(|entry| {
                let tool = entry.value();
                mister_smith_llm::ToolDefinition {
                    name: format!("{}.{}", tool.namespace, tool.name),
                    description: tool.description.clone(),
                    input_schema: tool.input_schema.clone(),
                }
            })
            .collect()
    }

    /// Execute a model-emitted [`mister_smith_llm::ToolCall`] through the ToolBus,
    /// preserving all existing security, timeout, metrics, and audit boundaries.
    ///
    /// Returns `Ok(ToolResult)` in both success and tool-error cases (the error is
    /// captured inside `ToolResult::error`). Returns `Err` only for structural
    /// problems such as an invalid tool name format.
    #[cfg(feature = "llm")]
    pub async fn execute_tool_call(
        &self,
        principal: Option<&ToolPrincipal>,
        call: &mister_smith_llm::ToolCall,
    ) -> Result<mister_smith_llm::ToolResult, AgentSystemError> {
        // Parse namespace.name from call.name
        let (namespace, name) = call.name.split_once('.').ok_or_else(|| {
            AgentSystemError::ToolBusError(format!(
                "Tool call name '{}' must be in 'namespace.name' format",
                call.name
            ))
        })?;

        // Delegate to existing invoke() with same security/timeout/metrics boundaries
        match self
            .invoke(principal, namespace, name, call.input.clone(), None)
            .await
        {
            Ok(output) => Ok(mister_smith_llm::ToolResult::success(
                call.call_id.clone(),
                output,
            )),
            Err(err) => Ok(mister_smith_llm::ToolResult::failure(
                call.call_id.clone(),
                err.to_string(),
            )),
        }
    }
}

impl Default for ToolBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_discover() {
        let bus = ToolBus::new();
        let agent_id = AgentId::new();

        bus.register(
            "analyzer",
            "data",
            agent_id,
            "Analyzes data",
            serde_json::json!({}),
            serde_json::json!({}),
        );

        assert_eq!(bus.count(), 1);
        let tools = bus.discover(None, Some("data")).unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "analyzer");
    }

    #[test]
    fn test_register_mcp() {
        let bus = ToolBus::new();
        bus.register_mcp(
            "web_search",
            "search",
            "session-123",
            "Web search tool",
            serde_json::json!({}),
            serde_json::json!({}),
        );

        let tool = bus.find("search", "web_search").unwrap();
        assert!(tool.mcp_session.is_some());
        assert!(tool.agent_id.is_none());
    }

    #[test]
    fn test_deregister() {
        let bus = ToolBus::new();
        bus.register(
            "tool1",
            "ns",
            AgentId::new(),
            "desc",
            serde_json::json!({}),
            serde_json::json!({}),
        );

        assert!(bus.deregister("ns", "tool1"));
        assert_eq!(bus.count(), 0);
        assert!(!bus.deregister("ns", "tool1"));
    }

    #[test]
    fn test_metrics() {
        let bus = ToolBus::new();
        bus.record_invocation("ns", "tool1", Duration::from_millis(50), true);
        bus.record_invocation("ns", "tool1", Duration::from_millis(100), false);

        let metrics = bus.get_metrics("ns", "tool1").unwrap();
        assert_eq!(metrics.invocation_count, 2);
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.total_latency_ms, 150);
    }
}
