//! Integration tests for the NATS auth callout service.

use std::sync::Arc;
use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use futures::StreamExt;
use mister_smith_security::auth_callout::{
    AuthCalloutHandler, PermissionTier, Permissions, TrustProfile, AUTH_CALLOUT_SUBJECT,
};
use mister_smith_security::config::{JwtConfig, KeySource};
use mister_smith_security::jwt::{AgentClaims, JwtManager};
use nkeys::KeyPair;
use serde_json::{json, Value};

fn account_signing_key() -> KeyPair {
    KeyPair::new_account()
}

fn handler() -> AuthCalloutHandler {
    let signing_key = account_signing_key();
    AuthCalloutHandler::new(signing_key.clone(), signing_key.public_key())
}

fn jwt_manager() -> Arc<JwtManager> {
    Arc::new(
        JwtManager::new(&JwtConfig {
            algorithm: "HS256".to_string(),
            access_token_ttl: Duration::from_secs(900),
            refresh_token_ttl: Duration::from_secs(3_600),
            issuer: Some("mister-smith-tests".to_string()),
            audience: vec!["nats-auth-callout".to_string()],
            delegation_chain_max_depth: 5,
            key_source: KeySource::Hmac {
                secret: b"mister-smith-auth-callout-test-secret".to_vec(),
            },
        })
        .expect("test JWT manager should initialize"),
    )
}

fn auth_token(jwt_manager: &JwtManager, agent_id: &str) -> String {
    jwt_manager
        .generate_token_pair(&AgentClaims {
            iss: Some("mister-smith-tests".to_string()),
            sub: agent_id.to_string(),
            aud: vec!["nats-auth-callout".to_string()],
            agent_id: agent_id.to_string(),
            agent_type: "worker".to_string(),
            ..Default::default()
        })
        .expect("test token generation should succeed")
        .access_token
}

#[derive(Default)]
struct RequestAuth {
    claimed_agent_id: Option<String>,
    auth_token: Option<String>,
    nkey: Option<String>,
    signature: Option<String>,
    nonce: Option<String>,
}

fn sign_nonce(key_pair: &KeyPair, nonce: &str) -> String {
    URL_SAFE_NO_PAD.encode(
        key_pair
            .sign(nonce.as_bytes())
            .expect("test nonce signing should succeed"),
    )
}

fn request_jwt(user_nkey: &str, agent_id: &str, server_id: &str) -> String {
    request_jwt_with_auth(
        user_nkey,
        server_id,
        RequestAuth {
            claimed_agent_id: Some(agent_id.to_string()),
            ..RequestAuth::default()
        },
    )
}

fn request_jwt_with_auth(user_nkey: &str, server_id: &str, auth: RequestAuth) -> String {
    let header = URL_SAFE_NO_PAD.encode(br#"{"typ":"JWT","alg":"ed25519-nkey"}"#);
    let claimed_agent_id = auth.claimed_agent_id.as_deref();
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
                "name": claimed_agent_id,
                "nonce": auth.nonce,
                "user": claimed_agent_id
            },
            "connect_opts": {
                "auth_token": auth.auth_token,
                "lang": "rust",
                "name": claimed_agent_id,
                "nkey": auth.nkey,
                "protocol": 1,
                "sig": auth.signature,
                "user": claimed_agent_id
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
fn authorize_unknown_agent_clamps_overridden_fallback_permissions_to_quarantined_ceiling() {
    let handler = handler().with_default_permissions(Permissions {
        publish_allow: vec!["tasks.>".to_string(), "system.health".to_string()],
        publish_deny: vec!["agents.>".to_string()],
        subscribe_allow: vec![
            "_INBOX.>".to_string(),
            "tasks.>".to_string(),
            "system.health".to_string(),
        ],
        subscribe_deny: vec!["agents.>".to_string()],
    });

    let result = handler.authorize("missing-agent").unwrap();

    assert_eq!(result.permission_tier, PermissionTier::Quarantined);
    assert!(result.fallback_applied);
    assert_eq!(
        result.permissions.publish_allow,
        vec!["system.health".to_string()]
    );
    assert_eq!(
        result.permissions.publish_deny,
        vec![
            "$SYS.>".to_string(),
            "$JS.>".to_string(),
            "agents.>".to_string(),
        ]
    );
    assert_eq!(
        result.permissions.subscribe_allow,
        vec!["system.health".to_string(), "_INBOX.>".to_string()]
    );
    assert_eq!(
        result.permissions.subscribe_deny,
        vec![
            "$SYS.>".to_string(),
            "$JS.>".to_string(),
            "agents.>".to_string(),
        ]
    );
}

#[test]
fn handle_auth_request_rejects_claimed_identity_without_verifiable_credentials() {
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
    assert!(response_claims["nats"]["jwt"].is_null());
    let error = response_claims["nats"]["error"]
        .as_str()
        .expect("error response should include a message");
    assert!(error.contains("auth callout request missing verifiable credentials"));
}

#[test]
fn handle_auth_request_emits_scoped_user_jwt_for_known_agent_with_bearer_token() {
    let jwt_manager = jwt_manager();
    let handler = handler().with_jwt_manager(jwt_manager.clone());
    handler.update_trust("agent-standard", TrustProfile::new("agent-standard", 0.55));

    let user_key = KeyPair::new_user();
    let auth_token = auth_token(&jwt_manager, "agent-standard");
    let response = handler
        .handle_auth_request(&request_jwt_with_auth(
            &user_key.public_key(),
            "server-auth-1",
            RequestAuth {
                claimed_agent_id: Some("agent-standard".to_string()),
                auth_token: Some(auth_token),
                ..RequestAuth::default()
            },
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
fn handle_auth_request_emits_scoped_user_jwt_for_verified_nkey_identity() {
    let handler = handler();
    let auth_key = KeyPair::new_user();
    let auth_key_public = auth_key.public_key();
    handler.update_trust(
        &auth_key_public,
        TrustProfile::new(auth_key_public.clone(), 0.55),
    );

    let response = handler
        .handle_auth_request(&request_jwt_with_auth(
            &auth_key_public,
            "server-auth-nkey",
            RequestAuth {
                nkey: Some(auth_key_public.clone()),
                nonce: Some("server-nonce".to_string()),
                signature: Some(sign_nonce(&auth_key, "server-nonce")),
                ..RequestAuth::default()
            },
        ))
        .unwrap();

    let response_claims = decode_payload(&response);
    assert_eq!(response_claims["sub"], auth_key_public);
    assert!(response_claims["nats"]["error"].is_null());
    assert!(response_claims["nats"]["jwt"].as_str().is_some());
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
    let auth_key = KeyPair::new_user();
    let auth_key_public = auth_key.public_key();
    handler.update_trust(
        &auth_key_public,
        TrustProfile::new(auth_key_public.clone(), 0.55),
    );
    handler.start(&client).await.unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    let response = client
        .request(
            AUTH_CALLOUT_SUBJECT,
            request_jwt_with_auth(
                &auth_key_public,
                "server-auth-2",
                RequestAuth {
                    nkey: Some(auth_key_public.clone()),
                    nonce: Some("server-auth-2-nonce".to_string()),
                    signature: Some(sign_nonce(&auth_key, "server-auth-2-nonce")),
                    ..RequestAuth::default()
                },
            )
            .into_bytes()
            .into(),
        )
        .await
        .unwrap();

    let jwt = std::str::from_utf8(&response.payload).unwrap();
    let response_claims = decode_payload(jwt);
    assert_eq!(response_claims["sub"], auth_key_public);
    assert_eq!(response_claims["aud"], "server-auth-2");
    assert!(response_claims["nats"]["jwt"].as_str().is_some());
}

#[tokio::test]
#[ignore = "requires NATS_URL"]
async fn multiple_handlers_reply_once_when_sharing_queue_group() {
    let nats_url = std::env::var("NATS_URL").expect("NATS_URL must be set for this test");
    let client = async_nats::connect(nats_url).await.unwrap();

    let auth_key = KeyPair::new_user();
    let auth_key_public = auth_key.public_key();
    let trust_profile = TrustProfile::new(auth_key_public.clone(), 0.55);

    let handler_one = handler();
    handler_one.update_trust(&auth_key_public, trust_profile.clone());
    handler_one.start(&client).await.unwrap();

    let handler_two = handler();
    handler_two.update_trust(&auth_key_public, trust_profile);
    handler_two.start(&client).await.unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    let reply_subject = format!("_INBOX.auth-callout.{}", uuid::Uuid::new_v4());
    let mut replies = client.subscribe(reply_subject.clone()).await.unwrap();

    client
        .publish_with_reply(
            AUTH_CALLOUT_SUBJECT,
            reply_subject.clone(),
            request_jwt_with_auth(
                &auth_key_public,
                "server-auth-queue",
                RequestAuth {
                    nkey: Some(auth_key_public.clone()),
                    nonce: Some("server-auth-queue-nonce".to_string()),
                    signature: Some(sign_nonce(&auth_key, "server-auth-queue-nonce")),
                    ..RequestAuth::default()
                },
            )
            .into_bytes()
            .into(),
        )
        .await
        .unwrap();

    let first = tokio::time::timeout(Duration::from_secs(1), replies.next())
        .await
        .unwrap()
        .expect("queue group should yield one response");
    let first_jwt = std::str::from_utf8(&first.payload).unwrap();
    let first_claims = decode_payload(first_jwt);
    assert_eq!(first_claims["sub"], auth_key_public);

    let duplicate = tokio::time::timeout(Duration::from_millis(200), replies.next()).await;
    assert!(
        duplicate.is_err(),
        "queue group should prevent duplicate auth responses"
    );
}
