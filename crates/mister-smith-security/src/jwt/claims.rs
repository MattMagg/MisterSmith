//! JWT claim types for agent authentication.

use std::collections::HashSet;

use mister_smith_core::SecurityError;
use serde::{Deserialize, Serialize};

/// Default maximum number of delegating agents tracked in `delegation_chain`.
pub const DEFAULT_MAX_DELEGATION_CHAIN_DEPTH: usize = 5;

/// JWT claim set combining standard RFC 7519 claims with agent-specific extensions.
///
/// Standard fields follow the JWT spec (`iss`, `sub`, `aud`, `exp`, `nbf`, `iat`, `jti`).
/// Agent-specific fields carry identity and authorization context through the token.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentClaims {
    // ---- Standard JWT claims ----
    /// Issuer — framework instance identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    /// Subject — agent ID as string.
    pub sub: String,
    /// Audience(s).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aud: Vec<String>,
    /// Expiration time (Unix timestamp).
    pub exp: u64,
    /// Not-before time (Unix timestamp).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nbf: Option<u64>,
    /// Issued-at time (Unix timestamp).
    pub iat: u64,
    /// JWT ID — unique token identifier for revocation.
    pub jti: String,

    // ---- Agent-specific claims ----
    /// Agent identifier.
    pub agent_id: String,
    /// Agent type (from `AgentType` enum).
    pub agent_type: String,
    /// Agent capabilities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
    /// Granted permissions in `action:resource:scope` format.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,
    /// Session tracking identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Chain of delegating agents.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delegation_chain: Vec<String>,
    /// Token purpose discriminator (`access` or `refresh`).
    #[serde(default)]
    pub token_use: String,
}

impl AgentClaims {
    /// Validate the delegation chain for empty entries, excessive depth, and cycles.
    pub fn validate_delegation_chain(&self, max_depth: usize) -> Result<(), SecurityError> {
        if self.delegation_chain.len() > max_depth {
            return Err(SecurityError::InvalidToken(format!(
                "delegation_chain exceeds max depth of {max_depth}"
            )));
        }

        let mut seen = HashSet::with_capacity(self.delegation_chain.len());
        for entry in &self.delegation_chain {
            let normalized = entry.trim();
            if normalized.is_empty() {
                return Err(SecurityError::InvalidToken(
                    "delegation_chain entries must be non-empty".to_string(),
                ));
            }

            if normalized == self.agent_id {
                return Err(SecurityError::InvalidToken(
                    "delegation_chain contains a circular reference".to_string(),
                ));
            }

            if !seen.insert(normalized) {
                return Err(SecurityError::InvalidToken(
                    "delegation_chain contains a circular reference".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Derive child claims from the current agent, appending the parent identity to the chain.
    pub fn delegated_to(
        &self,
        agent_id: impl Into<String>,
        agent_type: impl Into<String>,
    ) -> Self {
        let agent_id = agent_id.into();
        let mut child = self.clone();
        child.delegation_chain.push(self.agent_id.clone());
        child.sub = agent_id.clone();
        child.agent_id = agent_id;
        child.agent_type = agent_type.into();
        child.jti.clear();
        child
    }
}

/// Access + refresh token pair issued during authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    /// Short-lived JWT access token.
    pub access_token: String,
    /// Long-lived JWT refresh token.
    pub refresh_token: String,
    /// Token type — always "Bearer".
    pub token_type: String,
    /// Access token TTL in seconds.
    pub expires_in: u64,
}
