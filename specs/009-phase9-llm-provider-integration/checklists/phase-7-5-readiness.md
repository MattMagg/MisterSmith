# Prerequisite Readiness Checklist: Phase 7.5 Blockers

**Purpose**: Capture the current readiness state for the six Phase 7.5 hardening items that can
block Phase 9 subphases `9.4` and `9.5` without redefining that hardening as Phase 9 scope

**Created**: 2026-03-06
**Feature**: [spec.md](../spec.md) | [plan.md](../plan.md) | [tasks.md](../tasks.md) | [analyze.md](../analyze.md)

## Scope Guardrail

This checklist tracks prerequisite readiness only. A checked item means the blocker has been
verified or resolved outside the Phase 9 implementation scope. Unchecked items remain blockers for
`9.4` and `9.5` and must not be silently absorbed into Phase 9 tasks.

## Source Map

| Source | Why it matters |
| ------ | -------------- |
| `ROADMAP.md:500-514` | Canonical definition of Phase 7.5 and its six pre-Phase-9 hardening items |
| `docs/2026-03-05-implementation-deviation-report.md:225-232` | Explains why security and heartbeat gaps remain cross-phase risks |
| `docs/2026-03-05-implementation-deviation-report.md:308-315` | Approved blocker inventory for Phase 7.5 |
| `specs/009-phase9-llm-provider-integration/spec.md:66-80` | Phase 9 keeps the six items as prerequisites and blockers |
| `specs/009-phase9-llm-provider-integration/plan.md:178-181` | Planning posture: blockers stay visible and out of Phase 9 scope |
| `specs/009-phase9-llm-provider-integration/tasks.md:43-58` | Task-level scope guardrail for blocker handling |
| `spec/core-architecture/async-patterns.md:1939-2315` | ToolBus and agent-as-tool boundary that Phase 9 must extend safely |
| `spec/core-architecture/coding-standards.md:1596-1799` | Permission, timeout, audit, and testing expectations for tool execution |

## Current Blocker Status

| ID | Hardening item | Current status | Phase 9 impact |
| -- | -------------- | -------------- | -------------- |
| `P75-01` | Security integration for agent messaging, tool permissions, and audit logging | Unresolved / unverified | Blocks `9.5` directly and can invalidate `9.4` role integration evidence |
| `P75-02` | Router balancing (`round-robin`, `least-loaded`) | Unresolved | Can block honest `9.4` validation if provider-backed planning depends on router behavior |
| `P75-03` | Memory metadata, timestamps, versions, and access counts | Unresolved | Keeps Memory-agent hardening outside scope but still visible if `9.4` workflows depend on shared context quality |
| `P75-04` | Heartbeat receiver and failure detection | Unresolved | Blocks `9.4` and `9.5` readiness when role liveness and orchestration health cannot be verified |
| `P75-05` | Supervisor delegation to Phase 3 `SupervisedSystem` | Partial / unverified | Blocks `9.4` if provider-backed role execution needs real supervision semantics |
| `P75-06` | Priority mailbox wiring | Unresolved | Can block `9.4`/`9.5` if message ordering or urgent tool work must be validated honestly |

## Readiness Checks

- [ ] `P75-01` Security integration for agent messaging, tool permissions, and audit logging is
  verified at the agent layer.
  Current state: `crates/mister-smith-agents/src/tool_bus.rs:31-152` registers and discovers tools
  but does not expose a permission-checked invocation path or audit integration, while
  `crates/mister-smith-security/src/middleware/nats_mw.rs:16-87` and
  `crates/mister-smith-security/src/audit/mod.rs:19-120` show the adjacent security infrastructure
  already exists outside the agent crate.
  Evidence: `docs/2026-03-05-implementation-deviation-report.md:225-232` explicitly says Phase 7
  uses none of the Phase 5 security wiring, and
  `specs/007-phase7-agent-system/tasks.md:128-130` still leaves ToolBus permission-filtering and
  audit-log coverage open.
  Blocks: `9.5` cannot claim honest ToolBus-backed tool-calling parity while permission and audit
  boundaries remain unverified.

- [ ] `P75-02` Router balancing exposes verified `round-robin` and `least-loaded` behavior instead
  of placeholder routing only.
  Current state: `crates/mister-smith-agents/src/roles/router.rs:10-147` defines rule matching and
  first-match routing only; it does not model balancing strategy selection or worker load-aware
  dispatch.
  Evidence: `ROADMAP.md:510` and
  `docs/2026-03-05-implementation-deviation-report.md:313` list these strategies as explicit
  pre-Phase-9 hardening.
  Blocks: keep out of Phase 9 scope, but treat as a blocker if `9.4` validation needs real router
  behavior rather than a simplified placeholder.

- [ ] `P75-03` Memory-agent entries carry timestamps, versions, and access counts.
  Current state: `crates/mister-smith-agents/src/roles/memory.rs:37-123` stores a plain
  `HashMap<String, Value>` plus `entry_count`; it has no metadata envelope for versioning,
  timestamps, or access frequency.
  Evidence: `ROADMAP.md:511` and
  `docs/2026-03-05-implementation-deviation-report.md:314` define this metadata as Phase 7.5
  follow-on work.
  Blocks: do not fold Memory hardening into Phase 9, but keep it visible whenever shared-context
  quality is assumed by `9.4` role behavior.

- [ ] `P75-04` Heartbeat reception and phi-accrual failure detection are wired into the agent
  registry.
  Current state: `crates/mister-smith-agents/src/registry.rs:31-121` updates heartbeat timestamps
  but has no receiver loop or liveness monitor, while
  `crates/mister-smith-monitoring/src/failure_detector.rs:14-137` provides the reusable
  phi-accrual detector.
  Evidence: `docs/2026-03-05-implementation-deviation-report.md:232-233` calls out the missing
  receiver-side wiring, and `specs/007-phase7-agent-system/tasks.md:93` still leaves the registry
  liveness-monitor task open.
  Blocks: `9.4` and `9.5` stay blocker-sensitive because Gate 9 cannot honestly validate
  provider-backed role health without the receiver-side readiness seam.

- [ ] `P75-05` The Supervisor role delegates child lifecycle management to the Phase 3 supervision
  system rather than maintaining a local list only.
  Current state: `crates/mister-smith-agents/src/agent.rs:166-207` can spawn actors through
  `SupervisedSystem`, but `crates/mister-smith-agents/src/roles/supervisor.rs:25-101` still models
  supervision as a `Vec<AgentId>` with register/remove/query operations only.
  Evidence: `ROADMAP.md:513` and
  `docs/2026-03-05-implementation-deviation-report.md:316` explicitly call for delegation to the
  Phase 3 supervision system.
  Blocks: keep the supervisor refactor outside Phase 9 scope, but treat it as a blocker if
  provider-backed Planner/Critic/Executor execution depends on real supervision semantics.

- [ ] `P75-06` The `priority_mailbox` config flag is wired to actual priority-ordered processing.
  Current state: `crates/mister-smith-agents/src/config.rs:49-69` exposes
  `priority_mailbox: bool`, but `crates/mister-smith-agents/src/agent.rs:141-148` passes only
  mailbox capacity into spawn config and `crates/mister-smith-actor/src/mailbox.rs:1-188`
  implements FIFO bounded or unbounded queues only.
  Evidence: `ROADMAP.md:514`,
  `docs/2026-03-05-implementation-deviation-report.md:317`, and
  `specs/007-phase7-agent-system/tasks.md:78` all keep priority-aware ordering in the unfinished
  or unverified bucket.
  Blocks: if `9.4` or `9.5` claims rely on urgent message ordering or priority-aware tool work,
  this prerequisite must be cleared first rather than hidden inside Phase 9.

## Exit Condition

The Phase 9 prep gate remains blocked until each required prerequisite above is either:

1. verified with current evidence and tests outside Phase 9 scope, or
2. explicitly recorded as still unresolved so `9.4` and `9.5` do not proceed under false
   readiness assumptions.
