//! Agent sandbox primitives for persistent/ephemeral isolation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use nats_jwt::Token;
use nkeys::{KeyPair, KeyPairType};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use mister_smith_core::SecurityError;
use mister_smith_transport::SubjectTaxonomy;

use crate::auth_callout::Permissions;

const DENY_SUBJECTS: &[&str] = &["$SYS.>", "$JS.>"];
const ASSIGNMENT_SUBJECT_PATTERN: &str = "tasks.*.assignment";
const RESULT_SUBJECT_PATTERN: &str = "tasks.*.result";
const PROGRESS_SUBJECT_PATTERN: &str = "tasks.*.progress";

/// Lifecycle class used for sandbox credential assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum AgentClass {
    /// Stable identity, durable state, and long-lived credentials.
    Persistent,
    /// Short-lived, restricted credentials cleaned up after task completion.
    Ephemeral,
}

/// Per-agent NATS credentials scoped by lifecycle class.
///
/// NOTE: `jwt` and `nats_user` contain sensitive key material. A future hardening
/// pass should add `zeroize::ZeroizeOnDrop` to clear these fields on drop rather
/// than relying on the allocator to reclaim the backing memory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SandboxCredentials {
    /// Agent identifier.
    pub agent_id: String,
    /// Lifecycle class that selected the account and TTL.
    pub agent_class: AgentClass,
    /// Logical NATS account name used for the credential.
    pub nats_account: String,
    /// User public NKey bound to the credential.
    pub nats_user: String,
    /// Signed NATS user JWT.
    pub jwt: String,
    /// Creation time in epoch milliseconds.
    pub created_at: u64,
    /// Expiration time in epoch milliseconds.
    pub expires_at: u64,
}

impl SandboxCredentials {
    /// Returns `true` when the credential is expired at the provided timestamp.
    #[must_use]
    pub fn is_expired_at(&self, now_millis: u64) -> bool {
        now_millis >= self.expires_at
    }
}

/// Explicit rule for permitted cross-boundary communication.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossingRule {
    /// Source NATS account.
    pub source_account: String,
    /// Destination NATS account.
    pub target_account: String,
    /// NATS subject pattern permitted for the crossing.
    pub subject_pattern: String,
    /// Whether the crossing must go through quarantine.
    pub requires_quarantine: bool,
}

impl CrossingRule {
    /// Create a new crossing rule.
    #[must_use]
    pub fn new(
        source_account: impl Into<String>,
        target_account: impl Into<String>,
        subject_pattern: impl Into<String>,
        requires_quarantine: bool,
    ) -> Self {
        Self {
            source_account: source_account.into(),
            target_account: target_account.into(),
            subject_pattern: subject_pattern.into(),
            requires_quarantine,
        }
    }

    fn matches(&self, source_account: &str, target_account: &str, subject: &str) -> bool {
        self.source_account == source_account
            && self.target_account == target_account
            && nats_subject_matches(&self.subject_pattern, subject)
    }
}

/// Result of evaluating a boundary crossing request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossingDecision {
    /// The subject is allowed directly.
    Allow,
    /// The subject is allowed only through quarantine.
    Quarantine,
}

/// Boundary enforcement between persistent and ephemeral agent contexts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IOFirewall {
    /// NATS account for persistent agents.
    pub persistent_account: String,
    /// NATS account for ephemeral agents.
    pub ephemeral_account: String,
    /// Explicit cross-boundary rules.
    pub allowed_crossings: Vec<CrossingRule>,
}

impl IOFirewall {
    /// Create a new firewall with explicit crossing rules.
    #[must_use]
    pub fn new(
        persistent_account: impl Into<String>,
        ephemeral_account: impl Into<String>,
        allowed_crossings: Vec<CrossingRule>,
    ) -> Self {
        Self {
            persistent_account: persistent_account.into(),
            ephemeral_account: ephemeral_account.into(),
            allowed_crossings,
        }
    }

    /// Create a firewall with the default sandbox crossing rules.
    #[must_use]
    pub fn with_default_rules(
        persistent_account: impl Into<String>,
        ephemeral_account: impl Into<String>,
    ) -> Self {
        let persistent_account = persistent_account.into();
        let ephemeral_account = ephemeral_account.into();

        Self::new(
            persistent_account.clone(),
            ephemeral_account.clone(),
            vec![
                CrossingRule::new(
                    persistent_account.clone(),
                    ephemeral_account.clone(),
                    ASSIGNMENT_SUBJECT_PATTERN,
                    true,
                ),
                CrossingRule::new(
                    ephemeral_account.clone(),
                    persistent_account.clone(),
                    RESULT_SUBJECT_PATTERN,
                    true,
                ),
                CrossingRule::new(
                    ephemeral_account.clone(),
                    persistent_account.clone(),
                    PROGRESS_SUBJECT_PATTERN,
                    true,
                ),
            ],
        )
    }

    /// Evaluate whether the requested subject crossing is permitted.
    pub fn check_crossing(
        &self,
        source_account: &str,
        target_account: &str,
        subject: &str,
    ) -> Result<CrossingDecision, SecurityError> {
        validate_subject(subject)?;

        if source_account == target_account && self.is_known_account(source_account) {
            return Ok(CrossingDecision::Allow);
        }

        let rule = self
            .allowed_crossings
            .iter()
            .find(|rule| rule.matches(source_account, target_account, subject))
            .ok_or_else(|| {
                SecurityError::AuthorizationDenied(format!(
                    "cross-boundary subject '{subject}' is not permitted from '{source_account}' to '{target_account}'"
                ))
            })?;

        if rule.requires_quarantine {
            Ok(CrossingDecision::Quarantine)
        } else {
            Ok(CrossingDecision::Allow)
        }
    }

    fn is_known_account(&self, account: &str) -> bool {
        account == self.persistent_account || account == self.ephemeral_account
    }
}

/// Account configuration used to issue lifecycle-scoped credentials.
#[derive(Debug, Clone)]
pub struct SandboxAccountConfig {
    /// Logical account name for audit and metadata.
    pub account_name: String,
    /// Issuer account NKey embedded in the user JWT.
    pub issuer_account: String,
    /// Account signing key used to sign user JWTs.
    pub signing_key: KeyPair,
    /// JWT lifetime for issued users in this account.
    pub jwt_ttl: Duration,
}

impl SandboxAccountConfig {
    /// Create a new account configuration.
    #[must_use]
    pub fn new(
        account_name: impl Into<String>,
        issuer_account: impl Into<String>,
        signing_key: KeyPair,
        jwt_ttl: Duration,
    ) -> Self {
        Self {
            account_name: account_name.into(),
            issuer_account: issuer_account.into(),
            signing_key,
            jwt_ttl,
        }
    }
}

/// Issues and tracks active sandbox credentials for agents.
#[derive(Clone)]
pub struct SandboxCredentialIssuer {
    persistent: SandboxAccountConfig,
    ephemeral: SandboxAccountConfig,
    credentials: Arc<RwLock<HashMap<String, SandboxCredentials>>>,
}

impl SandboxCredentialIssuer {
    /// Create a new issuer with distinct persistent and ephemeral accounts.
    #[must_use]
    pub fn new(persistent: SandboxAccountConfig, ephemeral: SandboxAccountConfig) -> Self {
        Self {
            persistent,
            ephemeral,
            credentials: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Issue credentials for the given agent and lifecycle class.
    ///
    /// If credentials already exist for `agent_id`, the previous entry is silently
    /// replaced. The old JWT remains valid until its natural expiration since there
    /// is no revocation step. Callers that re-issue credentials (e.g. after a
    /// supervision restart) should explicitly [`cleanup`](Self::cleanup) the old
    /// entry first if revocation semantics are needed.
    pub fn create_credentials(
        &self,
        agent_id: impl Into<String>,
        agent_class: AgentClass,
    ) -> Result<SandboxCredentials, SecurityError> {
        let agent_id = agent_id.into();
        let agent_id = agent_id.trim();
        if agent_id.is_empty() {
            return Err(SecurityError::AuthenticationFailed(
                "sandbox agent_id must not be empty".to_string(),
            ));
        }

        let account = self.account(agent_class);
        if account.signing_key.key_pair_type() != KeyPairType::Account {
            return Err(SecurityError::TokenGenerationFailed(
                "sandbox signing key must be an account key".to_string(),
            ));
        }

        let user_key = KeyPair::new_user();
        let nats_user = user_key.public_key();
        let created_at = unix_timestamp_millis();
        let expires_at = created_at
            .checked_add(account.jwt_ttl.as_millis() as u64)
            .ok_or_else(|| {
                SecurityError::TokenGenerationFailed(
                    "sandbox credential expiration overflow".to_string(),
                )
            })?;
        let expires_secs = unix_timestamp_secs()
            .checked_add(account.jwt_ttl.as_secs() as i64)
            .ok_or_else(|| {
                SecurityError::TokenGenerationFailed("sandbox jwt expiration overflow".to_string())
            })?;

        let permissions = permissions_for(agent_id, agent_class);
        let jwt = permissions
            .clone()
            .apply_to(
                Token::new_user(account.issuer_account.clone(), nats_user.clone())
                    .name(agent_id.to_string())
                    .expires(expires_secs),
            )
            .sign(&account.signing_key);

        let credentials = SandboxCredentials {
            agent_id: agent_id.to_string(),
            agent_class,
            nats_account: account.account_name.clone(),
            nats_user,
            jwt,
            created_at,
            expires_at,
        };

        self.credentials
            .write()
            .insert(credentials.agent_id.clone(), credentials.clone());

        Ok(credentials)
    }

    /// Return the active credentials for an agent, if present.
    #[must_use]
    pub fn credentials(&self, agent_id: &str) -> Option<SandboxCredentials> {
        self.credentials.read().get(agent_id).cloned()
    }

    /// Remove the active credentials for an agent.
    #[must_use]
    pub fn cleanup(&self, agent_id: &str) -> Option<SandboxCredentials> {
        self.credentials.write().remove(agent_id)
    }

    /// Remove all expired credentials and return the number removed.
    #[must_use]
    pub fn cleanup_expired(&self, now_millis: u64) -> usize {
        let mut guard = self.credentials.write();
        let before = guard.len();
        guard.retain(|_, credentials| !credentials.is_expired_at(now_millis));
        before.saturating_sub(guard.len())
    }

    fn account(&self, agent_class: AgentClass) -> &SandboxAccountConfig {
        match agent_class {
            AgentClass::Persistent => &self.persistent,
            AgentClass::Ephemeral => &self.ephemeral,
        }
    }
}

fn permissions_for(agent_id: &str, agent_class: AgentClass) -> Permissions {
    let inbox = "_INBOX.>".to_string();
    let health = SubjectTaxonomy::system_health();
    let deny: Vec<String> = DENY_SUBJECTS
        .iter()
        .map(|subject| (*subject).to_string())
        .collect();

    match agent_class {
        AgentClass::Persistent => Permissions {
            publish_allow: vec![
                format!("agents.{agent_id}.>"),
                format!("state.persistent.{agent_id}.>"),
                ASSIGNMENT_SUBJECT_PATTERN.to_string(),
                SubjectTaxonomy::all_workflows(),
                health.clone(),
            ],
            publish_deny: deny.clone(),
            subscribe_allow: vec![
                format!("agents.{agent_id}.>"),
                format!("state.persistent.{agent_id}.>"),
                RESULT_SUBJECT_PATTERN.to_string(),
                PROGRESS_SUBJECT_PATTERN.to_string(),
                SubjectTaxonomy::all_workflows(),
                health,
                inbox,
            ],
            subscribe_deny: deny,
        },
        AgentClass::Ephemeral => Permissions {
            publish_allow: vec![
                format!("agents.{agent_id}.>"),
                format!("state.ephemeral.{agent_id}.>"),
                RESULT_SUBJECT_PATTERN.to_string(),
                PROGRESS_SUBJECT_PATTERN.to_string(),
                health.clone(),
            ],
            publish_deny: deny.clone(),
            subscribe_allow: vec![
                format!("agents.{agent_id}.>"),
                format!("state.ephemeral.{agent_id}.>"),
                ASSIGNMENT_SUBJECT_PATTERN.to_string(),
                health,
                inbox,
            ],
            subscribe_deny: deny,
        },
    }
}

fn validate_subject(subject: &str) -> Result<(), SecurityError> {
    if subject.trim().is_empty() {
        return Err(SecurityError::AuthorizationDenied(
            "sandbox subject must not be empty".to_string(),
        ));
    }
    if subject.contains(' ') {
        return Err(SecurityError::AuthorizationDenied(format!(
            "sandbox subject must not contain spaces: {subject}"
        )));
    }
    Ok(())
}

fn nats_subject_matches(pattern: &str, subject: &str) -> bool {
    if pattern == ">" {
        return true;
    }

    let mut subject_tokens = subject.split('.');
    let mut pattern_tokens = pattern.split('.');

    loop {
        match (pattern_tokens.next(), subject_tokens.next()) {
            (Some(">"), _) => return true,
            (Some("*"), Some(_)) => continue,
            (Some(pattern_token), Some(subject_token)) if pattern_token == subject_token => {
                continue
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

fn unix_timestamp_secs() -> i64 {
    (unix_timestamp_millis() / 1_000) as i64
}

fn unix_timestamp_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_matching_supports_single_and_multi_token_patterns() {
        assert!(nats_subject_matches("*", "tasks"));
        assert!(!nats_subject_matches("*", "tasks.analysis.assignment"));
        assert!(nats_subject_matches(
            "tasks.*.assignment",
            "tasks.analysis.assignment"
        ));
        assert!(nats_subject_matches("tasks.>", "tasks.analysis.assignment"));
        assert!(!nats_subject_matches(
            "tasks.*.result",
            "tasks.analysis.assignment"
        ));
    }
}
