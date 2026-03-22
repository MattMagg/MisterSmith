---
version: R8
created: 2026-03-22
type: prompt
tier: 1
timeline: last 2 months (late January 2026 — present)
---

# Deep Research Prompt: LLM Routing & Inference Economics

## Context

Mister Smith is a first-class multi-agent orchestration operating system in Rust, built on NATS/JetStream and Erlang OTP-inspired supervision trees. It is model-agnostic and designed to define the standard that the agent framework market will converge toward.

Phase 9 (LLM Providers) shipped a two-plane model router with health-aware circuit breakers, budget enforcement via JetStream KV CAS, SLM-default routing, and dual-stream formalization. The architecture is operational. The research question has shifted from "what to build" to "what has changed in the landscape that should influence the next iteration."

## Frontier-First Mandate

Do not choose an approach because it is popular, familiar, or already normalized by existing agent frameworks. Benchmark them. Learn from them. Then exceed them. Pull from distributed systems, ad-tech bidding, CDN routing, trading systems, and telecom switching when those fields offer stronger patterns.

Incremental imitation is failure. Favor well-reasoned designs that create real advantage.

## Research Objective

Survey everything published in the last ~2 months (late January 2026 to present) on LLM model routing, inference cost optimization, speculative decoding, token budgeting, and step-level intelligence. The goal is to discover what has changed since our last deep research round (early March 2026) and identify techniques that should influence Mister Smith's routing architecture.

This is an open-ended research task. Go beyond the dimensions listed below if you discover promising leads outside them.

## What Has Already Been Researched (Baseline — Do Not Rediscover)

The following are established findings from 7 research rounds (2,000+ papers). Treat these as known. Only surface new work on these topics if it significantly contradicts, extends, or supersedes them.

**Two-Plane Router**: Microsecond data plane (NATS request-reply ~50us) + control plane (JetStream KV watches). Validated by production gateways (Bifrost 11us at 5,000 RPS). Budget enforcement via JetStream KV atomic CAS with hierarchical tracking (org→team→user→tag).

**Learned Routing**: RouteLLM (27-85% cost savings), kNN matches complex routers, tiered classifier pipeline (rules → embeddings → ML → LLM slow path). Avengers-Pro (+7% accuracy at 27% cost reduction), BEST-Route (60% cost reduction <1% drop), SpareLLM (8.6x savings), EMAFusion (94.3% at 4x lower cost), LLM Bandit (MAB with user preferences).

**SLM-Default/LLM-Fallback**: 1-12B models match larger models at 10-100x lower cost for structured tasks. 0.5B outperforms GPT-4o with compute-optimal scaling (Liu 2025, 106 citations). Optimal CoT length exists per domain (Yang 2025, 81 citations).

**Step-Level Routing**: RSD achieves 4.4x FLOP reduction via start-cheap-escalate (Liao 2025, 63 citations). BiPRM 37.7% better error detection at 5% latency cost. R-PRM self-improving generative PRM. Streaming Content Monitors detect failures at 18% of tokens. PRM calibration via quantile regression required for MCTS-style search.

**Token Budgeting**: TALE (67% token cost reduction), CLAI cognitive load taxonomy (45% reduction), SelfBudgeter (61% response compression via RL), Budget Guidance (+26% accuracy under tight budgets via Gamma distribution).

**Health-Aware Routing**: Circuit breakers, phi accrual failure detection adapted for Inter-Token Latency, P2C+EWMA load balancing, penalty box outlier detection.

## Research Dimensions

### 1. New Routing Algorithms and Classifiers
- Have any new learned router architectures emerged that beat kNN or RouteLLM baselines?
- Are there new confidence estimation methods for routing decisions (beyond logit-based, Self-REF, verbalized)?
- What advances exist in training-free or zero-shot routing that eliminates the need for preference data?
- Has anyone built a routing classifier that operates at sub-microsecond latency in Rust or C++?
- Are there new theoretical frameworks for optimal routing (information-theoretic bounds, regret analysis)?

### 2. Speculative Decoding and Step-Level Intelligence
- What new PRM architectures or training methods have appeared since BiPRM/R-PRM?
- Have there been advances in KV cache transfer for mid-task model switching (reducing the 10GB overhead for LLaMA-2 70B at 4K context)?
- Are there new speculative decoding variants specifically designed for multi-agent settings?
- Has anyone combined PRMs with MCTS/tree search in production at scale?
- What new step boundary detection methods exist beyond CoT entropy?

### 3. SLM Economics and Guided Decoding
- What new small language models (<12B) have been released with strong structured output capabilities?
- Have there been advances in constrained decoding (beyond XGrammar/Outlines)?
- Are there production reports of SLM-default routing in real multi-agent workloads?
- What is the current frontier for model distillation targeting agent task formats?
- Has anyone demonstrated sub-1B models that reliably handle agentic tool-calling?

### 4. Inference Infrastructure Evolution
- What changes to vLLM, SGLang, TensorRT-LLM, or new engines affect routing architecture?
- Advances in disaggregated serving, KV cache sharing, or PrefillShare-style optimizations?
- New hardware-aware scheduling or NUMA-aware inference that changes the latency calculus?
- Rust-native inference runtime developments (candle, burn, ort bindings)?
- Are there new continuous batching or scheduling algorithms that affect how routing decisions interact with inference queues?

### 5. Cost Optimization at Multi-Agent Scale
- New production data on routing cost savings in systems running >10 agents?
- Advances in token budgeting, cognitive load estimation, or CoT length optimization?
- New hierarchical budget enforcement patterns for multi-tenant platforms?
- Economic models for LLM inference pricing and cost prediction?
- Are there new approaches to cost attribution and chargeback in multi-agent pipelines where multiple models contribute to a single output?

### 6. Cross-Domain Routing Patterns
- New applications of ad-tech bidding, CDN routing, trading systems, or telecom switching to model selection?
- Auction-based or market-based mechanisms for LLM provider allocation?
- Techniques from network routing (BGP, OSPF, SDN) applied to model routing?
- Has anyone applied queueing theory (M/G/1, priority queues, fair scheduling) to LLM request routing with empirical validation?
- Are there techniques from database query optimization (cost-based planning, adaptive query execution) that transfer to model routing decisions?

### 7. Emerging Threats to Current Architecture
- Evidence that the two-plane design is suboptimal for new inference patterns (e.g., very long context, multimodal, multi-turn)?
- New model architectures (MoE routing, early-exit transformers) that change the routing calculus?
- Developments that make learned routing obsolete (e.g., universal models that eliminate the need to route)?
- Are frontier models converging in capability to the point where routing provides diminishing returns?
- Do new pricing models (per-token, per-request, subscription, reserved capacity) change the economic assumptions underlying cost-optimized routing?

## Per-Dimension Output Structure

For each research dimension, provide:

1. **Current state of the art** — what exists today, with specific citations (authors, year, venue, DOI/URL if available)
2. **Key techniques** — the specific algorithms, architectures, or patterns discovered
3. **Applicability to Rust + NATS** — how well does each technique transfer to a Rust actor system with NATS messaging?
4. **Delta from baseline** — what is genuinely NEW versus what we already know?
5. **Implementation complexity** — rough assessment of effort and prerequisites
6. **Expected impact** — what improvement does this offer over the current Mister Smith router?

## Synthesis

After completing all dimensions, provide a synthesis that:
- Ranks the top 5 findings by strategic value for Mister Smith
- Identifies which current architectural assumptions are challenged
- Recommends specific next actions (prototype, benchmark, adopt, monitor)
- Notes any dimension that yielded thin results (say so rather than padding)

## Research Methodology

1. Search broadly across the last ~2 months (late January 2026 to present). Include arXiv preprints, conference proceedings, blog posts, GitHub releases, and industry reports.
2. Follow promising leads with targeted deep dives — do not stop at the first result
3. Look beyond agent frameworks into adjacent fields (trading, telecom, CDN, ad-tech) for transferable patterns
4. For each technique, assess whether it has been validated in production or is purely academic
5. Be skeptical of marketing claims — look for benchmarks, papers, and real-world results
6. If a dimension yields thin results, say so rather than padding with speculation
7. Cross-reference against the baseline above — only surface work that genuinely extends what we know
