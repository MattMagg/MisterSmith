//! Delegation service and bounded capability tests for Phase 10.

use std::time::Duration;

use mister_smith_core::{AuthorityPrincipal, DelegationScope, RevocationState, SecurityError};
use mister_smith_security::config::{JwtConfig, KeySource};
use mister_smith_security::delegation::DelegationService;
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
fn delegation_service_issues_provenance_and_validates_scope() {
    let service = DelegationService::new();
    let recipient = mister_smith_core::AgentId::from_uuid(uuid::Uuid::new_v4());
    let (capability, provenance) = service
        .issue_capability(
            AuthorityPrincipal::Policy("operator".to_string()),
            recipient,
            DelegationScope::InvokeTool,
            Duration::from_secs(300),
            None,
            None,
        )
        .expect("root capability should issue");

    let validated = service
        .validate_capability(&capability, &provenance, Some(DelegationScope::InvokeTool))
        .expect("issued capability should validate");

    assert_eq!(validated.capability.scope, DelegationScope::InvokeTool);
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
