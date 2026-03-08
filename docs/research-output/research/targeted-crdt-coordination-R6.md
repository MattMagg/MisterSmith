---
version: R6
created: 2026-03-07
updated: 2026-03-07
sources: Ultra2x deep research
round: 6 (Frontier Deep Dives)
---

# Architecting Observation-Driven Agent Coordination: Integrating CRDTs and NATS JetStream in Mister Smith

## Executive Summary

The integration of Conflict-free Replicated Data Types (CRDTs) into "Mister Smith" -- a Rust-based, NATS/JetStream-orchestrated multi-agent framework -- represents a paradigm shift from explicit message-passing to **observation-driven coordination** (stigmergy). By allowing agents to modify shared state and observe updates without centralized locking, CRDTs unlock lock-free, partition-tolerant concurrent execution.

However, empirical data reveals that CRDTs are not a universal silver bullet. The CodeCRDT study (600 trials) demonstrated that while observation-driven coordination achieves 100% syntactic convergence with zero merge failures, it yields highly variable performance: up to a **21.1% speedup** on independent tasks, but up to a **39.4% slowdown** on tightly coupled tasks due to LLM code volume inflation (82-189%) [1]. Furthermore, while CRDTs resolve character-level conflicts, they still produce a **5-10% semantic conflict rate** (e.g., duplicate logic) requiring application-level reconciliation [1].

To maximize ROI, Mister Smith must adopt a **hybrid architecture**:
1. **Delta-CRDTs over JetStream Pub/Sub** for high-churn, loosely coupled shared artifacts (documents, task graphs, capability registries) to minimize bandwidth and latency [2] [3].
2. **JetStream KV CAS (Compare-And-Swap)** for strict serialization points (budget drains, leader election) where linearizability is mandatory [4] [5].
3. **Ephemeral Pub/Sub Streams** for high-rate telemetry and one-shot external side-effects.

Additionally, critical operational risks must be mitigated. Jepsen testing on NATS 2.12.1 revealed that JetStream's default 2-minute `fsync` interval can lead to data loss during concurrent crashes [6] [7]. Furthermore, CRDT metadata (tombstones) grows unboundedly, requiring coordinated garbage collection [8]. Finally, OTP-style agent rehydration must be paired with the Transactional Outbox pattern to prevent duplicate side-effects upon crash recovery [9].

### Decision Matrix: When to use CRDTs vs. Pub/Sub vs. KV CAS

| Coordination Primitive | Best Fit Workloads | Consistency Model | Latency / Throughput Profile | Mister Smith Use Case |
| :--- | :--- | :--- | :--- | :--- |
| **Delta-CRDTs over Pub/Sub** | Collaborative editing, task registries, capability discovery, DAGs. | Strong Eventual Consistency (SEC) [10]. | Sub-millisecond local reads/writes; high throughput via asynchronous deltas [2]. | Shared agent workspaces, stigmergic planning, distributed TODO lists. |
| **JetStream KV CAS** | Budgets, leader election, strict state transitions. | Linearizable (Monotonic reads/writes) [4] [11]. | Higher latency (synchronous round-trip to server); lower throughput [5]. | Enforcing non-negative budgets, exclusive global locks. |
| **JetStream Streams** | Telemetry, external API commands, audit logs. | At-least-once / Exactly-once (with deduplication) [11]. | High throughput, tunable latency (batching) [12]. | Agent observability, executing idempotent external side-effects. |

### Honest Assessment: Core Primitive or Specialized Optimization?
CRDTs should be adopted as a **new core primitive**, but *strictly scoped to shared state artifacts*. They provide a category of capability -- lock-free, deterministic convergence of shared context -- that message-passing cannot natively achieve without immense cognitive overhead for the LLM. However, they do not replace message-driven streams for side-effects or KV CAS for strict invariants.

---

## Workload Taxonomy & Coordination Paradigms

The success of observation-driven coordination depends entirely on the coupling of the underlying task.

### CodeCRDT's Speedup vs. Slowdown Dynamics
The CodeCRDT evaluation of LLM agents coordinating via shared state revealed a critical dichotomy. For tasks with independent components (e.g., a Visualizer app), parallel agents achieved a **51.8% faster per-character generation rate** [1]. However, for highly coupled tasks (e.g., a Markdown Editor), the system experienced a **5.8% slower per-character rate** and up to a 39.4% overall slowdown [1].

This slowdown is not due to CRDT inefficiency, but rather **LLM code volume inflation**. Agents operating independently via observation tend to generate 82-189% more code (adding redundant safety checks and optimizations) because they lack a globally planned, tightly coupled execution path [1].

### Mapping Mister Smith Workloads to Coordination Modes

| Workload Type | Recommended Primitive | Rationale & LLM Implications |
| :--- | :--- | :--- |
| **Shared TODO / Claiming** | OR-Set (CRDT) + Optimistic Claim | Agents observe the set, claim a task via a delta update, and verify. Reduces LLM prompt size by only showing pending tasks. |
| **Capability Registries** | LWW-Element-Set (CRDT) | Agents broadcast their availability. LWW handles rapid status toggles (online/offline) efficiently [8]. |
| **Collaborative Code/Docs** | Sequence CRDT (Diamond-types) | Allows concurrent, lock-free editing. LLMs observe diffs rather than passing full files via messages. |
| **Budget / Quota Tracking** | JetStream KV CAS | CRDT counters cannot natively enforce global invariants (e.g., budget >= 0) without complex escrow [13]. KV CAS provides strict linearizability [5]. |
| **External Side-Effects** | JetStream Work Queue | Side-effects (e.g., calling a payment API) are not commutative. Requires exactly-once stream semantics [11]. |

### LLM Cognitive Load and Observation Frequency
Observation-driven coordination shifts the cognitive load from *communication* to *context integration*. Instead of parsing explicit messages ("Agent A completed Task 1"), the LLM observes the updated state ("Task 1 is marked done"). To prevent prompt context window exhaustion, Mister Smith must filter CRDT state before injecting it into the LLM prompt, presenting only the localized delta-changes or the specific sub-graph relevant to the agent's current focus.

---

## Mapping Mister Smith Primitives to CRDT Types

Selecting the correct mathematical CRDT model is vital to prevent application-level anomalies.

### Capability Registries: OR-Sets vs. 2P-Sets
For agent capability discovery, an **OR-Set (Observed-Remove Set)** is required. In a 2P-Set (Two-Phase Set), once an element is removed, it can *never* be added again [8]. If an agent goes offline (removed) and comes back online (added), a 2P-Set would permanently block the re-addition. An OR-Set uses unique tags for each addition, allowing elements to be added and removed repeatedly [8].

### Budget Tracking: The Non-Negative Invariant Problem
Standard CRDT counters (PN-Counters) scale with the number of replicas but cannot natively enforce global invariants, such as preventing a budget from dropping below zero [8]. If two agents concurrently decrement a budget of 1, the final state will be -1.
* **Solution A (Escrow)**: Allocate a specific fraction of the budget to each agent's local replica (escrow transactions) [13] [14].
* **Solution B (KV CAS)**: Use NATS JetStream KV CAS for budget decrements. The synchronous round-trip guarantees the budget never breaches 0 [5]. For Mister Smith, **Solution B** is recommended for simplicity and strict safety.

### Task Dependency Graphs: Monotonic DAGs
To represent execution plans, Mister Smith should use a **Monotonic DAG CRDT**. General graph CRDTs can accidentally form cycles during concurrent edge additions [15]. A Monotonic DAG enforces a local precondition: an edge can only be added if it strengthens an existing partial order, guaranteeing acyclicity upon convergence [15].

### Collaborative Artifacts: Sequence CRDTs
For shared code or text generation, Sequence CRDTs (like RGA or YATA) are required. They assign unique identifiers to characters/blocks and use tombstones for deletions, ensuring that concurrent inserts interleave deterministically without manual conflict resolution [8] [16].

---

## Transport Architecture: Delta-CRDTs vs. JetStream KV

Mister Smith operates on NATS JetStream. We must choose between using JetStream KV as the authoritative state versus disseminating delta-CRDTs over standard JetStream subjects.

### JetStream KV CAS: Synchronous Bottlenecks
JetStream KV provides immediate consistency for monotonic reads/writes and supports atomic Compare-And-Swap (CAS) via the `Nats-Expected-Last-Subject-Sequence` header [5].
* **The Bottleneck**: CAS operations require a synchronous round-trip to the server. If multiple agents attempt to update a shared state concurrently, only one succeeds; the others must fetch the new revision and retry [5]. This limits throughput and makes client-side batching impossible for dependent operations [5].

### Delta-CRDTs over Pub/Sub: Bandwidth Efficiency
Delta-CRDTs solve the state-transmission problem by generating small incremental states (deltas) instead of shipping the full state [2] [3].
* **The Architecture**: Agents maintain an in-memory CRDT replica. Mutations generate a delta-state, which is published to a JetStream subject (e.g., `crdt.task_graph.deltas`). Other agents subscribe to this subject, receive the delta, and join it with their local state [2].
* **Advantage**: This is entirely asynchronous and lock-free. Agents do not wait for server validation to proceed, maximizing throughput and hiding network latency [5].

### Handling NATS Cluster Partitions
The CAP theorem dictates that JetStream's linearizable KV store cannot be totally available during a network partition [6]. If a partition isolates an agent from the JetStream quorum, KV CAS operations will fail. Conversely, an in-memory delta-CRDT allows the agent to continue reading and writing locally (High Availability). Once the partition heals, the agent publishes its buffered delta-groups, and Strong Eventual Consistency (SEC) guarantees convergence [10] [3].

---

## Performance, Scale, and Rust Implementations

The choice of Rust CRDT library drastically impacts Mister Smith's resource footprint at 10-1,000 concurrent writers.

### Rust CRDT Benchmarks

| Library | Architecture / Algorithm | Performance Profile | Best Use Case in Mister Smith |
| :--- | :--- | :--- | :--- |
| **Diamond-Types** | B-Tree / Range Tree (YATA/RGA) | **4.6 Million ops/sec**. Processes 260k edits in 56ms. Minimal memory (1.1 MB) [17]. | High-frequency collaborative text/code generation. |
| **Loro** | Replayable Event Graph (Rust) | High performance, optimized for JSON-like nested structures and rich text [18] [19]. | Complex agent state, nested capability registries, JSON artifacts. |
| **Automerge-rs** | Columnar compression (Rust) | Slower than Diamond-types but highly mature. Excellent binary compression [18] [17]. | Long-term document storage requiring deep history retention. |

### The Cost of Causality and Metadata
CRDTs achieve lock-free merges by retaining metadata. For Sequence CRDTs and OR-Sets, every deleted element becomes a **tombstone** [8]. A 1,000-character document that has been heavily edited might internally contain 50,000 tombstones [8].

Furthermore, causal CRDTs rely on **Dotted Version Vectors** or causal contexts to track which updates have been seen [20] [21]. While highly compressible, this metadata grows linearly with the number of active replicas (agents) [8]. At 1,000 concurrent writers, the vector clock overhead becomes non-trivial, necessitating periodic compaction.

---

## Consistency, Semantic Conflicts, and Failure Modes

### The Semantic Conflict Reality
CRDTs guarantee *syntactic* convergence -- all replicas will hold the exact same bytes [10]. However, they do not guarantee *semantic* correctness. The CodeCRDT study found a **5-10% semantic conflict rate** where LLM agents generated duplicate variable declarations or conflicting logic that merged perfectly at the character level but failed to compile [1].
* **Mitigation**: Mister Smith must deploy an "Evaluator Agent" that observes the converged CRDT state, runs semantic checks (e.g., `cargo check`), and issues corrective delta-updates if the state is semantically invalid [1].

### Optimistic Claim Protocols (At-Most-One Execution)
To prevent multiple agents from executing the same task, Mister Smith should implement an optimistic `read-derive-delta-write-verify` protocol over a CRDT Map (LWW semantics per key) [1]:
1. **Scan**: Agent reads local CRDT for `assignedTo == null`.
2. **Claim**: Agent writes delta setting `assignedTo = self_id`.
3. **Verify**: Agent waits a short sync delay (e.g., 50ms) and re-reads the local state.
4. **Proceed**: If `assignedTo == self_id` after convergence, the claim succeeded. Otherwise, it backs off [1].

### Jepsen NATS Findings: Mitigating Data Loss
Jepsen testing on NATS 2.12.1 highlighted a critical failure mode: by default, JetStream calls `fsync` only once every two minutes, but acknowledges messages immediately [6]. A coordinated power failure or OS crash can result in the loss of acknowledged writes [6] [7].
* **Mitigation**: For critical Mister Smith streams (e.g., budget tracking, final artifact commits), configure the JetStream stream with `sync_interval: always`. This forces an `fsync` before acknowledgment, ensuring absolute durability at the cost of reducing throughput to a few hundred msgs/sec [6] [11].

---

## Agent Architecture & OTP Supervision Integration

Integrating CRDTs with Rust's OTP-style supervision trees requires careful handling of state rehydration and side-effects.

### Snapshotting and Rehydration
When an agent crashes, the supervisor restarts it. The agent must rehydrate its local CRDT state.
1. **Snapshot**: A dedicated compactor agent periodically writes a compressed binary snapshot of the CRDT to a JetStream KV bucket.
2. **Rehydration**: The restarted agent fetches the latest snapshot from KV, then subscribes to the delta-CRDT JetStream subject, replaying only the delta-messages that occurred *after* the snapshot's sequence number.

### Preventing Duplicate Side-Effects (The Outbox Pattern)
If an agent crashes after updating a CRDT but before triggering an external API (e.g., sending an email), replaying the CRDT log upon restart might cause the agent to re-trigger the email.
* **Mitigation**: Implement the **Transactional Outbox Pattern** [9]. The agent writes the intended side-effect as a pending operation into the CRDT state. A separate, idempotent executor process observes the CRDT, executes the API call, and updates the CRDT to mark it complete.
* **JetStream Deduplication**: Leverage JetStream's `Nats-Msg-Id` header. If the agent attempts to publish a duplicate side-effect command during a replay window, JetStream's deduplication (default 2-minute window) will silently drop the duplicate [22] [11].

---

## Lessons from Production Deployments

### Figma: Centralized Relays vs. True P2P
Figma utilizes CRDT concepts but relies on a centralized server to dictate the total order of events, bypassing the need for complex vector clocks [23]. For Mister Smith, JetStream acts as this centralized sequencer. By routing delta-CRDTs through JetStream, we achieve a globally ordered log of events, simplifying causal stability calculations compared to a true P2P mesh.

### Riak: Active Anti-Entropy (AAE)
Riak utilizes Active Anti-Entropy (AAE) via on-disk Merkle trees to detect and repair divergent replicas in the background [24]. Mister Smith can implement a lightweight AAE mechanism where agents periodically publish a hash of their local CRDT state. If hashes diverge, agents can request a full state sync to repair dropped delta-messages.

### Lattica & Galeas: Edge and IoT
Lattica (2025) and Galeas (2025) demonstrate that delta-CRDTs combined with pub/sub are highly effective in constrained, cross-NAT environments [25] [26]. This validates Mister Smith's architecture for edge deployments, allowing agents to operate locally during network partitions and sync seamlessly via JetStream when connectivity is restored.

### Security and Multi-Tenancy
To secure multi-tenant agent workspaces, Mister Smith should leverage NATS decentralized JWT authentication and NKEYS [27] [28].
* **Per-Subject ACLs**: Restrict agents to specific CRDT subjects (e.g., `crdt.tenant_A.>`) using JWT permissions [29].
* **Encryption at Rest**: Enable JetStream's native encryption at rest (ChaCha20-Poly1305 or AES-GCM) using `$JS_KEY` to protect stored delta-logs and snapshots [30].

---

## Implementation Roadmap & Evaluation Plan

To conclusively determine if CRDTs should become the core primitive, Mister Smith should execute a phased evaluation.

### 1. Prototype Scope
Implement two distinct artifacts using both delta-CRDTs (over subjects) and JetStream KV CAS:
1. **Task Registry**: An OR-Set for agents to claim TODOs.
2. **Budget Counter**: A shared token pool for LLM API usage.

### 2. Experiment Matrix

| Variable | Test Conditions | Objective |
| :--- | :--- | :--- |
| **Concurrency** | 10, 100, 1,000 concurrent agent writers. | Measure CPU/Memory overhead and JetStream backpressure. |
| **Network State** | Healthy, 200ms latency, 30-second partition. | Validate SEC convergence and local-first responsiveness. |
| **Failure Injection** | Random agent SIGKILLs during writes. | Verify OTP rehydration and Outbox pattern idempotence. |

### 3. Success Thresholds (Go/No-Go Criteria)
* **Performance**: Delta-CRDT local read/write latency must remain under 5ms at p95.
* **Convergence**: 100% syntactic convergence across all partition tests.
* **Semantic Safety**: Semantic conflict rate must be quantifiable and automatically resolvable by the Evaluator Agent in >95% of cases.
* **Memory**: CRDT metadata (tombstones/dots) must not exceed 50MB per agent after 10,000 operations (validating GC/compaction logic).

### 4. Migration and Rollback Plan
Deploy the CRDT architecture in **Shadow Mode**. Agents will continue to coordinate via explicit messages, but will dual-write their state to the new CRDT structures. An observability process will compare the CRDT state against the legacy state to detect divergence. If the CRDT state proves stable, read paths can be incrementally shifted. If catastrophic metadata growth or unresolvable semantic conflicts occur, the system can seamlessly roll back to the legacy message-passing architecture.

## References

1. https://arxiv.org/pdf/2510.18893
2. https://arxiv.org/abs/1603.01529
3. https://arxiv.org/pdf/1603.01529.pdf
4. https://docs.nats.io/nats-concepts/jetstream/key-value-store
5. https://medium.com/@sudojha/nats-kv-vs-jetstream-performance-analysis-and-architectural-trade-offs-0866cf151e6d
6. https://jepsen.io/analyses/nats-2.12.1
7. https://github.com/nats-io/nats-server/issues/7564
8. https://iankduncan.com/engineering/2025-11-27-crdt-dictionary/
9. https://medium.com/threadsafe/exactly-once-processing-across-kafka-and-databases-using-the-outbox-pattern-f08fd640f683
10. https://www.lip6.fr/Marc.Shapiro/papers/2011/CRDTs_SSS-2011.pdf
11. https://docs.nats.io/jetstream
12. https://onidel.com/blog/nats-jetstream-rabbitmq-kafka-2025-benchmarks
13. https://www.lip6.fr/Marc.Shapiro/papers/2018/CRDTs-Springer2018-authorversion.pdf
14. https://www.dpss.inesc-id.pt/~rodrigo/srds15.pdf
15. https://hal.inria.fr/inria-00555588/document
16. https://github.com/yjs/yjs
17. https://josephg.com/blog/crdts-go-brrr/
18. https://velt.dev/blog/best-crdt-libraries-real-time-data-sync
19. https://crdt.tech/implementations
20. https://riak.com/posts/technical/vector-clocks-revisited-part-2-dotted-version-vectors/index.html?p=9929.html
21. https://dl.acm.org/doi/10.1145/3695249
22. https://docs.nats.io/using-nats/developer/develop_jetstream/model_deep_dive
23. https://www.figma.com/blog/how-figmas-multiplayer-technology-works/
24. https://docs.riak.com/riak/kv/latest/learn/concepts/active-anti-entropy/index.html
25. https://arxiv.org/abs/2510.00183
26. https://www.sciencedirect.com/science/article/pii/S1077314225001602
27. https://docs.nats.io/running-a-nats-service/nats_admin/security/jwt
28. https://docs.nats.io/using-nats/nats-tools/nsc/basics
29. https://docs.nats.io/running-a-nats-service/configuration/securing_nats/authorization
30. https://docs.nats.io/running-a-nats-service/nats_admin/jetstream_admin/encryption_at_rest
