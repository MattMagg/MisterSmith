# Phase 1 Contract Baseline

## 1. Core Type Contracts

Required identifiers and enums:

- IDs: `AgentId`, `TaskId`, `MessageId`, `ToolId`
- Enums: `MessagePriority`, `AgentState`, `AgentAvailability`, `AgentType`
- Supervision: `RestartPolicy`, `RestartScope`, `SupervisionStrategy`
- Error/result: `SystemError` + framework result alias

Contract rules:

- `MessagePriority` MUST preserve `Critical=0`, `High=1`, `Normal=2`, `Low=3`, `Bulk=4`.
- `AgentState` MUST describe lifecycle state; `AgentAvailability` MUST describe transport/runtime presence.
- Active Phase 1 references MUST not define conflicting names for the same contract concept.

## 2. Core Trait Contracts

Required trait interfaces:

- `Actor`
- `Agent`
- `Tool`
- `Resource`
- `Supervisor`
- `Transport`

Contract rules:

- Trait signatures are interface contracts only for Phase 1.
- `Tool` signature fields (`execute`, `schema`, `capabilities`, `tool_id`, `version`) MUST be consistent between canonical and integration references.

## 3. Configuration Contracts

Required configuration domains:

- Runtime domain
- Agent domain
- Transport domain
- Security domain

Contract rules:

- Domain coverage is mandatory; exact struct-name parity across all docs is not mandatory.
- Layering order MUST be deterministic: defaults -> file -> environment overrides.
- Validation failures MUST be explicit and actionable.

## 4. Governance and Evidence

Required evidence command families:

- Type presence and collision checks (`rg`)
- Trait consistency checks (`rg`)
- Compile validation (`cargo build -p mister-smith-core`, `cargo build -p mister-smith-config`)
- Documentation quality check (`markdownlint` on feature docs)

Legacy snippet policy:

- Legacy illustrative snippets are allowed only if they explicitly point to canonical definitions.
