# Contract: Agent Trait Bridge

## Overview

Defines how Phase 7 agents bridge the existing `Actor` trait (Phase 3, `&mut self`, associated types) with the `Agent` trait (Phase 1, `&self`, `Tool` supertrait) to enable both supervision-tree integration and tool-registry compatibility.

## AgentRuntime Wrapper

Each specialized agent role (Worker, Coordinator, etc.) is a struct implementing the `Actor` trait with role-specific associated types. `AgentRuntime<A: Actor>` wraps the actor and provides the `Agent` trait interface.

### Actor Side (Phase 3 compatibility)

```
trait Actor {
    type Message: Send + 'static;
    type State: Send + 'static;
    type Error: Send + Error + 'static;
    type Response: Send + 'static;

    async fn handle_message(&mut self, msg: Self::Message, state: &mut Self::State) -> Result<Self::Response, Self::Error>;
    fn pre_start(&mut self) -> Result<(), Self::Error>;
    fn post_stop(&mut self) -> Result<(), Self::Error>;
    fn actor_id(&self) -> AgentId;
}
```

### Agent Side (Tool/orchestration compatibility)

```
trait Agent: Tool {
    type Context: Send + Sync;
    type Error: Send + Error + 'static;

    async fn process(&self, message: Value) -> Result<Value, Self::Error>;
    fn role(&self) -> AgentType;
    fn context(&self) -> &Self::Context;
    async fn initialize(&mut self, context: Self::Context) -> Result<(), Self::Error>;
}
```

### Bridge Contract

- `AgentRuntime<A>` holds `ActorRef<A>` for message passing and an `Arc<AgentContext>` for shared state.
- `process()` serializes the JSON message into `A::Message`, sends via `ActorRef::ask()`, and deserializes the response.
- `role()` delegates to the wrapped agent's type declaration.
- Tool registration exposes the agent's capabilities via `Tool::schema()` and `Tool::capabilities()`.

### Message Type Contract

Each role defines a message enum:

```
enum WorkerMessage {
    AssignTask(TaskAssignment),
    CancelTask(TaskId),
    QueryStatus,
}

enum CoordinatorMessage {
    SubmitTask(TaskAssignment),
    SubtaskResult(TaskId, Value),
    TeamMemberFailed(AgentId),
}
```

### State Persistence Contract

- Agent state is persisted on: `pre_start` (initial), state transitions (Running, Paused, Error), and periodic snapshots (configurable interval).
- State recovery on restart: `pre_start` loads last persisted state from `mister-smith-persistence` before processing messages.
- State format: `serde_json::Value` stored via existing `upsert_state()` in the persistence layer.

## Implementor Requirements

To create a new agent role:

1. Define a message enum implementing `Send + 'static`
2. Define a state struct implementing `Send + 'static + Serialize + Deserialize`
3. Implement `Actor` with the message and state types
4. Register the role's capabilities and accepted message types
5. Provide default configuration via `AgentConfig`
