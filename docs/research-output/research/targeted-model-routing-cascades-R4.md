---
version: R4
created: 2026-03-07
updated: 2026-03-07
sources: Consensus (50+ papers, 20+ searches)
round: 4 (Academic Search)
---

# Research Digest: LLM Model Routing, Cascade Optimization, and Mixture-of-Agents

**Generated**: 2026-03-07
**Source**: Consensus Academic Search (200M+ papers)
**Filter**: year_min=2025 only (cutting-edge research)
**Scope**: 20+ targeted queries across routing, cascading, MoA, speculative decoding, and related domains

---

## Table of Contents

1. [LLM Routing Optimization and Cost-Quality Pareto Frontiers](#1-llm-routing-optimization-and-cost-quality-pareto-frontiers)
2. [Cascade Inference and Hierarchical Escalation](#2-cascade-inference-and-hierarchical-escalation)
3. [Mixture-of-Agents and Multi-LLM Ensemble Methods](#3-mixture-of-agents-and-multi-llm-ensemble-methods)
4. [Router Architectures and Learning Strategies](#4-router-architectures-and-learning-strategies)
5. [Difficulty Estimation and Query Complexity Prediction](#5-difficulty-estimation-and-query-complexity-prediction)
6. [Test-Time Compute Scaling and Best-of-N Strategies](#6-test-time-compute-scaling-and-best-of-n-strategies)
7. [Speculative Decoding and Collaborative Inference](#7-speculative-decoding-and-collaborative-inference)
8. [Confidence Calibration, Abstention, and Reliability Guarantees](#8-confidence-calibration-abstention-and-reliability-guarantees)
9. [Multi-Agent Debate, Verification, and Self-Correction](#9-multi-agent-debate-verification-and-self-correction)
10. [Token Budget Optimization and Adaptive Reasoning Depth](#10-token-budget-optimization-and-adaptive-reasoning-depth)
11. [Semantic Caching and Response Reuse](#11-semantic-caching-and-response-reuse)
12. [Model Capability Profiling and Benchmarking](#12-model-capability-profiling-and-benchmarking)
13. [Latency-Aware Serving and SLO Optimization](#13-latency-aware-serving-and-slo-optimization)
14. [Tool Calling and Agentic Orchestration](#14-tool-calling-and-agentic-orchestration)
15. [Surveys and Taxonomies](#15-surveys-and-taxonomies)
16. [Emerging Directions](#16-emerging-directions)
17. [Synthesis: Implications for Mister Smith](#17-synthesis-implications-for-mister-smith)

---

## 1. LLM Routing Optimization and Cost-Quality Pareto Frontiers

### Avengers-Pro: Beyond GPT-5 via Performance-Efficiency Optimized Routing
- **Authors**: Zhang, Li, Chen, Zhang, Ye, Bai, Hu (2025)
- **Citations**: 8
- **Key Finding**: A test-time routing framework that embeds and clusters incoming queries, then routes each to the most suitable model based on a performance-efficiency score. Across 8 leading models (GPT-5-medium, Gemini-2.5-pro, Claude-opus-4.1), it achieves a true Pareto frontier: +7% accuracy over the strongest single model, or equivalent accuracy at 27% lower cost, or ~90% accuracy at 63% lower cost.
- **Relevance to Mister Smith**: Demonstrates the concrete value proposition of model routing -- Mister Smith's `ModelProvider` trait and routing layer can achieve similar Pareto optimality by embedding queries and routing to the cheapest model that meets a quality threshold.

### LLM Bandit: Preference-Conditioned Dynamic Routing
- **Authors**: Li (2025)
- **Citations**: 18
- **Key Finding**: Formulates LLM selection as a multi-armed bandit problem with user-specified preferences at inference time. The selection policy generalizes to unseen LLMs, critical for adapting to new models as they emerge.
- **Relevance to Mister Smith**: The bandit formulation maps directly to Mister Smith's agent roles -- different agent types (Coordinator, Analyst, Implementer) could express different cost-quality preferences, and the framework can adapt to new providers without retraining.

### BEST-Route: Adaptive Routing with Test-Time Optimal Compute
- **Authors**: Ding, Mallick, Zhang, Wang et al. (2025)
- **Citations**: 9
- **Key Finding**: Introduces the insight that for small models, generating multiple responses and selecting the best can be cheaper than a single large-model response. Routes queries and chooses both the model AND the number of samples to generate, achieving up to 60% cost reduction with <1% performance drop.
- **Relevance to Mister Smith**: The "model + sample count" routing decision is more expressive than simple model selection. Mister Smith could implement this via its existing `AgentRuntime` -- spawning N parallel completions from a cheap model when the router estimates that approach is cost-optimal.

### SpareLLM: Task-Specific Minimum-Cost Selection under Equivalence Constraints
- **Authors**: Jo, Trummer (2025)
- **Citations**: 1
- **Journal**: Proceedings of the ACM on Management of Data
- **Key Finding**: Users specify an "equivalence constraint" (tolerance for deviation from the most powerful LLM), and SpareLLM profiles multiple LLMs during a calibration phase to identify those meeting the threshold. Achieves up to 8.6x cost savings while generating equivalent outputs 90% of the time. Accounts for 91.1% of points on the Pareto curve.
- **Relevance to Mister Smith**: The profiling-then-routing paradigm is well-suited to Mister Smith's bootstrapped environment. During initialization, the system could profile available providers against representative queries, then route optimally during operation.

---

## 2. Cascade Inference and Hierarchical Escalation

### Cost-Saving LLM Cascades with Early Abstention
- **Authors**: Zellinger, Liu, Thomson (2025)
- **Citations**: 5
- **Key Finding**: In LLM cascades (small model first, escalate to large model if needed), allowing early models to also abstain from answering reduces overall cost by 13.0% and error rate by 5.0%, exploiting the correlation between error patterns of small and large models. Trades 4.1% increase in abstention rate for these gains.
- **Relevance to Mister Smith**: Abstention is a first-class concept for the cascade pipeline. Mister Smith's agent supervision tree could implement abstention as an explicit state in the AgentRuntime, triggering escalation to a more capable provider or human review.

### Cascaded Language Models for Cost-Effective Human-AI Decision-Making
- **Authors**: Fanconi, Schaar (2025)
- **Citations**: 1
- **Key Finding**: A three-tier cascade: base model -> large model -> human expert. A deferral policy decides base-vs-large, then an abstention policy decides LLM-vs-human. Includes online learning from human feedback to adapt policies to changing task difficulty over time.
- **Relevance to Mister Smith**: The three-tier architecture (cheap LLM -> expensive LLM -> human) maps precisely to Mister Smith's supervision philosophy. The online adaptation mechanism is critical for production systems where task distributions shift.

### EMAFusion: Self-Optimizing LLM Selection and Integration
- **Authors**: Shah, Shridhar, Chatterjee, Sen (2025)
- **Citations**: 0
- **Key Finding**: Combines a taxonomy-based router for familiar queries, a learned router for ambiguous inputs, and a cascading approach that progressively escalates from cheaper to more expensive models based on multi-judge confidence. Achieves 94.3% accuracy at 4x lower cost than average; 17.1 percentage point improvement over GPT-4 at 1/20th cost.
- **Relevance to Mister Smith**: The hybrid routing architecture (rules + learned model + cascade fallback) is a pragmatic design that Mister Smith could implement incrementally -- start with taxonomy-based routing, add learned routing later.

---

## 3. Mixture-of-Agents and Multi-LLM Ensemble Methods

### Rethinking Mixture-of-Agents: Self-MoA
- **Authors**: Li, Lin, Xia, Jin (2025)
- **Citations**: 21
- **Key Finding**: Surprisingly, Self-MoA (aggregating outputs from ONLY the single top-performing LLM, not multiple different LLMs) outperforms standard MoA that mixes different LLMs by 6.6% on AlpacaEval 2.0 and 3.8% average across benchmarks. MoA performance is very sensitive to output quality, and mixing different LLMs often lowers average quality. Identifies specific scenarios where mixing IS helpful.
- **Relevance to Mister Smith**: Critical insight -- naive multi-model ensemble can degrade quality. Mister Smith's orchestrator should default to self-consistency (multiple samples from the best model) and only mix models when there's evidence of complementary strengths.

### Mixture-of-Search-Agents (MoSA): Multi-LLM Collaborative Search
- **Authors**: Yang, Li, Lam, Cheng (2025)
- **Citations**: 7
- **Key Finding**: Uses MCTS as a backbone for multiple LLMs to propose and aggregate reasoning steps. Combines independent exploration with iterative refinement among LLMs, consistently improving over single-agent and multi-agent baselines in mathematical and commonsense reasoning.
- **Relevance to Mister Smith**: The MCTS + multi-model architecture maps well to Mister Smith's supervision trees. The Coordinator agent could implement MCTS-style search using different providers for exploration and refinement.

### Symbolic-MoE: Adaptive Skill-Based Routing
- **Authors**: Chen, Yun, Stengel-Eskin, Chen, Bansal (2025)
- **Citations**: 20
- **Key Finding**: Routes at the instance level (not task level) by emphasizing skills (e.g., "algebra" in math, "molecular biology" in biomedical). A skill-based recruiting strategy selects relevant expert LLMs, each generates reasoning, and an aggregator synthesizes a final response. Beats GPT-4o-mini and multi-agent approaches by 8.15% avg. Efficient batch strategy enables 16 expert models on 1 GPU.
- **Relevance to Mister Smith**: Skill-based instance-level routing is highly relevant for Mister Smith's 9 agent roles, each with distinct capabilities. The batch strategy for model loading is a practical engineering insight for the runtime.

### Mixture of Thoughts (MoT): Latent-Level Collaboration
- **Authors**: Fein-Ashley, Parikh, Kannan, Prasanna (2025)
- **Citations**: 0
- **Key Finding**: Instead of aggregating text outputs, MoT performs latent-level collaboration -- a lightweight router selects top-K experts, interaction layers project hidden states into a shared latent space for cross-attention. Surpasses Avengers (state-of-the-art) by +0.38% ID and +2.92% OOD, with single-pass inference and no iterative aggregation overhead.
- **Relevance to Mister Smith**: Novel but requires model internals access. Relevant if Mister Smith supports self-hosted open-source models where hidden states are available. Not applicable to API-only providers.

### ModelSwitch: Multi-LLM Repeated Sampling
- **Authors**: Chen, Xun, Zhou, Qi et al. (2025)
- **Citations**: 13
- **Key Finding**: Builds on repeated-sampling-then-voting but incorporates multiple models (even weaker ones) to leverage complementary strengths. Uses consistency as a signal to dynamically switch between models. Outperforms self-consistency and multi-agent debate while reducing costs. Only needs a few comparable LLMs.
- **Relevance to Mister Smith**: A lightweight alternative to full MoA. The consistency-based switching signal is easy to implement in Mister Smith's event-driven architecture -- publish candidate responses on NATS, vote on consistency.

### Industrial MoA for Code Optimization
- **Authors**: Ashiga, Voskanyan et al. (2025)
- **Citations**: 1
- **Key Finding**: First MoA application to industrial code optimization. MoA excels with open-source models, achieving 14.3%-22.2% cost savings and 28.6%-32.2% faster optimization. GA-based ensemble better with commercial models.
- **Relevance to Mister Smith**: Validates MoA for code-related tasks, directly relevant to Mister Smith's Implementer and Reviewer agent roles.

---

## 4. Router Architectures and Learning Strategies

### Router-R1: RL-Based Multi-Round Routing and Aggregation
- **Authors**: Zhang, Feng, You (2025)
- **Citations**: 5
- **Key Finding**: Formulates routing as a sequential decision process using RL. The router itself is an LLM that interleaves "think" (deliberation) and "route" (model invocation) actions, integrating responses into evolving context. Uses cost rewards alongside outcome rewards. Generalizes to unseen models using simple descriptors (pricing, latency, example performance).
- **Relevance to Mister Smith**: The "router-as-LLM" with sequential decision-making is a natural fit for Mister Smith's Coordinator agent. The generalization to unseen models via descriptors aligns with the `ModelProvider::capabilities()` trait method.

### HierRouter: Hierarchical Routing via MDP
- **Authors**: Gupta, Kannan, Prasanna (2025)
- **Citations**: 0
- **Key Finding**: Formulates multi-hop inference as a finite-horizon MDP, training a PPO-based RL agent to iteratively select models at each stage. Conditions on evolving context and accumulated cost. Improves response quality by up to 2.4x over individual models.
- **Relevance to Mister Smith**: The hierarchical MDP formulation maps to Mister Smith's supervision tree structure. Multi-hop inference is exactly what the Coordinator agent does when decomposing complex tasks.

### ICL-Router: In-Context Learned Model Representations
- **Authors**: Wang, Li, Zhang et al. (2025)
- **Citations**: 0
- **Key Finding**: Uses in-context vectors to represent model capabilities, enabling seamless integration of new models without retraining the router. Two-stage approach: embed queries, then learn model capability profiles.
- **Relevance to Mister Smith**: The zero-retraining integration of new models is critical for Mister Smith's goal of being model-agnostic. New providers can be added dynamically with just a profiling step.

### kNN Beats Complex Learned Routers
- **Authors**: Li (2025)
- **Citations**: 1
- **Key Finding**: A well-tuned kNN approach matches or outperforms state-of-the-art learned routers across diverse tasks. Locality properties of model performance in embedding space enable simple non-parametric methods to achieve strong routing with lower sample complexity.
- **Relevance to Mister Smith**: Critical engineering insight -- start with simple kNN routing before investing in complex architectures. Mister Smith's initial routing implementation should benchmark kNN before building anything more sophisticated.

### Model-SAT: Capability Instruction Tuning for Routing
- **Authors**: Zhang, Zhan, Ye (2025)
- **Citations**: 11
- **Key Finding**: Constructs "capability instructions" with model capability representations, user instructions, and performance inquiry prompts. A model capability encoder extends its representation to a lightweight LLM. New models can be profiled via 50 tasks x 20 shots without full retraining. SOTA routing without candidate inference.
- **Relevance to Mister Smith**: The "aptitude test" paradigm for model profiling is practical. During Mister Smith's bootstrap, run each provider through a standardized capability test, then route based on results.

### IRT-Router: Item Response Theory for LLM Routing
- **Authors**: Song, Huang et al. (2025)
- **Citations**: 7
- **Key Finding**: Applies Item Response Theory (from psychometrics) to model the relationship between LLM capabilities and query attributes. Provides interpretable insights (model ability scores, query difficulty estimates). Online query warm-up via semantic similarity enhances cold-start performance. Tested on 20 LLMs and 12 datasets.
- **Relevance to Mister Smith**: IRT provides a principled, interpretable framework for capability modeling. The ability/difficulty decomposition maps cleanly to Mister Smith's config system -- each provider gets an ability score per domain, each query gets a difficulty estimate.

### MA-Router: Multi-Modal Attention Routing with RL
- **Authors**: Shan, Xi, Zhang, Liu (2025)
- **Citations**: 0
- **Key Finding**: Combines DeBERTa-v3 semantic encoding with GNN-based syntactic structure analysis and cross-attention fusion. Hierarchical RL agent dynamically adjusts routing threshold. Reduces GPT-4 call rate by 32.7% while maintaining 95.2% response quality.
- **Relevance to Mister Smith**: Demonstrates that multi-signal routing (semantic + syntactic + historical performance) significantly outperforms single-signal approaches. Mister Smith's router should consider multiple query features.

---

## 5. Difficulty Estimation and Query Complexity Prediction

### DAAO: Difficulty-Aware Agentic Orchestration
- **Authors**: Su, Lan, Xia et al. (2025)
- **Citations**: 0
- **Key Finding**: Dynamically generates query-specific multi-agent workflows guided by predicted query difficulty. Uses a VAE for difficulty estimation, a modular operator allocator, and a cost/performance-aware LLM router. Self-adjusting policy updates difficulty estimates based on workflow success.
- **Relevance to Mister Smith**: Directly maps to Mister Smith's orchestrator pattern. The VAE-based difficulty estimator could run as a lightweight preprocessing step before the Coordinator agent allocates work. The self-adjusting policy aligns with Mister Smith's supervision and health monitoring.

### Cognitive Load-Aware Inference (CLAI)
- **Authors**: Zhang (2025)
- **Citations**: 2
- **Key Finding**: Operationalizes Cognitive Load Theory for LLM inference, formalizing intrinsic, extraneous, and germane cognitive load as quantifiable metrics. Reframes inference as a cognitive economics optimization problem. Achieves up to 45% token reduction without accuracy loss. CLAI-Tune exhibits emergent problem decomposition.
- **Relevance to Mister Smith**: The cognitive load decomposition provides a principled framework for the Analyst agent to estimate query complexity. The three-load model (intrinsic difficulty, overhead, productive reasoning) maps to routing decisions.

---

## 6. Test-Time Compute Scaling and Best-of-N Strategies

### Sample Complexity of Test-Time Scaling Paradigms
- **Authors**: Huang, Li, Wu, Yang, Talwalkar, Ramchandran, Jordan, Jiao (2025)
- **Citations**: 1
- **Key Finding**: Establishes a formal separation: self-consistency requires Theta(1/Delta^2) samples while best-of-N only needs Theta(1/Delta), where Delta is the probability gap. Self-correction with verifier feedback enables Transformers to simulate online learning over expert pools at test time.
- **Relevance to Mister Smith**: Provides theoretical grounding for choosing between voting and best-of-N strategies. Mister Smith should prefer best-of-N with a verifier over majority voting, especially when the probability gap is small.

### CarBoN: Calibrated Best-of-N Sampling
- **Authors**: Tang, Chen, Cavallaro (2025)
- **Citations**: 0
- **Key Finding**: Introduces test-time calibration for best-of-N sampling -- learns input-specific temperature and additive shift to guide generation toward high-reward reasoning paths. Achieves 4x fewer rollouts to reach the same accuracy. Theoretical guarantees on improving expected reward lower bound.
- **Relevance to Mister Smith**: If using best-of-N with open-source models, calibrated sampling can dramatically reduce cost. Could be integrated into Mister Smith's `ModelProvider::complete()` by adjusting generation parameters per-query.

---

## 7. Speculative Decoding and Collaborative Inference

### CoSine: Collaborative Speculative Inference
- **Authors**: Gao, Liu, Xu, Huang (2025)
- **Citations**: 3
- **Key Finding**: Decouples sequential speculative decoding from parallel verification across multiple nodes. Routes requests to specialized drafters based on expertise, uses confidence-based token fusion. Achieves 23.2% latency decrease and 32.5% throughput increase.
- **Relevance to Mister Smith**: The multi-node collaborative decoding maps to Mister Smith's NATS-based distributed architecture. Drafter specialization per domain is analogous to agent role specialization.

### Mirror Speculative Decoding: Breaking the Serial Barrier
- **Authors**: Bhendawade, Nishu et al. (2025)
- **Citations**: 0
- **Key Finding**: Breaks the latency-acceptance tradeoff by running draft and target models in dual complementary pipelines across heterogeneous accelerators. Achieves 2.8x-5.8x wall-time speedups on 14B-66B models, 30% improvement over EAGLE3.
- **Relevance to Mister Smith**: Demonstrates that heterogeneous hardware utilization is key for speculative inference. Mister Smith's runtime could schedule draft models on different compute tiers.

### Confidence-Modulated Speculative Decoding
- **Authors**: Sen, Dasgupta, Waghela (2025)
- **Citations**: 1
- **Key Finding**: Uses entropy and margin-based uncertainty measures to dynamically adjust draft length per iteration. Reduces rollback frequency, improves resource utilization. Plug-in method compatible with any LLM.
- **Relevance to Mister Smith**: The plug-in nature makes this adoptable without modifying the underlying `ModelProvider` trait. Could be implemented as middleware between the agent and the provider.

### Collaborative Decoding via Speculation (CoS)
- **Authors**: Fu, Jiang et al. (2025)
- **Citations**: 5
- **Key Finding**: Generalizes speculative decoding to N-model collaboration. Proves CoS is never slower than standard collaborative decoding. Alternating each model as proposer and verifier enhances efficiency. 1.11x-2.23x faster than standard collaborative decoding.
- **Relevance to Mister Smith**: The formal guarantee ("never slower") makes this a safe default strategy for multi-model inference pipelines in Mister Smith.

### SpecServe: SLO-Aware Speculative Decoding
- **Authors**: Huang, Wu et al. (2025)
- **Citations**: 8
- **Key Finding**: Dynamically adjusts speculative strategies based on real-time request loads. Proposes a theoretical model for predicting speculative decoding efficiency. Achieves 1.14x-14.3x speedups over state-of-the-art while meeting SLOs.
- **Relevance to Mister Smith**: The SLO-awareness is critical for production deployment. Mister Smith's health probes and monitoring infrastructure could feed load signals to an adaptive speculative strategy.

---

## 8. Confidence Calibration, Abstention, and Reliability Guarantees

### SteerConf: Steering LLMs for Confidence Elicitation
- **Authors**: Zhou, Jin, Shi, Li (2025)
- **Citations**: 4
- **Key Finding**: Guides LLMs to produce confidence scores in specified directions via steering prompts. Measures consistency across steered confidences for calibration. No training or fine-tuning required. Significantly outperforms existing calibration methods.
- **Relevance to Mister Smith**: Zero-training confidence elicitation is immediately applicable. Mister Smith agents can request confidence scores from any provider, then use consistency measures for routing and abstention decisions.

### Learnable Conformal Abstention Policies
- **Authors**: Tayebati, Kumar et al. (2025)
- **Citations**: 6
- **Key Finding**: Integrates RL with conformal prediction to dynamically optimize abstention thresholds. Improves accuracy by up to 3.2%, AUROC for hallucination detection by 22.19%, while meeting 90% coverage target.
- **Relevance to Mister Smith**: Conformal prediction provides statistical guarantees that Mister Smith's supervision system can enforce. The RL-based threshold optimization can adapt to changing provider characteristics.

### CCPO: Conformal Constrained Policy Optimization
- **Authors**: Si, Jang, Lee, Bastani (2025)
- **Citations**: 0
- **Key Finding**: Combines multiple LLMs with varying cost/accuracy tradeoffs in an agentic manner, with conformal prediction guarantees on reliability. Achieves up to 30% cost reduction without compromising reliability.
- **Relevance to Mister Smith**: Provides a principled framework for cost optimization with formal reliability guarantees -- exactly what a production multi-agent system needs.

---

## 9. Multi-Agent Debate, Verification, and Self-Correction

### MARS: Multi-Agent Review System
- **Authors**: Wang, Wang et al. (2025)
- **Citations**: 0
- **Key Finding**: Replaces expensive round-table debate with an author-reviewer-meta-reviewer pattern (inspired by peer review). Matches MAD accuracy while reducing token usage and inference time by ~50%.
- **Relevance to Mister Smith**: The author/reviewer/meta-reviewer roles map directly to Mister Smith's agent role system. The Implementer generates, the Reviewer evaluates, the Coordinator (meta-reviewer) decides. 50% cost reduction over debate is significant.

### MACI: Dual-Dial Control for Multi-Agent Debate
- **Authors**: Chang, Chang (2025)
- **Citations**: 1
- **Key Finding**: Introduces two independent dials: an information dial (gates evidence by quality) and a behavior dial (schedules contentiousness from exploration to consolidation). Provides provable termination and nonincreasing dispersion guarantees. Budget-feasible scheduler.
- **Relevance to Mister Smith**: The provable termination guarantee is critical for production systems. Mister Smith's supervision tree needs guaranteed convergence, not open-ended debate. The dual-dial mechanism maps to observable metrics in the monitoring system.

### SPOC: Spontaneous Self-Correction
- **Authors**: Zhao, Xu et al. (2025)
- **Citations**: 2
- **Key Finding**: Enables interleaved solution and verification in a single inference pass, with dynamic termination based on verification outcomes. Multi-agent perspective (proposer + verifier) in the same model. Boosts Llama-3.1-8B accuracy by 8.8% on MATH500.
- **Relevance to Mister Smith**: Single-pass self-correction can be requested via the `ModelProvider` interface using structured prompts. No multi-model overhead.

### S2R: Self-Verify and Self-Correct via RL
- **Authors**: Ma, Wang et al. (2025)
- **Citations**: 8
- **Key Finding**: With only 3.1k behavior initialization samples, Qwen2.5-math-7B accuracy jumps from 51.0% to 81.6%. Uses both outcome-level and process-level RL. Outperforms models trained on equivalent long-CoT distilled data.
- **Relevance to Mister Smith**: Demonstrates that self-correction can be trained cheaply. If Mister Smith supports fine-tuned models, S2R-style training can dramatically improve smaller model quality, reducing the need to route to expensive models.

---

## 10. Token Budget Optimization and Adaptive Reasoning Depth

### SelfBudgeter: Adaptive Token Allocation
- **Authors**: Li, Dong, Ma, Zhang, Sui (2025)
- **Citations**: 18
- **Key Finding**: Incorporates a budget estimation mechanism BEFORE reasoning. The model predicts how many tokens it will need, then is trained (via RL) to adhere to the budget. Achieves 61% response length compression on 1.5B model and 48% on 7B model, with nearly undiminished accuracy. Users see the budget estimate upfront.
- **Relevance to Mister Smith**: Budget prediction before generation enables proactive cost control. Mister Smith's Coordinator can estimate total cost before dispatching to agents, and enforce per-task budgets.

### Reasoning on a Budget: Survey of Adaptive Test-Time Compute
- **Authors**: Alomrani, Zhang et al. (2025)
- **Citations**: 5
- **Key Finding**: Comprehensive survey distinguishing L1-controllability (fixed budgets) and L2-adaptiveness (dynamic scaling based on difficulty/confidence). Benchmarks proprietary LLMs, identifies hybrid thinking models as emerging trend.
- **Relevance to Mister Smith**: Provides the taxonomy for Mister Smith's compute allocation strategy. L1 for budget-constrained environments, L2 for quality-first scenarios.

### Budget Guidance: Steering LLM Thinking
- **Authors**: Li, Zhao, Zhang, Gan (2025)
- **Citations**: 10
- **Key Finding**: A lightweight predictor models a Gamma distribution over remaining thinking length during generation. Guides reasoning to target budget without fine-tuning. +26% accuracy on MATH-500 under tight budgets. Exhibits emergent question difficulty estimation.
- **Relevance to Mister Smith**: No fine-tuning required -- can be implemented as a middleware layer in Mister Smith's provider infrastructure. The emergent difficulty estimation is a bonus for routing decisions.

### CoT-X: Cross-Model Chain-of-Thought Transfer
- **Authors**: Bi, Chen et al. (2025)
- **Citations**: 0
- **Key Finding**: Compresses reasoning traces via semantic segmentation with importance scoring for transfer across models of different scales. Up to 40% higher accuracy than truncation under same token budgets. Bayesian optimization reveals power-law relationship between model size and cross-domain robustness.
- **Relevance to Mister Smith**: Reasoning trace compression enables efficient cascade handoffs -- a small model's reasoning can be summarized and passed to a larger model when escalating, rather than starting from scratch.

---

## 11. Semantic Caching and Response Reuse

### Semantic Caching for Low-Cost LLM Serving
- **Authors**: Liu, Atalar et al. (2025)
- **Citations**: 0
- **Key Finding**: Principled, learning-based framework for semantic cache eviction accounting for mismatch costs between queries and cached responses. Formulates both offline optimization and online learning variants with provably efficient algorithms.
- **Relevance to Mister Smith**: Mister Smith's JetStream KV store is a natural backend for semantic caching. The learning-based eviction policy could be integrated with the existing `HybridStateManager`.

### ContextCache: Context-Aware Semantic Cache for Multi-Turn Queries
- **Authors**: Yan, Ni et al. (2025)
- **Citations**: 2
- **Key Finding**: Two-stage retrieval: vector-based retrieval on current query, then self-attention integration of current and historical dialogue for precise contextual matching. Cached responses exhibit ~10x lower latency than direct LLM invocation.
- **Relevance to Mister Smith**: Multi-turn awareness is critical for Mister Smith's agents, which maintain conversation context. The 10x latency improvement justifies the engineering investment in caching infrastructure.

### Ensemble Embedding for Semantic Caching
- **Authors**: Ghaffari, Bahranifard, Akbari (2025)
- **Citations**: 0
- **Key Finding**: Combines multiple embedding models through a trained meta-encoder for cache similarity detection. Achieves 92% hit ratio while correctly rejecting 85% of non-equivalent queries.
- **Relevance to Mister Smith**: If Mister Smith implements semantic caching, using ensemble embeddings for similarity detection reduces both false positive and false negative cache hits.

---

## 12. Model Capability Profiling and Benchmarking

### EvalTree: Profiling Weaknesses via Hierarchical Capability Trees
- **Authors**: Zeng, Wang, Hajishirzi, Koh (2025)
- **Citations**: 13
- **Key Finding**: Constructs a capability tree where each node represents a capability in natural language, linked to benchmark instances. Extracts nodes where the LM performs poorly to generate a weakness profile. Weakness-guided data collection improves LM performance more than other strategies.
- **Relevance to Mister Smith**: EvalTree-style profiling can inform routing decisions -- know where each provider is weak, and route away from those weaknesses. Can be integrated into Mister Smith's model registry.

### InferenceDynamics: Structured Capability and Knowledge Profiling
- **Authors**: Shi, Zheng et al. (2025)
- **Citations**: 5
- **Key Finding**: Models capability and knowledge dimensions of LLMs for scalable group-level routing. Demonstrates effectiveness on MMLU-Pro, GPQA, BigGenBench, and LiveBench. Designed for large pools of specialized LLMs.
- **Relevance to Mister Smith**: Multi-dimensional capability profiles per provider model can be stored in Mister Smith's persistence layer and queried at routing time.

### RouterArena: Comprehensive Router Comparison Platform
- **Authors**: Lu, Liu et al. (2025)
- **Citations**: 0
- **Key Finding**: First open platform for comparing LLM routers with broad domain coverage, distinguishable difficulty levels, extensive evaluation metrics, and automated leaderboard updates.
- **Relevance to Mister Smith**: Provides a benchmarking methodology for Mister Smith's own routing implementation. Can validate routing strategies against standardized baselines.

---

## 13. Latency-Aware Serving and SLO Optimization

### SeaLLM: Service-Aware Latency-Optimized Resource Sharing
- **Authors**: Zhao, Chen et al. (2025)
- **Citations**: 0
- **Key Finding**: Service-aware scheduling for multiple LLMs sharing GPUs, with unified KV cache sharing. Improves normalized latency by up to 13.6x and SLO attainment by up to 3.64x.
- **Relevance to Mister Smith**: While Mister Smith primarily uses API providers, the KV cache sharing concept applies if deploying open-source models. The SLO-aware scheduling informs how Mister Smith's health probes should trigger model switching.

### BrownoutServe: SLO-Aware MoE Serving under Bursty Workloads
- **Authors**: Hu, Xu et al. (2025)
- **Citations**: 0
- **Key Finding**: "United experts" integrate knowledge from multiple MoE experts to reduce inference latency. Dynamic brownout mechanism adaptively adjusts processing under load. Achieves 2.07x throughput improvement and reduces SLO violations by 90.28%.
- **Relevance to Mister Smith**: The brownout/degradation pattern maps to Mister Smith's `CircuitBreaker` and health monitoring. Under load, gracefully degrade to cheaper models rather than violating latency SLOs.

---

## 14. Tool Calling and Agentic Orchestration

### ARTIST: Agentic Reasoning and Tool Integration via RL
- **Authors**: Singh, Magazine, Pandya, Nambi (2025)
- **Citations**: 37
- **Key Finding**: Tightly couples agentic reasoning, RL, and tool integration. Models autonomously decide when, how, and which tools to invoke within multi-turn reasoning chains. Up to 22% absolute improvement over base models. RL training leads to deeper reasoning and more effective tool use.
- **Relevance to Mister Smith**: Directly relevant to Mister Smith's ToolBus <-> LLM function calling bridge. The RL-trained tool invocation strategy outperforms prompted approaches.

### NaviAgent: Graph-Navigated Bilevel Planning for Function Calling
- **Authors**: Jiang, Zhou et al. (2025)
- **Citations**: 0
- **Key Finding**: Constructs a Tool Dependency Heterogeneous Graph (TDHG) encoding API schema structure and historical invocation behavior. A heuristic search strategy guides efficient toolchain selection. Outperforms ReAct, ToolLLM by 13.5%-19.0%.
- **Relevance to Mister Smith**: The TDHG concept maps to Mister Smith's MCP tool registry. Building a dependency graph of tools enables more efficient multi-tool orchestration.

### PerfOrch: Multi-Stage Performance-Guided LLM Orchestration for Code
- **Authors**: Chen, Qi et al. (2025)
- **Citations**: 0
- **Key Finding**: Studies 17 LLMs across 5 programming languages, revealing pronounced performance heterogeneity by language, development stage, and problem category. Stage-wise validation and rollback mechanisms achieve 96.22% correctness (vs GPT-4o's 78.66%). Plug-and-play architecture.
- **Relevance to Mister Smith**: Validates that for code generation (directly relevant to Implementer agent), multi-model orchestration dramatically outperforms single-model approaches. The rollback mechanism maps to Mister Smith's supervision strategies.

### M1-Parallel: Parallel Multi-Agent Teams
- **Authors**: Zhang, Zhu et al. (2025)
- **Citations**: 2
- **Key Finding**: Concurrently runs multiple multi-agent teams in parallel using event-driven communication with asynchronous messaging. Achieves up to 2.2x speedup with early termination while preserving accuracy. Repeated sampling provides sufficient diversity without explicit diversification.
- **Relevance to Mister Smith**: The event-driven async messaging pattern is exactly Mister Smith's NATS-based architecture. Parallel team execution maps to spawning multiple agent instances.

---

## 15. Surveys and Taxonomies

### Doing More with Less: Survey on Routing Strategies
- **Authors**: Varangot-Reille, Bouvard et al. (2025)
- **Citations**: 10
- **Key Finding**: Comprehensive survey formalizing routing as a performance-cost optimization problem. Reviews when (pre-generation vs post-generation), why (cost, quality, latency), and how (similarity-based, supervised, RL-based, generative) to route. Identifies standardization, non-financial costs, and adaptive strategies as open challenges.
- **Relevance to Mister Smith**: This survey provides the canonical reference for Mister Smith's routing design. The pre-generation vs post-generation timing distinction is architecturally significant.

### Ensemble Learning for LLMs: A Survey
- **Authors**: Ashiga, Jie et al. (2025)
- **Citations**: 5
- **Key Finding**: Categorizes LLM ensembles into 7 methods: weight merging, knowledge fusion, mixture-of-experts, reward ensemble, output ensemble, routing, and cascading. Lays groundwork for extending to multimodal LLMs.
- **Relevance to Mister Smith**: Provides the complete taxonomy of ensemble strategies that Mister Smith could support. Weight merging and knowledge fusion require model internals; routing and cascading are API-compatible.

### Towards Efficient Multi-LLM Inference: Survey
- **Authors**: Behera, Champati et al. (2025)
- **Citations**: 4
- **Key Finding**: Surveys routing (single model selection) vs cascading/hierarchical inference (sequential escalation). Comparative analysis across key performance metrics. Identifies adaptive model selection based on task complexity as the critical open problem.
- **Relevance to Mister Smith**: Clarifies the routing-vs-cascading design space. Mister Smith should support both patterns, selectable per use case.

### Model Fusion: Comprehensive Review
- **Authors**: Zhou, Zhang et al. (2025)
- **Citations**: 1
- **Key Finding**: Classifies fusion into parameter-level merging and knowledge distillation-based fusion. Highlights challenges in model heterogeneity, semantic alignment, and scalability.
- **Relevance to Mister Smith**: Model fusion is a complementary strategy to routing. If Mister Smith supports self-hosted models, merged models could serve as strong default providers.

---

## 16. Emerging Directions

### 16.1 Conformal Prediction for LLM Reliability

A clear emerging trend: conformal prediction provides distribution-free statistical guarantees on LLM output quality, enabling principled abstention and cascade triggering. Three papers (CCPO, Learnable Conformal Abstention, SAFER) demonstrate its applicability to multi-model systems. This provides a formal foundation for Mister Smith's quality assurance -- rather than ad-hoc confidence thresholds, conformal prediction offers mathematically grounded coverage guarantees.

### 16.2 LLM-as-Router: The Router Is a Language Model

Router-R1 and MA-Router represent a paradigm shift where the router itself is an LLM that reasons about which model to call. This recursive architecture (LLM reasoning about LLMs) is computationally expensive but shows strong generalization. As smaller reasoning models improve, this overhead may become acceptable. Mister Smith's Coordinator agent is already positioned for this -- it can use a small, fast model for routing decisions.

### 16.3 Latent-Space Collaboration (Mixture of Thoughts)

MoT's approach of collaborating in latent/hidden-state space (rather than text output space) represents a potential paradigm shift. Current API-based providers don't expose hidden states, but the trend toward open-weight models (Llama, Mistral, Qwen) makes this increasingly feasible. If Mister Smith supports local model serving, latent-space collaboration could deliver ensemble quality at single-pass cost.

### 16.4 Edge-Cloud Hybrid Inference

CE-LSLM and FedHLM demonstrate architectures where edge-deployed SLMs handle easy queries locally, escalating to cloud LLMs only when uncertainty exceeds a threshold. FedHLM achieves 95% reduction in LLM transmissions. This edge-cloud pattern maps to Mister Smith's potential deployment topology where lightweight agents run on edge devices with NATS connectivity to cloud-hosted LLMs.

### 16.5 Self-Correction as an Alternative to Multi-Model Ensembles

SPOC, ReVISE, and S2R show that a single model can achieve ensemble-like quality improvements through self-correction mechanisms (interleaved generation and verification). S2R's 30-percentage-point accuracy jump from only 3.1k training samples suggests that self-correction is more cost-effective than multi-model orchestration for many use cases. This challenges the assumption that multi-model routing is always necessary.

### 16.6 Cognitive Load Theory Applied to LLMs

CLAI's operationalization of Cognitive Load Theory is a novel cross-disciplinary approach. Decomposing inference cost into intrinsic (problem difficulty), extraneous (wasted computation), and germane (productive reasoning) load provides a principled framework for budget allocation that goes beyond simple token counting.

### 16.7 Bayesian Routing for Reward Models

BayesianRouter's application of Thompson sampling to reward model selection demonstrates that routing concepts extend beyond generation models. In Mister Smith's context, this could apply to selecting between different evaluation/verification strategies for agent outputs.

---

## 17. Synthesis: Implications for Mister Smith

### 17.1 Architecture Recommendations

Based on this research, Mister Smith's Phase 9 LLM Provider integration should incorporate:

1. **Tiered Routing Architecture**: Start with taxonomy/rule-based routing for known query types, add kNN-based learned routing for ambiguous queries, and cascade to expensive models as a fallback (EMAFusion pattern). The kNN baseline is surprisingly competitive and should be the first implementation.

2. **Self-MoA as Default Ensemble**: When quality improvement is needed, default to Self-MoA (multiple samples from the best model with voting) rather than mixing different models. Only activate cross-model MoA when profiling data shows complementary strengths (Li et al., 2025).

3. **Difficulty-Aware Dispatch**: Integrate a lightweight difficulty estimator (inspired by DAAO's VAE or IRT-Router's psychometric model) into the Coordinator agent. Route easy queries to cheap models, hard queries to expensive ones, and very hard queries to multi-model ensembles.

4. **Abstention as a First-Class Concept**: Build abstention into the `ModelProvider` trait. When a model's confidence is below threshold, it should explicitly abstain rather than produce low-quality output. Cascade policies should handle abstention gracefully.

5. **Token Budget Prediction**: Before dispatching to a provider, estimate the token budget needed (SelfBudgeter pattern). This enables proactive cost control and SLO enforcement.

### 17.2 NATS Integration Points

- **Query embedding and routing**: Publish embedded queries on a NATS routing subject; router agents consume and assign to model-specific subjects.
- **Best-of-N voting**: Publish N candidate responses on a voting subject; an aggregator agent selects or synthesizes the best.
- **Cascade escalation**: Use NATS request-reply with timeouts for cascade tiers. If Tier 1 abstains within timeout, escalate to Tier 2.
- **Semantic caching**: Use JetStream KV as the cache backend with embedding-based lookup.
- **Performance telemetry**: Publish routing decisions and outcomes on a telemetry subject for online learning.

### 17.3 Key Quantitative Findings for Design Decisions

| Strategy | Cost Reduction | Quality Impact | Complexity |
|----------|---------------|----------------|------------|
| Pareto-optimal routing (Avengers-Pro) | 27-63% | 0% to +7% | Medium |
| Cascade with early abstention | 13% | +5% error reduction | Low |
| Best-of-N from cheap model (BEST-Route) | 60% | <1% drop | Low |
| Self-MoA (same model, multiple samples) | Varies | +3.8-6.6% improvement | Low |
| Symbolic-MoE (skill-based routing) | Varies | +8.15% vs multi-agent | Medium |
| SelfBudgeter token compression | N/A (48-61% fewer tokens) | Negligible | Medium |
| Semantic caching | 50-60% compute reduction | Comparable | Medium |
| MARS (review pattern vs debate) | 50% token reduction | Equivalent | Low |

### 17.4 Critical Open Questions

1. **Router overhead**: The router itself has a cost. For low-latency applications, the routing decision must be fast enough that savings exceed overhead.
2. **Provider volatility**: LLM providers update models frequently. Routing profiles become stale. Online learning (bandit-based approaches) addresses this but adds complexity.
3. **Multi-modal routing**: Most routing research focuses on text. As Mister Smith adds support for vision/audio providers, routing must handle modality-specific complexity.
4. **Privacy constraints**: Some routing approaches require sending the full query to a classifier. For sensitive data, the routing signal must be derived from metadata or anonymized features.

---

## Paper Index (Alphabetical by First Author)

| First Author | Title (Short) | Year | Citations | Section |
|-------------|---------------|------|-----------|---------|
| Alomrani | Reasoning on a Budget (Survey) | 2025 | 5 | 10 |
| Ashiga (code) | Industrial MoA for Code Optimization | 2025 | 1 | 3 |
| Ashiga (survey) | Ensemble Learning for LLMs (Survey) | 2025 | 5 | 15 |
| Behera | Efficient Multi-LLM Inference (Survey) | 2025 | 4 | 15 |
| Bhendawade | Mirror Speculative Decoding | 2025 | 0 | 7 |
| Bi | CoT-X: Cross-Model CoT Transfer | 2025 | 0 | 10 |
| Chang | MACI: Dual-Dial Multi-Agent Control | 2025 | 1 | 9 |
| Chen (code) | PerfOrch: Multi-Stage Code Orchestration | 2025 | 0 | 14 |
| Chen (MoSA) | Multi-LLM Collaborative Search | 2025 | 7 | 3 |
| Chen (routing) | Symbolic-MoE | 2025 | 20 | 3 |
| Chen (sampling) | ModelSwitch: Multi-LLM Repeated Sampling | 2025 | 13 | 3 |
| Ding | BEST-Route | 2025 | 9 | 1 |
| Fanconi | Cascaded LLMs for Human-AI Decision-Making | 2025 | 1 | 2 |
| Fein-Ashley | Mixture of Thoughts (MoT) | 2025 | 0 | 3 |
| Fu | Collaborative Decoding via Speculation (CoS) | 2025 | 5 | 7 |
| Gao | CoSine: Collaborative Speculative Inference | 2025 | 3 | 7 |
| Ghaffari | Ensemble Embedding for Semantic Caching | 2025 | 0 | 11 |
| Gupta | HierRouter | 2025 | 0 | 4 |
| Hu | BrownoutServe: SLO-Aware MoE | 2025 | 0 | 13 |
| Huang (sample) | Sample Complexity of TTC | 2025 | 1 | 6 |
| Huang (spec) | SpecServe | 2025 | 8 | 7 |
| Jo | SpareLLM | 2025 | 1 | 1 |
| Li (bandit) | LLM Bandit | 2025 | 18 | 1 |
| Li (budget) | SelfBudgeter | 2025 | 18 | 10 |
| Li (budget guide) | Budget Guidance | 2025 | 10 | 10 |
| Li (kNN) | kNN Beats Complex Routers | 2025 | 1 | 4 |
| Li (MoA) | Self-MoA: Rethinking MoA | 2025 | 21 | 3 |
| Liu | Semantic Caching (principled) | 2025 | 0 | 11 |
| Lu | RouterArena | 2025 | 0 | 12 |
| Ma | S2R: Self-Verify via RL | 2025 | 8 | 9 |
| Poon | Online Multi-LLM via Contextual Bandits | 2025 | 3 | 4 |
| Sen | Confidence-Modulated Speculative Decoding | 2025 | 1 | 7 |
| Shah | EMAFusion | 2025 | 0 | 2 |
| Shan | MA-Router | 2025 | 0 | 4 |
| Shi | InferenceDynamics | 2025 | 5 | 12 |
| Si | CCPO: Conformal Constrained Policy Optimization | 2025 | 0 | 8 |
| Singh | ARTIST: Agentic Reasoning + Tool Integration | 2025 | 37 | 14 |
| Song | IRT-Router | 2025 | 7 | 4 |
| Su | DAAO: Difficulty-Aware Orchestration | 2025 | 0 | 5 |
| Tang | CarBoN: Calibrated Best-of-N | 2025 | 0 | 6 |
| Tayebati | Learnable Conformal Abstention | 2025 | 6 | 8 |
| Varangot-Reille | Routing Strategies Survey | 2025 | 10 | 15 |
| Wang (ICL) | ICL-Router | 2025 | 0 | 4 |
| Wang (MARS) | MARS: Multi-Agent Review System | 2025 | 0 | 9 |
| Yan | ContextCache | 2025 | 2 | 11 |
| Zellinger | LLM Cascades with Early Abstention | 2025 | 5 | 2 |
| Zeng | EvalTree | 2025 | 13 | 12 |
| Zhang (Avengers) | Avengers-Pro | 2025 | 8 | 1 |
| Zhang (CLAI) | Cognitive Load-Aware Inference | 2025 | 2 | 5 |
| Zhang (Model-SAT) | Capability Instruction Tuning | 2025 | 11 | 4 |
| Zhang (parallel) | M1-Parallel | 2025 | 2 | 14 |
| Zhang (Router-R1) | Router-R1 | 2025 | 5 | 4 |
| Zhao (SPOC) | Spontaneous Self-Correction | 2025 | 2 | 9 |
| Zhao (SeaLLM) | SeaLLM | 2025 | 0 | 13 |
| Zhou (conformal) | Robust UQ via Conformal Prediction | 2025 | 0 | 8 |
| Zhou (fusion) | Model Fusion Survey | 2025 | 1 | 15 |
| Zhou (SteerConf) | SteerConf | 2025 | 4 | 8 |
