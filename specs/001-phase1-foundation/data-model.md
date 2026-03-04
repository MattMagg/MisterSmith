# Data Model: Phase 1 Foundation Contracts

## Entity: CanonicalCoreTypeSet

- Purpose: Represents the authoritative Phase 1.1 type and error baseline.
- Attributes:
  - `id_types`: `AgentId`, `TaskId`, `MessageId`, `ToolId`
  - `priority_enum`: `MessagePriority` with explicit `0..4` discriminants
  - `agent_state_enum`: `AgentState`
  - `agent_availability_enum`: `AgentAvailability`
  - `agent_type_enum`: `AgentType`
  - `supervision_contracts`: `RestartPolicy`, `RestartScope`, `SupervisionStrategy`
  - `error_aliases`: `SystemError`, framework result alias
- Validation rules:
  - Exactly one canonical definition in designated source files.
  - No unresolved naming collision in active Phase 1 references.

## Entity: CoreTraitContractSet

- Purpose: Represents the authoritative Phase 1.2 trait interface baseline.
- Attributes:
  - `actor_trait`
  - `agent_trait`
  - `tool_trait`
  - `resource_trait`
  - `supervisor_trait`
  - `transport_trait`
- Validation rules:
  - Trait signatures remain stable and cross-reference consistent.
  - `Tool` signature consistency is enforced between canonical and integration references.

## Entity: ConfigurationContractSet

- Purpose: Represents typed Phase 1.3 configuration contract domains.
- Attributes:
  - `runtime_domain`
  - `agent_domain`
  - `transport_domain`
  - `security_domain`
  - `layering_model`: defaults -> file -> environment override
  - `validation_rules`: explicit failure semantics for malformed values
- Validation rules:
  - Deterministic precedence is documented.
  - Invalid configuration never resolves via silent fallback.

## Entity: Gate1ValidationEvidence

- Purpose: Binds requirements to measurable command outputs.
- Attributes:
  - `core_type_presence_check`
  - `restart_policy_collision_check`
  - `tool_signature_check`
  - `message_priority_crosscheck`
  - `core_compile_check`
  - `config_compile_check`
- Validation rules:
  - Every functional requirement maps to >=1 evidence command.

## Relationships

- `CanonicalCoreTypeSet` is a prerequisite for `CoreTraitContractSet` and `ConfigurationContractSet`.
- `Gate1ValidationEvidence` verifies all other entities.
