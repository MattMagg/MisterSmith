# Phase 6: Persistence and State

## Purpose and Scope

Define durable and distributed state contracts used by agents and workflows. This phase integrates
relational persistence and JetStream KV state patterns for reliability across restarts.

### In Scope

- PostgreSQL schema and access model
- JetStream KV-based distributed state patterns
- Persistence operations, repositories, and transaction guidance

### Out of Scope

- Core transport design
- Agent orchestration logic

## Inputs and Dependencies

### Upstream Dependencies

- Phase 2 (resource and connection lifecycle patterns)
- Phase 4 (NATS transport and message schemas)
- Phase 5 (credential and security constraints)

### Key Source Inputs

- `ROADMAP.md` Phase 6 and Gate 6
- `VALIDATION_REPORT.md` boundary consistency checks (NATS -> DB priority mapping)
- `VERSION_REFERENCE.md` sqlx/async-nats version baseline

### Required Specification Anchors

- `spec/data-management/postgresql-implementation.md`
- `spec/data-management/database-schemas.md`
- `spec/data-management/data-persistence.md`
- `spec/data-management/jetstream-kv.md`
- `spec/data-management/storage-patterns.md`
- `spec/data-management/persistence-operations.md`
- `spec/data-management/data-integration-patterns.md`
- `spec/data-management/connection-management.md`

## Outputs and Downstream Consumers

### Produces

- Relational persistence contract for tasks, state, and audit paths
- Distributed KV coordination/state contract for ephemeral operational state
- Persistence operation patterns used by agents and workflows

### Consumed By

- Phase 7 agent lifecycle/orchestration state tracking
- Phase 8 process and shutdown behavior requiring durable flush semantics
- Cross-phase validation for message/state boundary consistency

## Gate Criteria and Validation

### Gate Criteria

- SQL schema constraints and message-priority semantics align with message schema docs
- JetStream KV usage patterns match transport-layer assumptions
- Persistence operation patterns define transaction and consistency boundaries
- Data-integration guidance maps agent workflows to storage responsibilities

### Validation Approach

- Verify priority scale remains 0-4 across message and DB specifications
- Confirm persistence status/lifecycle enums align with agent lifecycle semantics
- Check references against current async-nats/sqlx expectations in repo docs

### Validation Evidence

- Consistent schema constraints between `database-schemas.md` and implementation guidance
- Explicit references from orchestration/lifecycle docs to persistence outputs

## Official-Doc Best Practices

- Keep SQL access patterns and transaction semantics aligned with SQLx 0.8 APIs and connection-pool behavior ([sqlx 0.8.6](https://docs.rs/sqlx/0.8.6/sqlx/)).
- Use JetStream KV through async-nats 0.46 KV APIs and watch/revision semantics for distributed coordination ([async-nats KV module](https://docs.rs/async-nats/0.46.0/async_nats/jetstream/kv/)).
- Keep relational constraints explicit (checks, indexes, foreign keys) using current PostgreSQL behavior and migration-safe patterns ([PostgreSQL docs](https://www.postgresql.org/docs/current/)).

## Known Risks / Unknowns

### Risks

- Divergence between SQL schema and operational persistence examples
- Ambiguous ownership split between PostgreSQL and JetStream KV
- Cross-domain coupling can introduce undocumented boundary assumptions

### Required Follow-ups

- Keep persistence status/state transitions synchronized with lifecycle updates
- Document any future ownership shifts between durable and ephemeral stores

## Authoritative Spec Files

- `spec/data-management/postgresql-implementation.md`
- `spec/data-management/database-schemas.md`
- `spec/data-management/data-persistence.md`
- `spec/data-management/jetstream-kv.md`
- `spec/data-management/persistence-operations.md`
- `spec/data-management/storage-patterns.md`
- `spec/data-management/data-integration-patterns.md`
