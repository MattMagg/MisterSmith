# Prerequisite Readiness Checklist: Phase 7.5 Blockers

**Purpose**: Capture the current readiness state for the six Phase 7.5 hardening items that were
previously treated as blanket blockers for Phase 9 subphases `9.4` and `9.5`.

**Created**: 2026-03-06  
**Updated**: 2026-03-06  
**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md) | [tasks.md](../tasks.md) | [analyze.md](../analyze.md)

## Scope Guardrail

This checklist tracks prerequisite readiness only. A checked item means the item has been either:

1. resolved at the relevant seam, or
2. explicitly verified as non-blocking for the current Gate 9 implementation path.

Unchecked items would be true blockers. Checked items are not an assertion that every Phase 7.5
follow-on is complete; they only mean Phase 9 can proceed honestly without hiding unresolved work.

## Source Map

| Source | Why it matters |
| ------ | -------------- |
| `ROADMAP.md:500-514` | Canonical definition of Phase 7.5 and its six pre-Phase-9 hardening items |
| `docs/audits/2026-03-05-implementation-deviation-report.md:225-232` | Explains why security and heartbeat gaps were originally carried as cross-phase risks |
| `docs/audits/2026-03-05-implementation-deviation-report.md:308-317` | Approved blocker inventory for the original Phase 7.5 follow-on set |
| `specs/009-phase9-llm-provider-integration/spec.md:66-80` | Phase 9 keeps the six items visible as prerequisites rather than silently absorbing them |
| `specs/009-phase9-llm-provider-integration/plan.md:178-181` | Planning posture: prerequisite work stays bounded and explicit |
| `specs/009-phase9-llm-provider-integration/tasks.md:43-58` | Task-level scope guardrail for blocker handling |
| `spec/core-architecture/async-patterns.md:1939-2315` | ToolBus and agent-as-tool boundary that Phase 9 must extend safely |
| `spec/core-architecture/coding-standards.md:1596-1799` | Permission, timeout, audit, and testing expectations for tool execution |

## Current Readiness Status

| ID | Hardening item | Current readiness status | Gate 9 impact |
| -- | -------------- | ------------------------ | ------------- |
| `P75-01` | Tool permissions and audit boundary | Cleared for current Gate 9 path | ToolBus permission and audit path is implemented; message-layer wiring stays deferred |
| `P75-02` | Router balancing (`round-robin`, `least-loaded`) | Deferred but verified non-blocking | Non-blocking while Gate 9 stays on Planner/Critic/Executor + Orchestrator seams |
| `P75-03` | Memory metadata, timestamps, versions, and access counts | Deferred but verified non-blocking | Non-blocking because current Gate 9 does not depend on Memory-agent metadata semantics |
| `P75-04` | Heartbeat receiver and failure detection | Deferred but verified non-blocking | Does not gate current Gate 9 if validation avoids receiver-side liveness claims |
| `P75-05` | Supervisor delegation | Deferred but verified non-blocking | Non-blocking while Gate 9 uses existing runtime supervision seams and avoids full `SupervisorAgent` claims |
| `P75-06` | Priority mailbox wiring | Deferred but verified non-blocking | Does not gate current Gate 9 because no current Phase 9 task requires priority-ordered actor mailboxes |

## Readiness Checks

- [x] `P75-01` The ToolBus permission, timeout, and audit boundary required by current Gate 9 scope
  is verified at the agent layer.
  Current state: `crates/mister-smith-agents/src/tool_bus.rs:82-425` now exposes a
  security-aware `ToolBus`, principal-scoped discovery, native and MCP invocation backends, shared
  timeout enforcement, audit recording, and typed error mapping. The coverage in
  `crates/mister-smith-agents/tests/tool_bus_tests.rs:89-239` verifies namespace-filtered
  discovery, successful audited invocation, blocked unauthorized invocation, and timeout handling.
  Gate decision: clears the only direct prerequisite that actually blocked current Gate 9 work.

- [x] `P75-02` Router balancing is still deferred, but it is verified as non-blocking for the
  current Gate 9 path.
  Current state: `crates/mister-smith-agents/src/roles/router.rs:66-148` still implements
  ordered rule matching only, with no `round-robin` or `least-loaded` strategy selection.
  Gate decision: current Gate 9 work is anchored to Planner, Critic, Executor, Orchestrator, and
  ToolBus seams, not `RouterAgent`, so routing-balancer hardening remains deferred rather than a
  prerequisite for starting implementation.

- [x] `P75-03` Memory metadata hardening is still deferred, but it is verified as non-blocking for
  the current Gate 9 path.
  Current state: `crates/mister-smith-agents/src/roles/memory.rs:37-123` still stores plain
  `HashMap<String, Value>` entries plus `entry_count`, without timestamps, versions, or access
  counts.
  Gate decision: Gate 9 does not need Memory-agent metadata semantics to begin provider abstraction
  or role wiring, so this stays visible as deferred work rather than a prerequisite blocker.

- [x] `P75-04` Heartbeat receiver and phi-accrual detection remain deferred, but they are verified
  as non-blocking for the current Gate 9 path.
  Current state: `crates/mister-smith-agents/src/registry.rs:99-103` only updates heartbeat
  timestamps, while `crates/mister-smith-app/src/bridges.rs:25-100` contains app-layer
  heartbeat-to-detector scaffolding that is not wired into the agent registry path.
  Gate decision: current Gate 9 work may proceed as long as its validation does not claim
  receiver-side liveness or heartbeat-driven failover semantics.

- [x] `P75-05` Full `SupervisorAgent` delegation remains deferred, but it is verified as
  non-blocking for the current Gate 9 path.
  Current state: `crates/mister-smith-agents/src/roles/supervisor.rs:25-101` still models
  supervision as child-list bookkeeping, while `crates/mister-smith-agents/src/agent.rs:170-212`
  already exposes `spawn_supervised()` through the Phase 3 supervision runtime.
  Gate decision: Gate 9 can proceed without claiming that `SupervisorAgent` itself has already been
  refactored into full supervision-system delegation.

- [x] `P75-06` Priority-mailbox wiring remains deferred, but it is verified as non-blocking for the
  current Gate 9 path.
  Current state: `crates/mister-smith-agents/src/config.rs:61-67` still exposes
  `priority_mailbox`, `crates/mister-smith-agents/src/agent.rs:146-152` still forwards only
  mailbox capacity into spawn config, and `crates/mister-smith-actor/src/mailbox.rs:1-199`
  remains FIFO-only.
  Gate decision: no current Gate 9 task requires urgent mailbox ordering, so this remains deferred
  Phase 7.5 work rather than a prerequisite for starting implementation.

## Exit Condition

Phase 9 may proceed when its implementation and validation remain inside the currently verified
Gate 9 seams:

1. Planner, Critic, Executor, and Orchestrator role integration
2. provider-neutral model contracts and provider adapters
3. ToolBus-backed tool export and tool execution

If later Gate 9 work expands into `RouterAgent`, `MemoryAgent`, receiver-side heartbeat semantics,
`SupervisorAgent` delegation, or priority mailboxes, the corresponding prerequisite item must be
reopened instead of being assumed solved.
