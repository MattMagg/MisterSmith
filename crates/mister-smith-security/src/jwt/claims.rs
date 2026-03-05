//! JWT claim types for agent authentication.

use serde::{Deserialize, Serialize};

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
