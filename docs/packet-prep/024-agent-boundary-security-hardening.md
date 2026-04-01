# Packet 024: Agent-Boundary Security Hardening

## Packet Name

Agent-boundary security hardening

## Why This Packet Exists

Mister Smith already has real security and delegation substrate. It also already has bounded MCP
discovery and enforcement surfaces. But the next expansion steps increase risk:

- stronger step-level control
- real coordinator-subagent runtime
- more external capability discovery

This packet exists to harden the boundary before those later packets widen trust and delegation.

## Why This Stage Is Correct

`docs/direction.md` is explicit: broader autonomy and interoperability should not expand faster
than enforcement, quarantine, validation, and delegation controls. This is the last clean point to
strengthen the boundary before more federation-like behavior becomes tempting.
This packet should freeze boundary rules, not silently choose a later interoperability protocol
baseline on its own.

## Repo Truth Status

- Packet outcome today: `planned-only`
- Foundation truth status: `landed-not-default`
- Live-default today:
  - ToolBus already distinguishes discover vs execute and can require delegated authority
  - transport envelopes already carry signatures, nonces, and capability tokens
  - accepted delegated HTTP task ingress continuity from packet `016` is a real supported-path
    proof note
- Landed but not yet one hardened boundary posture:
  - RBAC policy checks, auth callout, delegation validation, quarantine, state validation, and
    audit emission hooks are real shipped repo contracts
  - bounded MCP discovery and delegated call-boundary enforcement are landed
- Missing for this packet:
  - one sharper default least-privilege posture across ToolBus, MCP, and later remote capability
    adapters
  - one explicit runtime rule for when traffic must be quarantined, sanitized, or rejected

## Current Repo Grounding

### Live on the default runtime path now

- JWT auth, RBAC, audit logging, and delegation services
- `ToolBus` capability descriptors, discover vs execute actions, and delegation-aware enforcement
- `MessageEnvelope` support for signatures, nonces, and capability tokens
- accepted delegated HTTP task ingress continuity from packet `016`

### Landed in repo but not yet one frozen boundary-hardening packet

- bounded external capability inspection via `describe_external_capabilities`
- Phase 9.1 auth callout, quarantine, and state-validation surfaces
- security substrate exists, but the repo does not yet enforce all boundary-hardening patterns as
  one coherent runtime packet
- some research-backed hardening posture is still spread across crates, tests, and old closure docs
  rather than carried as one clearly frozen runtime contract

### Missing pieces

- stricter boundary quarantine for cross-agent content and memory retrieval
- consistent short-lived identity and least-privilege posture for every agent boundary
- one deterministic control-flow and capability-scoping story across local agents, MCP, and future
  interoperability surfaces
- clear persistent-vs-ephemeral agent split

### High-Signal Repo Anchors

- `crates/mister-smith-security/src/rbac/mod.rs`
  - `AuthorizationRequest`
  - `PolicyEngine`
  - deny-wins semantics
  - This is the current deterministic authorization core.
- `crates/mister-smith-security/src/delegation.rs`
  - `DelegationService`
  - `ValidatedDelegation`
  - `validate_external_envelope`
  - `external_delegation_envelope`
  - This is the current delegated-authority and revocation seam.
- `crates/mister-smith-security/src/auth_callout.rs`
  - `AuthCalloutHandler`
  - `with_jwt_manager`
  - `with_delegation_service`
  - `Permissions::quarantined`
  - JWT claim handling for NATS authorization callout
  - This is the current dynamic-credential boundary.
- `crates/mister-smith-security/src/quarantine.rs`
  - `QuarantineActor`
  - `inspect_quarantine_payload`
  - `record_quarantine_audit_event`
  - This is the current sanitize/reject/isolate seam.
- `crates/mister-smith-security/src/state_validator.rs`
  - `JsonSchemaStateValidator`
  - `register_schema`
  - This is the current structural-validation seam.
- `crates/mister-smith-agents/src/sandbox.rs`
  - `with_quarantine_actor`
  - This is the runtime hookup from agent execution into quarantine handling.
- `crates/mister-smith-security/src/sandbox.rs`
  - `check_crossing`
  - This is the current cross-boundary enforcement seam.
- `crates/mister-smith-persistence/src/repository/agent.rs`
  - `AgentRepository::get_state`
  - `quarantine_actor`
  - This is the strongest persisted-state mediation seam for boundary crossing and quarantine.
- `crates/mister-smith-agents/src/tool_bus.rs`
  - `ToolPrincipal`
  - policy-engine enforcement
  - delegated MCP call wrapping
  - This is the live call-boundary enforcement point.
- `crates/mister-smith-mcp/src/server.rs`
  - `ToolCallRequest`
  - `CapabilityCatalogEntry`
  - `handle_tools_list`
  - This is the external MCP exposure boundary.
- `crates/mister-smith-mcp/src/compatibility.rs`
  - `describe_external_capabilities`
  - `compatibility_server_accepts_valid_delegated_request`
  - `describe_external_capabilities_requires_discover_delegation`
  - `describe_external_capabilities_returns_catalog_with_matching_discover_delegation`
  - This is the strongest current discovery-gated boundary-hardening seam for MCP exposure.
- `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
  - accepted delegated `POST /api/v1/tasks` proof
  - explicit note that rejected delegated HTTP requests do not create a workflow-backed reject
    surface on the current path
  - This is the strongest live proof note for the delegated task-ingress boundary that packet `024`
    must harden without widening.
- `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`
  - bounded MCP discovery surface on the existing zero-trust boundary
  - explicit discover-vs-execute contract for external capability inspection
  - This is the clearest landed proof note for the MCP-side boundary packet `024` must preserve.
- `crates/mister-smith-agents/tests/quarantine_tests.rs`
- `crates/mister-smith-agents/tests/tool_bus_tests.rs`
- `crates/mister-smith-security/tests/auth_callout_tests.rs`
- `crates/mister-smith-security/tests/validator_tests.rs`
- `crates/mister-smith-security/tests/sandbox_tests.rs`
  - These are the existing deterministic proof anchors for the hardening seam.

## Official Docs / Primary Sources

### Protocol And Transport Baselines

- [NATS authorization](https://docs.nats.io/running-a-nats-service/configuration/securing_nats/authorization)  
  Why it matters: subject-level least privilege is the first hard boundary in the actual transport.
- [NATS auth callout](https://docs.nats.io/running-a-nats-service/configuration/securing_nats/auth_callout)  
  Why it matters: dynamic per-connection credentials are the cleanest official NATS pattern for adaptive scoping.
- [MCP versioning](https://modelcontextprotocol.io/specification/versioning)  
  Why it matters: use this first so packet `024` does not silently mix MCP revisions while packet
  `027` is still pre-spec.
- [MCP authorization](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization)  
  Why it matters: official protocol guidance for authorization expectations on MCP boundaries when
  the dossier set later reuses the same pinned revision for interoperability work.

### Operational Hardening Guidance

- [MCP security best practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)  
  Why it matters: keep this as official advisory hardening guidance for trust boundaries and
  consent, not as the version-frozen MCP protocol baseline.
- [SPIFFE overview](https://spiffe.io/docs/latest/spiffe-about/overview/)  
  Why it matters: workload-identity comparator if Mister Smith later needs stronger non-user
  identity beyond the current local runtime posture.
- [JSON Schema specification](https://json-schema.org/specification)  
  Why it matters: primary schema-validation reference for transport-safe structural enforcement.

## Research Findings That Matter

- The security corpus is explicit that LLMs are not reliable policy enforcers.
- AgentSandbox-style persistent/ephemeral separation is the strongest documented architectural
  defense in the repo research.
- Trust scales exposure. Stronger multi-agent systems need stricter deterministic boundaries, not
  softer ones.
- Cross-boundary data should move through quarantine and validation, not directly into agent context.

## Best-Practice Guidance

- Keep authorization and policy enforcement infrastructure-level and deterministic.
- Prefer dynamic, least-privilege credentials over broad static permissions.
- Validate structure at the transport edge and sanitize semantics before agent consumption.
- Split persistent state holders from disposable task executors.
- Keep discover and execute permissions separate everywhere.
- Let packet `027` own protocol-version pinning and remote capability metadata. Packet `024`
  should make the boundary safe regardless of which interop revision is later frozen.
- Preserve audit and revocation surfaces as first-class runtime facts, not optional logs.

## Likely Architecture Shape

- hardened agent boundary gateway around tool, MCP, and future inter-agent calls
- dynamic credential issuance and narrowing
- content quarantine and schema-validation sidecars or middleware
- explicit persistent-agent vs ephemeral-agent runtime roles
- unified capability-scoping rules across HTTP, ToolBus, and protocol adapters

## Risks / Constraints / Non-Goals

- Do not rely on prompt-only guardrails as the main defense.
- Do not widen this packet into general enterprise IAM or compliance work.
- Do not make interoperability easier before the boundary model is stronger.
- Do not give wildcard transport permissions to task-executing agents.

## Open Questions Before Spec Writing

- Should workload identity stay JWT-only, or should SPIFFE-style identity be introduced later?
- Where should semantic quarantine live relative to ToolBus and MCP boundaries?
- Which capabilities should be short-lived by default?
- How should revocation and audit evidence be projected into operator-facing views?
- Does the first slice require persistent/ephemeral agent separation immediately, or can it start
  with tighter mediation around existing agents?

## Fixed Constraints Before Spec Writing

- Keep packet `024` about deterministic least-privilege, quarantine, schema enforcement,
  delegation, and identity boundaries. Do not widen it into general IAM or interop design.
- Keep discover and execute permission separate across ToolBus, MCP, and later adapters.
- Treat LLMs as untrusted policy subjects, not as the policy engine.
- Freeze the boundary model before making later packets widen delegation or remote capability
  reach.

## Recommended Inputs For Future SpecKit Packet

Read these in order: repo routers -> current delegated-boundary proof -> Phase 9.1 contracts ->
security and MCP seams -> official protocol and identity docs.

- `docs/direction.md`
- `docs/current-state.md`
- `docs/research-output/consolidated/04-security-and-trust.md`
- `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`
  - use as the current bounded MCP discovery-boundary proof note before freezing harder boundary
    rules around later interop work
- `specs/011-phase9.1-security-hardening/spec.md`
- `specs/011-phase9.1-security-hardening/contracts/auth-callout.md`
- `specs/011-phase9.1-security-hardening/contracts/agent-sandbox.md`
- `specs/011-phase9.1-security-hardening/contracts/state-validator.md`
- `crates/mister-smith-security/src/rbac/mod.rs`
  - start from `AuthorizationRequest` and `PolicyEngine`
- `crates/mister-smith-security/src/delegation.rs`
  - start from `DelegationService`, `ValidatedDelegation`, and
    `validate_external_envelope`
- `crates/mister-smith-security/src/auth_callout.rs`
  - start from `AuthCalloutHandler`, `with_delegation_service`, and the current NATS auth-callout
    claim and fallback-permission logic
- `crates/mister-smith-security/src/quarantine.rs`
  - start from `QuarantineActor` and quarantine audit emission
- `crates/mister-smith-security/src/state_validator.rs`
  - start from `JsonSchemaStateValidator` and `register_schema`
- `crates/mister-smith-agents/src/sandbox.rs`
  - start from `with_quarantine_actor`
- `crates/mister-smith-security/src/sandbox.rs`
  - start from `check_crossing`
- `crates/mister-smith-persistence/src/repository/agent.rs`
  - start from `AgentRepository::get_state` and the persisted-state quarantine mediation helpers
- `crates/mister-smith-agents/src/tool_bus.rs`
  - start from `ToolPrincipal` and delegated execute/discover enforcement
- `crates/mister-smith-mcp/src/server.rs`
  - start from `ToolCallRequest` and `CapabilityCatalogEntry`
- `crates/mister-smith-mcp/src/compatibility.rs`
  - start from `describe_external_capabilities` and the delegated-request enforcement tests
- `crates/mister-smith-agents/tests/quarantine_tests.rs`
- `crates/mister-smith-agents/tests/tool_bus_tests.rs`
- `crates/mister-smith-security/tests/auth_callout_tests.rs`
- `crates/mister-smith-security/tests/validator_tests.rs`
- `crates/mister-smith-security/tests/sandbox_tests.rs`
- only after the repo-local boundary contracts are clear, re-confirm the official docs and primary
  sources linked earlier
