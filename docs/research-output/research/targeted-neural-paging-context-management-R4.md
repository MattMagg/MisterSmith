---
version: R4
created: 2026-03-07
updated: 2026-03-07
sources: Consensus (61 papers, 20 searches)
round: 4 (Academic Search)
---

# Neural Paging & Learned Context Management for LLM Agents

## Research Digest -- Consensus Academic Search

**Date:** 2026-03-07
**Scope:** Peer-reviewed papers from 2025 onward
**Relevance:** Mister Smith multi-agent orchestration framework (Rust + NATS + OTP-style supervision)

---

## Table of Contents

1. [OS-Inspired Memory Architectures for Agents](#1-os-inspired-memory-architectures-for-agents)
2. [Tiered & Hierarchical Memory Systems](#2-tiered--hierarchical-memory-systems)
3. [KV Cache Eviction: Learned & Adaptive Policies](#3-kv-cache-eviction-learned--adaptive-policies)
4. [KV Cache Compression: Quantization, Merging, & Transform Coding](#4-kv-cache-compression-quantization-merging--transform-coding)
5. [Token Importance Scoring & Predictive Pruning](#5-token-importance-scoring--predictive-pruning)
6. [Context Distillation & Prompt Compression](#6-context-distillation--prompt-compression)
7. [Sparse Attention & Adaptive Budget Allocation](#7-sparse-attention--adaptive-budget-allocation)
8. [KV Cache Offloading, Paging, & Distributed Serving](#8-kv-cache-offloading-paging--distributed-serving)
9. [Multi-Agent Shared Memory & Context Routing](#9-multi-agent-shared-memory--context-routing)
10. [Knowledge Graph & Temporal Memory for Agents](#10-knowledge-graph--temporal-memory-for-agents)
11. [Cognitive-Science-Inspired Agent Memory](#11-cognitive-science-inspired-agent-memory)
12. [Hippocampal & Recurrent Memory Hybrids](#12-hippocampal--recurrent-memory-hybrids)
13. [Lifelong Learning & Continual Adaptation](#13-lifelong-learning--continual-adaptation)
14. [Streaming & Infinite-Context Inference](#14-streaming--infinite-context-inference)
15. [Context Window Extension via RoPE Scaling](#15-context-window-extension-via-rope-scaling)
16. [Multi-Turn Tool Use & Stateful Agent Interaction](#16-multi-turn-tool-use--stateful-agent-interaction)
17. [Surveys & Taxonomies](#17-surveys--taxonomies)
18. [Emerging Directions](#18-emerging-directions)
19. [Synthesis: Implications for Mister Smith](#19-synthesis-implications-for-mister-smith)

---

## 1. OS-Inspired Memory Architectures for Agents

### MemOS: A Memory OS for AI System
- **Authors:** Li, Song, Xi, Wang, Tang, Niu, Chen, et al. (50+ authors)
- **Year:** 2025 | **Citations:** 22
- **Journal:** ArXiv (abs/2507.03724)
- **Key Finding:** Proposes treating LLM memory as an OS-managed resource. Introduces the **MemCube** abstraction -- a unit that encapsulates memory content plus metadata (provenance, versioning). MemCubes can be composed, migrated, and fused over time, enabling flexible transitions between plaintext, activation-based, and parameter-level memory representations. Unifies scheduling and lifecycle management of heterogeneous memory types.
- **Mister Smith Relevance:** **CRITICAL.** This is the closest academic analog to the "neural paging" concept. Mister Smith's agent memory could adopt MemCube-like abstractions backed by JetStream KV for ephemeral state and PostgreSQL for persistent memory, with the framework managing lifecycle transitions between tiers. The MemOS architecture maps directly onto Mister Smith's existing dual-store (HybridStateManager) pattern.

### Memory OS of AI Agent (MemoryOS)
- **Authors:** Kang, Ji, Zhao, Bai
- **Year:** 2025 | **Citations:** 14
- **Journal:** ArXiv (abs/2506.06326)
- **Key Finding:** Three-level storage hierarchy (short-term, mid-term, long-term personal memory) inspired by OS memory management. Short-to-mid updates use FIFO; mid-to-long uses **segmented page organization**. Achieves 49.11% F1 improvement and 46.18% BLEU-1 improvement over baselines on GPT-4o-mini for the LoCoMo benchmark.
- **Mister Smith Relevance:** **HIGH.** The three-tier architecture with explicit paging between tiers maps onto Mister Smith's existing STM/MTM/LTM patterns. The "segmented page organization" strategy for promotion is directly implementable using JetStream KV (STM/MTM) and PostgreSQL (LTM).

---

## 2. Tiered & Hierarchical Memory Systems

### A-MEM: Agentic Memory for LLM Agents
- **Authors:** Xu, Liang, Mei, Gao, Tan, Zhang
- **Year:** 2025 | **Citations:** 127
- **Journal:** ArXiv (abs/2502.12110)
- **Key Finding:** Zettelkasten-inspired memory system where each memory is a structured "note" with contextual descriptions, keywords, and tags. When new memories arrive, the system identifies connections to existing memories and updates historical memory attributes -- enabling **memory evolution** rather than static storage. Superior to baselines across six foundation models.
- **Mister Smith Relevance:** **HIGH.** The dynamic linking and memory evolution pattern could inform how Mister Smith agents maintain and update their knowledge networks. The Zettelkasten indexing strategy is implementable over the existing persistence layer.

### Multiple Memory Systems (MMS)
- **Authors:** Zhang, Wang, Ma, Zhao, Yu
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2508.15294)
- **Key Finding:** Cognitive-psychology-inspired system that processes short-term memory into multiple long-term fragments, then constructs paired "retrieval memory units" and "contextual memory units" for efficient recall. One-to-one correspondence between retrieval and context units enables targeted knowledge access.
- **Mister Smith Relevance:** **MEDIUM.** The dual-unit pattern (retrieval index + full context) is a useful design pattern for Mister Smith's agent memory, where lightweight retrieval indices could be stored in JetStream KV while full context resides in PostgreSQL.

### S-AI-GPT Memory Architecture
- **Authors:** Slaoui
- **Year:** 2025 | **Citations:** 0
- **Journal:** SSRN Electronic Journal
- **Key Finding:** Biologically-inspired architecture with three components: Dynamic Contextual Memory (short-term), GPTMemoryAgent (long-term), and GPT-MemoryGland (affective trace encoding). Orchestrated by a "hormonal engine" that enables adaptive forgetting, emotional persistence, and context-aware prioritization.
- **Mister Smith Relevance:** **LOW-MEDIUM.** The hormonal modulation concept is novel but speculative. The adaptive forgetting mechanism is more practically relevant -- implementing priority-based memory decay in Mister Smith's agent state management.

---

## 3. KV Cache Eviction: Learned & Adaptive Policies

### CAKE: Cascading and Adaptive KV Cache Eviction with Layer Preferences
- **Authors:** Qin, Cao, Lin, Hu, Fan, Cheng, Lin, Li
- **Year:** 2025 | **Citations:** 17
- **Journal:** ArXiv (abs/2503.12491)
- **Key Finding:** Frames KV cache eviction as a "cake-slicing problem" where different layers get different cache budgets based on their attention dynamics in spatial and temporal dimensions. Maintains model performance with only **3.2% of the KV cache** and achieves 10x speedup in decoding latency at 128K tokens.
- **Mister Smith Relevance:** **HIGH.** The layer-preference-aware budget allocation principle can inform how Mister Smith distributes context budget across agents of different roles. Agents performing different functions (e.g., researcher vs. code writer) may need different "cache budget" allocations.

### AhaKV: Adaptive Holistic Attention-Driven KV Cache Eviction
- **Authors:** Gu, Jiang, Jin, Guo, Zhang, Xu
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2506.03762)
- **Key Finding:** Identifies that accumulated attention scores are **positionally biased** (decreasing with token position), causing retained tokens to cluster at initial positions. Addresses this by adaptively tuning softmax scale based on information entropy and incorporating value vector information for importance scoring.
- **Mister Smith Relevance:** **MEDIUM.** The positional bias insight is important for any context management system -- Mister Smith should not over-weight initial context simply because it was seen first.

### SAGE-KV: Self-Attention Guided Eviction
- **Authors:** Wang, Upasani, Wu, Gandhi, Li, Hu, Li, Thakker
- **Year:** 2025 | **Citations:** 6
- **Journal:** ArXiv (abs/2503.08879)
- **Key Finding:** LLMs implicitly "know" which tokens can be dropped after prefilling. One-time top-k selection at both token and head levels achieves 4x memory efficiency over StreamLLM with improved accuracy.
- **Mister Smith Relevance:** **MEDIUM.** The insight that models self-identify expendable context could inform agent-level context pruning where the LLM itself participates in deciding what to page out.

### SmallKV: Small Model Assisted KV Cache Compensation
- **Authors:** Zhao, Peng, Nguyen, Li, Wang, Zhao, Fu
- **Year:** 2025 | **Citations:** 1
- **Journal:** ArXiv (abs/2508.02751)
- **Key Finding:** Identifies two critical problems: (1) **saliency shift** -- token importance changes during decoding, making irreversible eviction dangerous; (2) **marginal information over-compression** -- collectively important tokens treated as unimportant individually. Uses a smaller model to compensate for eviction errors in the larger model.
- **Mister Smith Relevance:** **HIGH.** The saliency shift problem is critical for multi-agent systems where context importance changes as tasks evolve. Mister Smith could use lightweight proxy models to track evolving context importance.

---

## 4. KV Cache Compression: Quantization, Merging, & Transform Coding

### KVTC: KV Cache Transform Coding
- **Authors:** Staniszewski, Lancucki
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2511.01815)
- **Key Finding:** Classical media compression techniques (PCA decorrelation + adaptive quantization + entropy coding) applied to KV caches. Achieves **20x compression** with maintained accuracy and **40x+** for specific use cases. Outperforms token eviction, quantization, and SVD methods.
- **Mister Smith Relevance:** **HIGH.** Directly applicable to Mister Smith's context storage. Agent context could be compressed using transform coding before offloading to JetStream KV or PostgreSQL, dramatically reducing storage costs.

### KVTuner: Layer-wise Mixed Precision KV Cache Quantization
- **Authors:** Li, Xing, Li, Qu, Zhen, Liu, Yao, Pan, Yuan
- **Year:** 2025 | **Citations:** 6
- **Journal:** ArXiv (abs/2502.04420)
- **Key Finding:** Keys are generally more important than values for quantization error reduction. Achieves nearly lossless **3.25-bit mixed precision** KV cache quantization with 21.25% throughput improvement.
- **Mister Smith Relevance:** **MEDIUM.** Informs how Mister Smith could differentially compress keys vs. values when storing agent context.

### ZSMerge: Zero-Shot KV Cache Compression
- **Authors:** Liu, Wang, Liu, Tang
- **Year:** 2025 | **Citations:** 2
- **Key Finding:** Residual merging mechanism preserves critical context through compensated attention scoring. Achieves **20:1 compression** while sustaining generation quality. Zero-shot adaptation compatible with diverse architectures.
- **Mister Smith Relevance:** **MEDIUM.** The residual merging concept -- preserving a "delta" when merging similar context entries -- is applicable to Mister Smith's mid-term memory consolidation.

### DMS: Dynamic Memory Sparsification (Inference-Time Hyper-Scaling)
- **Authors:** Lancucki, Staniszewski, Nawrot, Ponti
- **Year:** 2025 | **Citations:** 8
- **Journal:** ArXiv (abs/2506.05345)
- **Key Finding:** Instead of prematurely discarding tokens, DMS delays eviction and **implicitly merges representations**. Achieves 8x compression with only 1K training steps while outperforming training-free methods. Enables "hyper-scaling" -- generating more reasoning tokens within the same compute budget.
- **Mister Smith Relevance:** **HIGH.** The delayed eviction + implicit merging pattern is a superior approach to context management. Rather than hard-evicting old context, Mister Smith could maintain compressed representations that preserve key information.

### QuickSilver: Modular Token-Level Inference Optimization
- **Authors:** Khanna, Guru, Sridhar, Ahmed, et al.
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2506.22396)
- **Key Finding:** Four synergistic mechanisms: (1) Dynamic Token Halting for converged representations, (2) KV Cache Skipping for selective memory writes, (3) Contextual Token Fusion collapsing redundant tokens, (4) Adaptive quantization. 39.6% FLOP reduction on frozen models.
- **Mister Smith Relevance:** **MEDIUM.** The "contextual token fusion" concept -- collapsing redundant tokens into shared representations -- is directly applicable to Mister Smith's context deduplication across agents sharing similar context.

---

## 5. Token Importance Scoring & Predictive Pruning

### TokenButler: Token Importance is Predictable
- **Authors:** Akhauri, Abouelhamayed, Gao, Chang, Jain, Abdelfattah
- **Year:** 2025 | **Citations:** 2
- **Journal:** ArXiv (abs/2503.07518)
- **Key Finding:** **Learns to predict token importance** using a lightweight predictor (<1.2% parameter overhead). Query-aware, high-granularity predictions outperform heuristic importance estimation by 8%+ in downstream accuracy. Near-oracle accuracy on co-referential retrieval tasks.
- **Mister Smith Relevance:** **CRITICAL.** This is the "learned" part of "neural paging." Mister Smith could deploy lightweight importance predictors alongside each agent to determine which context to keep in-memory vs. page to storage, analogous to a learned page replacement policy.

### OBCache: Optimal Brain KV Cache Pruning
- **Authors:** Gu, Liang, Zhao, Diao
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2510.07651)
- **Key Finding:** Formulates cache eviction as structured pruning using Optimal Brain Damage theory. Quantifies token saliency by measuring perturbation in attention outputs, yielding closed-form scores for keys, values, and joint pairs. Output-aware signals consistently improve accuracy.
- **Mister Smith Relevance:** **MEDIUM.** The principled saliency scoring framework could inform Mister Smith's context importance metrics beyond simple attention weights.

### SDTP: Saliency-Driven Dynamic Token Pruning
- **Authors:** Tao, Tang, Wang, Zhu, Hu, Wang
- **Year:** 2025 | **Citations:** 2
- **Journal:** ArXiv (abs/2504.04514)
- **Key Finding:** Lightweight saliency prediction module estimates per-token importance from hidden states, added to different transformer layers for hierarchical pruning. Prunes 65% of tokens while maintaining comparable performance, achieving 1.75x speedup. **Combinable with KV cache compression** for further gains.
- **Mister Smith Relevance:** **HIGH.** The hierarchical pruning approach -- progressively dropping less important context at each layer -- maps to Mister Smith's tiered memory architecture where context is progressively compressed at each tier.

### FIER: Fine-Grained KV Cache Retrieval
- **Authors:** Wang, Liu, Wang, Ren, Deng, Hu, Chen, Yang
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2508.08256)
- **Key Finding:** Uses **1-bit quantized keys** for importance estimation, achieving efficient token-level retrieval. Matches full KV performance using only 11% of cache budget. Addresses the problem that important tokens are sparsely distributed across long contexts.
- **Mister Smith Relevance:** **MEDIUM.** The 1-bit key index for fast importance lookup could be adapted for Mister Smith's context retrieval -- maintaining lightweight indices in JetStream KV for rapid context selection.

---

## 6. Context Distillation & Prompt Compression

### KV-Distill: Learnable Context Compression
- **Authors:** Chari, Qin, Van Durme
- **Year:** 2025 | **Citations:** 7
- **Journal:** ArXiv (abs/2503.10337)
- **Key Finding:** Distills long KV caches into shorter representations using student-teacher pairing with KL divergence. **Question-independent** compression enabling pre-computation. Domain-specific fine-tuning achieves **up to 99% length reduction** while preserving downstream performance.
- **Mister Smith Relevance:** **CRITICAL.** Directly applicable to Mister Smith's context paging. Agent context could be compressed via KV-Distill before storage, then restored on demand. The question-independent property means compression can happen asynchronously.

### ACON: Agent Context Optimization
- **Authors:** Kang, Chen, Han, Inan, Wutschitz, Chen, Sim, Rajmohan (Microsoft)
- **Year:** 2025 | **Citations:** 1
- **Journal:** ArXiv (abs/2510.00615)
- **Key Finding:** Unified framework for compressing both environment observations and interaction histories. Uses **failure-driven guideline optimization** -- analyzing paired trajectories where compression causes failure to update compression rules. Distillable into smaller compressors (95%+ accuracy preserved). 26-54% memory reduction.
- **Mister Smith Relevance:** **HIGH.** The failure-driven compression optimization is a powerful paradigm for Mister Smith. The system could learn what context is safe to compress by analyzing task failures, then apply those learned guidelines to future context management.

### DAST: Dynamic Allocation of Soft Tokens
- **Authors:** Chen, Li, Xu, Li, Su, Shan, Zheng
- **Year:** 2025 | **Citations:** 3
- **Key Finding:** Combines perplexity-based local information density with attention-driven global importance to dynamically allocate compression budget to information-rich chunks rather than uniform distribution.
- **Mister Smith Relevance:** **MEDIUM.** The non-uniform compression budget allocation principle -- spending more "compression tokens" on information-dense regions -- applies to how Mister Smith allocates context budgets across agent roles.

### EHPC: Evaluator Head-based Prompt Compression
- **Authors:** Fei, Niu, Xie, Liu, Bai, Han
- **Year:** 2025 | **Citations:** 4
- **Journal:** ArXiv (abs/2501.12959)
- **Key Finding:** Identifies specific attention heads ("evaluator heads") that naturally select the most important tokens. Training-free compression using only first few layers to "skim" input, passing only important tokens for full inference.
- **Mister Smith Relevance:** **MEDIUM.** The evaluator head concept suggests Mister Smith could use early-layer attention patterns from the LLM itself to decide what context to retain.

---

## 7. Sparse Attention & Adaptive Budget Allocation

### Twilight: Adaptive Attention Sparsity via Top-p Pruning
- **Authors:** Lin, Tang, Yang, Wang, Tang, Tian, Stoica, Han, Gao
- **Year:** 2025 | **Citations:** 7
- **Journal:** ArXiv (abs/2502.02770)
- **Key Finding:** Applies nucleus (top-p) sampling to sparse attention, achieving **adaptive budgeting** rather than fixed top-k. Can prune up to 98% of redundant tokens, yielding 15.4x acceleration in self-attention and 3.9x end-to-end speedup.
- **Mister Smith Relevance:** **HIGH.** The adaptive budget principle -- letting the content determine how much context to retain rather than fixing a budget -- is a key design principle for Mister Smith's context management.

### DELTA: Dynamic Layer-Aware Token Attention
- **Authors:** Entezari Zarch, Gao, Jiang, Annavarm
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2510.09883)
- **Key Finding:** Partitions layers into three groups: initial layers (full attention), selection layers (identify salient tokens), and sparse layers (attend only to selected subset). Preserves full KV cache in GPU memory but avoids full-attention computation. Matches full attention on AIME/GPQA while reducing attended tokens by 5x.
- **Mister Smith Relevance:** **MEDIUM.** The three-group layer partition concept informs how Mister Smith could structure its context pipeline -- initial full-context processing, then importance selection, then sparse utilization.

### PSA: Progressive Sparse Attention
- **Authors:** Zhou, Yin, Zuo, Cheng
- **Year:** 2025 | **Citations:** 3
- **Journal:** ArXiv (abs/2503.00392)
- **Key Finding:** Adaptively adjusts KV cache budget per-token and per-layer based on real attention distributions rather than fixed budget k. Reduces KV cache usage by up to 8.8x with unified GPU memory management.
- **Mister Smith Relevance:** **MEDIUM.** The per-agent, per-task adaptive budget allocation principle.

### Pi-Attention: Periodic Sparse Transformers
- **Authors:** Liu, Yu
- **Year:** 2025 | **Citations:** 0
- **Key Finding:** Factorizes attention into local neighborhoods + periodic stride skips + adaptive fusion gate. O(kL + pi*log L) receptive field with linear per-layer complexity. 8.3% lower perplexity than RingAttention using 50% fewer GPUs.
- **Mister Smith Relevance:** **LOW-MEDIUM.** Architectural insight for model selection rather than direct framework implementation.

---

## 8. KV Cache Offloading, Paging, & Distributed Serving

### LMCache: Enterprise-Scale KV Cache Layer
- **Authors:** Cheng, Liu, Yao, An, Chen, Feng, Huang, Shen, Du, Jiang
- **Year:** 2025 | **Citations:** 6
- **Journal:** ArXiv (abs/2510.09665)
- **Key Finding:** First efficient open-source KV caching solution that extracts, stores, and shares KV caches across engines and queries. Transforms LLM engines from independent token processors into a collection of engines with **KV cache as the storage and communication medium**. Supports cache offloading + prefill-decode disaggregation. Up to **15x throughput improvement**.
- **Mister Smith Relevance:** **CRITICAL.** LMCache's model of KV cache as a shared storage/communication medium is directly applicable. Mister Smith could use NATS/JetStream as the KV cache transport layer between agents, enabling context sharing and reuse across the agent ensemble.

### HotPrefix: Hotness-Aware KV Cache Scheduling
- **Authors:** Li, Gu, Huan, Wang, Yao, Tian, Chen
- **Year:** 2025 | **Citations:** 0
- **Journal:** PACM on Management of Data
- **Key Finding:** Dynamic hotness tracking + selective cache admission (only high-hotness caches in CPU memory) + hotness promotion (periodically promoting hot prefixes from CPU to GPU). Reduces inference latency by 2.25x over vLLM.
- **Mister Smith Relevance:** **HIGH.** The hotness-based promotion/demotion pattern maps directly to Mister Smith's tiered memory. Frequently accessed agent context stays in JetStream KV (fast tier), while cold context is demoted to PostgreSQL.

### FuseSpill: KV Cache Spillover Management
- **Authors:** Jiang, Zhang, He, Luo, Lu, Chen, Zhang, Du, Huang, Lu
- **Year:** 2025-2026 | **Citations:** 0
- **Journal:** IEEE TPDS
- **Key Finding:** Comprehensive spillover cost model + KV cache swap orchestrator that disaggregates cache across heterogeneous devices + response length predictor for length-aware sequence selection. 20-40% throughput increase.
- **Mister Smith Relevance:** **MEDIUM.** The spillover cost model concept -- quantifying the cost of different eviction strategies -- is applicable to Mister Smith's resource management decisions.

### FlashForge: Prefix-Aware Attention for Shared Contexts
- **Authors:** Wang, Ning, Fang, Zhang, Lin, Ma, Zhou, et al.
- **Year:** 2025 | **Citations:** 3
- **Journal:** ArXiv (abs/2505.17694)
- **Key Finding:** Shared-prefix attention kernel that combines memory access for shared prefixes during decoding. 1.9x speedup and **120.9x memory access reduction** vs. FlashDecoding.
- **Mister Smith Relevance:** **MEDIUM.** When multiple agents share system prompts or common context, FlashForge-style prefix sharing could dramatically reduce memory overhead.

### BanaServe: Disaggregated LLM Serving
- **Authors:** He, Xu, Wu, Hu, Ma, Shen, Chen, Xu, Qu, Ye
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2510.13223)
- **Key Finding:** Dynamic orchestration with layer-level weight migration and attention-level KV cache migration. Global KV Cache Store with layer-wise overlapped transmission. 1.2-3.9x throughput over vLLM.
- **Mister Smith Relevance:** **MEDIUM.** The global KV cache store concept aligns with Mister Smith's NATS-backed distributed state management.

---

## 9. Multi-Agent Shared Memory & Context Routing

### SagaLLM: Transaction Guarantees for Multi-Agent Planning
- **Authors:** Chang, Geng
- **Year:** 2025 | **Citations:** 11
- **Journal:** Proc. VLDB Endow., Vol. 18
- **Key Finding:** Integrates the **Saga transactional pattern** with persistent memory, automated compensation, and independent validation agents. Ensures workflow-wide consistency and recovery through modular checkpointing and compensable execution. Addresses context loss and inter-agent coordination failures.
- **Mister Smith Relevance:** **CRITICAL.** This is the most architecturally aligned paper for Mister Smith. The Saga pattern maps directly to OTP-style supervision trees with compensating transactions. Mister Smith's supervision strategies (OneForOne, OneForAll, RestForOne) could integrate Saga-style memory checkpointing for context recovery on agent failure.

### Collaborative Memory: Multi-User Memory Sharing with Access Control
- **Authors:** Rezazadeh, Li, Lou, Zhao, Wei, Bao
- **Year:** 2025 | **Citations:** 3
- **Journal:** ArXiv (abs/2505.18279)
- **Key Finding:** Two memory tiers: private (per-originating user) and shared (selectively shared). Each fragment carries immutable provenance attributes (contributing agents, accessed resources, timestamps). Bipartite graph encodes asymmetric, time-evolving access controls.
- **Mister Smith Relevance:** **CRITICAL.** This directly maps to Mister Smith's security model (RBAC + audit logging). Mister Smith agents could have private memory segments and shared team memory, with NATS subject-based access control enforcing read/write policies and the audit system tracking provenance.

### RCR-Router: Role-Aware Context Routing
- **Authors:** Liu, Kong, Yang, Yang, Li, Dong, Nanjekye, et al.
- **Year:** 2025 | **Citations:** 3
- **Journal:** ArXiv (abs/2508.04903)
- **Key Finding:** First routing approach that dynamically selects semantically relevant memory subsets **per agent based on role and task stage** within a strict token budget. Lightweight scoring policy guides selection. Reduces token usage by up to 30% while maintaining quality.
- **Mister Smith Relevance:** **CRITICAL.** This is exactly what Mister Smith needs for its 9 agent roles. Each agent type (researcher, code writer, reviewer, etc.) should receive a role-filtered view of shared context rather than the full context, with a token budget manager enforcing limits.

### CoThinker: Multi-Agent Coordination under Cognitive Load Theory
- **Authors:** Shang, Liu, Liang, Zhang, Hu, Guo
- **Year:** 2025 | **Citations:** 2
- **Journal:** ArXiv (abs/2506.06843)
- **Key Finding:** Applies Cognitive Load Theory to LLM multi-agent systems. Distributes intrinsic cognitive load through **agent specialization** and manages transactional load via structured communication and a **collective working memory**. Demonstrates emergent "collective cognition" patterns.
- **Mister Smith Relevance:** **HIGH.** Mister Smith's team orchestration could implement CoThinker's cognitive load distribution -- assigning context complexity budgets to agents and managing inter-agent communication overhead as "transactional load."

---

## 10. Knowledge Graph & Temporal Memory for Agents

### Zep / Graphiti: Temporal Knowledge Graph for Agent Memory
- **Authors:** Rasmussen, Paliychuk, Beauvais, Ryan, Chalef
- **Year:** 2025 | **Citations:** 26
- **Journal:** ArXiv (abs/2501.13956)
- **Key Finding:** Temporally-aware knowledge graph engine (Graphiti) that dynamically synthesizes conversational and business data while maintaining historical relationships. Outperforms MemGPT on Deep Memory Retrieval (94.8% vs 93.4%). 18.5% accuracy improvement and 90% latency reduction on LongMemEval vs. baselines. Particularly strong on cross-session synthesis and long-term context maintenance.
- **Mister Smith Relevance:** **HIGH.** Temporal knowledge graphs could serve as Mister Smith's long-term memory backend, with Graphiti-style synthesis occurring in the persistence layer. The temporal awareness is crucial for multi-session agent workflows.

### LiCoMemory: Lightweight Cognitive Agentic Memory
- **Authors:** Huang, Guo, Zhang, Zhou, Jiang, Zhou
- **Year:** 2025 | **Citations:** 1
- **Journal:** ArXiv (abs/2511.01448)
- **Key Finding:** Introduces CogniGraph -- a lightweight hierarchical graph using entities and relations as semantic indexing layers. Temporal and hierarchy-aware search with integrated reranking. Outperforms baselines in temporal reasoning and multi-session consistency while notably reducing update latency.
- **Mister Smith Relevance:** **MEDIUM-HIGH.** The hierarchical graph with semantic indexing could be implemented using PostgreSQL's JSON capabilities or a dedicated graph store, providing efficient knowledge retrieval for Mister Smith's agent teams.

---

## 11. Cognitive-Science-Inspired Agent Memory

### Nemori: Self-Organizing Agent Memory
- **Authors:** Nan, Ma, Wu, Chen
- **Year:** 2025 | **Citations:** 5
- **Journal:** ArXiv (abs/2508.03341)
- **Key Finding:** Two core innovations: (1) **Two-Step Alignment** (from Event Segmentation Theory) -- autonomously organizes conversation into semantically coherent episodes; (2) **Predict-Calibrate** (from Free-energy Principle) -- learns from prediction gaps rather than pre-defined heuristics. Outperforms prior SOTA on LoCoMo and LongMemEval, with advantages most pronounced in longer contexts.
- **Mister Smith Relevance:** **HIGH.** The predict-calibrate principle is powerful for Mister Smith -- agents could track their own prediction errors as a signal for what context to consolidate into long-term memory, enabling self-improving memory management.

### Episodic Memory Position Paper
- **Authors:** Pink, Wu, Vo, Turek, Mu, Huth, Toneva
- **Year:** 2025 | **Citations:** 18
- **Journal:** ArXiv (abs/2502.06975)
- **Key Finding:** Argues that episodic memory (single-shot learning of instance-specific contexts) is the missing piece for long-term LLM agents. Identifies five key properties of episodic memory that enable adaptive behavior and presents a roadmap for integrating them.
- **Mister Smith Relevance:** **HIGH.** Mister Smith agents need episodic memory for task history -- remembering specific past interactions, their outcomes, and context. This maps to the persistence layer's audit log + structured retrieval.

### EMem: Event-Centric Conversational Memory
- **Authors:** Zhou
- **Year:** 2025 | **Citations:** 0
- **Key Finding:** Decomposes sessions into enriched Elementary Discourse Units (EDUs) -- self-contained event-like propositions with normalized entities. Non-compressive memory preservation using heterogeneous graphs. Matches or surpasses baselines with much shorter QA contexts.
- **Mister Smith Relevance:** **MEDIUM.** The EDU decomposition pattern could be useful for structuring Mister Smith's message history into queryable event records.

---

## 12. Hippocampal & Recurrent Memory Hybrids

### Artificial Hippocampus Networks (AHN)
- **Authors:** Fang, Yu, Zhong, Ye, Xiong, Wei (ByteDance)
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2510.07318)
- **Key Finding:** Maintains sliding window KV cache as lossless short-term memory while a learnable AHN module compresses out-of-window information into fixed-size long-term memory. Instantiated using Mamba2, DeltaNet, and Gated DeltaNet. Reduces inference FLOPs by 40.5% and memory cache by 74.0% while improving accuracy on 128K sequences.
- **Mister Smith Relevance:** **HIGH.** The dual-memory architecture (lossless recent + compressed historical) is directly applicable to Mister Smith's agent context management. Recent agent interactions stay in full fidelity; older context is compressed into fixed-size summaries.

### MemMamba: Memory Patterns in State Space Models
- **Authors:** Wang, Chen, Yan, Lu, Sun
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2510.03279)
- **Key Finding:** Mathematically derives Mamba's memory decay mechanism -- exponential decay of long-range memory. Proposes state summarization + cross-layer/cross-token attention to alleviate forgetting while preserving linear complexity. 48% inference speedup over baseline Mamba.
- **Mister Smith Relevance:** **MEDIUM.** Understanding the decay characteristics of different model architectures informs Mister Smith's model-agnostic design -- the framework should compensate for model-specific memory limitations.

### Learnable Token Eviction for Linear Attention
- **Authors:** He, Garner
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2510.20787)
- **Key Finding:** End-to-end trainable lightweight CNN aggregates information from adjacent tokens to adaptively retain critical KV-pairs per head, maintaining constant time/space complexity. Effective on retrieval-intensive benchmarks.
- **Mister Smith Relevance:** **LOW-MEDIUM.** The learnable eviction mechanism informs architectural choices for Mister Smith's model provider selection.

---

## 13. Lifelong Learning & Continual Adaptation

### Lifelong Learning of LLM Agents: A Roadmap
- **Authors:** Zheng, Shi, Cai, Li, Zhang, Li, Yu, Ma
- **Year:** 2025 | **Citations:** 33
- **Journal:** ArXiv (abs/2501.07278)
- **Key Finding:** Comprehensive survey categorizing lifelong learning into three modules: perception (multimodal input), memory (evolving knowledge storage/retrieval), and action (grounded interaction). Highlights how these pillars collectively enable continuous adaptation, mitigate catastrophic forgetting, and improve long-term performance.
- **Mister Smith Relevance:** **HIGH.** Provides the theoretical framework for Mister Smith's agents to learn and improve over time, with the memory module being most directly relevant to context management.

### LifelongAgentBench
- **Authors:** Zheng, Cai, Li, Zhang, Li, Zhang, Song, Ma
- **Year:** 2025 | **Citations:** 4
- **Journal:** ArXiv (abs/2505.11942)
- **Key Finding:** First benchmark for lifelong learning in LLM agents. Reveals that conventional experience replay has limited effectiveness due to irrelevant information and context length constraints. Introduces group self-consistency mechanism for improvement.
- **Mister Smith Relevance:** **MEDIUM.** Benchmarking insights -- Mister Smith should not naively replay past context but should curate relevant experience.

### CMT: Compression Memory Training
- **Authors:** Li, Sun, Hu, Hu, Zhang
- **Year:** 2025 | **Citations:** 1
- **Key Finding:** Compresses new documents into a memory bank without changing LLM parameters. Memory-aware objective + self-matching + top-k aggregation for encoding, retrieval, and aggregation. Reduces catastrophic forgetting risk.
- **Mister Smith Relevance:** **MEDIUM.** The compressed memory bank pattern -- storing compressed knowledge rather than raw context -- is applicable to Mister Smith's long-term persistence layer.

### Mem0: Production-Ready Scalable Long-Term Memory
- **Authors:** Chhikara, Khant, Aryan, Singh, Yadav
- **Year:** 2025 | **Citations:** 72
- **Journal:** ArXiv (abs/2504.19413)
- **Key Finding:** Dynamically extracts, consolidates, and retrieves salient information from conversations. Graph-based memory variant captures relational structures. 26% improvement over OpenAI in LLM-as-Judge metric. 91% lower p95 latency and 90%+ token cost savings vs. full-context approaches.
- **Mister Smith Relevance:** **HIGH.** Mem0 is a production-ready system demonstrating that structured persistent memory is dramatically more efficient than full-context approaches. The 90% token cost savings validates Mister Smith's tiered memory architecture approach.

---

## 14. Streaming & Infinite-Context Inference

### StreamingVLM: Infinite Video Stream Understanding
- **Authors:** Xu, Xiao, Chen, He, Peng, Lu, Han (MIT HAN Lab)
- **Year:** 2025 | **Citations:** 2
- **Journal:** ArXiv (abs/2510.09608)
- **Key Finding:** Compact KV cache reusing attention sinks + short window of recent tokens + long window of text tokens for infinite-length streaming. Training aligned with inference via overlapped chunk SFT. 66% win rate vs. GPT-4O mini on 2+ hour videos.
- **Mister Smith Relevance:** **MEDIUM.** The attention sink pattern (keeping initial tokens as anchors) is relevant to Mister Smith's system prompt management -- system prompts should be treated as persistent "sinks" that are never evicted.

### SCOUT: Sub-Quadratic Attention via Segment Compression
- **Authors:** Jafari, Fan, Jamialahmadi, Farinneya, Chen, Tahaei
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2509.00935)
- **Key Finding:** Hybrid architecture compressing tokens within fixed-size segments, then attending only over compressed "checkpoint" representations. Sub-quadratic growth matching full-attention quality at 400M and 1.3B scales.
- **Mister Smith Relevance:** **LOW-MEDIUM.** The segment compression + checkpoint concept informs how Mister Smith could create periodic "context snapshots" for efficient retrieval.

---

## 15. Context Window Extension via RoPE Scaling

### LongRoPE2: Near-Lossless Context Window Scaling
- **Authors:** Shang, Zhang, Wang, Zhang, Lopez, Yang, Chen, Yang (Microsoft)
- **Year:** 2025 | **Citations:** 8
- **Journal:** ArXiv (abs/2502.20082)
- **Key Finding:** Extends LLaMA3-8B to 128K effective context with 98.5%+ short-context retention, using only 10B tokens (80x fewer than Meta's approach). Evolutionary search guided by "needle-driven" perplexity + mixed context window training.
- **Mister Smith Relevance:** **MEDIUM.** Important for Mister Smith's model selection guidance -- models with effective context extension reduce the urgency of complex paging systems. However, even 128K contexts are insufficient for long-running multi-agent workflows, so paging remains necessary.

---

## 16. Multi-Turn Tool Use & Stateful Agent Interaction

### FuncBenchGen: Multi-Step Tool Use Evaluation
- **Authors:** Maekawa, Hassell, Pezeshkpour, Mitchell, Hruschka
- **Year:** 2025 | **Citations:** 1
- **Journal:** ArXiv (abs/2509.26553)
- **Key Finding:** Strong models make syntactically valid function calls but **propagate incorrect or stale argument values** across steps -- revealing brittle state tracking. A simple mitigation (explicitly restating prior values at each step) yields 62.5% -> 81.3% success rate improvement for GPT-5.
- **Mister Smith Relevance:** **CRITICAL.** This directly impacts Mister Smith's ToolBus design. The finding that LLMs lose track of state across tool calls means Mister Smith must explicitly inject relevant state into each tool call context, not rely on the LLM's implicit state tracking.

### DialogTool: Stateful Tool Use in Multi-Turn Dialogues
- **Authors:** Wang, Huang, Wang, Xi, Lu, Zhang, Hu, Liu, Pan, Wong
- **Year:** 2025 | **Citations:** 6
- **Key Finding:** Even SOTA LLMs cannot perform well with tools over long horizons. Evaluates the full tool lifecycle (creation, awareness, selection, execution, response) across 13 LLMs.
- **Mister Smith Relevance:** **HIGH.** Validates Mister Smith's architectural decision to maintain tool state externally (in the ToolBus) rather than relying on the LLM's internal state tracking.

### Accelerating Multi-Turn Workflows via Context Templating
- **Authors:** Wang, Wen, Zhang
- **Year:** 2025 | **Citations:** 0
- **Key Finding:** Context templating + opportunistic prefill for multi-turn agent workflows.
- **Mister Smith Relevance:** **MEDIUM.** Template-based context management could accelerate Mister Smith's repeated agent invocation patterns.

---

## 17. Surveys & Taxonomies

### Rethinking Memory in AI: Taxonomy, Operations, Topics
- **Authors:** Du, Huang, Zheng, Wang, Montella, Lapata, Wong, Pan
- **Year:** 2025 | **Citations:** 16
- **Journal:** ArXiv (abs/2505.00675)
- **Key Finding:** Defines six fundamental memory operations: **Consolidation, Updating, Indexing, Forgetting, Retrieval, Compression**. Maps these to research topics across long-term, long-context, parametric modification, and multi-source memory. Categorizes representations into parametric and contextual forms.
- **Mister Smith Relevance:** **CRITICAL.** This taxonomy provides the definitive vocabulary for Mister Smith's memory subsystem design. Each of the six operations should have a corresponding implementation in the framework.

### Agentic RAG Survey
- **Authors:** Singh, Ehtesham, Kumar, Khoei
- **Year:** 2025 | **Citations:** 128
- **Journal:** ArXiv (abs/2501.09136)
- **Key Finding:** Comprehensive exploration of how autonomous agents embedded in RAG pipelines dynamically manage retrieval, refine context, and adapt workflows. Covers reflection, planning, tool use, and multi-agent collaboration patterns.
- **Mister Smith Relevance:** **HIGH.** Mister Smith's agent system already incorporates many Agentic RAG patterns. This survey validates the architectural direction and identifies patterns to adopt.

### AUGUSTUS: Multimodal Agent with Contextualized Memory
- **Authors:** Jain, Maheshwari, Yu, Hwu, Shi
- **Year:** 2025 | **Citations:** 0
- **Journal:** ArXiv (abs/2510.15261)
- **Key Finding:** Graph-structured multimodal contextual memory using semantic tags for concept-driven retrieval. 3.5x faster than multimodal RAG for ImageNet classification. Outperforms MemGPT on MSC benchmark.
- **Mister Smith Relevance:** **MEDIUM.** The semantic tag approach for memory indexing is more efficient than vector similarity search for certain retrieval patterns.

---

## 18. Emerging Directions

### 18.1 Memory as a First-Class OS Resource

The MemOS and MemoryOS papers represent a paradigm shift: treating LLM memory not as an afterthought but as a **managed system resource** with lifecycle, versioning, and scheduling semantics. The MemCube abstraction (content + metadata + provenance) is analogous to pages in virtual memory systems, with explicit support for migration between memory types (plaintext <-> activation <-> parameter).

**Novelty Assessment:** This direction is in its infancy (both papers from mid-2025) but represents the most promising long-term vision for Mister Smith's architecture.

### 18.2 Learned/Predictive Context Importance

TokenButler's demonstration that token importance is **predictable** via lightweight learned predictors opens the door to truly "neural" paging policies. Instead of heuristic LRU/LFU-style eviction, a learned predictor could anticipate which context will be needed based on the current query/task state.

**Novelty Assessment:** This is the cutting edge of the "neural paging" concept -- moving from reactive eviction to proactive importance prediction.

### 18.3 Failure-Driven Compression Learning (ACON)

Microsoft's ACON framework learns what context is safe to compress by analyzing failure modes -- cases where compression caused task failure. This is a form of reinforcement learning for memory management.

**Novelty Assessment:** Highly novel and directly applicable to multi-agent systems where different agent roles may have different compression sensitivity profiles.

### 18.4 Saga-Pattern Memory Transactions (SagaLLM)

Applying database transaction patterns (Saga) to multi-agent memory management, with compensating transactions for recovery. Published in VLDB -- indicating database community interest in LLM memory management.

**Novelty Assessment:** The intersection of distributed systems (Saga pattern) and LLM memory is new and directly relevant to Mister Smith's OTP-style supervision.

### 18.5 Cognitive Load Distribution (CoThinker)

Applying Cognitive Load Theory to multi-agent LLM systems -- distributing context complexity budgets across specialized agents rather than overloading any single agent.

**Novelty Assessment:** First principled framework for context budget distribution in multi-agent systems. Directly actionable for Mister Smith.

### 18.6 KV Cache as Communication Medium (LMCache)

Reframing KV caches as a **shared communication substrate** between inference engines, not just local optimization. This transforms context management from a per-agent concern to a distributed systems problem.

**Novelty Assessment:** This reframing is powerful for Mister Smith, where NATS/JetStream already serves as the message transport -- extending it to serve as the KV cache transport layer creates a unified communication/context substrate.

### 18.7 Implicit Token Merging (DMS)

Dynamic Memory Sparsification's approach of **delaying eviction and implicitly merging** representations rather than hard-deleting tokens is a more nuanced approach to context compression. It preserves information through representation fusion.

**Novelty Assessment:** This is a refinement of the eviction paradigm that could significantly improve context quality in Mister Smith's compressed memory tiers.

---

## 19. Synthesis: Implications for Mister Smith

### Direct Architectural Mappings

| Research Concept | Mister Smith Component | Implementation Path |
|---|---|---|
| MemOS MemCubes | Agent state records | JetStream KV entries with metadata envelopes |
| MemoryOS 3-tier hierarchy | HybridStateManager | JetStream KV (STM/MTM) + PostgreSQL (LTM) |
| Collaborative Memory access control | Security + RBAC | NATS subject ACLs + JWT claims for memory segments |
| SagaLLM compensating transactions | Supervision strategies | Extend OneForOne/OneForAll with memory checkpoints |
| RCR-Router role-aware context | Agent role system | Per-role context filtering in AgentRuntime |
| TokenButler learned importance | Context paging policy | Lightweight predictor per agent role |
| KV-Distill compression | Context offloading | Compress before JetStream KV storage |
| KVTC transform coding | Long-term context storage | PCA + quantization for PostgreSQL-stored context |
| LMCache shared KV | NATS transport | Extend NATS subjects for context sharing |

### Recommended Research-Informed Design Principles

1. **Adaptive Budgets, Not Fixed Limits.** (Twilight, PSA, CAKE) Context budgets should be dynamic and content-aware, not hardcoded per agent.

2. **Delay Eviction, Merge Instead.** (DMS, ZSMerge) Rather than hard-deleting old context, compress and merge it into summary representations.

3. **Provenance Is Non-Negotiable.** (Collaborative Memory, MemOS) Every memory fragment must carry metadata: source agent, timestamp, contributing tools, access policy.

4. **Learn From Failures.** (ACON) Track cases where context compression caused task degradation and use that signal to improve compression policies.

5. **Role-Aware Context Routing.** (RCR-Router, CoThinker) Different agent roles need different context subsets. Route context based on role + task stage, not broadcast everything.

6. **Explicit State Injection for Tool Calls.** (FuncBenchGen) Never rely on the LLM's implicit state tracking across tool calls. Always explicitly inject relevant state.

7. **Temporal Awareness in Memory.** (Zep/Graphiti, LiCoMemory) Memory systems must track time -- recency, temporal ordering, and historical relationships matter for retrieval quality.

8. **Episodic Memory for Learning.** (Pink et al. position paper) Agents need instance-specific memory of past task executions, not just generalized knowledge, to improve over time.

---

## Paper Index (Alphabetical by First Author)

| # | Paper | Authors | Year | Citations | Section |
|---|---|---|---|---|---|
| 1 | ACON | Kang et al. | 2025 | 1 | 6 |
| 2 | AhaKV | Gu et al. | 2025 | 0 | 3 |
| 3 | A-MEM | Xu et al. | 2025 | 127 | 2 |
| 4 | Artificial Hippocampus Networks | Fang et al. | 2025 | 0 | 12 |
| 5 | AttnComp | Zhao et al. | 2025 | 3 | 6 |
| 6 | AUGUSTUS | Jain et al. | 2025 | 0 | 17 |
| 7 | BanaServe | He et al. | 2025 | 0 | 8 |
| 8 | CAKE | Qin et al. | 2025 | 17 | 3 |
| 9 | CMT | Li et al. | 2025 | 1 | 13 |
| 10 | CoThinker | Shang et al. | 2025 | 2 | 9 |
| 11 | Collaborative Memory | Rezazadeh et al. | 2025 | 3 | 9 |
| 12 | Context Templating | Wang et al. | 2025 | 0 | 16 |
| 13 | DAST | Chen et al. | 2025 | 3 | 6 |
| 14 | DELTA | Zarch et al. | 2025 | 0 | 7 |
| 15 | DMS (Hyper-Scaling) | Lancucki et al. | 2025 | 8 | 4 |
| 16 | EHPC | Fei et al. | 2025 | 4 | 6 |
| 17 | EMem | Zhou | 2025 | 0 | 11 |
| 18 | Episodic Memory Position | Pink et al. | 2025 | 18 | 11 |
| 19 | FIER | Wang et al. | 2025 | 0 | 5 |
| 20 | FlashForge | Wang et al. | 2025 | 3 | 8 |
| 21 | FuncBenchGen | Maekawa et al. | 2025 | 1 | 16 |
| 22 | FuseSpill | Jiang et al. | 2025-26 | 0 | 8 |
| 23 | HM-RAG | Liu et al. | 2025 | 16 | 17 |
| 24 | HotPrefix | Li et al. | 2025 | 0 | 8 |
| 25 | KV-Distill | Chari et al. | 2025 | 7 | 6 |
| 26 | KVmix | Li et al. | 2025 | 0 | 4 |
| 27 | KVTC | Staniszewski et al. | 2025 | 0 | 4 |
| 28 | KVTuner | Li et al. | 2025 | 6 | 4 |
| 29 | LiCoMemory | Huang et al. | 2025 | 1 | 10 |
| 30 | Lifelong LLM Agents Roadmap | Zheng et al. | 2025 | 33 | 13 |
| 31 | LifelongAgentBench | Zheng et al. | 2025 | 4 | 13 |
| 32 | LMCache | Cheng et al. | 2025 | 6 | 8 |
| 33 | LongRoPE2 | Shang et al. | 2025 | 8 | 15 |
| 34 | Mem0 | Chhikara et al. | 2025 | 72 | 13 |
| 35 | MemMamba | Wang et al. | 2025 | 0 | 12 |
| 36 | MemOS | Li et al. | 2025 | 22 | 1 |
| 37 | MemoryOS | Kang et al. | 2025 | 14 | 1 |
| 38 | MMS | Zhang et al. | 2025 | 0 | 2 |
| 39 | MPCache | Zeng et al. | 2025 | 1 | 3 |
| 40 | Nemori | Nan et al. | 2025 | 5 | 11 |
| 41 | OBCache | Gu et al. | 2025 | 0 | 5 |
| 42 | Pi-Attention | Liu et al. | 2025 | 0 | 7 |
| 43 | PSA | Zhou et al. | 2025 | 3 | 7 |
| 44 | QuickSilver | Khanna et al. | 2025 | 0 | 4 |
| 45 | RCR-Router | Liu et al. | 2025 | 3 | 9 |
| 46 | Rethinking Memory in AI | Du et al. | 2025 | 16 | 17 |
| 47 | S-AI-GPT Memory | Slaoui | 2025 | 0 | 2 |
| 48 | SAGE-KV | Wang et al. | 2025 | 6 | 3 |
| 49 | SagaLLM | Chang et al. | 2025 | 11 | 9 |
| 50 | SCOUT | Jafari et al. | 2025 | 0 | 14 |
| 51 | SDTP | Tao et al. | 2025 | 2 | 5 |
| 52 | SmallKV | Zhao et al. | 2025 | 1 | 3 |
| 53 | StreamingVLM | Xu et al. | 2025 | 2 | 14 |
| 54 | DialogTool | Wang et al. | 2025 | 6 | 16 |
| 55 | TokenButler | Akhauri et al. | 2025 | 2 | 5 |
| 56 | Twilight | Lin et al. | 2025 | 7 | 7 |
| 57 | Working Memory in LLMs | Huang et al. | 2025 | 0 | 11 |
| 58 | Agentic RAG Survey | Singh et al. | 2025 | 128 | 17 |
| 59 | ZSMerge | Liu et al. | 2025 | 2 | 4 |
| 60 | Chunked Prefills | Agrawal et al. | 2025 | 1 | 8 |
| 61 | Prompt Leakage via KV-Cache | Wu et al. | 2025 | 18 | 8 |

---

*Total papers surveyed: 61 | Search queries executed: 20 | All papers from 2025+*
*Research conducted via Consensus Academic Search API*
