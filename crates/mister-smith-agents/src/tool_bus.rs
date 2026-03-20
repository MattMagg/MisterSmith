use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use dashmap::DashMap;
use mister_smith_core::{
    AgentId, CapabilityActionKind, DelegatedAction, DelegatedActionPolicy, DelegationScope,
    ExecutionBranchId, RevocationState, TaskId, Tool, ToolError,
};
use mister_smith_events::{
    AutonomyEvent, AutonomyEventEnvelope, CapabilitySummary, EventBus,
    ExternalCapabilityDecisionOutcome, ExternalCapabilityDecisionSummary,
    ExternalCapabilityDecisionSurface,
};
use mister_smith_mcp::client::McpClient;
use mister_smith_mcp::errors::McpError;
use mister_smith_mcp::ToolCallRequest;
use mister_smith_security::audit::{
    AuditEventType, AuditLogger, AuditOutcome, DelegationAuditContext, SecurityAuditEvent,
};
use mister_smith_security::delegation::external_delegation_envelope;
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

/// Discoverable descriptor for a local or remote tool capability surface.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityDescriptor {
    /// Stable capability descriptor identifier.
    pub descriptor_id: String,
    /// Human-readable title for the surface.
    pub title: String,
    /// Human-readable description for the surface.
    pub description: String,
    /// Local backing agent for the surface, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_agent_id: Option<AgentId>,
    /// Typed actions available through the capability surface.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<DelegatedAction>,
}

impl CapabilityDescriptor {
    fn for_tool(
        namespace: &str,
        name: &str,
        local_agent_id: Option<AgentId>,
        required_scope: Option<DelegationScope>,
    ) -> Self {
        let descriptor_id = format!("tool:{namespace}.{name}");
        let descriptor_key = descriptor_id.clone();
        let resource_id = format!("{namespace}.{name}");
        let title = resource_id.clone();
        let description = match local_agent_id {
            Some(agent_id) => {
                format!("Local tool capability for {resource_id} owned by {agent_id}")
            }
            None => format!("Tool capability for {resource_id}"),
        };

        let action = |kind: CapabilityActionKind, required_scope| DelegatedAction {
            descriptor_id: descriptor_key.clone(),
            action_id: format!("{descriptor_key}#{}", kind.policy_action()),
            title: format!("{} {resource_id}", kind.policy_action()),
            description: format!("{} access for tool {resource_id}", kind.policy_action()),
            kind,
            policy: DelegatedActionPolicy {
                action: kind.policy_action().to_string(),
                resource: "tool".to_string(),
                scope: namespace.to_string(),
                resource_id: Some(resource_id.clone()),
            },
            required_scope,
            revocation_key: format!("{descriptor_key}#{}", kind.policy_action()),
        };

        Self {
            descriptor_id,
            title,
            description,
            local_agent_id,
            actions: vec![
                action(CapabilityActionKind::Discover, None),
                action(CapabilityActionKind::Execute, required_scope),
            ],
        }
    }

    fn action(&self, kind: CapabilityActionKind) -> Option<&DelegatedAction> {
        self.actions.iter().find(|action| action.kind == kind)
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
    #[serde(default)]
    pub capability_descriptor: CapabilityDescriptor,
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
            None,
        );
    }

    /// Register an invocable native tool that requires a privileged delegation scope.
    #[allow(clippy::too_many_arguments)]
    pub fn register_privileged_native_tool(
        &self,
        name: impl Into<String>,
        namespace: impl Into<String>,
        agent_id: AgentId,
        description: impl Into<String>,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
        required_scope: DelegationScope,
        tool: Arc<dyn Tool>,
    ) {
        let name = name.into();
        let namespace = namespace.into();
        let capability_descriptor =
            CapabilityDescriptor::for_tool(&namespace, &name, Some(agent_id), Some(required_scope));
        self.insert_entry(
            name,
            namespace,
            Some(agent_id),
            None,
            description.into(),
            input_schema,
            output_schema,
            Some(ToolBackend::Native(tool)),
            Some(capability_descriptor),
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
            None,
        );
    }

    /// Register an invocable MCP-backed tool that requires a privileged delegation scope.
    #[allow(clippy::too_many_arguments)]
    pub fn register_privileged_mcp_tool(
        &self,
        name: impl Into<String>,
        namespace: impl Into<String>,
        mcp_session: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
        output_schema: serde_json::Value,
        required_scope: DelegationScope,
        client: Arc<McpClient>,
    ) {
        let name = name.into();
        let namespace = namespace.into();
        let mcp_session = mcp_session.into();
        let capability_descriptor =
            CapabilityDescriptor::for_tool(&namespace, &name, None, Some(required_scope));
        self.insert_entry(
            name,
            namespace,
            None,
            Some(mcp_session),
            description.into(),
            input_schema,
            output_schema,
            Some(ToolBackend::Mcp(client)),
            Some(capability_descriptor),
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
        capability_descriptor: Option<CapabilityDescriptor>,
    ) {
        let key = (namespace.clone(), name.clone());
        let capability_descriptor = capability_descriptor
            .unwrap_or_else(|| CapabilityDescriptor::for_tool(&namespace, &name, agent_id, None));
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
                capability_descriptor,
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

            let decision = if let Some(action) = tool
                .capability_descriptor
                .action(CapabilityActionKind::Discover)
            {
                self.evaluate_authorization_for_action(principal, action)?
            } else {
                self.evaluate_authorization(
                    principal,
                    "discover",
                    &tool.namespace,
                    Some(&tool.name),
                )?
            };
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

        let execute_action = self.resolve_execute_action(&entry, principal)?;
        let authorization_decision = if let Some(action) = execute_action.as_ref() {
            self.evaluate_authorization_for_action(principal, action)?
        } else {
            self.evaluate_authorization(principal, "execute", namespace, Some(name))?
        };
        if let Some(decision) = authorization_decision.as_ref() {
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
                if let Some(summary) = principal.and_then(|principal| {
                    execute_action.as_ref().map(|action| {
                        external_capability_decision_from_claims(
                            principal.branch_id,
                            &principal.claims,
                            action,
                            ExternalCapabilityDecisionOutcome::Rejected,
                            Some(decision),
                            &decision.reason,
                            None,
                        )
                    })
                }) {
                    self.publish_external_capability_decision(principal, summary)
                        .await;
                }
                return Err(AgentSystemError::PermissionDenied(decision.reason.clone()));
            }
        }

        let delegation_validation = match self
            .validate_delegation(principal, execute_action.as_ref())
            .await
        {
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
                if let Some(summary) = principal.and_then(|principal| {
                    execute_action.as_ref().map(|action| {
                        external_capability_decision_from_claims(
                            principal.branch_id,
                            &principal.claims,
                            action,
                            ExternalCapabilityDecisionOutcome::Rejected,
                            authorization_decision.as_ref(),
                            &error.to_string(),
                            match &error {
                                mister_smith_core::DelegationError::Revoked { .. } => {
                                    Some(RevocationState::Revoked)
                                }
                                mister_smith_core::DelegationError::Expired { .. } => {
                                    Some(RevocationState::Expired)
                                }
                                _ => principal
                                    .claims
                                    .delegation_capability
                                    .as_ref()
                                    .map(|capability| capability.revocation_state),
                            },
                        )
                    })
                }) {
                    self.publish_external_capability_decision(principal, summary)
                        .await;
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
        if let Some(summary) = delegation_validation.as_ref().and_then(|validated| {
            execute_action.as_ref().map(|action| {
                external_capability_decision_from_validated(
                    principal.and_then(|principal| principal.branch_id),
                    validated,
                    action,
                    authorization_decision.as_ref(),
                )
            })
        }) {
            self.publish_external_capability_decision(principal, summary)
                .await;
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
                let request = delegation_validation
                    .as_ref()
                    .map(|validated| {
                        let mut request = ToolCallRequest::new(params.clone());
                        if let Some(action) = execute_action.as_ref() {
                            request = request.with_delegation(external_delegation_envelope(
                                validated,
                                Some(action),
                            ));
                        }
                        request
                    })
                    .unwrap_or_else(|| ToolCallRequest::new(params));

                match tokio::time::timeout(deadline, client.call_tool_request(name, request)).await
                {
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

    fn evaluate_authorization_for_action(
        &self,
        principal: Option<&ToolPrincipal>,
        action: &DelegatedAction,
    ) -> Result<Option<PolicyDecision>, AgentSystemError> {
        let Some(policy_engine) = &self.policy_engine else {
            return Ok(None);
        };

        let principal = principal.ok_or_else(|| {
            AgentSystemError::PermissionDenied(format!(
                "authenticated principal required for tool {}",
                action.policy.action
            ))
        })?;

        let mut context = HashMap::new();
        context.insert("scope".to_string(), action.policy.scope.clone());
        context.insert("descriptor_id".to_string(), action.descriptor_id.clone());
        context.insert("action_id".to_string(), action.action_id.clone());

        Ok(Some(policy_engine.evaluate(&AuthorizationRequest {
            principal: principal.claims.clone(),
            action: action.policy.action.clone(),
            resource: action.policy.resource.clone(),
            resource_id: action.policy.resource_id.clone(),
            context,
        })))
    }

    async fn validate_delegation(
        &self,
        principal: Option<&ToolPrincipal>,
        delegated_action: Option<&DelegatedAction>,
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
        let Some(delegated_action) = delegated_action else {
            return Ok(None);
        };
        let Some(_required_scope) = delegated_action.required_scope else {
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

        match delegation_service.validate_claims_for_action(&principal.claims, delegated_action) {
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

    fn resolve_execute_action(
        &self,
        entry: &ToolEntry,
        principal: Option<&ToolPrincipal>,
    ) -> Result<Option<DelegatedAction>, AgentSystemError> {
        let action = entry
            .capability_descriptor
            .action(CapabilityActionKind::Execute)
            .cloned();

        let Some(existing_scope) = action.as_ref().and_then(|action| action.required_scope) else {
            return Ok(None);
        };

        if let Some(required_scope) =
            principal.and_then(|principal| principal.required_delegation_scope)
        {
            if existing_scope != required_scope {
                return Err(AgentSystemError::PermissionDenied(format!(
                    "tool '{}.{}' requires delegation scope {:?} but principal requested {:?}",
                    entry.namespace, entry.name, existing_scope, required_scope
                )));
            }
        }

        Ok(action)
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

    async fn publish_external_capability_decision(
        &self,
        principal: Option<&ToolPrincipal>,
        payload: ExternalCapabilityDecisionSummary,
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

        let event = AutonomyEvent::DelegationDecisionRecorded(AutonomyEventEnvelope {
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
            tracing::warn!(
                %error,
                "failed to publish external capability decision from tool bus"
            );
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
        descriptor_id: validated.capability.descriptor_id.clone(),
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

fn external_capability_decision_from_validated(
    branch_id: Option<ExecutionBranchId>,
    validated: &ValidatedDelegation,
    action: &DelegatedAction,
    policy_decision: Option<&PolicyDecision>,
) -> ExternalCapabilityDecisionSummary {
    let capability = &validated.capability;
    let mut rationale = Vec::new();
    if let Some(descriptor_id) = capability.descriptor_id.as_deref() {
        rationale.push(format!(
            "descriptor '{descriptor_id}' matched the requested external action"
        ));
    }
    if let Some(required_scope) = action.required_scope {
        rationale.push(format!(
            "required scope {:?} matched capability scope {:?}",
            required_scope, capability.scope
        ));
    }
    if let Some(decision) = policy_decision {
        rationale.push(format!("policy engine reason: {}", decision.reason));
    }
    rationale.push(format!(
        "authority chain depth {} remained {:?} at the boundary",
        validated.provenance.links.len(),
        capability.revocation_state
    ));

    ExternalCapabilityDecisionSummary {
        boundary_surface: Some(ExternalCapabilityDecisionSurface::ToolBus),
        branch_id,
        capability_id: Some(capability.capability_id),
        capability_descriptor_id: capability.descriptor_id.clone(),
        action_descriptor_id: Some(action.descriptor_id.clone()),
        action_id: Some(action.action_id.clone()),
        action_title: Some(action.title.clone()),
        scope: Some(capability.scope),
        required_scope: action.required_scope,
        policy_action: Some(action.policy.action.clone()),
        policy_resource: Some(action.policy.resource.clone()),
        policy_scope: Some(action.policy.scope.clone()),
        policy_resource_id: action.policy.resource_id.clone(),
        revocation_state: Some(capability.revocation_state),
        chain_depth: validated.provenance.links.len(),
        outcome: ExternalCapabilityDecisionOutcome::Allowed,
        observed_at: Some(Utc::now()),
        rationale,
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
        descriptor_id: capability.descriptor_id.clone(),
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

fn external_capability_decision_from_claims(
    branch_id: Option<ExecutionBranchId>,
    claims: &AgentClaims,
    action: &DelegatedAction,
    outcome: ExternalCapabilityDecisionOutcome,
    policy_decision: Option<&PolicyDecision>,
    reason: &str,
    revocation_state_override: Option<RevocationState>,
) -> ExternalCapabilityDecisionSummary {
    let capability = claims.delegation_capability.as_ref();
    let chain_depth = claims
        .provenance_chain
        .as_ref()
        .map(|provenance| provenance.links.len())
        .unwrap_or_default();
    let revocation_state =
        revocation_state_override.or(capability.map(|capability| capability.revocation_state));

    let mut rationale = vec![reason.to_string()];
    if let Some(descriptor_id) =
        capability.and_then(|capability| capability.descriptor_id.as_deref())
    {
        rationale.push(format!(
            "capability descriptor at the boundary was '{descriptor_id}'"
        ));
    } else {
        rationale.push(
            "no bounded delegation capability was present at the external boundary".to_string(),
        );
    }
    rationale.push(format!(
        "external action requested descriptor '{}'",
        action.descriptor_id
    ));
    if let Some(required_scope) = action.required_scope {
        rationale.push(format!(
            "external action required scope {:?} while the capability carried {:?}",
            required_scope,
            capability.map(|capability| capability.scope)
        ));
    }
    if let Some(decision) = policy_decision {
        rationale.push(format!("policy engine reason: {}", decision.reason));
    }

    ExternalCapabilityDecisionSummary {
        boundary_surface: Some(ExternalCapabilityDecisionSurface::ToolBus),
        branch_id,
        capability_id: capability.map(|capability| capability.capability_id),
        capability_descriptor_id: capability
            .and_then(|capability| capability.descriptor_id.clone()),
        action_descriptor_id: Some(action.descriptor_id.clone()),
        action_id: Some(action.action_id.clone()),
        action_title: Some(action.title.clone()),
        scope: capability.map(|capability| capability.scope),
        required_scope: action.required_scope,
        policy_action: Some(action.policy.action.clone()),
        policy_resource: Some(action.policy.resource.clone()),
        policy_scope: Some(action.policy.scope.clone()),
        policy_resource_id: action.policy.resource_id.clone(),
        revocation_state,
        chain_depth,
        outcome,
        observed_at: Some(Utc::now()),
        rationale,
    }
}

fn delegation_audit_context(summary: &CapabilitySummary) -> DelegationAuditContext {
    DelegationAuditContext {
        capability_id: Some(summary.capability_id),
        descriptor_id: summary.descriptor_id.clone(),
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
