# Data Model: Agent-Boundary Security Hardening

## Boundary capability entities

### `BoundaryCapabilityDescriptor`

- `descriptor_id`: stable surface identifier such as `tool:namespace.name`
- `boundary_family`: boundary class such as `tool_bus` or `mcp_tool`
- `title`: human-readable surface title
- `description`: short operator-facing description
- `discover_action`: action binding for bounded discovery
- `execute_action`: action binding for execution
- `required_scope`: execution scope required for the bound surface

### `BoundaryActionBinding`

- `descriptor_id`: capability descriptor the action belongs to
- `action_id`: exact delegated action identifier
- `kind`: discover or execute
- `scope`: delegated scope required, if any
- `resource`: policy resource family
- `resource_id`: exact resource name
- `revocation_key`: revocation handle bound to the action

## Identity and credential entities

### `BoundaryCredentialLease`

- `principal_id`: current boundary principal identifier
- `permission_tier`: full, standard, restricted, or quarantined
- `ttl_secs`: effective credential lifetime
- `publish_allow`: allowed publish subjects
- `subscribe_allow`: allowed subscribe subjects
- `fallback_applied`: whether minimal fallback posture was used
- `delegation_ref`: optional active delegated capability reference

### `SandboxBoundaryClass`

- `agent_class`: persistent or ephemeral
- `account_name`: logical account used for the credential
- `subject_reach`: allowed same-account or crossing-rule reach
- `cleanup_rule`: how credentials and temporary state are cleaned up

## Quarantine and validation entities

### `ValidatedSharedState`

- `state_type`: validator schema reference
- `schema_version`: schema identifier used for validation
- `taint_label`: clean, sanitized, suspicious, or rejected
- `forwardable`: whether the payload may continue toward agent context
- `monitored`: whether the payload should remain under heightened monitoring
- `reason`: deterministic human-readable explanation for any sanitized or monitored outcome

### `QuarantineInspectionRecord`

- `boundary`: boundary class such as `cross_boundary` or `shared_state`
- `source`: source side of the transfer
- `target`: target side of the transfer
- `resource`: crossed subject or state key
- `action`: pass, sanitize, reject, or quarantine
- `taint_label`: final validation label
- `reason`: deterministic human-readable explanation for sanitize, monitored suspicious, reject, or
  quarantine outcomes
- `detected_pattern`: optional malicious pattern marker

## Boundary evidence entity

### `BoundaryEvidenceSummary`

- `surface`: boundary surface such as `tool_bus`, `mcp`, `task_ingress`, or `shared_state`
- `descriptor_id`: exact capability descriptor when applicable
- `action_id`: exact action identifier when applicable
- `outcome`: allowed, sanitized, rejected, quarantined, revoked, or missing
- `continuity_ref`: optional packet `016` continuity reference when the decision is task-ingress
- `audit_ref`: audit or event reference for later operator inspection
- `fabricated`: must stay false for metadata-only projections

## Invariants

- discover and execute are separate action bindings even when they share one descriptor
- descriptor match alone is not enough; action binding must match too
- auth-callout fallback never grants more than quarantined access
- persistent and ephemeral separation remains a boundary rule for credentials and shared-state
  mediation
- shared-state reads and cross-boundary payloads are validated before agent consumption
- suspicious content may be monitored but still requires explicit labeling and a deterministic
  reason
- packet `016` continuity may be preserved, but a live rejection surface must not be fabricated
- MCP metadata uses one descriptor with two first-class actions, not one flattened action summary
