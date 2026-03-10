use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use mister_smith_core::SecurityError;
use mister_smith_security::sandbox::{
    AgentClass, CrossingDecision, IOFirewall, SandboxAccountConfig, SandboxCredentialIssuer,
};
use nkeys::KeyPair;
use serde_json::Value;

fn account_config(name: &str, ttl: Duration) -> SandboxAccountConfig {
    let signing_key = KeyPair::new_account();
    SandboxAccountConfig::new(name, signing_key.public_key(), signing_key, ttl)
}

fn issuer() -> SandboxCredentialIssuer {
    SandboxCredentialIssuer::new(
        account_config("persistent-account", Duration::from_secs(900)),
        account_config("ephemeral-account", Duration::from_secs(60)),
    )
}

fn decode_payload(jwt: &str) -> Value {
    let payload = jwt
        .split('.')
        .nth(1)
        .expect("jwt should have a payload segment");
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .expect("payload should be valid base64url");
    serde_json::from_slice(&bytes).expect("payload should be valid json")
}

#[test]
fn sandbox_credentials_use_distinct_accounts_and_non_overlapping_permissions() {
    let issuer = issuer();

    let persistent = issuer
        .create_credentials("coordinator-1", AgentClass::Persistent)
        .expect("persistent credentials should be issued");
    let ephemeral = issuer
        .create_credentials("worker-1", AgentClass::Ephemeral)
        .expect("ephemeral credentials should be issued");

    assert_eq!(persistent.agent_class, AgentClass::Persistent);
    assert_eq!(persistent.nats_account, "persistent-account");
    assert_eq!(ephemeral.agent_class, AgentClass::Ephemeral);
    assert_eq!(ephemeral.nats_account, "ephemeral-account");
    assert!(persistent.expires_at > ephemeral.expires_at);

    let persistent_claims = decode_payload(&persistent.jwt);
    let ephemeral_claims = decode_payload(&ephemeral.jwt);

    let persistent_publish = persistent_claims["nats"]["pub"]["allow"]
        .as_array()
        .expect("persistent jwt should include publish permissions");
    let persistent_subscribe = persistent_claims["nats"]["sub"]["allow"]
        .as_array()
        .expect("persistent jwt should include subscribe permissions");
    let ephemeral_publish = ephemeral_claims["nats"]["pub"]["allow"]
        .as_array()
        .expect("ephemeral jwt should include publish permissions");

    assert!(
        persistent_publish
            .iter()
            .filter_map(Value::as_str)
            .any(|subject| subject == "tasks.*.assignment"),
        "persistent credentials should be allowed to emit task assignments"
    );
    assert!(
        persistent_publish
            .iter()
            .filter_map(Value::as_str)
            .any(|subject| subject.starts_with("state.persistent.coordinator-1")),
        "persistent credentials should be scoped to persistent state subjects"
    );
    assert!(
        persistent_subscribe
            .iter()
            .filter_map(Value::as_str)
            .any(|subject| subject == "tasks.*.result"),
        "persistent credentials should be allowed to receive task results"
    );
    assert!(
        persistent_subscribe
            .iter()
            .filter_map(Value::as_str)
            .any(|subject| subject == "tasks.*.progress"),
        "persistent credentials should be allowed to receive task progress"
    );
    assert!(
        ephemeral_publish
            .iter()
            .filter_map(Value::as_str)
            .any(|subject| subject == "tasks.*.result" || subject == "tasks.*.progress"),
        "ephemeral credentials should be allowed to return task results/progress"
    );
    assert!(
        ephemeral_publish
            .iter()
            .filter_map(Value::as_str)
            .any(|subject| subject.starts_with("state.ephemeral.worker-1")),
        "ephemeral credentials should be scoped to ephemeral state subjects"
    );
    assert!(
        !ephemeral_publish
            .iter()
            .filter_map(Value::as_str)
            .any(|subject| subject.contains("state.persistent")),
        "ephemeral credentials must not include persistent state permissions"
    );
    assert!(
        !ephemeral_publish
            .iter()
            .filter_map(Value::as_str)
            .any(|subject| subject == "workflow.>"),
        "ephemeral credentials must not include workflow-wide permissions"
    );

    let ephemeral_subscribe = ephemeral_claims["nats"]["sub"]["allow"]
        .as_array()
        .expect("ephemeral jwt should include subscribe permissions");
    assert!(
        ephemeral_subscribe
            .iter()
            .filter_map(Value::as_str)
            .any(|subject| subject == "tasks.*.assignment"),
        "ephemeral credentials should be allowed to receive task assignments"
    );
}

#[test]
fn cleanup_removes_ephemeral_credentials_and_expired_entries_can_be_reaped() {
    let issuer = issuer();
    let credentials = issuer
        .create_credentials("worker-2", AgentClass::Ephemeral)
        .expect("ephemeral credentials should be issued");

    assert!(issuer.credentials("worker-2").is_some());
    assert_eq!(issuer.cleanup("worker-2").unwrap().agent_id, "worker-2");
    assert!(issuer.credentials("worker-2").is_none());

    let expiring = issuer
        .create_credentials("worker-3", AgentClass::Ephemeral)
        .expect("ephemeral credentials should be issued");
    let removed = issuer.cleanup_expired(expiring.expires_at + 1);
    assert_eq!(removed, 1);
    assert!(issuer.credentials("worker-3").is_none());
    assert!(credentials.expires_at >= credentials.created_at);
}

#[test]
fn io_firewall_enforces_same_account_and_cross_boundary_rules() {
    let firewall = IOFirewall::with_default_rules("persistent-account", "ephemeral-account");

    assert_eq!(
        firewall
            .check_crossing(
                "persistent-account",
                "persistent-account",
                "state.persistent.coordinator-1.snapshot",
            )
            .expect("same-account traffic should be allowed"),
        CrossingDecision::Allow,
    );

    assert_eq!(
        firewall
            .check_crossing(
                "persistent-account",
                "ephemeral-account",
                "tasks.analysis.assignment",
            )
            .expect("task assignment crossing should be allowed through quarantine"),
        CrossingDecision::Quarantine,
    );

    let err = firewall
        .check_crossing(
            "ephemeral-account",
            "persistent-account",
            "state.persistent.coordinator-1.snapshot",
        )
        .expect_err("persistent state access should be blocked");
    assert!(matches!(err, SecurityError::AuthorizationDenied(_)));
}
