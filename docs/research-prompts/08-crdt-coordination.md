---
version: R1
created: 2026-03-07
updated: 2026-03-07
type: prompt
tier: 1
---

# Deep Research Prompt: CRDT-Based Agent Coordination

## Context

Mister Smith is a Rust-based multi-agent orchestration framework built on NATS/JetStream messaging and OTP-style supervision trees. Agents currently coordinate through explicit message passing over NATS pub/sub (direct messages, request-reply, broadcast).

Our research has identified CRDTs (Conflict-free Replicated Data Types) as a fundamentally different coordination model: agents modify shared state and observe each other's modifications, rather than exchanging explicit messages. This is the formal computer science analog to stigmergy — indirect coordination through the environment.

This finding has the potential to give Mister Smith a coordination primitive that no competing agent framework possesses.

## Frontier-First Mandate

Do not evaluate CRDTs as "another tool in the box." Evaluate them as a potential core coordination primitive — a fundamentally different way for agents to collaborate that may be superior to message passing for certain workload patterns. If CRDT-based coordination can replace or augment explicit message passing, it creates a category of capability that doesn't exist in any current agent framework.

## Research Objective

Determine whether and how CRDTs can serve as a coordination primitive for LLM-based multi-agent systems. Specifically: what workloads benefit from observation-driven (CRDT) coordination vs. message-driven (pub/sub) coordination? How do CRDTs compose with NATS JetStream KV? What are the performance characteristics at agent scale?

## What We Already Know (Do Not Rediscover)

- CodeCRDT (Pugachev 2025): 100% convergence, zero merge failures in 600 concurrent code generation trials, 5-10% semantic conflict rate, up to 21.1% speedup
- Lattica (Yang et al. 2025): Full decentralized framework using CRDTs + DHTs + NAT traversal for distributed AI
- Delta-CRDTs over pub/sub (Galeas et al. 2025): Satisfactory latency and consistency for distributed agent knowledge management
- Stigmergy-RL formal equivalence (Vellinger 2025): JetStream KV with TTL as coordination primitive is mathematically grounded
- Rust CRDT ecosystem: `crdts` crate, `automerge` crate exist
- JetStream KV provides atomic Compare-And-Set (CAS) operations with revision-based optimistic concurrency

We need: implementation depth, performance boundaries, failure modes, the hybrid model design (when CRDTs vs. when messages), and the JetStream integration architecture.

## Research Dimensions

### 1. CRDT Types for Agent Coordination
- Which CRDT types are most applicable to agent coordination? (G-Counter for token budgets? OR-Set for capability registries? LWW-Register for agent state? RGA/Sequence for shared documents?)
- What does the CRDT type system look like for a multi-agent framework? Is there a canonical set of CRDTs that covers the coordination patterns agents need?
- What about domain-specific CRDTs designed for agent workloads? Has anyone built CRDTs for coordination artifacts (plans, task assignments, dependency graphs)?
- How do CRDTs handle schema evolution? If the shared state structure changes, what happens to existing replicas?

### 2. CRDTs Over JetStream KV
- JetStream KV uses revision-based CAS for conflict resolution. How does this interact with CRDT merge semantics? Can JetStream KV be used as a CRDT transport, or does it need modification?
- Delta-CRDTs only transmit changes (deltas) rather than full state. Can these deltas be published as NATS messages and applied to local replicas?
- What is the latency profile? NATS KV operations are in-memory with optional persistence — how does this compare to purpose-built CRDT stores?
- What happens during NATS cluster partitions? CRDTs are designed for eventual consistency — does JetStream's Raft consensus layer conflict with CRDT semantics?
- Can NATS KV watches (real-time notifications on key changes) serve as the CRDT change propagation mechanism?

### 3. Observation-Driven vs. Message-Driven Coordination
- CodeCRDT demonstrates observation-driven coordination for code generation. What other agent workloads benefit from this model?
- When is observation-driven coordination strictly better than message passing? When is it strictly worse?
- What about hybrid models — agents use CRDTs for shared artifact coordination (code, documents, plans) and NATS pub/sub for event notifications and commands?
- How does observation-driven coordination scale with agent count? Message passing scales linearly (N messages for N agents). Does CRDT observation scale better or worse?
- What is the cognitive overhead for agent reasoning? Does an agent need to understand CRDT merge semantics, or can this be hidden behind an abstraction?

### 4. Performance and Scale
- What are the performance characteristics of CRDT operations at the scale relevant to agent systems (10-1000 concurrent writers)?
- How large can a CRDT grow before merge operations become expensive? What garbage collection / compaction strategies exist?
- Delta-CRDTs reduce bandwidth. What is the bandwidth savings vs. full-state CRDTs for agent coordination workloads?
- What is the memory overhead of maintaining CRDT replicas per agent? Does this compete with context window memory?
- Has anyone benchmarked CRDT performance in Rust? How do `crdts` and `automerge` compare on throughput and memory?

### 5. Consistency, Conflicts, and Failure Modes
- CRDTs guarantee eventual consistency. What does "eventual" mean for agent coordination? Is there a latency bound?
- Semantic conflicts (CodeCRDT reports 5-10%) are different from data conflicts. How do agents detect and resolve semantic conflicts?
- What happens when an agent reads stale CRDT state and makes a decision based on it? Is this better or worse than an agent receiving a late message?
- How do CRDTs interact with supervision trees? If an agent crashes mid-CRDT-update, is the shared state corrupted?
- What are the known failure modes of CRDTs in distributed systems? (unbounded growth, metadata overhead, causal consistency violations)

### 6. Integration with Agent Architecture
- What does the API look like for agent developers? Should agents interact with CRDTs explicitly or through a higher-level coordination abstraction?
- Can CRDTs serve as the backing store for agent memory (tiered memory: STM as CRDT, MTM as JetStream, LTM as PostgreSQL)?
- How do CRDTs compose with the existing actor model? Does each actor maintain its own CRDT replica, or is there a shared CRDT service?
- Can the task dependency graph be represented as a CRDT, allowing agents to concurrently modify the plan?
- How does CRDT-based coordination interact with budget enforcement and rate limiting?

### 7. Adjacent-Field Patterns
- What does the collaborative editing literature (Google Docs, Figma, VS Code Live Share) say about CRDT performance at scale?
- What do distributed databases (Riak, AntidoteDB, Redis CRDTs) say about operational patterns?
- What does the distributed gaming literature say about using CRDTs for real-time shared world state?
- Are there CRDT patterns from IoT / edge computing that apply to resource-constrained agent deployments?

## Output Structure

For each dimension:
1. **State of the art** — what exists, with citations
2. **Key techniques** — specific data structures, algorithms, or protocols
3. **Applicability to Mister Smith** — concrete integration with NATS JetStream KV, actor model, supervision trees
4. **Performance characteristics** — latency, throughput, memory, bandwidth (with numbers where available)
5. **Open problems** — what doesn't work yet, what's unsolved

Conclude with:
- A concrete design sketch for CRDT coordination in Mister Smith (which CRDT types, where they live, how agents interact with them)
- A decision framework: when to use CRDTs vs. NATS pub/sub vs. JetStream streams
- An honest assessment of whether CRDTs are a core architectural primitive or a specialized optimization for specific workloads

## Research Methodology

1. Start with CodeCRDT, Lattica, and Galeas et al. — trace citation graphs
2. Deep dive into the CRDT literature (Shapiro et al., Kleppmann, Almeida et al.) for foundational understanding
3. Investigate Rust CRDT implementations: `crdts`, `automerge`, `diamond-types` — capabilities, performance, maturity
4. Look at production CRDT deployments (Figma, Redis CRDTs, Riak) for operational lessons
5. Search for "CRDT multi-agent", "conflict-free coordination", "observation-driven multi-agent"
6. Look at the stigmergy and swarm intelligence literature for theoretical foundations
7. Prioritize 2025-2026 papers but include foundational CRDT work (Shapiro 2011, Kleppmann 2019)
