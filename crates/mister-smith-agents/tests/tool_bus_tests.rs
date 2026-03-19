use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use mister_smith_agents::{AgentSystemError, ToolBus};
use mister_smith_core::{
    AgentId, AuthorityPrincipal, DelegationScope, Tool, ToolCapabilities, ToolError, ToolId,
    ToolSchema,
};
use mister_smith_events::{AutonomyEvent, EventBus, ExternalCapabilityDecisionOutcome};
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

async fn recv_external_capability_decision(
    rx: &mut tokio::sync::broadcast::Receiver<mister_smith_events::Event>,
) -> mister_smith_events::ExternalCapabilityDecisionSummary {
    for _ in 0..4 {
        let event = tokio::time::timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("autonomy event should arrive")
            .expect("broadcast should stay open");
        let autonomy_event = serde_json::from_value::<AutonomyEvent>(event.payload.clone())
            .expect("broadcast payload should decode as AutonomyEvent");
        if let AutonomyEvent::DelegationDecisionRecorded(envelope) = autonomy_event {
            return envelope.payload;
        }
    }

    panic!("expected external capability decision event");
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
            Some("tool:data.echo".to_string()),
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

    bus.register_privileged_native_tool(
        "echo",
        "data",
        agent_id,
        "Echoes the payload",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        DelegationScope::InvokeTool,
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
    assert_eq!(
        events[0]
            .delegation
            .as_ref()
            .and_then(|delegation| delegation.descriptor_id.as_deref()),
        Some("tool:data.echo")
    );
}

#[tokio::test]
async fn invoke_allows_legacy_descriptorless_delegation_for_privileged_tools() {
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
            None,
            Duration::from_secs(60),
            None,
            None,
        )
        .expect("legacy capability should issue");
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

    bus.register_privileged_native_tool(
        "echo",
        "data",
        agent_id,
        "Echoes the payload",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        DelegationScope::InvokeTool,
        Arc::new(EchoTool { id: ToolId::new() }),
    );

    let result = bus
        .invoke(
            Some(&principal),
            "data",
            "echo",
            json!({ "value": "legacy" }),
            Some(Duration::from_millis(50)),
        )
        .await
        .expect("legacy descriptorless delegation should remain valid");

    assert_eq!(result, json!({ "echo": { "value": "legacy" } }));

    let events = audit.recent_events(10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, AuditEventType::Delegation);
    assert_eq!(events[0].outcome, AuditOutcome::Success);
    assert_eq!(
        events[0]
            .delegation
            .as_ref()
            .and_then(|delegation| delegation.descriptor_id.as_deref()),
        None
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
            Some("tool:data.echo".to_string()),
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

    bus.register_privileged_native_tool(
        "echo",
        "data",
        agent_id,
        "Echoes the payload",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        DelegationScope::InvokeTool,
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

#[test]
fn privileged_tool_registration_exposes_capability_descriptor() {
    let bus = ToolBus::new();
    let agent_id = AgentId::new();

    bus.register_privileged_native_tool(
        "echo",
        "data",
        agent_id,
        "Echoes the payload",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        DelegationScope::InvokeTool,
        Arc::new(EchoTool { id: ToolId::new() }),
    );

    let tool = bus.find("data", "echo").expect("tool should be registered");
    assert_eq!(tool.capability_descriptor.descriptor_id, "tool:data.echo");
    assert_eq!(tool.capability_descriptor.local_agent_id, Some(agent_id));
    assert_eq!(
        tool.capability_descriptor
            .actions
            .iter()
            .find(|action| action.kind == mister_smith_core::CapabilityActionKind::Execute)
            .and_then(|action| action.required_scope),
        Some(DelegationScope::InvokeTool)
    );
}

#[tokio::test]
async fn invoke_allows_unprivileged_tool_for_delegated_principal() {
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
            Some("tool:data.secret".to_string()),
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
                capability,
                provenance,
                chain_depth: 1,
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
            json!({ "value": "helper" }),
            Some(Duration::from_millis(50)),
        )
        .await
        .expect("unprivileged helper tool should not require descriptor-bound delegation");

    assert_eq!(result, json!({ "echo": { "value": "helper" } }));

    let events = audit.recent_events(10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].outcome, AuditOutcome::Success);
    assert_eq!(events[0].delegation, None);
}

#[tokio::test]
async fn invoke_publishes_allowed_external_capability_decision_for_privileged_tools() {
    let event_bus = Arc::new(EventBus::default());
    let mut rx = event_bus.subscribe_broadcast();
    let delegation_service = Arc::new(DelegationService::new());
    let bus = ToolBus::with_security(
        Some(Arc::new(PolicyEngine::new(&RbacConfig::default()))),
        None,
    )
    .with_delegation_service(delegation_service.clone())
    .with_event_bus(event_bus);
    let agent_id = AgentId::new();
    let workflow_id = mister_smith_core::TaskId::new();
    let branch_id = mister_smith_core::ExecutionBranchId::new();
    let (capability, provenance) = delegation_service
        .issue_capability(
            AuthorityPrincipal::Policy("operator".to_string()),
            agent_id,
            DelegationScope::InvokeTool,
            Some("tool:data.echo".to_string()),
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
                capability,
                provenance: provenance.clone(),
                chain_depth: provenance.links.len(),
            },
        ),
    )
    .with_workflow(workflow_id)
    .with_branch(branch_id)
    .requiring_delegation(DelegationScope::InvokeTool);

    bus.register_privileged_native_tool(
        "echo",
        "data",
        agent_id,
        "Echoes the payload",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        DelegationScope::InvokeTool,
        Arc::new(EchoTool { id: ToolId::new() }),
    );

    let result = bus
        .invoke(
            Some(&principal),
            "data",
            "echo",
            json!({ "value": "allowed" }),
            Some(Duration::from_millis(50)),
        )
        .await
        .expect("descriptor-bound privileged tool should be allowed");

    assert_eq!(result, json!({ "echo": { "value": "allowed" } }));

    let decision = recv_external_capability_decision(&mut rx).await;
    assert_eq!(decision.outcome, ExternalCapabilityDecisionOutcome::Allowed);
    assert_eq!(decision.branch_id, Some(branch_id));
    assert!(decision.observed_at.is_some());
    assert_eq!(
        decision.capability_descriptor_id.as_deref(),
        Some("tool:data.echo")
    );
    assert_eq!(
        decision.action_id.as_deref(),
        Some("tool:data.echo#execute")
    );
    assert!(decision
        .rationale
        .iter()
        .any(|line| line.contains("matched the requested external action")));
    assert!(
        decision
            .rationale
            .iter()
            .any(|line| line
                .contains("required scope InvokeTool matched capability scope InvokeTool"))
    );
}

#[tokio::test]
async fn invoke_rejects_descriptor_mismatch_for_privileged_tools() {
    let audit = Arc::new(AuditLogger::new(&AuditConfig::default()));
    let event_bus = Arc::new(EventBus::default());
    let mut rx = event_bus.subscribe_broadcast();
    let delegation_service = Arc::new(DelegationService::new());
    let bus = ToolBus::with_security(
        Some(Arc::new(PolicyEngine::new(&RbacConfig::default()))),
        Some(audit.clone()),
    )
    .with_delegation_service(delegation_service.clone())
    .with_event_bus(event_bus);
    let agent_id = AgentId::new();
    let workflow_id = mister_smith_core::TaskId::new();
    let branch_id = mister_smith_core::ExecutionBranchId::new();
    let (capability, provenance) = delegation_service
        .issue_capability(
            AuthorityPrincipal::Policy("operator".to_string()),
            agent_id,
            DelegationScope::InvokeTool,
            Some("tool:data.other".to_string()),
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
    .with_workflow(workflow_id)
    .with_branch(branch_id)
    .requiring_delegation(DelegationScope::InvokeTool);

    bus.register_privileged_native_tool(
        "echo",
        "data",
        agent_id,
        "Echoes the payload",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        DelegationScope::InvokeTool,
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
        .expect_err("descriptor mismatch should be rejected");

    assert!(matches!(err, AgentSystemError::PermissionDenied(_)));

    let events = audit.recent_events(10);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_type, AuditEventType::Delegation);
    assert_eq!(events[0].outcome, AuditOutcome::Blocked);
    assert!(events[0]
        .delegation
        .as_ref()
        .and_then(|delegation| delegation.rejection_reason.as_ref())
        .is_some_and(|reason| reason.contains("does not authorize action descriptor")));

    let decision = recv_external_capability_decision(&mut rx).await;
    assert_eq!(
        decision.outcome,
        ExternalCapabilityDecisionOutcome::Rejected
    );
    assert_eq!(decision.branch_id, Some(branch_id));
    assert!(decision.observed_at.is_some());
    assert_eq!(
        decision.action_descriptor_id.as_deref(),
        Some("tool:data.echo")
    );
    assert!(decision
        .rationale
        .iter()
        .any(|line| line.contains("does not authorize action descriptor")));
}

#[tokio::test]
async fn invoke_records_rejected_external_decision_when_capability_is_missing() {
    let event_bus = Arc::new(EventBus::default());
    let mut rx = event_bus.subscribe_broadcast();
    let delegation_service = Arc::new(DelegationService::new());
    let bus = ToolBus::with_security(
        Some(Arc::new(PolicyEngine::new(&RbacConfig::default()))),
        None,
    )
    .with_delegation_service(delegation_service)
    .with_event_bus(event_bus);
    let agent_id = AgentId::new();
    let workflow_id = mister_smith_core::TaskId::new();
    let branch_id = mister_smith_core::ExecutionBranchId::new();
    let principal = mister_smith_agents::tool_bus::ToolPrincipal::new(
        agent_id,
        claims(agent_id, &["execute:tool:data"]),
    )
    .with_workflow(workflow_id)
    .with_branch(branch_id)
    .requiring_delegation(DelegationScope::InvokeTool);

    bus.register_privileged_native_tool(
        "echo",
        "data",
        agent_id,
        "Echoes the payload",
        json!({ "type": "object" }),
        json!({ "type": "object" }),
        DelegationScope::InvokeTool,
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
        .expect_err("missing delegation capability should be rejected");

    assert!(matches!(err, AgentSystemError::PermissionDenied(_)));

    let decision = recv_external_capability_decision(&mut rx).await;
    assert_eq!(
        decision.outcome,
        ExternalCapabilityDecisionOutcome::Rejected
    );
    assert_eq!(decision.branch_id, Some(branch_id));
    assert!(decision.observed_at.is_some());
    assert_eq!(decision.capability_id, None);
    assert_eq!(decision.scope, None);
    assert_eq!(decision.revocation_state, None);
    assert_eq!(
        decision.action_descriptor_id.as_deref(),
        Some("tool:data.echo")
    );
    assert!(decision.rationale.iter().any(|line| {
        line.contains("no bounded delegation capability was present at the external boundary")
    }));
}
