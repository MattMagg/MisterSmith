use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use mister_smith_core::{AgentId, DelegationScope, ExecutionBranchId, TaskId, Tool, ToolError};
use mister_smith_events::{AutonomyEvent, AutonomyEventEnvelope, CapabilitySummary, EventBus};
use mister_smith_mcp::client::McpClient;
use mister_smith_mcp::errors::McpError;
use mister_smith_security::audit::{
    AuditEventType, AuditLogger, AuditOutcome, DelegationAuditContext, SecurityAuditEvent,
};
use mister_smith_security::jwt::AgentClaims;
use mister_smith_security::rbac::{AuthorizationRequest, PolicyDecision, PolicyEngine};
use mister_smith_security::{DelegationService, ValidatedDelegation};
use serde::{Deserialize, Serialize};

use crate::errors::AgentSystemError;

type ToolKey = (String, String);

/// Authenticated caller context for tool discovery and invocation.
#[derive(Debug, Clone)]
pub struct ToolPrincipal {
    pub agent_id: AgentId,
    pub claims: AgentClaims,
    pub workflow_id: Option<TaskId>,
    pub branch_id: Option<ExecutionBranchId>,
    pub required_delegation_scope: Option<DelegationScope>,
}

impl ToolPrincipal {
    pub fn new(agent_id: AgentId, claims: AgentClaims) -> Self {
        Self {
            agent_id,
            claims,
            workflow_id: None,
            branch_id: None,
            required_delegation_scope: None,
        }
    }

    pub fn with_workflow(mut self, workflow_id: TaskId) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }

    pub fn with_branch(mut self, branch_id: ExecutionBranchId) -> Self {
        self.branch_id = Some(branch_id);
        self
    }

    pub fn requiring_delegation(mut self, scope: DelegationScope) -> Self {
        self.required_delegation_scope = Some(scope);
        self
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
    delegation_service: Option<Arc<DelegationService>>,
    event_bus: Option<Arc<EventBus>>,
}

impl ToolBus {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(DashMap::new()),
            backends: Arc::new(DashMap::new()),
            metrics: Arc::new(DashMap::new()),
            policy_engine: None,
            audit_logger: None,
            delegation_service: None,
            event_bus: None,
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
            delegation_service: None,
            event_bus: None,
        }
    }

    pub fn with_delegation_service(mut self, delegation_service: Arc<DelegationService>) -> Self {
        self.delegation_service = Some(delegation_service);
        self
    }

    pub fn with_event_bus(mut self, event_bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(event_bus);
        self
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
                    None,
                );
                return Err(AgentSystemError::PermissionDenied(decision.reason));
            }
        }

        let delegation_validation = match self.validate_delegation(principal).await {
            Ok(validation) => validation,
            Err((error, summary)) => {
                let delegation_context = summary.as_ref().map(delegation_audit_context);
                self.record_audit_event(
                    principal,
                    namespace,
                    name,
                    AuditOutcome::Blocked,
                    "validate_delegation",
                    Some(&error.to_string()),
                    None,
                    delegation_context,
                );
                if let Some(summary) = summary {
                    self.publish_delegation_update(principal, summary).await;
                }
                return Err(AgentSystemError::PermissionDenied(error.to_string()));
            }
        };

        if let Some(summary) = delegation_validation
            .as_ref()
            .map(|validated| capability_summary(validated, None))
        {
            self.publish_delegation_update(principal, summary).await;
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
                    delegation_validation.as_ref().map(|validated| {
                        delegation_audit_context(&capability_summary(validated, None))
                    }),
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
                    delegation_validation.as_ref().map(|validated| {
                        delegation_audit_context(&capability_summary(validated, None))
                    }),
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

    async fn validate_delegation(
        &self,
        principal: Option<&ToolPrincipal>,
    ) -> Result<
        Option<ValidatedDelegation>,
        (
            mister_smith_core::DelegationError,
            Option<CapabilitySummary>,
        ),
    > {
        let Some(principal) = principal else {
            return Ok(None);
        };
        let Some(required_scope) = principal.required_delegation_scope else {
            return Ok(None);
        };
        let Some(delegation_service) = &self.delegation_service else {
            return Err((
                mister_smith_core::DelegationError::InvalidChain(
                    "delegation service is required for privileged tool execution".to_string(),
                ),
                None,
            ));
        };

        match delegation_service.validate_claims(&principal.claims, Some(required_scope)) {
            Ok(Some(validated)) => Ok(Some(validated)),
            Ok(None) => Err((
                mister_smith_core::DelegationError::InvalidChain(
                    "privileged tool execution requires a bounded delegation capability"
                        .to_string(),
                ),
                None,
            )),
            Err(error) => {
                let summary = capability_summary_from_claims_error(&principal.claims, &error);
                Err((error, summary))
            }
        }
    }

    async fn publish_delegation_update(
        &self,
        principal: Option<&ToolPrincipal>,
        payload: CapabilitySummary,
    ) {
        let Some(event_bus) = &self.event_bus else {
            return;
        };
        let Some(principal) = principal else {
            return;
        };
        let Some(workflow_id) = principal.workflow_id else {
            return;
        };

        let event = AutonomyEvent::DelegationUpdated(AutonomyEventEnvelope {
            workflow_id,
            graph_id: None,
            branch_id: principal.branch_id,
            payload,
            operator_visible: true,
        });

        if let Err(error) = event_bus
            .publish(event.into_event("mister-smith-agents::tool-bus"))
            .await
        {
            tracing::warn!(%error, "failed to publish delegation update from tool bus");
        }
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
        delegation: Option<DelegationAuditContext>,
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
            event_type: if delegation.is_some() {
                AuditEventType::Delegation
            } else {
                AuditEventType::Authorization
            },
            principal: principal.map(|principal| principal.agent_id.to_string()),
            resource: Some(format!("tool:{namespace}.{name}")),
            action: Some(action.to_string()),
            outcome,
            details,
            delegation,
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
    /// Returns `Ok(ToolResult)` only for successful execution payloads.
    ///
    /// Structural and execution-boundary failures (invalid tool name format,
    /// authorization denial, timeout, missing/unavailable tool, etc.) are
    /// returned as `Err(AgentSystemError)` to preserve ToolBus semantics.
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

        // Delegate to existing invoke() with same security/timeout/metrics boundaries.
        // Boundary failures intentionally propagate as Err to preserve ToolBus semantics.
        let output = self
            .invoke(principal, namespace, name, call.input.clone(), None)
            .await?;
        Ok(mister_smith_llm::ToolResult::success(
            call.call_id.clone(),
            output,
        ))
    }

    /// Provider-facing adapter that preserves protocol expectations of always
    /// returning a [`mister_smith_llm::ToolResult`], even on ToolBus failures.
    #[cfg(feature = "llm")]
    pub async fn execute_tool_call_provider_result(
        &self,
        principal: Option<&ToolPrincipal>,
        call: &mister_smith_llm::ToolCall,
    ) -> mister_smith_llm::ToolResult {
        match self.execute_tool_call(principal, call).await {
            Ok(result) => result,
            Err(err) => {
                mister_smith_llm::ToolResult::failure(call.call_id.clone(), err.to_string())
            }
        }
    }
}

fn capability_summary(
    validated: &ValidatedDelegation,
    rejection_reason: Option<String>,
) -> CapabilitySummary {
    CapabilitySummary {
        capability_id: validated.capability.capability_id,
        issuer: validated.capability.issuer.clone(),
        recipient: validated.capability.recipient,
        scope: validated.capability.scope,
        parent_capability: validated.capability.parent_capability,
        expires_at: validated.capability.expires_at,
        provenance: validated.provenance.clone(),
        revocation_state: validated.capability.revocation_state,
        rejection_reason,
    }
}

fn capability_summary_from_claims_error(
    claims: &AgentClaims,
    error: &mister_smith_core::DelegationError,
) -> Option<CapabilitySummary> {
    let capability = claims.delegation_capability.clone()?;
    let provenance = claims.provenance_chain.clone()?;
    let revocation_state = match error {
        mister_smith_core::DelegationError::Revoked { .. } => {
            mister_smith_core::RevocationState::Revoked
        }
        mister_smith_core::DelegationError::Expired { .. } => {
            mister_smith_core::RevocationState::Expired
        }
        _ => capability.revocation_state,
    };

    Some(CapabilitySummary {
        capability_id: capability.capability_id,
        issuer: capability.issuer,
        recipient: capability.recipient,
        scope: capability.scope,
        parent_capability: capability.parent_capability,
        expires_at: capability.expires_at,
        provenance,
        revocation_state,
        rejection_reason: Some(error.to_string()),
    })
}

fn delegation_audit_context(summary: &CapabilitySummary) -> DelegationAuditContext {
    DelegationAuditContext {
        capability_id: Some(summary.capability_id),
        parent_capability: summary.parent_capability,
        issuer: Some(summary.issuer.clone()),
        recipient: Some(summary.recipient.to_string()),
        scope: Some(summary.scope),
        revocation_state: Some(summary.revocation_state),
        expires_at: Some(summary.expires_at),
        rejection_reason: summary.rejection_reason.clone(),
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
