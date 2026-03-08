# Memory, Context Management & Neural Paging -- Consolidated State of Knowledge

---
version: consolidated-v1
created: 2026-03-07
sources: R3 (synthesis), R4 (targeted + discovery), R5 (discovery), R7c (discovery), R7d (discovery)
topic: Memory, Context Management & Neural Paging
status: authoritative
---

## Executive Summary

Memory management is the single highest-impact architectural concern for multi-agent LLM systems. Across seven research rounds spanning 2,000+ papers, the evidence converges on one conclusion: treating agent memory as an OS-managed, tiered, lifecycle-aware resource -- not as a passive context buffer -- yields transformative gains in latency, cost, and quality.

The headline numbers:

- **MemOS**: 49% F1 improvement on long-conversation benchmarks via 3-tier STM/MTM/LTM with segmented paging (R3, R4)
- **Mem0**: 91% lower p95 latency, 90% token cost savings via dynamic extraction/consolidation with graph-based memory (R3, R4)
- **KV-Distill**: Up to 99% context length reduction while preserving downstream performance (R4)
- **Persistent quantized KV cache**: Agent resume latency drops from 15.7s to 0.6s via 4-bit quantized disk persistence on Apple M4 Pro (R7c)
- **PICASO**: Permutation-invariant context composition achieving constant-time inference scaling via SSM state averaging (R7d)
- **CAKE**: Maintains model performance with only 3.2% of KV cache, 10x decoding speedup at 128K tokens (R4)
- **A-MEM**: 127 citations, Zettelkasten-inspired dynamic memory linking outperforms baselines across six foundation models (R4)
- **MIRIX**: 35% higher accuracy than RAG with 99.9% storage reduction, 85.4% SOTA on LOCOMO (R4 discovery)

For Mister Smith specifically: the existing dual-store architecture (JetStream KV + PostgreSQL via HybridStateManager) maps directly onto the tiered memory paradigm. JetStream KV serves as the STM/MTM tier with TTL-based decay; PostgreSQL serves as LTM. The missing piece is the **memory management layer** -- the abstractions for paging, consolidation, role-aware routing, and learned eviction that sit between the agent runtime and the storage primitives.

---

## High-Confidence Findings

These findings are independently confirmed across multiple research rounds and multiple independent sources. Confidence is high.

**1. Tiered memory (STM/MTM/LTM) is mandatory, not optional.** Expanding context windows is computationally ruinous and economically wasteful. Every major research thread converges on hierarchical memory: working memory (in-actor state, microseconds), episodic memory (JetStream KV, low milliseconds, TTL-based), semantic memory (vector DB, milliseconds), and archival/procedural memory (PostgreSQL, higher latency). The evidence base spans MemOS (22 citations), MemoryOS (14 citations), Mem0 (72 citations), MemGPT/Letta (cited by all three R3 synthesis reports), and the Collaborative Memory Framework.

**2. Memory must be a managed OS resource, not an afterthought.** The MemOS MemCube abstraction -- a unit encapsulating content + metadata (provenance, versioning, access policy) -- is the academic formalization of this principle. Every memory fragment must carry: source agent, timestamp, contributing tools, access policy, and version. This maps to JetStream KV entries with metadata envelopes in Mister Smith.

**3. Context compression dramatically outperforms context expansion.** KV-Distill achieves 99% length reduction. KVTC achieves 20x compression via classical media coding (PCA + adaptive quantization + entropy coding). DMS achieves 8x compression via delayed eviction + implicit merging. These are not marginal gains -- they are order-of-magnitude improvements that make tiered memory practical.

**4. Role-aware context routing is essential for multi-agent systems.** RCR-Router demonstrates that dynamically selecting memory subsets per agent based on role and task stage -- within a strict token budget -- reduces token usage by 30% while maintaining quality. Broadcasting full context to all agents is wasteful and harmful. Each of Mister Smith's 9 agent roles should receive a filtered context view.

**5. Asynchronous background consolidation is the correct pattern.** Letta's "sleep-time compute" and Mem0's vector-similarity-based deduplication both demonstrate that memory consolidation should run as a supervised background process, not block the reasoning loop. In Mister Smith, these consolidation agents should run within the supervision tree as background actors with configurable scheduling.

**6. Persistent quantized KV cache eliminates re-prefill bottlenecks for multi-agent teams.** The "Agent Memory Below the Prompt" study (R7c) demonstrates that persisting 4-bit quantized KV cache to SSD reduces agent resume from ~15.7s to ~0.6s on Apple M4 Pro. Agents naturally interleave, so the 500ms reload latency hides behind another agent's decode step. No major framework currently implements this.

---

## Key Techniques & Architectures

### Tiered Memory (STM/MTM/LTM)

**Mechanism:** Three-level storage hierarchy inspired by OS memory management. Short-term memory (STM) lives in the actor's in-process state. Mid-term memory (MTM) uses fast distributed KV stores with TTL-based eviction. Long-term memory (LTM) uses persistent databases with vector similarity search.

**Evidence:**
- **MemOS** (Li et al., 2025, 22 citations): MemCube abstraction unifying content + metadata. Lifecycle-managed transitions between plaintext, activation, and parameter-level representations. [R4 Section 1]
- **MemoryOS** (Kang et al., 2025, 14 citations): Three-level hierarchy with FIFO for STM-to-MTM promotion and segmented page organization for MTM-to-LTM promotion. 49.11% F1 improvement, 46.18% BLEU-1 improvement on LoCoMo benchmark with GPT-4o-mini. [R3 Section 1.2, R4 Section 1]
- **Mem0** (Chhikara et al., 2025, 72 citations): Dynamic extraction, consolidation, graph-based relational structures. 91% lower p95 latency, 90%+ token cost savings vs. full-context. 26% improvement over OpenAI in LLM-as-Judge metric. [R3 Section 1.2, R4 Section 13]
- **MemGPT/Letta**: OS-style virtual context management with explicit interrupts, FIFO buffer, recall DB. "Infinite context illusion" within fixed windows. Cited independently by all three R3 synthesis reports. [R3 Section 1.2]
- **Collaborative Memory Framework** (Rezazadeh et al., 2025, 3 citations): Two-tier private/shared memory with immutable provenance per fragment. Bipartite graph for asymmetric, time-evolving access controls. [R4 Section 9]

**Mister Smith Integration Path:**
1. STM: In-actor Rust state (bounded by configurable context limit per role)
2. MTM: JetStream KV with TTL-based decay -- immediate consistency, monotonic reads already provided
3. LTM: PostgreSQL (already in Phase 6) extended with pgvector for semantic search
4. Memory API crate exposing `recall`, `put`, `consolidate`, `snapshot` async primitives
5. Background consolidation actors within the supervision tree

| Tier | Backing Store | Latency | Capacity | Eviction |
|:---|:---|:---|:---|:---|
| STM (Working) | In-actor Rust state | Microseconds | Context window | LRU / relevance scoring |
| MTM (Episodic) | JetStream KV | Low milliseconds | Large | TTL + access frequency |
| LTM (Semantic) | PostgreSQL + pgvector | Milliseconds | Unbounded | Consolidation + dedup |
| Archival | PostgreSQL / JetStream streams | Higher ms | Unbounded | Retention policy |

---

### A-MEM: Agentic Memory with Dynamic Linking

**Mechanism:** Zettelkasten-inspired memory system where each memory is a structured "note" with contextual descriptions, keywords, and tags. When new memories arrive, the system identifies connections to existing memories and updates historical memory attributes, enabling memory evolution rather than static storage.

**Evidence:**
- **A-MEM** (Xu et al., 2025, 127 citations): Superior to baselines across six foundation models. The high citation count indicates rapid community adoption. The dynamic linking pattern -- where storing a new memory triggers re-evaluation and linking of existing memories -- is the key innovation. [R4 Section 2]
- **Multiple Memory Systems (MMS)** (Zhang et al., 2025): Processes STM into multiple LTM fragments, constructing paired "retrieval memory units" and "contextual memory units" for efficient recall. [R4 Section 2]

**Mister Smith Integration Path:**
- Each memory entry stored as a structured record in JetStream KV (STM/MTM) or PostgreSQL (LTM) with: content, keywords/tags, linked memory IDs, creation timestamp, last access timestamp, source agent ID
- On new memory creation, a lightweight linking pass identifies and updates related existing memories (implementable as an async background task)
- The dual-unit pattern (lightweight retrieval index in JetStream KV, full context in PostgreSQL) directly leverages the existing dual-store architecture

---

### KV-Distill and Context Compression

**Mechanism:** Distills long KV caches into shorter representations using student-teacher training with KL divergence. Question-independent compression enables pre-computation. Related techniques apply classical media compression (PCA, quantization, entropy coding) or implicit merging via delayed eviction.

**Evidence:**
- **KV-Distill** (Chari et al., 2025, 7 citations): Up to 99% length reduction while preserving downstream performance. Question-independent property means compression can happen asynchronously -- critical for background consolidation. [R4 Section 6]
- **KVTC** (Staniszewski & Lancucki, 2025): Classical transform coding achieves 20x compression (40x+ for specific use cases), outperforming token eviction, quantization, and SVD methods. [R4 Section 4]
- **DMS (Dynamic Memory Sparsification)** (Lancucki et al., 2025, 8 citations): Delayed eviction + implicit merging achieves 8x compression with only 1K training steps. Enables "hyper-scaling" -- more reasoning tokens within the same compute budget. [R4 Section 4]
- **ZSMerge** (Liu et al., 2025, 2 citations): Residual merging preserving critical context via compensated attention scoring. 20:1 compression while sustaining quality. Zero-shot adaptation. [R4 Section 4]
- **ACON** (Kang et al., 2025, Microsoft, 1 citation): Failure-driven guideline optimization -- analyzes paired trajectories where compression causes failure to update compression rules. 26-54% memory reduction, distillable into smaller compressors at 95%+ accuracy. [R4 Section 6]

**Mister Smith Integration Path:**
1. Pre-storage compression: Before writing agent context to JetStream KV (MTM tier), apply transform coding to reduce storage footprint
2. Asynchronous consolidation: KV-Distill-style compression runs in background actors, producing compressed representations for LTM
3. Failure-driven learning: Track cases where compressed context caused task failure; use that signal to refine per-role compression policies (ACON pattern)
4. Design principle: **delay eviction, merge instead** -- rather than hard-deleting old context, compress into summary representations that preserve key information

---

### Token Importance & Neural Paging (Learned Eviction)

**Mechanism:** Lightweight learned predictors estimate per-token importance, enabling proactive context management that approaches Belady's optimal algorithm. Replaces heuristic LRU/LFU with query-aware, high-granularity importance scoring.

**Evidence:**
- **TokenButler** (Akhauri et al., 2025, 2 citations): Learns to predict token importance with <1.2% parameter overhead. Outperforms heuristic estimation by 8%+ in downstream accuracy. Near-oracle accuracy on co-referential retrieval. [R4 Section 5]
- **Neural Paging** (2026): Differentiable Page Controller acts as neural MMU, predicting future data requirements. Reduces asymptotic complexity from O(N^2) to O(N*K^2) for long-horizon reasoning. [R3 Section 1.3]
- **SDTP** (Tao et al., 2025, 2 citations): Saliency prediction module estimates per-token importance from hidden states. Prunes 65% of tokens while maintaining performance, 1.75x speedup. Combinable with KV cache compression. [R4 Section 5]
- **OBCache** (Gu et al., 2025): Optimal Brain Damage theory applied to cache eviction. Closed-form saliency scores for keys, values, and joint pairs. [R4 Section 5]
- **SmallKV** (Zhao et al., 2025, 1 citation): Identifies the **saliency shift problem** -- token importance changes during decoding, making irreversible eviction dangerous. Uses a smaller model to compensate. [R4 Section 3]
- **SAGE-KV** (Wang et al., 2025, 6 citations): LLMs implicitly "know" which tokens can be dropped after prefilling. One-time top-k selection achieves 4x memory efficiency. [R4 Section 3]

**Mister Smith Integration Path:**
- Design the Memory API with a pluggable `EvictionPolicy` trait, defaulting to heuristic (LRU/LFU + TTL) but swappable for learned predictors
- The saliency shift finding (SmallKV) means eviction decisions should be re-evaluable -- context paged to MTM should be promotable back to STM if importance increases
- Per-agent lightweight importance predictors (TokenButler pattern) can run alongside each agent actor, informing the Memory Manager about what to keep vs. page out
- Current maturity: R3 consensus is "monitor, not yet implement" for neural paging; R4 evidence is stronger. Recommendation: implement the pluggable interface now, build heuristic controllers first, add learned controllers as a second phase

---

### Persistent Quantized KV Cache (4-bit Disk Persistence)

**Mechanism:** Persist each agent's KV cache to disk in 4-bit quantized form. When an agent is suspended (swapped out in a multi-agent turn-taking system), its context is saved to SSD. On resume, the quantized cache is loaded directly, eliminating the expensive re-prefill step.

**Evidence:**
- **"Agent Memory Below the Prompt"** (Feb 2026): On Apple M4 Pro, reloading full agent context drops from ~15.7s (FP16 re-prefill) to ~0.6s (4-bit cache load from SSD). Agents naturally interleave in multi-agent teams, so the 500ms reload latency hides behind another agent's decode step. [R7c]
- No major framework currently implements persistent KV cache by default. [R7c]

**Mister Smith Integration Path:**
- This is a breakthrough for running large agent teams on limited hardware (edge, laptop, developer machines)
- Implement a "KV cache store" backed by JetStream or local disk for agent contexts
- When an agent actor is suspended by the supervision tree, serialize its quantized KV cache to the store
- On reactivation, load from store instead of re-prefilling -- transforms a 15.7s penalty into a 0.6s penalty
- The JetStream KV tier naturally serves as this persistent cache layer, with 4-bit quantized blobs as values
- Scheduling optimization: interleave agent execution so cache loads overlap with other agents' decode steps

---

### Permutation-Invariant Context Composition (PICASO, SSMs, Category Theory)

**Mechanism:** Instead of concatenating retrieved context fragments (which introduces arbitrary ordering and quadratic scaling), compose multiple independent context states into a single fixed-dimensional state using State Space Models. Enforces permutation invariance by averaging states across all possible orderings.

**Evidence:**
- **PICASO** (2025): Uses SSM state composition to merge context fragments mathematically. Requires zero online model processing time -- autoregressive generation begins directly from composed states. Achieves constant-time inference scaling regardless of the volume of episodic memory retrieved. [R7d]
- **Category theory foundations**: Functorial mappings between polynomial representations of individual agent models ensure mathematical consistency across trustless agent collaboration. Causal Context Meshes prevent "context pollution" across parallel task domains. [R7d]

**Mister Smith Integration Path:**
- This is the most theoretically rigorous approach to the "context composition" problem in multi-agent systems
- When multiple agents contribute context to a shared task (team coordination), PICASO-style composition avoids the O(n^2) cost of concatenation
- Practical implementation: use SSM-based context encoding for memory-focused actors that distribute pre-computed, dimensionally stable states via JetStream KV
- Reduces network bandwidth and token processing overhead during complex multi-agent reasoning chains
- Current maturity: research-grade, but the mathematical foundations are sound; candidate for 12-18 month implementation

---

### Context Summarization (SUPO, ReSum, Event-Centric)

**Mechanism:** Compress tool-use history and interaction records via learned summarization, converting verbose execution traces into compact reasoning states. Some approaches co-optimize summarization with task behavior; others use event-centric decomposition that preserves provenance.

**Evidence:**
- **SUPO** (Lu et al., 2025, 2 citations): Trains LLM agents to compress tool-use history via summarization while co-optimizing both tool-use behavior and summarization strategy end-to-end via RL. Scales beyond fixed context limits. [R4 discovery]
- **ReSum** (Wu et al., 2025, 9 citations): Enables indefinite agent exploration through periodic context summarization. Converts interaction histories into compact reasoning states. With only 1K training samples, achieves SOTA on BrowseComp. [R4 discovery]
- **EMem (Event-Centric Memory)** (Zhou, 2025): Decomposes sessions into Elementary Discourse Units -- self-contained event-like propositions with normalized entities. Non-compressive: preserves information while making it more accessible. Matches baselines with shorter QA contexts. [R4 Section 11, R4 discovery]

**Mister Smith Integration Path:**
- Implement a `ContextManager` trait with configurable summarization intervals
- Use event-centric decomposition for agent logs: each tool call and LLM response becomes an indexed event proposition (serves both as context compression and provenance)
- Store summarized context in JetStream KV for fast retrieval by resuming agents
- Key insight from SUPO: summarization strategy should be **co-optimized with task behavior**, not bolted on as an afterthought. The quality of summaries depends on what information the agent actually needs for its role.

---

### Hierarchical/Episodic Memory (MIRIX, H-MEM)

**Mechanism:** Multi-level memory organized by semantic abstraction or memory type, supporting different access patterns: fast procedural recall, temporal episodic retrieval, slow semantic search.

**Evidence:**
- **MIRIX** (Wang & Chen, 2025, 16 citations): Defines six memory types: Core, Episodic, Semantic, Procedural, Resource Memory, Knowledge Vault. Multi-agent framework dynamically coordinates updates and retrieval. 35% higher accuracy than RAG with 99.9% storage reduction. SOTA 85.4% on LOCOMO. [R4 discovery]
- **H-MEM** (Sun & Zeng, 2025, 5 citations): Multi-level memory with positional index encoding pointing to semantically related sub-memories. Index-based routing enables efficient retrieval without exhaustive similarity computation. Outperforms five baselines on LoCoMo. [R4 discovery]
- **Episodic Memory Position Paper** (Pink et al., 2025, 18 citations): Identifies five key properties of episodic memory for long-term agents: single-shot learning, temporal context, self-referentiality, constructive recall, emotional association. [R4 Section 11, R4 discovery]
- **Nemori** (Nan et al., 2025, 5 citations): Self-organizing memory via Event Segmentation Theory (two-step alignment) and Free-energy Principle (predict-calibrate). Learns from prediction gaps rather than pre-defined heuristics. Outperforms prior SOTA on LoCoMo and LongMemEval. [R4 Section 11]
- **Artificial Hippocampus Networks** (Fang et al., ByteDance, 2025): Sliding window KV cache (lossless STM) + learnable AHN module compressing out-of-window into fixed-size LTM. Reduces FLOPs by 40.5%, memory by 74.0%, while improving accuracy on 128K sequences. [R4 Section 12]

**Mister Smith Integration Path:**
- Define `MemoryStore` trait hierarchy: `EpisodicMemory`, `SemanticMemory`, `ProceduralMemory`
- Use JetStream KV for fast procedural memory (tool call patterns, recent actions)
- Use PostgreSQL + pgvector for semantic memory (embeddings + similarity search)
- MIRIX's six-type taxonomy maps onto Mister Smith's nine agent roles: different roles need different memory type mixes (e.g., the researcher role needs strong semantic memory; the executor needs strong procedural memory)
- Implement H-MEM's hierarchical index as a tree over JetStream KV keys for efficient retrieval routing
- The Nemori predict-calibrate principle: agents track their own prediction errors as signals for what to consolidate into LTM

---

### Joint Attention for Agent Coordination

**Mechanism:** Agents deliberately align their focus of attention on shared salient context elements (goals, crucial facts) rather than independently processing everything. Reduces combinatorial search in multi-agent tasks.

**Evidence:**
- **Lee et al.** (ICLR 2021): Enforcing joint attention among RL agents substantially boosts performance on multi-agent tasks by reducing combinatorial search. [R7c]
- **Distributed cognition analysis** (Fahey, 2026): Shifting from point-to-point messages to a shared world model "dramatically improves coherence and scalability." Multi-agent AI naturally embodies distributed cognition. [R7c]

**Mister Smith Integration Path:**
- Implement a shared "attention state" in JetStream KV that agents read before each reasoning step -- a compact representation of what the team is collectively focused on
- The existing JetStream KV watch mechanism naturally enables agents to observe shared attention state changes without explicit message passing (stigmergic pattern)
- Broadcast key facts to all agents at once, or use a central coordinator to align agent "views" -- tagging shared updates with semantic meaning
- This is complementary to role-aware context routing: joint attention selects WHAT to focus on; role-aware routing selects HOW MUCH of it each role receives

---

### KV Cache as Shared Communication Medium (LMCache, PrefillShare)

**Mechanism:** Instead of each agent maintaining an independent KV cache, share cache state across engines and agents. Eliminates redundant prefill computation when multiple agents process overlapping context.

**Evidence:**
- **LMCache** (Cheng et al., 2025, 6 citations): First open-source solution for extracting, storing, and sharing KV caches across engines. Transforms engines from independent token processors into a collection sharing KV cache as communication medium. Up to 15x throughput improvement. [R4 Section 8]
- **PrefillShare** (2026): Heterogeneous task-specific agent models share a single frozen prefill module. Prefill-only tuning aligns latent spaces of specialized decoders to consume a universal KV cache. Massive TTFT reduction and throughput improvement. [R7d]
- **HotPrefix** (Li et al., 2025): Dynamic hotness tracking + selective cache admission. High-hotness caches in fast memory, cold caches demoted. 2.25x latency reduction over vLLM. [R4 Section 8]
- **FlashForge** (Wang et al., 2025, 3 citations): Shared-prefix attention kernel combining memory access for shared prefixes. 1.9x speedup, 120.9x memory access reduction. [R4 Section 8]

**Mister Smith Integration Path:**
- This reframes context management from a per-agent concern to a **distributed systems concern** -- and Mister Smith already has the distributed systems infrastructure (NATS/JetStream)
- When multiple agents share system prompts or common task context, store the shared prefix KV cache once and let all agents reference it via JetStream
- Hotness-based promotion/demotion (HotPrefix pattern) maps directly to the tiered memory architecture: frequently accessed context stays in JetStream KV (fast tier), cold context demotes to PostgreSQL
- For GPU-backed inference: PrefillShare's disaggregated serving model (shared prefill, specialized decoders) is the architecture to target. NATS becomes the transport for KV cache pointers between prefill and decode nodes

---

### Additional Techniques

**Saga-Pattern Memory Transactions (SagaLLM):** Integrates the Saga transactional pattern with persistent memory and automated compensating transactions. Published in VLDB (11 citations). Maps directly to Mister Smith's OTP supervision strategies -- extend OneForOne/OneForAll with memory checkpoints for context recovery on agent failure. [R4 Section 9]

**CoThinker -- Cognitive Load Distribution:** Applies Cognitive Load Theory to multi-agent LLM systems. Distributes intrinsic cognitive load through agent specialization; manages transactional load via structured communication and collective working memory. First principled framework for context budget distribution. [R4 Section 9]

**Zep/Graphiti -- Temporal Knowledge Graphs:** Temporally-aware KG engine (26 citations). Outperforms MemGPT on Deep Memory Retrieval (94.8% vs 93.4%). 18.5% accuracy improvement and 90% latency reduction on LongMemEval. [R4 Section 10]

**Rethinking Memory Taxonomy (Du et al., 2025, 16 citations):** Defines six fundamental memory operations: Consolidation, Updating, Indexing, Forgetting, Retrieval, Compression. Each should have a corresponding implementation in Mister Smith's memory subsystem. [R4 Section 17]

**Adaptive Budget Allocation (Twilight, CAKE, PSA):** Context budgets should be dynamic and content-aware, not fixed per agent. Twilight achieves 15.4x attention acceleration by pruning 98% of redundant tokens via top-p sampling. CAKE maintains performance with 3.2% KV cache via layer-preference-aware allocation. [R4 Sections 3, 7]

**Multi-Turn Tool State Injection (FuncBenchGen):** LLMs propagate incorrect/stale argument values across tool call steps. Explicitly restating prior values at each step improves success from 62.5% to 81.3% for GPT-5. Mister Smith must explicitly inject relevant state into each tool call context. [R4 Section 16]

---

## Open Questions & Gaps

**1. Neural paging controller training for multi-agent systems.** TokenButler and Neural Paging demonstrate learned eviction for single-model inference. How to train these controllers for multi-agent systems where context importance is role-dependent and dynamically shifting is unresolved. The saliency shift problem (SmallKV) makes this harder.

**2. Compression-quality tradeoff calibration per agent role.** ACON's failure-driven approach is promising but requires per-role calibration data. How much compression each of Mister Smith's 9 roles can tolerate without task degradation is an empirical question requiring benchmarking.

**3. Shared KV cache security.** When agents share context via a common KV cache (LMCache pattern), a compromised agent could poison shared memory. The "Agent Smith" infectious jailbreak vector (R7d) demonstrates that shared memory banks are exploitable attack surfaces. Mandatory semantic firewalls and quarantine actors are needed but their performance cost is unknown.

**4. PICASO practical integration.** Permutation-invariant context composition via SSMs is mathematically rigorous but has not been demonstrated in a production multi-agent orchestration system. The gap between the theoretical foundation and a practical Rust implementation needs bridging.

**5. Consolidation scheduling under load.** Background consolidation actors compete for compute with active reasoning agents. Optimal scheduling policies for consolidation (frequency, priority, resource allocation) under varying workloads are not well-studied.

**6. Cross-session memory consistency.** When agents resume from persistent quantized KV cache across sessions, ensuring consistency between the cached state and any changes to shared team memory that occurred during suspension is unaddressed.

**7. Memory provenance at scale.** The Collaborative Memory Framework requires immutable provenance per fragment. At high agent counts and interaction rates, the provenance metadata overhead and query cost may become significant. No evidence exists on scaling characteristics.

---

## Implementation Priority for Mister Smith

Ordered by impact-to-effort ratio, mapped to Mister Smith's existing infrastructure.

### Tier 1: Implement Now (0-6 months, high impact, engineering-ready)

| Priority | Feature | Backing Evidence | Effort | Key Metric Target |
|:---|:---|:---|:---|:---|
| **P1** | Memory API crate with tiered STM/MTM/LTM | MemOS, MemoryOS, Mem0 | Medium | <5ms MTM retrieval, 90% token cost reduction |
| **P2** | JetStream KV-backed MTM with TTL decay | MemOS (49% F1), Mem0 (91% p95) | Low | Configurable TTL per agent role |
| **P3** | Role-aware context routing | RCR-Router (30% token reduction) | Low-Medium | Per-role token budgets, 30% savings |
| **P4** | Explicit state injection for tool calls | FuncBenchGen (62.5%->81.3%) | Low | Eliminate stale argument propagation |
| **P5** | Background consolidation actors | Letta sleep-time, Mem0 dedup | Low-Medium | Background process, non-blocking |

### Tier 2: Next Phase (6-12 months, high impact, moderate complexity)

| Priority | Feature | Backing Evidence | Effort | Key Metric Target |
|:---|:---|:---|:---|:---|
| **P6** | Persistent quantized KV cache | Agent Memory Below Prompt (15.7s->0.6s) | Medium | <1s agent resume |
| **P7** | Context compression pipeline (pre-storage) | KV-Distill (99%), KVTC (20x) | Medium | 10x+ storage reduction |
| **P8** | A-MEM-style dynamic memory linking | A-MEM (127 citations) | Medium | Cross-reference density metric |
| **P9** | Saga-pattern memory checkpoints | SagaLLM (VLDB, 11 citations) | Medium | Memory recovery on agent failure |
| **P10** | Shared prefix KV cache across agents | LMCache (15x), FlashForge (120.9x) | Medium-High | Eliminate redundant prefill |

### Tier 3: Research Integration (12-18 months, experimental)

| Priority | Feature | Backing Evidence | Effort | Key Metric Target |
|:---|:---|:---|:---|:---|
| **P11** | Pluggable learned eviction policy | TokenButler (<1.2% overhead) | High | 8%+ accuracy over heuristic |
| **P12** | PICASO-style context composition | PICASO, category theory | High | Constant-time scaling |
| **P13** | Temporal knowledge graph for LTM | Zep/Graphiti (94.8% retrieval) | Medium-High | Cross-session synthesis |
| **P14** | Failure-driven compression learning | ACON (Microsoft) | Medium | Self-improving compression |
| **P15** | Episodic memory with predict-calibrate | Nemori, Pink et al. | High | Self-improving memory management |

### Tier 4: Long-term Research (18+ months)

| Priority | Feature | Backing Evidence | Effort |
|:---|:---|:---|:---|
| **P16** | Full neural paging controller | Neural Paging (O(N*K^2)) | Very High |
| **P17** | Joint attention coordination protocol | Lee et al. (ICLR) | High |
| **P18** | PrefillShare disaggregated serving integration | PrefillShare (2026) | Very High |

---

## Design Principles (Research-Derived)

These principles emerge from the convergence of findings across all research rounds:

1. **Adaptive Budgets, Not Fixed Limits.** (Twilight, PSA, CAKE) Context budgets should be dynamic and content-aware. Different agent roles, task stages, and content densities warrant different allocations.

2. **Delay Eviction, Merge Instead.** (DMS, ZSMerge) Rather than hard-deleting old context, compress and merge into summary representations that preserve key information. Hard eviction is irreversible and the saliency shift problem makes it dangerous.

3. **Provenance Is Non-Negotiable.** (Collaborative Memory, MemOS) Every memory fragment must carry metadata: source agent, timestamp, contributing tools, access policy. Without provenance, debugging non-deterministic multi-agent workflows is impossible.

4. **Learn From Failures.** (ACON) Track cases where context compression or eviction caused task degradation. Use that signal to improve compression policies over time. This is the path from heuristic to learned memory management.

5. **Role-Aware Context Routing.** (RCR-Router, CoThinker) Different agent roles need different context subsets. Route context based on role + task stage, not broadcast everything.

6. **Explicit State Injection for Tool Calls.** (FuncBenchGen) Never rely on the LLM's implicit state tracking across tool calls. Always explicitly inject relevant state.

7. **Temporal Awareness in Memory.** (Zep/Graphiti, LiCoMemory) Memory systems must track time -- recency, temporal ordering, and historical relationships matter for retrieval quality.

8. **Episodic Memory for Learning.** (Pink et al.) Agents need instance-specific memory of past task executions, not just generalized knowledge, to improve over time.

---

## Sources

### Primary Sources (read in full for this synthesis)

| File | Round | Type | Key Contribution |
|:---|:---|:---|:---|
| `synthesis/frontier-agent-architecture-R3.md` | R3 | Triple synthesis | Tiered memory architecture, Neural Paging concept, MemOS/Mem0 evidence, 36-month roadmap |
| `research/targeted-neural-paging-context-management-R4.md` | R4 | Academic search (61 papers, 20 searches) | 19-section deep dive: OS-inspired memory, KV cache eviction/compression, token importance, context distillation, multi-agent shared memory, cognitive memory, hippocampal hybrids, lifelong learning |
| `research/discovery-sweep-R4.md` | R4 | Discovery (96 papers, 32 searches) | Themes 5 (H-MEM, MIRIX, episodic memory) and 12 (SUPO, ReSum, event-centric context summarization) |
| `research/discovery-sweep-R5.md` | R5 | Discovery (974 screened, 50 included) | Cognitive synergy protocols (OSC), profile-aware supervision, knowledge-aware routing -- contextual findings |
| `research/discovery-sweep-R7c.md` | R7 | Discovery (user-added) | Persistent quantized KV cache (15.7s->0.6s), joint attention, distributed cognition, shared blackboard model |
| `research/discovery-sweep-R7d.md` | R7 | Discovery (user-added) | PICASO permutation-invariant context composition, PrefillShare shared KV cache, category theory for context meshes, disaggregated serving |

### Key Papers (by citation count, memory-relevant only)

| Paper | Citations | Year | Key Finding |
|:---|:---|:---|:---|
| Agentic RAG Survey (Singh et al.) | 128 | 2025 | Comprehensive RAG + agent integration patterns |
| A-MEM (Xu et al.) | 127 | 2025 | Zettelkasten-inspired dynamic memory linking |
| Mem0 (Chhikara et al.) | 72 | 2025 | 91% p95 reduction, 90% token savings, graph memory |
| Lifelong Learning Roadmap (Zheng et al.) | 33 | 2025 | Three-pillar lifelong agent learning framework |
| Zep/Graphiti (Rasmussen et al.) | 26 | 2025 | Temporal knowledge graph, 94.8% retrieval accuracy |
| MemOS (Li et al.) | 22 | 2025 | MemCube abstraction, OS-managed LLM memory |
| Episodic Memory (Pink et al.) | 18 | 2025 | Five properties of episodic memory for agents |
| CAKE (Qin et al.) | 17 | 2025 | 3.2% KV cache, 10x speedup at 128K tokens |
| Rethinking Memory (Du et al.) | 16 | 2025 | Six fundamental memory operations taxonomy |
| MIRIX (Wang & Chen) | 16 | 2025 | Six memory types, 85.4% LOCOMO SOTA |
| MemoryOS (Kang et al.) | 14 | 2025 | 49% F1 improvement, segmented paging |
| SagaLLM (Chang & Geng) | 11 | 2025 | Saga transactions for multi-agent memory |
| KV-Distill (Chari et al.) | 7 | 2025 | 99% context compression |
| DMS (Lancucki et al.) | 8 | 2025 | 8x compression via delayed eviction + merging |
| LMCache (Cheng et al.) | 6 | 2025 | KV cache as shared communication medium, 15x throughput |
