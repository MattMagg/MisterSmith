# Phase 3: Actor System and Supervision

## Purpose and Scope

Define the core concurrency model: typed actor primitives and hierarchical supervision behavior.
This is the architectural chokepoint for all downstream agent orchestration work.

### In Scope

- Mailbox and actor reference semantics
- Actor lifecycle and actor-system ownership boundaries
- Supervision tree topology and restart policy behavior
- Failure detection, escalation, and restart scopes

### Out of Scope

- External transport bindings
- Agent-specific orchestration strategies
- Persistence implementation details

## Inputs and Dependencies

### Upstream Dependencies

- Phase 2 (runtime, events, async controls, monitoring)
- Phase 1 (types/traits used by actor and supervisor contracts)

### Key Source Inputs

- `ROADMAP.md` Phase 3 and Gate 3
- `VALIDATION_REPORT.md` resolution of `SupervisionStrategy` inconsistencies

### Required Specification Anchors

- `spec/core-architecture/async-patterns.md` (Actor model section)
- `spec/core-architecture/component-architecture.md`
- `spec/core-architecture/supervision-trees.md`
- `spec/core-architecture/supervision-and-events.md`
- `spec/core-architecture/type-definitions.md`

## Outputs and Downstream Consumers

### Produces

- Actor primitives and typed interaction model (`Mailbox`, `ActorRef`, actor lifecycle)
- Supervision strategy model (`RestartPolicy`, `RestartScope`, escalation rules)
- Event and failure semantics consumed by orchestration and operations

### Consumed By

- Phase 7 agent lifecycle and orchestration
- Phase 8 process shutdown and resilience expectations
- Phase 4/6 components that rely on stable actor execution behavior

## Gate Criteria and Validation

### Gate Criteria

- Actor lifecycle states align with canonical `AgentState` transitions
- Supervision model uses `RestartPolicy` and `RestartScope` consistently
- Failure and restart semantics are tied to emitted system events
- Hierarchical supervision behavior is defined for key restart policies

### Validation Approach

- Cross-check `supervision-trees.md`, `supervision-and-events.md`, and `type-definitions.md`
- Verify no reintroduction of conflicting pre-validation strategy names
- Ensure Phase 7 references map directly to Phase 3 contracts

### Validation Evidence

- Consistent restart terminology/state transitions across core and data-management docs
- Explicit dependency references from agent lifecycle/orchestration specs

## Official-Doc Best Practices

- Use Tokio channel/task primitives with explicit bounded mailbox behavior to prevent unbounded actor queues ([Tokio mpsc](https://docs.rs/tokio/1.49.0/tokio/sync/mpsc/) and [Tokio task](https://docs.rs/tokio/1.49.0/tokio/task/)).
- Keep supervision semantics explicit and hierarchical (restart, escalation, isolation) following OTP principles ([Erlang/OTP supervision principles](https://www.erlang.org/doc/design_principles/sup_princ.html)).
- Encode restart/backoff policy in data structures, not implicit control flow, to preserve determinism and testability ([Tokio time utilities](https://docs.rs/tokio/1.49.0/tokio/time/)).

## Known Risks / Unknowns

### Risks

- Restart semantics can be underspecified at failure boundaries
- Mailbox/backpressure behavior can conflict with transport assumptions
- Supervision escalation policies can drift across documents

### Required Follow-ups

- Treat core supervision definitions as canonical before downstream edits
- Revalidate agent orchestration assumptions after supervision semantics changes

## Authoritative Spec Files

- `spec/core-architecture/async-patterns.md`
- `spec/core-architecture/supervision-trees.md`
- `spec/core-architecture/supervision-and-events.md`
- `spec/core-architecture/type-definitions.md`
- `spec/core-architecture/component-architecture.md`
