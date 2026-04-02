# Contract: Capability Boundary Surface

**Spec**: [../spec.md](../spec.md) | **Plan**: [../plan.md](../plan.md)

## Design Goal

Freeze one least-privilege capability contract across ToolBus and MCP so discovery stays bounded,
execution stays action-bound, and delegated authority is checked before handler execution.

## Canonical Mapping

The contract for packet `024` is:

- every capability surface publishes one stable descriptor
- discovery and execution are separate actions on that descriptor
- descriptor match alone is not enough for execution
- execution requires the exact action binding, required scope, and revocation key
- bounded discovery may expose the descriptor and action requirements without granting execute
  authority

No packet `024` surface may collapse discover and execute into one ambient permission.

## Canonical Capability Shape

Example authoritative shape:

```json
{
  "descriptor_id": "tool:smith.describe_external_capabilities",
  "boundary_family": "mcp.tool",
  "discover_action": {
    "action_id": "tool:describe_external_capabilities#discover",
    "kind": "discover",
    "required_scope": null,
    "revocation_key": "tool:describe_external_capabilities#discover"
  },
  "execute_action": {
    "action_id": "tool:describe_external_capabilities#execute",
    "kind": "execute",
    "required_scope": "InvokeTool",
    "revocation_key": "tool:describe_external_capabilities#execute"
  }
}
```

Behavior:

- `discover_action` is for capability inspection only
- `execute_action` is for invocation only
- if a caller presents only discover authority and attempts execution, the boundary rejects it
- if a caller presents execute authority without a descriptor binding, the boundary rejects it
- if descriptor identifiers match but action identifiers or revocation keys do not, the boundary
  rejects the call before dispatch

## ToolBus Contract

The ToolBus contract remains grounded in:

- `crates/mister-smith-agents/src/tool_bus.rs`
- `crates/mister-smith-security/src/delegation.rs`

Expected behavior:

- `CapabilityDescriptor` continues to publish separate discover and execute actions
- the live enforcement path continues to require the exact delegated action needed by the boundary
- action-bound execute paths reject descriptorless delegated capabilities
- local and remote tool surfaces use the same least-privilege posture

## MCP Contract

The MCP contract remains grounded in:

- `crates/mister-smith-mcp/src/client.rs`
- `crates/mister-smith-mcp/src/bridge.rs`
- `crates/mister-smith-mcp/src/server.rs`
- `crates/mister-smith-mcp/src/compatibility.rs`

Expected behavior:

- `tools/list` and `describe_external_capabilities` stay on discover authority
- `tools/call` stays on execute authority
- `handle_tools_call` validates the exact expected boundary action before handler execution
- `describe_external_capabilities` exposes both `discover_action` and `execute_action`
- MCP client and bridge preserve the same two-action descriptor shape end to end
- bounded discovery continues without widening into ambient execute permission

## Protocol Source Rule

Packet `024` uses:

- MCP versioning pages
- MCP `2025-11-25` authorization pages

Packet `024` does **not** use latest or mixed MCP protocol pages as its frozen contract.
MCP security best-practices docs may inform hardening guidance, but they do not replace the
version-pinned protocol baseline.

## Relationship To Packet 016

Packet `024` preserves packet `016` continuity rules:

- accepted delegated task ingress remains baseline truth
- no new workflow-backed live rejection surface is invented here
- capability-boundary hardening must compose with that continuity rather than reopening it

## Deferred

This contract does not freeze:

- generic interop protocol design
- broader A2A mapping
- a new identity program
- compliance or operator-dashboard work beyond the named capability boundary
