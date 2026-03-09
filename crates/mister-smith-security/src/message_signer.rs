//! HMAC-SHA256 message signing for transport envelopes.

use std::collections::{BTreeMap, HashSet, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::{Mutex, RwLock};
use ring::hmac;
use serde::Serialize;

use mister_smith_core::SecurityError;
use mister_smith_transport::MessageEnvelope;

/// Minimum HMAC key length in bytes (NIST SP 800-107: at least the hash output length).
const MIN_KEY_LENGTH: usize = 32;

/// Symmetric HMAC key material used for message signing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HmacKey {
    key_id: String,
    secret: Vec<u8>,
}

impl HmacKey {
    /// Construct a new HMAC key from an identifier and raw secret bytes.
    pub fn new(key_id: impl Into<String>, secret: Vec<u8>) -> Self {
        Self {
            key_id: key_id.into(),
            secret,
        }
    }

    fn validate(&self) -> Result<(), SecurityError> {
        if self.key_id.trim().is_empty() {
            return Err(SecurityError::KeyLoadFailed(
                "message-signing key_id must not be empty".to_string(),
            ));
        }

        if self.secret.is_empty() {
            return Err(SecurityError::KeyLoadFailed(
                "message-signing secret must not be empty".to_string(),
            ));
        }

        if self.secret.len() < MIN_KEY_LENGTH {
            return Err(SecurityError::KeyLoadFailed(format!(
                "message-signing secret must be at least {MIN_KEY_LENGTH} bytes \
                 (NIST SP 800-107); got {} bytes",
                self.secret.len()
            )));
        }

        Ok(())
    }

    fn ring_key(&self) -> hmac::Key {
        hmac::Key::new(hmac::HMAC_SHA256, &self.secret)
    }
}

/// Runtime configuration for the HMAC message signer.
#[derive(Debug, Clone)]
pub struct MessageSigningConfig {
    /// Active key used for newly-signed messages.
    pub active_key: HmacKey,
    /// Whether unsigned messages are rejected during validation.
    pub require_signatures: bool,
    /// Maximum number of recent nonces retained for replay detection.
    pub nonce_window_size: usize,
    /// How long rotated keys remain valid for verification.
    pub grace_period: Duration,
}

impl MessageSigningConfig {
    /// Create a config with sensible defaults for replay tracking and rotation.
    pub fn new(active_key: HmacKey) -> Self {
        Self {
            active_key,
            require_signatures: true,
            nonce_window_size: 10_000,
            grace_period: Duration::from_secs(300),
        }
    }
}

#[derive(Debug, Clone)]
struct GraceKey {
    key: HmacKey,
    expires_at: Instant,
}

#[derive(Debug)]
struct NonceTracker {
    last_timestamp_ms: u128,
    counter: u64,
    seen: HashSet<String>,
    order: VecDeque<String>,
    capacity: usize,
}

impl NonceTracker {
    fn new(capacity: usize) -> Self {
        Self {
            last_timestamp_ms: 0,
            counter: 0,
            seen: HashSet::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Generate a monotonic nonce prefixed with the signing key ID so that
    /// the replay cache is effectively namespaced per sender/key.
    fn generate_nonce(&mut self, key_id: &str) -> String {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        if now_ms > self.last_timestamp_ms {
            self.last_timestamp_ms = now_ms;
            self.counter = 0;
        } else {
            self.counter = self.counter.saturating_add(1);
        }

        format!(
            "{}:{:020}-{:016x}",
            key_id, self.last_timestamp_ms, self.counter
        )
    }

    fn is_replay(&self, nonce: &str) -> bool {
        self.seen.contains(nonce)
    }

    fn record(&mut self, nonce: &str) {
        if self.capacity == 0 || self.seen.contains(nonce) {
            return;
        }

        let nonce_string = nonce.to_string();
        self.seen.insert(nonce_string.clone());
        self.order.push_back(nonce_string);

        while self.order.len() > self.capacity {
            if let Some(evicted) = self.order.pop_front() {
                self.seen.remove(&evicted);
            }
        }
    }

    fn validate_and_record(&mut self, nonce: &str) -> Result<(), SecurityError> {
        if self.is_replay(nonce) {
            return Err(SecurityError::ReplayDetected {
                nonce: nonce.to_string(),
            });
        }

        self.record(nonce);
        Ok(())
    }
}

/// Trait for message-envelope signing and replay protection.
pub trait MessageSigner: Send + Sync {
    /// Compute an HMAC-SHA256 signature over canonical envelope contents.
    fn sign(&self, envelope: &MessageEnvelope) -> Result<String, SecurityError>;

    /// Verify a signature against the provided envelope contents.
    fn verify(&self, envelope: &MessageEnvelope, signature: &str) -> Result<bool, SecurityError>;

    /// Generate a monotonic nonce for a new outbound message.
    fn generate_nonce(&self) -> String;

    /// Return whether a nonce has already been observed.
    fn is_replay(&self, nonce: &str) -> bool;

    /// Record a nonce as observed.
    fn record_nonce(&self, nonce: &str);

    /// Rotate the active key, keeping the previous one alive for the grace window.
    fn rotate_key(&self, new_key: HmacKey) -> Result<(), SecurityError>;

    /// Return whether envelopes without signatures should be rejected.
    fn requires_signatures(&self) -> bool;

    /// Validate and record a nonce atomically.
    ///
    /// # Concurrency Warning
    ///
    /// The default implementation calls [`is_replay`](Self::is_replay) and
    /// [`record_nonce`](Self::record_nonce) as **separate** operations,
    /// which is susceptible to TOCTOU races under concurrent access.
    /// Implementors **must** override this method with an atomic
    /// check-and-record operation when the signer may be called from
    /// multiple tasks or threads.
    fn validate_nonce(&self, nonce: &str) -> Result<(), SecurityError> {
        if self.is_replay(nonce) {
            return Err(SecurityError::ReplayDetected {
                nonce: nonce.to_string(),
            });
        }

        self.record_nonce(nonce);
        Ok(())
    }

    /// Validate an inbound envelope end-to-end.
    fn validate_envelope(&self, envelope: &MessageEnvelope) -> Result<(), SecurityError> {
        match (envelope.signature.as_deref(), envelope.nonce.as_deref()) {
            (None, None) if !self.requires_signatures() => Ok(()),
            (None, _) => Err(SecurityError::MissingSignature),
            (_, None) => Err(SecurityError::MissingNonce),
            (Some(signature), Some(nonce)) => {
                if !self.verify(envelope, signature)? {
                    return Err(SecurityError::InvalidSignature);
                }

                self.validate_nonce(nonce)
            }
        }
    }
}

/// HMAC-SHA256 signer with nonce replay tracking and grace-key rotation.
pub struct HmacMessageSigner {
    active_key: RwLock<HmacKey>,
    grace_keys: Mutex<Vec<GraceKey>>,
    nonce_tracker: Mutex<NonceTracker>,
    require_signatures: bool,
    grace_period: Duration,
}

impl HmacMessageSigner {
    /// Build a new signer from the provided runtime configuration.
    pub fn new(config: MessageSigningConfig) -> Result<Self, SecurityError> {
        config.active_key.validate()?;

        if config.nonce_window_size == 0 {
            return Err(SecurityError::SigningFailed(
                "nonce_window_size must be greater than zero".to_string(),
            ));
        }

        Ok(Self {
            active_key: RwLock::new(config.active_key),
            grace_keys: Mutex::new(Vec::new()),
            nonce_tracker: Mutex::new(NonceTracker::new(config.nonce_window_size)),
            require_signatures: config.require_signatures,
            grace_period: config.grace_period,
        })
    }

    fn canonicalize_envelope(envelope: &MessageEnvelope) -> Result<Vec<u8>, SecurityError> {
        let headers = envelope
            .headers
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .collect::<BTreeMap<_, _>>();

        let canonical = CanonicalEnvelope {
            message_id: &envelope.message_id,
            timestamp: &envelope.timestamp,
            schema_version: &envelope.schema_version,
            message_type: &envelope.message_type,
            correlation_id: &envelope.correlation_id,
            trace_id: &envelope.trace_id,
            source_agent_id: &envelope.source_agent_id,
            target_agent_id: &envelope.target_agent_id,
            priority: envelope.priority,
            payload: &envelope.payload,
            headers,
            plane: &envelope.plane,
            stream_class: &envelope.stream_class,
            nonce: &envelope.nonce,
            capability_token: &envelope.capability_token,
        };

        serde_json::to_vec(&canonical)
            .map_err(|error| SecurityError::SigningFailed(error.to_string()))
    }

    fn prune_expired_grace_keys(&self) {
        let now = Instant::now();
        self.grace_keys.lock().retain(|key| key.expires_at > now);
    }

    fn verification_keys(&self) -> Vec<HmacKey> {
        self.prune_expired_grace_keys();

        let mut keys = vec![self.active_key.read().clone()];
        keys.extend(self.grace_keys.lock().iter().map(|key| key.key.clone()));
        keys
    }
}

impl MessageSigner for HmacMessageSigner {
    fn sign(&self, envelope: &MessageEnvelope) -> Result<String, SecurityError> {
        let canonical = Self::canonicalize_envelope(envelope)?;
        let active_key = self.active_key.read();
        let signature = hmac::sign(&active_key.ring_key(), &canonical);
        Ok(hex::encode(signature.as_ref()))
    }

    fn verify(&self, envelope: &MessageEnvelope, signature: &str) -> Result<bool, SecurityError> {
        let signature_bytes = match hex::decode(signature) {
            Ok(bytes) => bytes,
            Err(_) => return Ok(false),
        };

        let canonical = Self::canonicalize_envelope(envelope)?;

        for key in self.verification_keys() {
            if hmac::verify(&key.ring_key(), &canonical, &signature_bytes).is_ok() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn generate_nonce(&self) -> String {
        let key_id = self.active_key.read().key_id.clone();
        self.nonce_tracker.lock().generate_nonce(&key_id)
    }

    fn is_replay(&self, nonce: &str) -> bool {
        self.nonce_tracker.lock().is_replay(nonce)
    }

    fn record_nonce(&self, nonce: &str) {
        self.nonce_tracker.lock().record(nonce);
    }

    fn rotate_key(&self, new_key: HmacKey) -> Result<(), SecurityError> {
        new_key
            .validate()
            .map_err(|error| SecurityError::KeyRotationFailed(error.to_string()))?;

        self.prune_expired_grace_keys();

        let mut active = self.active_key.write();
        let previous_key = active.clone();
        *active = new_key;
        drop(active);

        self.grace_keys.lock().push(GraceKey {
            key: previous_key,
            expires_at: Instant::now() + self.grace_period,
        });

        Ok(())
    }

    fn requires_signatures(&self) -> bool {
        self.require_signatures
    }

    fn validate_nonce(&self, nonce: &str) -> Result<(), SecurityError> {
        self.nonce_tracker.lock().validate_and_record(nonce)
    }
}

#[derive(Serialize)]
struct CanonicalEnvelope<'a> {
    message_id: &'a uuid::Uuid,
    timestamp: &'a chrono::DateTime<chrono::Utc>,
    schema_version: &'a str,
    message_type: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    correlation_id: &'a Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trace_id: &'a Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_agent_id: &'a Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_agent_id: &'a Option<uuid::Uuid>,
    priority: mister_smith_transport::MessagePriority,
    payload: &'a [u8],
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    headers: BTreeMap<&'a str, &'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plane: &'a Option<mister_smith_transport::MessagePlane>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_class: &'a Option<mister_smith_transport::StreamClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    nonce: &'a Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    capability_token: &'a Option<String>,
}
