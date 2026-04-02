# Research Notes: Agent-Boundary Security Hardening

## Current repo truth

- ToolBus already models `CapabilityDescriptor` with separate discover and execute actions in
  `crates/mister-smith-agents/src/tool_bus.rs`.
- MCP already enforces discover-versus-execute separation at the server boundary, and
  `handle_tools_call` already validates the delegated boundary action before handler execution in
  `crates/mister-smith-mcp/src/server.rs`.
- The MCP compatibility layer already exposes bounded discovery through
  `describe_external_capabilities` in `crates/mister-smith-mcp/src/compatibility.rs`.
- `DelegationService` already validates exact `DelegatedAction` bindings, revocation, expiry, and
  bounded external envelopes in `crates/mister-smith-security/src/delegation.rs`.
- Auth callout already narrows credentials by trust tier in
  `crates/mister-smith-security/src/auth_callout.rs`.
- The validation and quarantine seams already cover size, sanitization, schema validation,
  malicious-pattern inspection, taint labels, and quarantine outcomes in
  `crates/mister-smith-security/src/state_validator.rs` and
  `crates/mister-smith-security/src/quarantine.rs`.
- Sandbox and persistence seams already preserve persistent-versus-ephemeral separation and
  mediated shared-state access in `crates/mister-smith-security/src/sandbox.rs`,
  `crates/mister-smith-agents/src/sandbox.rs`, and
  `crates/mister-smith-persistence/src/repository/agent.rs`.
- Packet `016` already closed the accepted delegated HTTP task-ingress continuity slice and kept
  live rejection proof out of scope because the runtime still has no workflow-backed reject path.
- Packet `022` is landed on the current repo truth, so packet `024` must compose with the durable
  lifecycle substrate rather than with stale pre-landing assumptions.

## Exact hardening gaps on current `main`

### G1: Legacy descriptorless execute authority still exists

- `DelegationService::validate_descriptor_binding` still accepts `None` for capability
  descriptors.
- That legacy allowance means an action-bound execute path can still pass without the descriptor
  binding that packet `024` now requires.

### G2: MCP capability metadata still flattens the action shape

- ToolBus already has distinct discover and execute actions.
- MCP client-facing descriptor metadata still collapses that surface to one action summary instead
  of publishing `discover_action` and `execute_action` end to end.

### G3: Quarantine evidence reasons are incomplete

- Rejected and quarantined outcomes already explain themselves.
- Sanitized pass-through results and monitored suspicious pass-through results do not always carry
  a deterministic human-readable reason in the audit surface.

### G4: Auth-callout fallback can be widened by override

- `AuthCalloutService::with_default_permissions` can change the fallback baseline.
- Packet `024` needs fallback to stay no broader than the quarantined ceiling even when broader
  defaults are configured.

## Official sources and why they matter

### MCP protocol baseline

- Use MCP `2025-11-25` versioned protocol pages as the packet baseline.
- Keep protocol claims pinned to that version instead of mixing latest and versioned guidance.

### MCP operational guidance

- MCP security best-practices pages matter as hardening guidance.
- They are not the frozen protocol contract for packet `024`.

### Transport and validation sources

- NATS authorization remains the transport least-privilege baseline.
- NATS auth callout remains the current credential-issuance baseline.
- JSON Schema remains the structural validation reference for boundary payloads.

### Comparator only

- SPIFFE is still useful comparator material for later identity work.
- It is not part of the implementation baseline for this packet.

## Research signals that matter here

### Deterministic enforcement beats prompt-only defenses

- Boundary rules need to be infrastructure-enforced, not prompt-enforced.
- Packet `024` therefore freezes deterministic boundary checks instead of adding softer agent
  instructions.

### Persistent and ephemeral separation stays the strongest existing defense

- The Phase 9.1 contracts and current sandbox code already converge on persistent-versus-ephemeral
  separation, mediated crossings, and explicit cleanup.
- Packet `024` keeps that posture as a boundary rule instead of redesigning identity.

### Shared-state reads must stay mediated

- The repo already treats shared-state reads as boundary crossings.
- Packet `024` keeps the rule that shared-state content must be validated before agent
  consumption.

### Discover and execute must stay distinct

- MS-77 already proved that bounded discovery works without ambient execute authority.
- Packet `024` hardens that same boundary instead of widening it.

## Bounded conclusion

Packet `024` is an implementation-ready hardening freeze over existing repo seams. It does not
create a new identity system, a broader interop contract, or a larger compliance program. The
honest packet scope is:

- least-privilege capability scoping
- exact descriptor-and-action binding
- explicit discover-and-execute metadata
- quarantine and schema enforcement before agent consumption
- persistent-versus-ephemeral sandbox rules
- auth-callout fallback clamped to quarantined access
- packet `016` continuity preserved without inventing a new live reject surface
