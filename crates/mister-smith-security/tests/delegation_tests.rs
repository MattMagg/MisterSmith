//! Delegation service and bounded capability tests for Phase 10.

use std::time::Duration;

use mister_smith_core::{
    AuthorityPrincipal, CapabilityActionKind, DelegatedAction, DelegatedActionPolicy,
    DelegationScope, RevocationState, SecurityError,
};
use mister_smith_security::config::{JwtConfig, KeySource};
use mister_smith_security::delegation::{external_delegation_envelope, DelegationService};
use mister_smith_security::jwt::{AgentClaims, JwtManager, DEFAULT_MAX_DELEGATION_CHAIN_DEPTH};

struct DelegationHarness {
    jwt_manager: JwtManager,
    secret: Vec<u8>,
}

impl DelegationHarness {
    fn new(access_token_ttl: Duration) -> Self {
        let secret = b"delegation-test-secret-key-for-hmac-256-min-32-bytes!!".to_vec();
        let config = JwtConfig {
            algorithm: "HS256".to_string(),
            access_token_ttl,
            refresh_token_ttl: Duration::from_secs(3_600),
            issuer: Some("mister-smith-delegation-tests".to_string()),
            audience: vec!["delegation-tests".to_string()],
            delegation_chain_max_depth: DEFAULT_MAX_DELEGATION_CHAIN_DEPTH,
            key_source: KeySource::Hmac {
                secret: secret.clone(),
            },
        };

        Self {
            jwt_manager: JwtManager::new(&config).expect("delegation harness should initialize"),
            secret,
        }
    }

    fn delegated_claims(&self, child_agent_id: &str) -> AgentClaims {
        parent_claims().delegated_to(child_agent_id.to_string(), "worker".to_string())
    }

    fn issue_access_token(&self, claims: &AgentClaims) -> String {
        self.jwt_manager
            .generate_token_pair(claims)
            .expect("delegated token generation should succeed")
            .access_token
    }

    fn encode_raw(&self, claims: &AgentClaims) -> String {
        jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
            claims,
            &jsonwebtoken::EncodingKey::from_secret(&self.secret),
        )
        .expect("raw delegated claims should encode")
    }
}

fn parent_claims() -> AgentClaims {
    AgentClaims {
        sub: "coordinator-1".to_string(),
        agent_id: "coordinator-1".to_string(),
        agent_type: "coordinator".to_string(),
        capabilities: vec!["delegate".to_string()],
        permissions: vec!["execute:privileged:workflow".to_string()],
        ..Default::default()
    }
}

#[test]
fn delegation_valid_chain_roundtrip_survives_issue_and_validation() {
    let harness = DelegationHarness::new(Duration::from_secs(300));
    let delegated_claims = harness.delegated_claims("executor-1");

    let access_token = harness.issue_access_token(&delegated_claims);
    let validated = harness
        .jwt_manager
        .validate_token(&access_token)
        .expect("delegated access token should validate");

    assert_eq!(validated.agent_id, "executor-1");
    assert_eq!(validated.agent_type, "worker");
    assert_eq!(validated.delegation_chain, vec!["coordinator-1"]);
}

#[test]
fn delegation_expired_token_is_rejected_for_delegated_claims() {
    let harness = DelegationHarness::new(Duration::from_secs(0));
    let delegated_claims = harness.delegated_claims("executor-expiring");

    let access_token = harness.issue_access_token(&delegated_claims);

    std::thread::sleep(Duration::from_secs(6));

    assert!(matches!(
        harness.jwt_manager.validate_token(&access_token),
        Err(SecurityError::TokenExpired)
    ));
}

#[test]
fn delegation_revoked_token_is_rejected_for_delegated_claims() {
    let harness = DelegationHarness::new(Duration::from_secs(300));
    let delegated_claims = harness.delegated_claims("executor-revoked");

    let access_token = harness.issue_access_token(&delegated_claims);
    let validated = harness
        .jwt_manager
        .validate_token(&access_token)
        .expect("delegated token should validate before revocation");

    harness.jwt_manager.revoke_token(&validated.jti);

    assert!(matches!(
        harness.jwt_manager.validate_token(&access_token),
        Err(SecurityError::TokenRevoked)
    ));
}

#[test]
fn delegation_cyclic_chain_is_rejected_during_validation() {
    let harness = DelegationHarness::new(Duration::from_secs(300));
    let now = chrono::Utc::now().timestamp() as u64;
    let cyclic_claims = AgentClaims {
        iss: Some("mister-smith-delegation-tests".to_string()),
        sub: "executor-cycle".to_string(),
        aud: vec!["delegation-tests".to_string()],
        exp: now + 300,
        iat: now,
        jti: uuid::Uuid::new_v4().to_string(),
        agent_id: "executor-cycle".to_string(),
        agent_type: "worker".to_string(),
        delegation_chain: vec!["root-agent".to_string(), "executor-cycle".to_string()],
        token_use: "access".to_string(),
        ..Default::default()
    };

    let forged_token = harness.encode_raw(&cyclic_claims);

    assert!(matches!(
        harness.jwt_manager.validate_token(&forged_token),
        Err(SecurityError::InvalidToken(message))
            if message.contains("delegation_chain contains a circular reference")
    ));
}

#[test]
fn delegation_service_rejects_claims_that_exceed_configured_chain_depth() {
    let service = DelegationService::new_with_delegation_chain_max_depth(1);
    let claims = parent_claims()
        .delegated_to("intermediate-agent".to_string(), "worker".to_string())
        .delegated_to("executor-depth".to_string(), "worker".to_string());

    assert!(matches!(
        service.validate_claims(&claims, None),
        Err(mister_smith_core::DelegationError::InvalidChain(message))
            if message.contains("delegation_chain exceeds max depth")
    ));
}

#[test]
fn delegation_service_issues_provenance_and_validates_scope() {
    let service = DelegationService::new();
    let recipient = mister_smith_core::AgentId::from_uuid(uuid::Uuid::new_v4());
    let (capability, provenance) = service
        .issue_capability(
            AuthorityPrincipal::Policy("operator".to_string()),
            recipient,
            DelegationScope::InvokeTool,
            Some("tool:data.echo".to_string()),
            Duration::from_secs(300),
            None,
            None,
        )
        .expect("root capability should issue");

    let validated = service
        .validate_capability(&capability, &provenance, Some(DelegationScope::InvokeTool))
        .expect("issued capability should validate");

    assert_eq!(validated.capability.scope, DelegationScope::InvokeTool);
    assert_eq!(
        validated.capability.descriptor_id.as_deref(),
        Some("tool:data.echo")
    );
    assert_eq!(validated.chain_depth, 1);
    assert_eq!(
        validated.provenance.terminal_capability,
        capability.capability_id
    );
}

#[test]
fn delegation_service_rejects_revoked_capabilities() {
    let service = DelegationService::new();
    let recipient = mister_smith_core::AgentId::from_uuid(uuid::Uuid::new_v4());
    let (capability, provenance) = service
        .issue_capability(
            AuthorityPrincipal::Policy("operator".to_string()),
            recipient,
            DelegationScope::InvokeTool,
            Some("tool:data.echo".to_string()),
            Duration::from_secs(300),
            None,
            None,
        )
        .expect("capability should issue");

    service.revoke_capability(capability.capability_id);

    assert!(matches!(
        service.validate_capability(&capability, &provenance, Some(DelegationScope::InvokeTool)),
        Err(mister_smith_core::DelegationError::Revoked { .. })
    ));
    assert_eq!(
        service.revocation_state(&capability),
        RevocationState::Revoked
    );
}

#[test]
fn delegation_service_rejects_revoked_actions() {
    let service = DelegationService::new();
    let recipient = mister_smith_core::AgentId::from_uuid(uuid::Uuid::new_v4());
    let (capability, provenance) = service
        .issue_capability(
            AuthorityPrincipal::Policy("operator".to_string()),
            recipient,
            DelegationScope::InvokeTool,
            Some("tool:data.echo".to_string()),
            Duration::from_secs(300),
            None,
            None,
        )
        .expect("capability should issue");
    let action = DelegatedAction {
        descriptor_id: "tool:data.echo".to_string(),
        action_id: "tool:data.echo#execute".to_string(),
        title: "execute data.echo".to_string(),
        description: "execute access for tool data.echo".to_string(),
        kind: CapabilityActionKind::Execute,
        policy: DelegatedActionPolicy {
            action: "execute".to_string(),
            resource: "tool".to_string(),
            scope: "data".to_string(),
            resource_id: Some("data.echo".to_string()),
        },
        required_scope: Some(DelegationScope::InvokeTool),
        revocation_key: "tool:data.echo#execute".to_string(),
    };

    service.revoke_action(action.revocation_key.clone());

    assert!(matches!(
        service.validate_action(&capability, &provenance, &action),
        Err(mister_smith_core::DelegationError::ActionRevoked { revocation_key })
            if revocation_key == "tool:data.echo#execute"
    ));
}

#[test]
fn delegation_service_validates_external_envelope_after_transport_serialization() {
    let service = DelegationService::new();
    let recipient = mister_smith_core::AgentId::from_uuid(uuid::Uuid::new_v4());
    let (capability, provenance) = service
        .issue_capability(
            AuthorityPrincipal::Policy("operator".to_string()),
            recipient,
            DelegationScope::InvokeTool,
            Some("tool:data.echo".to_string()),
            Duration::from_secs(300),
            None,
            None,
        )
        .expect("capability should issue");
    let action = DelegatedAction {
        descriptor_id: "tool:data.echo".to_string(),
        action_id: "tool:data.echo#execute".to_string(),
        title: "execute data.echo".to_string(),
        description: "execute access for tool data.echo".to_string(),
        kind: CapabilityActionKind::Execute,
        policy: DelegatedActionPolicy {
            action: "execute".to_string(),
            resource: "tool".to_string(),
            scope: "data".to_string(),
            resource_id: Some("data.echo".to_string()),
        },
        required_scope: Some(DelegationScope::InvokeTool),
        revocation_key: "tool:data.echo#execute".to_string(),
    };

    let validated = service
        .validate_action(&capability, &provenance, &action)
        .expect("action should validate locally");
    let encoded = serde_json::to_value(external_delegation_envelope(&validated, Some(&action)))
        .expect("envelope should serialize");
    let decoded = serde_json::from_value(encoded).expect("envelope should deserialize");

    let validated_after_transport = service
        .validate_external_envelope(&decoded)
        .expect("envelope should validate after transport");

    assert_eq!(
        validated_after_transport.capability.capability_id,
        capability.capability_id
    );
    assert_eq!(validated_after_transport.chain_depth, 1);
}

#[test]
fn delegation_service_rejects_external_envelope_with_invalid_provenance_chain() {
    let service = DelegationService::new();
    let recipient = mister_smith_core::AgentId::from_uuid(uuid::Uuid::new_v4());
    let (capability, provenance) = service
        .issue_capability(
            AuthorityPrincipal::Policy("operator".to_string()),
            recipient,
            DelegationScope::InvokeTool,
            Some("tool:data.echo".to_string()),
            Duration::from_secs(300),
            None,
            None,
        )
        .expect("capability should issue");
    let action = DelegatedAction {
        descriptor_id: "tool:data.echo".to_string(),
        action_id: "tool:data.echo#execute".to_string(),
        title: "execute data.echo".to_string(),
        description: "execute access for tool data.echo".to_string(),
        kind: CapabilityActionKind::Execute,
        policy: DelegatedActionPolicy {
            action: "execute".to_string(),
            resource: "tool".to_string(),
            scope: "data".to_string(),
            resource_id: Some("data.echo".to_string()),
        },
        required_scope: Some(DelegationScope::InvokeTool),
        revocation_key: "tool:data.echo#execute".to_string(),
    };

    let validated = service
        .validate_action(&capability, &provenance, &action)
        .expect("action should validate locally");
    let mut invalid_envelope = external_delegation_envelope(&validated, Some(&action));
    invalid_envelope
        .provenance
        .links
        .push(invalid_envelope.provenance.links[0].clone());

    assert!(matches!(
        service.validate_external_envelope(&invalid_envelope),
        Err(mister_smith_core::DelegationError::InvalidChain(message))
            if message.contains("delegation provenance")
    ));
}

#[test]
fn delegation_service_rejects_descriptorless_capability_for_actions() {
    let service = DelegationService::new();
    let recipient = mister_smith_core::AgentId::from_uuid(uuid::Uuid::new_v4());
    let (capability, provenance) = service
        .issue_capability(
            AuthorityPrincipal::Policy("operator".to_string()),
            recipient,
            DelegationScope::InvokeTool,
            None,
            Duration::from_secs(300),
            None,
            None,
        )
        .expect("legacy capability should issue");
    let action = DelegatedAction {
        descriptor_id: "tool:data.echo".to_string(),
        action_id: "tool:data.echo#execute".to_string(),
        title: "execute data.echo".to_string(),
        description: "execute access for tool data.echo".to_string(),
        kind: CapabilityActionKind::Execute,
        policy: DelegatedActionPolicy {
            action: "execute".to_string(),
            resource: "tool".to_string(),
            scope: "data".to_string(),
            resource_id: Some("data.echo".to_string()),
        },
        required_scope: Some(DelegationScope::InvokeTool),
        revocation_key: "tool:data.echo#execute".to_string(),
    };

    let error = service
        .validate_action(&capability, &provenance, &action)
        .expect_err("descriptorless capability should be rejected on action-bound execution");

    assert!(matches!(
        error,
        mister_smith_core::DelegationError::InvalidChain(message)
            if message.contains("missing descriptor binding for action descriptor")
    ));
}
