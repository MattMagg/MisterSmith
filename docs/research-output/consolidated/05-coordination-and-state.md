# Coordination, State Management & NATS Patterns -- Consolidated State of Knowledge

**Consolidated**: 2026-03-07
**Sources**: R3 synthesis, R4 academic (stigmergy + discovery), R5 discovery, R6 deep dive (CRDTs), R7c discovery, R7d discovery
**Scope**: NATS JetStream patterns, CRDTs, stigmergy, coordination protocols, session types, shared state, event-triggered consensus, causal context meshes

---

## Executive Summary

Seven research rounds converge on a single architectural thesis: **NATS JetStream can serve as both the messaging backbone and the shared-state substrate for multi-agent coordination in Mister Smith**, but it must be augmented with CRDT-based observation-driven coordination for shared artifacts, formal protocol verification via multiparty session types, and event-triggered consensus to reduce communication overhead at scale.

The core finding is a **three-tier coordination model**:

1. **Delta-CRDTs over JetStream Pub/Sub** for high-churn, loosely coupled shared artifacts (task graphs, capability registries, collaborative documents). Agents coordinate by observing shared state rather than exchanging explicit messages. CodeCRDT achieves 100% syntactic convergence with zero merge failures across 600 trials. Diamond-types processes 4.6 million ops/sec in Rust.

2. **JetStream KV Compare-And-Swap (CAS)** for strict serialization points (budget enforcement, leader election, configuration hot-reload) where linearizability is mandatory. CAS operations require synchronous round-trips but guarantee invariants like non-negative budgets.

3. **Core NATS request-reply and JetStream streams** for ephemeral routing (~50 us RTT, 40x faster than HTTP), durable event logs, telemetry, and side-effect execution with exactly-once semantics.

Formal verification via Multiparty Session Types (MPST) can provide compile-time protocol safety for agent choreographies, leveraging Rust's affine type system to guarantee deadlock-freedom. Event-triggered consensus from control theory reduces inter-agent communication by 40-60% while maintaining stability guarantees. Permutation-invariant context composition via State Space Models enables constant-time context merging regardless of memory volume.

**Critical risks**: Jepsen testing on NATS 2.12.1 revealed that the default 2-minute `fsync` interval can lose acknowledged writes during coordinated crashes. CRDT tombstone metadata grows unboundedly without garbage collection. Semantic conflict rates of 5-10% persist even with perfect syntactic convergence. The "Agent Smith" infectious jailbreak can compromise an entire agent network exponentially through shared memory.

---

## High-Confidence Findings

These findings are supported by multiple independent sources across research rounds, with empirical data or production-validated evidence.

| Finding | Confidence | Evidence Basis |
|:--------|:-----------|:---------------|
| NATS collapses API gateways, service meshes, load balancers, config stores, durable logs, and service registries into a single binary | **High** (3 independent reports converge) | R3 triple synthesis; Eviny, MachineMetrics case studies |
| Core NATS request-reply averages ~50 us RTT (40x faster than HTTP/REST) | **High** | R3 benchmark convergence across all 3 source reports |
| Pull consumers are superior to push consumers for agent-to-agent streaming with backpressure | **High** (3 reports converge) | R3 |
| CRDTs provide 100% syntactic convergence with zero merge failures for observation-driven coordination | **High** | R6 citing CodeCRDT (600 trials); R4 discovery |
| CRDTs cause 5-10% semantic conflict rate requiring application-level reconciliation | **High** | R6 citing CodeCRDT |
| CRDT-based coordination yields 21.1% speedup on independent tasks but 39.4% slowdown on tightly coupled tasks | **High** | R6 citing CodeCRDT |
| Diamond-types achieves 4.6M ops/sec, 260k edits in 56ms, 1.1 MB memory footprint | **High** | R6 citing josephg benchmarks |
| JetStream default `fsync` interval (2 min) risks data loss during crashes | **High** | R6 citing Jepsen analysis of NATS 2.12.1 |
| MPST in Rust provides compile-time deadlock-freedom for agent protocols | **Moderate-High** | R7c (Mozilla Servo case study), R7d (session-types + rumpsteak libraries) |
| Event-triggered consensus reduces communication overhead while maintaining stability | **Moderate** | R5 citing Yang et al. 2025, Xiao et al. 2025, Wang & Zhu 2025 (IEEE TASE) |
| Multi-agent teams improve performance on parallelizable tasks but degrade it on sequential tasks | **High** | R7c citing Google Research (Kim & Liu 2026, 180 configurations) |
| Stigmergy is the formal computer science analog to CRDTs -- agents modify shared state, others observe | **Moderate-High** | R4 (52 papers), R4 discovery, R6 |

---

## Key Techniques & Architectures

### NATS JetStream Patterns

**Subject-based hierarchical routing** eliminates external load balancers and API gateways. The canonical pattern `llm.complete.{provider}.{model}.{tier}.{region}` uses NATS wildcards (`*` single-level, `>` multi-level) for flexible interest expression. Queue groups provide zero-config load balancing -- new instances join by subscribing.

**Pull consumers for demand-driven flow.** For reliable agent-to-agent streaming, JetStream pull consumers are superior to push consumers. The consumer explicitly requests batches (`Fetch(N)`), creating implicit one-to-one flow control that prevents fast LLM producers from overwhelming slow downstream agents.

**Append-only streams for agent memory.** Conversations modeled as append-only JetStream streams (`agent.mem.{agent_id}.{session_id}`) provide strict ordering, replayability, auditability, and time-travel debugging. JetStream writes are linearizable -- committed and replicated across the cluster before acknowledgment.

**Hybrid stream + KV pattern.** Store each interaction as an append record in JetStream for full audit/tracing, while keeping a KV entry (`conversations.latest.{tenant}.{conversation_id}` => `sequence_id`) pointing to the latest sequence for fast random access. The stream provides replay and full history; the KV pointer enables quick resume.

**Exactly-once semantics via deduplication.** Publishers include `Nats-Msg-Id` headers. JetStream tracks IDs within a configurable duplicate window (default 2 min) and silently drops duplicates. Combined with `AckSync`, this guarantees exactly-once processing.

**KV watches for hot-reload configuration.** Provider configs stored in KV buckets with CAS-based updates (`Nats-Expected-Last-Subject-Sequence` header). Agents use `WatchAll` to receive real-time pushed updates when API keys are rotated or models deprecated.

**Speculative execution ("first response wins").** Agent publishes to multiple providers simultaneously via wildcard subjects, accepts the first response, and sends a cooperative cancellation on `cancel.llm.{request_id}`. Practical only because NATS routing overhead (~50 us) is negligible -- you cannot hedge across providers if each routing hop costs 5-10 ms.

**Acknowledgment flows with exponential backoff:**

| Outcome | Action | Effect |
|:--------|:-------|:-------|
| Success | `msg.ack()` | Remove from consumer |
| Transient error | `msg.nak()` | Redelivery with backoff (e.g., 5s, 30s, 300s) |
| Long-running tool | `msg.in_progress()` | Reset `AckWait` timer |
| Poison message | `msg.term()` | Halt redelivery, trigger DLQ advisory |

**Dead Letter Queue (DLQ) pattern.** Subscribe to `$JS.EVENT.ADVISORY.CONSUMER.MAX_DELIVERIES.>` and `$JS.EVENT.ADVISORY.CONSUMER.MSG_TERMINATED.>`, extract `stream_seq`, retrieve the failed message, move to a dedicated DLQ stream.

**Service discovery via NATS micro.** Agents register as discoverable services responding to `$SRV.PING` (liveness), `$SRV.INFO` (capabilities/metadata), `$SRV.STATS` (latency/errors/counts). Eliminates Consul/Eureka/Istio. The Go/JS clients have native `micro` packages; Rust requires custom implementation via `async-nats` pub/sub and request primitives.

**Edge federation via leaf nodes.** Leaf nodes connect edge servers to a central hub, buffer messages locally during disconnection, and support subject remapping for data locality. Superclusters via gateways enable geo-distributed agents with geo-affinity routing. MachineMetrics deployed NATS on thousands of factory-edge devices replacing Kinesis, using JetStream for edge persistence and the object store for WASM module distribution.

**Source**: R3 (`nats-native-agent-patterns-R3.md`)

---

### Stigmergy (Indirect Coordination via Shared State)

Stigmergy -- agents modifying a shared environment and other agents observing those modifications -- is the biological and theoretical foundation for CRDT-based coordination. Research across 52 academic papers (R4) validates this model for LLM multi-agent systems.

**JetStream KV as pheromone medium.** JetStream KV with TTL-based key expiry maps directly to pheromone evaporation. Agents write "signals" (task priorities, capability advertisements, workload indicators) to KV keys with time-limited lifetimes. Other agents observe and react. No centralized scheduler required.

**Blackboard architecture.** A shared memory space (blackboard) where agents post and read updates replaces point-to-point messaging for certain workloads. Fahey (2026) notes this "dramatically improves coherence and scalability" versus point-to-point messages. JetStream KV is a natural blackboard implementation.

**Formal equivalence: stigmergy as reinforcement learning.** R4 academic research establishes a mathematical bridge: stigmergic coordination can be formalized as a special case of multi-agent RL where the environment state serves as the reward signal. This connects CRDT-based shared-state coordination with established convergence guarantees from RL theory.

**SwarmBench cautionary evidence.** Current LLMs significantly struggle with pure swarm coordination under informational decentralization (limited local perception, local-only communication). Performance is highly task-dependent. This means Mister Smith's architecture (supervision trees, event bus, monitoring) provides the structured scaffolding that LLMs need to overcome their coordination weaknesses -- agents cannot self-organize without infrastructure support.

**Source**: R4 (`targeted-stigmergy-swarm-coordination-R4.md`), R4 discovery, R7c

---

### CRDT-Based Coordination

CRDTs represent a paradigm shift from explicit message-passing to observation-driven coordination. Agents modify shared state and observe updates without centralized locking, achieving lock-free, partition-tolerant concurrent execution.

**CodeCRDT results (600 trials):**
- 100% syntactic convergence, zero merge failures
- Up to 21.1% speedup on independent tasks (51.8% faster per-character generation rate)
- Up to 39.4% slowdown on tightly coupled tasks (5.8% slower per-character rate)
- 82-189% code volume inflation from LLMs generating redundant safety checks
- 5-10% semantic conflict rate (duplicate logic, conflicting implementations that merge perfectly at character level but fail to compile)

**CRDT type selection for Mister Smith workloads:**

| Workload | CRDT Type | Rationale |
|:---------|:----------|:----------|
| Shared task claiming | OR-Set + optimistic claim protocol | Agents observe set, claim via delta, verify after sync delay (~50ms). 2P-Set is wrong -- permanently blocks re-addition of removed elements |
| Capability registries | LWW-Element-Set | Handles rapid online/offline toggles efficiently |
| Collaborative code/docs | Sequence CRDT (Diamond-types) | Lock-free concurrent editing; LLMs observe diffs not full files |
| Budget/quota tracking | JetStream KV CAS (NOT CRDT) | PN-Counters cannot enforce global invariants like budget >= 0 |
| External side-effects | JetStream Work Queue (NOT CRDT) | Side-effects are not commutative; requires exactly-once semantics |
| Task dependency graphs | Monotonic DAG CRDT | Enforces acyclicity: edges can only strengthen existing partial order |

**Rust CRDT library benchmarks:**

| Library | Performance | Best Use Case |
|:--------|:-----------|:-------------|
| **Diamond-types** | **4.6M ops/sec**, 260k edits in 56ms, 1.1 MB memory | High-frequency collaborative text/code generation |
| **Loro** | High performance, optimized for JSON-like nested structures | Complex agent state, nested capability registries |
| **Automerge-rs** | Slower but highly mature, excellent binary compression | Long-term document storage with deep history |

**Delta-CRDTs over pub/sub for bandwidth efficiency.** Instead of shipping full state, agents generate small incremental deltas published to JetStream subjects (e.g., `crdt.task_graph.deltas`). Other agents subscribe, receive deltas, and join with local state. Entirely asynchronous and lock-free -- agents do not wait for server validation.

**Optimistic claim protocol (at-most-one execution):**
1. **Scan**: Read local CRDT for `assignedTo == null`
2. **Claim**: Write delta setting `assignedTo = self_id`
3. **Verify**: Wait ~50ms sync delay, re-read local state
4. **Proceed**: If `assignedTo == self_id` after convergence, claim succeeded; otherwise back off

**Metadata growth problem.** Every deleted element becomes a tombstone. A 1,000-character heavily-edited document might contain 50,000 internal tombstones. Causal CRDTs use Dotted Version Vectors that grow linearly with active replicas. At 1,000 concurrent writers, vector clock overhead is non-trivial -- requires periodic compaction.

**Evaluator Agent for semantic conflicts.** Since CRDTs guarantee syntactic but not semantic correctness, Mister Smith must deploy an Evaluator Agent that observes converged CRDT state, runs semantic checks (e.g., `cargo check`), and issues corrective delta-updates for invalid state.

**Source**: R6 (`targeted-crdt-coordination-R6.md`), R4 discovery

---

### Hybrid Model: When to Use CRDTs vs. Pub/Sub vs. Streams

The decision matrix from R6 is the authoritative guide:

| Primitive | Best-Fit Workloads | Consistency Model | Latency Profile | Mister Smith Use Case |
|:----------|:------------------|:------------------|:----------------|:---------------------|
| **Delta-CRDTs over Pub/Sub** | Collaborative editing, task registries, capability discovery, DAGs | Strong Eventual Consistency (SEC) | Sub-ms local reads/writes; high throughput via async deltas | Shared agent workspaces, stigmergic planning, distributed TODO lists |
| **JetStream KV CAS** | Budgets, leader election, strict state transitions | Linearizable (monotonic reads/writes) | Higher latency (synchronous round-trip); lower throughput | Non-negative budgets, exclusive global locks, config hot-reload |
| **JetStream Streams** | Telemetry, external API commands, audit logs | At-least-once / exactly-once (with dedup) | High throughput, tunable latency (batching) | Agent observability, idempotent side-effects, conversation memory |
| **Core NATS** | Routing, real-time telemetry, cancellation | At-most-once | ~50 us RTT | LLM request dispatch, live token streaming, fire-and-forget metrics |

**Key principle**: CRDTs handle shared-artifact coordination (what agents collectively build); pub/sub handles event-notification coordination (what agents need to know); streams handle durable side-effects (what must not be lost); KV CAS handles invariant enforcement (what must be strictly serialized).

**Shadow-mode migration strategy.** Deploy CRDTs in shadow mode: agents continue coordinating via explicit messages but dual-write to CRDT structures. An observability process compares CRDT state against legacy state. If stable, shift read paths incrementally. If metadata growth or semantic conflicts are catastrophic, roll back seamlessly.

**Source**: R6

---

### Multiparty Session Types (MPST)

MPST applies pi-calculus to Rust's type system, providing compile-time verification that multi-agent communication protocols are deadlock-free and protocol-compliant.

**Mechanism.** A "global type" defines the entire multi-agent choreography. This is mathematically projected into "local types" for each participating agent. By embedding local types within Rust's affine typing system, the compiler guarantees:
- Deadlock-free asynchronous message reordering
- Protocol compliance before binary execution
- No infinite wait states in distributed execution loops

**Rust libraries:**
- `session-types` crate: binary session types with Rust generics
- `rumpsteak`: multiparty session types with async support
- Successfully applied to Mozilla Servo: replaced messaging with session-typed channels, gaining compile-time protocol safety and deadlock-freedom

**Integration path for Mister Smith.** Define agent choreographies (Planner -> Executor -> Reviewer -> Integrator) as global session types. Use Rust macro expansions to generate required message enumerations. The actor-per-stream model aligns naturally -- each actor's mailbox corresponds to a session-typed channel endpoint.

**Limitations.** Requires upfront protocol design. Does not handle dynamic topology changes (agents joining/leaving mid-protocol). Best suited for well-defined interaction patterns rather than ad-hoc collaboration.

**Contrast with runtime verification.** MAS-ProVe (2026) showed that simply adding process-level verification (judges, reward models) often fails in multi-agent contexts, suggesting stronger guarantees -- like session types -- are needed.

**Source**: R7c (`discovery-sweep-R7c.md`), R7d (`discovery-sweep-R7d.md`)

---

### Event-Triggered Consensus

Multiple works from control theory introduce event-triggered consensus protocols that reduce communication overhead while maintaining stability under dynamic topologies.

**Core idea.** Instead of continuous communication (every tick), agents communicate only when a triggering condition is met (state divergence exceeds a threshold). This reduces network traffic by 40-60% while preserving convergence guarantees.

**Variants identified (R5):**
- **PSO-GA co-design for cluster consensus**: Particle Swarm Optimization + Genetic Algorithm to co-design event-triggering thresholds and control gains for multi-agent cluster consensus (Yang et al. 2025, IEEE TASE)
- **Interval type-2 fuzzy models**: Adaptive event-triggered consensus under nonlinear dynamics with heterogeneous topologies (Xiao et al. 2025, IEEE TASE)
- **Distributed hybrid dynamic event-triggered schemes**: Combine open-loop estimation with adaptive control to minimize data exchanges while ensuring safe operational limits (Wang & Zhu 2025, IEEE TASE)

**Mister Smith integration.** Replace continuous heartbeat-based coordination with event-triggered schemes. The existing PhiAccrualFailureDetector already adapts to inter-heartbeat intervals -- extend this with event-triggered thresholds so agents communicate state changes only when divergence exceeds a configurable epsilon. Particularly valuable for edge/bandwidth-constrained deployments.

**Source**: R5 (`discovery-sweep-R5.md`)

---

### Permutation-Invariant Context Composition

The PICASO framework (Permutation-Invariant Context Composition with State Space Models) uses category-theoretic relations to compose multiple independent context states into a single, fixed-dimensional state.

**Mechanism.** Because the chronological ordering of retrieved context fragments is often arbitrary in retrieval-augmented generation, the framework enforces permutation invariance by mathematically averaging states obtained via the composition algorithm across all possible orderings. This requires zero online model processing time -- autoregressive generation begins directly from composed states.

**Performance impact.** Achieves constant-time inference scaling regardless of the volume of episodic memory retrieved. Eliminates the quadratic scaling costs and context pollution of concatenating raw context chunks.

**Mister Smith integration.** Memory-focused actors distribute pre-computed, dimensionally stable latent states across JetStream KV. When an agent needs context from multiple prior interactions or other agents' outputs, the states are composed mathematically rather than concatenated textually. This drastically reduces network bandwidth and token processing overhead during complex reasoning chains.

**Source**: R7d (`discovery-sweep-R7d.md`)

---

### Causal Context Meshes

Category theory enables the creation of Causal Context Meshes -- functorial mappings between the polynomial representations of individual agent models that ensure mathematical consistency across trustless agent collaboration.

**Problem addressed.** In parallel multi-agent reasoning, context from different agents can "pollute" each other -- one agent's partial state contaminates another agent's reasoning, causing cascading errors. This is the "semantic impedance mismatch" problem.

**Mechanism.** Functorial mappings between polynomial representations of agent models ensure that state evolution across parallel task domains remains causally consistent. Each agent's state transitions are tracked in a category-theoretic framework that preserves causal relationships while preventing unauthorized cross-domain influence.

**Mister Smith integration.** This is a theoretical framework more than an implementable library today. The practical takeaway: when designing the shared-state layer (CRDTs + JetStream), explicitly model causal dependencies between agent state updates. Use NATS subject namespacing to enforce causal isolation (e.g., `crdt.{domain}.{artifact}` prevents cross-domain state leakage), and implement causal ordering via Lamport timestamps or vector clocks on CRDT delta messages.

**Source**: R7d (`discovery-sweep-R7d.md`)

---

## NATS-Specific Integration Details

### JetStream KV CAS: Revision-Based Operations

KV CAS uses the `Nats-Expected-Last-Subject-Sequence` header for atomic compare-and-swap. If multiple agents attempt to update a shared state concurrently, only one succeeds; others must fetch the new revision and retry.

**The bottleneck**: CAS operations require synchronous round-trips. Client-side batching is impossible for dependent operations. This limits throughput and makes KV CAS unsuitable for high-churn shared artifacts -- use delta-CRDTs instead.

**KV read-your-writes gap**: NATS KV guarantees monotonic writes and reads, but does NOT guarantee "read your writes" if reads are served by followers. For strict consistency, direct gets to the stream leader are required. For most configuration use cases, the KV watch event-driven model is sufficient since the watch event fires after the write is committed.

### Partition Behavior

**JetStream KV during partitions**: The CAP theorem dictates that linearizable KV CAS cannot be totally available during a network partition. If a partition isolates an agent from the JetStream quorum, KV CAS operations will fail.

**Delta-CRDTs during partitions**: In-memory delta-CRDTs allow agents to continue reading and writing locally (high availability). Once the partition heals, buffered delta-groups are published and Strong Eventual Consistency (SEC) guarantees convergence. This is a fundamental advantage for edge deployments.

**Work-queue split brain**: Work-queue retention is NOT resilient across intermittent leaf nodes. Use Limits/Interest retention for edge mirrors.

### fsync Risks (Jepsen)

Jepsen testing on NATS 2.12.1 revealed: JetStream's default `fsync` interval is 2 minutes, but it acknowledges messages immediately. A coordinated power failure or OS crash loses acknowledged writes.

**Mitigation for critical streams**: Configure `sync_interval: always` for budget tracking, final artifact commits, and any irreversible state. This forces fsync before acknowledgment, ensuring absolute durability at the cost of reducing throughput to hundreds of msgs/sec.

### Consumer Scale Limits

Avoid creating 100k+ filtered/durable consumers -- meta-leader Raft traffic and consumer-info calls overload the server. Use republish patterns or shared consumers with server-side subject transforms to reduce consumer count.

### Canonical Subject and KV Schemas

**Subject taxonomy:**

| Purpose | Subject Pattern | Transport |
|:--------|:---------------|:----------|
| LLM Request | `llm.complete.{provider}.{model}.{region}` | Core NATS request-reply + queue groups |
| Multi-tenant LLM | `llm.complete.{tenant}.{provider}.{model}.{type}` | Core NATS request-reply + queue groups |
| Agent Memory | `agent.mem.{agent_id}.{session_id}` | JetStream (Limits retention) |
| Conversation Log | `conversation.{tenant}.{agent}.{conv_id}` | JetStream append-only |
| Token Streaming | `llm.stream.{tenant}.{conv_id}.{producer}.{shard}` | Core NATS (live) or JetStream (durable) |
| CRDT Deltas | `crdt.{artifact_type}.{artifact_id}.deltas` | JetStream |
| Telemetry (real-time) | `llm.telemetry.{tenant}.{event}.{provider}.{model}` | Core NATS fire-and-forget |
| Telemetry (durable) | `llm.telemetry.{tenant}.{event}.{provider}.{model}` | JetStream stream `TELEMETRY` |
| Service Discovery | `$SRV.INFO.llm_adapter.{id}` | NATS micro |
| Cancellation | `cancel.llm.{request_id}` | Core NATS |

**JetStream stream configuration:**

| Stream | Subjects | Retention | Notes |
|:-------|:---------|:----------|:------|
| `AGENT_MEMORY` | `agent.mem.>` | Limits (MaxAge) | Conversation history |
| `CONVERSATIONS` | `conversation.>` | Limits (MaxAge, MaxBytes) | Per-tenant logs |
| `CRDT_DELTAS` | `crdt.>` | Limits (MaxAge) | Delta-CRDT dissemination |
| `TELEMETRY` | `llm.telemetry.>` | Limits (MaxAge: 48h) | Durable audit |
| `DLQ` | `dlq.>` | Limits (MaxAge: 30d) | Failed messages |

---

## Open Questions & Gaps

1. **CRDT garbage collection in practice.** Tombstone growth is unbounded without coordinated GC. No research quantifies GC overhead at Mister Smith's expected scale (10-1,000 concurrent agents). The Riak AAE (Active Anti-Entropy) pattern using Merkle trees is a candidate but adds complexity.

2. **Semantic conflict resolution automation.** CodeCRDT shows 5-10% semantic conflict rates. The Evaluator Agent concept is proposed but not validated at scale. What is the cost of running `cargo check` (or equivalent) after every merge?

3. **MPST with dynamic topology.** Session types require upfront protocol specification. Mister Smith's agent teams are dynamically composed. How to handle agents joining/leaving mid-protocol without invalidating type guarantees?

4. **Event-triggered consensus thresholds for LLM agents.** Control theory papers validate event-triggered schemes for homogeneous agents with well-defined state spaces. LLM agents have high-dimensional, stochastic state. Optimal triggering thresholds for this domain are unexplored.

5. **NATS micro in Rust.** Go/JS have native `micro` packages; Rust requires custom implementation. Exact effort and API surface not benchmarked.

6. **JetStream on ARM/edge.** Performance of JetStream file storage on resource-constrained edge devices not characterized.

7. **Permutation-invariant composition with non-SSM models.** PICASO assumes State Space Model internals. Integration with transformer-based LLMs (the dominant architecture) may require adaptation.

8. **Causal context meshes.** The category-theoretic framework is compelling but lacks reference implementations. The practical gap between the theory and a working Rust implementation is substantial.

9. **Infectious jailbreak via shared CRDTs.** The "Agent Smith" attack vector shows that shared memory is an infection channel. CRDTs amplify this risk because all agents observe the same state. Mandatory quarantine actors and semantic firewalls between CRDT state and LLM prompts are necessary but add latency.

10. **Delta-CRDT snapshot and rehydration under OTP restarts.** When an agent crashes, its supervisor restarts it. The agent must rehydrate its local CRDT state from a snapshot + delta replay. The latency budget for this rehydration is unquantified.

---

## Implementation Priority for Mister Smith

Ordered by impact, feasibility, and risk mitigation. Phases align with existing architecture (19 crates through Phase 8; Phase 9 LLM providers in progress).

### Tier 1: Foundational (Phase 9 scope or immediate follow-on)

| Priority | Capability | Effort | Justification |
|:---------|:-----------|:-------|:-------------|
| **P0** | JetStream pull consumers for agent-to-agent streaming with backpressure | Low | Already in `mister-smith-nats` crate; extend to LLM provider adapters |
| **P0** | KV CAS for config hot-reload (provider configs, routing weights, API keys) | Low | `async-nats` KV API directly; critical for Phase 9 LLM provider management |
| **P0** | Append-only streams for conversation memory + KV pointers for fast resume | Low-Med | Hybrid pattern from R3; enables stateless agent instances |
| **P0** | Exactly-once via `Nats-Msg-Id` deduplication on all JetStream publishes | Low | Prevents duplicate tool executions; trivial to implement |
| **P0** | `sync_interval: always` on critical streams (budgets, artifacts) | Low | Mitigates Jepsen fsync risk; config-only change |
| **P1** | DLQ service subscribing to JetStream advisories | Med | Prevents infinite retry loops; operational necessity |
| **P1** | Speculative execution ("first response wins") for LLM routing | Med | Requires inbox management and cancellation; high latency-reduction value |
| **P1** | Subject hierarchy + queue groups for model routing | Low | Core NATS patterns; Phase 9 routing plane |

### Tier 2: Coordination Primitives (Post-Phase 9)

| Priority | Capability | Effort | Justification |
|:---------|:-----------|:-------|:-------------|
| **P2** | Delta-CRDT layer using Diamond-types for shared agent workspaces | Med-High | 4.6M ops/sec in Rust; requires CRDT <-> JetStream bridge, snapshot/rehydration, GC |
| **P2** | OR-Set for task claiming with optimistic claim protocol | Med | Replaces centralized task assignment; requires CRDT infrastructure from above |
| **P2** | Monotonic DAG CRDT for execution plan graphs | Med | Guarantees acyclicity during concurrent plan modifications |
| **P2** | Evaluator Agent for semantic conflict detection post-CRDT-merge | Med | Mitigates 5-10% semantic conflict rate |
| **P2** | Transactional Outbox pattern for crash-safe side-effects | Med | Prevents duplicate side-effects during OTP restart + CRDT replay |

### Tier 3: Advanced Guarantees (Research-grade)

| Priority | Capability | Effort | Justification |
|:---------|:-----------|:-------|:-------------|
| **P3** | MPST session types for core agent protocols | High | Compile-time deadlock-freedom; requires upfront protocol design |
| **P3** | Event-triggered consensus replacing continuous heartbeats | Med | Reduces communication 40-60%; requires threshold tuning for LLM agents |
| **P3** | CRDT quarantine actors as semantic firewalls against infectious jailbreaks | Med-High | Security-critical; filters all CRDT state before LLM prompt injection |
| **P3** | Permutation-invariant context composition for agent memory retrieval | High | Constant-time context merging; requires SSM integration or adaptation |
| **P3** | Active Anti-Entropy via Merkle trees for CRDT divergence repair | Med | Background consistency repair for dropped deltas |
| **P3** | Biomimetic fault tolerance (consensus-based threat validation for semantic health) | High | Sub-ms Byzantine voting on agent behavioral health; requires observer actor swarm |

---

## Sources

| File | Round | Content |
|:-----|:------|:--------|
| `synthesis/nats-native-agent-patterns-R3.md` | R3 | Triple synthesis: NATS routing, memory, config, telemetry, streaming, service mesh, edge, security, implementation blueprints |
| `research/targeted-stigmergy-swarm-coordination-R4.md` | R4 | 52 papers: stigmergy, blackboard architectures, swarm intelligence, decentralized coordination, shared memory, task allocation, Byzantine resilience |
| `research/targeted-crdt-coordination-R6.md` | R6 | Deep dive: delta-CRDTs, Diamond-types benchmarks, CodeCRDT evaluation, JetStream KV CAS, Monotonic DAG CRDT, Jepsen analysis, OTP rehydration, Outbox pattern |
| `research/discovery-sweep-R4.md` | R4 | Discovery: CRDTs as formal analog to stigmergy, DAG execution, MaAS, inter-agent security |
| `research/discovery-sweep-R5.md` | R5 | Discovery: event-triggered consensus (PSO-GA, fuzzy, hybrid), FoA semantic routing, MAS^2 recursive self-generation, cognitive synergy (OSC CKMs) |
| `research/discovery-sweep-R7c.md` | R7 | Discovery: MPST session types in Rust (Mozilla Servo), distributed cognition, Google scaling laws (180 configs), Vercel fewer-is-more, agent hijacking |
| `research/discovery-sweep-R7d.md` | R7 | Discovery: MPST + pi-calculus in Rust, permutation-invariant context composition (PICASO), causal context meshes (functorial mappings), biomimetic immunity, game-theoretic mechanism design, infectious jailbreaks (Agent Smith), AdaptOrch topology routing |

**Key external references cited across sources:**
- CodeCRDT (Pugachev 2025): https://arxiv.org/pdf/2510.18893
- Delta-CRDTs (Almeida et al. 2018): https://arxiv.org/abs/1603.01529
- Diamond-types benchmarks: https://josephg.com/blog/crdts-go-brrr/
- Jepsen NATS 2.12.1 analysis: https://jepsen.io/analyses/nats-2.12.1
- NATS JetStream KV docs: https://docs.nats.io/nats-concepts/jetstream/key-value-store
- PICASO (2025): https://arxiv.org/html/2502.17605v1
- Rumpsteak session types: https://lib.rs/crates/rumpsteak-types
- AdaptOrch (2026): https://arxiv.org/html/2602.16873
- Agent Smith infectious jailbreak: https://www.researchgate.net/publication/380897242
