# Research: Phase 3 — Actor System & Supervision Trees

**Date**: 2026-03-04
**Feature**: [spec.md](spec.md)

## External Research Tools

During implementation, use the following MCP-connected tools for research beyond the local codebase:

| Tool | Use For | Examples |
|------|---------|----------|
| **Context7 MCP** | Up-to-date, version-specific library documentation and API references | Tokio mpsc/oneshot exact API signatures, `tokio::spawn` JoinHandle semantics, async-trait macro behavior, `tokio::sync` primitives |
| **Tavily** | Web search for Rust patterns, existing implementations, and community best practices (load `tavily-best-practices` skill first) | Rust actor model implementations (Actix, Kameo, Ractor patterns), supervision tree designs, panic-catching in tokio tasks, bounded channel backpressure patterns |

**When to use**: Before writing code against any external dependency API. Don't assume API signatures from memory — verify with Context7. Before implementing non-trivial patterns (actor cell loop, supervision escalation, graceful shutdown) — validate approach with Tavily.

**Key lookups for this phase**:
- `tokio::sync::mpsc` — bounded `try_send` vs `send`, `Sender::closed()`, permit API
- `tokio::sync::oneshot` — drop semantics, `RecvError` behavior
- `tokio::spawn` — `JoinHandle`, `JoinError::is_panic()`, abort behavior
- `tokio::task::block_in_place` — for ActorSystem Drop (D8)
- `async-trait` — generated Future bounds, Send requirements
- `std::panic::AssertUnwindSafe` + `catch_unwind` — if needed for actor panic isolation

## R1: Existing Core Traits for Actor/Supervisor

**Decision**: Use the existing `Actor` and `Supervisor` traits from `mister-smith-core/src/traits.rs` as the public API contract. Extend with concrete implementations in the new crates.

**Rationale**: The `Actor` trait already defines the exact interface the spec requires:
```rust
#[async_trait]
pub trait Actor: Send + 'static {
    type Message: Send + 'static;
    type State: Send + 'static;
    type Error: Send + std::error::Error + 'static;
    async fn handle_message(&mut self, message: Self::Message, state: &mut Self::State) -> Result<(), Self::Error>;
    fn pre_start(&mut self) -> Result<(), Self::Error>;
    fn post_stop(&mut self) -> Result<(), Self::Error>;
    fn actor_id(&self) -> AgentId;
}
```
Similarly, the `Supervisor` trait defines `supervise`, `supervision_strategy`, `restart_policy`, `escalation_policy`, `supervisor_id`.

**Alternatives considered**:
- Define new traits in the actor crate: rejected because it would violate Constitution Principle I (canonical single source). The core crate IS the canonical source for traits.
- Extend Actor trait with additional methods: rejected because it would be a breaking change to core. Instead, use wrapper structs and extension traits in the actor crate.

## R2: Supervision Types Already Implemented

**Decision**: All supervision types exist in `mister-smith-core/src/supervision.rs` and are ready to use.

**Rationale**: Verified implementations match spec exactly:
- `RestartPolicy`: `OneForOne`, `OneForAll`, `RestForOne`
- `RestartScope`: `Permanent`, `Transient`, `Temporary`
- `EscalationPolicy`: `Terminate`, `Restart`, `Escalate`, `LogAndIgnore`
- `BackoffStrategy`: `Fixed(Duration)`, `Exponential { initial, max, multiplier }`, `Linear { initial, increment }`
- `SupervisionStrategy`: struct combining all above with `max_failures`, `failure_window`

The `BackoffStrategy::Exponential` already has `initial`, `max`, and `multiplier` — resolving CHK021 from the checklist. The spec's FR-009 mentioning only "backoff_multiplier" was an incomplete reference; the canonical type has all three parameters.

**Alternatives considered**: None needed — types are already canonical.

## R3: Error Types Available

**Decision**: Use existing `ActorError` and `SupervisionError` from `mister-smith-core/src/error.rs`.

**Rationale**: `ActorError` variants cover all needed cases:
- `StartupFailed(Box<dyn Error>)` — pre_start failure
- `MailboxFull` — bounded mailbox at capacity
- `ActorStopped` — message sent to terminated actor
- `SystemStopped` — system shutting down
- `AskTimeout` — ask response timeout
- `MessageHandlingFailed` — handle_message error
- `DeserializationFailed` — message deserialization (future Phase 4)

`SupervisionError` variants:
- `StrategyFailed` — restart policy execution failed
- `RestartFailed` — actor restart failed
- `EscalationFailed` — escalation to parent failed
- `RestartLimitExceeded` — max_restarts exceeded
- `TreeCorrupted` — supervision tree in inconsistent state

Both have `#[from]` conversions to `SystemError`.

## R4: EventBus Integration Pattern

**Decision**: Actor lifecycle events are emitted as `Event` with `EventType::Agent(AgentEventType::*)` through the `EventBus`.

**Rationale**: The events crate provides `AgentEventType` with exactly the variants needed:
- `Created` — actor spawned
- `Started` — actor entered Running state (or restarted)
- `Stopped` — actor terminated
- `Failed` — actor error
- `StateChanged` — any state transition
- `MessageReceived` / `MessageProcessed` — for throughput metrics

The `EventBuilder` provides fluent construction with `.with_correlation_id()` and `.with_causation_id()` for distributed tracing (FR-013).

The `EventBus` implements `EventPublisher` from core, so it can be injected into the ActorSystem via the trait interface.

## R5: Health Monitoring Integration

**Decision**: Implement `HealthCheck` for the actor system, reporting actor counts, error states, and tree depth.

**Rationale**: The monitoring crate's `HealthCheck` trait requires:
```rust
async fn check(&self) -> Result<Status, Box<dyn Error + Send + Sync>>;
fn component_id(&self) -> ComponentId;
fn check_interval(&self) -> Duration;
```

The actor system health check will:
1. Count actors in each state (Running, Error, Restarting, etc.)
2. Return `Healthy` if no actors in Error, `Degraded` if <10% in Error, `Unhealthy` if >10% in Error or tree corrupted
3. Report metadata: `total_actors`, `actors_by_state`, `tree_depth`, `restart_count`

This integrates naturally with `HealthMonitor.register_check()`.

## R6: Mailbox Implementation via Tokio Channels

**Decision**: Use `tokio::sync::mpsc` for tell (bounded/unbounded) and `tokio::sync::oneshot` for ask reply channels.

**Rationale**:
- `mpsc::channel(capacity)` for bounded mailbox — `try_send` returns `TrySendError::Full` which maps to `ActorError::MailboxFull`
- `mpsc::unbounded_channel()` for unbounded mailbox — always succeeds
- `oneshot::channel()` for ask pattern — sender wraps message + oneshot::Sender, actor cell routes reply through the oneshot

These are the standard Tokio primitives for this pattern, well-tested and production-ready. The `mpsc::Sender` is `Clone + Send + Sync`, making `ActorRef` automatically thread-safe.

**Alternatives considered**:
- crossbeam channels: rejected because crossbeam channels are sync, not async. Tokio mpsc integrates with the async runtime natively.
- flume: viable alternative but adds an external dependency. Tokio mpsc is already a workspace dependency.
- Custom ring buffer: rejected — premature optimization with no evidence of mpsc being a bottleneck.

## R7: Actor Cell Architecture

**Decision**: Wrap each actor in an `ActorCell<A: Actor>` that manages the message loop, state, lifecycle transitions, and supervision notifications.

**Rationale**: The actor cell is the internal runtime wrapper that:
1. Owns the actor instance (`A`) and its state (`A::State`)
2. Runs the message processing loop (`mpsc::Receiver` → `handle_message`)
3. Manages lifecycle state transitions (Initializing → Running → Stopping → Terminated)
4. Catches panics via `tokio::spawn` + `JoinHandle` error inspection
5. Notifies the supervisor on failure/stop via a supervisor notification channel

This is the standard "actor cell" pattern from Akka/Actix. It separates the user-facing API (Actor trait) from the runtime machinery.

**Alternatives considered**:
- No wrapper (user implements the loop): rejected — too much boilerplate per actor, error-prone lifecycle management.
- Process-per-actor (OS threads): rejected — doesn't scale to 1000+ actors.

## R8: Ask Pattern Type Design

**Decision**: Use an enum-based ask pattern where the message type includes a reply variant wrapping a `oneshot::Sender`.

**Rationale**: The existing `Actor` trait's `handle_message` returns `Result<(), Self::Error>`. To support ask (request-response), the actor needs a way to send back a response. Two approaches:

Option A: Modify Actor trait to return `Result<Option<Response>, Error>` — rejected, would change the core trait.

Option B (chosen): The message enum includes variants with embedded reply channels:
```rust
enum MyMessage {
    Tell(String),                                    // fire-and-forget
    Ask { query: String, reply: oneshot::Sender<String> },  // request-response
}
```

The `ActorRef<M>` provides a typed `ask` method that constructs the Ask variant, injects the oneshot sender, and returns a future that resolves to the response. This requires the message type to support a `with_reply_sender` pattern, which can be facilitated by a `Message` trait or macro.

## R9: Graceful Shutdown Strategy

**Decision**: Reverse-start-order shutdown with mailbox drain.

**Rationale**: FR-014 requires reverse-start order (leaves first, root last). The implementation:
1. Walk the supervision tree from leaves to root
2. For each actor: close the mailbox sender (no new messages accepted)
3. Wait for in-flight messages to be processed (with a timeout)
4. Call `post_stop` on the actor
5. Mark actor as Terminated
6. Move to parent level

The timeout prevents infinite hangs if an actor is stuck processing a message. Default timeout: 5 seconds per actor, configurable via ActorSystem settings.
