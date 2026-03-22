---
version: R8
created: 2026-03-22
type: prompt
tier: 1
timeline: last 2 months (late January 2026 — present)
---

# Deep Research Prompt: Memory, Context Engineering & Neural Paging

## Context

Mister Smith is a first-class multi-agent orchestration operating system in Rust, built on NATS/JetStream and Erlang OTP-inspired supervision trees. It is model-agnostic and designed to define the standard that the agent framework market will converge toward.

The architecture already has a dual-store foundation: JetStream KV for fast distributed state (STM/MTM tier) and PostgreSQL for persistent long-term memory (LTM tier). Phase 6 (Persistence & State) shipped HybridStateManager with quarantined shared-state boundaries. What does not yet exist is the memory management layer — the abstractions for paging, consolidation, role-aware context routing, and learned eviction that sit between the agent runtime and the storage primitives. This research round targets the frontier of that layer.

## Frontier-First Mandate

Do not choose an approach because it is popular, familiar, or already normalized by existing agent frameworks. Benchmark them. Learn from them. Then exceed them. Pull from operating systems research, database buffer management, CPU cache hierarchies, CDN edge caching, and cognitive science when those fields offer stronger patterns.

Incremental imitation is failure. Favor well-reasoned designs that create real advantage.

## Research Objective

Survey everything published in the last ~2 months (late January 2026 to present) on agent memory architectures, context management, KV cache engineering, context compression, neural paging, learned eviction, multi-agent shared memory, and memory-aware scheduling. The goal is to discover what has changed since our last deep research round (early March 2026) and identify techniques that should influence Mister Smith's memory management architecture.

This is an open-ended research task. Go beyond the dimensions listed below if you discover promising leads outside them.

## What Has Already Been Researched (Baseline — Do Not Rediscover)

The following are established findings from 7 research rounds (2,000+ papers). Treat these as known. Only surface new work on these topics if it significantly contradicts, extends, or supersedes them.

**Tiered Memory (STM/MTM/LTM) — Mandatory Architecture.** MemOS achieves 49% F1 improvement on long-conversation benchmarks via 3-tier memory with segmented paging and MemCube abstraction (content + metadata as managed OS resource). Mem0 achieves 91% lower p95 latency and 90% token cost savings via dynamic extraction, consolidation, and graph-based relational structures. MemoryOS demonstrates FIFO-based STM-to-MTM promotion and segmented page organization for MTM-to-LTM transitions. MemGPT/Letta pioneered OS-style virtual context management with explicit interrupts and FIFO buffer for the "infinite context illusion." Consensus across all sources: expanding context windows is computationally ruinous; hierarchical memory is mandatory.

**Agentic Memory with Dynamic Linking.** A-MEM (Xu et al., 127 citations) implements Zettelkasten-inspired memory where storing a new memory triggers re-evaluation and dynamic linking of existing memories. Superior to baselines across six foundation models. The dynamic linking pattern — not static storage — is the key innovation.

**Persistent Quantized KV Cache.** Agent resume latency drops from 15.7s to 0.6s via 4-bit quantized disk persistence on Apple M4 Pro. Agents naturally interleave in multi-agent teams, so the 500ms reload hides behind another agent's decode step. No major framework currently implements this. JetStream KV naturally serves as this persistent cache layer.

**Learned Eviction and Token Importance.** TokenButler predicts token importance with <1.2% parameter overhead, outperforming heuristics by 8%+. KV-Distill achieves 99% context compression via student-teacher distillation. SAGE-KV shows LLMs implicitly know which tokens can be dropped after prefilling (one-time top-k, 4x memory efficiency). SmallKV identifies the saliency shift problem — token importance changes during decoding, making irreversible eviction dangerous.

**Permutation-Invariant Context Composition.** PICASO uses SSM state averaging to compose multiple independent context fragments into a single fixed-dimensional state. Zero online model processing time. Constant-time inference scaling regardless of episodic memory volume. Category theory provides functorial mappings ensuring mathematical consistency. Causal Context Meshes prevent context pollution across parallel task domains.

**Context Summarization and Compression.** SUPO co-optimizes tool-use summarization with task behavior via RL. ReSum enables indefinite agent exploration through periodic summarization. EMem decomposes sessions into Elementary Discourse Units (non-compressive, preserves information). CLAI cognitive load taxonomy (intrinsic/extraneous/germane) achieves 45% token reduction. ACON failure-driven guideline optimization produces 26-54% memory reduction.

**Hierarchical and Episodic Memory.** MIRIX 6-type taxonomy (Core, Episodic, Semantic, Procedural, Resource, Knowledge Vault) achieves 35% higher accuracy than RAG with 99.9% storage reduction, 85.4% SOTA on LOCOMO. H-MEM hierarchical index with positional encoding. Nemori self-organizing memory via Event Segmentation Theory and Free-energy Principle. Artificial Hippocampus Networks reduce FLOPs 40.5%, memory 74%.

**Critical Operational Finding.** LLMs lose state across tool calls (FuncBenchGen). Explicitly restating prior values at each step improves success from 62.5% to 81.3% for GPT-5. Explicit framework-level state management is mandatory, not optional.

**Multi-Agent Shared Memory.** LMCache transforms KV cache from per-engine resource into shared communication medium (15x throughput). PrefillShare: heterogeneous agents share a single frozen prefill module. HotPrefix dynamic hotness tracking with selective cache admission (2.25x latency reduction). SagaLLM Saga-pattern compensation transactions for memory rollback on agent failure.

## Research Dimensions

### 1. Persistent KV Cache and Quantization Techniques

- Have there been advances in quantized KV cache persistence beyond 4-bit (2-bit, mixed precision, adaptive quantization per layer)?
- Are there new approaches to incremental or delta-based KV cache serialization that avoid full cache writes on every checkpoint?
- Has anyone demonstrated persistent KV cache in a multi-agent setting where cache coherence across agents must be maintained?
- What is the current state of hardware-accelerated KV cache compression (GPU-direct storage, CXL memory, NVMe-oF)?
- Are there production reports or benchmarks of persistent KV cache at scale (>10 agents, >100K context tokens per agent)?

### 2. Neural Paging and Learned Eviction Strategies

- What new learned eviction policies have appeared beyond TokenButler, SAGE-KV, and OBCache?
- Has anyone addressed the saliency shift problem (SmallKV) with a dynamic re-scoring mechanism that doesn't require a secondary model?
- Are there new approaches that combine learned eviction with context compression (evict-then-compress vs. compress-then-evict)?
- Has the neural paging metaphor (differentiable page controller as neural MMU) been implemented in any production or near-production system?
- What advances exist in role-aware or task-stage-aware eviction (different eviction policies for different agent roles)?

### 3. Tiered Memory Implementations Beyond MemOS/Mem0/A-MEM

- Have new tiered memory frameworks appeared that address gaps in MemOS/Mem0 (e.g., better consolidation scheduling, stronger provenance, multi-tenant isolation)?
- Are there new memory taxonomies or memory type definitions that extend or challenge MIRIX's 6-type framework?
- Has anyone built tiered memory that operates across heterogeneous storage backends (local SSD + distributed KV + cloud object store) with unified access semantics?
- What advances exist in memory consolidation scheduling — when and how aggressively to promote STM to MTM to LTM under varying workloads?
- Are there new approaches to memory versioning or time-travel that go beyond append-only logs?

### 4. Context Compression and Summarization Advances

- What new context compression techniques have appeared beyond KV-Distill, KVTC, DMS, and ZSMerge?
- Have there been advances in lossless or near-lossless compression that preserves fine-grained factual detail (not just semantic gist)?
- Are there new failure-driven compression learning methods that extend ACON's approach (learning from compression-induced task failures)?
- Has anyone combined context compression with retrieval-augmented generation to create compress-then-retrieve pipelines?
- What is the frontier for compressing multi-turn conversation context specifically (not just document summarization)?

### 5. Attention and Memory Management at Infrastructure Level

- What changes to vLLM, SGLang, TensorRT-LLM, or new serving engines affect memory management architecture?
- Are there new disaggregated serving approaches where memory management is a first-class distributed systems concern?
- Advances in CXL memory pooling, GPU memory oversubscription, or tiered memory hardware that change what's possible for agent memory?
- New approaches to attention computation that reduce memory requirements (linear attention, sparse attention, sliding window variants) relevant to multi-agent workloads?
- Has anyone built memory management middleware that sits between the agent framework and the inference engine, managing KV cache as a shared resource?

### 6. Multi-Agent Shared Memory Architectures

- What new approaches exist for safe shared memory between agents that address the poisoning/injection risk ("Agent Smith" infectious jailbreak via shared memory)?
- Have there been advances in shared prefix caching beyond LMCache, PrefillShare, and FlashForge?
- Are there new consistency models for multi-agent shared memory (beyond eventual consistency — causal consistency, session guarantees, snapshot isolation)?
- Has anyone implemented memory access control at the token or embedding level (not just document level)?
- What new patterns exist for knowledge transfer between agents that don't share a model architecture?

### 7. Memory-Aware Agent Scheduling and Resource Management

- Are there new scheduling algorithms that co-optimize agent execution order with memory residency (keep agents with hot caches running)?
- Has anyone modeled the agent scheduling problem as a cache-aware job scheduling problem from OS/database research?
- What advances exist in predictive memory prefetching for agents (anticipating which context an agent will need before it's activated)?
- Are there production results showing the impact of memory-aware scheduling on multi-agent throughput and latency?
- Has anyone combined memory-aware scheduling with budget-aware routing (routing to the agent whose context is already cached)?

## Per-Dimension Output Structure

For each research dimension, provide:

1. **Current state of the art** — what exists today, with specific citations (authors, year, venue, DOI/URL if available)
2. **Key techniques** — the specific algorithms, architectures, or patterns discovered
3. **Applicability to Rust + NATS** — how well does each technique transfer to a Rust actor system with NATS/JetStream KV as the memory transport?
4. **Delta from baseline** — what is genuinely NEW versus what we already know?
5. **Implementation complexity** — rough assessment of effort and prerequisites
6. **Expected impact** — what improvement does this offer over the current Mister Smith dual-store architecture?

## Synthesis

After completing all dimensions, provide a synthesis that:
- Ranks the top 5 findings by strategic value for Mister Smith's memory management layer
- Identifies which current architectural assumptions are challenged (tiered memory as the right abstraction, JetStream KV as MTM, PostgreSQL as LTM, heuristic eviction as default)
- Recommends specific next actions (prototype, benchmark, adopt, monitor)
- Notes any dimension that yielded thin results (say so rather than padding)
- Assesses whether the "memory as managed OS resource" paradigm (MemOS/MemCube) is still the frontier or has been superseded

## Research Methodology

1. Search broadly across the last ~2 months (late January 2026 to present). Include arXiv preprints, conference proceedings, blog posts, GitHub releases, and industry reports.
2. Follow promising leads with targeted deep dives — do not stop at the first result
3. Look beyond agent frameworks into adjacent fields (OS buffer management, database caching, CDN edge caching, CPU cache replacement, cognitive science memory models) for transferable patterns
4. For each technique, assess whether it has been validated in production or is purely academic
5. Be skeptical of marketing claims — look for benchmarks, papers, and real-world results
6. If a dimension yields thin results, say so rather than padding with speculation
7. Cross-reference against the baseline above — only surface work that genuinely extends what we know
