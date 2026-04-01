# Research Notes: Agent-Boundary Security Hardening

## Current repo truth

- ToolBus already models `CapabilityDescriptor` with separate discover and execute actions in
  `crates/mister-smith-agents/src/tool_bus.rs`
- MCP already publishes descriptor and action metadata, and `handle_tools_call` validates the
  exact delegated boundary action before handler execution in
  `crates/mister-smith-mcp/src/server.rs`
- `describe_external_capabilities` already keeps capability discovery bounded to a discover action
  on the MCP side in `crates/mister-smith-mcp/src/compatibility.rs`
- `DelegationService` already validates exact `DelegatedAction` bindings, revocation, and bounded
  external envelopes in `crates/mister-smith-security/src/delegation.rs`
- auth callout already narrows permissions by trust tier and falls back to quarantined permissions
  in `crates/mister-smith-security/src/auth_callout.rs`
- the validator and quarantine seams already define size, schema, malicious-pattern, sanitize,
  reject, and quarantine behavior in `crates/mister-smith-security/src/state_validator.rs` and
  `crates/mister-smith-security/src/quarantine.rs`
- sandbox and persistence seams already freeze persistent-versus-ephemeral separation plus
  mediated shared-state access in `crates/mister-smith-security/src/sandbox.rs`,
  `crates/mister-smith-agents/src/sandbox.rs`, and
  `crates/mister-smith-persistence/src/repository/agent.rs`
- packet `016` already closed the accepted delegated task-ingress continuity slice and explicitly
  kept live rejection proof out of scope

## Official docs and why they matter

### MCP protocol baseline

- MCP versioning page
  - keep the packet on one versioned protocol baseline instead of drifting between latest and
    versioned pages
- MCP `2025-11-25` authorization page
  - keep authorization expectations tied to the same pinned revision the packet prep dossier names

### Operational guidance, not frozen protocol

- MCP security best practices
  - use as operational advice about trust boundaries, consent, and hardening
  - do not treat it as the canonical protocol contract for packet `024`

### Transport and validation

- NATS authorization
  - subject-level least privilege remains the first hard boundary for current transport posture
- NATS auth callout
  - dynamic per-connection credentials remain the cleanest official fit for the current auth
    posture
- JSON Schema specification
  - primary source for structural validation at the boundary

### Comparator only

- SPIFFE overview
  - useful comparator for future workload identity work
  - not part of the implementation baseline for packet `024`

## Research signals that matter here

### Deterministic enforcement beats prompt-only defenses

- the consolidated security research is explicit that LLMs are unreliable policy enforcers
- packet `024` therefore needs to freeze infrastructure-level boundary rules, not stronger prompt
  instructions

### Persistent and ephemeral separation is the strongest architectural defense already pointed to

- the research and Phase 9.1 contracts converge on persistent-versus-ephemeral separation, I/O
  firewall rules, and quarantine mediation as the most useful boundary pattern to preserve

### Cross-boundary memory and shared-state reads must be mediated

- infectious jailbreak, MINJA, and memory-injection findings all support the current repo move:
  do not pass shared-state content straight into agent context

### Discover-versus-execute separation is already the cleanest current boundary posture

- MS-77 and the MCP server/client seams already prove the repo has the right shape for bounded
  discovery without ambient execute authority
- packet `024` should freeze and generalize that posture rather than reopening it

## Bounded conclusion

Packet `024` should not invent a new identity system, a broader interop contract, or a larger
compliance program. The honest next packet is a boundary-hardening freeze over what the repo
already has: least-privilege capability scoping, exact descriptor/action binding, quarantine and
schema enforcement, persistent-versus-ephemeral sandbox rules, auth-callout narrowing, and packet
`016` continuity.

Because earlier packets are still moving, this packet stays explicitly provisional until a refresh
pass confirms that its reused contracts still match landed repo truth.
