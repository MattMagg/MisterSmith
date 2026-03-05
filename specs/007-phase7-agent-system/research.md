# Research: Phase 7 — Agent System

**Date**: 2026-03-05
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

## Research Summary

Phase 7 builds on extensive prerequisite research conducted during the pre-phase analysis. All critical architectural decisions have been resolved through external research (NATS docs, async-nats 0.46 API, Rust actor framework comparison, Microsoft Multi-Agent Reference Architecture, MCP permission patterns). No NEEDS CLARIFICATION items remain.

---

## R1: Actor-Agent Bridge Pattern

**Decision**: Each agent role implements the existing `Actor` trait (from `mister-smith-core`) with role-specific `Message`, `State`, `Error`, and `Response` associated types. An `AgentRuntime` wrapper struct bridges the `Actor` trait (mailbox-driven message processing) with the `Agent` trait (tool-compatible interface with `process()`, `role()`, `context()`).

**Rationale**: The existing `Actor` trait uses `&mut self` with mutable state, while the `Agent` trait uses `&self` with interior mutability. The bridge pattern lets actors run in the supervision tree (Phase 3) while exposing the Agent/Tool interface for tool registry integration (FR-6). This avoids modifying either existing trait.

**Alternatives considered**:
- **Merge Actor and Agent traits**: Rejected — different ownership models (`&mut self` vs `&self`), and Actor's associated types make it non-object-safe. Merging would break Phase 3 compatibility.
- **Abandon Actor trait for agents**: Rejected — Phase 3's `ActorCell`, `ActorRef`, and supervision tree integration depend on the Actor trait. Reimplementing supervision would duplicate effort.
- **Newtype delegation**: Considered but adds boilerplate without real benefit over a bridge wrapper.

---

## R2: Agent Registry Design

**Decision**: In-memory registry per node using `DashMap<AgentId, AgentEntry>` for concurrent access. Cross-node discovery via NATS subject subscriptions on `agents.>` wildcard. Registry entries persisted to PostgreSQL for crash recovery.

**Rationale**: NATS's built-in subject-based routing already provides distributed discovery. Subscribing to `agents.*.status` gives real-time updates across nodes without a separate consensus protocol. DashMap matches the existing pattern used in Phase 5 (JWT revocation list, RBAC role storage).

**Alternatives considered**:
- **Centralized database registry**: Rejected — adds single point of failure and latency for every discovery query. NATS subjects already provide the pub/sub semantics needed.
- **etcd/Consul-based service discovery**: Rejected — adds external dependency. NATS is already the communication backbone; using it for discovery keeps the dependency footprint minimal.
- **Raft consensus for registry**: Rejected — over-engineered for the use case. Eventual consistency via heartbeats is sufficient since agent availability is already fuzzy (agents can fail between query and use).

---

## R3: Task Scheduling Strategy

**Decision**: Pull-based scheduling where agents subscribe to task assignment subjects filtered by their type and capabilities. Schedulers publish task assignments to `tasks.{type}.assignment` subjects. Workers consume from JetStream pull consumers with explicit ack, ensuring at-most-one active assignment per message.

**Rationale**: Pull consumers with AckPolicy::Explicit are the NATS-recommended pattern for new projects (per official NATS JetStream documentation). This matches the DurableTransport trait implemented in the prerequisites. The pull model gives workers flow control — they pull work when ready rather than being pushed work they can't handle.

**Alternatives considered**:
- **Push-based scheduling (Coordinator pushes to specific workers)**: Rejected — requires the Coordinator to track worker load. Pull-based lets workers self-select based on their capacity.
- **Central scheduler with round-robin**: Considered as a complement. The scheduler publishes to subjects; workers pull. No dedicated scheduler process needed for simple cases.
- **Work-stealing queue**: Rejected — adds complexity. JetStream consumers already provide fair distribution across multiple subscribers to the same subject.

---

## R4: Team Lifecycle Management

**Decision**: Teams are ephemeral supervision subtrees created by Coordinators. A `Team` struct holds the supervisor, member agent refs, and the orchestrating task context. Team disbanding stops all member agents and removes the supervision subtree. Incomplete subtasks on team disband are marked as cancelled.

**Rationale**: Binding team lifecycle to the orchestrating task prevents resource leaks (orphaned agents). Using supervision subtrees gives automatic failure handling — if a team member fails, the team's supervisor applies its restart strategy before the Coordinator needs to intervene.

**Alternatives considered**:
- **Persistent teams (survive across tasks)**: Rejected for initial implementation — adds lifecycle complexity. Teams can be reused by creating a new team with the same agents, but the supervision subtree is fresh.
- **Pool-based (pre-allocated worker pools)**: Deferred — useful optimization for high-throughput scenarios but adds scheduling complexity. Can be added later without breaking the team abstraction.

---

## R5: Tool Bus Architecture

**Decision**: Central `ToolBus` with in-memory tool registry (name → ToolEntry). Tool invocations go through: caller → ToolBus.invoke(name, params) → permission check (PolicyEngine) → target agent message → response → audit log → return to caller. MCP tools are registered as ToolEntry instances backed by MCP client sessions instead of local agent refs.

**Rationale**: A single invocation path for both native agent tools and MCP tools gives callers a uniform interface. Permission checking at the ToolBus boundary ensures all invocations are authorized. This matches the Microsoft Multi-Agent Reference Architecture pattern of a central registry with capability-based access control.

**Alternatives considered**:
- **Decentralized tool discovery (each agent queries NATS)**: Rejected — no central point for permission enforcement. Every agent would need its own PolicyEngine integration.
- **Direct agent-to-agent invocation (bypass registry)**: Rejected — bypasses permission checks and audit logging. Only the ToolBus should mediate tool invocations.
- **Separate registries for native vs MCP tools**: Rejected — violates the uniform interface principle. Callers shouldn't need to know whether a tool is native or MCP-backed.

---

## R6: Heartbeat and Failure Detection

**Decision**: Each agent spawns a background Tokio task that publishes heartbeats at a configurable interval (default: 5s) to `agents.{id}.heartbeat`. The registry's liveness monitor uses the existing phi accrual failure detector from `mister-smith-monitoring::HealthMonitor`. Failure detection threshold configurable (default: phi > 8.0 ≈ 99.97% confidence of failure).

**Rationale**: Phi accrual failure detection adapts to network jitter and varying heartbeat intervals, providing more accurate failure detection than fixed-timeout approaches. The implementation already exists in Phase 2 and is battle-tested with 51 tests.

**Alternatives considered**:
- **Fixed timeout (3 missed heartbeats = dead)**: Simpler but fragile under network jitter. Phi accrual is already implemented and superior.
- **Active health probing (request-reply health checks)**: Complementary, not replacement. Heartbeats are passive; active probes can be added per agent for deeper health assessment. FR-7 supports both via "custom health checks."

---

## R7: Message Priority Processing

**Decision**: Priority-aware mailbox processing using the existing bounded mailbox from `mister-smith-actor::mailbox`. Messages are dequeued in priority order within the mailbox. Priority is carried in `MessageEnvelope.metadata` as a `priority: u8` field (0 = highest). The mailbox implementation uses a `BinaryHeap` ordered by priority, replacing the current FIFO `tokio::sync::mpsc` for agents that opt into priority processing.

**Rationale**: Not all agents need priority processing (e.g., heartbeat processors). Making it opt-in via configuration avoids overhead for simple agents. BinaryHeap gives O(log n) insert and O(1) peek for highest priority.

**Alternatives considered**:
- **Multiple mailboxes per priority level**: Rejected — adds complexity in the actor cell. A single priority queue is simpler and sufficient for the expected message volume.
- **Priority lanes (separate channels merged with select!)**: Considered — more complex but gives true preemption. Deferred as a future optimization if priority processing proves insufficient.

---

## R8: Specialized Role Implementation Depth

**Decision**: Phase 7 implements each of the 9 specialized roles as concrete structs with their core behavior. Roles process their domain-specific messages and integrate with the appropriate infrastructure (supervisor → supervision tree, worker → task execution, coordinator → team orchestration, etc.). Complex domain logic within roles (e.g., Planner's plan generation algorithm, Critic's evaluation rubrics) uses pluggable handler traits — the role provides the orchestration skeleton, the application provides the domain logic.

**Rationale**: Roles must be usable out-of-the-box for the Gate 7 validation (Coordinator decomposes task, Workers execute, Supervisor restarts on failure). But the framework is model-agnostic — it shouldn't hardcode any planning or evaluation logic. Pluggable handlers let applications inject their own logic while the framework manages lifecycle, communication, and fault tolerance.

**Alternatives considered**:
- **Abstract roles only (trait definitions, no implementations)**: Rejected — Gate 7 requires a working end-to-end demo. Abstract-only would defer too much to the application.
- **Full domain logic in roles**: Rejected — violates Constitution Principle IV (model-agnostic). Planning and evaluation logic depends on the LLM or domain being orchestrated.

---

## Research Sources

| Source | Topic | Key Finding |
|--------|-------|-------------|
| [NATS JetStream Consumers](https://docs.nats.io/using-nats/developer/develop_jetstream/consumers) | Pull consumers | Pull consumers recommended for new projects; AckPolicy::Explicit for precise control |
| [async-nats 0.46 docs](https://docs.rs/async-nats/latest/async_nats/) | Acker API | `Acker` struct with `ack()`, `nak(delay)`, `term()`, `in_progress()` |
| [Rust Actor Frameworks (Seyhun, 2025)](https://tqwewe.com/blog/comparing-rust-actor-libraries) | Actor patterns | Ractor and Kameo favored for supervision; our custom impl aligns with best patterns |
| [Microsoft Multi-Agent Ref Architecture](https://microsoft.github.io/multi-agent-reference-architecture/) | Orchestration | Orchestrator + Registry + Classifier + MCP pattern validated |
| [MCP Permissions / Permit.io](https://docs.permit.io/mcp-permissions) | Tool security | ReBAC for AI agents; permission model matches our RBAC extension |
| [NATS Deduplication](https://nats.io/blog/new-per-subject-discard-policy) | Message dedup | MsgId header for publisher-side dedup with configurable window |
| [Cerbos MCP Auth](https://cerbos.dev/blog/dynamic-authorization-for-ai-agents-guide-to-fine-grained-permissions-mcp-servers) | Dynamic auth | Fine-grained permissions per tool invocation |
| [Pylar.ai RBAC for AI](https://pylar.ai/blog/designing-rbac-for-ai-agents-complete-framework) | Agent RBAC | Context-aware, fine-grained controls for autonomous agents |
