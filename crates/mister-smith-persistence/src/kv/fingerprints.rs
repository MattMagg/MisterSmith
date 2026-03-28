//! Typed JetStream KV helpers for packet-021 profile fingerprints.

use async_nats::jetstream::kv::Store;
use chrono::Utc;

use mister_smith_core::{PersistenceError, ProfileFingerprint, ProfileFingerprintRef};

use super::state::{ConflictStrategy, StateManager};

const PROFILE_FINGERPRINT_PREFIX: &str = "profile-fingerprint";
const DISALLOWED_SUMMARY_KEYS: &[&str] = &[
    "transcript",
    "transcripts",
    "raw_transcript",
    "raw_transcripts",
    "raw_payload",
    "messages",
    "message_history",
];

/// Build the stable KV key for a persisted profile fingerprint.
pub fn profile_fingerprint_key(target_kind: &str, target_selector: &str) -> String {
    format!(
        "{PROFILE_FINGERPRINT_PREFIX}:{}:{}",
        normalize_key_segment(target_kind),
        normalize_key_segment(target_selector),
    )
}

/// Typed persistence facade for advisory profile fingerprints.
pub struct ProfileFingerprintStore {
    state: StateManager,
}

impl ProfileFingerprintStore {
    /// Create a new store over an existing JetStream KV bucket.
    pub fn new(store: Store) -> Self {
        Self {
            state: StateManager::new(store, ConflictStrategy::LastWriteWins),
        }
    }

    /// Save or replace a fingerprint keyed by target kind and selector.
    pub async fn save(
        &self,
        fingerprint: &ProfileFingerprint,
    ) -> Result<u64, PersistenceError> {
        validate_profile_fingerprint(fingerprint)?;
        self.state
            .save(
                &profile_fingerprint_key(&fingerprint.target_kind, &fingerprint.target_selector),
                fingerprint,
            )
            .await
    }

    /// Load a fingerprint for the given target, returning `None` when absent.
    pub async fn get(
        &self,
        target_kind: &str,
        target_selector: &str,
    ) -> Result<Option<ProfileFingerprint>, PersistenceError> {
        let fingerprint = self
            .state
            .get(&profile_fingerprint_key(target_kind, target_selector))
            .await?;
        if let Some(fingerprint) = fingerprint.as_ref() {
            validate_profile_fingerprint(fingerprint)?;
        }
        Ok(fingerprint)
    }

    /// Load the current non-expired fingerprint for the given target, when any.
    pub async fn current(
        &self,
        target_kind: &str,
        target_selector: &str,
    ) -> Result<Option<ProfileFingerprint>, PersistenceError> {
        Ok(self
            .get(target_kind, target_selector)
            .await?
            .filter(|fingerprint| fingerprint.expires_at > Utc::now()))
    }

    /// Build the lightweight operator-facing reference for a fingerprint.
    pub fn reference(fingerprint: &ProfileFingerprint) -> ProfileFingerprintRef {
        ProfileFingerprintRef {
            fingerprint_id: fingerprint.fingerprint_id,
            fingerprint_key: profile_fingerprint_key(
                &fingerprint.target_kind,
                &fingerprint.target_selector,
            ),
            confidence: fingerprint.confidence,
            expires_at: fingerprint.expires_at,
        }
    }
}

fn validate_profile_fingerprint(
    fingerprint: &ProfileFingerprint,
) -> Result<(), PersistenceError> {
    if fingerprint.target_kind.trim().is_empty() {
        return Err(PersistenceError::SerializationFailed(
            "profile fingerprint target_kind must not be empty".to_string(),
        ));
    }
    if fingerprint.target_selector.trim().is_empty() {
        return Err(PersistenceError::SerializationFailed(
            "profile fingerprint target_selector must not be empty".to_string(),
        ));
    }
    if fingerprint.source_refs.is_empty() {
        return Err(PersistenceError::SerializationFailed(
            "profile fingerprint must include at least one source reference".to_string(),
        ));
    }
    if !fingerprint.summary_payload.is_object() {
        return Err(PersistenceError::SerializationFailed(
            "profile fingerprint summary_payload must be a JSON object".to_string(),
        ));
    }
    if contains_disallowed_summary_content(&fingerprint.summary_payload) {
        return Err(PersistenceError::SerializationFailed(
            "profile fingerprint summary_payload must not duplicate raw transcript content"
                .to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&fingerprint.confidence) {
        return Err(PersistenceError::SerializationFailed(
            "profile fingerprint confidence must be between 0.0 and 1.0".to_string(),
        ));
    }
    if fingerprint.expires_at <= fingerprint.updated_at {
        return Err(PersistenceError::SerializationFailed(
            "profile fingerprint expires_at must be after updated_at".to_string(),
        ));
    }
    Ok(())
}

fn contains_disallowed_summary_content(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(map) => map.iter().any(|(key, value)| {
            DISALLOWED_SUMMARY_KEYS.contains(&key.as_str())
                || contains_disallowed_summary_content(value)
        }),
        serde_json::Value::Array(values) => values.iter().any(contains_disallowed_summary_content),
        _ => false,
    }
}

fn normalize_key_segment(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use mister_smith_core::{InterventionType, ProfileFingerprint, ProfileFingerprintId};

    #[test]
    fn profile_fingerprint_key_normalizes_segments() {
        assert_eq!(
            profile_fingerprint_key("Executor", "branch/A"),
            "profile-fingerprint:executor:branch_a"
        );
    }

    #[test]
    fn validation_rejects_summary_payload_with_raw_transcript_fields() {
        let fingerprint = ProfileFingerprint {
            fingerprint_id: ProfileFingerprintId::new(),
            target_kind: "executor".to_string(),
            target_selector: "branch-a".to_string(),
            source_refs: vec!["workflow:test".to_string()],
            summary_payload: serde_json::json!({
                "health_state": "degraded",
                "raw_transcript": "verbatim transcript",
            }),
            dominant_failure_modes: vec!["missing_context".to_string()],
            preferred_interventions: vec![InterventionType::ContextRefresh],
            confidence: 0.82,
            updated_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(6),
        };

        let error = validate_profile_fingerprint(&fingerprint).expect_err("should reject");
        assert!(matches!(error, PersistenceError::SerializationFailed(_)));
    }
}
