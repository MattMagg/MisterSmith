# Data Model: Capability Discovery And Interoperability

## 1. NormalizedCapabilityDescriptor

Represents one discovered capability after source-specific discovery data has been normalized.

### Permission Fields

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `descriptor_id` | string | Stable capability identifier used inside Mister Smith. |
| `source` | enum | Origin of the discovery data: `local_toolbus`, `mcp_2025_11_25`, or `a2a_v0_3_0`. |
| `source_reference` | string | Source-native identifier such as a local tool descriptor id, MCP capability id, or A2A skill id. |
| `title` | string | Human-readable name for the capability. |
| `description` | string | Human-readable summary of the capability. |
| `service_endpoint` | optional string | Addressable remote endpoint when present. |
| `input_modes` | string list | Accepted input content or schema modes. |
| `output_modes` | string list | Produced output content or schema modes. |
| `lifecycle_hint` | optional string | Source-native hint that the capability may create or update a long-running task. |
| `permission_reference` | optional object | Separate pointer to the policy or delegation surface required for execution. |
| `provenance_note` | string | Operator-visible explanation of where the descriptor came from. |

### Validation Rules

- `source` is mandatory and immutable after normalization.
- `permission_reference` is descriptive only and must not be interpreted as granted authority.
- remote descriptors may omit `service_endpoint` only when the source is not directly callable.

## 2. CapabilitySource

Typed source classification for discovery inputs.

### Allowed Values

| Value | Meaning |
| ----- | ------- |
| `local_toolbus` | Capability discovered from Mister Smith's local ToolBus registry. |
| `mcp_2025_11_25` | Capability discovered from a pinned MCP `2025-11-25` surface. |
| `a2a_v0_3_0` | Capability discovered from an A2A `v0.3.0` Agent Card. |

## 3. PermissionReference

Describes where execution permission must be checked.

### Lifecycle Fields

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `action_id` | optional string | Source-specific action identifier when available. |
| `required_scope` | optional string | Delegation or auth scope needed for execution. |
| `revocation_key` | optional string | Revocation or policy key when the source provides one. |
| `authority_surface` | string | Boundary where execution permission is enforced. |

### Rules

- `PermissionReference` may exist even when execution is not currently allowed.
- absence of `PermissionReference` does not imply ambient permission.

## 4. RemoteTaskLifecycleBinding

Maps one remote lifecycle model into Mister Smith workflow, result, and autonomy views.

### Provenance Fields

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `protocol` | enum | Protocol version for the remote lifecycle: `a2a_v0_3_0`. |
| `remote_state` | string | Source-native remote task state. |
| `local_workflow_state` | string | Closest Mister Smith workflow state projection. |
| `proof_boundary_label` | string | Text that explains what is and is not proven by this mapping. |
| `input_required` | boolean | Whether the local view should represent a blocked-on-input posture. |
| `terminal` | boolean | Whether the remote state is terminal. |
| `operator_rationale` | string | Operator-visible explanation for the mapping. |

### Transition Notes

- packet `027` treats the A2A states `submitted`, `working`, `input-required`, `completed`,
  `canceled`, `failed`, `rejected`, `auth-required`, and `unknown` as source-native values
- mapping remains explicit and loss-aware; unsupported mappings must be called out directly

## 5. RemoteCapabilityUseProvenance

Operator-facing projection that records discovery and lifecycle context together.

### Fields

| Field | Type | Meaning |
| ----- | ---- | ------- |
| `descriptor_id` | string | Normalized capability identifier. |
| `source` | enum | Discovery source that produced the descriptor. |
| `lifecycle_binding_id` | optional string | Reference to the lifecycle binding used for the remote task. |
| `authority_surface` | string | Where execution permission still had to be enforced. |
| `continuity_reference` | optional string | Reference to packet `016`-style continuity or provenance context, when applicable. |
| `display_summary` | string | Short operator-facing explanation. |

## Relationships

- one `CapabilitySource` can feed many `NormalizedCapabilityDescriptor` records
- one `NormalizedCapabilityDescriptor` may reference zero or one `PermissionReference`
- one `NormalizedCapabilityDescriptor` may participate in zero or many `RemoteTaskLifecycleBinding`
  records over time
- one `RemoteCapabilityUseProvenance` record joins one normalized descriptor with zero or one
  lifecycle binding
