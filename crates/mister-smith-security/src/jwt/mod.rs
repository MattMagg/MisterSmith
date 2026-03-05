//! JWT authentication: token generation, validation, refresh, and revocation.

mod claims;
mod keys;

pub use claims::{AgentClaims, TokenPair};
pub use crate::config::KeySource;

use crate::config::JwtConfig;
use dashmap::DashMap;
use mister_smith_core::SecurityError;
use std::time::Instant;
use tracing::{debug, warn};

/// Manages JWT token lifecycle: generation, validation, refresh, and revocation.
///
/// Thread-safe (`Send + Sync`) — the revocation list uses [`DashMap`] for
/// concurrent access without locking.
pub struct JwtManager {
    encoding_key: jsonwebtoken::EncodingKey,
    decoding_key: jsonwebtoken::DecodingKey,
    algorithm: jsonwebtoken::Algorithm,
    validation: jsonwebtoken::Validation,
    access_ttl: std::time::Duration,
    refresh_ttl: std::time::Duration,
    issuer: Option<String>,
    audience: Vec<String>,
    /// Revoked token JTIs mapped to the time they were revoked.
    revoked: DashMap<String, Instant>,
}

impl JwtManager {
    /// Create a new `JwtManager` from the given configuration.
    ///
    /// # Errors
    ///
    /// Returns `SecurityError::KeyLoadFailed` if the key source is invalid.
    pub fn new(config: &JwtConfig) -> Result<Self, SecurityError> {
        let algorithm = parse_algorithm(&config.algorithm)?;
        let encoding_key = keys::load_encoding_key(&config.key_source, algorithm)?;
        let decoding_key = keys::load_decoding_key(&config.key_source, algorithm)?;

        let mut validation = jsonwebtoken::Validation::new(algorithm);
        validation.leeway = 5; // 5 second clock skew tolerance
        validation.validate_exp = true;
        validation.validate_nbf = true;

        if !config.audience.is_empty() {
            validation.set_audience(&config.audience);
        } else {
            validation.validate_aud = false;
        }

        if let Some(ref iss) = config.issuer {
            validation.set_issuer(&[iss]);
        }

        debug!(algorithm = ?algorithm, "JwtManager initialized");

        Ok(Self {
            encoding_key,
            decoding_key,
            algorithm,
            validation,
            access_ttl: config.access_token_ttl,
            refresh_ttl: config.refresh_token_ttl,
            issuer: config.issuer.clone(),
            audience: config.audience.clone(),
            revoked: DashMap::new(),
        })
    }

    /// Generate an access + refresh token pair for the given claims.
    pub fn generate_token_pair(&self, claims: &AgentClaims) -> Result<TokenPair, SecurityError> {
        let now = chrono::Utc::now().timestamp() as u64;

        // Access token
        let mut access_claims = claims.clone();
        access_claims.iat = now;
        access_claims.exp = now + self.access_ttl.as_secs();
        if access_claims.jti.is_empty() {
            access_claims.jti = uuid::Uuid::new_v4().to_string();
        }
        if self.issuer.is_some() && access_claims.iss.is_none() {
            access_claims.iss.clone_from(&self.issuer);
        }
        if access_claims.aud.is_empty() && !self.audience.is_empty() {
            access_claims.aud.clone_from(&self.audience);
        }

        let header = jsonwebtoken::Header::new(self.algorithm);
        let access_token = jsonwebtoken::encode(&header, &access_claims, &self.encoding_key)
            .map_err(|e| SecurityError::TokenGenerationFailed(e.to_string()))?;

        // Refresh token — longer TTL, new JTI
        let mut refresh_claims = access_claims.clone();
        refresh_claims.exp = now + self.refresh_ttl.as_secs();
        refresh_claims.jti = uuid::Uuid::new_v4().to_string();

        let refresh_token = jsonwebtoken::encode(&header, &refresh_claims, &self.encoding_key)
            .map_err(|e| SecurityError::TokenGenerationFailed(e.to_string()))?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_ttl.as_secs(),
        })
    }

    /// Validate a token and extract the agent claims.
    ///
    /// Checks signature, expiration, audience, issuer, and revocation status.
    pub fn validate_token(&self, token: &str) -> Result<AgentClaims, SecurityError> {
        let token_data =
            jsonwebtoken::decode::<AgentClaims>(token, &self.decoding_key, &self.validation)
                .map_err(crate::error::from_jwt_error)?;

        // Check revocation
        if self.is_revoked(&token_data.claims.jti) {
            warn!(jti = %token_data.claims.jti, "revoked token used");
            return Err(SecurityError::TokenRevoked);
        }

        Ok(token_data.claims)
    }

    /// Refresh an access token using a valid refresh token.
    ///
    /// Validates the refresh token, then generates a new token pair with the
    /// same claims but fresh expiration.
    pub fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair, SecurityError> {
        let claims = self.validate_token(refresh_token)?;
        self.generate_token_pair(&claims)
    }

    /// Revoke a token by its JTI (JWT ID).
    pub fn revoke_token(&self, jti: &str) {
        debug!(jti = %jti, "token revoked");
        self.revoked.insert(jti.to_string(), Instant::now());
    }

    /// Check if a token has been revoked.
    pub fn is_revoked(&self, jti: &str) -> bool {
        self.revoked.contains_key(jti)
    }

    /// Remove expired entries from the revocation list.
    ///
    /// Entries older than the refresh token TTL are cleaned up since those
    /// tokens would have expired naturally.
    pub fn cleanup_revoked(&self) {
        let cutoff = self.refresh_ttl;
        let now = Instant::now();
        self.revoked
            .retain(|_, revoked_at| now.duration_since(*revoked_at) < cutoff);
    }
}

/// Parse an algorithm string like "RS256" into a jsonwebtoken Algorithm.
fn parse_algorithm(s: &str) -> Result<jsonwebtoken::Algorithm, SecurityError> {
    match s {
        "HS256" => Ok(jsonwebtoken::Algorithm::HS256),
        "HS384" => Ok(jsonwebtoken::Algorithm::HS384),
        "HS512" => Ok(jsonwebtoken::Algorithm::HS512),
        "RS256" => Ok(jsonwebtoken::Algorithm::RS256),
        "RS384" => Ok(jsonwebtoken::Algorithm::RS384),
        "RS512" => Ok(jsonwebtoken::Algorithm::RS512),
        "PS256" => Ok(jsonwebtoken::Algorithm::PS256),
        "PS384" => Ok(jsonwebtoken::Algorithm::PS384),
        "PS512" => Ok(jsonwebtoken::Algorithm::PS512),
        "ES256" => Ok(jsonwebtoken::Algorithm::ES256),
        "ES384" => Ok(jsonwebtoken::Algorithm::ES384),
        "EdDSA" => Ok(jsonwebtoken::Algorithm::EdDSA),
        _ => Err(SecurityError::InvalidToken(format!(
            "unsupported algorithm: {s}"
        ))),
    }
}
