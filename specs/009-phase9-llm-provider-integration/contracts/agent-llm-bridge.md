# Contract: Agent-LLM Bridge

## Overview

Phase 9 adds LLM-backed behavior to the existing agent system as an optional capability. The bridge
must preserve the current orchestration and supervision boundaries while wiring a selected
`ModelProvider` into Planner, Critic, and Executor roles.

## Source Map

| Source | Contract impact |
| ------ | --------------- |
| `spec/data-management/agent-orchestration.md` §10.4 | Keeps Planner, Critic, Executor, and Orchestrator wiring inside existing coordination seams. |
| `spec/data-management/message-schemas.md` §5 | Confirms hook-event subjects remain deferred and are not added by this bridge. |
| `spec/agent-domains/SPECIALIZED_AGENT_DOMAINS_ANALYSIS.md` §15 | Keeps Neural/AI Operations role work out of this contract. |
| `spec/core-architecture/type-definitions.md` | Anchors `AgentType` boundaries and shared error typing for the role bridge. |
| `spec/core-architecture/async-patterns.md` | Preserves agent-as-tool and ToolBus boundaries while providers are wired into roles. |
| `spec/core-architecture/coding-standards.md` | Requires feature gating, typed errors, permission handling, and explicit validation coverage. |

## Feature Gating

```toml
[features]
llm = ["dep:mister-smith-llm"]
```

The `mister-smith-agents` crate must build without `mister-smith-llm` when the `llm` feature is not
enabled.

## Integration Surface

The bridge must extend the existing files and seams:

- `src/agent.rs`
- `src/orchestrator.rs`
- `src/errors.rs`
- `src/roles/planner.rs`
- `src/roles/critic.rs`
- `src/roles/executor.rs`

## Behavioral Contract

### Planner

- Accepts a high-level task and context.
- Calls the configured `ModelProvider` when the `llm` feature is enabled.
- Returns a structured subtask decomposition that the existing Orchestrator can consume.
- Must not leak provider-specific response types beyond the role boundary.

### Critic

- Evaluates outputs using provider-backed reasoning when configured.
- Returns structured feedback using existing agent-system error handling.
- Remains feature-gated and optional.

### Executor

- Executes model-backed action flows when configured.
- May participate in tool-calling loops through the ToolBus bridge.
- Must preserve existing timeout and error semantics.

### Orchestrator

- Continues to own the decompose -> assign -> aggregate flow.
- Consumes structured Planner output through existing scheduler and team paths.
- Must not gain provider-specific branching beyond selecting the configured `ModelProvider`.

## Gate 9 Contract

The bridge is complete when this flow succeeds for both Anthropic and OpenAI:

1. Planner receives a high-level task.
2. Planner calls a real `ModelProvider`.
3. The model returns a structured subtask decomposition.
4. Orchestrator assigns subtasks to Workers through existing orchestration paths.
5. Tool calls, when requested, round-trip through the ToolBus.

## Blocker Contract

The following remain prerequisites or blockers for `9.4` and `9.5`, not in-scope deliverables:

- Tool permission and audit hardening
- Router balancing strategies
- Memory metadata and versioning
- Heartbeat receiver and failure detection
- Supervisor delegation to the Phase 3 supervision system
- Priority mailbox wiring

If any unresolved item prevents reliable Planner, Critic, Executor, or ToolBus behavior, the plan
must report it as a blocker instead of extending Phase 9 scope.
