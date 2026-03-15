//! NATS auth callout support for dynamic per-connection credential scoping.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_nats::Client;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use futures::StreamExt;
use nats_jwt::Token;
use nkeys::{KeyPair, KeyPairType};
use parking_lot::RwLock;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::{debug, warn};
use uuid::Uuid;

use chrono::Utc;
use mister_smith_core::{DelegationCapability, ProvenanceChain, SecurityError};
use mister_smith_transport::SubjectTaxonomy;

use crate::delegation::DelegationService;
use crate::jwt::JwtManager;

/// System subject used by the NATS server auth callout protocol.
pub const AUTH_CALLOUT_SUBJECT: &str = "$SYS.REQ.USER.AUTH";
const AUTH_CALLOUT_QUEUE_GROUP: &str = "mister-smith.auth-callout";

const AUTH_REQUEST_TYPE: &str = "authorization_request";
const AUTH_RESPONSE_TYPE: &str = "authorization_response";
const AUTH_RESPONSE_VERSION: i32 = 2;
const DEFAULT_VIOLATION_PENALTY: f64 = 0.1;

/// Trust-to-permission tier used for dynamic NATS user JWT generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionTier {
    /// Full operational access for highly trusted agents.
    Full,
    /// Standard operational access for normally trusted agents.
    Standard,
    /// Reduced access for partially trusted agents.
    Restricted,
    /// Minimal access for unknown or untrusted agents.
    Quarantined,
}

impl PermissionTier {
    /// Map a trust score into a permission tier.
    #[must_use]
    pub fn from_trust_score(trust_score: f64) -> Self {
        if trust_score >= 0.9 {
            Self::Full
        } else if trust_score >= 0.5 {
            Self::Standard
        } else if trust_score >= 0.2 {
            Self::Restricted
        } else {
            Self::Quarantined
        }
    }

    fn jwt_ttl_secs(self, max_jwt_ttl_secs: u64) -> u64 {
        let tier_ttl = match self {
            Self::Full => 300,
            Self::Standard => 120,
            Self::Restricted => 60,
            Self::Quarantined => 30,
        };
        tier_ttl.min(max_jwt_ttl_secs)
    }

    fn permissions_for(self, agent_id: &str) -> Permissions {
        let inbox = "_INBOX.>".to_string();
        let health = SubjectTaxonomy::system_health();
        let deny = vec!["$SYS.>".to_string(), "$JS.>".to_string()];

        match self {
            Self::Full => Permissions {
                publish_allow: vec![
                    SubjectTaxonomy::all_agents(),
                    "tasks.>".to_string(),
                    SubjectTaxonomy::all_workflows(),
                    SubjectTaxonomy::all_system(),
                ],
                publish_deny: deny.clone(),
                subscribe_allow: vec![
                    SubjectTaxonomy::all_agents(),
                    "tasks.>".to_string(),
                    SubjectTaxonomy::all_workflows(),
                    SubjectTaxonomy::all_system(),
                    inbox,
                ],
                subscribe_deny: deny,
            },
            Self::Standard => Permissions {
                publish_allow: vec![
                    SubjectTaxonomy::all_agents(),
                    "tasks.>".to_string(),
                    SubjectTaxonomy::all_workflows(),
                    health.clone(),
                ],
                publish_deny: deny.clone(),
                subscribe_allow: vec![
                    SubjectTaxonomy::all_agents(),
                    "tasks.>".to_string(),
                    SubjectTaxonomy::all_workflows(),
                    health,
                    inbox,
                ],
                subscribe_deny: deny,
            },
            Self::Restricted => Permissions {
                publish_allow: vec![format!("agents.{agent_id}.>"), health.clone()],
                publish_deny: deny.clone(),
                subscribe_allow: vec![format!("agents.{agent_id}.>"), health, inbox],
                subscribe_deny: deny,
            },
            Self::Quarantined => Permissions::quarantined(),
        }
    }
}

/// Publish/subscribe subject ACLs used to issue scoped NATS user JWTs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    /// Subjects the agent may publish to.
    pub publish_allow: Vec<String>,
    /// Subjects the agent may not publish to.
    pub publish_deny: Vec<String>,
    /// Subjects the agent may subscribe to.
    pub subscribe_allow: Vec<String>,
    /// Subjects the agent may not subscribe to.
    pub subscribe_deny: Vec<String>,
}

impl Permissions {
    /// Minimal health-only permissions used for quarantined fallback.
    #[must_use]
    pub fn quarantined() -> Self {
        let health = SubjectTaxonomy::system_health();
        Self {
            publish_allow: vec![health.clone()],
            publish_deny: vec!["$SYS.>".to_string(), "$JS.>".to_string()],
            subscribe_allow: vec![health, "_INBOX.>".to_string()],
            subscribe_deny: vec!["$SYS.>".to_string(), "$JS.>".to_string()],
        }
    }

    pub(crate) fn apply_to(self, mut token: Token<nats_jwt::User>) -> Token<nats_jwt::User> {
        for subject in self.publish_allow {
            token = token.allow_publish(subject);
        }
        for subject in self.publish_deny {
            token = token.deny_publish(subject);
        }
        for subject in self.subscribe_allow {
            token = token.allow_subscribe(subject);
        }
        for subject in self.subscribe_deny {
            token = token.deny_subscribe(subject);
        }
        token
    }
}

/// Trust state for an agent participating in auth callout authorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustProfile {
    /// Agent identifier used as the trust store key.
    pub agent_id: String,
    /// Current trust score clamped to the `[0.0, 1.0]` interval.
    pub trust_score: f64,
    /// Tier derived from the current trust score.
    pub permission_tier: PermissionTier,
    /// Number of recorded trust-degrading violations.
    pub violation_count: u32,
    /// Last assessment time in epoch milliseconds.
    pub last_assessment: u64,
}

impl TrustProfile {
    /// Create a new trust profile from an agent identifier and trust score.
    #[must_use]
    pub fn new(agent_id: impl Into<String>, trust_score: f64) -> Self {
        let agent_id = agent_id.into();
        let trust_score = trust_score.clamp(0.0, 1.0);
        Self {
            agent_id,
            trust_score,
            permission_tier: PermissionTier::from_trust_score(trust_score),
            violation_count: 0,
            last_assessment: unix_timestamp_millis(),
        }
    }

    fn record_violation(&mut self) {
        self.trust_score = (self.trust_score - DEFAULT_VIOLATION_PENALTY).clamp(0.0, 1.0);
        self.permission_tier = PermissionTier::from_trust_score(self.trust_score);
        self.violation_count = self.violation_count.saturating_add(1);
        self.last_assessment = unix_timestamp_millis();
    }

    fn normalized(mut self) -> Self {
        self.trust_score = self.trust_score.clamp(0.0, 1.0);
        self.permission_tier = PermissionTier::from_trust_score(self.trust_score);
        self.last_assessment = unix_timestamp_millis();
        self
    }
}

/// Resolved authorization outcome for a single agent connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizationResult {
    /// Agent identifier resolved from the auth callout request.
    pub agent_id: String,
    /// Tier selected from the current trust profile or fallback path.
    pub permission_tier: PermissionTier,
    /// Concrete NATS subject permissions encoded into the issued user JWT.
    pub permissions: Permissions,
    /// Effective JWT lifetime in seconds.
    pub jwt_ttl_secs: u64,
    /// Whether the fallback path was used instead of a stored trust profile.
    pub fallback_applied: bool,
    /// Valid delegated capability carried by the authenticated principal, when any.
    pub delegation_capability: Option<DelegationCapability>,
    /// Reconstructable authority lineage for the capability, when any.
    pub provenance_chain: Option<ProvenanceChain>,
}

/// Stateful NATS auth callout handler backed by a trust store and account signer.
#[derive(Clone)]
pub struct AuthCalloutHandler {
    trust_store: Arc<RwLock<HashMap<String, TrustProfile>>>,
    signing_key: KeyPair,
    issuer_account: String,
    default_permissions: Permissions,
    max_jwt_ttl_secs: u64,
    jwt_manager: Option<Arc<JwtManager>>,
    delegation_service: Option<Arc<DelegationService>>,
}

impl AuthCalloutHandler {
    /// Create a new auth callout handler.
    ///
    /// `issuer_account` must be the account public NKey that will own issued
    /// user JWTs. `signing_key` may be that account key or an account signing
    /// key whose parent account is `issuer_account`.
    pub fn new(signing_key: KeyPair, issuer_account: impl Into<String>) -> Self {
        Self {
            trust_store: Arc::new(RwLock::new(HashMap::new())),
            signing_key,
            issuer_account: issuer_account.into(),
            default_permissions: Permissions::quarantined(),
            max_jwt_ttl_secs: 300,
            jwt_manager: None,
            delegation_service: None,
        }
    }

    /// Override the minimal fallback permissions used for quarantined access.
    #[must_use]
    pub fn with_default_permissions(mut self, default_permissions: Permissions) -> Self {
        self.default_permissions = default_permissions;
        self
    }

    /// Override the maximum JWT TTL allowed for any issued user token.
    #[must_use]
    pub fn with_max_jwt_ttl_secs(mut self, max_jwt_ttl_secs: u64) -> Self {
        self.max_jwt_ttl_secs = max_jwt_ttl_secs;
        self
    }

    /// Configure bearer-token validation for logical agent identities.
    #[must_use]
    pub fn with_jwt_manager(mut self, jwt_manager: Arc<JwtManager>) -> Self {
        self.jwt_manager = Some(jwt_manager);
        self
    }

    /// Configure the bounded delegation service used for bearer-token validation.
    #[must_use]
    pub fn with_delegation_service(mut self, delegation_service: Arc<DelegationService>) -> Self {
        self.delegation_service = Some(delegation_service);
        self
    }

    /// Return the issuer account public NKey used in issued tokens.
    #[must_use]
    pub fn issuer_account(&self) -> &str {
        &self.issuer_account
    }

    /// Start handling NATS auth callout requests on `$SYS.REQ.USER.AUTH`.
    ///
    /// This spawns a background task and returns once the subscription has been
    /// established.
    pub async fn start(&self, nats_client: &Client) -> Result<(), SecurityError> {
        let mut subscriber = nats_client
            .queue_subscribe(
                AUTH_CALLOUT_SUBJECT.to_string(),
                AUTH_CALLOUT_QUEUE_GROUP.to_string(),
            )
            .await
            .map_err(|error| {
                SecurityError::AuthenticationFailed(format!(
                    "failed to queue subscribe to {AUTH_CALLOUT_SUBJECT}: {error}"
                ))
            })?;

        let handler = self.clone();
        let client = nats_client.clone();

        tokio::spawn(async move {
            while let Some(message) = subscriber.next().await {
                let Some(reply_subject) = message.reply.clone() else {
                    warn!(subject = %message.subject, "auth callout request missing reply subject");
                    continue;
                };

                let request_jwt = match std::str::from_utf8(&message.payload) {
                    Ok(jwt) => jwt,
                    Err(error) => {
                        warn!(%error, "ignoring auth callout payload that is not valid UTF-8");
                        continue;
                    }
                };

                let response_jwt = match handler.handle_auth_request(request_jwt) {
                    Ok(response_jwt) => response_jwt,
                    Err(error) => {
                        warn!(%error, "failed to handle auth callout request");
                        continue;
                    }
                };

                if let Err(error) = client
                    .publish(reply_subject, response_jwt.into_bytes().into())
                    .await
                {
                    warn!(%error, "failed to publish auth callout response");
                }
            }
        });

        Ok(())
    }

    /// Resolve the effective trust tier and permissions for an agent.
    ///
    /// Missing trust store entries always fall back to quarantined permissions.
    pub fn authorize(&self, agent_id: &str) -> Result<AuthorizationResult, SecurityError> {
        self.authorize_authenticated(&AuthenticatedRequest {
            agent_id: agent_id.to_string(),
            claims: None,
        })
    }

    fn authorize_authenticated(
        &self,
        authenticated: &AuthenticatedRequest,
    ) -> Result<AuthorizationResult, SecurityError> {
        let agent_id = authenticated.agent_id.trim();
        if agent_id.is_empty() {
            return Err(SecurityError::AuthenticationFailed(
                "agent_id must not be empty".to_string(),
            ));
        }

        let profile = self.trust_store.read().get(agent_id).cloned();
        let result = match profile {
            Some(profile) => {
                let permission_tier = PermissionTier::from_trust_score(profile.trust_score);
                AuthorizationResult {
                    agent_id: agent_id.to_string(),
                    permission_tier,
                    permissions: permission_tier.permissions_for(agent_id),
                    jwt_ttl_secs: constrained_jwt_ttl_secs(
                        permission_tier.jwt_ttl_secs(self.max_jwt_ttl_secs),
                        authenticated
                            .claims
                            .as_ref()
                            .and_then(|claims| claims.delegation_capability.as_ref()),
                    ),
                    fallback_applied: false,
                    delegation_capability: authenticated
                        .claims
                        .as_ref()
                        .and_then(|claims| claims.delegation_capability.clone()),
                    provenance_chain: authenticated
                        .claims
                        .as_ref()
                        .and_then(|claims| claims.provenance_chain.clone()),
                }
            }
            None => AuthorizationResult {
                agent_id: agent_id.to_string(),
                permission_tier: PermissionTier::Quarantined,
                permissions: self.default_permissions.clone(),
                jwt_ttl_secs: constrained_jwt_ttl_secs(
                    PermissionTier::Quarantined.jwt_ttl_secs(self.max_jwt_ttl_secs),
                    authenticated
                        .claims
                        .as_ref()
                        .and_then(|claims| claims.delegation_capability.as_ref()),
                ),
                fallback_applied: true,
                delegation_capability: authenticated
                    .claims
                    .as_ref()
                    .and_then(|claims| claims.delegation_capability.clone()),
                provenance_chain: authenticated
                    .claims
                    .as_ref()
                    .and_then(|claims| claims.provenance_chain.clone()),
            },
        };

        debug!(
            agent_id = %result.agent_id,
            tier = ?result.permission_tier,
            fallback = result.fallback_applied,
            "resolved auth callout authorization"
        );

        Ok(result)
    }

    /// Update or insert the trust profile for an agent.
    pub fn update_trust(&self, agent_id: &str, profile: TrustProfile) {
        let mut profile = profile.normalized();
        profile.agent_id = agent_id.to_string();
        self.trust_store
            .write()
            .insert(agent_id.to_string(), profile);
    }

    /// Record a trust-degrading violation for an agent.
    pub fn record_violation(&self, agent_id: &str) {
        let mut guard = self.trust_store.write();
        let entry = guard
            .entry(agent_id.to_string())
            .or_insert_with(|| TrustProfile::new(agent_id.to_string(), 0.0));
        entry.record_violation();
    }

    /// Parse a raw auth callout request JWT and return the signed response JWT.
    pub fn handle_auth_request(&self, request_jwt: &str) -> Result<String, SecurityError> {
        let request = decode_jwt_payload::<AuthorizationRequestClaims>(request_jwt)?;

        if request.nats.claim_type.as_deref() != Some(AUTH_REQUEST_TYPE) {
            return self.build_error_response(&request, "unsupported auth callout request type");
        }

        let authenticated = match self.authenticate_request(&request) {
            Ok(authenticated) => authenticated,
            Err(error) => return self.build_error_response(&request, &error.to_string()),
        };
        let authorization = self.authorize_authenticated(&authenticated)?;

        let response = match self.generate_user_jwt(&request.nats.user_nkey, &authorization) {
            Ok(user_jwt) => self.build_success_response(&request, &user_jwt),
            Err(error) => self.build_error_response(&request, &error.to_string()),
        }?;

        Ok(response)
    }

    fn authenticate_request(
        &self,
        request: &AuthorizationRequestClaims,
    ) -> Result<AuthenticatedRequest, SecurityError> {
        if let Some(auth_token) = request.nats.connect_opts.auth_token.as_deref() {
            let claims = self.authenticate_bearer_token(auth_token)?;
            if let Some(claimed_agent_id) = request.claimed_agent_id() {
                if claimed_agent_id != claims.agent_id {
                    return Err(SecurityError::AuthenticationFailed(format!(
                        "bearer token agent_id '{}' does not match claimed identity '{claimed_agent_id}'",
                        claims.agent_id
                    )));
                }
            }
            return Ok(AuthenticatedRequest {
                agent_id: claims.agent_id.clone(),
                claims: Some(claims),
            });
        }

        let nkey = request.nats.connect_opts.nkey.as_deref().ok_or_else(|| {
            SecurityError::AuthenticationFailed(
                "auth callout request missing verifiable credentials".to_string(),
            )
        })?;
        let signature = request
            .nats
            .connect_opts
            .signature
            .as_deref()
            .ok_or_else(|| {
                SecurityError::AuthenticationFailed(
                    "auth callout request missing verifiable credentials".to_string(),
                )
            })?;
        let nonce = request.nats.client_info.nonce.as_deref().ok_or_else(|| {
            SecurityError::AuthenticationFailed(
                "auth callout request missing verifiable credentials".to_string(),
            )
        })?;

        self.authenticate_nkey(nkey, signature, nonce)?;

        if let Some(claimed_agent_id) = request.claimed_agent_id() {
            if claimed_agent_id != nkey {
                return Err(SecurityError::AuthenticationFailed(format!(
                    "verified nkey '{nkey}' does not match claimed identity '{claimed_agent_id}'"
                )));
            }
        }

        Ok(AuthenticatedRequest {
            agent_id: nkey.to_string(),
            claims: None,
        })
    }

    fn authenticate_bearer_token(
        &self,
        auth_token: &str,
    ) -> Result<crate::jwt::AgentClaims, SecurityError> {
        let jwt_manager = self.jwt_manager.as_ref().ok_or_else(|| {
            SecurityError::AuthenticationFailed(
                "auth callout bearer token validation requires a configured JwtManager".to_string(),
            )
        })?;

        let claims = jwt_manager.validate_token(auth_token)?;
        if let Some(delegation_service) = &self.delegation_service {
            delegation_service
                .validate_claims(&claims, None)
                .map_err(|error| SecurityError::AuthenticationFailed(error.to_string()))?;
        }
        Ok(claims)
    }

    fn authenticate_nkey(
        &self,
        nkey: &str,
        signature: &str,
        nonce: &str,
    ) -> Result<(), SecurityError> {
        let key_pair = KeyPair::from_public_key(nkey).map_err(|error| {
            SecurityError::AuthenticationFailed(format!(
                "invalid auth callout nkey '{nkey}': {error}"
            ))
        })?;

        if key_pair.key_pair_type() != KeyPairType::User {
            return Err(SecurityError::AuthenticationFailed(format!(
                "auth callout nkey must be a user public key: {nkey}"
            )));
        }

        let signature = URL_SAFE_NO_PAD.decode(signature).map_err(|error| {
            SecurityError::AuthenticationFailed(format!(
                "failed to decode auth callout signature: {error}"
            ))
        })?;

        key_pair
            .verify(nonce.as_bytes(), &signature)
            .map_err(|error| {
                SecurityError::AuthenticationFailed(format!(
                    "invalid auth callout nkey signature: {error}"
                ))
            })?;

        Ok(())
    }

    fn generate_user_jwt(
        &self,
        user_nkey: &str,
        authorization: &AuthorizationResult,
    ) -> Result<String, SecurityError> {
        let user_key = KeyPair::from_public_key(user_nkey).map_err(|error| {
            SecurityError::AuthenticationFailed(format!("invalid user nkey '{user_nkey}': {error}"))
        })?;
        if user_key.key_pair_type() != KeyPairType::User {
            return Err(SecurityError::AuthenticationFailed(format!(
                "auth callout user_nkey must be a user public key: {user_nkey}"
            )));
        }

        let expires = unix_timestamp_secs()
            .checked_add(authorization.jwt_ttl_secs as i64)
            .ok_or_else(|| {
                SecurityError::TokenGenerationFailed("user jwt expiration overflow".to_string())
            })?;

        let token = authorization.permissions.clone().apply_to(
            Token::new_user(self.issuer_account.clone(), user_nkey.to_string())
                .name(authorization.agent_id.clone())
                .expires(expires),
        );

        let jwt = token.sign(&self.signing_key);
        Ok(jwt)
    }

    fn build_success_response(
        &self,
        request: &AuthorizationRequestClaims,
        user_jwt: &str,
    ) -> Result<String, SecurityError> {
        let now = unix_timestamp_secs();
        let exp = now
            .checked_add(self.max_jwt_ttl_secs as i64)
            .ok_or_else(|| {
                SecurityError::TokenGenerationFailed(
                    "authorization response expiration overflow".to_string(),
                )
            })?;

        let claims = AuthorizationResponseClaims {
            aud: request.nats.server_id.id.clone(),
            exp: Some(exp),
            iat: now,
            iss: self.signing_key.public_key(),
            jti: Uuid::new_v4().to_string(),
            name: "Mister Smith Auth Callout".to_string(),
            sub: request.nats.user_nkey.clone(),
            nats: AuthorizationResponseNatsClaims {
                jwt: Some(user_jwt.to_string()),
                error: None,
                issuer_account: self.issuer_account.clone(),
                claim_type: AUTH_RESPONSE_TYPE.to_string(),
                version: AUTH_RESPONSE_VERSION,
            },
        };

        encode_nkey_jwt(&claims, &self.signing_key)
    }

    fn build_error_response(
        &self,
        request: &AuthorizationRequestClaims,
        error: &str,
    ) -> Result<String, SecurityError> {
        let now = unix_timestamp_secs();
        let exp = now
            .checked_add(PermissionTier::Quarantined.jwt_ttl_secs(self.max_jwt_ttl_secs) as i64)
            .ok_or_else(|| {
                SecurityError::TokenGenerationFailed(
                    "authorization error response expiration overflow".to_string(),
                )
            })?;

        let claims = AuthorizationResponseClaims {
            aud: request.nats.server_id.id.clone(),
            exp: Some(exp),
            iat: now,
            iss: self.signing_key.public_key(),
            jti: Uuid::new_v4().to_string(),
            name: "Mister Smith Auth Callout Error".to_string(),
            sub: request.nats.user_nkey.clone(),
            nats: AuthorizationResponseNatsClaims {
                jwt: None,
                error: Some(error.to_string()),
                issuer_account: self.issuer_account.clone(),
                claim_type: AUTH_RESPONSE_TYPE.to_string(),
                version: AUTH_RESPONSE_VERSION,
            },
        };

        encode_nkey_jwt(&claims, &self.signing_key)
    }
}

#[derive(Debug, Deserialize)]
struct AuthorizationRequestClaims {
    nats: AuthorizationRequestNatsClaims,
}

#[derive(Debug, Clone)]
struct AuthenticatedRequest {
    agent_id: String,
    claims: Option<crate::jwt::AgentClaims>,
}

impl AuthorizationRequestClaims {
    fn claimed_agent_id(&self) -> Option<&str> {
        [
            self.nats.client_info.user.as_deref(),
            self.nats.connect_opts.user.as_deref(),
            self.nats.client_info.name.as_deref(),
            self.nats.connect_opts.name.as_deref(),
        ]
        .into_iter()
        .flatten()
        .find(|candidate| !candidate.trim().is_empty())
    }
}

#[derive(Debug, Deserialize)]
struct AuthorizationRequestNatsClaims {
    server_id: AuthorizationServerId,
    user_nkey: String,
    client_info: AuthorizationClientInfo,
    connect_opts: AuthorizationConnectOptions,
    #[serde(default, rename = "type")]
    claim_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AuthorizationServerId {
    id: String,
}

#[derive(Debug, Default, Deserialize)]
struct AuthorizationClientInfo {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    nonce: Option<String>,
    #[serde(default)]
    user: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct AuthorizationConnectOptions {
    #[serde(default)]
    auth_token: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    nkey: Option<String>,
    #[serde(default, rename = "sig")]
    signature: Option<String>,
    #[serde(default)]
    user: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuthorizationResponseClaims {
    aud: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    exp: Option<i64>,
    iat: i64,
    iss: String,
    jti: String,
    name: String,
    sub: String,
    nats: AuthorizationResponseNatsClaims,
}

#[derive(Debug, Serialize)]
struct AuthorizationResponseNatsClaims {
    #[serde(skip_serializing_if = "Option::is_none")]
    jwt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    issuer_account: String,
    #[serde(rename = "type")]
    claim_type: String,
    version: i32,
}

fn constrained_jwt_ttl_secs(
    default_ttl_secs: u64,
    capability: Option<&DelegationCapability>,
) -> u64 {
    let Some(capability) = capability else {
        return default_ttl_secs;
    };

    let expires_in = capability
        .expires_at
        .signed_duration_since(Utc::now())
        .num_seconds()
        .max(0) as u64;

    default_ttl_secs.min(expires_in)
}

fn decode_jwt_payload<T>(jwt: &str) -> Result<T, SecurityError>
where
    T: DeserializeOwned,
{
    let Some(payload) = jwt.split('.').nth(1) else {
        return Err(SecurityError::InvalidToken(
            "auth callout request jwt must contain three segments".to_string(),
        ));
    };

    let decoded = URL_SAFE_NO_PAD.decode(payload).map_err(|error| {
        SecurityError::InvalidToken(format!("failed to decode auth callout payload: {error}"))
    })?;

    serde_json::from_slice(&decoded).map_err(|error| {
        SecurityError::InvalidToken(format!(
            "failed to parse auth callout payload json: {error}"
        ))
    })
}

fn encode_nkey_jwt<T>(claims: &T, signing_key: &KeyPair) -> Result<String, SecurityError>
where
    T: Serialize,
{
    let header = URL_SAFE_NO_PAD.encode(br#"{"typ":"JWT","alg":"ed25519-nkey"}"#);
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(claims).map_err(|error| {
        SecurityError::TokenGenerationFailed(format!(
            "failed to serialize auth callout claims: {error}"
        ))
    })?);

    let signing_input = format!("{header}.{payload}");
    let signature = signing_key
        .sign(signing_input.as_bytes())
        .map_err(|error| {
            SecurityError::TokenGenerationFailed(format!(
                "failed to sign auth callout response: {error}"
            ))
        })?;
    let signature = URL_SAFE_NO_PAD.encode(signature);

    Ok(format!("{signing_input}.{signature}"))
}

fn unix_timestamp_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn unix_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
