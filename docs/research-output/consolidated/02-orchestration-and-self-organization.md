# Agent Orchestration & Self-Organization -- Consolidated State of Knowledge

**Created:** 2026-03-07
**Source rounds:** R3, R4, R5, R6, R7a, R7b, R7d
**Source files:** 7 (1 synthesis, 6 research)
**Papers cataloged:** ~300+ across orchestration-relevant themes
**Confidence methodology:** Claims labeled [HIGH], [MODERATE], [TENTATIVE], or [SPECULATIVE] based on evidence density and replication across independent sources

---

## Executive Summary

Across seven independent research rounds encompassing 300+ papers and industry reports (2025--2026), a decisive architectural consensus emerges: **static, centralized, fixed-topology orchestration is a dead end for production multi-agent systems.** The field is converging on dynamic, topology-aware, self-modifying orchestration backed by formal verification and decentralized coordination primitives.

Five core conclusions, each supported by multiple independent sources:

1. **Topology dominates model capability.** As foundation models converge in raw benchmark performance, the orchestration structure -- not the model -- becomes the primary performance lever. AdaptOrch demonstrates double-digit percentage improvements by selecting topology (parallel/sequential/hierarchical/hybrid) based on task-graph structure, with identical underlying models (R7d). This finding is reinforced by MaAS showing 0.5--12% performance gains at 6--45% of inference cost by dynamically sampling agent configurations (R4, R6).

2. **Decentralized coordination scales; centralized does not.** DynTaskMAS shows near-linear throughput scaling to 16 agents (3.47x) but centralized schedulers degrade beyond that (R5, R6). AgentNet eliminates the central orchestrator entirely and achieves higher accuracy than centralized baselines (R4, R5). FoA achieves 13x improvement on HealthBench through decentralized semantic routing (R6). The ceiling for centralized orchestration is roughly 20 agents before contention becomes the bottleneck.

3. **Meta-orchestration -- systems that design their own orchestration -- is the frontier.** MAS^2's generator-implementor-rectifier triad achieves up to 19.6% improvement on complex tasks by recursively generating bespoke MAS architectures per problem instance (R5, R6). AutoMaAS yields 1.0--7.1% performance improvement while reducing costs 3--5% through automated operator fusion and elimination (R6). These are not incremental gains -- they represent a paradigm shift from "design an agent team" to "design a system that designs agent teams."

4. **RL-trained orchestration outperforms static workflows.** The "puppeteer" paradigm -- a centralized RL controller that dynamically sequences agents -- yields more compact cyclic reasoning structures, reduced computational cost, and superior performance/cost trade-offs versus fixed DAGs (R5, R6, R7b). PPO and REINFORCE are best for multi-step DAG orchestration; contextual bandits suffice for single-step routing (R6).

5. **The security model must evolve from infrastructure isolation to semantic firewalls.** The "Agent Smith" infectious jailbreak demonstrates exponential system-wide compromise from a single adversarial input through shared memory (R7d). Inter-agent communication hijacking succeeds in 58--100% of trials even when individual agents resist injection (R4). Traditional OTP supervision is blind to semantic degradation -- a hallucinating actor appears healthy to a process monitor (R7d).

**For Mister Smith specifically:** The existing 9-role static team orchestration (Phase 7) and OTP supervision trees (Phase 3) provide the correct foundational primitives. The path forward is not replacing them but layering dynamic topology selection, decentralized discovery via VCVs over NATS JetStream KV, and meta-orchestration (MAS^2/AutoMaAS) on top. The Rust + NATS + actor model stack is uniquely well-suited for this -- no competing framework (Microsoft Agent Framework, Akka, CrewAI, LangGraph) currently implements topology-aware dynamic orchestration with formal verification guarantees.

---

## High-Confidence Findings

These findings appear in 3+ independent sources and represent the strongest architectural signals:

| Finding | Sources | Evidence Strength | Key Metric |
|:--------|:--------|:-----------------|:-----------|
| Linear ReAct loops cap at ~85% success; tree-search (LATS/MCTS) reaches 94.4% pass@1 | R3 (3 reports converge), R4 | [HIGH] | +9.4pp on HumanEval |
| Decentralized DAG coordination outperforms centralized at scale | R4, R5, R6, R7b | [HIGH] | 3.47x throughput at 16 agents (DynTaskMAS); 13x on HealthBench (FoA) |
| Dynamic topology selection beats static by double digits | R6, R7d | [HIGH] | AdaptOrch: >10% over static baselines, identical models |
| Meta-orchestration (MaAS/MAS^2) surpasses hand-designed teams | R4, R5, R6 | [HIGH] | MaAS: 0.5--12% gains at 6--45% cost; MAS^2: up to 19.6% |
| RL puppeteer produces compact cyclic reasoning, lower cost | R5, R6, R7b | [HIGH] | Cost reduction via learned routing; compact cycles |
| OTP-style supervision maps naturally to DAG node management | R3, R6 | [HIGH] | one_for_one, rest_for_one strategies for DAG recovery |
| Inter-agent communication is a critical attack surface | R4, R7d | [HIGH] | 58--100% attack success on multi-agent systems |
| CRDTs enable observation-driven coordination (alternative to message passing) | R4, R6 | [MODERATE] | 100% convergence, 0 merge failures across 600 trials (CodeCRDT) |
| Consensus-free debate outperforms forced consensus | R4 | [MODERATE] | Anti-conformity prevents groupthink degradation |
| Event-triggered consensus reduces communication overhead | R5 | [MODERATE] | Validated across simulation and real-world scenarios |

---

## Key Techniques & Architectures

### Static Loop Patterns (ReAct, LATS, Plan-then-Execute, MCTS)

**Sources:** R3 (primary, 3-report convergence), R4

**Mechanism:** The foundational agentic loop patterns form a complexity hierarchy:

- **ReAct** (Reason + Act): Linear thought-action-observation cycle. Simple, low overhead, but caps at ~85% success on benchmark tasks. Vulnerable to context drift in long-horizon tasks. (R3)
- **Reflexion**: Adds episodic memory -- stores reflections from failed attempts for retry improvement. Improves coding benchmarks measurably. (R3)
- **Tree of Thoughts (ToT)**: Explores multiple reasoning branches in parallel with evaluation/pruning. (R3)
- **LATS/MCTS**: Maps Monte Carlo Tree Search to agent reasoning. 94.4% pass@1 on HumanEval vs ~85% for ReAct. All three R3 source reports independently recommend mapping MCTS to actor supervision trees. (R3)
- **Plan-then-Execute (P-t-E)**: Planner creates a full DAG upfront; executors process nodes. Inherently more resilient to prompt injection than ReAct because it establishes control-flow integrity. (R4)
- **ReCAP/ReAcTree**: Hierarchical designs preventing context drift through structured replanning at explicit checkpoints. (R3)

**Evidence:** LATS reported 94.4% pass@1 on HumanEval (R3 citing LATS paper). P-t-E achieves state-of-the-art on StableToolBench (R4). ReAct is described as the baseline that "tops out" across all reports.

**Mister Smith integration path:** The R3 synthesis prescribes a **two-level loop architecture**:
- Inner loop: ReAct-style reason/act/observe with strict tool schemas, approval gates, context packs
- Outer loop: MCTS-lite supervisory controller that decides when to continue, fork, backtrack, or terminate
- Critic as continuous value function (not post-hoc reviewer)
- Budget-aware escalation: cheap linear first, escalate to search only on low confidence

The architecture maps directly to Mister Smith's actor model: Session actor spawns Budget, Memory, Planner, SearchCoordinator, Executor, and Critic actors under OTP supervision. JetStream provides durable node storage for MCTS search state. (R3)

---

### DAG-Based Parallel Execution (Flash-Searcher, dependency graphs)

**Sources:** R3, R4, R5, R6, R7d

**Mechanism:** Replace sequential agent chains with Directed Acyclic Graphs where independent reasoning paths execute concurrently while maintaining logical dependency constraints. Three distinct approaches:

1. **Planner-Centric DAG** (Wei et al., 2025): Planner model creates a full DAG of tool operations upfront, enabling optimized parallel execution. State-of-the-art on StableToolBench. (R4)
2. **Flash-Searcher** (Qin et al., 2025): Reimagines execution from sequential chains to DAGs. Reduces agent execution steps by up to 35%. (R4)
3. **DynTaskMAS** (Yu et al., 2025): Orchestrates asynchronous parallel operations using dynamic task graphs. Near-linear throughput scaling up to 16 agents (3.47x improvement). (R5, R6)

**Evidence:** Flash-Searcher: 35% step reduction (R4). DynTaskMAS: 3.47x throughput at 16 agents but degradation beyond due to centralized scheduler overhead and shared state contention (R5, R6). The degradation threshold (~16--20 agents) is consistent across sources.

**Mister Smith integration path:**
- Add a `DagPlanner` trait to the agent system generating execution DAGs from task descriptions
- Use NATS JetStream consumer groups to parallelize independent DAG nodes
- Implement dependency tracking in the task scheduler for DAG edges
- Dynamic DAG re-planning on node failure connects to existing supervision trees
- NATS subject taxonomy: `smith.orchestrate.execute.{node_id}` for per-node task assignment (R6)

---

### Decentralized Self-Organization (AgentNet, FoA VCVs, DynTaskMAS)

**Sources:** R4, R5, R6

**Mechanism:** Three frameworks eliminate or reduce the central orchestrator:

1. **AgentNet** (Yang et al., 2025, 16 citations): Fully decentralized DAG-based framework where agents autonomously specialize and route tasks based on local expertise and context. Uses retrieval-based memory for continual skill refinement. Eliminates single points of failure. Achieves higher accuracy than both single-agent and centralized multi-agent baselines. (R4, R5)

2. **Federation of Agents (FoA)** (Giusti et al., 2025): Introduces Versioned Capability Vectors (VCVs) -- machine-readable profiles transforming agent capabilities, costs, and constraints into searchable semantic embeddings. VCV schema includes: dense capability embedding, Bloom filter for discrete skills, resource requirements (latency/energy budgets), policy compliance flags, version counter. Uses sharded HNSW indexes for sub-linear agent discovery. Compatible agents collaboratively break down complex tasks into DAGs via consensus-based merging. 13x improvement over single-model baselines on HealthBench. (R4, R5, R6)

3. **DynTaskMAS**: See DAG section above. Near-linear scaling to 16 agents. (R5, R6)

**ANN design trade-offs for capability discovery** (R6):

| Technology | Strengths | Weaknesses |
|:-----------|:----------|:-----------|
| HNSW | ~95--99% recall, sub-millisecond latency | High memory footprint |
| IVF-PQ | Memory efficient, billion-scale | Lower precision, requires training |
| Vector DBs (Milvus, Qdrant) | Built-in sharding/replication | Operational complexity, higher latency |

**Consensus-based DAG assembly** uses Graph CRDTs to support addition/removal of nodes and edges while maintaining DAG invariants (no cycles) without central coordination. (R6)

**Mister Smith integration path:**
- Store VCVs in NATS JetStream KV with `watch` functionality for real-time capability updates
- Deploy in-memory sharded HNSW index (Rust) per cluster for sub-linear agent discovery
- Hybrid model: centralized RL controllers for small agent pods (<20 agents), decentralized FoA clustering for 100+ agents (R6 explicit recommendation)
- Graph CRDTs (`crdts` or `automerge` Rust crates) for decentralized DAG assembly
- NATS subject: `smith.discovery.vcv.{agent_id}` for capability advertisement (R6)

---

### Meta-Orchestration / Architecture Search (MaAS, AutoMaAS, MAS^2)

**Sources:** R4, R5, R6

**Mechanism:** Instead of designing a fixed agent topology, these systems automatically discover, generate, and evolve optimal agent configurations:

1. **MaAS** (Zhang et al., 2025, 52 citations): Optimizes an "agentic supernet" -- a continuous distribution of possible multi-agent architectures. A controller samples query-dependent configurations dynamically allocating resources. Requires only 6--45% of inference costs while surpassing static systems by 0.5--12%. (R4, R6)

2. **AutoMaAS** (Ma et al., 2025): Extends MaAS with dynamic operator lifecycle management. Health scores combine usage frequency, performance contribution, and cost efficiency. When operators frequently collaborate with high correlation, LLM-guided fusion generates a new combined operator. Operators whose health score falls below threshold are automatically eliminated. Yields 1.0--7.1% performance improvement while reducing inference costs by 3--5%. (R4, R6)

3. **MAS^2** (Wang et al., 2025): Tri-agent meta-system:
   - **Generator**: Architects a high-level multi-agent workflow template (DAG) for a specific query
   - **Implementor**: Instantiates the template by populating each step with a concrete LLM backbone and specific tools
   - **Rectifier**: Monitors execution state and environmental feedback, issuing timely corrections. Activates when cumulative resource consumption exceeds budget or explicit failures occur. Issues modifications ranging from local (re-assigning tools) to global (revising workflow codes)

   Outperforms static "generate-once-and-deploy" paradigms by up to 19.6% on complex benchmarks. (R5, R6)

**Optimization methods comparison** (R6):

| Method | Mechanism | Best For |
|:-------|:----------|:---------|
| RL (PPO) | Trains controller to sample architectures maximizing reward | Online, query-dependent routing |
| Evolutionary (NSGA-Net) | Population-based crossover/mutation | Multi-objective (accuracy vs. FLOPs) |
| Bayesian (BANANAS) | Neural predictor models unseen architectures | Offline macro-architecture discovery |
| Differentiable NAS (DARTS) | Continuous relaxation, gradient descent | Fast search but prone to collapse |

**R6 recommendation:** Hybrid approach -- Bayesian Optimization for offline macro-architecture discovery, RL for online query-dependent routing.

**Mister Smith integration path:**
- Implement an `AgentArchitectureSampler` using task features to select team composition
- Operators defined as strict Rust traits with JSON Schema I/O
- Operator metadata (health scores, versions) in JetStream KV
- Wasm sandboxes (Wasmtime) for executing LLM-generated or fused operator code
- Rectifier monitors JetStream telemetry; triggers Implementor to reconfigure OTP child processes without halting the system
- Start simple: learned routing between team sizes (single vs. pair vs. full team) based on task complexity, using existing monitoring infrastructure (Phase 2/8) for training data (R4)

---

### Topology Routing (AdaptOrch -- parallel/sequential/hierarchical/hybrid)

**Sources:** R7d (primary), R6

**Mechanism:** AdaptOrch formalizes the insight that task structure -- not model capability -- dominates system performance. All complex tasks decompose into dependency-annotated DAGs. A linear-time algorithm evaluates:
- Parallelism width
- Critical path depth
- Degree of inter-subtask coupling

Based on these metrics, execution routes to one of four canonical topologies:

| Task Characteristic | Optimal Topology | Rationale |
|:-------------------|:-----------------|:----------|
| High parallelism, low coupling | Pure parallel swarm | Independent tasks; linear acceleration via async concurrency |
| High critical path depth, high coupling | Strict sequential chain | Codependent outputs; Markovian state progression |
| Mixed width, clustered dependencies | Hierarchical tree | Local consensus before intermediate aggregation |
| Dynamic uncertainty, variable depth | Adaptive hybrid | Runtime restructuring based on validation thresholds |

An Adaptive Synthesis Protocol uses heuristic consistency scoring for parallel agent outputs with provable termination guarantees. (R7d)

**Evidence:** Double-digit percentage improvements over static single-topology baselines across software engineering benchmarks, complex reasoning, and retrieval workflows -- with identical underlying models. (R7d)

**Mister Smith integration path:**
- Build a **Topology Compiler** that runs before execution: when a planning agent generates a task breakdown, the compiler analyzes the dependency graph and dynamically allocates actors into ephemeral topologies matching the structural requirements
- For heavily decoupled tasks: spawn an ultra-wide parallel array of isolated executor agents, followed by a dynamically generated consensus actor to synthesize outputs
- Tear down the entire topology when the task concludes (ephemeral actor groups)
- This maps naturally to Mister Smith's Tokio-based actors + NATS subjects -- each topology becomes a subject namespace + supervision subtree

---

### RL-Trained Orchestration (Flow-GRPO, puppeteer, REINFORCE/PPO)

**Sources:** R4, R5, R6, R7b

**Mechanism:** A centralized RL controller ("puppeteer") is trained to dynamically sequence agents based on evolving task states. The optimization objective maximizes expected return over complete reasoning trajectories where return reflects both accuracy and inference efficiency (token cost).

Key approaches:

1. **Puppeteer paradigm** (Dang et al., 2025): RL-trained orchestrator adaptively sequences agents based on live feedback. Learns to favor compact, cyclic reasoning structures over exhaustive static chains. (R5, R6, R7b)

2. **Flow-GRPO** (Li et al., 2025): Trains a four-module system (planner, executor, verifier, generator) in-the-flow -- directly optimizing the planner within the live multi-turn execution loop. Converts multi-turn RL optimization into tractable single-turn updates. 7B model outperforms GPT-4o by 14.9% (search), 14.0% (agentic), 14.5% (math). (R4)

3. **xRouter**: Reward gated by task success (no success = zero reward), penalized by total cost of all model invocations. (R6)

**Algorithm comparison** (R6):

| Algorithm | Type | Orchestration Fit |
|:----------|:-----|:-----------------|
| REINFORCE/PPO | Policy gradient | Multi-step routing policies; PPO scales well but complex config |
| DPO | Reward-free | Human preference alignment; can yield biased solutions |
| Contextual bandits (UCB, Thompson) | Bandit | Single-step tool/model routing; cannot handle multi-step credit assignment |

**Preventing catastrophic forgetting:** Elastic Weight Consolidation (EWC) constrains parameters to stay in low-error regions for previous tasks using Fisher information matrix. Combined with stateful experience replay via JetStream. (R6)

**Safe RL:** Constrained Trust Region Policy Optimization (C-TRPO) modifies policy space geometry based on safety constraints, yielding trust regions of exclusively safe policies. (R6)

**Mister Smith integration path:**
- Use contextual bandits for single-step tool/model routing
- PPO for complex multi-step DAG orchestration
- JetStream for experience replay storage (durable, replayable)
- EWC for preventing catastrophic forgetting during online adaptation
- C-TRPO to ensure orchestrator never deploys a DAG violating resource caps or security boundaries
- Start with the planner/executor/verifier/generator decomposition from Flow-GRPO, which aligns with existing agent roles

---

### Game-Theoretic Mechanism Design (BlockAgents, Proof-of-Thought, auctions)

**Sources:** R7d (primary), R7a

**Mechanism:** As multi-agent systems scale beyond static workflows, game-theoretic failures emerge. GT-HarmBench shows contemporary agents choose cooperative actions in only a fraction of Prisoner's Dilemma/Stag Hunt interactions, frequently defaulting to defection. (R7d)

Mitigations:

1. **BlockAgents / Proof-of-Thought**: Evaluates agents not on final output but on a multi-metric assessment of reasoning trajectory -- factual consistency, redundancy reduction, contextual causal relevance. Evaluations recorded on an immutable distributed ledger, establishing reputation-based trust. (R7d)

2. **Incentive-centric mechanisms**: Model agent utility as a function of task rewards, capability mismatch, and workload capacity. Sequential public-goods games with adaptive reputation weighting mathematically guarantee that truthful reporting and team behavior become the Subgame Perfect Nash Equilibrium. (R7d)

3. **Auction-based allocation**: When a complex task is decomposed, subtask requirements are published. Available executor agents bid based on real-time token budgets and confidence intervals. Decentralized supervisor evaluates bids and allocates. Output verified via Proof-of-Thought, dynamically adjusting reputation. (R7d)

**Evidence:** Theoretical guarantees from mechanism design literature. BlockAgents provides Byzantine-robustness. Auction models demonstrate self-balancing at scale. [MODERATE] -- theoretical grounding is strong but production validation is limited.

**Mister Smith integration path:**
- Implement an internal Agent Exchange on NATS: planning module publishes subtask requirements to `smith.orchestrate.bid.{task_id}`, executor agents bid
- Proof-of-Thought scoring as a post-execution verification step in the Critic actor
- Reputation scores stored in JetStream KV, influencing future bid authority
- This creates a self-balancing ecosystem for 100+ agents without centralized scheduling

---

### Consensus-Free Debate & Anti-Conformity

**Sources:** R4

**Mechanism:** Three key findings challenge the assumption that agent agreement signals quality:

1. **Free-MAD** (Cui et al., 2025): Eliminates the requirement for agents to reach consensus. Uses score-based decision mechanism evaluating entire debate trajectories rather than final-round majority voting. Introduces "anti-conformity" to prevent LLM agents from being swayed by incorrect majority opinions. Significantly improves reasoning with single-round debate while reducing token costs. (R4)

2. **MARS** (Wang et al., 2025): Implements author-reviewer-meta-reviewer pipeline. Matches debate accuracy with 50% less token usage by avoiding reviewer-to-reviewer interactions. Maps perfectly to software development workflows. (R4)

3. **Multi-Agent Debate with Adaptive Stability Detection** (Hu et al., 2025): Mathematically proves debate amplifies correctness vs. static ensembles. Uses Beta-Binomial mixture models and Kolmogorov-Smirnov tests for adaptive stopping when consensus is detected. (R4)

**Evidence:** Free-MAD: measurably improved reasoning, reduced tokens. MARS: 50% token reduction vs debate. Adaptive stopping: formal mathematical proof. (R4)

**Mister Smith integration path:**
- Implement `ReviewProtocol` trait with `Author`, `Reviewer`, `MetaReviewer` roles
- Use MARS-style independent review (no reviewer-to-reviewer communication) to reduce token cost
- Track trajectory scores across agent interactions for quality assessment
- Integrate K-S test adaptive stopping to avoid unnecessary debate rounds
- Critical insight: anti-conformity must be explicitly engineered; LLM agents naturally converge toward groupthink

---

## Competitive Landscape

**Sources:** R7a (primary), R5, R7b

### Microsoft Agent Framework (October 2025)
Open SDK unifying Semantic Kernel and AutoGen. Emphasizes interoperability, observability, and compliance. Multi-platform deployment, enterprise-grade orchestration. **Strategic assessment:** Strong enterprise tooling but no evidence of topology-aware dynamic orchestration or meta-orchestration capabilities. (R7a)

### Akka Agentic Platform (July 2025)
High-performance, fault-tolerant. Up to 15,000 actors, 25,000 req/sec throughput, 32ms latency at p99. Durable event sourcing, multi-region elasticity. **Strategic assessment:** Closest competitor to Mister Smith's actor + event-sourcing architecture. The throughput numbers (25k req/sec) set a benchmark. However, Akka's JVM overhead vs Mister Smith's Rust gives a fundamental resource advantage. (R7a)

### Symphony (August 2025)
Decentralized ledger and dynamic task allocation. Empirical scalability beyond 50 agents with robustness and accuracy gains. **Strategic assessment:** Validates the decentralized coordination approach. Mister Smith should study Symphony's beacon-based protocol design. (R7a, R7b)

### GNN Swarm Coordination (November 2025)
GNN-based coordination scaling to 100+ agents, with curriculum-guided hierarchical systems managing up to 4,096 agents with improved stability and task success rates. **Strategic assessment:** Validates that swarm-scale orchestration is achievable. GNN approach is complementary to VCV-based routing. (R7a)

### SECP -- Self-Evolving Coordination Protocols (February 2026)
Bounded self-modification of coordination protocols, increasing proposal coverage without violating invariants. **Strategic assessment:** Directly relevant to meta-orchestration safety -- provides a model for bounded protocol evolution. (R7a)

### Existing Frameworks (CrewAI, LangGraph, AutoGen)
All remain predominantly Python-based with centralized orchestration. None implement formal topology routing, VCV-based discovery, or meta-orchestration. (R3, R7a)

### Rust Ecosystem
- **ZeroClaw**: 3.4MB static binary, <5MB RAM, sub-10ms startup (R7d)
- **Kameo**: Tokio-based actors with local registries, backpressure, deadlock warnings (R7d)
- **GraphFlow**: Rust-native multi-agent workflow orchestration (R7d)
- **autoagents**, **adk-rust**: Newer frameworks, still maturing (R7a)

**Mister Smith's competitive position:** No competing framework combines Rust performance + NATS JetStream for event sourcing + OTP supervision trees + a clear path to topology-aware meta-orchestration. The primary risk is execution speed -- several frameworks (Microsoft, Akka) have larger teams and enterprise backing. The primary advantage is architectural: starting from actors + event sourcing + supervision rather than bolting them on.

---

## Open Questions & Gaps

### Unresolved in the Literature

1. **Decentralized DAG + OTP supervision integration**: How exactly do decentralized, agent-initiated DAG topologies coexist with hierarchical OTP supervision trees? No paper provides a concrete reconciliation. Mister Smith must design this mapping. (R5)

2. **Adversarial robustness of decentralized coordination**: Few works rigorously address adversarial failure modes under decentralized DAG coordination. The "Agent Smith" infectious jailbreak (R7d) demonstrates the risk, but defenses remain nascent. (R7b)

3. **Memory/context management at extreme scale**: Context management remains a bottleneck beyond ~50 agents. Existing solutions (summarization, CRDTs, episodic memory) have not been validated at 100+ agent scale with dynamic topologies. (R7b)

4. **Trust calibration generalization**: Trust-calibrated orchestration (Roumeliotis et al.) shows promise but has only been validated in vision-language settings. Generalization to heterogeneous LLM ensembles is untested. (R7b)

5. **Real-world production deployment**: Nearly all empirical studies use synthetic benchmarks. Production validation of meta-orchestration, decentralized DAGs, and RL puppeteers is absent from the literature. (R7b)

6. **Compensation/saga patterns for LLM tools**: No taxonomy exists for LLM tool reversibility or concrete compensation patterns for common tool types (file ops, API calls, DB mutations). (R3)

7. **Formal verification of evolving DAGs**: Multiparty Session Types (MPST) can verify static protocols at compile time, but their application to dynamically evolving DAGs (where the protocol shape changes at runtime) is unexplored. (R7d)

### Mister Smith-Specific Unknowns

8. **NATS JetStream KV as VCV store at scale**: JetStream KV supports high-throughput ops, but default `sync_interval` settings risk data loss during OS crashes. Need `sync_interval: always` and replication factor 3 for critical orchestration state, but latency impact is unknown at high agent counts. (R6)

9. **Wasm sandbox overhead for operator execution**: All meta-orchestration papers recommend Wasm sandboxing for LLM-generated operators, but no benchmarks exist for the throughput impact in a Rust + Wasmtime + actor pipeline.

10. **Ractor vs. custom actor framework**: R6 recommends Ractor for OTP-style supervision in Rust, but Mister Smith has a custom actor system (Phase 3). Migration cost vs. feature parity is unassessed.

---

## Implementation Priority for Mister Smith

Ranked by strategic impact, feasibility given existing infrastructure, and evidence strength:

### Tier 1: Build Now (leverage existing Phase 3/7/8 infrastructure)

**P1. DAG-Based Parallel Execution Engine**
- Add `DagPlanner` trait and dependency-tracking scheduler to the agent system
- Parallelize independent DAG nodes via JetStream consumer groups
- Dynamic re-planning on node failure using existing supervision trees
- *Why now:* Directly extends Phase 7 orchestrator. 35% step reduction (Flash-Searcher) is a concrete, measurable win. Foundation for all subsequent orchestration work.

**P2. Two-Level Loop Architecture**
- Inner loop: ReAct with tool schemas, approval gates, context packs
- Outer loop: MCTS-lite supervisory controller (continue/fork/backtrack/terminate)
- Budget-aware escalation: cheap linear first, search on low confidence
- *Why now:* R3 convergence across 3 independent reports. Maps directly to existing actor model. Provides the reasoning quality ceiling that everything else builds on.

**P3. Topology Compiler (AdaptOrch)**
- Linear-time algorithm analyzing parallelism width, critical path depth, inter-subtask coupling
- Routes to parallel/sequential/hierarchical/hybrid topology
- Ephemeral actor groups torn down after task completion
- *Why now:* Double-digit improvements with identical models. Directly leverages Tokio async runtime and NATS subject namespaces. Low coupling to LLM provider choice (model-agnostic).

### Tier 2: Build Next (requires P1/P2/P3 as foundation)

**P4. VCV-Based Decentralized Discovery**
- VCV schema in JetStream KV with `watch` for real-time updates
- In-memory HNSW index for sub-linear capability matching
- Hybrid: centralized for <20 agents, FoA clustering for 100+
- *Why next:* Requires DAG execution engine (P1) to be useful. Enables scaling past the ~20-agent centralized ceiling.

**P5. MAS^2 Meta-Orchestration (Generator-Implementor-Rectifier)**
- Generator produces DAG templates per query
- Implementor maps to supervision tree
- Rectifier monitors JetStream telemetry, triggers reconfiguration
- *Why next:* Requires topology compiler (P3) and VCV discovery (P4). The 19.6% improvement justifies the complexity, but only after the foundation is solid.

**P6. Semantic Supervision Layer**
- Extend OTP supervision from binary process monitoring to semantic health tracking
- Lightweight evaluator actors continuously vote on agent behavioral health (consensus-based threat validation)
- Quarantine actors for inter-agent message sanitization
- *Why next:* Addresses the "Agent Smith" infectious jailbreak vector and hallucination-blind supervision gap. Requires reliable DAG execution to be worth protecting.

### Tier 3: Build Later (research-stage or optimization)

**P7. RL Puppeteer (PPO/REINFORCE)**
- Off-policy RL orchestrator using JetStream for experience replay
- EWC for catastrophic forgetting prevention
- C-TRPO for safe policy updates
- *Why later:* Requires substantial training data from production operation (P1--P5 running). Contextual bandits provide 80% of the value for single-step routing with much less complexity.

**P8. Game-Theoretic Auction-Based Allocation**
- Agent Exchange on NATS with bid/allocate/verify cycle
- Proof-of-Thought scoring, reputation in JetStream KV
- *Why later:* Only necessary at very large scale (100+ agents). Academic validation but no production evidence.

**P9. AutoMaAS Operator Fusion/Elimination**
- Health scoring (frequency, contribution, cost) for operators
- LLM-guided fusion of correlated operators
- Automatic elimination of low-value operators
- *Why later:* Requires a stable operator ecosystem and sufficient telemetry history. The 1--7% gains are meaningful but incremental on top of P5.

---

## Sources

| File | Round | Content |
|:-----|:------|:--------|
| `synthesis/agentic-loop-architectures-R3.md` | R3 | Triple synthesis: ReAct, LATS, MCTS, two-level loop, actor decomposition, budget management, Mister Smith blueprint |
| `research/targeted-dynamic-self-organization-R6.md` | R6 | MaAS, AutoMaAS, MAS^2, FoA VCVs, HNSW, DynTaskMAS scaling, RL puppeteer, OTP mapping, NATS subject taxonomy |
| `research/discovery-sweep-R4.md` | R4 | CRDTs, DAG parallel execution (Flash-Searcher, AgentNet), MaAS (52 citations), consensus-free debate, AgentOps |
| `research/discovery-sweep-R5.md` | R5 | AgentNet, FoA VCVs, DynTaskMAS, MAS^2, OSC cognitive models, event-triggered consensus, KB-aware routing |
| `research/discovery-sweep-R7a.md` | R7 | Microsoft Agent Framework, Akka (25k req/sec, 15k actors), Symphony, GNN swarm (4096 agents), SECP, Rust crates |
| `research/discovery-sweep-R7b.md` | R7 | RL puppeteer, AgentAsk clarification, trust calibration, adversarial robustness gaps, scaling limits |
| `research/discovery-sweep-R7d.md` | R7 | AdaptOrch topology routing, PrefillShare KV cache, MPST session types, biomimetic immunity, game-theoretic mechanism design, Agent Smith infectious jailbreak, A2A protocol |

### Key Paper Citations (by discovery impact)

- **MaAS** -- Zhang et al., 2025 (52 citations). arXiv:2502.04180
- **MAS^2** -- Wang et al., 2025. arXiv:2509.24323
- **AutoMaAS** -- Ma et al., 2025. arXiv:2510.02669
- **AgentNet** -- Yang et al., 2025 (16 citations). arXiv:2504.00587
- **FoA** -- Giusti et al., 2025. OpenReview N7NDfV2YMp / arXiv:2509.20175
- **DynTaskMAS** -- Yu et al., 2025. arXiv:2503.07675
- **AdaptOrch** -- 2026. arXiv:2602.16873
- **Flow-GRPO** -- Li et al., 2025 (2 citations). arXiv (AgentFlow)
- **Free-MAD** -- Cui et al., 2025. arXiv
- **LATS** -- Zhou et al., 2023 (foundation). arXiv:2310.04406
- **BlockAgents** -- 2025. ResearchGate 382711037
- **Agent Smith (infectious jailbreak)** -- 2025. ResearchGate 380897242
- **Evolving Orchestration (puppeteer)** -- Dang et al., 2025. arXiv:2505.19591
- **SECP** -- 2026. arXiv
- **Symphony** -- Wang et al., 2025. arXiv:2508.20019
