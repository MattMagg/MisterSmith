# Phase 1 — Foundation

## Purpose / Scope

Establish the single canonical type and trait contract surface for all downstream phases.

### In scope

- Canonical IDs and core enums (`AgentId`, `TaskId`, `MessageId`, `ToolId`, `AgentState`, `AgentAvailability`, `AgentType`, `MessagePriority`)
- Canonical supervision model (`RestartPolicy`, `RestartScope`, `SupervisionStrategy`, `EscalationPolicy`, `BackoffStrategy`)
- Core trait signatures (`Actor`, `Agent`, `Tool`, `Resource`, `Supervisor`, `Transport`)
- Error hierarchy and framework-wide `Result` alias patterns
- Config schema structure and load-time validation contracts

### Out of scope

- Runtime startup/shutdown behavior
- Actor execution loops
- External transports and persistence I/O

## Inputs / Depends-On

### Upstream phases

- None

### Authoritative spec references

- `spec/core-architecture/type-definitions.md` (canonical types)
- `spec/core-architecture/module-organization-type-system.md` (canonical Tool/Agent/Resource trait signatures)
- `spec/core-architecture/runtime-and-errors.md`
- `spec/core-architecture/component-architecture.md`
- `spec/core-architecture/integration-contracts.md`
- `spec/core-architecture/implementation-config.md`
- `spec/core-architecture/async-patterns.md` (Actor trait definition, Section 3)
- `spec/core-architecture/system-integration.md` (Tool trait cross-check)
- `spec/operations/configuration-management.md`
- `VERSION_REFERENCE.md`
- `VALIDATION_REPORT.md`

## Outputs / Produces

- Canonical type registry consumed by every phase
- Canonical trait contract set for implementation crates
- Canonical naming split between lifecycle (`AgentState`) and transport presence (`AgentAvailability`)
- Configuration contract baseline for runtime/security/transport/persistence

## Gate Criteria

### Done means

- Exactly one canonical definition exists for each core type in `type-definitions.md`
- `MessagePriority` is consistently 5 levels with discriminants `0..=4`
- No conflicting `RestartPolicy` type names (enum vs struct) in active specs
- Tool trait signatures are consistent between architecture integration docs
- `EscalationPolicy` and `BackoffStrategy` have canonical definitions in `type-definitions.md` (required by `SupervisionStrategy` struct)
- `Agent` trait canonical signature is established (resolve divergence between `module-organization-type-system.md` and `integration-contracts.md`)

### How to validate

- `rg -n "pub enum AgentState|pub enum AgentAvailability|pub enum MessagePriority|pub enum AgentType|pub enum RestartPolicy|pub enum RestartScope" spec/core-architecture/type-definitions.md`
- `rg -n "pub struct AgentId|pub struct TaskId|pub struct MessageId|pub struct ToolId" spec/core-architecture/type-definitions.md`
- `rg -n "pub struct SupervisionStrategy|pub enum EscalationPolicy|pub enum BackoffStrategy" spec/core-architecture/type-definitions.md`
- `rg -n "pub enum SystemError" spec/core-architecture/type-definitions.md`
- `rg -n "pub struct RestartPolicy\\b|pub enum RestartPolicy\\b" spec/` (full spec search — catches operations/process-management conflict)
- `rg -n "pub trait Tool" spec/core-architecture/module-organization-type-system.md spec/core-architecture/system-integration.md spec/core-architecture/async-patterns.md`
- `rg -n "pub trait Agent" spec/core-architecture/module-organization-type-system.md spec/core-architecture/integration-contracts.md`
- `rg -n "MessagePriority" spec/testing/test-schemas.md spec/data-management/message-schemas.md spec/transport/nats-transport.md`

## Official-Doc Best Practices

- Prefer UUID newtypes over raw strings for domain identity boundaries in Rust APIs ([Rust newtype pattern](https://doc.rust-lang.org/book/ch20-03-advanced-types.html#using-the-newtype-pattern-for-type-safety-and-abstraction)).
- Use `thiserror` for domain errors and derive-based, explicit conversion paths ([docs.rs/thiserror 1.0.69](https://docs.rs/thiserror/1.0.69/thiserror/)).
- Use serde derive + strongly typed enums for wire-safe schema definitions ([Serde derive](https://serde.rs/derive.html)).

## Known Risks / Unknowns

- **Risk**: Legacy docs still include non-canonical illustrative type snippets.
  - **Follow-up**: Keep references pointing to canonical Phase 1.1 section when illustrative snippets cannot be removed immediately.
- **Risk**: Future phase docs may re-introduce name collisions (`AgentState`, `RestartPolicy`, `Tool` trait).
  - **Follow-up**: Add grep checks above to every roadmap reconciliation pass.
- **Risk**: `EscalationPolicy` has 3+ conflicting definitions across spec files (`async-patterns.md`, `agent-orchestration.md`, `process-management-specifications.md`) with different variant sets.
  - **Follow-up**: Reconcile into a single canonical enum in `type-definitions.md`. The OTP-level variants (Escalate, LogAndIgnore, Shutdown) and process-management variants (RestartProcess, NotifyOperator, etc.) may need namespace isolation.
- **Risk**: `Agent` trait is defined differently in `module-organization-type-system.md` (Tool-based extension) vs. `integration-contracts.md` (lifecycle-based). Both are referenced by Phase 1.
  - **Follow-up**: Designate `module-organization-type-system.md` as canonical per ROADMAP Phase 1.2 and update `integration-contracts.md` to reference or extend it.
- **Risk**: `configuration-management.md` contains Claude CLI-specific config blocks (`[worker.claude_cli]`, `MISTER_SMITH_CLAUDE_*` env vars) which violate model-agnostic principle.
  - **Follow-up**: Generalize to LLM-agnostic configuration before implementation.
