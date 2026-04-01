# Packet 027: Capability Discovery and Interoperability

## Packet Name

Capability discovery and interoperability

## Why This Packet Exists

Mister Smith already has bounded local capability discovery on the MCP side and a ToolBus with
capability descriptors. What it does not yet have is a stronger, protocol-aware interoperability
layer for discovering, matching, and safely consuming external capability surfaces across runtime
boundaries.

## Why This Stage Is Correct

`docs/direction.md` places capability discovery and interoperability after stronger security,
routing discipline, and supervision. That still fits here. This packet should explicitly depend on
`022`, `023`, and `024`, then reuse their lifecycle, proof-boundary, and security contracts
instead of redefining them.
It should also freeze one narrow protocol baseline on purpose, not slide forward to whatever
upstream version happens to be newest when the future packet is drafted.

## Repo Truth Status

- Packet outcome today: `planned-only`
- Foundation truth status: `landed-not-default`
- Live-default today:
  - no broad remote-agent interoperability path is part of the default supported runtime flow
- Landed but not yet one frozen interoperability contract:
  - bounded capability descriptors exist in `ToolBus`
  - MCP catalog exposure and delegated call-boundary enforcement are landed
- Deterministic-only today:
  - `describe_external_capabilities` and the compatibility-server discovery tests prove one bounded
    discovery surface, but they do not make broad remote interoperability part of the default live
    runtime path
- Missing for this packet:
  - one frozen common capability descriptor across local tools, MCP, and later remote agents
  - one frozen lifecycle mapping from remote task state into Mister Smith workflow/result surfaces
  - one version-pinned protocol posture for the first interop slice

## Current Repo Grounding

### Landed in repo but not the default live path

- `ToolBus` capability descriptors and delegated actions
- bounded MCP capability discovery and enforcement via `describe_external_capabilities`
- additive external-agent interoperability surfaces and capability adapters in the repo
- task/session/autonomy identifiers that can support cross-boundary lifecycle mapping

### Partial foundations

- discovery exists for bounded local surfaces, but not yet as one stronger interoperability layer
- the runtime has some external-boundary provenance, but not yet a full cross-protocol task model

### Deterministically grounded but not yet live as broad interoperability

- `describe_external_capabilities` and its compatibility-server tests prove one bounded discovery
  surface on the MCP boundary
- those proofs do not make remote task lifecycle mapping or multi-protocol interoperability part of
  the default runtime path

### Missing pieces

- discovery contract that spans local tools, MCP surfaces, and future A2A-style agents
- capability metadata normalization and matching rules
- interoperable long-running task mapping and status handling
- secure credential and consent model for remote capability consumption

### High-Signal Repo Anchors

- `crates/mister-smith-agents/src/tool_bus.rs`
  - `ToolPrincipal`
  - `CapabilityDescriptor`
  - `ToolEntry.capability_descriptor`
  - `register_mcp`
  - `register_mcp_tool`
  - This is the current local capability registry seam.
- `crates/mister-smith-mcp/src/server.rs`
  - `ToolCallRequest`
  - `CapabilityCatalogEntry`
  - `handle_tools_list`
  - `capability_catalog`
  - This is the current external MCP exposure seam.
- `crates/mister-smith-mcp/src/client.rs`
  - `ExternalCapabilityDescriptor`
  - `McpTool`
  - This is the current inbound external capability-description seam.
- `crates/mister-smith-mcp/src/compatibility.rs`
  - `describe_external_capabilities`
  - `build_smith_compatibility_server`
  - registered capability catalog state
  - This is the current bounded discovery and compatibility adapter seam.
- `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
  - This is the current external-boundary proof note to preserve.
- `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`
  - This is the clearest repo note for the already-landed bounded discovery surface and its
    discover-vs-execute contract.
- `crates/mister-smith-mcp/src/compatibility.rs` tests
  - `describe_external_capabilities_requires_discover_delegation`
  - `describe_external_capabilities_returns_catalog_with_matching_discover_delegation`
  - These are the strongest current deterministic proof anchors for the discovery boundary.
- `crates/mister-smith-agents/tests/tool_bus_tests.rs`
  - This is the strongest local proof anchor for capability descriptors and delegated
    discover-vs-execute behavior before any cross-protocol mapping is frozen.

### Protocol Freeze Guidance

- Start from the protocol version or release page before choosing any normative A2A or MCP
  reference set.
- For A2A, keep this dossier pinned to `v0.3.0`. Do not silently mix `v0.3.0`, `latest`, and
  `dev` pages inside the same packet.
- For MCP, start from the versioning page, then keep lifecycle, tools, and authorization on one
  pinned revision. Do not mix `latest`, `2025-06-18`, and `2025-11-25` pages in the same packet.
- Do not let packet `027` widen into protocol-head chasing. The first job is one honest interop
  slice, not tracking every upstream release.

## Official Docs / Primary Sources

- [MCP versioning](https://modelcontextprotocol.io/specification/versioning)  
  Why it matters: this is the first stop before freezing any MCP revision in a future packet.
- [MCP lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle)  
  Why it matters: capability negotiation and lifecycle are the base contract for MCP interoperability.
- [MCP tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)  
  Why it matters: official server-side contract for callable tool discovery and invocation.
- [MCP authorization](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization)  
  Why it matters: official authorization discovery and token-scope contract for MCP boundaries.
- [MCP tasks utility](https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/tasks)  
  Why it matters: official task metadata and long-running status contract for MCP request flows.
- [A2A specification](https://a2a-protocol.org/v0.3.0/specification/)  
  Why it matters: official high-level contract for the first interoperability slice and the pinned
  dossier baseline.
- [A2A agent discovery](https://a2a-protocol.org/v0.3.0/topics/agent-discovery/)  
  Why it matters: stable agent-card and discovery rules for the first interop slice.
- [A2A life of a task](https://a2a-protocol.org/v0.3.0/topics/life-of-a-task/)  
  Why it matters: stable task/context model for long-running remote interactions.

## Research Findings That Matter

- The coordination transfer brief says protocol metadata and replay/buffering semantics should
  become a real seam before stronger protocol correctness claims.
- The dynamic-orchestration brief says capability and topology expansion only become honest once
  the lower substrate is stronger.
- The security corpus says discovery must stay separated from execute permission and must stay
  under strong enforcement.

## Best-Practice Guidance

- Let packet `027` own capability metadata normalization and remote lifecycle mapping. Do not let
  it absorb packet `023` truth-schema work or packet `028` strong-coordination policy.
- Pick one first interop slice explicitly. Do not write a packet that tries to normalize MCP
  discovery, A2A lifecycle mapping, and broad remote-agent federation all at once.
- Keep capability discovery separate from capability execution.
- Normalize capabilities around stable identifiers, schemas, lifecycle, and auth requirements.
- Reuse task and context identifiers across interoperability boundaries where possible.
- Make capability change notifications explicit and observable.
- Treat remote agent cards and descriptors as inputs to policy, not as implicit trust.
- Prefer narrow adapters and explicit protocol bridges over magical universal wrappers.

## Likely Architecture Shape

- unified capability registry model for local tools, MCP servers, and A2A agents
- adapter layer that maps remote capability/task models into Mister Smith runtime metadata
- lifecycle bridge for remote task start, progress, interruption, and terminal states
- operator-visible provenance for remote capability use and boundary decisions

## Risks / Constraints / Non-Goals

- Do not widen this into generic mesh or federation work.
- Do not let interoperability bypass local security and proof-boundary rules.
- Do not make Agent Cards or MCP capability ads equal to trusted execution permission.
- Do not force every remote protocol into one lossy lowest-common-denominator schema.

## Open Questions Before Spec Writing

- What is the minimal common capability descriptor that Mister Smith should freeze?
- How should remote task lifecycles map onto local `workflow_id` and result surfaces?
- What capability metadata is mandatory for safe discovery?
- How should list-changed notifications and remote capability drift be projected to operators?
- Should the first slice prioritize MCP-to-local normalization or A2A-to-local normalization?

## Fixed Constraints Before Spec Writing

- Keep packet `027` about capability normalization and lifecycle mapping, not generic federation or
  mesh design.
- Consume packet `022`, `023`, and `024` contracts instead of redefining lifecycle, truth, or
  boundary security here.
- Keep discovery separate from execution permission and operator trust.
- Do not change protocol baselines without an explicit freeze-time recheck of the pinned official
  versions.

## Recommended Inputs For Future SpecKit Packet

Read these in order: repo routers -> upstream packets `022` through `024` -> bounded external
surface proof notes -> local capability seams -> pinned MCP and pinned A2A docs.

- `docs/direction.md`
- `docs/current-state.md`
- `docs/packet-prep/022-durable-workflow-core.md`
  - use to inherit lifecycle and durable identifier assumptions before mapping remote tasks
- `docs/packet-prep/023-runtime-truth-and-run-trace.md`
  - use to inherit proof-boundary and run-trace semantics before projecting remote capability use
- `docs/packet-prep/024-agent-boundary-security-hardening.md`
  - use to inherit least-privilege and discover-vs-execute policy before freezing interop
- `docs/research-output/analysis/2026-03-28-coordination-state-protocol-transfer-brief.md`
- `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`
- `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
  - use both notes together: `MS-77` for the discovery boundary already landed on `main`, packet
    `016` for delegated-ingress continuity and boundary provenance that must not be regressed
- use the MCP versioning page first, then use the exact MCP `2025-11-25` lifecycle, tools, and
  authorization pages linked above as the current dossier baseline unless the future packet
  deliberately records a version bump
- use the exact A2A `v0.3.0` specification, discovery, and task-lifecycle pages linked above as
  the current dossier baseline unless the future packet explicitly records a version bump
- `crates/mister-smith-agents/src/tool_bus.rs`
  - start from `CapabilityDescriptor`, `ToolPrincipal`, `register_mcp`, and
    `register_mcp_tool`
- `crates/mister-smith-mcp/src/server.rs`
  - start from `ToolCallRequest`, `CapabilityCatalogEntry`, and `capability_catalog`
- `crates/mister-smith-mcp/src/client.rs`
  - start from `ExternalCapabilityDescriptor` and `McpTool`
- `crates/mister-smith-mcp/src/compatibility.rs`
  - start from `describe_external_capabilities`, `build_smith_compatibility_server`, and the
    delegation-gated discovery tests
- `crates/mister-smith-core/src/autonomy.rs`
  - use the existing result-envelope and task-view types as the local target shape for any remote
    lifecycle mapping
- `crates/mister-smith-mcp/src/compatibility.rs`
  - specifically read
    `compatibility_server_accepts_valid_delegated_request`,
    `compatibility_server_lists_plain_tool_names`,
    `describe_external_capabilities_requires_discover_delegation`, and
    `describe_external_capabilities_returns_catalog_with_matching_discover_delegation`
- only after the repo-local lifecycle/result target is clear, use the MCP tasks utility page
  above when freezing cross-boundary task-status mapping
- `crates/mister-smith-agents/tests/tool_bus_tests.rs`
  - use for local capability-descriptor and delegated-action expectations
- only after the repo-local lifecycle, truth, and security contracts are clear, re-confirm the
  official docs and primary sources linked earlier
