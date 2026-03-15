//! JWT claim types for agent authentication.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use mister_smith_core::{
    AgentId, CapabilityId, DelegationCapability, ProvenanceChain, ProvenanceLink, RevocationState,
    SecurityError,
};
use serde::{Deserialize, Serialize};

use crate::delegation::authority_principal_for_agent_id;

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
    /// Bounded capability for privileged delegated work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delegation_capability: Option<DelegationCapability>,
    /// Reconstructable authority lineage for the capability.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance_chain: Option<ProvenanceChain>,
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

        match (&self.delegation_capability, &self.provenance_chain) {
            (None, None) => {}
            (Some(_), None) | (None, Some(_)) => {
                return Err(SecurityError::InvalidToken(
                    "delegation capability and provenance chain must be present together"
                        .to_string(),
                ));
            }
            (Some(capability), Some(provenance)) => {
                if capability.recipient.to_string() != self.agent_id {
                    return Err(SecurityError::InvalidToken(
                        "delegation capability recipient does not match agent_id".to_string(),
                    ));
                }
                if provenance.terminal_capability != capability.capability_id {
                    return Err(SecurityError::InvalidToken(
                        "delegation provenance terminal capability does not match claims"
                            .to_string(),
                    ));
                }
                if provenance.links.is_empty() {
                    return Err(SecurityError::InvalidToken(
                        "delegation provenance must contain at least one link".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Derive child claims from the current agent, appending the parent identity to the chain.
    pub fn delegated_to(&self, agent_id: impl Into<String>, agent_type: impl Into<String>) -> Self {
        let agent_id = agent_id.into();
        let mut child = self.clone();
        child.delegation_chain.push(self.agent_id.clone());
        child.sub = agent_id.clone();
        child.agent_id = agent_id;
        child.agent_type = agent_type.into();
        child.jti.clear();
        let child_capability = derive_child_capability(self, &child.agent_id);
        child.provenance_chain = derive_child_provenance(self, child_capability.as_ref());
        child.delegation_capability = child_capability;
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

fn derive_child_capability(
    parent: &AgentClaims,
    child_agent_id: &str,
) -> Option<DelegationCapability> {
    let parent_capability = parent.delegation_capability.as_ref()?;
    let child_agent_id = Uuid::parse_str(child_agent_id).ok()?;
    Some(DelegationCapability {
        capability_id: CapabilityId::new(),
        issuer: authority_principal_for_agent_id(&parent.agent_id),
        recipient: AgentId::from_uuid(child_agent_id),
        scope: parent_capability.scope,
        expires_at: inherited_expiry(parent_capability.expires_at, parent.exp),
        parent_capability: Some(parent_capability.capability_id),
        revocation_state: RevocationState::Active,
    })
}

fn derive_child_provenance(
    parent: &AgentClaims,
    child_capability: Option<&DelegationCapability>,
) -> Option<ProvenanceChain> {
    let child_capability = child_capability?;
    let mut provenance = parent.provenance_chain.clone()?;
    provenance.links.push(ProvenanceLink {
        issuer: authority_principal_for_agent_id(&parent.agent_id),
        recipient: child_capability.recipient,
        capability_id: child_capability.capability_id,
        scope: child_capability.scope,
        expires_at: child_capability.expires_at,
    });
    provenance.terminal_capability = child_capability.capability_id;
    Some(provenance)
}

fn inherited_expiry(capability_expiry: DateTime<Utc>, token_expiry: u64) -> DateTime<Utc> {
    match DateTime::<Utc>::from_timestamp(token_expiry as i64, 0) {
        Some(token_expiry) => capability_expiry.min(token_expiry),
        None => capability_expiry,
    }
}
