# Contract: AuditLogger

**Module**: `mister_smith_security::audit`

## Public API

### AuditLogger

```rust
pub struct AuditLogger { /* private */ }

impl AuditLogger {
    /// Create a new AuditLogger.
    pub fn new(config: &AuditConfig) -> Self;

    /// Record a security audit event.
    /// Appends to the hash chain and publishes to EventBus if available.
    pub fn record(&self, event: SecurityAuditEvent);

    /// Record an authentication event (convenience method).
    pub fn record_auth(
        &self,
        principal: &str,
        outcome: AuditOutcome,
        details: HashMap<String, String>,
    );

    /// Record an authorization event (convenience method).
    pub fn record_authz(
        &self,
        principal: &str,
        action: &str,
        resource: &str,
        outcome: AuditOutcome,
    );

    /// Get recent audit events (for health/debug endpoints).
    pub fn recent_events(&self, limit: usize) -> Vec<SecurityAuditEvent>;

    /// Verify the integrity of the audit hash chain.
    /// Returns the index of the first tampered entry, if any.
    pub fn verify_chain(&self) -> Result<(), usize>;

    /// Check for suspicious activity patterns.
    /// Returns alert events if thresholds are exceeded.
    pub fn check_alerts(&self) -> Vec<SecurityAuditEvent>;
}
```

### AuditConfig

```rust
pub struct AuditConfig {
    /// Enable/disable audit logging.
    pub enabled: bool,
    /// Maximum number of events to retain in memory.
    pub max_events: usize,
    /// Threshold for repeated auth failures before alert (per source, per minute).
    pub auth_failure_alert_threshold: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_events: 10_000,
            auth_failure_alert_threshold: 5,
        }
    }
}
```

### Hash Chain

Each `SecurityAuditEvent` includes a `previous_hash` field containing the SHA-256 hash of the prior entry. The first entry has `previous_hash: None`. This provides tamper-evidence — modifying any entry breaks the chain for all subsequent entries.

### Alert Thresholds

| Pattern | Threshold | Alert Type |
|---------|-----------|------------|
| Repeated auth failures from same source | 5 per minute | `SuspiciousActivity` |
| Privilege escalation attempt | Any | `SuspiciousActivity` |
| Certificate expiry warning | < 30 days | `CertificateEvent` |

### Thread Safety

`AuditLogger` is `Send + Sync`. Internal event storage uses `parking_lot::RwLock<VecDeque<SecurityAuditEvent>>`.

### Test Contract

```rust
#[test] fn record_auth_success();
#[test] fn record_auth_failure();
#[test] fn record_authz_denied();
#[test] fn hash_chain_integrity();
#[test] fn tampered_entry_detected();
#[test] fn auth_failure_alert_threshold();
#[test] fn recent_events_returns_limited_results();
#[test] fn max_events_enforced();
```
