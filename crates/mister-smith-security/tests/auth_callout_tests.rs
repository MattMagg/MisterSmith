//! Integration tests for the NATS auth callout service.

use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use mister_smith_security::auth_callout::{
    AuthCalloutHandler, PermissionTier, TrustProfile, AUTH_CALLOUT_SUBJECT,
};
use nkeys::KeyPair;
use serde_json::{json, Value};

fn account_signing_key() -> KeyPair {
    KeyPair::new_account()
}

fn handler() -> AuthCalloutHandler {
    let signing_key = account_signing_key();
    AuthCalloutHandler::new(signing_key.clone(), signing_key.public_key())
}

fn request_jwt(user_nkey: &str, agent_id: &str, server_id: &str) -> String {
    let header = URL_SAFE_NO_PAD.encode(br#"{"typ":"JWT","alg":"ed25519-nkey"}"#);
    let claims = json!({
        "aud": AUTH_CALLOUT_SUBJECT,
        "iat": 1_700_000_000_i64,
        "iss": "test-auth-service",
        "jti": "request-jti",
        "sub": "request-subject",
        "nats": {
            "server_id": {
                "host": "127.0.0.1",
                "id": server_id,
                "name": "nats-test",
                "version": "2.11.0"
            },
            "user_nkey": user_nkey,
            "client_info": {
                "host": "127.0.0.1",
                "id": 42,
                "kind": "Client",
                "name": agent_id,
                "nonce": "client-nonce",
                "user": agent_id
            },
            "connect_opts": {
                "lang": "rust",
                "name": agent_id,
                "protocol": 1,
                "user": agent_id
            },
            "request_nonce": "server-request-nonce",
            "type": "authorization_request",
            "version": 2
        }
    });
    let payload =
        URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).expect("claims should serialize"));

    format!("{header}.{payload}.test-signature")
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
fn trust_score_maps_to_expected_permission_tier_boundaries() {
    assert_eq!(PermissionTier::from_trust_score(0.95), PermissionTier::Full);
    assert_eq!(PermissionTier::from_trust_score(0.90), PermissionTier::Full);
    assert_eq!(
        PermissionTier::from_trust_score(0.89),
        PermissionTier::Standard
    );
    assert_eq!(
        PermissionTier::from_trust_score(0.50),
        PermissionTier::Standard
    );
    assert_eq!(
        PermissionTier::from_trust_score(0.49),
        PermissionTier::Restricted
    );
    assert_eq!(
        PermissionTier::from_trust_score(0.20),
        PermissionTier::Restricted
    );
    assert_eq!(
        PermissionTier::from_trust_score(0.19),
        PermissionTier::Quarantined
    );
}

#[test]
fn authorize_unknown_agent_uses_quarantined_fallback() {
    let handler = handler();

    let result = handler.authorize("missing-agent").unwrap();

    assert_eq!(result.permission_tier, PermissionTier::Quarantined);
    assert_eq!(result.jwt_ttl_secs, 30);
    assert!(result.fallback_applied);
    assert_eq!(
        result.permissions.publish_allow,
        vec!["system.health".to_string()]
    );
    assert_eq!(
        result.permissions.subscribe_allow,
        vec!["system.health".to_string(), "_INBOX.>".to_string()]
    );
}

#[test]
fn handle_auth_request_emits_scoped_user_jwt_for_known_agent() {
    let handler = handler();
    handler.update_trust("agent-standard", TrustProfile::new("agent-standard", 0.55));

    let user_key = KeyPair::new_user();
    let response = handler
        .handle_auth_request(&request_jwt(
            &user_key.public_key(),
            "agent-standard",
            "server-auth-1",
        ))
        .unwrap();

    let response_claims = decode_payload(&response);
    assert_eq!(response_claims["sub"], user_key.public_key());
    assert_eq!(response_claims["aud"], "server-auth-1");
    assert_eq!(response_claims["nats"]["type"], "authorization_response");
    assert_eq!(response_claims["nats"]["version"], 2);
    assert_eq!(
        response_claims["nats"]["issuer_account"],
        handler.issuer_account()
    );

    let user_jwt = response_claims["nats"]["jwt"]
        .as_str()
        .expect("response should contain a user jwt");
    let user_claims = decode_payload(user_jwt);
    assert_eq!(user_claims["sub"], user_key.public_key());
    assert_eq!(user_claims["nats"]["type"], "user");
    assert_eq!(user_claims["nats"]["version"], 2);
    assert_eq!(
        user_claims["nats"]["pub"]["allow"],
        json!(["agents.>", "tasks.>", "workflow.>", "system.health"])
    );
    assert_eq!(
        user_claims["nats"]["sub"]["allow"],
        json!([
            "agents.>",
            "tasks.>",
            "workflow.>",
            "system.health",
            "_INBOX.>"
        ])
    );
    assert_eq!(
        user_claims["exp"].as_i64().unwrap() - user_claims["iat"].as_i64().unwrap(),
        120
    );
}

#[test]
fn record_violation_degrades_trust_and_next_authorization() {
    let handler = handler();
    handler.update_trust("agent-degrades", TrustProfile::new("agent-degrades", 0.95));

    assert_eq!(
        handler.authorize("agent-degrades").unwrap().permission_tier,
        PermissionTier::Full
    );

    for _ in 0..5 {
        handler.record_violation("agent-degrades");
    }

    let degraded = handler.authorize("agent-degrades").unwrap();
    assert_eq!(degraded.permission_tier, PermissionTier::Restricted);
    assert_eq!(degraded.jwt_ttl_secs, 60);
    assert_eq!(
        degraded.permissions.publish_allow,
        vec![
            "agents.agent-degrades.>".to_string(),
            "system.health".to_string()
        ]
    );
}

#[tokio::test]
#[ignore = "requires NATS_URL"]
async fn service_replies_over_nats_auth_callout_subject() {
    let nats_url = std::env::var("NATS_URL").expect("NATS_URL must be set for this test");
    let client = async_nats::connect(nats_url).await.unwrap();

    let handler = handler();
    handler.update_trust(
        "agent-over-nats",
        TrustProfile::new("agent-over-nats", 0.55),
    );
    handler.start(&client).await.unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    let user_key = KeyPair::new_user();
    let response = client
        .request(
            AUTH_CALLOUT_SUBJECT,
            request_jwt(&user_key.public_key(), "agent-over-nats", "server-auth-2")
                .into_bytes()
                .into(),
        )
        .await
        .unwrap();

    let jwt = std::str::from_utf8(&response.payload).unwrap();
    let response_claims = decode_payload(jwt);
    assert_eq!(response_claims["sub"], user_key.public_key());
    assert_eq!(response_claims["aud"], "server-auth-2");
    assert!(response_claims["nats"]["jwt"].as_str().is_some());
}
