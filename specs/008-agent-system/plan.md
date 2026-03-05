# Implementation Plan: Phase 7 — Agent System

**Branch**: `008-agent-system` | **Date**: 2026-03-05 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/008-agent-system/spec.md`

## Summary

Build the `mister-smith-agents` crate — the multi-agent orchestration layer that composes Phase 1-6 foundations (actors, supervision, transport, security, persistence) into a coordinated agent runtime. The crate implements agent lifecycle management, inter-agent communication over NATS, task scheduling with durable delivery, team orchestration with fan-out/fan-in patterns, a tool registry with RBAC-gated invocation and MCP bridge, and nine specialized agent roles (Supervisor, Worker, Coordinator, Monitor, Planner, Executor, Critic, Router, Memory).

## Technical Context

**Language/Version**: Rust, MSRV 1.88.0
**Primary Dependencies**: mister-smith-core (types, traits), mister-smith-actor (ActorCell, ActorRef, mailbox), mister-smith-supervision (SupervisedSystem, restart strategies), mister-smith-transport (Transport, DurableTransport, MessageEnvelope), mister-smith-nats (NatsTransport, JetStream), mister-smith-mcp (MCP client/server, tool bridge), mister-smith-security (PolicyEngine, JwtManager, AuditLogger), mister-smith-persistence (repositories, state persistence), mister-smith-events (EventBus), mister-smith-monitoring (HealthMonitor, phi accrual)
**Storage**: PostgreSQL (via Phase 6 persistence layer), JetStream KV (via Phase 6 dual-store)
**Testing**: cargo test (unit + integration), cargo clippy -- -D warnings
**Target Platform**: Linux server (primary), macOS (development)
**Project Type**: Library crate (`mister-smith-agents`)
**Performance Goals**: <50ms agent spawn, <5ms message latency, 500+ agents/node, 1K tasks/sec
**Constraints**: Existing Actor trait uses associated types (Message, State, Error, Response) — agents must implement this. Agent trait in core extends Tool — agents are both actors and tools.
**Scale/Scope**: 1 new crate, ~15 modules, 9 agent role implementations

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Canonical Single Source | PASS | Agent types use canonical `AgentId`, `AgentType`, `AgentState` from `mister-smith-core/src/ids.rs` and `enums.rs`. No redefinitions. |
| II. Spec-First Design | PASS | Spec written and validated at `specs/008-agent-system/spec.md` before any implementation code. |
| III. Phase-Gated Build Order | PASS | Phases 1-6 complete with 882+ passing tests. Phase 7 prerequisites implemented and committed. |
| IV. Model-Agnostic Architecture | PASS | Spec explicitly states LLM/model integration is out of scope. Agents are model-agnostic containers. |
| V. Erlang/OTP Fault Tolerance | PASS | Supervision integration is FR-1 core requirement. Uses Phase 3 SupervisedSystem with OneForOne/OneForAll/RestForOne. Actor model with bounded mailboxes. |
| VI. Evidence-Based Validation | PASS | Gate 7 validation requires end-to-end orchestration test (Coordinator → Workers → Supervisor restart → result aggregation). |
| VII. Explicit Dependency Management | PASS | All 11 upstream dependencies enumerated in spec with specific usage. |

## Project Structure

### Documentation (this feature)

```text
specs/008-agent-system/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   ├── agent-trait.md       # Agent contract for implementors
│   ├── tool-bus.md          # Tool registry and invocation contract
│   └── team-orchestration.md # Team creation and management contract
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/mister-smith-agents/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Crate root, re-exports
│   ├── agent.rs                # Core AgentRuntime wrapper (Actor + Agent trait bridge)
│   ├── registry.rs             # AgentRegistry — in-memory + NATS discovery
│   ├── messaging.rs            # Inter-agent messaging helpers (send, request, broadcast)
│   ├── scheduler.rs            # Task scheduler — matching, assignment, deadline monitoring
│   ├── team.rs                 # Team — creation, lifecycle, disbanding
│   ├── orchestrator.rs         # Coordinator orchestration logic — decompose, assign, aggregate
│   ├── tool_bus.rs             # ToolBus — registry, discovery, invocation, MCP bridge
│   ├── heartbeat.rs            # Heartbeat emitter and liveness monitor
│   ├── config.rs               # Agent configuration types
│   ├── errors.rs               # AgentSystemError enum
│   └── roles/
│       ├── mod.rs              # Role module root
│       ├── supervisor.rs       # Supervisor agent
│       ├── worker.rs           # Worker agent
│       ├── coordinator.rs      # Coordinator agent
│       ├── monitor.rs          # Monitor agent
│       ├── planner.rs          # Planner agent
│       ├── executor.rs         # Executor agent
│       ├── critic.rs           # Critic agent
│       ├── router.rs           # Router agent
│       └── memory.rs           # Memory agent
└── tests/
    ├── lifecycle_tests.rs      # Agent spawn, stop, restart, state recovery
    ├── messaging_tests.rs      # Inter-agent communication patterns
    ├── scheduling_tests.rs     # Task assignment and deadline monitoring
    ├── team_tests.rs           # Team orchestration, failure recovery
    ├── tool_bus_tests.rs       # Tool registration, discovery, invocation, MCP
    └── role_tests.rs           # Specialized role behavior tests
```

**Structure Decision**: Single new crate `mister-smith-agents` following the workspace pattern. All agent infrastructure + 9 role implementations in one crate to avoid premature splitting. The `roles/` submodule isolates specialized implementations from infrastructure code.

## Complexity Tracking

No constitution violations. No complexity justification needed.
