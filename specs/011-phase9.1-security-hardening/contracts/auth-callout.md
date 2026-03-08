# Contract: Auth Callout

## Overview

The Auth Callout contract defines the NATS Auth Callout service that dynamically generates
JWTs based on agent trust profiles. Instead of static JWTs, agents receive dynamically-scoped
permissions tailored to their current behavioral trust level.

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `docs/research-output/consolidated/04-security-and-trust.md` | Auth Callout patterns and trust model |
| NATS Auth Callout protocol documentation | Protocol specification for `$SYS.REQ.USER.AUTH` |
| `spec/security/` | Existing JWT and RBAC patterns extended |

## Protocol

The NATS Auth Callout protocol works as follows:

1. NATS server receives a client connection attempt.
2. Server publishes a request to `$SYS.REQ.USER.AUTH` with the client's connection info.
3. The Auth Callout service (our handler) receives the request.
4. The handler looks up the agent's trust profile and generates a scoped JWT.
5. The handler responds with the authorization result (JWT or rejection).

## Public API

```rust
pub struct AuthCalloutHandler {
    trust_store: Arc<RwLock<HashMap<String, TrustProfile>>>,
    signing_key: nkeys::KeyPair,
    default_permissions: Permissions,
    max_jwt_ttl_secs: u64,
}

impl AuthCalloutHandler {
    /// Start the Auth Callout service, subscribing to $SYS.REQ.USER.AUTH
    pub async fn start(&self, nats_client: &async_nats::Client) -> Result<(), SecurityError>;

    /// Look up trust profile and generate scoped JWT
    pub fn authorize(&self, agent_id: &str) -> Result<AuthorizationResult, SecurityError>;

    /// Update an agent's trust profile
    pub fn update_trust(&self, agent_id: &str, profile: TrustProfile);

    /// Record a security violation, potentially degrading trust
    pub fn record_violation(&self, agent_id: &str);
}
```

## Trust-to-Permission Mapping

| Permission Tier | Trust Score | Subject Access | JWT TTL |
| --------------- | ----------- | -------------- | ------- |
| `Full` | >= 0.9 | All authorized subjects | 300s |
| `Standard` | >= 0.5 | Normal operational subjects | 120s |
| `Restricted` | >= 0.2 | Limited subjects | 60s |
| `Quarantined` | < 0.2 | Minimal subjects (health only) | 30s |

## Fallback Behavior

When the Auth Callout service is unavailable:

1. NATS server falls back to static authorization (if configured).
2. If no static auth is configured, the handler's `default_permissions` (minimal access) apply.
3. The fallback MUST NOT grant elevated permissions — always default to `Quarantined` tier.

## Trust Degradation

- Each security violation decrements `trust_score` by a configurable amount (default 0.1).
- Violations include: signature verification failure, replay attempt, unauthorized subject
  access, quarantine triggering.
- Trust recovery is time-based: score increases by a configurable amount per assessment period
  if no violations occur.
- Trust score is clamped to [0.0, 1.0].

## Behavioral Requirements

1. The Auth Callout service MUST be operationally independent (separate process/service).
2. JWT generation MUST be fast (<1ms) to avoid connection delays.
3. Trust profiles MUST be persisted to survive service restarts.
4. Multiple Auth Callout instances MUST be supported for high availability.
5. The service MUST log all authorization decisions for audit.

## Validation Requirements

- Dynamic JWT generation with correct permission scoping per trust tier.
- Trust degradation after violations narrows permissions on next connection.
- Fallback to minimal permissions when trust store lookup fails.
- JWT TTL enforcement per tier.
- Multiple concurrent authorization requests handled without contention.
