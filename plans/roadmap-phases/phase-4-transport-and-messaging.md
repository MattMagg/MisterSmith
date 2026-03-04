# Phase 4: Transport and Messaging

## Purpose and Scope

Define protocol-agnostic and protocol-specific transport behavior plus message schema contracts.
This phase makes the actor system externally reachable while preserving envelope consistency.

### In Scope

- Transport trait and envelope abstraction
- NATS transport design and subject routing conventions
- Message schema definitions for core/workflow/system flows
- HTTP and gRPC transport surfaces with security integration hooks

### Out of Scope

- Security policy enforcement logic (Phase 5)
- Storage persistence contracts (Phase 6)

## Inputs and Dependencies

### Upstream Dependencies

- Phase 1 (types and transport contracts)
- Phase 2 (event/metrics integration and resource lifecycles)

### Key Source Inputs

- `ROADMAP.md` Phase 4 and Gate 4
- `VERSION_REFERENCE.md` transport versions (async-nats 0.46.0, Axum 0.8.8, Tonic 0.14.5)
- `VALIDATION_REPORT.md` transport migration and schema-consistency outcomes

### Required Specification Anchors

- `spec/transport/transport-core.md`
- `spec/transport/transport-layer-specifications.md`
- `spec/transport/nats-transport.md`
- `spec/transport/http-transport.md`
- `spec/transport/grpc-transport.md`
- `spec/data-management/message-schemas.md`
- `spec/data-management/core-message-schemas.md`
- `spec/data-management/workflow-message-schemas.md`
- `spec/data-management/system-message-schemas.md`
- `spec/data-management/message-framework.md`

## Outputs and Downstream Consumers

### Produces

- Protocol-agnostic transport contract and envelope
- NATS design contract for inter-agent communication
- HTTP and gRPC integration surfaces for control-plane and external clients
- Canonical message schemas used by agents and persistence

### Consumed By

- Phase 5 security middleware/interceptor enforcement
- Phase 6 persistence operations for message durability and audit trails
- Phase 7 agent communication and orchestration flows
- Phase 8 operations endpoints and health/reporting surfaces

## Gate Criteria and Validation

### Gate Criteria

- Transport abstraction and envelope semantics are consistent across NATS/HTTP/gRPC docs
- async-nats 0.46 behavior and feature-gate assumptions are reflected in transport specs
- Message schema docs are consistent across core/workflow/system variants
- HTTP and gRPC docs include explicit Phase 5 security integration hooks

### Validation Approach

- Verify message priority remains 0-4 across transport/data-management schemas
- Confirm request/reply correlation and error-propagation semantics are consistent
- Ensure transport references do not rely on unresolved historical artifacts

### Validation Evidence

- End-to-end message flow trace consistency (spawn -> communicate -> persist -> respond)
- Aligned references between transport docs and message schema docs

## Official-Doc Best Practices

- Align NATS usage with async-nats 0.46 APIs and feature-gated modules (JetStream/KV/Object Store/Service) ([async-nats 0.46 docs](https://docs.rs/async-nats/0.46.0/async_nats/)).
- Use JetStream context APIs for account/stream/consumer operations and avoid stale method names ([JetStream Context API](https://docs.rs/async-nats/0.46.0/async_nats/jetstream/context/struct.Context.html)).
- Keep HTTP transport aligned with Axum 0.8 extractor/routing model ([Axum 0.8.8](https://docs.rs/axum/0.8.8/axum/)).
- Keep gRPC contract and transport behavior aligned with Tonic 0.14 + Prost 0.14 generated APIs ([Tonic 0.14.5](https://docs.rs/tonic/0.14.5/tonic/) and [Prost 0.14.3](https://docs.rs/prost/0.14.3/prost/)).

## Known Risks / Unknowns

### Risks

- Drift between message schema documents can break boundary assumptions
- Backpressure expectations can diverge between mailbox and NATS publish paths
- gRPC numeric fields can miss explicit bounds where enums are expected

### Required Follow-ups

- Keep transport docs synchronized with schema updates
- Keep security enforcement ownership in Phase 5 while maintaining integration hooks here

## Authoritative Spec Files

- `spec/transport/transport-core.md`
- `spec/transport/transport-layer-specifications.md`
- `spec/transport/nats-transport.md`
- `spec/transport/http-transport.md`
- `spec/transport/grpc-transport.md`
- `spec/data-management/message-schemas.md`
- `spec/data-management/message-framework.md`
