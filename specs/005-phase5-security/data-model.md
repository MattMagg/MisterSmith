# Data Model: Phase 5 Security

**Feature Branch**: `005-phase5-security`
**Date**: 2026-03-04

## Entities

### AgentClaims

JWT claim set combining standard RFC 7519 claims with agent-specific extensions.

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `iss` | `Option<String>` | Issuer (framework instance identifier) | Validated against allowed issuers |
| `sub` | `String` | Subject (agent ID as string) | Required, non-empty |
| `aud` | `Vec<String>` | Audience(s) | Validated against expected audiences |
| `exp` | `u64` | Expiration (Unix timestamp) | Must be in the future |
| `nbf` | `Option<u64>` | Not-before (Unix timestamp) | Must be in the past |
| `iat` | `u64` | Issued-at (Unix timestamp) | Auto-set on generation |
| `jti` | `String` | JWT ID (unique token identifier) | UUID v4, used for revocation |
| `agent_id` | `String` | Agent identifier | Required, valid AgentId format |
| `agent_type` | `String` | Agent type (from AgentType enum) | Must match valid AgentType variant |
| `capabilities` | `Vec<String>` | Agent capabilities | Optional, freeform |
| `permissions` | `Vec<String>` | Granted permissions (`action:resource:scope`) | Optional, validated syntax |
| `session_id` | `Option<String>` | Session tracking identifier | Optional UUID |
| `delegation_chain` | `Vec<String>` | Chain of delegating agents | Optional, ordered list |

### TokenPair

Access + refresh token pair issued during authentication.

| Field | Type | Description |
|-------|------|-------------|
| `access_token` | `String` | Short-lived JWT (default 15 min TTL) |
| `refresh_token` | `String` | Long-lived JWT (default 24 hour TTL) |
| `token_type` | `String` | Always "Bearer" |
| `expires_in` | `u64` | Access token TTL in seconds |

### JwtConfig

Configuration for the JWT subsystem.

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `algorithm` | `Algorithm` | Signing algorithm | `RS256` |
| `access_token_ttl` | `Duration` | Access token lifetime | 15 minutes |
| `refresh_token_ttl` | `Duration` | Refresh token lifetime | 24 hours |
| `issuer` | `Option<String>` | Token issuer claim | None |
| `audience` | `Vec<String>` | Required audience claims | Empty |
| `key_source` | `KeySource` | Where to load signing keys from | Required |

### KeySource (enum)

| Variant | Fields | Description |
|---------|--------|-------------|
| `Hmac` | `secret: Vec<u8>` | Symmetric HMAC secret |
| `RsaPem` | `private_pem: PathBuf, public_pem: PathBuf` | RSA key pair (PEM files) |
| `EcPem` | `private_pem: PathBuf, public_pem: PathBuf` | ECDSA key pair (PEM files) |
| `EdPem` | `private_pem: PathBuf, public_pem: PathBuf` | EdDSA key pair (PEM files) |

### Permission

Represents a single permission grant.

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `action` | `String` | Action verb (read, write, delete, admin, *) | Non-empty, lowercase |
| `resource` | `String` | Resource pattern (agent, task, system, *) | Non-empty, supports wildcards |
| `scope` | `String` | Scope qualifier (own, tenant, all, *) | Non-empty |
| `constraints` | `Option<PolicyConstraints>` | Optional ABAC conditions | See PolicyConstraints |

**Syntax**: `action:resource:scope` (e.g., `read:agent:own`, `admin:system:all`, `*:*:*`)

### Role

Named collection of permissions with optional hierarchy.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Unique role identifier |
| `description` | `Option<String>` | Human-readable description |
| `permissions` | `Vec<Permission>` | Direct permissions for this role |
| `parent` | `Option<String>` | Parent role name for inheritance |

**Default roles**: `admin`, `developer`, `operator`, `viewer`

**Hierarchy**: `admin` → `developer` → `operator` → `viewer` (each inherits permissions from its parent)

### PolicyConstraints (optional ABAC conditions)

| Field | Type | Description |
|-------|------|-------------|
| `time_window` | `Option<TimeWindow>` | Business hours restriction |
| `ip_ranges` | `Option<Vec<IpRange>>` | Allowed IP ranges |
| `resource_owner` | `Option<bool>` | Must own the resource |

### TimeWindow

| Field | Type | Description |
|-------|------|-------------|
| `start_hour` | `u8` | Start hour (0-23) |
| `end_hour` | `u8` | End hour (0-23) |
| `timezone` | `String` | IANA timezone |
| `days` | `Vec<Weekday>` | Active weekdays |

### PolicyDecision

Result of a policy evaluation.

| Field | Type | Description |
|-------|------|-------------|
| `allowed` | `bool` | Whether the request is allowed |
| `reason` | `String` | Human-readable explanation |
| `matching_policy` | `Option<String>` | Policy that produced this decision |
| `evaluated_at` | `DateTime<Utc>` | Timestamp of evaluation |

### AuthorizationRequest

Input to the policy engine.

| Field | Type | Description |
|-------|------|-------------|
| `principal` | `AgentClaims` | Who is making the request |
| `action` | `String` | What action is being performed |
| `resource` | `String` | Target resource type |
| `resource_id` | `Option<String>` | Specific resource instance |
| `context` | `HashMap<String, String>` | Additional context (IP, timestamp, etc.) |

### TlsConfig

Configuration for the TLS subsystem.

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `enabled` | `bool` | Master TLS toggle | `false` |
| `cert_path` | `Option<PathBuf>` | Server certificate PEM path | None |
| `key_path` | `Option<PathBuf>` | Server private key PEM path | None |
| `ca_path` | `Option<PathBuf>` | CA certificate for client verification | None |
| `mtls_enabled` | `bool` | Require client certificates | `false` |
| `min_protocol_version` | `TlsVersion` | Minimum TLS version | `TLS13` |
| `generate_self_signed` | `bool` | Auto-generate certs for dev/test | `false` |
| `reload_interval` | `Option<Duration>` | Certificate reload check interval | None |
| `expiry_warning_days` | `u32` | Days before expiry to warn | `30` |

### SecurityAuditEvent

Structured audit log entry.

| Field | Type | Description |
|-------|------|-------------|
| `event_id` | `String` | Unique event identifier (UUID) |
| `timestamp` | `DateTime<Utc>` | When the event occurred |
| `event_type` | `AuditEventType` | Category of security event |
| `principal` | `Option<String>` | Who triggered the event |
| `resource` | `Option<String>` | Target resource |
| `action` | `Option<String>` | Action attempted |
| `outcome` | `AuditOutcome` | Result of the action |
| `details` | `HashMap<String, String>` | Additional context |
| `source_ip` | `Option<String>` | Originating IP address |
| `previous_hash` | `Option<String>` | Hash chain link for tamper-evidence |

### AuditEventType (enum)

`Authentication`, `Authorization`, `TokenLifecycle`, `CertificateEvent`, `SuspiciousActivity`, `SystemAccess`, `ConfigurationChange`

### AuditOutcome (enum)

`Success`, `Failure`, `Blocked`, `Warning`

## Relationships

```
SecurityConfig (expanded)
├── JwtConfig ─── KeySource
│   └── produces → AgentClaims ─── TokenPair
├── RbacConfig
│   ├── Role[] ─── Permission[] ─── PolicyConstraints?
│   └── PolicyEngine evaluates AuthorizationRequest → PolicyDecision
├── TlsConfig ─── CertificateManager
│   └── produces → ServerConfig, ClientConfig (rustls)
└── AuditConfig
    └── SecurityAuditEvent[] (hash-chained)
```

## State Transitions

### Token Lifecycle

```
Generated → Valid → Refreshed → Valid (new access token)
                 → Expired (natural TTL expiry)
                 → Revoked (explicit revocation)
```

### Certificate Lifecycle

```
Generated/Loaded → Active → NearExpiry (warning at N days) → Expired
                         → Rotated (new cert loaded, old stays for existing connections)
```
