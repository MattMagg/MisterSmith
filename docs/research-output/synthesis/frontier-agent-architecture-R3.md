---
version: R3
created: 2026-03-07
updated: 2026-03-07
sources: Ultra2x (3 reports) → Synthesized
round: 3 (Triple Synthesis)
---

# Frontier Agent Architecture: A Comprehensive Research Synthesis for Mister Smith

## Executive Summary

This report synthesizes three independent research investigations into frontier architectural concepts for AI agent orchestration, evaluated against Mister Smith's Rust + NATS/JetStream + OTP-style supervision stack. The convergence of findings across all three reports yields high-confidence conclusions on the most impactful directions for the framework.

**High-confidence consensus findings** (independently reached by all three reports):

1. **The "LLM-as-OS" paradigm is the dominant architectural metaphor for 2026 agent systems.** Treating the orchestration runtime as a microkernel -- with agents as processes, tools as syscalls, context windows as RAM, and external stores as disk -- yields measurable improvements. MemOS demonstrates 49% better long-context retention [A1], AIOS achieves 2.1x faster execution [A13], and Mem0 delivers 91% lower p95 latency with 90% token cost savings [A17]. Mister Smith's existing actor model and NATS IPC map cleanly onto this paradigm.

2. **Capability-based security is essential and RBAC is insufficient.** All three reports converge on unforgeable capability tokens (Macaroons, ZCAP-LD) inspired by seL4's formally verified microkernel as the correct security model for multi-agent systems. Prompt injection renders traditional ACL/RBAC vulnerable to the "confused deputy" problem [A30][A31]. Wasmtime/WASI sandboxing for tool execution provides the strongest isolation-to-performance ratio [A4].

3. **Tiered memory (STM/MTM/LTM) with OS-style paging is mandatory.** Relying on expanding context windows is computationally ruinous. Every report recommends a hierarchical memory architecture backed by JetStream KV (mid-term) and vector databases (long-term), with consolidation agents running as supervised background processes.

4. **Protocol interoperability (MCP + A2A) is non-negotiable.** The "protocol wars" between Anthropic's MCP, Google's A2A, and emerging WebMCP demand a "speak-all-protocols" adapter layer. Proprietary protocols are a guaranteed path to obsolescence. Early adoption of open standards provides strategic advantage.

5. **Timeless distributed systems abstractions will outlast AI paradigm shifts.** Despite predictions that models will internalize orchestration, LLMs still lack grounded world models, reliable planning, and long-term memory [A11][A46]. Erlang/OTP supervision trees, IPC message buses, and microkernel design are the correct foundations -- they have powered telecom systems for decades and represent the ultimate hedge against non-deterministic LLM behavior.

**Areas of productive disagreement across reports:**

- **Swarm intelligence as coordination mechanism:** Report A advocates strongly for stigmergic blackboards with JetStream KV TTLs, citing 80% token reduction [A26]. Report B rates swarm patterns as "low-medium" differentiation. Report C notes LLMs currently struggle with decentralized coordination on SwarmBench [C54]. Synthesis: stigmergy is high-value for specific use cases (massive agent counts, environment-mediated coordination) but should not replace deterministic orchestration for critical paths.

- **Hardware-aware execution priority:** Report A positions NUMA-aware actor pinning and disaggregated serving as high-impact. Report C considers hardware optimization "lower priority" since LLM inference time dominates latency. Synthesis: hardware awareness matters at scale when orchestration overhead becomes visible; it is incremental rather than transformative for most deployments.

- **CRDTs for distributed coordination:** Report C uniquely highlights CodeCRDT for multi-agent coordination [C41], which neither Report A nor B covers in depth. Report A recommends Delta-CRDTs for geo-distributed blackboards [A28][A29]. This convergence from different angles suggests CRDTs deserve serious investigation.

**Frontier concepts recommended for immediate prototyping:**
1. JetStream-backed tiered memory with OS-style paging
2. Capability-based security with Wasmtime/WASI tool sandboxing
3. Native MCP/A2A protocol adapters with canonical internal Rust schema
4. Continuous evaluation with JetStream-backed golden trace replay
5. Rust compile-time protocol verification via session types

**Concepts to monitor but not yet implement:**
- Neural Paging controllers (learned context eviction)
- Full formal verification of LLM-in-the-loop behaviors
- Multi-agent reinforcement learning (AT-GRPO and similar)
- FPGA/ASIC-accelerated orchestration

---

## 1. The Agent-OS Paradigm: Memory, Kernels, and Process Abstractions

### 1.1 The Microkernel Analogy

All three reports independently arrive at the same core architectural metaphor: the agent orchestration runtime should be structured as a microkernel operating system.

| OS Concept | Agent Mapping | Mister Smith Implementation |
|:---|:---|:---|
| Processes | Agents (actors) | Tokio-spawned actor tasks with bounded mailboxes |
| Threads | Agent sub-tasks | Tokio tasks within actor scope |
| System calls | Tool invocations | Message-based tool bus with capability checks |
| IPC | Agent-to-agent messaging | NATS subjects and JetStream consumers |
| Virtual memory / RAM | Context window | In-actor working memory (STM) |
| Disk / swap | External storage | JetStream KV (MTM), vector DB (LTM) |
| Kernel | Supervisor tree | OTP-style supervision with restart strategies |
| MMU / page controller | Context manager | Heuristic or learned eviction controller |
| Capability tokens | Permission grants | Unforgeable cryptographic tokens (Macaroons/ZCAP-LD) |

**Research evidence:**

- **AIOS** (Rutgers, 2024) explicitly maps OS abstractions to agent systems, isolating LLM-specific services into a kernel layer and achieving 2.1x faster execution through resource isolation [A13].
- **KAOS** runs on openKylin with management agents and shared resource scheduling, demonstrating a full multi-agent OS on a real kernel [C7].
- **AgentStore** implements a meta-agent app-store for heterogeneous agents, analogous to a package manager [C9].
- **seL4** provides the gold standard for microkernel design: minimal trusted computing base (TCB), formally verified invariants, and capability-based authorization. Its design principles -- small TCB, formalizable invariants, capability objects for safe communication and resource delegation -- translate directly to agent kernel design [B1][B2].

**Mister Smith mapping (high confidence):** Mister Smith's existing architecture already embodies this pattern. The supervisor tree acts as the kernel, actors are user-space processes, and NATS provides IPC. The key missing piece is a formal "kernel API" crate that explicitly surfaces scheduling primitives, capability enforcement, and context paging as first-class abstractions.

**Implementation recommendation:** Design a `mister-smith-kernel` crate (or evolve the existing runtime) that exposes:
- Lightweight scheduling primitives and quotas for actors (agents-as-processes)
- Capability tokens for tool access and inter-agent IPC
- A context store abstraction supporting paging of in-context state to external stores (JetStream KV or vector DB)

Keep the kernel small and move complex policies into supervised agents. This follows the microkernel principle: minimize the TCB and push policy to user-space.

### 1.2 Tiered Memory Architecture

All three reports converge on hierarchical, multi-tier memory as the single most impactful architectural pattern for agent systems. The evidence is strong and the implementation path is clear.

**Memory tier design:**

| Tier | Analogy | Backing Store | Latency | Capacity | Eviction Policy |
|:---|:---|:---|:---|:---|:---|
| **Working Memory (STM)** | CPU cache / RAM | In-actor Rust state | Microseconds | Limited (context window) | LRU / relevance scoring |
| **Episodic Memory (MTM)** | Page file / SSD | JetStream Key-Value | Low milliseconds | Large | TTL + access frequency |
| **Semantic Memory (LTM)** | Disk / archive | Vector database | Milliseconds | Unbounded | Consolidation + dedup |
| **Archival / Procedural** | Cold storage | PostgreSQL / JetStream streams | Higher milliseconds | Unbounded | Retention policy |

**Research evidence and benchmarks:**

- **MemOS** (2025): Hierarchical STM/MTM/LPM storage with dialogue-chain FIFO and segmented paging. Achieves 49.11% improvement on F1 scores and 46.18% on BLEU-1 for long conversations on the LoCoMo benchmark [A1].
- **MemGPT / Letta**: OS-style virtual context management with explicit interrupts. Implements `core_memory_append` and `memory_replace` tool calls. Uses a FIFO buffer and recall DB to manage and purge context. Enables an "infinite context illusion" within fixed windows [A14][A16][C18][C20].
- **Mem0**: Dynamic extraction, consolidation, and graph-based relational structures. Achieves 91% lower p95 latency and 90% token cost savings through intelligent memory management [A17][B15][B16].
- **Collaborative Memory Framework**: Two-tier memory (private per-agent, shared for knowledge transfer) governed by dynamic access graphs [C15].
- **IBM Episodic Memory**: Stores past workflows as episodic memories to suggest next tasks, enabling agents to learn from prior executions [C58].

**Consolidation and sleep-time compute:** Multiple reports highlight asynchronous background agents for memory consolidation. Letta's "sleep-time compute" allows background actors to handle memory consolidation without blocking the main reasoning loop [A15]. Mem0 uses vector similarity thresholds for deduplication and clustering to improve precision/space tradeoffs [B15][B16]. These consolidation agents should run as supervised background processes within Mister Smith's supervision tree.

**JetStream as the memory backbone (high confidence):** JetStream KV provides immediate consistency and monotonic reads [A20], making it ideal for the MTM tier. Its TTL feature naturally implements temporal decay for episodic memories. Durable consumers enable asynchronous memory graph construction. JetStream streams provide archival storage with replay capability for audit and evaluation.

**Implementation roadmap:**
1. Design Memory API (Rust crate) with async `recall`, `put`, `consolidate`, and `snapshot` primitives
2. Implement in-memory working context per actor with configurable size limits
3. Build JetStream KV adapter for episodic/mid-term memory with TTL-based eviction
4. Create pluggable vector-store connector for semantic recall (support multiple backends)
5. Implement supervised background consolidation agents
6. Target metrics: retrieval latency, consolidation throughput, storage reduction (use Mem0's published numbers as baseline targets)

### 1.3 Neural Paging and Learned Context Controllers

The frontier of memory management moves beyond heuristic eviction (LRU/LFU) toward learned controllers. This concept appears in Report A and is referenced implicitly in Report B's discussion of MemGPT paging.

**Neural Paging** introduces a secondary, lightweight, differentiable Page Controller that acts as a neural Memory Management Unit (MMU) [A18]. This controller predicts future data requirements and evicts low-utility tokens to approximate Belady's optimal algorithm. The key result: it reduces the asymptotic complexity of long-horizon reasoning from quadratic O(N^2) to O(N*K^2) [A19].

**Current assessment:** All reports agree this is too research-heavy for immediate production. However, Mister Smith should design its memory interface with a pluggable eviction strategy, allowing heuristic controllers (LRU/LFU) to be swapped for learned controllers in the future. The Memory API should abstract over the eviction policy, not hardcode it.

---

## 2. Compile-Time Protocol Verification and Formal Methods

### 2.1 Session Types for Agent Protocols

All three reports identify Rust's type system as a unique advantage for statically verifying agent communication protocols. This is a high-impact, medium-viability concept that no competing framework (typically Python-based) can replicate.

**Core idea:** Using Multiparty Session Types (MPST), developers define a global communication protocol that the Rust compiler enforces. The type system guarantees that channels are used linearly (exactly once), ensuring protocol adherence, message delivery, and deadlock freedom [A5][A36][B3][B4].

**Key research:**

- **MultiCrusty**: A Rust library providing multiparty session types for deadlock-free protocol verification [C28].
- **Affine Multiparty Session Types (AMPST)**: Extends session types to safely handle process cancellations and panics, ensuring failures propagate correctly across supervision trees without orphaned messages [A6][B4]. This is directly relevant to Mister Smith's OTP-style supervision.
- **`par` crate**: Session types for Rust with practical binary session type implementations [A5].
- **`session_types` crate**: Alternative implementation for binary session types.
- **Typestate patterns**: Rust can encode state machines at compile time using phantom types and traits, ensuring agents only call tools in valid states [C30].

**Practical applications for Mister Smith:**

1. **Tool invocation protocol**: Define session-typed client/server stubs for the tool invocation round-trip (request -> validate -> execute -> return result). The compiler rejects any code path that skips validation or fails to handle the result.
2. **Delegation chains**: Encode delegation as a linear type that must be consumed exactly once, preventing orphaned delegations or double-spending of capability tokens.
3. **Supervision lifecycle**: Use typestate to encode agent lifecycle states (Starting -> Running -> Draining -> Stopped), preventing invalid state transitions at compile time.
4. **Acyclic delegation graphs**: While full graph-level cycle detection is beyond current Rust type system capabilities (would need dependent types), local acyclicity can be enforced through careful type design.

**Bridging async NATS messaging with session types:** Report B identifies a key engineering challenge: session types assume synchronous, ordered channels, while NATS provides asynchronous pub/sub. The adapter layer must translate between session-typed APIs and NATS async messaging, potentially using correlation IDs and typed response channels.

**Verus and formal verification:** Report B uniquely highlights Verus, which extends Rust for SMT-based verification using ghost state and linear ghost permissions [B5]. This enables reasoning about resource budgets and invariants. For Mister Smith, Verus could verify properties of the kernel/scheduler without requiring full end-to-end proofs.

**Verdi and IronFleet:** For verified distributed coordination, Report B references Verdi (Coq framework for verified distributed systems) [B6] and IronFleet (mechanized verification of distributed systems) [B7]. These are multi-year research efforts but provide methodologies that could be applied to verify Mister Smith's consensus-dependent components.

**Assessment (consensus across reports):** Compile-time session types for common agent interaction patterns are implementable now and should be pursued. Full-stack formal verification (including LLM-in-the-loop behavior) is theoretical/long-term -- no primary evidence exists of end-to-end success for LLM decision semantics. Focus verification on deterministic subsystems first: protocols, state stores, capability enforcement.

### 2.2 Capability Tokens as Linear Types

A novel synthesis across reports: capability tokens should be implemented as affine (linear-like) types in Rust. When an agent receives a capability token to invoke a tool, the token is consumed on use. This prevents:
- Double-invocation of one-time capabilities
- Capability hoarding (tokens expire or are consumed)
- Unauthorized delegation (tokens can only be attenuated, not amplified)

This directly leverages Rust's ownership model -- no runtime overhead, no garbage collection, compile-time enforcement.

---

## 3. Zero-Trust Security, Sandboxing, and Prompt Injection Mitigation

### 3.1 Capability-Based Security (High Confidence)

All three reports independently arrive at capability-based security as the correct model for multi-agent systems. This is the strongest consensus finding across the research.

**The problem with RBAC:** Traditional Access Control Lists suffer from the "confused deputy" problem in agent systems. A malicious prompt can trick a highly privileged agent into abusing its broad permissions [A31]. OWASP classifies prompt injection as a top LLM vulnerability [B21].

**The solution: unforgeable capability tokens.** Inspired by seL4's formally verified capability model [A31][B1][B2]:

- A capability couples designation (the object) with authority (the right to use it) [A32]
- Capabilities can be attenuated (restricted in scope) but never amplified
- Capabilities can be delegated with progressive restriction
- If an agent is compromised via prompt injection, it can only abuse the specific, narrow capabilities it currently holds

**Implementation technologies:**

| Technology | Mechanism | Fit for Mister Smith |
|:---|:---|:---|
| **Macaroons** | Cookies with contextual caveats for decentralized authorization [A33] | Strong fit -- caveats enable progressive attenuation |
| **ZCAP-LD** | W3C authorization capabilities for linked data [A3] | Good for interop with web-based agent ecosystems |
| **Rust affine types** | Compile-time single-use enforcement | Unique advantage -- zero runtime overhead |
| **CHERIoT** | Hardware-enforced capability model [A32] | Future hardware integration path |

**Implementation in Mister Smith:**
1. Implement an Access Manager service that issues cryptographic capability tokens to agents
2. When an agent invokes a tool, it must present the specific capability token
3. Tokens are attenuated on delegation (child agents receive narrower capabilities than parents)
4. Encode capability tokens as affine types consumed on use (compile-time enforcement)
5. Persist capability-issued events to JetStream for audit [B10]
6. Integrate with existing JWT/RBAC infrastructure (Phase 5) as a migration path

### 3.2 Sandboxing for Tool Execution

Reports A and C provide complementary analysis of sandboxing technologies for agent tool execution.

| Technology | Security Model | Performance | Best Use Case |
|:---|:---|:---|:---|
| **Wasmtime/WASI** | Capability-based; no ambient authority [A4][A34] | Microsecond cold starts; near-native AOT | Stateless functions, data processing, untrusted code |
| **Firecracker (MicroVMs)** | Hardware-enforced virtualization [A4] | Fast boot (ms), heavier than WASM | Heavy, OS-dependent tools requiring full isolation |
| **gVisor** | User-space kernel syscall interception [A4] | High overhead for syscall-heavy workloads | Containerized legacy tools |
| **Linux Containers** | Namespaces and cgroups [A4] | Fast but relies on host OS privilege | Insufficient for untrusted LLM-generated code |
| **bubblewrap (bwrap)** | OS-level namespace isolation [C71] | Lightweight | Claude Code uses this on Linux |

**SandboxEscapeBench** (referenced in Report C) demonstrates that when sandboxes have holes, LLMs can exploit them [C82]. This underscores the need for defense-in-depth rather than relying on a single isolation layer.

**Recommendation (high confidence):** Wasmtime/WASI is the optimal default for Mister Smith tool execution. It provides strict capability-based filesystem and network access, preventing exfiltration even if the agent is compromised [A34]. WASM's capability model aligns naturally with Mister Smith's capability token architecture. Firecracker should be available as an option for tools requiring full OS environments.

### 3.3 Additional Security Measures

Report C uniquely highlights several additional security patterns:

- **Approval gates**: For high-risk/mutating actions, generate signed proposals requiring human confirmation [C69]. This should be a first-class primitive in the tool invocation protocol.
- **Prompt injection defense**: Sanitize and structure user inputs; use formal grammars for tool interfaces to reject malformed requests. Dual-LLM patterns (one generates, one validates) can catch injection attempts [B22].
- **Audit trails**: Log all agent attempts and decisions (capability grants, tool calls, side effects) to a tamper-evident store [C69]. Mister Smith already has Phase 5's SHA-256 hash chain audit log, which should be extended for capability events.
- **Hardware enclaves**: Future work might leverage TEEs (Intel SGX) for ultimate isolation of sensitive agent operations [C reference].

### 3.4 VeriGuard: Runtime Behavioral Enforcement

Report B uniquely identifies VeriGuard as a promising pattern for runtime security [B20]. VeriGuard synthesizes behavioral policies and deploys runtime monitors to enforce them. Key capabilities:
- Synthesize formal behavioral contracts for mutating tool actions
- Deploy runtime monitors that check agent behavior against contracts
- Reduce attack success rates on prompt injection benchmarks

**Implementation path:** Experiment with VeriGuard-style contract synthesis and runtime monitors on a narrow set of mutating actions. Measure false positives/negatives on synthetic prompt injection attempts. This complements capability-based security (preventive) with behavioral monitoring (detective).

---

## 4. Stigmergic Coordination and Swarm Intelligence

### 4.1 Stigmergic Blackboards over JetStream

Report A makes the strongest case for stigmergy -- indirect coordination through environment modification, inspired by ant colony pheromone trails [A7]. Reports B and C provide complementary perspectives on swarm patterns.

**The problem with direct agent communication:** Direct LLM-to-LLM chat (like early AutoGen's GroupChat) is expensive, prone to runaway loops, and scales poorly. Each message consumes tokens from every participating agent's context window.

**Stigmergic alternative:** Agents do not message each other directly. Instead, they read and write to a shared "blackboard" or pressure field [A7][A24]. Foundation models are uniquely suited for this because their in-context learning acts as "pheromone memory," reinforcing successful strategies through positive feedback.

**Implementation in Mister Smith:** Use JetStream Key-Value stores as the stigmergic blackboard:
- Agents publish structured state updates ("pheromones") to specific KV keys
- JetStream's TTL feature naturally implements "pheromone evaporation," ensuring outdated information decays [A25]
- Multiple agents can observe the same KV bucket via watches, reacting to environmental changes
- Claimed token reduction: up to 80% compared to direct chat [A26]

**Counterpoint from Report C:** SwarmBench testing shows current LLMs struggle with long-range planning under decentralized constraints [C54]. This suggests stigmergy works best for reactive, environment-mediated tasks rather than complex multi-step planning.

**Synthesis assessment:** Stigmergic coordination is a high-value pattern for specific use cases -- massive agent swarms, environment monitoring, reactive coordination -- but should complement, not replace, deterministic orchestration for critical paths. JetStream KV with TTL-based decay is a natural implementation target that requires minimal new infrastructure.

### 4.2 Swarm Patterns and Dynamic Coordination

Report C provides additional swarm coordination patterns beyond stigmergy:

- **Dynamic speaker selection** (AutoGen's SelectorGroupChat): Routes each turn to the most relevant agent based on context [C53]. Implementable now as a supervisor/topology module.
- **Selective LLM invocation**: Combine cheap local agents (rule-based, heuristic) with expensive LLM calls, invoking the LLM only for "interesting" events. Rust-based swarm frameworks (Ebbiforge, Swarms-rs) demonstrate extremely low per-agent tick costs [B18][B19].
- **Multi-agent RL (AT-GRPO)**: Applied to train LLM agents jointly, yielding large gains on planning tasks (14-47% to ~99% accuracy) [C56]. This is promising but requires RL infrastructure and is a mid-term investment.
- **CodeCRDT**: Agents coordinate by observing a shared CRDT state instead of direct messaging, avoiding conflicts in concurrent work [C41]. This is a novel pattern that combines CRDT benefits with agent coordination.

**Implementation recommendation:** Implement swarm patterns as reusable supervisor/topology modules:
- A `SwarmSupervisor` managing many lightweight agents with selective LLM invocation
- A `SelectorSupervisor` implementing dynamic speaker selection
- NATS subjects for low-latency swarm messaging; JetStream for event recording and replay

---

## 5. Hardware-Aware Execution and Inference Optimization

### 5.1 Inference Engine Landscape

Report A provides the most detailed analysis of inference engines relevant to agent orchestration. Report B adds hardware-aware scheduling concepts. Report C provides a tempering perspective on prioritization.

| Inference Engine | Design Focus | Key Strengths | Best Use Case for Agents |
|:---|:---|:---|:---|
| **vLLM** | Continuous batching, PagedAttention [A8][A22] | Highest throughput at extreme concurrency; near-zero KV cache waste | High-concurrency multi-agent swarms |
| **TensorRT-LLM** | Deep hardware optimization (Hopper/Blackwell) [A22] | Best single-request throughput; lowest latency on H100/B200 | Latency-critical single-agent reasoning |
| **SGLang** | Structured generation, RadixAttention [A22] | Stable per-token latency; efficient state management | Workflows requiring precise JSON/structured outputs |

**PagedAttention (high confidence):** vLLM's PagedAttention manages KV cache as virtual memory pages, eliminating waste from pre-allocated contiguous blocks. This directly parallels Mister Smith's memory tier architecture -- both treat scarce resources (GPU memory, context window) as paged, managed resources [A8][B12].

**Disaggregated serving:** Report A recommends routing prefill tasks (compute-intensive) to high-performance GPUs (H100s) and decode tasks (memory-bandwidth-bound) to cost-effective GPUs (L40S) [A21]. Mister Smith's scheduler should expose this as a routing policy.

**Speculative decoding (EAGLE-3):** A lightweight autoregressive head proposes multiple tokens simultaneously, reducing latency for agent reasoning loops [A9]. This is transparent to the orchestration layer but should be supported in inference adapter configurations.

### 5.2 NUMA-Aware Actor Pinning

Report A makes the case for hardware-aware Rust actor scheduling:

- Using the `core_affinity` crate to pin Tokio worker threads to specific CPU cores prevents costly context switches and cache invalidations [A10]
- Pinning JetStream client actors and LLM routing actors to specific NUMA nodes can yield sub-millisecond orchestration latency [A23]
- As inference latency drops (via speculative decoding, better hardware), orchestration overhead becomes a visible percentage of end-to-end latency

**Report C counterpoint:** The major bottleneck is usually LLM inference time itself, not message passing or Rust overhead. Hardware tuning yields gains but is incremental for most deployments.

**Synthesis:** NUMA-aware pinning matters at scale (high agent counts, high message throughput) but is not a day-one priority. Design the scheduler to accept placement hints from the start so hardware-aware routing can be enabled without architectural changes.

### 5.3 Hardware-Aware Inference Adapters

Report B provides the most detailed implementation guidance for inference adapters:

1. **vLLM adapter**: Route LLM requests with PagedAttention-aware KV cache management. Use Rust frontends for request scheduling and batching [B12][B13].
2. **TensorRT-LLM adapter**: Leverage guidance/llgtrt (a Rust-based OpenAI-compatible inference frontend) as a reference implementation [B26][B27].
3. **Actor scheduler hints**: Extend the actor runtime to emit inference affinity hints and batching preferences, routing LLM-bound work to appropriate inference endpoints.
4. **Rust GPU kernels**: Rust-CUDA / rust-gpu are maturing [B28][B30], enabling Rust-native GPU kernel generation for specialized inference operations.

**Metrics targets:** End-to-end latency, throughput, GPU utilization. Compare against published vLLM/TensorRT-LLM benchmarks as baselines.

---

## 6. Protocol Interoperability: The 2026 Standards Landscape

### 6.1 The Protocol Wars

All three reports agree that protocol interoperability is critical and that proprietary protocols are a strategic mistake. Report A provides the most vivid framing ("TCP/IP vs. OSI wars"), while Reports B and C add implementation detail.

| Protocol | Backer / Governance | Primary Focus | Status (2026) | Key Technical Detail |
|:---|:---|:---|:---|:---|
| **MCP** | Anthropic / Linux Foundation (AAIF) [A37] | Agent-to-Tool and data source connections [A38] | 97M+ downloads; "USB-C for AI" [A2] | Standardizes tool/data interfaces |
| **A2A** | Google / Linux Foundation [A39][A40] | Agent-to-Agent collaboration and task delegation | 100+ enterprise supporters; JSON-RPC over HTTP/SSE [A2] | Agent Cards for discovery, long-running tasks, streaming |
| **WebMCP** | Google Chrome / Microsoft [A2] | Browser-native structured tool exposure | Early preview in Chrome 146 [A41] | `navigator.modelContext` API; reduces web agent token costs |

**Key insight from Report A:** Lack of interoperability is the leading cause of agent project failure ("agent sprawl") [A2]. This finding elevates protocol support from a "nice to have" to a strategic imperative.

### 6.2 Implementation Architecture

**"Speak-all-protocols" adapter layer:** Mister Smith should not invent a proprietary protocol. Instead, implement protocol adapters that translate between external standards and a canonical internal Rust/JetStream message schema.

```
External Protocols          Mister Smith Internal
+-----------+              +----------------------+
| MCP       |--adapter---->|                      |
+-----------+              | Canonical Rust Schema |
| A2A       |--adapter---->| over NATS/JetStream  |
+-----------+              |                      |
| WebMCP    |--adapter---->|                      |
+-----------+              +----------------------+
```

**Implementation details from Report B:**
- A2A HTTP adapter: Translate Agent Cards and JSON-RPC to NATS subjects and JetStream persistence
- MCP connector: Map model-context/tool standardized payloads to internal tool bus
- Agent gateway component: Mediate between external agents and internal actor model, using kgateway design patterns [B29]
- Auth translation: JWT (external) to internal capability tokens at adapter boundary

**Report C adds:** A2A messages can be encoded as special NATS subjects, allowing Mister Smith agents to interoperate with external A2A agents over the existing messaging fabric [C75]. MCP servers can run locally (e.g., Git or DB adapters) to let agents consume data uniformly [C73].

**Mister Smith already has MCP support** (Phase 4: `mister-smith-mcp` crate with client/server, tool registry, NATS bridge). The A2A adapter is the primary new work item.

---

## 7. Distributed and Federated Agent Execution

### 7.1 JetStream as the Distribution Fabric

All three reports agree that NATS/JetStream provides the canonical distribution fabric for Mister Smith. JetStream uses a NATS-optimized Raft algorithm for stream persistence [A27][B8][B10], providing immediate consistency guarantees.

**Multi-cluster architecture:** NATS supports clustering and multi-cluster superclusters via gateways and leaf nodes [B11][C45]. This enables geo-distributed agent graphs without custom networking code.

**Implementation patterns:**
- **Event sourcing**: JetStream streams for persistent agent state and decision logs
- **Cross-cluster messaging**: NATS subjects and gateways for discovery and routing across clusters
- **Local supervision**: Run supervisor trees per node; implement cross-node supervision via capability-protected control channels
- **Deterministic replay**: JetStream message replay for debugging, evaluation, and state migration

### 7.2 CRDTs for Conflict-Free Distributed Coordination

Reports A and C converge on CRDTs as a key technology for distributed agent state, approaching from different angles.

**Report A: Delta-CRDTs for geo-distributed blackboards.** Delta-CRDTs allow agents to update local state independently and merge changes without consensus bottlenecks. They are ideal for distributed stigmergic blackboards where eventual consistency is acceptable [A28][A29].

**Report C: CodeCRDT for multi-agent task coordination.** CodeCRDT applies CRDTs to multi-agent code generation, where agents coordinate by observing a shared CRDT state to avoid conflicts [C41]. This extends the CRDT concept from data synchronization to task coordination.

**Available Rust implementations:** Automerge (Rust-native CRDT library) and `crdts` crate provide production-ready CRDT implementations. These can be integrated with JetStream for persistence of CRDT state snapshots.

### 7.3 Federated Execution Patterns

Report B uniquely discusses serverless-style execution and agent migration:

- **Serverless agents**: Spawn agent "functions" on demand with state in external stores (JetStream, PostgreSQL)
- **State migration**: Supervisors checkpoint actor state to JetStream; agents can be migrated between nodes by restoring from checkpoint
- **Partition tolerance**: Design for network partitions by distinguishing consensus-critical state (Raft groups) from eventually-consistent state (CRDTs, pub/sub)

**Key engineering challenge:** Mapping actor supervision semantics across network partitions. When a supervisor and its child agent are on different nodes, failure detection, restart strategies, and state recovery become distributed systems problems.

---

## 8. Continuous Evaluation and Runtime Monitoring

### 8.1 The Evaluation Gap

All three reports identify evaluation as a critical unsolved problem in agent systems.

**Report A:** Static benchmarks like SWE-bench suffer from data contamination and fail to capture deployed agent dynamics [A43][A44]. Evaluation must shift from pre-deployment hurdle to continuous operational process.

**Report C:** AgentBench provides multi-environment tests (8 tasks testing reasoning, planning, tools) [C62]. GAIA shows even GPT-4+plugins at ~15% accuracy vs. 92% human [C65]. McKinsey/QuantumBlack proposes multi-layer evaluation: test LLM outputs, full agent trajectories, and overall system behavior [C67].

**Multi-agent specific metrics from Report C:**
- Handoffs per task
- Duplicate work detection
- Deadlock occurrence
- Invariant violations
- Context drift
- Memory consistency
- Tool-use correctness

### 8.2 Evaluation-as-a-Service (EaaS)

Report A proposes embedding evaluation directly into the framework [A45]:

1. **Golden traces**: Capture successful agent execution traces using JetStream message replay
2. **Shadow runs**: Execute alongside live agents, comparing reasoning trajectories against golden traces
3. **Regression detection**: Identify goal drift, failure modes (runaway loops), and performance degradation in real-time
4. **Binary pass/fail with explanations**: Reduce human variance in evaluation by providing clear pass/fail outcomes with explainable failure traces [B33][B34]

### 8.3 VeriGuard-Style Behavioral Contracts

Report B uniquely identifies the VeriGuard pattern [B20] for synthesizing and enforcing behavioral policies at runtime:

1. Synthesize formal behavioral contracts for agent actions (especially mutating operations)
2. Deploy runtime monitors that check agent behavior against contracts
3. Detect and block violations before they cause damage
4. Measure detection time for regressions and decrease in attack success rates

**Implementation:** Start with contracts for a narrow set of high-risk mutating actions. Deploy monitors and measure false positive/negative rates. Expand coverage incrementally.

### 8.4 Chaos Engineering for Agents

Report C uniquely suggests applying chaos engineering principles to agent systems: introduce faults (tool failures, network delays, prompt injection attempts) to probe failure modes. This aligns naturally with Mister Smith's OTP-style supervision -- the framework already handles failures; chaos engineering validates that it handles them correctly.

---

## 9. The Meta-Question: Timeless Abstractions and Framework Longevity

### 9.1 Will Models Internalize Orchestration?

All three reports address this question and reach the same conclusion: no, not in the foreseeable future.

**The debate:**
- **Bull case for model self-sufficiency**: Anthropic's Dario Amodei predicts models will replace software engineers within a year [A11].
- **Bear case**: DeepMind's Demis Hassabis notes current models struggle with long-term memory, planning, and physical world reasoning [A11]. Yann LeCun argues autoregressive LLMs are a "dead end" for AGI because they lack grounded world models and cannot predict consequences of actions [A46][A11].
- **Report C's nuanced take**: As computing evolved, abstractions like OS processes and TCP/IP remained fundamental. Similar persistent abstractions will be needed for agents regardless of model capability [C67].

**Consensus:** Because models cannot reliably plan or maintain state over long horizons, the orchestration framework remains the critical bridge. Even if future models gain planning abilities, they will still need governance, safety enforcement, resource management, and multi-agent coordination -- all framework concerns.

### 9.2 Timeless Abstractions to Adopt

All three reports converge on the same set of "timeless" abstractions that will survive AI paradigm shifts:

1. **Erlang/OTP-style supervision trees**: Embrace failure through organized, hierarchical fault management. If an agent hallucinates or a tool fails, the supervisor isolates the failure, restarts the process, and prevents cascading collapse [A12]. This pattern has powered telecom systems for decades.

2. **IPC message buses**: Decoupled, asynchronous communication between agents. NATS provides this today with JetStream for durability.

3. **Microkernel design**: Minimal trusted computing base with policy pushed to user-space. Keep the kernel small and verifiable.

4. **Capability-based authorization**: Unforgeable tokens coupling designation with authority. This pattern predates computing and will outlast any specific AI paradigm.

5. **Typed protocol contracts**: Whether via session types, interface definitions, or schema validation, explicit contracts between communicating parties are fundamental.

6. **Multi-tier memory primitives**: Working/episodic/semantic/archival tiers with clear semantics, regardless of the specific backing stores.

7. **Hardware-aware service endpoints**: Scheduler hints for inference routing, allowing the framework to adapt to evolving hardware landscapes.

**Report B's formulation:** Mister Smith should evolve into a modular runtime with a minimal kernel crate, typed protocol libraries, a Memory API, hardware-aware schedulers, and protocol adapters. Maintain the actor model and OTP-style supervision as first-class, but surface kernel-like primitives for resource and capability control.

---

## 10. Synthesis: Prioritized Roadmap

### 10.1 Prioritization Matrix

Combining all three reports' assessments with impact, viability, and effort analysis:

| Rank | Concept | Impact | Viability | Effort | Time Horizon |
|:---|:---|:---|:---|:---|:---|
| 1 | **JetStream-backed tiered memory (STM/MTM/LTM)** | Very High | Engineering-ready | Medium | 0-6 months |
| 2 | **Capability-based security + WASM sandboxing** | Very High | Engineering-ready | Medium | 0-6 months |
| 3 | **MCP/A2A protocol adapters** | High | Engineering-ready | Low-Medium | 0-6 months |
| 4 | **Continuous evaluation (EaaS) + behavioral contracts** | High | Engineering-ready | Low-Medium | 0-6 months |
| 5 | **Stigmergic blackboards (JetStream KV + TTL)** | High | Engineering-ready | Low | 3-9 months |
| 6 | **Compile-time session types for agent protocols** | High | Near-term experimental | Medium-High | 6-12 months |
| 7 | **Minimal agent kernel crate** | High | Experimental-implementable | Medium-High | 6-12 months |
| 8 | **NUMA-aware actor pinning** | Medium | Engineering-ready | Low | 6-12 months |
| 9 | **Hardware-aware inference adapters (vLLM/TensorRT)** | High | Engineering-ready | Medium | 6-12 months |
| 10 | **CRDT-based distributed coordination** | Medium-High | Experimental | Medium | 12-18 months |
| 11 | **Federated agent execution with live migration** | Medium | Experimental | High | 12-24 months |
| 12 | **Swarm/MARL patterns** | Medium | Experimental | High | 12-24 months |
| 13 | **Neural Paging controllers** | High (future) | Research-only | Very High | 18-36 months |
| 14 | **Full formal verification (kernel + protocols)** | Very High (future) | Research-only | Very High | 24-36+ months |

### 10.2 36-Month Roadmap

**Phase 1: Foundation (0-6 months)**
- Implement JetStream-backed tiered memory (STM/MTM) with Memory API crate
- Deploy Wasmtime/WASI tool sandboxing as default execution environment
- Build native MCP hosting (extend existing `mister-smith-mcp`) and A2A adapter
- Embed continuous evaluation telemetry with JetStream golden trace capture
- Implement capability token prototype (Macaroons) integrated with existing security crate

**Phase 2: Scale and Coordination (6-18 months)**
- Deploy stigmergic blackboards using JetStream KV with TTL-based decay
- Implement NUMA-aware actor pinning for sub-millisecond orchestration latency
- Build hardware-aware inference adapters (vLLM, TensorRT-LLM) with scheduler hints
- Introduce compile-time session types for critical agent protocols (tool invocation, delegation)
- Implement VeriGuard-style runtime behavioral monitors for high-risk operations
- Integrate CRDT-based coordination for distributed agent tasks
- Add vector DB connector for LTM semantic recall

**Phase 3: The Verified OS (18-36 months)**
- Formalize the agent kernel API with minimal TCB
- Extend capability-based security with ZCAP-LD for cross-framework delegation
- Apply Rust session types comprehensively across all agent interaction patterns
- Experiment with pluggable Neural Paging controllers (learned eviction)
- Investigate Verus-based formal verification for kernel invariants
- Explore federated agent execution with live migration across JetStream superclusters

### 10.3 Concepts to Avoid

All three reports converge on what NOT to build:

1. **Direct LLM-to-LLM chat as primary coordination**: Unscalable, expensive, and fragile. Use stigmergy or structured delegation instead.
2. **Proprietary communication protocols**: Building a custom agent standard in 2026 is a path to obsolescence. Adopt MCP + A2A.
3. **Monolithic agent runtimes**: Keep the kernel small. Push policy to user-space agents.
4. **Over-reliance on emergent behavior for correctness**: Swarm intelligence is valuable for specific problems but is not a general replacement for deterministic coordination.
5. **Framework-less agent assumption**: The idea that models will need no orchestration is speculative. History suggests frameworks adapt rather than disappear.

### 10.4 Minimal Reproducible Experiments (PoCs)

For concepts where evidence is immature, Reports B and C recommend specific experiments:

1. **Context paging with JetStream**: Build a supervisor that pages inactive actor context to JetStream KV and reloads on demand. Measure page-in latency distribution and task success rate under different eviction thresholds. Use MemGPT benchmarks as conceptual baseline [B14][B10].

2. **Session-typed tool invocation**: Implement a single tool-invocation protocol with MPST-generated client/server stubs in Rust. Verify compile-time error detection for protocol mismatches. Measure developer friction and runtime overhead [B3][B4].

3. **VeriGuard runtime enforcement**: Synthesize a behavioral contract for a mutating tool action. Deploy a monitor and measure false positives/negatives on synthetic prompt injection attempts [B20][B21][B22].

4. **CRDT-based multi-agent coordination**: Implement a shared Automerge document accessible to multiple agents. Measure conflict rates, merge latency, and coordination quality compared to message-passing baseline [C41].

5. **Stigmergic blackboard**: Deploy a JetStream KV-backed blackboard with TTL decay for a 100+ agent task. Compare token costs and coordination quality against direct messaging [A26].

### 10.5 Deployment Scenario Considerations

Report B provides operational guidance across deployment scenarios:

- **Single-node / small-cluster (latency-sensitive)**: Emphasize hardware-aware inference adapters, in-memory working memory, and low-latency NATS pub/sub. Minimize JetStream page-in latency via local caching.
- **Geo-distributed / high-availability**: Emphasize JetStream multi-cluster with Raft-backed persistence, robust supervisor checkpointing, and careful placement of consensus groups.
- **Large-scale multi-tenant / cloud**: Emphasize capability-based isolation, memory tiering to control storage costs, and hardware-aware routing to optimize inference costs. Use continuous evaluation for tenant-specific SLAs.

---

## References

### Report A Sources

[A1] *Memory OS of AI Agent*. https://arxiv.org/abs/2506.06326
[A2] *The 2026 AI Agent Protocol Wars Explained: MCP vs A2A vs WebMCP*. https://www.hungyichen.com/en/insights/ai-agent-protocol-wars.html
[A3] *Authorization Capabilities for Linked Data v0.3*. https://w3c-ccg.github.io/zcap-spec/
[A4] *Firecracker, gVisor, Containers, and WebAssembly - Comparing Isolation Technologies for AI Agents*. https://www.softwareseni.com/firecracker-gvisor-containers-and-webassembly-comparing-isolation-technologies-for-ai-agents/
[A5] *GitHub - faiface/par: session types for Rust*. https://github.com/faiface/par
[A6] *Affine Rust Programming with Multiparty Session Types*. https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2022.4
[A7] *Emergent Coordination in Multi-Agent Systems via Pressure Fields and Temporal Decay*. https://arxiv.org/html/2601.08129v2
[A8] *Efficient Memory Management for Large Language Model Serving with PagedAttention*. https://arxiv.org/abs/2309.06180
[A9] *An Introduction to Speculative Decoding for Reducing Latency in AI Inference*. https://developer.nvidia.com/blog/an-introduction-to-speculative-decoding-for-reducing-latency-in-ai-inference/
[A10] *How to configure CPU cores to be used in a Tokio application with core_affinity*. https://blog.veeso.dev/blog/en/how-to-configure-cpu-cores-to-be-used-on-a-tokio-with-core--affinity/
[A11] *AGI Debate 2026: Amodei, Hassabis, LeCun Disagree*. https://algeriatech.news/agi-debate-human-level-ai-llm-limits-2026/
[A12] *The Supervision Tree Patterns That Make Systems Bulletproof*. https://medium.com/@kanishks772/the-supervision-tree-patterns-that-make-systems-bulletproof-356199f178bb
[A13] *AIOS: LLM Agent Operating System*. https://arxiv.org/abs/2403.16971
[A14] *MemGPT: Towards LLMs as Operating Systems*. https://arxiv.org/abs/2310.08560
[A15] *Agent Memory: How to Build Agents that Learn and Remember (Letta)*. https://www.letta.com/blog/agent-memory
[A16] *Stateful AI Agents: A Deep Dive into Letta (MemGPT) Memory Models*. https://medium.com/@piyush.jhamb4u/stateful-ai-agents-a-deep-dive-into-letta-memgpt-memory-models-a2ffc01a7ea1
[A17] *Mem0 Architecture*. https://arxiv.org/abs/2504.19413
[A18] *Neural Paging: Learning Context Management Policies for Turing-Complete Agents*. https://arxiv.org/html/2603.02228v1
[A19] *Neural Paging (full paper)*. https://arxiv.org/abs/2603.02228
[A20] *Key/Value Store - NATS Docs*. https://docs.nats.io/nats-concepts/jetstream/key-value-store
[A21] *Why vLLM is the best choice for AI inference today*. https://developers.redhat.com/articles/2025/10/30/why-vllm-best-choice-ai-inference-today
[A22] *Comparing SGLANG, vLLM, and TensorRT-LLM with GPT-OSS-120B*. https://www.clarifai.com/blog/comparing-sglang-vllm-and-tensorrt-llm-with-gpt-oss-120b
[A23] *How to Process Streaming Data with Sub-Millisecond Latency in Rust*. https://oneuptime.com/blog/post/2026-01-25-streaming-data-sub-millisecond-latency-rust/view
[A24] *Emergent Coordination in Multi-Agent Systems via Pressure Fields and Temporal Decay (v3)*. https://arxiv.org/html/2601.08129v3
[A25] *A Pheromone-Based Coordination Mechanism Applied in Peer-to-Peer*. https://www.researchgate.net/publication/221234728_A_Pheromone-Based_Coordination_Mechanism_Applied_in_Peer-to-Peer
[A26] *Stigmergy Pattern for Multi-Agent LLM Systems: Fewer Tokens, Lower Costs*. https://dev.to/keepalifeus/stigmergy-pattern-for-multi-agent-llm-systems-80-token-reduction-2lc9
[A27] *NATS JetStream Docs*. https://docs.nats.io/jetstream/
[A28] *The CRDT Dictionary: A Field Guide to CRDTs*. https://iankduncan.com/engineering/2025-11-27-crdt-dictionary/
[A29] *Akka Distributed Data*. https://doc.akka.io/docs/akka/current/distributed-data.html
[A30] *Prompt injection: types, real-world CVEs, and enterprise impact*. https://it.vectra.ai/topics/prompt-injection
[A31] *seL4 Whitepaper*. https://sel4.systems/About/seL4-whitepaper.pdf
[A32] *CHERIoT Programmers' Guide*. https://cheriot.org/book/concepts.html
[A33] *Macaroons: Cookies with Contextual Caveats for Decentralized Authorization in the Cloud*. https://www.ndss-symposium.org/ndss2014/ndss-2014-programme/macaroons-cookies-contextual-caveats-decentralized-authorization-cloud/
[A34] *Security - Wasmtime*. https://docs.wasmtime.dev/security.html
[A35] *Sandboxing - Claude Code Docs*. https://code.claude.com/docs/en/sandboxing
[A36] *Implementing Multiparty Session Types in Rust*. https://pmc.ncbi.nlm.nih.gov/articles/PMC7282848/
[A37] *Donating the Model Context Protocol and establishing AAIF*. https://www.anthropic.com/news/donating-the-model-context-protocol-and-establishing-of-the-agentic-ai-foundation
[A38] *Introducing the Model Context Protocol*. https://www.anthropic.com/news/model-context-protocol
[A39] *Announcing the Agent2Agent Protocol (A2A)*. https://developers.googleblog.com/en/a2a-a-new-era-of-agent-interoperability/
[A40] *GitHub - a2aproject/A2A*. https://github.com/a2aproject/A2A
[A41] *WebMCP is available for early preview*. https://developer.chrome.com/blog/webmcp-epp
[A42] *A2A MCP Server*. https://lobehub.com/mcp/yw0nam-mcp-a2a-gateway
[A43] *Evaluation-Driven Development and Operations of LLM Agents*. https://arxiv.org/html/2411.13768v3
[A44] *Introducing SWE-bench Verified*. https://openai.com/index/introducing-swe-bench-verified/
[A45] *Systematic Evaluation of Raft using Evaluation-as-a-Service*. https://cse.buffalo.edu/tech-reports/2025-02.pdf
[A46] *Yann LeCun says LLMs are a dead end*. https://www.facebook.com/groups/aiartuniverse/posts/1473521931079703/

### Report B Sources

[B1] *seL4 Comprehensive Formal Verification*. https://sel4.systems/Research/pdfs/comprehensive-formal-verification-os-microkernel.pdf
[B2] *L4/seL4 Design Overview (Klein/Liedtke)*. https://read.seas.harvard.edu/~kohler/class/cs260r-17/klein10sel4.pdf
[B3] *Affine Rust Programming with Multiparty Session Types (full paper)*. http://mrg.doc.ic.ac.uk/publications/affine-rust-programming-with-multiparty-session-types/main.pdf
[B4] *AMPST - Affine Multiparty Session Types*. https://arxiv.org/pdf/2204.13464
[B5] *Verus (Rust verification extensions)*. https://users.ece.cmu.edu/~chanheec/verus-ghost.pdf
[B6] *Verdi (Coq framework for verified distributed systems)*. https://github.com/uwplse/verdi
[B7] *IronFleet (mechanized verification of distributed systems)*. https://www.andrew.cmu.edu/user/bparno/papers/ironfleet-cacm.pdf
[B8] *Raft specification*. https://raft.github.io/raft.pdf
[B9] *NATS official docs*. https://docs.nats.io/
[B10] *NATS JetStream concepts*. https://docs.nats.io/nats-concepts/jetstream
[B11] *NATS multi-cluster / supercluster*. https://www.synadia.com/glossary/multi-cluster
[B12] *vLLM paper (PagedAttention)*. https://arxiv.org/pdf/2309.06180
[B13] *vLLM repo*. https://github.com/vllm-project/vllm
[B14] *MemGPT paper*. https://readwise-assets.s3.amazonaws.com/media/wisereads/articles/memgpt-towards-llms-as-operati/MEMGPT.pdf
[B15] *Mem0 long-term memory architecture*. https://arxiv.org/html/2603.04740v1
[B16] *Mem0 blog: Long-term memory for AI agents*. https://mem0.ai/blog/long-term-memory-ai-agents
[B17] *AutoGen multi-agent conversation patterns*. https://microsoft.github.io/autogen/0.2/docs/Use-Cases/agent_chat/
[B18] *Ebbiforge swarm intelligence*. https://github.com/juyterman1000/ebbforge-swarm-intelligence
[B19] *Swarms-rs (Rust swarm crate)*. https://lib.rs/crates/swarms-rs
[B20] *VeriGuard: Behavioral policy synthesis and runtime monitoring*. https://arxiv.org/html/2510.05156v1
[B21] *OWASP LLM Prompt Injection Prevention Cheat Sheet*. https://cheatsheetseries.owasp.org/cheatsheets/LLM_Prompt_Injection_Prevention_Cheat_Sheet.html
[B22] *Securing Amazon Bedrock Agents Against Indirect Prompt Injections*. https://aws.amazon.com/blogs/machine-learning/securing-amazon-bedrock-agents-a-guide-to-safeguarding-against-indirect-prompt-injections/
[B23] *A2A documentation*. https://agent2agent.info/docs/
[B24] *A2A specification*. https://github.com/a2aproject/A2A/blob/main/docs/specification.md
[B25] *Building Smarter AI Agents with MCP*. https://medium.com/@srujanrana07/building-smarter-ai-agents-with-mcp-message-control-protocol-83f3cc708c59
[B26] *TensorRT-LLM architecture overview*. https://nvidia.github.io/TensorRT-LLM/architecture/overview.html
[B27] *guidance-ai/llgtrt (Rust inference frontend)*. https://github.com/guidance-ai/llgtrt
[B28] *Rust-CUDA update*. https://rust-gpu.github.io/blog/2025/08/11/rust-cuda-update/
[B29] *kgateway: Rust-powered AgentGateway*. https://www.solo.io/blog/why-traditional-gateways-failed-ai-workloads-and-how-kgateway-rust-powered-agentgateway-fixes-it
[B30] *Red Hat: Meet vLLM*. https://www.redhat.com/en/blog/meet-vllm-faster-more-efficient-llm-inference-and-serving
[B31] *Agent coordination patterns*. https://arxiv.org/html/2405.10299v1
[B32] *Baseten: High-throughput embedding inference*. https://www.baseten.co/blog/how-we-built-bei-high-throughput-embedding-inference/
[B33] *Databricks: Agent evaluation*. https://www.databricks.com/glossary/agent-evaluation
[B34] *Galileo AI: Agent evaluation*. https://galileo.ai/blog/ai-agent-evaluation
[B35] *Arthur AI: Best practices for building agents - continuous evaluations*. https://www.arthur.ai/blog/best-practices-for-building-agents-part-3-continuous-evaluations

### Report C Sources

[C5] Koubaa et al., "Agent-OS" blueprint (scheduling, context management, memory, access control).
[C7] KAOS: Multi-agent OS on openKylin.
[C9] AgentStore: Meta-agent app-store for heterogeneous agents.
[C12] Historical MAS work (blackboard OAA, FIPA ACL).
[C15] Collaborative Memory Framework: Two-tier memory with dynamic access graphs.
[C18] MemGPT: Virtual memory for LLMs.
[C20] MemGPT: FIFO buffer and recall DB implementation details.
[C28] MultiCrusty: Multiparty session types in Rust.
[C30] Typestate patterns in Rust for state machine encoding.
[C41] CodeCRDT: CRDT-based multi-agent code generation.
[C45] NATS multi-cluster via gateways.
[C50] Emergent Coordination (Riedl et al.): Prompt design steering group behavior.
[C53] AutoGen SelectorGroupChat: Dynamic speaker selection.
[C54] SwarmBench: LLM swarm evaluation benchmark.
[C56] Stronger-MAS / AT-GRPO: Multi-agent RL for planning tasks.
[C58] IBM episodic memory: Storing workflows for task suggestion.
[C62] AgentBench: Multi-environment agent evaluation.
[C65] GAIA: Complex assistant task benchmark (GPT-4+plugins ~15% vs. 92% human).
[C67] McKinsey/QuantumBlack: Multi-layer agent evaluation framework.
[C69] Capability-based security, approval gates, and audit patterns.
[C71] Claude Code sandboxing with bubblewrap (bwrap).
[C73] MCP: Open standard for agent-to-data connections.
[C75] A2A: Agent-to-Agent protocol specification.
[C82] SandboxEscapeBench: LLM sandbox escape benchmarking.

### Cross-Report Shared References (deduplicated)

The following sources were cited independently by multiple reports, lending them higher credibility:

- **seL4 microkernel design and verification**: [A31], [B1], [B2] -- cited by all three reports
- **MemGPT / Letta**: [A14], [B14], [C18] -- cited by all three reports
- **Mem0 memory architecture**: [A17], [B15], [B16] -- cited by Reports A and B
- **vLLM PagedAttention**: [A8], [B12], [B13] -- cited by all three reports
- **AMPST / Multiparty Session Types in Rust**: [A5], [A6], [B3], [B4], [C28] -- cited by all three reports
- **A2A Protocol**: [A39], [A40], [B23], [B24], [C75] -- cited by all three reports
- **MCP Protocol**: [A37], [A38], [B25], [C73] -- cited by all three reports
- **NATS JetStream**: [A20], [A27], [B9], [B10], [C45] -- cited by all three reports
- **AutoGen multi-agent patterns**: [B17], [C53] -- cited by Reports B and C
- **OWASP prompt injection**: [A30], [B21] -- cited by Reports A and B
- **Supervision tree patterns**: [A12] -- cited by Report A, implicit in all three
