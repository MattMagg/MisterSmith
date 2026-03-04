# Implementation Plan: Phase 1 — Foundation

**Date**: 2026-03-04 | **Phase**: `plans/roadmap-phases/phase-1-foundation.md`
**Produces**: `mister-smith-core` crate (1.1 types + 1.2 traits), `mister-smith-config` crate (1.3 configuration)

## Summary

Phase 1 establishes the canonical type and trait contract surface for the entire Mister Smith framework. It produces two crates with zero runtime behavior: `mister-smith-core` (ID newtypes, enums, error hierarchy, trait definitions) and `mister-smith-config` (typed configuration loading, validation, environment overlay). Every downstream phase imports from these crates. Getting the type design right is critical — changes here cascade everywhere.

## Technical Context

**Language/Version**: Rust 1.88.0 (MSRV, driven by async-nats 0.46.0)
**Primary Dependencies**: thiserror 1.0.69, serde 1.0.228, uuid 1.11.0, async-trait 0.1.83
**Config Dependencies**: toml 0.8.x, serde 1.0.228, thiserror 1.0.69
**Storage**: N/A (no persistence in Phase 1)
**Testing**: `cargo test` (unit tests only — no async runtime needed for most Phase 1 code)
**Target Platform**: Linux server (Kubernetes deployment), macOS development
**Project Type**: Library crates (Cargo workspace)
**Constraints**: No external services, no async runtime, no I/O. Pure types, traits, and error definitions.

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Canonical Single Source | PASS | `type-definitions.md` is canonical for types; `module-organization-type-system.md` for traits. This plan lists all canonical sources. |
| II. Spec-First Design | PASS | Every type and trait traces to a spec file. Trace table included below. |
| III. Phase-Gated Build Order | PASS | Phase 1 has no upstream dependencies. Gate criteria defined in `phase-1-foundation.md`. |
| IV. Model-Agnostic Architecture | PASS | No LLM-specific types or traits in Phase 1. `configuration-management.md` contains Claude CLI-specific config blocks (`[worker.claude_cli]`) that must be excluded from implementation. |
| V. Erlang/OTP-Style Fault Tolerance | PASS | Supervision types (RestartPolicy, SupervisionStrategy) are defined here; implementation deferred to Phase 3. |
| VI. Evidence-Based Validation | PASS | Gate checks are grep/compilation commands, not assertions. |
| VII. Explicit Dependency Management | PASS | VERSION_REFERENCE.md is authoritative. All versions pinned below. |

## Crate Structure

```text
mister-smith/
├── Cargo.toml                          # Workspace root
├── crates/
│   ├── mister-smith-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # Crate root, re-exports
│   │       ├── ids.rs                  # AgentId, TaskId, MessageId, ToolId newtypes
│   │       ├── enums.rs                # AgentState, AgentAvailability, AgentType, MessagePriority
│   │       ├── supervision.rs          # RestartPolicy, RestartScope, SupervisionStrategy, EscalationPolicy, BackoffStrategy
│   │       ├── error.rs                # SystemError, sub-errors, ErrorSeverity, RecoveryStrategy, FrameworkResult
│   │       └── traits.rs              # Actor, Agent, Tool, Resource, Supervisor, Transport
│   └── mister-smith-config/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                  # Crate root, re-exports
│           ├── types.rs                # RuntimeConfig, SupervisionConfig, MonitoringConfig, AgentConfig, TransportConfig, SecurityConfig
│           ├── loader.rs               # TOML/YAML parsing, env overlay, file discovery
│           ├── validation.rs           # Load-time validation
│           └── error.rs                # ConfigValidationError
```

## Dependency Versions (from VERSION_REFERENCE.md)

### mister-smith-core

| Crate | Version | Purpose |
|-------|---------|---------|
| thiserror | 1.0.69 | Error derives (staying on 1.x per spec decision) |
| serde | 1.0.228 | Serialize/Deserialize derives |
| uuid | 1.11.0 | UUID v4 for ID newtypes (features: v4, serde) |
| async-trait | 0.1.83 | dyn-compatible async trait methods |

### mister-smith-config

| Crate | Version | Purpose |
|-------|---------|---------|
| serde | 1.0.228 | Deserialization of config structs |
| toml | 0.8.x | TOML config file parsing |
| thiserror | 1.0.69 | Config error types |
| mister-smith-core | (workspace) | Core types referenced by config structs |

**Note on `async-trait`**: With MSRV 1.88.0, native async fn in traits is available (Rust 1.75+). However, `async-trait` is required for any trait intended for dynamic dispatch (`dyn Trait`). Phase 1 trait definitions use `#[async_trait]` because `Actor`, `Agent`, `Tool`, `Resource`, `Supervisor`, and `Transport` are all designed to be used as trait objects.

## Entities to Implement

### 1.1 Core Types (mister-smith-core)

**Source**: `spec/core-architecture/type-definitions.md` (Canonical Core Types section)

#### ID Newtypes

| Type | Inner Type | Derives | Spec Source |
|------|-----------|---------|-------------|
| `AgentId` | `Uuid` | Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize | type-definitions.md:59 |
| `TaskId` | `Uuid` | Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize | type-definitions.md:62 |
| `MessageId` | `Uuid` | Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize | type-definitions.md:65 |
| `ToolId` | `Uuid` | Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize | type-definitions.md:68 |

Each ID newtype must implement:
- `new()` constructor (generates UUID v4)
- `from_uuid(uuid: Uuid)` constructor
- `Display` trait (delegates to inner UUID)
- `Default` should NOT be derived (UUIDs should be explicitly created)

#### Core Enums

| Type | Variants | Repr | Spec Source |
|------|----------|------|-------------|
| `MessagePriority` | Critical=0, High=1, Normal=2, Low=3, Bulk=4 | `#[repr(u8)]` | type-definitions.md:71-78 |
| `AgentState` | Initializing, Running, Paused, Stopping, Terminated, Error, Restarting | — | type-definitions.md:81-90 |
| `AgentAvailability` | Idle, Busy, Error, Offline, Starting, Stopping | — | type-definitions.md:93-101 |
| `AgentType` | Supervisor, Worker, Coordinator, Monitor, Planner, Executor, Critic, Router, Memory | — | type-definitions.md:103-114 |

**Key semantic distinction**: `AgentState` is the lifecycle state machine (Phase 7 agent lifecycle). `AgentAvailability` is the transport/runtime availability signal for status channels and heartbeats.

#### Supervision Types

| Type | Kind | Fields/Variants | Spec Source |
|------|------|----------------|-------------|
| `RestartPolicy` | enum | OneForOne, OneForAll, RestForOne | type-definitions.md:117-121 |
| `RestartScope` | enum | Permanent, Transient, Temporary | type-definitions.md:123-128 |
| `SupervisionStrategy` | struct | restart_policy, max_failures, failure_window, escalation_policy, backoff_strategy | type-definitions.md:130-137 |
| `EscalationPolicy` | enum | **NEEDS RECONCILIATION** (see Key Decisions below) | type-definitions.md (not yet canonical) |
| `BackoffStrategy` | enum | **NEEDS RECONCILIATION** (see Key Decisions below) | type-definitions.md (not yet canonical) |

### 1.1 Error Hierarchy (mister-smith-core)

**Source**: `spec/core-architecture/runtime-and-errors.md` (Core Error Types section)

#### Top-Level Error

| Type | Variants | Spec Source |
|------|----------|-------------|
| `SystemError` | Runtime, Supervision, Configuration, Resource, Network, Persistence, Actor, Task, Stream, Event, Tool | runtime-and-errors.md:72-95 |

#### Sub-Error Types

| Type | Variants | Spec Source |
|------|----------|-------------|
| `RuntimeError` | BuildFailed, StartupFailed, ShutdownFailed, ConfigurationInvalid | runtime-and-errors.md:98-107 |
| `SupervisionError` | StrategyFailed, RestartFailed, EscalationFailed, RestartLimitExceeded, TreeCorrupted | runtime-and-errors.md:110-121 |
| `ActorError` | StartupFailed, MailboxFull, ActorStopped, SystemStopped, AskTimeout, DeserializationFailed, MessageHandlingFailed | runtime-and-errors.md:124-139 |
| `TaskError` | ExecutionFailed, TimedOut, TaskCancelled, ExecutorShutdown, QueueFull, SerializationFailed | runtime-and-errors.md:142-155 |
| `StreamError` | ProcessingFailed, ProcessorFailed, SinkFull, SinkBlocked, StreamEnded, BackpressureFailed | runtime-and-errors.md:158-171 |
| `EventError` | HandlerFailed, SerializationFailed, PublicationFailed, SubscriptionFailed, StoreFailed | runtime-and-errors.md:174-185 |
| `ToolError` | ExecutionFailed, NotFound, AccessDenied, ParameterValidationFailed, Timeout | runtime-and-errors.md:188-199 |
| `ConfigError` | ValidationFailed, FileNotFound, ParseFailed, MergeFailed | runtime-and-errors.md:202-211 |
| `ResourceError` | AcquisitionFailed, PoolExhausted, HealthCheckFailed, CleanupFailed | runtime-and-errors.md:214-223 |
| `NetworkError` | ConnectionFailed, Timeout, ProtocolError | runtime-and-errors.md:226-233 |
| `PersistenceError` | DatabaseFailed, SerializationFailed, DataCorrupted | runtime-and-errors.md:236-243 |

#### Error Support Types

| Type | Kind | Spec Source |
|------|------|-------------|
| `ErrorSeverity` | enum: Low, Medium, High, Critical | runtime-and-errors.md:246-252 |
| `RecoveryStrategy` | enum: Retry, Restart, Escalate, Reload, CircuitBreaker, Failover, Ignore | runtime-and-errors.md:254-263 |
| `FrameworkResult<T>` | type alias: `Result<T, SystemError>` | type-definitions.md:139 |

### 1.2 Core Traits (mister-smith-core)

**Source**: `spec/core-architecture/module-organization-type-system.md` (Section 2.1, canonical per ROADMAP Phase 1.2)

| Trait | Key Methods | Associated Types | Spec Source |
|-------|------------|-----------------|-------------|
| `Actor` | handle_message, pre_start, post_stop, actor_id | Message, State, Error | module-organization-type-system.md:367-381 |
| `Agent` | process, role, context, initialize, dependencies, create_with_dependencies | Context, Error (extends Tool) | module-organization-type-system.md:407-424 |
| `Tool` | execute, schema, capabilities, tool_id, version | — | module-organization-type-system.md:397-404 |
| `Resource` | acquire, release, is_healthy, health_check, resource_id | Config, Error | module-organization-type-system.md:430-442 |
| `Supervisor` | supervise, supervision_strategy, restart_policy, escalation_policy, supervisor_id | Child, Error | module-organization-type-system.md:384-394 |
| `Transport` | send, broadcast, subscribe, request_response, connect, disconnect, connection_status | Message, Subscription, ConnectionInfo | integration-contracts.md:200-214 |

**Trait design note**: `Agent` extends `Tool` in the canonical spec (`pub trait Agent: Tool + Send + Sync + 'static`). This means any Agent can be used as a Tool — this is the agent-as-tool pattern.

### 1.3 Configuration Types (mister-smith-config)

**Source**: `spec/core-architecture/implementation-config.md`

| Struct | Key Fields | Spec Source |
|--------|-----------|-------------|
| `AgentConfig` | runtime, supervision, monitoring | implementation-config.md:29-38 |
| `RuntimeConfig` | worker_threads, blocking_threads, max_memory | implementation-config.md:41-56 |
| `SupervisionConfig` | max_restart_attempts, restart_window, escalation_timeout | implementation-config.md:58-74 |
| `MonitoringConfig` | health_check_interval, metrics_export_interval, log_level | implementation-config.md:76-92 |

**Additional config structs** (from `configuration-management.md`):

| Struct | Key Fields | Spec Source |
|--------|-----------|-------------|
| `TransportConfig` | NATS, HTTP, gRPC connection settings | configuration-management.md:2.1.2 |
| `SecurityConfig` | enabled, encryption_enabled, tls_enabled, auth_required | configuration-management.md:2.2.1 |

**Config error type** (from implementation-config.md):

| Type | Variants | Spec Source |
|------|----------|-------------|
| `ConfigValidationError` | ValidationError, MissingField, InvalidValue, EnvVarError, FileError, DeserializationError | implementation-config.md:342-361 |

## Key Decisions Before Implementation

### Decision 1: EscalationPolicy Canonical Definition

**Problem**: `EscalationPolicy` has 4 conflicting definitions across spec files:
- `async-patterns.md`: `Escalate, LogAndIgnore, Shutdown`
- `agent-lifecycle.md` / `agent-orchestration.md`: `Terminate, Restart, Escalate`
- `process-management-specifications.md`: `RestartProcess, RestartService, NotifyOperator, FailoverToSecondary, GracefulShutdown`

**Recommendation**: The `agent-lifecycle.md` / `agent-orchestration.md` definition (`Terminate, Restart, Escalate`) is the most appropriate for the supervision system because:
1. It aligns with OTP semantics (escalate = propagate to parent supervisor)
2. The `process-management-specifications.md` variants are operational concerns (Phase 8), not supervision concerns
3. The `async-patterns.md` variants are a subset that can be covered by `Terminate` (= Shutdown) and `Escalate`

**Action**: Canonicalize as `{ Terminate, Restart, Escalate }` in `type-definitions.md` before implementation. Add `LogAndIgnore` as a fourth variant if needed for the "swallow the error" case.

### Decision 2: BackoffStrategy Canonical Definition

**Problem**: `BackoffStrategy` has 2 definitions:
- `agent-lifecycle.md`: `Fixed(Duration), Exponential { initial, max, multiplier }, Linear { initial, increment }`
- `integration-patterns.md`: `Fixed { interval }, Linear { initial, increment }, Exponential { initial, factor, max }, Custom(fn)`

**Recommendation**: Use the `agent-lifecycle.md` definition. The `Custom(fn)` variant from `integration-patterns.md` is not serializable and is better handled through a separate configuration mechanism.

**Action**: Canonicalize as `{ Fixed(Duration), Exponential { initial, max, multiplier }, Linear { initial, increment } }` in `type-definitions.md`.

### Decision 3: SystemError Shape — Flat Strings vs Nested #[from] Enums

**Problem**: `type-definitions.md` (Phase 1.1 canonical section) defines `SystemError` with flat `String` variants:
```rust
pub enum SystemError {
    Configuration(String),
    Runtime(String),
    Transport(String),
    Security(String),
    Persistence(String),
}
```

But `runtime-and-errors.md` defines a richer hierarchy with `#[from]` conversions from typed sub-errors (`RuntimeError`, `SupervisionError`, `ActorError`, etc.).

**Recommendation**: Use the `runtime-and-errors.md` definition with typed sub-errors. The flat-string version loses type information and makes error handling less precise. The richer hierarchy also enables the `severity()` and `recovery_strategy()` methods on `SystemError`.

**Action**: Implement the `runtime-and-errors.md` version. Update `type-definitions.md` canonical section to match.

### Decision 4: Agent Trait — Tool Extension vs Standalone

**Problem**: `module-organization-type-system.md` defines `Agent: Tool + Send + Sync + 'static` (Agent extends Tool). `integration-contracts.md` defines a standalone `Agent` trait with lifecycle-based methods. Both are referenced by Phase 1.

**Recommendation**: Per ROADMAP Phase 1.2, `module-organization-type-system.md` is canonical. Keep `Agent: Tool`.

**Action**: Use the `module-organization-type-system.md` definition. Note that `integration-contracts.md`'s lifecycle methods are Phase 7 concerns and will be added when the agent lifecycle is implemented.

### Decision 5: Config Validation Library

**Problem**: `implementation-config.md` references `validator` (0.20), `schemars` (0.8), and `jsonschema` (0.18 — now 0.42.2 with breaking API changes). These are heavy dependencies for Phase 1.

**Recommendation**: For Phase 1, implement basic validation with hand-written `validate()` methods on config structs. Defer `validator` / `schemars` / `jsonschema` to a later phase when the config system matures. This keeps the dependency footprint minimal and avoids the jsonschema migration problem.

**Action**: Implement `validate() -> Result<(), ConfigValidationError>` on each config struct manually. Add `validator`/`schemars` as a future enhancement.

## Cross-References to Existing Plans

| Plan | Covers | Overlap with Phase 1 |
|------|--------|---------------------|
| `plans/batch1-core-architecture/agent05-module-organization-implementation.md` | Workspace structure, module hierarchy | Workspace Cargo.toml, feature flags, module layout. **Caution**: agent05 uses outdated versions (Rust 1.75, tokio 1.45) and a monolithic crate structure. Phase 1 uses the ROADMAP's multi-crate layout. |
| `plans/batch1-core-architecture/agent06-type-system-implementation.md` | Type definitions, error hierarchy, newtypes | Core types and error hierarchy. **Caution**: agent06 uses raw `type AgentId = Uuid` aliases instead of the canonical newtype pattern (`pub struct AgentId(pub Uuid)`). Phase 1 uses newtypes per `type-definitions.md`. |
| `plans/batch1-core-architecture/agent02-component-architecture-implementation.md` | Component hierarchy, trait relationships | Trait definitions. Phase 1 extracts trait signatures only; agent02 covers full implementations. |

## Gate Criteria (from phase-1-foundation.md)

### Done Means

1. Exactly one canonical definition exists for each core type
2. `MessagePriority` is consistently 5 levels with discriminants `0..=4`
3. No conflicting `RestartPolicy` type names in active specs
4. Tool trait signatures are consistent between architecture integration docs
5. `EscalationPolicy` and `BackoffStrategy` have canonical definitions
6. `Agent` trait canonical signature is established

### How to Validate

```bash
# Core types compile
cargo build -p mister-smith-core

# Config loads and validates
cargo build -p mister-smith-config

# Tests pass
cargo test -p mister-smith-core
cargo test -p mister-smith-config

# No conflicting definitions in spec (informational)
rg -n "pub enum AgentState|pub enum AgentAvailability|pub enum MessagePriority" spec/core-architecture/type-definitions.md
rg -n "pub struct AgentId|pub struct TaskId|pub struct MessageId|pub struct ToolId" spec/core-architecture/type-definitions.md
rg -n "pub struct SupervisionStrategy|pub enum EscalationPolicy|pub enum BackoffStrategy" spec/core-architecture/type-definitions.md
```

## Risk Summary

| Risk | Severity | Mitigation |
|------|----------|------------|
| EscalationPolicy has 4 conflicting definitions | High | Reconcile before implementation (Decision 1) |
| BackoffStrategy has 2 conflicting definitions | Medium | Reconcile before implementation (Decision 2) |
| SystemError flat vs nested shape | Medium | Use nested hierarchy (Decision 3) |
| Agent trait extension ambiguity | Medium | Use module-organization-type-system.md as canonical (Decision 4) |
| Config validation library churn (jsonschema 0.18 vs 0.42) | Low | Defer heavy validation deps; use manual validation (Decision 5) |
| Legacy docs with non-canonical type snippets | Low | Annotate as illustrative; canonical section takes precedence |
| `configuration-management.md` contains Claude CLI-specific config blocks | Low | Exclude from implementation; generalize before Phase 1.3 |
