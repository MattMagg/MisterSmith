# Model Routing & Cost Optimization -- Consolidated State of Knowledge

**Date**: 2026-03-07
**Sources**: 7 research files spanning Rounds 3-7 (industry reports, 50+ academic papers, 4 discovery sweeps, 1 frontier deep dive)
**Scope**: Everything relevant to model routing, cost optimization, and inference economics for the Mister Smith framework

---

## Executive Summary

Model routing and cost optimization is the single most mature domain in the Mister Smith research corpus. Seven independent research rounds -- spanning triple-synthesized industry reports (R3), 50+ academic papers from 2025-2026 (R4), frontier deep dives into step-level intelligence (R6), and four discovery sweeps (R4, R5, R7c, R7d) -- converge on a layered architecture with high confidence. The central conclusion is that intelligent routing delivers 27-85% cost reduction while maintaining or improving quality, and that Mister Smith's Rust + NATS + actor architecture provides a decisive infrastructure advantage over Python-based alternatives.

Three tiers of findings emerge. **Tier 1 (proven, ship now):** Two-plane router architecture with health-aware circuit breakers, hierarchical budget enforcement via JetStream KV CAS, deterministic heuristic routing, and SLM-default/LLM-fallback cascading. These are validated by multiple independent sources with concrete production evidence. **Tier 2 (promising, ship next):** Learned routing via kNN/ONNX embeddings (RouteLLM-style), Process Reward Model-gated speculative decoding (RSD achieving 4.4x FLOP reduction), and Cognitive Load-Aware token budgeting (CLAI/TALE achieving 45-67% token reduction). These have strong academic evidence but limited production deployment data. **Tier 3 (speculative, experiment):** Shared KV cache / disaggregated serving (PrefillShare), knowledge-aware routing with privacy-preserving KB signals (KABB), market-based auction routing, and topology-aware orchestration (AdaptOrch). These represent frontier innovations with high potential but require significant infrastructure investment.

The strategic implication for Mister Smith is clear: the framework's NATS-native two-plane router, combined with actor-supervised cascade pipelines and JetStream KV-backed budget enforcement, positions it to deliver cost-quality Pareto optimality that Python-based orchestrators (LangGraph, AutoGen, CrewAI) structurally cannot match. The ~50 us NATS request-reply latency and 100,000+ RPS throughput leave orders of magnitude of headroom for adding routing intelligence without impacting interactive latency.

---

## High-Confidence Findings

These findings are confirmed by 3+ independent sources OR both industry and academic validation.

### 1. Two-Plane Router Architecture is the Correct Design

**Evidence strength**: Converged across all three R3 industry reports; validated by academic surveys (R4: Varangot-Reille 2025, 10 citations; Behera 2025, 4 citations); reinforced by production gateways (Bifrost, Kong, Vercel, AWS).

**What it is**: Separate a microsecond-latency **data plane** (per-request routing with local computation + cached state) from a **control plane** (continuously updated telemetry, pricing, budgets, health, and learned parameters streamed via JetStream KV watches).

**Specific numbers**: NATS request-reply benchmarks demonstrate ~50 us average latency. Rust-based gateways like Bifrost achieve 11 us overhead at 5,000 RPS. LiteLLM (Python/FastAPI) achieves <500 RPS. [R3]

**Mister Smith implication**: Build the `ModelRouter` on Core NATS request-reply. Data plane state tables (model registry, cost model, budgets, health snapshot, routing policy) are all updated via JetStream KV watches -- "publish once, all routers update" without service restarts.

### 2. Health-Aware Routing and Circuit Breakers are Foundational Prerequisites

**Evidence strength**: All three R3 reports converge; validated by Kong, Envoy, Azure production guidance; academic papers on abstention cascades (R4: Zellinger 2025, 5 citations; Fanconi 2025).

**What it is**: Passive health checks (circuit breakers monitoring proxied traffic for timeouts, 429s, 500s) combined with NATS queue groups provide automatic failover without active polling overhead. Active health checks (probing) re-enable previously unhealthy targets.

**Specific signals**: Consecutive failure count, rolling error rate, P50/P95/P99 latency percentiles, rate-limit proximity (429/Retry-After), temporal success rate, capability mismatch. [R3]

**Critical operational guidance**: Azure explicitly warns that predicting throttling in advance via consumption tracking is "fraught with edge cases." Instead, honor `Retry-After` on 429 responses and break the circuit. [R3]

**Mister Smith implication**: This must ship in Stage 0. All advanced routing (bandits, auctions, learned models) produces noisy feedback when endpoints are unstable. Health-aware routing is a prerequisite for everything else.

### 3. Budget Enforcement Belongs in the Router, Not in Application Code

**Evidence strength**: All three R3 reports converge; LiteLLM production patterns; hierarchical budget research (R3 Section 8).

**What it is**: Hierarchical budget tracking (org -> team -> user -> request tag) using JetStream KV atomic Compare-And-Swap (CAS) operations. Reserve estimated tokens before sending, reconcile actual usage afterward.

**Specific patterns**: Hard caps (reject when exhausted), soft caps with degradation (downgrade to cheaper model), budget-conditioned routing (as budgets deplete, route cheaper), tag-based accounting (cost center attribution). [R3]

**Thundering herd mitigation**: Under high concurrency, multiple requests may pass budget checks before token usage is committed. CAS-based enforcement should demonstrate <1% overrun rate vs potentially unbounded overruns with naive check-then-spend. [R3]

**Mister Smith implication**: JetStream KV CAS is the exact primitive needed. Budget checks execute in the data plane as constant-time in-memory lookups, refreshed by control-plane updates.

### 4. Learned Routing Delivers 27-85% Cost Savings

**Evidence strength**: RouteLLM (open source, widely cited); Avengers-Pro (R4: Zhang 2025, 8 citations, +7% accuracy at 27% cost reduction OR 90% accuracy at 63% cost reduction); BEST-Route (R4: Ding 2025, 9 citations, 60% cost reduction with <1% performance drop); SpareLLM (R4: Jo 2025, 8.6x cost savings, 90% output equivalence); EMAFusion (R4: Shah 2025, 94.3% accuracy at 4x lower cost).

**Key insight from R4**: kNN beats complex learned routers. A well-tuned kNN approach matches or outperforms state-of-the-art learned routers across diverse tasks (R4: Li 2025, 1 citation). Start simple before investing in complex architectures.

**Tiered classifier pipeline** (R3, validated by R4):
| Stage | Latency Budget | Mechanism | Coverage |
|:---|:---|:---|:---|
| A | Microseconds | Capability filters + rule-based features | Hard constraints, 50-70% traffic |
| B | Microseconds-low ms | Embedding lookup / cached kNN similarity | Semantic matching |
| C | Milliseconds | Optional ML inference (BERT/ONNX) | High accuracy routing |
| D | Slow path | LLM-assisted routing | Complex/ambiguous queries |

**Mister Smith implication**: Implement Stage A as MVP, add Stage B with local ONNX embeddings (<50 ms latency, routes ~80% to Tier 1), then optionally Stage C. The local ONNX embedding router (FastEmbed blueprint) is the sweet spot for Mister Smith. [R3]

### 5. SLM-Default / LLM-Fallback Fundamentally Changes the Economics

**Evidence strength**: R4 discovery (Sharma & Mehta 2025 -- comprehensive evidence); R4 discovery (Liu 2025, 106 citations -- 0.5B outperforms GPT-4o with compute-optimal scaling); R4 discovery (Yang 2025, 81 citations -- optimal CoT length distribution, excessive length impairs reasoning).

**What it is**: 1-12B parameter models are often sufficient and sometimes superior for schema-constrained agentic workloads. With guided decoding (XGrammar, Outlines) and JSON Schema enforcement, SLMs match or surpass larger models at 10-100x lower cost. [R4 discovery]

**Specific numbers**: 0.5B model outperforms GPT-4o with compute-optimal test-time scaling. 7B model beats o1 and DeepSeek-R1. 10-100x cost reduction for structured outputs. [R4 discovery]

**Mister Smith implication**: Phase 9 should include a `LocalModelProvider` alongside API providers. For many structured tasks (JSON generation, schema validation, code formatting), local inference should be the primary tier. This shifts the question from "which cloud model" to "cloud vs. local."

### 6. Cascading is a Natural Fit for Actor Systems

**Evidence strength**: R3 (all three reports converge); R4 academic (Zellinger 2025 -- early abstention reduces cost 13% and error rate 5%; Fanconi 2025 -- three-tier cascade with online learning); R6 (RSD achieving 4.4x FLOP reduction).

**What it is**: Sequentially query models and stop early when a response is deemed "reliable enough." FrugalGPT reports up to 98% cost reductions. Abstention (small model explicitly declining) reduces cost by 13% and error rate by 5%. [R3, R4]

**Three-tier architecture**: Cheap LLM -> Expensive LLM -> Human expert. A deferral policy decides base-vs-large, then an abstention policy decides LLM-vs-human. Online learning adapts to shifting task distributions. [R4: Fanconi 2025]

**Mister Smith implication**: Cascading maps directly to supervised actor workflows with explicit escalation states. Implement as synchronous fallthrough cascade first (sequential actor calls with NATS request-reply), then add background speculative calls.

---

## Key Techniques & Architectures

### Task-Level Routing (RouteLLM, Learned Cascades)

**Mechanism**: A routing classifier (trained on preference data, embeddings, or capability profiles) maps each incoming query to the cheapest model expected to handle it well. Multiple router architectures exist: similarity-weighted Elo ranking, matrix factorization, BERT classifiers, causal LLM classifiers. [R3, R4]

**Key papers and numbers**:
- **RouteLLM**: Up to 85% cost reduction at near-strong-model quality on several benchmarks. Multiple router architectures available. Open source. [R3]
- **Avengers-Pro** (R4: Zhang 2025, 8 citations): +7% accuracy over strongest single model, OR equivalent accuracy at 27% lower cost, OR ~90% accuracy at 63% lower cost.
- **BEST-Route** (R4: Ding 2025, 9 citations): Routes queries AND chooses number of samples. Up to 60% cost reduction with <1% performance drop.
- **SpareLLM** (R4: Jo 2025): Profiling-then-routing. 8.6x cost savings, 90% output equivalence. Accounts for 91.1% of Pareto curve points.
- **EMAFusion** (R4: Shah 2025): Hybrid rules + learned + cascade fallback. 94.3% accuracy at 4x lower cost; 17.1pp improvement over GPT-4 at 1/20th cost.
- **LLM Bandit** (R4: Li 2025, 18 citations): Multi-armed bandit with user-specified preferences. Generalizes to unseen LLMs.
- **kNN** (R4: Li 2025): Matches or outperforms SOTA learned routers. Start here before building anything complex.

**Confidence signals for escalation** (R3):
- Post-hoc quality estimation (FrugalGPT scoring function)
- Confidence tokens (Self-REF -- outperforms verbalized confidence and token-probability signals)
- Logit-based confidence (per-token and sequence log-probability thresholds)
- Calibrated confidence (temperature scaling, supervised uncertainty estimation)
- Lightweight verifiers (small verifier models, structured output validation)

**Mister Smith integration path**: Implement kNN-based routing as the first learned router (Stage 1-2). Query embeddings computed locally via ONNX. Confidence should be a first-class capability in the `ModelProvider` trait. Abstention is an explicit state in `AgentRuntime`.

### Step-Level Routing (PRMs, RSD, Speculative Decoding)

**Mechanism**: Instead of routing entire tasks, evaluate and route at the reasoning-step granularity. A Process Reward Model (PRM) scores each intermediate step. If the score is too low, escalate that specific step to a more powerful model. [R6, R4 discovery]

**Key papers and numbers**:
- **Reward-Guided Speculative Decoding (RSD)** (R6, R4 discovery: Liao 2025, 63 citations): Draft model (1.5B) generates candidate steps, PRM evaluates, target model (70B) corrects if rejected. Draft model handles up to 65% of tokens without intervention. **Up to 4.4x fewer FLOPs** with +3.5 accuracy points improvement. [R6]
- **BiPRM** (R6: 2025): Bidirectional PRM with only 5% latency overhead (27.982ms to 29.393ms). 37.7% improvement in step-level error detection. L2R and R2L streams run in parallel. [R6]
- **R-PRM** (R6, R4 discovery: She 2025, 19 citations): Generative PRM with chain-of-thought critique. 11.9 F1 improvement on ProcessBench. Self-improves via DPO without human annotation. [R6]
- **Streaming Content Monitors (SCM)** (R6: 2025): Detect failures by evaluating only the first 18% of generated tokens. 95%+ detection accuracy. Enables mid-stream abort. [R6]
- **PRM Calibration** (R6: 2025): Off-the-shelf PRMs overestimate success probabilities. Quantile regression calibration generates reliable confidence bounds. Required for MCTS-style search. [R6]

**Step-level routing comparison** (R6):
| Policy | Mechanism | FLOP Reduction | Recommendation |
|:---|:---|:---|:---|
| Upfront Difficulty Prediction | Classify entire task upfront | Moderate | Baseline fallback only |
| DAAO (probing during reasoning) | Probe LLM during reasoning to stop early | Moderate | Combine with token budgeting |
| **RSD (start-cheap-escalate)** | Draft -> PRM evaluate -> Target correct | **Up to 4.4x** | **Primary strategy** |

**Context transfer overhead**: Switching from 1.5B to 70B mid-task requires KV cache transfer. LLaMA-2 70B at 4K context consumes ~10 GB. PagedAttention (vLLM) reduces KV cache waste to under 4%. NVIDIA Dynamo enables KV cache offloading to CPU/SSD. [R6]

**Mister Smith integration path**: Phase 1: Use CoT entropy or LLM-as-judge as training-free PRM proxy with static token budgets. Phase 2: Deploy dedicated 1.5B BiPRM with JetStream CAS rollback. Phase 3: Full RSD routing with learned budgeting and offline RL. The supervision tree handles step-level failures: abort draft actor, roll back JetStream KV state to previous revision, route to target model.

### Token Budgeting (CLAI, TALE, Cognitive Load Taxonomy)

**Mechanism**: Estimate the intrinsic complexity of a prompt before generation, then enforce a dynamic token budget that prevents LLMs from "overthinking" simple steps while allocating sufficient compute for genuinely hard problems. [R6, R4, R4 discovery]

**Key papers and numbers**:
- **TALE** (R6: 2025): Zero-shot budget estimator predicts optimal token budget. **67% token cost reduction with <3% accuracy decrease.** [R6]
- **CLAI** (R6, R4, R4 discovery: Zhang 2025, 2 citations): Operationalizes Cognitive Load Theory. Three load types: Intrinsic (problem complexity), Extraneous (wasteful computation), Germane (productive reasoning). CLAI-Prompt reduces tokens up to 45% without accuracy loss. CLAI-Tune exhibits emergent problem decomposition. [R6, R4, R4 discovery]
- **SelfBudgeter** (R4: Li 2025, 18 citations): Model predicts token budget before reasoning, trained via RL to adhere. 61% response length compression (1.5B model), 48% (7B model), with nearly undiminished accuracy. [R4]
- **Budget Guidance** (R4: Li 2025, 10 citations): Lightweight predictor models Gamma distribution over remaining thinking length. +26% accuracy on MATH-500 under tight budgets. Emergent difficulty estimation. No fine-tuning required. [R4]
- **Thinking-Optimal Scaling** (R4 discovery: Yang 2025, 81 citations): Excessive chain-of-thought impairs reasoning. Optimal CoT length distribution exists per domain. [R4 discovery]

**Budget enforcement without breaking CoT** (R6): Hard-capping with `max_tokens` can truncate valid reasoning. Instead use: (1) dynamic verbosity hinting via injected system prompts, (2) early stopping when approaching budget, (3) "continuation tickets" allowing resumption in a subsequent budgeted step. [R6]

**Mister Smith integration path**: Implement CLAI's three-load taxonomy as a preprocessing step in the Coordinator agent. SelfBudgeter or Budget Guidance (no fine-tuning needed) as middleware between agent and provider. Track "productive reasoning rate" (useful output tokens / total tokens) as an operational metric.

### SLM-Default / LLM-Fallback

**Mechanism**: Default to small (1-12B) local models for most tasks, escalating to expensive API models only when confidence is low or the task requires capabilities beyond the SLM. [R4 discovery, R3]

**Key papers and numbers**:
- 0.5B model outperforms GPT-4o with compute-optimal test-time scaling. [R4 discovery: Liu 2025, 106 citations]
- 7B model beats o1 and DeepSeek-R1 under same conditions. [R4 discovery: Liu 2025]
- 10-100x cost reduction for structured outputs with guided decoding. [R4 discovery: Sharma & Mehta 2025]
- Optimal CoT length distribution exists per domain -- excessive thinking hurts. [R4 discovery: Yang 2025, 81 citations]

**Mister Smith integration path**: Add `LocalModelProvider` in Phase 9. Implement guided decoding for structured outputs (JSON tool calls, code blocks). Build confidence-based routing: SLM attempts first, escalates to LLM on low confidence. Track cost-per-successful-task (CPS) metric to empirically optimize routing thresholds.

### Knowledge-Aware Routing (KB Signals, DAAO, KABB)

**Mechanism**: Augment static agent descriptions with dynamic signals derived from each agent's internal knowledge base, enabling adaptive routing without exposing proprietary data. [R5]

**Key papers and numbers**:
- **KB-Aware Orchestration** (R5: Trombino 2025): Privacy-preserving relevance signals from agent KB. Populates a shared semantic cache for future queries. Evidence strength: moderate (7/10). [R5]
- **Federation of Agents (FoA)** (R5: Giusti 2025): Versioned Capability Vectors (VCVs) make agent capabilities machine-searchable via semantic embeddings in sharded HNSW structures. Supports cost/policy-aware routing at scale. [R5]
- **DAAO** (R5, R4: Su 2025): Dynamically generates query-specific multi-agent workflows guided by predicted query difficulty using VAE. Modular operator allocation. Self-adjusting policy updates. [R5, R4]
- **KABB** (R5: Zhang 2025): Knowledge-Aware Bayesian Bandits for dynamic expert coordination. [R5]
- **IRT-Router** (R4: Song 2025, 7 citations): Item Response Theory from psychometrics. Interpretable ability/difficulty decomposition. Online warm-up via semantic similarity. Tested on 20 LLMs and 12 datasets. [R4]

**Evidence level**: Moderate. These are academically validated but lack production deployment data. The privacy-preserving KB signal approach is novel and uniquely suited to multi-agent systems where agents hold proprietary context.

**Mister Smith integration path**: Start with IRT-Router's ability/difficulty decomposition stored in the model registry. Add DAAO-style difficulty estimation as a preprocessing step. KB-aware routing is a later addition, gated behind privacy requirements.

### Shared KV Cache / Disaggregated Serving (PrefillShare)

**Mechanism**: Physically decouple the compute-bound prefill phase from the memory-bound decode phase. Heterogeneous task-specific agent models share a single frozen prefill module via "prefill-only tuning" that aligns latent spaces of specialized downstream decoders. [R7d]

**Key papers and numbers**:
- **PrefillShare** (R7d: 2026): Multiple task-specific models consume the exact same KV cache without redundant computation. "Massive reduction in latency" with "substantially higher throughput in multi-model agent workloads." Matches accuracy of fully fine-tuned independent models. [R7d]
- **SUN (Shared Use of Next-token Prediction)** (R7d: 2026): Extends the shared prefill concept. [R7d]
- **NVIDIA Dynamo** (R7d: 2026): Local KV indexers and non-blocking radix snapshots for state transfers between physically separate GPU nodes. [R7d]
- **Persistent KV Cache** (R7c: 2026): 4-bit quantized KV cache persisted to SSD. Agent resume latency drops from ~15.7 seconds (FP16) to ~0.6 seconds. [R7c]

**Evidence level**: Promising but frontier. PrefillShare is a 2026 paper with limited independent validation. The concept requires Mister Smith to evolve from routing text payloads to streaming tensor embeddings and KV cache pointers -- a fundamental data-plane change.

**Mister Smith integration path**: Not for initial implementation. Monitor adoption by inference serving frameworks (vLLM, TensorRT-LLM). When available, Mister Smith's NATS data plane could transmit distributed pointers to unified KV cache stores rather than re-serialized prompts. The persistent KV cache (disk-backed, 4-bit quantized) is more immediately actionable for local model deployments.

---

## Production Patterns & Case Studies

### Vercel: Fewer Is More

**Source**: R7c (Vercel postmortem, December 2025)

A text-to-SQL agent originally had 16 specialized tools. By **removing 80% of tools and letting the model use a generic bash tool**, accuracy rose from 80% to 100% and latency dropped 3.5x. [R7c]

**Implication**: Excessive specialization confuses LLMs and increases brittleness. Mister Smith should favor minimalism -- only add agent specialization when it demonstrably helps. This directly challenges the assumption that more agent roles = better performance.

### Google: Quantitative Scaling Laws for Agent Teams

**Source**: R7c (Kim & Liu, Google Research, 2026)

180 agent configurations evaluated. Multi-agent teams **dramatically improve** performance on parallelizable tasks but can **degrade** it on sequential tasks. Adding agents only pays off when subtasks are independent; otherwise coordination overhead causes ceiling effects or regressions. Built a predictive model for architecture selection. [R7c]

**Implication**: Mister Smith's orchestrator should dynamically choose team size based on task parallelism. The static 9-role team definition is suboptimal for sequential workloads. Implement dynamic team resizing -- monitor subtask independence and collapse agents if communication overhead outweighs benefits.

### Self-MoA Outperforms Cross-Model MoA

**Source**: R4 (Li 2025, 21 citations)

Self-MoA (aggregating outputs from ONLY the single top-performing LLM, not multiple different LLMs) outperforms standard MoA that mixes different LLMs by 6.6% on AlpacaEval 2.0 and 3.8% average across benchmarks. Mixing different LLMs often lowers average quality due to quality sensitivity. [R4]

**Implication**: Default to self-consistency (multiple samples from the best model with voting) rather than cross-model ensembles. Only activate cross-model MoA when profiling data shows complementary strengths.

### MoA for Code Optimization

**Source**: R4 (Ashiga 2025, 1 citation)

First MoA application to industrial code optimization. MoA excels with open-source models, achieving 14.3-22.2% cost savings and 28.6-32.2% faster optimization. GA-based ensemble better with commercial models. [R4]

**Implication**: Validates MoA for code-related tasks, directly relevant to Mister Smith's Implementer and Reviewer agent roles.

### Multi-Stage Code Orchestration (PerfOrch)

**Source**: R4 (Chen 2025)

17 LLMs across 5 programming languages. "Pronounced performance heterogeneity by language, development stage, and problem category." Stage-wise validation and rollback achieves 96.22% correctness vs GPT-4o's 78.66%. [R4]

**Implication**: For code generation, multi-model orchestration dramatically outperforms single-model approaches. Different models excel at different languages and development stages. Mister Smith's Implementer agent should route by language and task type.

### AdaptOrch: Topology Over Model Capability

**Source**: R7d (2026)

As foundation models converge in capability, the orchestration topology now dominates system-level performance over individual model choice. AdaptOrch uses a linear-time algorithm to analyze task graph properties (parallelism width, critical path depth, inter-subtask coupling) and dynamically routes to optimal topologies (parallel, sequential, hierarchical, hybrid). "Double-digit percentage improvements over static single-topology baselines, even when underlying models remain identical." [R7d]

**Implication**: Mister Smith needs a Topology Compiler that analyzes task dependency graphs before execution and selects the optimal orchestration pattern per task. Static workflow definitions are insufficient.

---

## Open Questions & Gaps

### Needs Experimentation (Not More Research)

1. **NATS vs HTTP/gRPC end-to-end latency**: No measured NATS vs HTTP/gRPC latency comparison exists in any source. The ~50 us NATS request-reply figure is from benchmarks, not from production-like routing pipelines. A microbenchmark with Rust actors + async-nats + mock providers is needed. [R3]

2. **Cascade latency against interactive SLOs**: No measured end-to-end cascaded pipeline latencies against sub-100 ms interactive SLOs across multi-provider API calls exist. [R3]

3. **Health threshold calibration**: No prescriptive health threshold values (numeric P95 cutoffs, failure-rate thresholds) are provided. These must be calibrated per-deployment and per-provider. [R3]

4. **Budget enforcement precision under concurrency**: CAS-based budget enforcement's <1% overrun claim needs validation under realistic concurrency bursts. [R3]

5. **kNN router performance on Mister Smith's actual query distribution**: kNN beats complex routers in benchmarks, but whether this holds for Mister Smith's specific workload (code-heavy, structured-output-heavy) is unknown.

6. **SLM sufficiency threshold per agent role**: Which of Mister Smith's 9 agent roles can run entirely on local SLMs, and which require API models? Requires empirical profiling.

### Needs More Research

1. **Token-level streaming acceptance in Rust + NATS**: No production-grade token-level streaming acceptance implementation exists in Rust/NATS. [R3]

2. **NATS-native auction implementation**: Market-based routing remains theoretical for NATS. No implementation guidance exists. [R3]

3. **PRM robustness outside mathematics**: Most PRM research is validated on math datasets. Step boundary definition for code generation, planning, and tool-use is less mature. DreamPRM-Code and ToolPRMBench are initial efforts. [R6]

4. **PrefillShare production viability**: The shared KV cache concept is frontier (2026). No production deployments documented. The data-plane transformation (JSON -> tensor pointers) is architecturally significant. [R7d]

5. **Multi-modal routing**: Most routing research focuses on text. As Mister Smith adds vision/audio support, routing must handle modality-specific complexity. [R4]

6. **Privacy constraints on routing signals**: Some routing approaches require sending the full query to a classifier. For sensitive data, routing must use metadata or anonymized features. [R4]

### Assumptions That May Be Wrong

1. **"More agent roles = better"**: Vercel's evidence (R7c) and Google's scaling laws (R7c) suggest this is false for many workloads. The default should be minimalism.

2. **"Cross-model MoA is better than self-consistency"**: Self-MoA outperforms cross-model MoA in most settings (R4: Li 2025, 21 citations).

3. **"Longer chain-of-thought = better reasoning"**: Optimal CoT length distribution exists per domain; excessive thinking impairs quality (R4 discovery: Yang 2025, 81 citations).

---

## Implementation Priority for Mister Smith

Ordered by ships first to ships last. Each tier builds on the previous.

### Tier 0: Foundation (Ships with Phase 9 MVP, 1-4 weeks)

| Component | Rationale | Evidence Strength |
|:---|:---|:---|
| **NATS request-reply routing** | Core data plane. Sub-100 us overhead target. | Proven (R3: all three reports) |
| **Deterministic heuristic router** | Captures 50-70% of traffic for zero cost. Token count, tool presence, modality, context size. | Proven (R3, R4) |
| **Queue group load balancing** | Built-in NATS primitive. Automatic failover on crash. | Proven (R3) |
| **Provider health monitor + circuit breaker** | Prerequisite for all advanced routing. Passive + active checks. | Proven (R3: Kong, Envoy, Azure) |
| **JetStream KV policy distribution** | Control plane for hot-reloading routing config, budgets, model registry. | Proven (R3) |
| **Request tracing** | OpenTelemetry spans across NATS messages. Per-request telemetry schema. | Proven (R3, existing Phase 8) |

**KPIs**: P99 routing overhead < 100 us. Automatic failover on provider failure. Zero message loss on crash.

### Tier 1: Cost Optimization (Ships 2-6 weeks after MVP)

| Component | Rationale | Evidence Strength |
|:---|:---|:---|
| **Hierarchical budget enforcement** | Prevents runaway spending. JetStream KV CAS. Reserve-then-reconcile. | Proven (R3) |
| **Budget-conditioned routing** | As budgets deplete, automatically shift to cheaper models. | Proven (R3) |
| **Local embedding-based semantic router** | ONNX embeddings, <50 ms latency, routes ~80% to Tier 1. | Strong (R3: FastEmbed blueprint) |
| **Synchronous fallthrough cascade** | Call small model, evaluate confidence, call larger if needed. | Strong (R3, R4: FrugalGPT, Zellinger) |
| **Abstention as first-class concept** | Small model explicitly declines -> triggers escalation. Reduces cost 13%, error rate 5%. | Strong (R4: Zellinger 2025) |
| **Semantic caching via JetStream KV** | ~10x lower latency for cache hits. Learning-based eviction. | Moderate-Strong (R4: ContextCache, Liu 2025) |

**KPIs**: Up to 85% cost reduction for simple queries. Budget enforcement <1% overrun rate. 10x latency improvement on cache hits.

### Tier 2: Learned Routing & Token Economics (Ships 6-12 weeks after MVP)

| Component | Rationale | Evidence Strength |
|:---|:---|:---|
| **kNN-based learned router** | Matches SOTA learned routers with lower complexity. | Strong (R4: Li 2025) |
| **RouteLLM-style BERT/ONNX router** | Sub-50 ms decisions. Up to 85% cost reduction. Trained on preference data. | Strong (R3, R4: RouteLLM, EMAFusion) |
| **SLM-default / LLM-fallback** | Local models for structured outputs. 10-100x cost reduction for routine tasks. | Strong (R4 discovery) |
| **Token budget prediction** | SelfBudgeter or Budget Guidance before dispatch. 48-61% fewer tokens. | Strong (R4: SelfBudgeter, 18 citations) |
| **CLAI cognitive load preprocessing** | Three-load taxonomy as difficulty estimator for routing and budgeting. | Moderate (R6, R4, R4 discovery) |
| **DAAO difficulty-aware orchestration** | VAE-based difficulty estimation with self-adjusting policy. | Moderate (R5, R4) |

**KPIs**: 2x+ cost reduction for classification-like tasks. 48-61% token reduction via budgeting.

### Tier 3: Step-Level Intelligence (Ships 12+ weeks, R&D)

| Component | Rationale | Evidence Strength |
|:---|:---|:---|
| **PRM-backed step verification** | BiPRM at 5% latency overhead for 37.7% better error detection. | Moderate (R6) |
| **RSD (start-cheap-escalate)** | 4.4x FLOP reduction. Draft model handles 65% of tokens. | Moderate (R6, R4 discovery: 63 citations) |
| **Streaming Content Monitors** | Detect failures at 18% of tokens. Mid-stream abort. | Moderate (R6) |
| **JetStream CAS micro-rollback** | Lock-free step-state management for mid-task model switching. | Design-level (R6) |
| **Offline contextual bandits** | Cobalt-style offline RL on logged trajectories. +9.0 Pass@1 improvement. | Moderate (R6) |
| **Speculative cascading** | Combining FrugalGPT cascades with speculative decoding. 2.8-5.8x speedups. | Moderate (R3, R4) |

**KPIs**: 70B-level quality at 1.5B-level cost. Micro-rollback without full task restart.

### Tier 4: Frontier / Experimental

| Component | Rationale | Evidence Strength |
|:---|:---|:---|
| **Shared KV cache (PrefillShare)** | Eliminates redundant prefill across multi-agent handoffs. | Frontier (R7d) |
| **Persistent KV cache to SSD** | 15.7s -> 0.6s agent resume latency. 4-bit quantized. | Frontier (R7c) |
| **Market-based / auction routing** | Dynamic pricing via JetStream KV. Posted-price as safe entry point. | Speculative (R3) |
| **Topology compiler (AdaptOrch)** | Dynamic topology selection based on task graph structure. | Promising (R7d) |
| **MoA for high-value async tasks** | 65.1% AlpacaEval 2.0 using only open-source models. | Strong quality, High latency cost (R3, R4) |

---

## Sources

### Primary Research Files

| File | Round | Type | Key Contribution |
|:---|:---|:---|:---|
| `synthesis/intelligent-model-routing-R3.md` | R3 | Triple synthesis of 3 industry reports | Two-plane architecture, staged roadmap, NATS patterns, budgets, health, cascading, MoA |
| `research/targeted-model-routing-cascades-R4.md` | R4 | Academic search (50+ papers, 2025 only) | Pareto frontiers, router architectures, speculative decoding, confidence calibration, token budgeting, semantic caching |
| `research/targeted-step-level-intelligence-R6.md` | R6 | Frontier deep dive | BiPRM, RSD, CLAI/TALE, streaming monitors, JetStream CAS rollback, step-level telemetry |
| `research/discovery-sweep-R4.md` | R4 | Discovery sweep (96 papers) | PRMs, CLAI, SLM-default, MaAS, CRDTs, AgentOps |
| `research/discovery-sweep-R5.md` | R5 | Discovery sweep (974 screened, 50 included) | KB-aware routing, DAAO, KABB, FoA/VCVs, decentralized DAGs |
| `research/discovery-sweep-R7c.md` | R7 | Discovery (user-added) | Vercel fewer-is-more, Google scaling laws, persistent KV cache, Rust agent frameworks |
| `research/discovery-sweep-R7d.md` | R7 | Discovery (user-added) | PrefillShare, disaggregated serving, AdaptOrch topology routing, MPST session types |

### Key Papers Referenced (Sorted by Citation Count)

| Paper | Citations | Finding |
|:---|:---|:---|
| Liu 2025 (0.5B vs GPT-4o) | 106 | Compute-optimal test-time scaling makes small models competitive |
| Yang 2025 (Thinking-Optimal) | 81 | Excessive CoT impairs reasoning; optimal length exists |
| Liao 2025 (RSD) | 63 | 4.4x FLOP reduction via PRM-gated speculative decoding |
| ARTIST (Singh 2025) | 37 | RL-trained agentic reasoning + tool integration |
| Li 2025 (Self-MoA) | 21 | Self-MoA outperforms cross-model MoA by 3.8-6.6% |
| Symbolic-MoE (Chen 2025) | 20 | Skill-based instance-level routing, +8.15% |
| R-PRM (She 2025) | 19 | Generative PRM, 11.9 F1 improvement |
| LLM Bandit (Li 2025) | 18 | MAB formulation generalizing to unseen LLMs |
| SelfBudgeter (Li 2025) | 18 | 48-61% response length compression, negligible accuracy loss |
| EvalTree (Zeng 2025) | 13 | Capability trees for weakness-guided profiling |
| ModelSwitch (Chen 2025) | 13 | Consistency-based switching between models |
| Model-SAT (Zhang 2025) | 11 | Capability instruction tuning, 50 tasks x 20 shots profiling |
| Budget Guidance (Li 2025) | 10 | Gamma distribution predictor, +26% accuracy under tight budgets |
| Routing Survey (Varangot-Reille 2025) | 10 | Canonical routing taxonomy |
| BEST-Route (Ding 2025) | 9 | Model + sample count routing, 60% cost reduction |
| Avengers-Pro (Zhang 2025) | 8 | True Pareto frontier: +7% accuracy or 63% cost reduction |
| SpecServe (Huang 2025) | 8 | SLO-aware speculative decoding, 1.14-14.3x speedups |
