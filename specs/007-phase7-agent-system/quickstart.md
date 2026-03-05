# Quickstart: Phase 7 — Agent System

## Prerequisites

- Phases 1-6 complete (all 882+ tests passing)
- Phase 7 prerequisites committed (DurableTransport, message idempotency, taxonomy freeze)
- NATS server running with JetStream enabled
- PostgreSQL with migrations 00001-00005 applied

## Build

```bash
cargo build --workspace        # Build all crates including mister-smith-agents
cargo test --workspace         # Run all tests
cargo clippy --workspace -- -D warnings
```

## Usage Example

### Spawn a Worker Agent

```rust
use mister_smith_agents::{AgentConfig, AgentRuntime, roles::Worker};
use mister_smith_actor::ActorSystem;
use mister_smith_nats::NatsTransport;

// Create actor system and transport
let system = ActorSystem::new(ActorSystemConfig::default());
let transport = NatsTransport::new(config);
transport.connect().await?;

// Configure and spawn a worker
let config = AgentConfig {
    agent_type: AgentType::Worker,
    heartbeat_interval: Duration::from_secs(5),
    mailbox_capacity: 1000,
    ..Default::default()
};
let worker = AgentRuntime::spawn(&system, Worker::new(handler), config, &transport).await?;
```

### Orchestrate a Team

```rust
use mister_smith_agents::roles::Coordinator;

// Spawn a coordinator
let coordinator = AgentRuntime::spawn(&system, Coordinator::new(decomposer, aggregator), coord_config, &transport).await?;

// Submit a task — coordinator handles decomposition, team assembly, and aggregation
coordinator.submit_task(TaskAssignment {
    task_type: "analysis".to_string(),
    input: json!({"data": "..."}),
    priority: 0,
    deadline: Some(Utc::now() + Duration::from_secs(300)),
    ..Default::default()
}).await?;
```

### Register and Invoke a Tool

```rust
use mister_smith_agents::ToolBus;

// Register the worker as a tool
tool_bus.register("analyzer", "data", worker.actor_ref(), schema, capabilities).await?;

// Another agent invokes the tool
let result = tool_bus.invoke(principal, "data", "analyzer", json!({"input": "..."}), Duration::from_secs(30)).await?;
```

## Gate 7 Validation

The end-to-end validation test (Gate 7 from ROADMAP.md):

1. Spawn a Coordinator with a task decomposer
2. Coordinator decomposes a multi-step task into subtasks
3. Coordinator assembles a team of Workers under a Supervisor
4. Workers execute subtasks and report results
5. Inject a Worker failure — Supervisor restarts it
6. Coordinator reassigns the incomplete subtask
7. Results aggregate back to the Coordinator
8. Verify: correct final result, no duplicate work, audit trail complete

```bash
cargo test --package mister-smith-agents -- gate7_end_to_end --nocapture
```

## Key Modules

| Module | Purpose |
|--------|---------|
| `agent.rs` | AgentRuntime — Actor/Agent bridge, spawn, lifecycle |
| `registry.rs` | AgentRegistry — in-memory + NATS discovery |
| `scheduler.rs` | Task scheduling and deadline monitoring |
| `team.rs` | Team creation, lifecycle, disbanding |
| `orchestrator.rs` | Task decomposition and result aggregation |
| `tool_bus.rs` | Tool registry, discovery, invocation, MCP bridge |
| `heartbeat.rs` | Heartbeat emission and liveness monitoring |
| `roles/*` | 9 specialized agent implementations |
