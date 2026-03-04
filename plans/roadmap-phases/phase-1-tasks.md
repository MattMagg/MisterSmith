# Tasks: Phase 1 — Foundation

**Input**: `plans/roadmap-phases/phase-1-plan.md`, `plans/roadmap-phases/phase-1-foundation.md`
**Prerequisites**: Key Decisions 1-5 from `phase-1-plan.md` must be resolved before implementation begins.
**Gate**: `cargo build -p mister-smith-core && cargo build -p mister-smith-config && cargo test --workspace`

## Format: `[ID] [P?] Description — file path`

- **[P]**: Can run in parallel (different files, no dependencies on concurrent tasks)
- Spec traces reference the authoritative spec file for each type/trait
- Dependencies listed at end of each phase

---

## Phase 1: Setup — Cargo Workspace and Crate Scaffolding

**Purpose**: Create the Cargo workspace, crate scaffolding, and CI configuration. No Rust logic yet.

- [x] T001 Create workspace root `Cargo.toml` with resolver 2, workspace members `crates/mister-smith-core` and `crates/mister-smith-config`, workspace-level package metadata (edition 2021, rust-version 1.88, license MIT OR Apache-2.0), and `[workspace.dependencies]` section pinning: thiserror 1.0.69, serde 1.0.228 (features: derive), serde_json 1.0.149, uuid 1.11.0 (features: v4, serde), async-trait 0.1.83, semver 1.0, toml 0.8, std::time::Duration (stdlib) — `Cargo.toml`
  - Spec trace: `ROADMAP.md` (Crate Map), `VERSION_REFERENCE.md`

- [x] T002 [P] Create `crates/mister-smith-core/Cargo.toml` with `[package]` (name, version 0.1.0, edition.workspace, rust-version.workspace) and `[dependencies]` referencing workspace deps: thiserror, serde, serde_json, uuid, async-trait, semver — `crates/mister-smith-core/Cargo.toml`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md` (Dependencies section)
  - **Note**: `serde_json` needed for Tool trait params/results; `semver` needed for Tool::version()

- [x] T003 [P] Create `crates/mister-smith-config/Cargo.toml` with `[package]` and `[dependencies]` referencing workspace deps: serde, toml, thiserror, plus `mister-smith-core = { path = "../mister-smith-core" }` — `crates/mister-smith-config/Cargo.toml`
  - Spec trace: `spec/core-architecture/implementation-config.md` (Dependencies)

- [x] T004 [P] Create stub `crates/mister-smith-core/src/lib.rs` with `#![deny(missing_docs, unsafe_code)]` and module declarations (ids, enums, supervision, error, traits) — `crates/mister-smith-core/src/lib.rs`

- [x] T005 [P] Create stub `crates/mister-smith-config/src/lib.rs` with `#![deny(missing_docs, unsafe_code)]` and module declarations (types, loader, validation, error) — `crates/mister-smith-config/src/lib.rs`

- [x] T006 [P] Create `.github/workflows/ci.yml` with Rust 1.88.0 toolchain, `cargo build --workspace`, `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt --all -- --check` — `.github/workflows/ci.yml`

- [x] T007 Create `rust-toolchain.toml` pinning to Rust 1.88.0 channel — `rust-toolchain.toml`

- [x] T008 Add `crates/` to `.gitignore` `target/` entry (ensure `/target/` covers workspace); add `Cargo.lock` to version control (binary crate practice for reproducibility) — `.gitignore`

**Checkpoint**: `cargo build --workspace` compiles with zero errors (empty crates). CI pipeline runs.

### Dependencies

- T002, T003, T004, T005, T006, T007, T008 all depend on T001 (workspace root must exist first)
- T002–T008 are independent of each other and can run in parallel after T001

---

## Phase 2: Core Types (mister-smith-core) — IDs, Enums, Errors

**Purpose**: Implement all canonical types from `spec/core-architecture/type-definitions.md` and the error hierarchy from `spec/core-architecture/runtime-and-errors.md`.

### ID Newtypes

- [x] T009 [P] Implement `AgentId` newtype: `pub struct AgentId(pub Uuid)` with derives (Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize), `new()` -> `Self(Uuid::new_v4())`, `from_uuid(uuid: Uuid)`, `Display` impl, `AsRef<Uuid>` — `crates/mister-smith-core/src/ids.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:59`

- [x] T010 [P] Implement `TaskId` newtype: same pattern as AgentId — `crates/mister-smith-core/src/ids.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:62`

- [x] T011 [P] Implement `MessageId` newtype: same pattern as AgentId — `crates/mister-smith-core/src/ids.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:65`

- [x] T012 [P] Implement `ToolId` newtype: same pattern as AgentId — `crates/mister-smith-core/src/ids.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:68`

**Note**: T009–T012 all write to the same file. Implement together as a single logical unit or split into sequential subtasks within the file.

### Core Enums

- [x] T013 [P] Implement `MessagePriority` enum with `#[repr(u8)]` discriminants: Critical=0, High=1, Normal=2, Low=3, Bulk=4. Derives: Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize. Add `impl Default for MessagePriority` returning `Normal`. Add unit test asserting discriminant values — `crates/mister-smith-core/src/enums.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:71-78`

- [x] T014 [P] Implement `AgentState` enum: Initializing, Running, Paused, Stopping, Terminated, Error, Restarting. Derives: Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize. Add doc comment: "Lifecycle state machine for Phase 7 agent lifecycle management." — `crates/mister-smith-core/src/enums.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:81-90`

- [x] T015 [P] Implement `AgentAvailability` enum: Idle, Busy, Error, Offline, Starting, Stopping. Derives: Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize. Add doc comment: "Transport/runtime availability signal for status channels and heartbeats." — `crates/mister-smith-core/src/enums.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:93-101`

- [x] T016 [P] Implement `AgentType` enum: Supervisor, Worker, Coordinator, Monitor, Planner, Executor, Critic, Router, Memory. Derives: Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize — `crates/mister-smith-core/src/enums.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:103-114`

**Note**: T013–T016 all write to `enums.rs`. Same consideration as IDs — implement as logical unit.

### Error Hierarchy

- [x] T017 Implement `RuntimeError` enum with variants: BuildFailed(#[from] std::io::Error), StartupFailed(String), ShutdownFailed(String), ConfigurationInvalid(String). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:98-107`

- [x] T018 Implement `SupervisionError` enum with variants: StrategyFailed(String), RestartFailed(String), EscalationFailed(String), RestartLimitExceeded, TreeCorrupted(String). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:110-121`

- [x] T019 Implement `ActorError` enum with variants: StartupFailed(Box<dyn std::error::Error + Send + Sync>), MailboxFull, ActorStopped, SystemStopped, AskTimeout, DeserializationFailed(String), MessageHandlingFailed(String). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:124-139`

- [x] T020 Implement `TaskError` enum with variants: ExecutionFailed(String), TimedOut, TaskCancelled, ExecutorShutdown, QueueFull, SerializationFailed(String). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:142-155`

- [x] T021 Implement `StreamError` enum with variants: ProcessingFailed(String), ProcessorFailed(String, String), SinkFull, SinkBlocked, StreamEnded, BackpressureFailed(String). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:158-171`

- [x] T022 Implement `EventError` enum with variants: HandlerFailed(String), SerializationFailed(String), PublicationFailed(String), SubscriptionFailed(String), StoreFailed(String). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:174-185`

- [x] T023 Implement `ToolError` enum with variants: ExecutionFailed(String), NotFound(String), AccessDenied(String), ParameterValidationFailed(String), Timeout(String). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:188-199`

- [x] T024 Implement `ConfigError` enum with variants: ValidationFailed(String), FileNotFound(String), ParseFailed(String), MergeFailed(String). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:202-211`

- [x] T025 Implement `ResourceError` enum with variants: AcquisitionFailed(String), PoolExhausted, HealthCheckFailed(String), CleanupFailed(String). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:214-223`

- [x] T026 Implement `NetworkError` enum with variants: ConnectionFailed(String), Timeout(String), ProtocolError(String). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:226-233`

- [x] T027 Implement `PersistenceError` enum with variants: DatabaseFailed(String), SerializationFailed(String), DataCorrupted(String). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:236-243`

- [x] T028 Implement `SystemError` enum with `#[from]` conversions from all sub-errors: Runtime(RuntimeError), Supervision(SupervisionError), Configuration(ConfigError), Resource(ResourceError), Network(NetworkError), Persistence(PersistenceError), Actor(ActorError), Task(TaskError), Stream(StreamError), Event(EventError), Tool(ToolError). Derive: Debug, Error — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:72-95`
  - Depends on: T017–T027

- [x] T029 Implement `ErrorSeverity` enum: Low, Medium, High, Critical. Derives: Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:246-252`

- [x] T030 Implement `RecoveryStrategy` enum: Retry { max_attempts: u32, delay: Duration }, Restart, Escalate, Reload, CircuitBreaker, Failover, Ignore. Derive: Debug, Clone — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:254-263`

- [x] T031 Implement `severity()` and `recovery_strategy()` methods on `SystemError` per the match arms in spec — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/runtime-and-errors.md:265-299`
  - Depends on: T028, T029, T030

- [x] T032 Implement `pub type FrameworkResult<T> = Result<T, SystemError>` — `crates/mister-smith-core/src/error.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:139`
  - Depends on: T028

- [x] T033 Unit tests for error hierarchy: verify `#[from]` conversions compile (e.g., `SystemError::from(RuntimeError::StartupFailed("test".into()))`), verify severity mapping, verify Display output — `crates/mister-smith-core/src/error.rs` (inline tests) or `crates/mister-smith-core/tests/error_tests.rs`
  - Depends on: T028, T031

**Checkpoint**: `cargo build -p mister-smith-core` compiles. All ID newtypes, enums, and errors are defined. `cargo test -p mister-smith-core` passes.

### Dependencies

- T009–T016 depend on T001–T004 (workspace and crate scaffold must exist)
- T009–T016 are mutually parallel (write to different logical sections; same file but no logical deps)
- T017–T027 are mutually parallel (all sub-error types are independent)
- T028 depends on T017–T027 (SystemError wraps all sub-errors)
- T031 depends on T028, T029, T030
- T032 depends on T028
- T033 depends on T028, T031

---

## Phase 3: Supervision Types (mister-smith-core)

**Purpose**: Implement the supervision model types required by `SupervisionStrategy` struct.

**Prerequisite**: Key Decisions 1 and 2 from `phase-1-plan.md` must be resolved (EscalationPolicy and BackoffStrategy canonical definitions).

- [x] T034 Implement `RestartPolicy` enum: OneForOne, OneForAll, RestForOne. Derives: Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize — `crates/mister-smith-core/src/supervision.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:117-121`

- [x] T035 [P] Implement `RestartScope` enum: Permanent, Transient, Temporary. Derives: Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize. Add doc comments explaining OTP semantics: Permanent = always restart, Transient = restart only on abnormal exit, Temporary = never restart — `crates/mister-smith-core/src/supervision.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:123-128`

- [x] T036 Implement `EscalationPolicy` enum (canonical, per Decision 1): Terminate, Restart, Escalate, LogAndIgnore. Derives: Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize. Add doc comments for each variant — `crates/mister-smith-core/src/supervision.rs`
  - Spec trace: `spec/data-management/agent-lifecycle.md:1224-1229`, `spec/core-architecture/async-patterns.md:1853-1857`
  - **Note**: This reconciles the 4 conflicting definitions. Canonical variants are Terminate, Restart, Escalate (from agent-lifecycle.md) plus LogAndIgnore (from async-patterns.md).

- [x] T037 Implement `BackoffStrategy` enum (canonical, per Decision 2): Fixed(Duration), Exponential { initial: Duration, max: Duration, multiplier: f64 }, Linear { initial: Duration, increment: Duration }. Derives: Debug, Clone, Serialize, Deserialize — `crates/mister-smith-core/src/supervision.rs`
  - Spec trace: `spec/data-management/agent-lifecycle.md:1233-1237`
  - **Note**: `BackoffStrategy` cannot derive Copy because it contains Duration fields in struct variants.

- [x] T038 Implement `SupervisionStrategy` struct: restart_policy (RestartPolicy), max_failures (u32), failure_window (Duration), escalation_policy (EscalationPolicy), backoff_strategy (BackoffStrategy). Derives: Debug, Clone, Serialize, Deserialize. Add `impl Default` with sensible defaults (OneForOne, max_failures=3, 60s window, Escalate, Exponential backoff) — `crates/mister-smith-core/src/supervision.rs`
  - Spec trace: `spec/core-architecture/type-definitions.md:130-137`
  - Depends on: T034, T036, T037

- [x] T039 Unit tests for supervision types: verify Default impl, verify serialization roundtrip for SupervisionStrategy, verify all enum variants exist — `crates/mister-smith-core/src/supervision.rs` (inline tests)
  - Depends on: T038

**Checkpoint**: All supervision types compile and serialize correctly.

### Dependencies

- T034–T037 depend on Phase 1 completion (crate scaffold)
- T034, T035, T036, T037 are mutually parallel
- T038 depends on T034, T036, T037 (struct fields reference these enums)
- T039 depends on T038

---

## Phase 4: Core Traits (mister-smith-core)

**Purpose**: Define the 6 core trait signatures that form the framework's extension points. These are trait definitions only — no implementations.

- [x] T040 Implement `Actor` trait with `#[async_trait]`: associated types `Message: Send + 'static`, `State: Send + 'static`, `Error: Send + std::error::Error + 'static`; methods `async fn handle_message(&mut self, message: Self::Message, state: &mut Self::State) -> Result<(), Self::Error>`, `fn pre_start(&mut self) -> Result<(), Self::Error>`, `fn post_stop(&mut self) -> Result<(), Self::Error>`, `fn actor_id(&self) -> AgentId` — `crates/mister-smith-core/src/traits.rs`
  - Spec trace: `spec/core-architecture/module-organization-type-system.md:367-381`
  - Depends on: T009 (AgentId)

- [x] T041 Implement `Tool` trait with `#[async_trait]`: methods `async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, ToolError>`, `fn schema(&self) -> ToolSchema`, `fn capabilities(&self) -> ToolCapabilities`, `fn tool_id(&self) -> ToolId`, `fn version(&self) -> semver::Version`. Define placeholder `ToolSchema` and `ToolCapabilities` structs (empty for now, will be expanded in Phase 7) — `crates/mister-smith-core/src/traits.rs`
  - Spec trace: `spec/core-architecture/module-organization-type-system.md:397-404`
  - Depends on: T012 (ToolId), T023 (ToolError)
  - **Note**: Add `serde_json` and `semver` to mister-smith-core dependencies for this trait. `ToolSchema` and `ToolCapabilities` are minimal placeholder structs; full definitions come in Phase 7.

- [x] T042 Implement `Agent` trait extending `Tool` with `#[async_trait]`: `pub trait Agent: Tool + Send + Sync + 'static`; associated types `Context: Send + Sync`, `Error: Send + std::error::Error + 'static`; methods `async fn process(&self, message: serde_json::Value) -> Result<serde_json::Value, Self::Error>`, `fn role(&self) -> AgentType`, `fn context(&self) -> &Self::Context`, `async fn initialize(&mut self, context: Self::Context) -> Result<(), Self::Error>`, `fn dependencies() -> Vec<std::any::TypeId> where Self: Sized` — `crates/mister-smith-core/src/traits.rs`
  - Spec trace: `spec/core-architecture/module-organization-type-system.md:407-424`
  - Depends on: T041 (Tool trait), T016 (AgentType)

- [x] T043 Implement `Resource` trait with `#[async_trait]`: associated types `Config: Send + Sync + Clone + 'static`, `Error: Send + std::error::Error + 'static`; methods `async fn acquire(config: Self::Config) -> Result<Self, Self::Error> where Self: Sized`, `async fn release(self) -> Result<(), Self::Error>`, `fn is_healthy(&self) -> bool`, `async fn health_check(&self) -> Result<HealthStatus, Self::Error>`, `fn resource_id(&self) -> ResourceId`. Define `ResourceId` as a newtype over Uuid (same pattern as AgentId). Define `HealthStatus` enum: Healthy, Degraded, Unhealthy, Unknown — `crates/mister-smith-core/src/traits.rs`
  - Spec trace: `spec/core-architecture/module-organization-type-system.md:430-442`
  - Depends on: T025 (ResourceError)

- [x] T044 Implement `Supervisor` trait with `#[async_trait]`: associated types `Child: Send + 'static`, `Error: Send + std::error::Error + 'static`; methods `async fn supervise(&self, children: Vec<Self::Child>) -> Result<(), Self::Error>`, `fn supervision_strategy(&self) -> &SupervisionStrategy`, `fn restart_policy(&self) -> RestartPolicy`, `fn escalation_policy(&self) -> EscalationPolicy`, `fn supervisor_id(&self) -> AgentId` — `crates/mister-smith-core/src/traits.rs`
  - Spec trace: `spec/core-architecture/module-organization-type-system.md:384-394`
  - Depends on: T038 (SupervisionStrategy), T034 (RestartPolicy), T036 (EscalationPolicy), T009 (AgentId)

- [x] T045 Implement `Transport` trait with `#[async_trait]`: associated types `Message: Send + Sync + Serialize + DeserializeOwned + 'static`, `Subscription: Send + 'static`, `ConnectionInfo: Send + Sync + 'static`; methods `async fn send(&self, destination: &str, message: Self::Message) -> Result<(), NetworkError>`, `async fn broadcast(&self, topic: &str, message: Self::Message) -> Result<(), NetworkError>`, `async fn subscribe(&self, pattern: &str) -> Result<Self::Subscription, NetworkError>`, `async fn request_response(&self, destination: &str, message: Self::Message, timeout: Duration) -> Result<Self::Message, NetworkError>`, `async fn connect(&mut self, config: &TransportConfig) -> Result<Self::ConnectionInfo, NetworkError>`, `async fn disconnect(&mut self) -> Result<(), NetworkError>`, `fn connection_status(&self) -> ConnectionStatus`. Define `ConnectionStatus` enum: Connected, Disconnected, Reconnecting. Define placeholder `TransportConfig` struct — `crates/mister-smith-core/src/traits.rs`
  - Spec trace: `spec/core-architecture/integration-contracts.md:200-214`
  - Depends on: T026 (NetworkError)
  - **Note**: Uses `NetworkError` from the error hierarchy (not a separate `TransportError`). `TransportConfig` is a minimal placeholder; full definition in Phase 4.

- [x] T045b Implement `EventPublisher` trait with `#[async_trait]` in `crates/mister-smith-core/src/traits.rs`: `pub trait EventPublisher: Send + Sync + 'static { async fn publish(&self, event: SystemEvent) -> Result<(), EventError>; }`. This trait breaks the circular dependency between monitoring and events crates — `HealthMonitor` takes `Option<Arc<dyn EventPublisher>>`, `EventBus` implements it. Define a minimal forward-declaration `SystemEvent` struct in core if needed, or use `serde_json::Value` as the event payload type to avoid pulling full event types into core. — `crates/mister-smith-core/src/traits.rs`
  - Spec trace: audit-report.md C3 (circular dependency resolution)
  - Depends on: T022 (EventError)

- [x] T046 Wire up all trait re-exports in `crates/mister-smith-core/src/lib.rs`: pub use traits::{Actor, Agent, Tool, Resource, Supervisor, Transport, EventPublisher}; pub use ids::{AgentId, TaskId, MessageId, ToolId}; pub use enums::{MessagePriority, AgentState, AgentAvailability, AgentType}; pub use supervision::{RestartPolicy, RestartScope, SupervisionStrategy, EscalationPolicy, BackoffStrategy}; pub use error::{SystemError, FrameworkResult, ErrorSeverity, RecoveryStrategy, ...all sub-errors} — `crates/mister-smith-core/src/lib.rs`
  - Depends on: all Phase 2, 3, 4 tasks

- [x] T047 Verify all traits compile with a test module containing dummy struct implementations (e.g., `struct MockActor;` implementing `Actor` trait to confirm the trait is implementable) — `crates/mister-smith-core/tests/trait_compilation_tests.rs`
  - Depends on: T040–T046

**Checkpoint**: All 6 trait definitions compile. Dummy implementations verify trait implementability. `cargo test -p mister-smith-core` passes all tests.

### Dependencies

- T040 depends on T009
- T041 depends on T012, T023
- T042 depends on T041, T016
- T043 depends on T025
- T044 depends on T038, T034, T036, T009
- T045 depends on error types from Phase 2
- T046 depends on T040–T045 (all traits and types must exist)
- T047 depends on T046
- T040, T041, T043, T045 are mutually parallel (write to same file but independent trait definitions)
- T042 depends on T041; T044 depends on supervision types

---

## Phase 5: Configuration (mister-smith-config)

**Purpose**: Implement typed configuration loading, validation, and environment overlay.

### Config Types

- [x] T048 Implement `RuntimeConfig` struct (single canonical definition — consumed by both config loading and Tokio runtime builder): worker_threads (Option<usize>), blocking_threads (usize, default 512), max_memory (usize, default 0), thread_stack_size (Option<usize>), thread_keep_alive (Duration, default 60s), enable_all (bool, default true), enable_time (bool, default true), enable_io (bool, default true). Derives: Debug, Clone, Serialize, Deserialize. Implement `Default`. The Tokio-specific fields (`thread_keep_alive`, `enable_*`) have `#[serde(default)]` so existing TOML configs without them still work. — `crates/mister-smith-config/src/types.rs`
  - Spec trace: `spec/core-architecture/implementation-config.md:41-56`, `spec/core-architecture/runtime-and-errors.md:319-342`, `spec/core-architecture/tokio-runtime.md:203-226`
  - **Note**: This is the SINGLE `RuntimeConfig` definition. Phase 2's `mister-smith-runtime` crate re-uses this struct (adds `build_runtime()` and preset constructors as extension methods via `impl RuntimeConfig` in the runtime crate). No duplicate `RuntimeConfig` in Phase 2.

- [x] T049 [P] Implement `SupervisionConfig` struct: max_restart_attempts (u32, default 3), restart_window (Duration, default 60s), escalation_timeout (Duration, default 30s). Derives: Debug, Clone, Serialize, Deserialize. Implement `Default` — `crates/mister-smith-config/src/types.rs`
  - Spec trace: `spec/core-architecture/implementation-config.md:58-74`

- [x] T050 [P] Implement `MonitoringConfig` struct: health_check_interval (Duration, default 30s), metrics_export_interval (Duration, default 60s), log_level (String, default "info"). Derives: Debug, Clone, Serialize, Deserialize. Implement `Default` — `crates/mister-smith-config/src/types.rs`
  - Spec trace: `spec/core-architecture/implementation-config.md:76-92`

- [x] T051 Implement `AgentConfig` struct: runtime (RuntimeConfig), supervision (SupervisionConfig), monitoring (MonitoringConfig). All fields have `#[serde(default)]`. Derives: Debug, Clone, Serialize, Deserialize. Implement `Default` — `crates/mister-smith-config/src/types.rs`
  - Spec trace: `spec/core-architecture/implementation-config.md:29-38`
  - Depends on: T048, T049, T050

- [x] T052 [P] Implement `TransportConfig` struct: nats_url (Option<String>), http_port (Option<u16>), grpc_port (Option<u16>). Derives: Debug, Clone, Serialize, Deserialize. Implement `Default`. This is a minimal placeholder; full transport config comes in Phase 4 — `crates/mister-smith-config/src/types.rs`
  - Spec trace: `spec/operations/configuration-management.md:2.1.2`

- [x] T053 [P] Implement `SecurityConfig` struct: enabled (bool, default false), tls_enabled (bool, default false), auth_required (bool, default false). Derives: Debug, Clone, Serialize, Deserialize. Implement `Default`. Minimal placeholder; full security config comes in Phase 5 — `crates/mister-smith-config/src/types.rs`
  - Spec trace: `spec/operations/configuration-management.md:2.2.1`

- [x] T054 Implement `FrameworkConfig` top-level struct: agent (AgentConfig), transport (TransportConfig), security (SecurityConfig). All fields `#[serde(default)]`. Derives: Debug, Clone, Serialize, Deserialize. Implement `Default` — `crates/mister-smith-config/src/types.rs`
  - Spec trace: `spec/operations/configuration-management.md:2.1.1`
  - Depends on: T051, T052, T053

### Config Error Type

- [x] T055 [P] Implement `ConfigValidationError` enum: ValidationError(String), MissingField(String), InvalidValue { field: String, reason: String }, EnvVarError(String), FileError(#[from] std::io::Error), DeserializationError(String). Derive: Debug, Error (thiserror). Add `Display` messages — `crates/mister-smith-config/src/error.rs`
  - Spec trace: `spec/core-architecture/implementation-config.md:342-361`

### Config Validation

- [x] T056 Implement `validate()` method on `RuntimeConfig`: worker_threads must be 1..=1024 if Some, blocking_threads must be 1..=512, max_memory must be non-negative. Returns `Result<(), ConfigValidationError>` — `crates/mister-smith-config/src/validation.rs`
  - Spec trace: `spec/core-architecture/implementation-config.md:41-56` (validation ranges)
  - Depends on: T048, T055

- [x] T057 [P] Implement `validate()` method on `SupervisionConfig`: max_restart_attempts 0..=100, restart_window 1s..=3600s, escalation_timeout 1s..=300s — `crates/mister-smith-config/src/validation.rs`
  - Spec trace: `spec/core-architecture/implementation-config.md:58-74`
  - Depends on: T049, T055

- [x] T058 [P] Implement `validate()` method on `MonitoringConfig`: health_check_interval 1s..=300s, metrics_export_interval 1s..=600s, log_level must be one of trace/debug/info/warn/error — `crates/mister-smith-config/src/validation.rs`
  - Spec trace: `spec/core-architecture/implementation-config.md:76-92`
  - Depends on: T050, T055

- [x] T059 Implement `validate()` method on `AgentConfig` that delegates to each sub-config's validate() and collects errors — `crates/mister-smith-config/src/validation.rs`
  - Depends on: T056, T057, T058

- [x] T060 Implement `validate()` method on `FrameworkConfig` that delegates to AgentConfig, TransportConfig, SecurityConfig validation — `crates/mister-smith-config/src/validation.rs`
  - Depends on: T059

### Config Loading

- [x] T061 Implement TOML config file loading: `pub fn load_from_file(path: &Path) -> Result<FrameworkConfig, ConfigValidationError>` that reads file, deserializes with toml crate, runs validate(), returns validated config — `crates/mister-smith-config/src/loader.rs`
  - Spec trace: `spec/core-architecture/implementation-config.md:382-421`
  - Depends on: T054, T055, T060

- [x] T062 Implement environment variable overlay: `pub fn apply_env_overlay(config: &mut FrameworkConfig, prefix: &str)` that reads env vars with prefix (e.g., `MISTER_SMITH_RUNTIME__WORKER_THREADS`) and overrides matching config fields. Use `__` as separator for nested fields — `crates/mister-smith-config/src/loader.rs`
  - Spec trace: `spec/core-architecture/implementation-config.md:397-404`
  - Depends on: T054

- [x] T063 Implement config file discovery: `pub fn discover_config_paths() -> Vec<PathBuf>` returning paths in priority order: `/etc/mister-smith/config.toml`, `~/.mister-smith/config.toml`, `./mister-smith.toml`, env-specific via `MS_ENVIRONMENT` — `crates/mister-smith-config/src/loader.rs`
  - Spec trace: `spec/core-architecture/implementation-config.md:441-461`

- [x] T064 Implement `pub fn load_config() -> Result<FrameworkConfig, ConfigValidationError>` that discovers config paths, loads from first existing file, applies env overlay, validates, returns final config — `crates/mister-smith-config/src/loader.rs`
  - Depends on: T061, T062, T063, T060

### Config Re-exports and Tests

- [x] T065 Wire up re-exports in `crates/mister-smith-config/src/lib.rs`: pub use types::{FrameworkConfig, AgentConfig, RuntimeConfig, SupervisionConfig, MonitoringConfig, TransportConfig, SecurityConfig}; pub use loader::{load_config, load_from_file}; pub use error::ConfigValidationError; pub use validation — `crates/mister-smith-config/src/lib.rs`
  - Depends on: T054, T055, T064

- [x] T066 Unit tests for config types: verify Default impls produce valid configs, verify serde roundtrip (serialize to TOML then deserialize back), verify partial TOML (only some fields set) deserializes correctly with defaults — `crates/mister-smith-config/tests/config_tests.rs`
  - Depends on: T054

- [x] T067 Unit tests for config validation: verify out-of-range values are rejected, verify valid configs pass, verify error messages are actionable — `crates/mister-smith-config/tests/validation_tests.rs`
  - Depends on: T060

- [x] T068 Unit tests for config loading: verify TOML file loading, verify env overlay overrides, verify file discovery order (use tempdir for test files) — `crates/mister-smith-config/tests/loader_tests.rs`
  - Depends on: T064

**Checkpoint**: `cargo build -p mister-smith-config` compiles. Config types serialize/deserialize correctly. Validation catches invalid values. Env overlay works. `cargo test -p mister-smith-config` passes.

### Dependencies

- T048–T053, T055 can all run in parallel (independent structs/enums)
- T051 depends on T048, T049, T050
- T054 depends on T051, T052, T053
- T056–T058 are mutually parallel; each depends on their respective config struct + T055
- T059 depends on T056, T057, T058
- T060 depends on T059
- T061 depends on T054, T055, T060
- T062, T063 can run in parallel; both depend on T054
- T064 depends on T061, T062, T063, T060
- T065 depends on T054, T055, T064
- T066–T068 depend on their respective implementation tasks

---

## Phase 6: Integration and Validation

**Purpose**: Verify cross-crate compilation, run all gate checks, and ensure the Phase 1 contract surface is complete.

- [x] T069 Verify `mister-smith-config` can import and use types from `mister-smith-core`: create a test in mister-smith-config that instantiates an `AgentId`, creates a `SupervisionStrategy`, and references `SystemError` — `crates/mister-smith-config/tests/cross_crate_tests.rs`
  - Depends on: T046, T065

- [x] T070 Run full workspace build: `cargo build --workspace` — verify zero errors, zero warnings — CLI
  - Depends on: T046, T065

- [x] T071 Run full test suite: `cargo test --workspace` — verify all tests pass — CLI
  - Depends on: T070

- [x] T072 Run clippy: `cargo clippy --workspace -- -D warnings` — verify zero warnings — CLI
  - Depends on: T070

- [x] T073 Run fmt check: `cargo fmt --all -- --check` — verify all code is formatted — CLI
  - Depends on: T070

- [x] T074 Run gate validation commands from `phase-1-foundation.md`: grep for canonical type definitions in spec, verify no conflicting definitions remain — CLI
  - Depends on: T070

- [x] T075 Document any spec reconciliation changes made during implementation (e.g., EscalationPolicy canonicalization) in a short changelog section at the bottom of `phase-1-plan.md` — `plans/roadmap-phases/phase-1-plan.md`
  - Depends on: T074

**Checkpoint**: Phase 1 gate is satisfied. Both crates compile cleanly. All tests pass. No conflicting type definitions. Downstream phases can begin.

### Dependencies

- T069 depends on all of Phase 4 and Phase 5
- T070–T074 are sequential (build before test, test before lint)
- T075 depends on T074

---

## Dependencies Summary

```text
Phase 1 (Setup)
  T001 ──┬── T002 ──┐
         ├── T003 ──┤
         ├── T004 ──┤
         ├── T005 ──┤
         ├── T006 ──┤
         ├── T007 ──┤
         └── T008 ──┘
                    │
Phase 2 (Core Types)          ┌── T013 (MessagePriority)
  T009-T012 (IDs) ────────────┼── T014 (AgentState)
  T013-T016 (Enums) ──────────┼── T015 (AgentAvailability)
  T017-T027 (Sub-errors) ─┐   └── T016 (AgentType)
  T028 (SystemError) ◄────┤
  T029-T030 (Severity, Recovery)
  T031 (severity/recovery methods) ◄── T028, T029, T030
  T032 (FrameworkResult) ◄── T028
  T033 (error tests) ◄── T028, T031
                    │
Phase 3 (Supervision Types)
  T034-T037 (enums) ───parallel───┐
  T038 (SupervisionStrategy) ◄────┤
  T039 (tests) ◄── T038           │
                    │              │
Phase 4 (Core Traits)             │
  T040 (Actor) ◄── T009           │
  T041 (Tool) ◄── T012, T023     │
  T042 (Agent) ◄── T041, T016    │
  T043 (Resource) ◄── T025       │
  T044 (Supervisor) ◄── T038 ────┘
  T045 (Transport) ◄── error types
  T046 (re-exports) ◄── T040-T045
  T047 (trait tests) ◄── T046
                    │
Phase 5 (Configuration)
  T048-T053 (config structs) ──parallel──┐
  T054 (FrameworkConfig) ◄── T051-T053   │
  T055 (config error) ──parallel──┐      │
  T056-T058 (validation) ◄── T055 ──────┤
  T059-T060 (aggregate validation) ◄────┤
  T061-T064 (loading) ◄── T054, T060    │
  T065 (re-exports) ◄── T054, T064     │
  T066-T068 (tests) ◄── implementations │
                    │                    │
Phase 6 (Integration)                   │
  T069 (cross-crate) ◄── T046, T065 ───┘
  T070 (workspace build) ◄── T069
  T071-T074 (validation) ◄── T070
  T075 (documentation) ◄── T074
```

## Execution Strategy

### Recommended Sequential Order

For a single implementer, work through phases in order:

1. **Phase 1** (T001–T008): ~30 minutes. Scaffold only.
2. **Phase 2** (T009–T033): ~2 hours. All types and errors in mister-smith-core.
3. **Phase 3** (T034–T039): ~30 minutes. Supervision types.
4. **Phase 4** (T040–T047): ~1.5 hours. Trait definitions.
5. **Phase 5** (T048–T068): ~2 hours. Configuration crate.
6. **Phase 6** (T069–T075): ~30 minutes. Integration and gate checks.

### Parallel Opportunities

With multiple implementers:

- **Implementer A**: Phase 2 (IDs + enums) then Phase 3 (supervision types)
- **Implementer B**: Phase 2 (error hierarchy) — works on error.rs while A works on ids.rs/enums.rs
- Once Phase 2–3 complete, Phase 4 (traits) and Phase 5 (config) can proceed in parallel since config depends on core types but not on trait definitions.
