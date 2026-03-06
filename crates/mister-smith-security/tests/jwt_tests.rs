//! Integration tests for the JWT authentication subsystem.

use mister_smith_security::config::{JwtConfig, KeySource};
use mister_smith_security::jwt::{AgentClaims, JwtManager};
use std::time::Duration;

fn hmac_config() -> JwtConfig {
    JwtConfig {
        algorithm: "HS256".to_string(),
        access_token_ttl: Duration::from_secs(300),
        refresh_token_ttl: Duration::from_secs(3600),
        issuer: Some("mister-smith-test".to_string()),
        audience: vec!["test-audience".to_string()],
        key_source: KeySource::Hmac {
            secret: b"test-secret-key-for-hmac-256-min-32-bytes!".to_vec(),
        },
    }
}

fn test_claims() -> AgentClaims {
    AgentClaims {
        sub: "agent-001".to_string(),
        agent_id: "agent-001".to_string(),
        agent_type: "worker".to_string(),
        capabilities: vec!["compute".to_string()],
        permissions: vec!["read:task:own".to_string()],
        ..Default::default()
    }
}

// -- Generate and validate roundtrip (US1-AS1, AS2) -----------------------

#[test]
fn generate_and_validate_roundtrip() {
    let mgr = JwtManager::new(&hmac_config()).unwrap();
    let claims = test_claims();
    let pair = mgr.generate_token_pair(&claims).unwrap();

    assert_eq!(pair.token_type, "Bearer");
    assert_eq!(pair.expires_in, 300);
    assert!(!pair.access_token.is_empty());
    assert!(!pair.refresh_token.is_empty());

    // Validate access token
    let validated = mgr.validate_token(&pair.access_token).unwrap();
    assert_eq!(validated.sub, "agent-001");
    assert_eq!(validated.token_use, "access");
    assert_eq!(validated.agent_id, "agent-001");
    assert_eq!(validated.agent_type, "worker");
    assert_eq!(validated.capabilities, vec!["compute"]);
    assert_eq!(validated.permissions, vec!["read:task:own"]);
}

#[test]
fn claims_populated_correctly() {
    let mgr = JwtManager::new(&hmac_config()).unwrap();
    let claims = test_claims();
    let pair = mgr.generate_token_pair(&claims).unwrap();
    let validated = mgr.validate_token(&pair.access_token).unwrap();

    assert_eq!(validated.iss.as_deref(), Some("mister-smith-test"));
    assert_eq!(validated.aud, vec!["test-audience"]);
    assert!(!validated.jti.is_empty());
    assert!(validated.iat > 0);
    assert!(validated.exp > validated.iat);
}

// -- Expired token rejection (US1-AS3) ------------------------------------

#[test]
fn expired_token_rejected() {
    let config = JwtConfig {
        access_token_ttl: Duration::from_secs(0), // immediate expiry
        ..hmac_config()
    };
    let mgr = JwtManager::new(&config).unwrap();
    let claims = test_claims();
    let pair = mgr.generate_token_pair(&claims).unwrap();

    // Token should have expired immediately (or within leeway)
    std::thread::sleep(Duration::from_secs(6)); // > 5s leeway
    let result = mgr.validate_token(&pair.access_token);
    assert!(result.is_err());
}

// -- Wrong key rejection (US1-AS4) ----------------------------------------

#[test]
fn wrong_key_rejects() {
    let mgr1 = JwtManager::new(&hmac_config()).unwrap();
    let mgr2 = JwtManager::new(&JwtConfig {
        key_source: KeySource::Hmac {
            secret: b"completely-different-key-at-least-32-bytes!!".to_vec(),
        },
        ..hmac_config()
    })
    .unwrap();

    let pair = mgr1.generate_token_pair(&test_claims()).unwrap();
    let result = mgr2.validate_token(&pair.access_token);
    assert!(result.is_err());
}

// -- Token refresh (US1-AS5) ----------------------------------------------

#[test]
fn token_refresh() {
    let mgr = JwtManager::new(&hmac_config()).unwrap();
    let pair = mgr.generate_token_pair(&test_claims()).unwrap();

    let new_pair = mgr.refresh_token(&pair.refresh_token).unwrap();
    assert_ne!(new_pair.access_token, pair.access_token);
    assert_ne!(new_pair.refresh_token, pair.refresh_token);

    // New tokens should validate
    let claims = mgr.validate_token(&new_pair.access_token).unwrap();
    assert_eq!(claims.agent_id, "agent-001");
    assert_eq!(claims.token_use, "access");

    let refresh_claims = mgr.validate_token(&new_pair.refresh_token).unwrap();
    assert_eq!(refresh_claims.token_use, "refresh");
}

#[test]
fn access_token_cannot_refresh() {
    let mgr = JwtManager::new(&hmac_config()).unwrap();
    let pair = mgr.generate_token_pair(&test_claims()).unwrap();

    let result = mgr.refresh_token(&pair.access_token);
    assert!(matches!(
        result,
        Err(mister_smith_core::SecurityError::InvalidToken(message))
            if message == "token_use must be 'refresh' for refresh flow"
    ));
}

// -- Token revocation (US1-AS6) -------------------------------------------

#[test]
fn token_revocation() {
    let mgr = JwtManager::new(&hmac_config()).unwrap();
    let pair = mgr.generate_token_pair(&test_claims()).unwrap();
    let claims = mgr.validate_token(&pair.access_token).unwrap();

    assert!(!mgr.is_revoked(&claims.jti));
    mgr.revoke_token(&claims.jti);
    assert!(mgr.is_revoked(&claims.jti));

    let result = mgr.validate_token(&pair.access_token);
    assert!(result.is_err());
}

#[test]
fn revocation_cleanup() {
    let config = JwtConfig {
        refresh_token_ttl: Duration::from_millis(100), // very short for test
        ..hmac_config()
    };
    let mgr = JwtManager::new(&config).unwrap();

    mgr.revoke_token("old-jti");
    assert!(mgr.is_revoked("old-jti"));

    std::thread::sleep(Duration::from_millis(150));
    mgr.cleanup_revoked();

    assert!(!mgr.is_revoked("old-jti"));
}

// -- Algorithm support (US1-AS7) ------------------------------------------

#[test]
fn hs384_support() {
    let config = JwtConfig {
        algorithm: "HS384".to_string(),
        key_source: KeySource::Hmac {
            secret: b"test-secret-key-for-hmac-384-needs-48-bytes-minimum!!!".to_vec(),
        },
        ..hmac_config()
    };
    let mgr = JwtManager::new(&config).unwrap();
    let pair = mgr.generate_token_pair(&test_claims()).unwrap();
    let claims = mgr.validate_token(&pair.access_token).unwrap();
    assert_eq!(claims.agent_id, "agent-001");
}

#[test]
fn hs512_support() {
    let config = JwtConfig {
        algorithm: "HS512".to_string(),
        key_source: KeySource::Hmac {
            secret: b"test-secret-key-for-hmac-512-needs-64-bytes-minimum-so-make-it-long-enough!!"
                .to_vec(),
        },
        ..hmac_config()
    };
    let mgr = JwtManager::new(&config).unwrap();
    let pair = mgr.generate_token_pair(&test_claims()).unwrap();
    let claims = mgr.validate_token(&pair.access_token).unwrap();
    assert_eq!(claims.agent_id, "agent-001");
}

// -- No issuer / no audience validation -----------------------------------

#[test]
fn no_issuer_no_audience() {
    let config = JwtConfig {
        issuer: None,
        audience: Vec::new(),
        ..hmac_config()
    };
    let mgr = JwtManager::new(&config).unwrap();
    let pair = mgr.generate_token_pair(&test_claims()).unwrap();
    let claims = mgr.validate_token(&pair.access_token).unwrap();
    assert_eq!(claims.agent_id, "agent-001");
    assert!(claims.iss.is_none());
}

// -- Invalid token format -------------------------------------------------

#[test]
fn garbage_token_rejected() {
    let mgr = JwtManager::new(&hmac_config()).unwrap();
    let result = mgr.validate_token("not.a.valid.jwt");
    assert!(result.is_err());
}

#[test]
fn empty_token_rejected() {
    let mgr = JwtManager::new(&hmac_config()).unwrap();
    let result = mgr.validate_token("");
    assert!(result.is_err());
}
