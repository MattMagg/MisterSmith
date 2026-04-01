# Contract: Capability Normalization

## Purpose

Define the minimum normalized discovery contract for packet `027`.

## Version And Scope

- packet scope: `027-capability-discovery-and-interoperability`
- MCP baseline: `2025-11-25`
- A2A baseline: `v0.3.0`
- this contract is scaffold-only until packet `027` is refreshed against the completed packet
  `022`, `023`, and `024` outputs

## Source Inputs

| Source | Source-native discovery object | Required normalized inputs |
| ------ | ------------------------------ | -------------------------- |
| Local ToolBus | `CapabilityDescriptor` and related delegated actions | descriptor id, title, description, actions, local owner when present |
| MCP `2025-11-25` | MCP catalog entry and tool metadata | external name, capability descriptor, boundary action, scope, revocation details |
| A2A `v0.3.0` | Agent Card and `AgentSkill` entries | protocol version, service URL, capabilities, authentication requirements, skill id, name, description, input modes, output modes |

## Required Normalized Fields

Every normalized descriptor MUST include:

- `descriptor_id`
- `source`
- `source_reference`
- `title`
- `description`
- `input_modes`
- `output_modes`
- `provenance_note`

The following are conditional:

- `service_endpoint`
- `lifecycle_hint`
- `permission_reference`

## Separation Rules

- discovery data MUST NOT be treated as execution permission
- `permission_reference` MUST remain a pointer to a later policy or delegation check
- source-native authentication or security metadata MAY inform policy but MUST NOT be collapsed
  into "trusted to execute"

## Drift Rules

- protocol references inside packet `027` MUST stay pinned to MCP `2025-11-25` and A2A `v0.3.0`
- if a later revision changes either pin, packet `027` MUST record that change explicitly instead
  of mixing sources silently
- list-change or card-change notifications MAY update discovery state, but they MUST NOT update
  execution authority automatically

## Deferred Items

- generic multi-protocol registry APIs
- dynamic trust scoring
- live remote execution authorization
