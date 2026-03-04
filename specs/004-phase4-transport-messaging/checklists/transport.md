# Transport Requirements Quality Checklist: Transport & Messaging

**Purpose**: Validate completeness, clarity, and consistency of transport layer requirements before planning. Priority on core transport (envelope, NATS, JetStream) as the framework's differentiator; secondary coverage for HTTP/gRPC/MCP.
**Created**: 2026-03-04
**Feature**: [spec.md](../spec.md)

## Requirement Completeness — Transport Abstraction (P1)

- [ ] CHK001 - Are all Transport trait operations (publish, subscribe, request, reply) individually specified with input/output contracts? [Completeness, Spec §FR-001]
- [ ] CHK002 - Are all MessageEnvelope fields (message_id, timestamp, schema_version, message_type, correlation_id, source_agent_id, target_agent_id, priority, payload) defined with types and constraints? [Completeness, Spec §FR-002]
- [ ] CHK003 - Is the maximum message payload size specified or is it configurable with documented default? [Gap, Spec §Edge Cases]
- [ ] CHK004 - Are serialization format requirements complete — is MessagePack schema evolution strategy defined (adding/removing fields across versions)? [Gap, Spec §FR-003]
- [ ] CHK005 - Is the envelope schema_version field's behavior defined when sender and receiver use different versions? [Gap, Spec §FR-002]
- [ ] CHK006 - Are Transport trait error return types specified for each operation (publish failure vs subscribe failure vs timeout)? [Completeness, Spec §FR-017]

## Requirement Completeness — NATS Transport (P2)

- [ ] CHK007 - Are NATS connection configuration requirements specified (server URL, auth credentials, TLS, connection name, max reconnects)? [Gap, Spec §FR-004]
- [ ] CHK008 - Is the complete subject taxonomy documented with all parameterized segments and wildcard patterns? [Completeness, Spec §FR-005]
- [ ] CHK009 - Are queue group naming conventions and assignment rules specified? [Gap, Spec §US2]
- [ ] CHK010 - Are reconnection retry policy parameters defined (initial delay, max delay, backoff factor, max attempts)? [Clarity, Spec §FR-007]
- [ ] CHK011 - Is the behavior during reconnection defined — are publish operations buffered, rejected, or blocking? [Gap, Spec §FR-007]
- [ ] CHK012 - Are requirements for in-flight messages during a connection drop specified (lost, buffered, retried)? [Gap, Spec §Edge Cases]

## Requirement Completeness — JetStream Durability (P3)

- [ ] CHK013 - Are JetStream stream configuration requirements specified (storage type, retention policy, max messages, max bytes, max age, replicas)? [Gap, Spec §FR-008]
- [ ] CHK014 - Are consumer configuration requirements specified (durable name, ack policy, ack wait, max deliver, filter subject)? [Gap, Spec §FR-009]
- [ ] CHK015 - Is the distinction between pull and push consumer use cases documented with guidance on when to use each? [Clarity, Spec §FR-009]
- [ ] CHK016 - Are exactly-once vs at-least-once delivery semantics explicitly scoped? [Clarity, Spec §US3]
- [ ] CHK017 - Are stream subject filter patterns defined for which message types go to which streams? [Gap, Spec §FR-008]
- [ ] CHK018 - Are dead letter handling requirements for JetStream max delivery exceeded defined? [Gap, Spec §FR-010]

## Requirement Clarity — Message Ordering & Delivery

- [ ] CHK019 - Is "per-subject ordering guaranteed" sufficiently precise — does it cover single-publisher only or multi-publisher on same subject? [Clarity, Spec §FR-035]
- [ ] CHK020 - Is the interaction between queue group delivery and ordering guarantees defined? [Gap, Spec §FR-035]
- [ ] CHK021 - Are backpressure thresholds quantified (buffer size, blocking timeout, error conditions)? [Clarity, Spec §FR-018]
- [ ] CHK022 - Is the distinction between core NATS (fire-and-forget) and JetStream (acknowledged) publish requirements clear? [Clarity, Spec §FR-004 vs §FR-008]

## Requirement Consistency — Cross-Transport

- [ ] CHK023 - Are error types consistent across all transport implementations (NATS, HTTP, gRPC) — same error categories, same granularity? [Consistency, Spec §FR-017]
- [ ] CHK024 - Is AgentAvailability (idle/busy/offline) consistently used across all transport stories, with AgentState reserved for lifecycle only? [Consistency, Spec §FR-020]
- [ ] CHK025 - Are health reporting requirements consistent — do all transports (NATS, HTTP, gRPC, MCP) report to the same health monitoring infrastructure? [Consistency, Spec §FR-019]
- [ ] CHK026 - Are timeout semantics consistent across request-reply (NATS §FR-006), HTTP (§SC-005), gRPC (§US5), and MCP tool calls (§US6)? [Consistency]

## Requirement Completeness — HTTP & WebSocket (P4)

- [ ] CHK027 - Are all REST API endpoints enumerated with HTTP methods, paths, request/response schemas? [Gap, Spec §FR-011]
- [ ] CHK028 - Are rate limiting requirements quantified (requests per second, per client, burst allowance)? [Clarity, Spec §FR-012]
- [ ] CHK029 - Are WebSocket event categories exhaustively listed (which events are streamable)? [Gap, Spec §FR-023]
- [ ] CHK030 - Are WebSocket connection limits specified (max concurrent connections, per-client limits)? [Gap, Spec §FR-022]
- [ ] CHK031 - Is the WebSocket reconnection behavior from the client perspective defined (auto-reconnect, backoff, state recovery)? [Gap, Spec §US4]
- [ ] CHK032 - Are HTTP error response formats standardized (error code schema, content type, headers)? [Gap, Spec §FR-011]

## Requirement Completeness — gRPC (P5)

- [ ] CHK033 - Are protobuf service definitions enumerated with method signatures? [Gap, Spec §FR-014]
- [ ] CHK034 - Is the gRPC-to-framework error mapping table complete (all gRPC status codes → framework errors and vice versa)? [Gap, Spec §FR-016]
- [ ] CHK035 - Are streaming RPC flow control requirements (backpressure, window size) defined? [Gap, Spec §FR-015]

## Requirement Completeness — MCP Integration (P6)

- [ ] CHK036 - Are MCP client and server implementation priorities (phasing order A→D) reflected in the spec or deferred to planning? [Gap, Spec §US6]
- [ ] CHK037 - Is the dependency between MCP server (exposing agent tools) and the agent tool system (Phase 7+) acknowledged? [Dependency, Spec §US6]
- [ ] CHK038 - Are MCP transport requirements (stdio, streamable-HTTP) distinguished from the framework's own transport trait (NATS, HTTP, gRPC)? [Clarity, Spec §FR-025]
- [ ] CHK039 - Is the NATS-MCP bridge's failure domain defined — what happens when NATS is available but the remote node's agent is down? [Gap, Spec §FR-033]
- [ ] CHK040 - Are MCP resource exposure boundaries precise — is the list of what IS and IS NOT exposed complete? [Clarity, Spec §FR-034]

## Acceptance Criteria & Measurability

- [ ] CHK041 - Are all 16 success criteria (SC-001 through SC-016) independently measurable with automated tests? [Measurability]
- [ ] CHK042 - Do success criteria cover all 6 user stories — is every story represented by at least one SC? [Coverage]
- [ ] CHK043 - Are SC-001 (10ms NATS latency) and SC-005 (50ms HTTP latency) realistic under the specified test conditions? [Measurability]
- [ ] CHK044 - Is SC-007 (10% variance in queue group distribution) measured over a statistically meaningful sample? [Clarity, Spec §SC-007]
- [ ] CHK045 - Are success criteria for WebSocket event delivery latency (SC-011, 1 second) and MCP tool execution (SC-013) defined with load conditions? [Clarity]

## Scenario Coverage — Supervision Tree Integration

- [ ] CHK046 - Are requirements for transport behavior during actor restarts specified (message buffering, connection handover)? [Gap]
- [ ] CHK047 - Are requirements for supervised transport actors defined — should NatsTransport, HttpTransport, McpServer run under supervision? [Gap]
- [ ] CHK048 - Is the interaction between transport health (FR-019) and supervision health checks (Phase 3) defined? [Gap]
- [ ] CHK049 - Are requirements for graceful transport shutdown during system shutdown specified (drain connections, flush buffers)? [Gap]

## Edge Case & Failure Coverage

- [ ] CHK050 - Are all 14 edge cases testable as written — do they specify concrete pass/fail criteria? [Measurability, Spec §Edge Cases]
- [ ] CHK051 - Are partial failure scenarios defined (e.g., NATS connected but JetStream unavailable)? [Gap]
- [ ] CHK052 - Are resource exhaustion scenarios defined (too many subscriptions, too many WebSocket connections, memory pressure)? [Gap]
- [ ] CHK053 - Are clock skew/time synchronization requirements defined for envelope timestamps across distributed nodes? [Gap]

## Notes

- Transport core (CHK001-CHK022) is highest priority — these requirements define the framework's core differentiator
- Supervision tree integration (CHK046-CHK049) connects Phase 4 to Phase 3 and is critical for production reliability
- MCP items (CHK036-CHK040) are lighter-weight given MCP is important but not foundational
- Items marked [Gap] indicate requirements that may need to be added during planning
- Items marked [Clarity] indicate existing requirements that need more precision
