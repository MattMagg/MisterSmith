use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use mister_smith_agents::{AgentSystemError, ToolBus};
use mister_smith_core::{
    AgentId, AuthorityPrincipal, DelegationScope, Tool, ToolCapabilities, ToolError, ToolId,
    ToolSchema,
};
use mister_smith_security::audit::{
    events::{AuditEventType, AuditOutcome},
    AuditLogger,
};
use mister_smith_security::config::{AuditConfig, RbacConfig};
use mister_smith_security::jwt::AgentClaims;
use mister_smith_security::rbac::PolicyEngine;
use mister_smith_security::{DelegationService, ValidatedDelegation};
use serde_json::json;

#[derive(Clone)]
struct EchoTool {
    id: ToolId,
}

#[async_trait]
impl Tool for EchoTool {
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        Ok(json!({ "echo": params }))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema
    }

    fn capabilities(&self) -> ToolCapabilities {
        ToolCapabilities
    }

    fn tool_id(&self) -> ToolId {
        self.id
    }

    fn version(&self) -> semver::Version {
        semver::Version::new(0, 1, 0)
    }
}

#[derive(Clone)]
struct SlowTool {
    id: ToolId,
    delay: Duration,
}

#[async_trait]
impl Tool for SlowTool {
    async fn execute(&self, _params: serde_json::Value) -> Result<serde_json::Value, ToolError> {
        tokio::time::sleep(self.delay).await;
        Ok(json!({ "status": "slow-ok" }))
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema
    }

    fn capabilities(&self) -> ToolCapabilities {
        ToolCapabilities
    }

    fn tool_id(&self) -> ToolId {
        self.id
    }

    fn version(&self) -> semver::Version {
        semver::Version::new(0, 1, 0)
    }
}

fn claims(agent_id: AgentId, permissions: &[&str]) -> AgentClaims {
    let now = chrono::Utc::now().timestamp() as u64;
    AgentClaims {
        sub: agent_id.to_string(),
        exp: now + 3600,
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
        agent_id: agent_id.to_string(),
        agent_type: "worker".to_string(),
        permissions: permissions
            .iter()
            .map(|permission| permission.to_string())
            .collect(),
        token_use: "access".to_string(),
        ..Default::default()
    }
}

fn delegated_claims(
    agent_id: AgentId,
    permissions: &[&str],
    validated: &ValidatedDelegation,
) -> AgentClaims {
    let mut claims = claims(agent_id, permissions);
    claims.delegation_capability = Some(validated.capability.clone());
    claims.provenance_chain = Some(validated.provenance.clone());
    claims
}

#[tokio::test]
async fn discover_filters_tools_by_namespace_permission() {
    let bus = ToolBus::with_security(
        Some(Arc::new(PolicyEngine::new(&RbacConfig::default()))),
        None,
    );
    let agent_id = AgentId::new();
    let principal = mister_smith_agents::tool_bus::ToolPrincipal::new(
        agent_id,
        claims(agent_id, &["discover:tool:data"]),
    );

    bus.register_native_tool(
        "echo",
        "data",
        AgentId::new(),
        "Echoes the payload",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        Arc::new(EchoTool { id: ToolId::new() }),
    );
    bus.register_native_tool(
        "secret",
        "admin",
        AgentId::new(),
        "Administrative tool",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        Arc::new(EchoTool { id: ToolId::new() }),
    );

    let visible = bus.discover(Some(&principal), None).unwrap();

    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].namespace, "data");
    assert_eq!(visible[0].name, "echo");
}

#[tokio::test]
async fn invoke_executes_native_tool_with_timeout_and_metrics() {
    let audit = Arc::new(AuditLogger::new(&AuditConfig::default()));
    let bus = ToolBus::with_security(
        Some(Arc::new(PolicyEngine::new(&RbacConfig::default()))),
        Some(audit.clone()),
    );
    let agent_id = AgentId::new();
    let principal = mister_smith_agents::tool_bus::ToolPrincipal::new(
        agent_id,
        claims(agent_id, &["execute:tool:data"]),
    );

    bus.register_native_tool(
        "echo",
        "data",
        agent_id,
        "Echoes the payload",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        Arc::new(EchoTool { id: ToolId::new() }),
    );

    let result = bus
        .invoke(
            Some(&principal),
            "data",
            "echo",
            json!({ "value": 42 }),
            Some(Duration::from_millis(50)),
        )
        .await
        .unwrap();

    assert_eq!(result, json!({ "echo": { "value": 42 } }));

    let metrics = bus.get_metrics("data", "echo").unwrap();
    assert_eq!(metrics.invocation_count, 1);
    assert_eq!(metrics.error_count, 0);

    let events = audit.recent_events(10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].outcome, AuditOutcome::Success);
    assert_eq!(events[0].action.as_deref(), Some("invoke_tool"));
}

#[tokio::test]
async fn invoke_denies_unauthorized_calls_and_records_audit_event() {
    let audit = Arc::new(AuditLogger::new(&AuditConfig::default()));
    let bus = ToolBus::with_security(
        Some(Arc::new(PolicyEngine::new(&RbacConfig::default()))),
        Some(audit.clone()),
    );
    let agent_id = AgentId::new();
    let principal = mister_smith_agents::tool_bus::ToolPrincipal::new(
        agent_id,
        claims(agent_id, &["execute:tool:data"]),
    );

    bus.register_native_tool(
        "secret",
        "admin",
        agent_id,
        "Administrative tool",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        Arc::new(EchoTool { id: ToolId::new() }),
    );

    let err = bus
        .invoke(
            Some(&principal),
            "admin",
            "secret",
            json!({ "value": "blocked" }),
            Some(Duration::from_millis(50)),
        )
        .await
        .unwrap_err();

    assert!(matches!(err, AgentSystemError::PermissionDenied(_)));

    let events = audit.recent_events(10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].outcome, AuditOutcome::Blocked);
    assert_eq!(events[0].action.as_deref(), Some("execute_tool"));
}

#[tokio::test]
async fn invoke_times_out_when_tool_exceeds_deadline() {
    let bus = ToolBus::with_security(None, None);
    let agent_id = AgentId::new();

    bus.register_native_tool(
        "slow",
        "data",
        agent_id,
        "Sleeps before returning",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        Arc::new(SlowTool {
            id: ToolId::new(),
            delay: Duration::from_millis(40),
        }),
    );

    let err = bus
        .invoke(
            None,
            "data",
            "slow",
            json!({}),
            Some(Duration::from_millis(5)),
        )
        .await
        .unwrap_err();

    assert!(matches!(err, AgentSystemError::Timeout(_)));
}

#[tokio::test]
async fn invoke_requires_valid_delegation_for_privileged_tools() {
    let audit = Arc::new(AuditLogger::new(&AuditConfig::default()));
    let delegation_service = Arc::new(DelegationService::new());
    let bus = ToolBus::with_security(
        Some(Arc::new(PolicyEngine::new(&RbacConfig::default()))),
        Some(audit.clone()),
    )
    .with_delegation_service(delegation_service.clone());
    let agent_id = AgentId::new();
    let (capability, provenance) = delegation_service
        .issue_capability(
            AuthorityPrincipal::Policy("operator".to_string()),
            agent_id,
            DelegationScope::InvokeTool,
            Duration::from_secs(60),
            None,
            None,
        )
        .expect("capability should issue");
    let principal = mister_smith_agents::tool_bus::ToolPrincipal::new(
        agent_id,
        delegated_claims(
            agent_id,
            &["execute:tool:data"],
            &ValidatedDelegation {
                capability: capability.clone(),
                provenance: provenance.clone(),
                chain_depth: provenance.links.len(),
            },
        ),
    )
    .requiring_delegation(DelegationScope::InvokeTool);

    bus.register_native_tool(
        "echo",
        "data",
        agent_id,
        "Echoes the payload",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        Arc::new(EchoTool { id: ToolId::new() }),
    );

    let result = bus
        .invoke(
            Some(&principal),
            "data",
            "echo",
            json!({ "value": "delegated" }),
            Some(Duration::from_millis(50)),
        )
        .await
        .expect("delegated privileged invocation should succeed");

    assert_eq!(result, json!({ "echo": { "value": "delegated" } }));

    let events = audit.recent_events(10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, AuditEventType::Delegation);
    assert_eq!(events[0].outcome, AuditOutcome::Success);
    assert_eq!(events[0].action.as_deref(), Some("invoke_tool"));
    assert_eq!(
        events[0]
            .delegation
            .as_ref()
            .and_then(|delegation| delegation.capability_id),
        Some(capability.capability_id)
    );
}

#[tokio::test]
async fn invoke_rejects_revoked_delegation_for_privileged_tools() {
    let audit = Arc::new(AuditLogger::new(&AuditConfig::default()));
    let delegation_service = Arc::new(DelegationService::new());
    let bus = ToolBus::with_security(
        Some(Arc::new(PolicyEngine::new(&RbacConfig::default()))),
        Some(audit.clone()),
    )
    .with_delegation_service(delegation_service.clone());
    let agent_id = AgentId::new();
    let (capability, provenance) = delegation_service
        .issue_capability(
            AuthorityPrincipal::Policy("operator".to_string()),
            agent_id,
            DelegationScope::InvokeTool,
            Duration::from_secs(60),
            None,
            None,
        )
        .expect("capability should issue");
    delegation_service.revoke_capability(capability.capability_id);
    let principal = mister_smith_agents::tool_bus::ToolPrincipal::new(
        agent_id,
        delegated_claims(
            agent_id,
            &["execute:tool:data"],
            &ValidatedDelegation {
                capability: capability.clone(),
                provenance: provenance.clone(),
                chain_depth: provenance.links.len(),
            },
        ),
    )
    .requiring_delegation(DelegationScope::InvokeTool);

    bus.register_native_tool(
        "echo",
        "data",
        agent_id,
        "Echoes the payload",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        Arc::new(EchoTool { id: ToolId::new() }),
    );

    let err = bus
        .invoke(
            Some(&principal),
            "data",
            "echo",
            json!({ "value": "blocked" }),
            Some(Duration::from_millis(50)),
        )
        .await
        .expect_err("revoked delegation should be rejected");

    assert!(matches!(err, AgentSystemError::PermissionDenied(_)));

    let events = audit.recent_events(10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, AuditEventType::Delegation);
    assert_eq!(events[0].outcome, AuditOutcome::Blocked);
    assert_eq!(events[0].action.as_deref(), Some("validate_delegation"));
    assert!(events[0]
        .delegation
        .as_ref()
        .and_then(|delegation| delegation.rejection_reason.as_ref())
        .is_some_and(|reason| reason.contains("revoked")));
}
