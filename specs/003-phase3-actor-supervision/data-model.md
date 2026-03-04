# Data Model: Phase 3 — Actor System & Supervision Trees

**Date**: 2026-03-04
**Feature**: [spec.md](spec.md)

## Entity Relationship Overview

```text
ActorSystem 1──owns──1 SupervisionTree
ActorSystem 1──tracks──* ActorHandle
ActorHandle 1──wraps──1 ActorCell<A>
ActorCell<A> 1──owns──1 A (Actor impl)
ActorCell<A> 1──owns──1 A::State
ActorCell<A> 1──receives──1 Mailbox<M> (receiver side)
ActorRef<M> *──sends-to──1 Mailbox<M> (sender side)
SupervisionTree 1──contains──* SupervisorNode
SupervisorNode 1──supervises──* ChildEntry
ChildEntry 1──references──1 ActorHandle
SupervisorNode *──escalates-to──1 SupervisorNode (parent)
```

## Core Entities — `mister-smith-actor` crate

### ActorRef\<M\>

The external handle for communicating with an actor. Generic over message type `M`.

| Field | Type | Description |
|-------|------|-------------|
| sender | `MailboxSender<M>` | Enum: Bounded(mpsc::Sender\<M\>) or Unbounded(UnboundedSender\<M\>) |
| actor_id | `AgentId` | Identity of the target actor |
| state | `Arc<AtomicU8>` | Shared reference to actor's current AgentState |

**Traits**: `Clone`, `Send`, `Sync`, `Debug`

**Methods**:
- `tell(message: M) -> Result<(), ActorError>` — fire-and-forget, returns MailboxFull or ActorStopped
- `ask<R>(message: M, timeout: Duration) -> Result<R, ActorError>` where M carries a oneshot::Sender\<R\> — request-response
- `actor_id() -> AgentId`
- `is_alive() -> bool` — checks if actor is in Running/Restarting state

**Validation**: tell fails with `MailboxFull` if bounded mailbox is at capacity. tell/ask fail with `ActorStopped` if actor is Terminated.

### MailboxSender\<M\> (internal enum)

| Variant | Type | Description |
|---------|------|-------------|
| Bounded | `mpsc::Sender<M>` | Capacity-limited, returns TrySendError on full |
| Unbounded | `mpsc::UnboundedSender<M>` | No capacity limit |

### MailboxConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| capacity | `Option<usize>` | `Some(1000)` | None = unbounded, Some(n) = bounded to n |

### ActorCell\<A: Actor\> (internal)

The runtime wrapper that manages an actor's execution. Not public API.

| Field | Type | Description |
|-------|------|-------------|
| actor | `A` | The actor instance |
| state | `A::State` | The actor's mutable state |
| actor_id | `AgentId` | Actor identity (preserved across restarts) |
| lifecycle_state | `Arc<AtomicU8>` | Current AgentState as u8 |
| mailbox_rx | `mpsc::Receiver<Envelope<A::Message>>` | Message receiver |
| supervisor_tx | `Option<mpsc::Sender<SupervisionEvent>>` | Notification channel to supervisor |
| event_publisher | `Option<Arc<dyn EventPublisher>>` | For emitting lifecycle events |

**Lifecycle state machine**:
```text
Initializing → Running → Stopping → Terminated
                  ↓                      ↑
                Error → Restarting → Running
                  ↓
              Terminated (if Temporary scope)
```

### Envelope\<M\> (internal)

Wraps a message with optional reply channel for ask pattern.

| Field | Type | Description |
|-------|------|-------------|
| message | `M` | The actual message payload |
| reply_tx | `Option<oneshot::Sender<Result<serde_json::Value, ActorError>>>` | Reply channel for ask |

### ActorContext

Provided to actors during spawning to enable child spawning and system access.

| Field | Type | Description |
|-------|------|-------------|
| actor_id | `AgentId` | This actor's identity |
| system | `ActorSystemRef` | Weak reference to the ActorSystem |
| self_ref | `ActorRef<Self::Message>` | Reference to self (for passing to children) |

### ActorSystemConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| default_mailbox_capacity | `Option<usize>` | `Some(1000)` | Default mailbox capacity |
| shutdown_timeout | `Duration` | `5s` | Per-actor shutdown timeout |
| enable_events | `bool` | `true` | Whether to emit lifecycle events |
| ask_default_timeout | `Duration` | `30s` | Default timeout for ask messages |

### ActorSystem

| Field | Type | Description |
|-------|------|-------------|
| config | `ActorSystemConfig` | System configuration |
| actors | `RwLock<HashMap<AgentId, ActorHandle>>` | Registry of all actor handles |
| supervision_tree | `Arc<SupervisionTree>` | The supervision hierarchy |
| event_publisher | `Option<Arc<dyn EventPublisher>>` | EventBus integration |
| shutdown | `Arc<AtomicBool>` | Global shutdown signal |
| start_order | `RwLock<Vec<AgentId>>` | Actor start order for reverse-shutdown |

**Methods**:
- `new(config: ActorSystemConfig) -> Self`
- `with_event_publisher(publisher: Arc<dyn EventPublisher>) -> Self`
- `spawn<A: Actor>(actor: A, state: A::State, config: SpawnConfig) -> Result<ActorRef<A::Message>, ActorError>`
- `spawn_supervised<A: Actor>(actor: A, state: A::State, config: SpawnConfig, supervisor_id: AgentId) -> Result<ActorRef<A::Message>, ActorError>`
- `get_ref<M>(actor_id: &AgentId) -> Option<ActorRef<M>>`
- `shutdown() -> Result<(), SystemError>`
- `actor_count() -> usize`

### SpawnConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| mailbox_config | `MailboxConfig` | capacity=1000 | Mailbox settings |
| restart_scope | `RestartScope` | `Permanent` | How this actor is restarted |

### ActorHandle (internal, type-erased)

| Field | Type | Description |
|-------|------|-------------|
| actor_id | `AgentId` | Actor identity |
| join_handle | `JoinHandle<()>` | Tokio task handle |
| lifecycle_state | `Arc<AtomicU8>` | Shared state reference |
| stop_tx | `oneshot::Sender<()>` | Signal to stop the actor |
| mailbox_sender | `Box<dyn Any + Send + Sync>` | Type-erased MailboxSender |

## Core Entities — `mister-smith-supervision` crate

### SupervisionTree

| Field | Type | Description |
|-------|------|-------------|
| nodes | `RwLock<HashMap<AgentId, SupervisorNode>>` | All supervisor nodes |
| root_id | `Option<AgentId>` | Root supervisor identity |
| event_publisher | `Option<Arc<dyn EventPublisher>>` | EventBus integration |

**Methods**:
- `new() -> Self`
- `add_supervisor(id: AgentId, strategy: SupervisionStrategy, parent: Option<AgentId>) -> Result<(), SupervisionError>`
- `add_child(supervisor_id: AgentId, child: ChildEntry) -> Result<(), SupervisionError>`
- `remove_child(supervisor_id: AgentId, child_id: AgentId) -> Result<(), SupervisionError>`
- `handle_failure(child_id: AgentId, error: ActorError) -> SupervisionDecision`
- `query_status() -> TreeStatus`
- `shutdown_order() -> Vec<AgentId>` — returns IDs in reverse-start order

### SupervisorNode

| Field | Type | Description |
|-------|------|-------------|
| id | `AgentId` | Supervisor identity |
| parent_id | `Option<AgentId>` | Parent supervisor (None for root) |
| children | `Vec<ChildEntry>` | Ordered list of children (start order) |
| strategy | `SupervisionStrategy` | Restart policy, limits, backoff |
| restart_history | `VecDeque<Instant>` | Timestamps of recent restarts |

### ChildEntry

| Field | Type | Description |
|-------|------|-------------|
| actor_id | `AgentId` | Child actor identity |
| restart_scope | `RestartScope` | Per-child restart behavior |
| start_order | `usize` | Position in start order (for RestForOne) |
| restart_count | `u32` | Cumulative restart count for this child |

### SupervisionDecision (enum)

| Variant | Description |
|---------|-------------|
| `Restart(Vec<AgentId>)` | Restart these actors (determined by policy) |
| `Escalate(AgentId, ActorError)` | Escalate to parent supervisor |
| `Stop(AgentId)` | Stop without restart (Temporary scope) |
| `Ignore` | No action (normal termination of Transient) |
| `Shutdown` | Root exhausted budget, shut down everything |

### SupervisionEvent (internal notification)

| Field | Type | Description |
|-------|------|-------------|
| child_id | `AgentId` | Which child is reporting |
| event_type | `SupervisionEventType` | What happened |
| error | `Option<ActorError>` | Error details if failure |
| correlation_id | `Uuid` | For distributed tracing |

### SupervisionEventType (internal enum)

| Variant | Description |
|---------|-------------|
| `ChildFailed` | Child actor returned error or panicked |
| `ChildStopped` | Child actor terminated normally |
| `ChildStarted` | Child actor started/restarted successfully |
| `RestartLimitExceeded` | Max restarts exceeded in time window |

### TreeStatus

| Field | Type | Description |
|-------|------|-------------|
| total_nodes | `usize` | All nodes in tree |
| nodes_by_state | `HashMap<AgentState, usize>` | Count per state |
| tree_depth | `usize` | Maximum depth from root |
| total_restarts | `u64` | Cumulative restart count |

### ActorSystemHealthCheck

Implements `HealthCheck` from monitoring crate.

| Field | Type | Description |
|-------|------|-------------|
| tree | `Arc<SupervisionTree>` | Reference to supervision tree |
| system | `ActorSystemRef` | Reference to actor system |

**Check logic**: Returns `Healthy` if no actors in Error state, `Degraded` if <10% in Error, `Unhealthy` if >10% or tree corrupted.

## State Transitions

### Actor Lifecycle States (maps to AgentState)

| From | To | Trigger |
|------|----|---------|
| — | Initializing | `ActorSystem.spawn()` called |
| Initializing | Running | `pre_start()` succeeds |
| Initializing | Error | `pre_start()` fails |
| Running | Stopping | Shutdown signal received |
| Running | Error | `handle_message()` returns Err or actor panics |
| Error | Restarting | Supervisor decides to restart |
| Error | Terminated | Supervisor decides not to restart (Temporary scope) |
| Restarting | Running | New instance created, `pre_start()` succeeds |
| Restarting | Error | New instance `pre_start()` fails |
| Stopping | Terminated | `post_stop()` completes |

### Supervision Decision Flow

```text
Child fails → Supervisor receives SupervisionEvent
  → Check RestartScope:
    - Temporary → Stop(child_id), don't restart
    - Transient + normal stop → Ignore
    - Permanent/Transient+error → Apply RestartPolicy:
      → Check restart_history against max_failures/failure_window:
        - Within budget → Restart(affected_ids per policy)
        - Budget exceeded → Escalate(parent_id, error)
          → Parent applies its own decision flow
          → If root exhausted → Shutdown
```
