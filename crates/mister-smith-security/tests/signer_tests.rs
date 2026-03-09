use std::sync::Arc;
use std::time::Duration;

use mister_smith_core::SecurityError;
use mister_smith_security::{
    audit::{AuditEventType, AuditLogger, AuditOutcome},
    config::AuditConfig,
    jwt::AgentClaims,
    middleware::nats_mw::SecureTransport,
    HmacKey, HmacMessageSigner, MessageSigner, MessageSigningConfig,
};
use mister_smith_transport::{InMemoryTransport, MessageEnvelope, Transport, TransportError};
use uuid::Uuid;

fn signing_key(id: &str, secret: &[u8]) -> HmacKey {
    HmacKey::new(id, secret.to_vec())
}

fn required_config() -> MessageSigningConfig {
    MessageSigningConfig {
        active_key: signing_key(
            "active",
            b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ),
        require_signatures: true,
        nonce_window_size: 10_000,
        grace_period: Duration::from_secs(300),
    }
}

fn optional_config() -> MessageSigningConfig {
    MessageSigningConfig {
        require_signatures: false,
        ..required_config()
    }
}

fn signer() -> HmacMessageSigner {
    HmacMessageSigner::new(required_config()).expect("test signer should initialize")
}

fn audit_logger() -> Arc<AuditLogger> {
    Arc::new(AuditLogger::new(&AuditConfig {
        enabled: true,
        max_events: 100,
        auth_failure_alert_threshold: 5,
    }))
}

fn transport_claims() -> AgentClaims {
    AgentClaims {
        sub: "agent-secure".to_string(),
        agent_id: "agent-secure".to_string(),
        agent_type: "worker".to_string(),
        ..Default::default()
    }
}

fn unsigned_envelope(message_type: &str) -> MessageEnvelope {
    MessageEnvelope::builder(message_type)
        .payload_raw(format!("payload:{message_type}").into_bytes())
        .header("x-agent", "planner")
        .header("x-region", "us-east-1")
        .build()
        .expect("test envelope should build")
}

fn signed_envelope(signer: &dyn MessageSigner, message_type: &str) -> (MessageEnvelope, String) {
    let mut envelope = unsigned_envelope(message_type);
    envelope.nonce = Some(signer.generate_nonce());
    let signature = signer.sign(&envelope).expect("sign should succeed");
    envelope.signature = Some(signature.clone());
    (envelope, signature)
}

#[test]
fn sign_verify_round_trip() {
    let signer = signer();
    let (envelope, signature) = signed_envelope(&signer, "security.roundtrip");

    assert!(signer.verify(&envelope, &signature).unwrap());
    assert!(signer.validate_envelope(&envelope).is_ok());
}

#[test]
fn forged_message_rejected() {
    let signer = signer();
    let (mut envelope, signature) = signed_envelope(&signer, "security.forgery");

    envelope.payload = b"forged".to_vec();

    assert!(!signer.verify(&envelope, &signature).unwrap());
    assert!(matches!(
        signer.validate_envelope(&envelope),
        Err(SecurityError::InvalidSignature)
    ));
}

#[test]
fn replay_rejected() {
    let signer = signer();
    let (envelope, _) = signed_envelope(&signer, "security.replay");

    signer
        .validate_envelope(&envelope)
        .expect("first delivery should pass");

    assert!(matches!(
        signer.validate_envelope(&envelope),
        Err(SecurityError::ReplayDetected { .. })
    ));
}

#[test]
fn same_nonce_from_distinct_senders_is_not_treated_as_replay() {
    let signer = signer();
    let shared_nonce = signer.generate_nonce();

    let mut sender_a = unsigned_envelope("security.sender-a");
    sender_a.source_agent_id = Some(Uuid::new_v4());
    sender_a.nonce = Some(shared_nonce.clone());
    let sender_a_signature = signer.sign(&sender_a).expect("sign should succeed");
    sender_a.signature = Some(sender_a_signature);

    let mut sender_b = unsigned_envelope("security.sender-b");
    sender_b.source_agent_id = Some(Uuid::new_v4());
    sender_b.nonce = Some(shared_nonce);
    let sender_b_signature = signer.sign(&sender_b).expect("sign should succeed");
    sender_b.signature = Some(sender_b_signature);

    signer
        .validate_envelope(&sender_a)
        .expect("first sender should pass replay validation");
    signer
        .validate_envelope(&sender_b)
        .expect("different sender should not collide on nonce replay tracking");
}

#[test]
fn same_nonce_from_same_sender_is_rejected_even_when_payload_changes() {
    let signer = signer();
    let source_agent_id = Uuid::new_v4();
    let shared_nonce = signer.generate_nonce();

    let mut first = unsigned_envelope("security.same-sender.first");
    first.source_agent_id = Some(source_agent_id);
    first.nonce = Some(shared_nonce.clone());
    let first_signature = signer.sign(&first).expect("sign should succeed");
    first.signature = Some(first_signature);

    let mut second = unsigned_envelope("security.same-sender.second");
    second.source_agent_id = Some(source_agent_id);
    second.nonce = Some(shared_nonce);
    let second_signature = signer.sign(&second).expect("sign should succeed");
    second.signature = Some(second_signature);

    signer
        .validate_envelope(&first)
        .expect("first delivery should pass");

    assert!(matches!(
        signer.validate_envelope(&second),
        Err(SecurityError::ReplayDetected { .. })
    ));
}

#[test]
fn key_rotation_grace_period_accepts_old_and_new_keys() {
    let signer = HmacMessageSigner::new(MessageSigningConfig {
        grace_period: Duration::from_millis(50),
        ..required_config()
    })
    .expect("test signer should initialize");

    let (old_envelope, old_signature) = signed_envelope(&signer, "security.rotation.old");

    signer
        .rotate_key(signing_key(
            "next",
            b"abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ))
        .expect("rotation should succeed");

    assert!(signer.verify(&old_envelope, &old_signature).unwrap());

    let (new_envelope, new_signature) = signed_envelope(&signer, "security.rotation.new");
    assert!(signer.verify(&new_envelope, &new_signature).unwrap());

    std::thread::sleep(Duration::from_millis(75));
    assert!(!signer.verify(&old_envelope, &old_signature).unwrap());
}

#[test]
fn nonce_window_overflow_evicts_oldest() {
    let signer = HmacMessageSigner::new(MessageSigningConfig {
        nonce_window_size: 2,
        ..required_config()
    })
    .expect("test signer should initialize");

    let nonce_a = signer.generate_nonce();
    let nonce_b = signer.generate_nonce();
    let nonce_c = signer.generate_nonce();

    signer.record_nonce(&nonce_a);
    signer.record_nonce(&nonce_b);
    signer.record_nonce(&nonce_c);

    assert!(!signer.is_replay(&nonce_a));
    assert!(signer.is_replay(&nonce_b));
    assert!(signer.is_replay(&nonce_c));
}

#[test]
fn missing_signature_rejected_when_required() {
    let signer = signer();
    let mut envelope = unsigned_envelope("security.missing-signature");
    envelope.nonce = Some(signer.generate_nonce());

    assert!(matches!(
        signer.validate_envelope(&envelope),
        Err(SecurityError::MissingSignature)
    ));
}

#[test]
fn backward_compatibility_accepts_legacy_message_when_optional() {
    let signer =
        HmacMessageSigner::new(optional_config()).expect("optional signer should initialize");
    let envelope = unsigned_envelope("security.legacy");

    assert!(signer.validate_envelope(&envelope).is_ok());
}

#[tokio::test]
async fn secure_transport_signs_outbound_publish() {
    let inner = InMemoryTransport::new();
    let mut raw_subscription = inner.subscribe("signed.publish").await.unwrap();
    let signer: Arc<dyn MessageSigner> = Arc::new(signer());
    let secure =
        SecureTransport::new(inner.clone(), None, transport_claims()).with_message_signer(signer);

    secure
        .publish("signed.publish", unsigned_envelope("security.publish"))
        .await
        .unwrap();

    let received = tokio::time::timeout(Duration::from_secs(1), raw_subscription.next())
        .await
        .unwrap()
        .unwrap();
    assert!(received.envelope.signature.is_some());
    assert!(received.envelope.nonce.is_some());
}

#[tokio::test]
async fn secure_transport_drops_forged_subscription_messages_and_audits() {
    let inner = InMemoryTransport::new();
    let signer: Arc<dyn MessageSigner> = Arc::new(signer());
    let audit = audit_logger();
    let secure = SecureTransport::new(inner.clone(), None, transport_claims())
        .with_message_signer(signer)
        .with_audit_logger(audit.clone());
    let mut subscription = secure.subscribe("signed.inbound").await.unwrap();

    inner
        .publish("signed.inbound", unsigned_envelope("security.unsigned"))
        .await
        .unwrap();

    let timeout = tokio::time::timeout(Duration::from_millis(100), subscription.next()).await;
    assert!(timeout.is_err(), "unsigned message should be filtered out");

    let events = audit.recent_events(10);
    assert!(events.iter().any(|event| {
        event.event_type == AuditEventType::SuspiciousActivity
            && event.outcome == AuditOutcome::Blocked
    }));
}

#[tokio::test]
async fn secure_transport_request_rejects_unsigned_response_with_typed_security_error() {
    let inner = InMemoryTransport::new();
    let signer: Arc<dyn MessageSigner> = Arc::new(signer());
    let secure =
        SecureTransport::new(inner.clone(), None, transport_claims()).with_message_signer(signer);
    let responder = inner.clone();
    let mut sub = inner.subscribe("signed.request").await.unwrap();

    tokio::spawn(async move {
        let request = sub.next().await.unwrap();
        assert!(request.envelope.signature.is_some());
        assert!(request.envelope.nonce.is_some());

        let response = MessageEnvelope::builder("security.response")
            .correlation_id(request.envelope.correlation_id.unwrap())
            .payload_raw(b"unsigned response".to_vec())
            .build()
            .unwrap();
        responder
            .publish(request.reply_subject.as_deref().unwrap(), response)
            .await
            .unwrap();
    });

    let result = secure
        .request(
            "signed.request",
            unsigned_envelope("security.request"),
            Duration::from_secs(1),
        )
        .await;

    assert!(
        matches!(
            result,
            Err(TransportError::Security(SecurityError::MissingSignature))
        ),
        "unexpected request result: {result:?}"
    );
}

#[tokio::test]
async fn secure_transport_drops_forged_queue_subscription_messages() {
    let inner = InMemoryTransport::new();
    let signer: Arc<dyn MessageSigner> = Arc::new(signer());
    let audit = audit_logger();
    let secure = SecureTransport::new(inner.clone(), None, transport_claims())
        .with_message_signer(signer)
        .with_audit_logger(audit.clone());
    let mut subscription = secure
        .queue_subscribe("signed.queue", "workers")
        .await
        .unwrap();

    inner
        .publish("signed.queue", unsigned_envelope("security.unsigned"))
        .await
        .unwrap();

    let timeout = tokio::time::timeout(Duration::from_millis(100), subscription.next()).await;
    assert!(
        timeout.is_err(),
        "unsigned message should be filtered out by queue subscription"
    );

    let events = audit.recent_events(10);
    assert!(events.iter().any(|event| {
        event.event_type == AuditEventType::SuspiciousActivity
            && event.outcome == AuditOutcome::Blocked
    }));
}

#[test]
fn header_mutation_invalidates_signature() {
    let signer = signer();
    let (mut envelope, signature) = signed_envelope(&signer, "security.headers");

    // Mutate a header value after signing — the signature must no longer verify.
    envelope
        .headers
        .insert("x-agent".to_string(), "attacker".to_string());

    assert!(
        !signer.verify(&envelope, &signature).unwrap(),
        "signature should be invalid after header mutation"
    );
}
