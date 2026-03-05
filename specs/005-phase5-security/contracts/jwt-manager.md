# Contract: JwtManager

**Module**: `mister_smith_security::jwt`

## Public API

### JwtManager

```rust
pub struct JwtManager { /* private */ }

impl JwtManager {
    /// Create a new JwtManager from configuration.
    pub fn new(config: &JwtConfig) -> Result<Self, SecurityError>;

    /// Generate a token pair (access + refresh) for the given agent claims.
    pub fn generate_token_pair(&self, claims: &AgentClaims) -> Result<TokenPair, SecurityError>;

    /// Validate an access token and extract claims.
    pub fn validate_token(&self, token: &str) -> Result<AgentClaims, SecurityError>;

    /// Refresh an access token using a valid refresh token.
    pub fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair, SecurityError>;

    /// Revoke a token by its JTI (JWT ID).
    pub fn revoke_token(&self, jti: &str);

    /// Check if a token has been revoked.
    pub fn is_revoked(&self, jti: &str) -> bool;

    /// Clean up expired entries from the revocation list.
    pub fn cleanup_revoked(&self);
}
```

### Error Cases

| Method | Error Condition | SecurityError Variant |
|--------|----------------|----------------------|
| `new` | Invalid key source (file not found, invalid PEM) | `KeyLoadFailed(String)` |
| `generate_token_pair` | Encoding failure | `TokenGenerationFailed(String)` |
| `validate_token` | Expired token | `TokenExpired` |
| `validate_token` | Invalid signature | `InvalidSignature` |
| `validate_token` | Revoked token | `TokenRevoked` |
| `validate_token` | Malformed token | `InvalidToken(String)` |
| `refresh_token` | Invalid refresh token | `InvalidToken(String)` |
| `refresh_token` | Expired refresh token | `TokenExpired` |

### Thread Safety

`JwtManager` is `Send + Sync`. The revocation list uses `DashMap<String, Instant>` for concurrent access without locking.

### Test Contract

```rust
#[test] fn generate_and_validate_roundtrip();
#[test] fn expired_token_rejected();
#[test] fn wrong_key_rejected();
#[test] fn revoked_token_rejected();
#[test] fn refresh_produces_new_access_token();
#[test] fn cleanup_removes_expired_revocations();
#[test] fn rs256_algorithm_supported();
#[test] fn es256_algorithm_supported();
#[test] fn hs256_algorithm_supported();
```
