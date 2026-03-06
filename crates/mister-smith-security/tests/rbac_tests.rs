//! Black-box RBAC integration tests for `PolicyEngine`.

use std::collections::HashMap;

use mister_smith_security::config::RbacConfig;
use mister_smith_security::jwt::AgentClaims;
use mister_smith_security::rbac::{
    AuthorizationRequest, Permission, PolicyConstraints, PolicyEngine, Role, TimeWindow,
};

fn claims_with_type(agent_type: &str) -> AgentClaims {
    AgentClaims {
        sub: "agent-1".to_string(),
        agent_id: "agent-1".to_string(),
        agent_type: agent_type.to_string(),
        ..Default::default()
    }
}

fn claims_with_permissions(permissions: &[&str]) -> AgentClaims {
    AgentClaims {
        sub: "agent-1".to_string(),
        agent_id: "agent-1".to_string(),
        permissions: permissions.iter().map(|p| p.to_string()).collect(),
        ..Default::default()
    }
}

#[test]
fn us2_as1_worker_reads_own_tasks() {
    let engine = PolicyEngine::new(&RbacConfig::default());
    let claims = claims_with_permissions(&["read:task:own"]);

    let mut context = HashMap::new();
    context.insert("scope".to_string(), "own".to_string());

    let decision = engine.evaluate(&AuthorizationRequest {
        principal: claims,
        action: "read".to_string(),
        resource: "task".to_string(),
        resource_id: None,
        context,
    });

    assert!(decision.allowed);
}

#[test]
fn us2_as2_worker_cannot_delete_system_resources() {
    let engine = PolicyEngine::new(&RbacConfig::default());
    let claims = claims_with_permissions(&["read:task:own"]);

    assert!(!engine.check_permission(&claims, "delete", "system"));
}

#[test]
fn us2_as3_admin_has_full_access() {
    let engine = PolicyEngine::new(&RbacConfig::default());
    let claims = claims_with_type("admin");

    assert!(engine.check_permission(&claims, "read", "task"));
    assert!(engine.check_permission(&claims, "write", "system"));
    assert!(engine.check_permission(&claims, "delete", "agent"));
}

#[test]
fn us2_as4_explicit_deny_wins() {
    let engine = PolicyEngine::new(&RbacConfig::default());

    let mut deny = Permission::parse("read:secret:all").unwrap();
    deny.deny = true;

    engine.add_role(Role {
        name: "restricted".to_string(),
        description: None,
        permissions: vec![Permission::parse("read:*:all").unwrap(), deny],
        parent: None,
    });

    let claims = claims_with_type("restricted");
    let decision = engine.evaluate(&AuthorizationRequest {
        principal: claims,
        action: "read".to_string(),
        resource: "secret".to_string(),
        resource_id: None,
        context: HashMap::new(),
    });

    assert!(!decision.allowed);
    assert!(decision.reason.contains("explicit deny"));
}

#[test]
fn us2_as5_default_deny_without_policy() {
    let engine = PolicyEngine::new(&RbacConfig::default());
    let claims = AgentClaims::default();

    let decision = engine.evaluate(&AuthorizationRequest {
        principal: claims,
        action: "read".to_string(),
        resource: "task".to_string(),
        resource_id: None,
        context: HashMap::new(),
    });

    assert!(!decision.allowed);
    assert_eq!(decision.reason, "no matching policy");
}

#[test]
fn us2_as6_role_hierarchy_inheritance() {
    let engine = PolicyEngine::new(&RbacConfig::default());

    engine.add_role(Role {
        name: "junior_dev".to_string(),
        description: None,
        permissions: vec![Permission::parse("write:agent:own").unwrap()],
        parent: Some("viewer".to_string()),
    });

    let claims = claims_with_type("junior_dev");
    assert!(engine.check_permission(&claims, "read", "agent"));
    assert!(engine.check_permission(&claims, "write", "agent"));
    assert!(!engine.check_permission(&claims, "delete", "agent"));
}

#[test]
fn us2_as7_time_window_constraint() {
    let engine = PolicyEngine::new(&RbacConfig::default());

    let mut permission = Permission::parse("read:agent:all").unwrap();
    permission.constraints = Some(PolicyConstraints {
        time_window: Some(TimeWindow {
            start_hour: 9,
            end_hour: 17,
            timezone: "UTC".to_string(),
            days: vec!["monday".to_string()],
        }),
        ip_ranges: None,
        resource_owner: None,
    });

    engine.add_role(Role {
        name: "business_hours".to_string(),
        description: None,
        permissions: vec![permission],
        parent: None,
    });

    let claims = claims_with_type("business_hours");
    let mut context = HashMap::new();
    context.insert("hour".to_string(), "20".to_string());
    context.insert("day".to_string(), "monday".to_string());

    let decision = engine.evaluate(&AuthorizationRequest {
        principal: claims,
        action: "read".to_string(),
        resource: "agent".to_string(),
        resource_id: None,
        context,
    });

    assert!(!decision.allowed);
}

#[test]
fn wildcard_permission_matching_and_string_parsing() {
    let permission = Permission::parse("read:*:all").unwrap();
    assert_eq!(permission.action, "read");
    assert_eq!(permission.resource, "*");
    assert_eq!(permission.scope, "all");

    let engine = PolicyEngine::new(&RbacConfig::default());
    let claims = claims_with_type("viewer");
    assert!(engine.check_permission(&claims, "read", "any-resource"));

    assert!(Permission::parse("invalid").is_err());
}
