# Research: Capability Discovery And Interoperability

## Scope

This research file supports packet `027` as a scaffold. It uses:

- current repo truth from `docs/direction.md` and `docs/current-state.md`
- the current finished packet outputs for packets `022`, `023`, and `024`, plus the packet `027`
  scaffold itself
- `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`
- `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`
- `docs/research-output/analysis/2026-03-28-coordination-state-protocol-transfer-brief.md`
- pinned MCP `2025-11-25` and A2A `v0.3.0` primary sources

## Decision 1: Use the current packet outputs for packets 022 to 024 as upstream input

**Decision**: packet `027` reuses the current packet output language from packets `022`, `023`,
and `024` as upstream contract input and adds a mandatory refresh gate before implementation.

**Rationale**: the user wants packet `027` authored now for scaffolding. A refresh gate preserves
speed without pretending those upstream packet outputs will never change again.

**Alternatives considered**:

- wait for packets `022`, `023`, and `024` before writing packet `027`
- restate all of their contracts inside packet `027`

## Decision 2: Freeze A2A `v0.3.0` as the first new interop slice

**Decision**: packet `027` freezes A2A `v0.3.0` discovery and lifecycle mapping as the first new
interop slice.

**Rationale**: the A2A discovery guidance identifies the Agent Card as the standard discovery
surface and lists the core fields needed for remote suitability checks: identity, service URL,
capabilities, authentication, and skills. The A2A specification also publishes a stable `TaskState`
set for remote task lifecycles. That makes A2A the cleanest first remote slice once MCP has
already been used for bounded discovery on the repo side.

**Alternatives considered**:

- freeze only discovery and defer lifecycle mapping
- make MCP tasks the first new slice instead of A2A

## Decision 3: Keep MCP `2025-11-25` pinned as an input and policy boundary

**Decision**: packet `027` keeps MCP pinned to `2025-11-25` and uses it as a normalization and
boundary input, not as the first new interoperability slice.

**Rationale**: the MCP tools spec declares `tools.listChanged` for tool-list drift, the lifecycle
spec fixes the `initialize` plus `initialized` handshake boundary, the authorization spec grounds
OAuth metadata discovery, and the tasks utility fixes one explicit task lifecycle for task-augmented
requests. These are important comparators and boundary inputs, but the repo already has bounded MCP
discovery from `MS-77`.

**Alternatives considered**:

- move to unpinned or mixed-version MCP pages
- make MCP lifecycle mapping the first new slice and postpone A2A

## Decision 4: The shared descriptor must carry source plus permission reference, not permission itself

**Decision**: the normalized descriptor carries source identity, lifecycle hints, schema or mode
information, and a separate permission reference.

**Rationale**: the A2A Agent Card includes capability and skill metadata plus authentication
requirements, but those fields describe what the remote agent says it can do. The repo research and
packet `024` boundary guidance both require discovery to stay separate from execution authority.

**Alternatives considered**:

- use one lowest-common-denominator discovery record with no source-specific detail
- treat remote skill discovery as implicit permission to execute

## Decision 5: The first lifecycle bridge should stay explicit and loss-aware

**Decision**: packet `027` defines an explicit A2A-to-Mister-Smith lifecycle binding instead of
pretending the remote states match local workflow states one-to-one.

**Rationale**: A2A `v0.3.0` defines task states including `submitted`, `working`, `input-required`,
`completed`, `canceled`, `failed`, `rejected`, `auth-required`, and `unknown`. The MCP tasks
utility uses a narrower status model centered on `working`, `input_required`, `completed`,
`failed`, and `cancelled`. Mister Smith already has its own workflow and proof-boundary language.
The bridge therefore needs explicit mapping rules and explicit unsupported-state handling.

**Alternatives considered**:

- force all remote states into one lossy local status with no provenance
- delay lifecycle mapping entirely and freeze discovery only

## Pinned Primary Sources

- MCP lifecycle: [modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle)
- MCP tools: [modelcontextprotocol.io/specification/2025-11-25/server/tools](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
- MCP authorization: [modelcontextprotocol.io/specification/2025-11-25/basic/authorization](https://modelcontextprotocol.io/specification/2025-11-25/basic/authorization)
- MCP tasks utility: [modelcontextprotocol.io/specification/2025-11-25/basic/utilities/tasks](https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/tasks)
- A2A agent discovery: [a2a-protocol.org/v0.3.0/topics/agent-discovery](https://a2a-protocol.org/v0.3.0/topics/agent-discovery/)
- A2A task lifecycle: [a2a-protocol.org/v0.3.0/topics/life-of-a-task](https://a2a-protocol.org/v0.3.0/topics/life-of-a-task/)
- A2A specification: [a2a-protocol.org/v0.3.0/specification](https://a2a-protocol.org/v0.3.0/specification/)

## Repo Anchors

- `crates/mister-smith-agents/src/tool_bus.rs`
- `crates/mister-smith-mcp/src/server.rs`
- `crates/mister-smith-mcp/src/client.rs`
- `crates/mister-smith-mcp/src/compatibility.rs`
- `crates/mister-smith-core/src/autonomy.rs`
- `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`
- `docs/plans/2026-03-20-packet-016-external-agent-boundary-continuity-evaluation.md`

## Open Follow-On For The Refresh Gate

- confirm the final packet `022` lifecycle and identifier vocabulary still matches the packet `027`
  lifecycle contract
- confirm the final packet `023` proof-boundary language still matches the packet `027`
  provenance contract
- confirm the final packet `024` security posture still matches the packet `027`
  discover-versus-execute boundary
