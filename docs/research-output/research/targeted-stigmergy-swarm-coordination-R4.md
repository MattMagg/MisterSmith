---
version: R4
created: 2026-03-07
updated: 2026-03-07
sources: Consensus (52 papers, 22 searches)
round: 4 (Academic Search)
---

# Research Digest: Stigmergic Coordination and Swarm Intelligence for LLM Multi-Agent Systems

**Generated**: 2026-03-07
**Search Scope**: Consensus academic search, year_min=2025
**Purpose**: Inform Mister Smith's coordination architecture -- a Rust-based multi-agent orchestration framework using NATS/JetStream pub/sub, OTP-style supervision trees, and model-agnostic LLM integration.

---

## Table of Contents

1. [Stigmergy and Pheromone-Based Coordination](#1-stigmergy-and-pheromone-based-coordination)
2. [Blackboard Architectures for LLM Multi-Agent Systems](#2-blackboard-architectures-for-llm-multi-agent-systems)
3. [Swarm Intelligence Applied to LLM Orchestration](#3-swarm-intelligence-applied-to-llm-orchestration)
4. [Decentralized Coordination and Dynamic Topology](#4-decentralized-coordination-and-dynamic-topology)
5. [Shared Memory, Context Routing, and Indirect Communication](#5-shared-memory-context-routing-and-indirect-communication)
6. [Task Allocation, Routing, and Agent Selection](#6-task-allocation-routing-and-agent-selection)
7. [Fault Tolerance, Byzantine Resilience, and Consensus](#7-fault-tolerance-byzantine-resilience-and-consensus)
8. [Evolutionary and Self-Improving Agent Systems](#8-evolutionary-and-self-improving-agent-systems)
9. [Multi-Agent Safety and Adversarial Robustness](#9-multi-agent-safety-and-adversarial-robustness)
10. [Hierarchical Coordination and Theoretical Foundations](#10-hierarchical-coordination-and-theoretical-foundations)
11. [Comprehensive Surveys and Taxonomies](#11-comprehensive-surveys-and-taxonomies)
12. [Emerging Directions](#12-emerging-directions)
13. [Synthesis: Implications for Mister Smith](#13-synthesis-implications-for-mister-smith)

---

## 1. Stigmergy and Pheromone-Based Coordination

### 1.1 From Pheromones to Policies: Reinforcement Learning for Engineered Biological Swarms
- **Authors**: Vellinger, Antonic, Tuci (2025)
- **Citation Count**: 1
- **Journal**: ArXiv abs/2509.20095
- **Key Finding**: Establishes a **formal mathematical equivalence** between pheromone-mediated aggregation and reinforcement learning -- demonstrating that stigmergic signals function as distributed reward mechanisms. Pheromone dynamics mathematically mirror cross-learning updates (a fundamental RL algorithm). In dynamic environments, persistent pheromone trails create positive feedback loops that hinder adaptation by locking swarms into obsolete choices. Introducing a minority of **exploratory agents insensitive to pheromones** restores collective plasticity.
- **Mister Smith Relevance**: **HIGH**. This paper provides the theoretical foundation for implementing digital pheromone trails in NATS KV stores with temporal decay. The insight about exploratory agents maps directly to Mister Smith's agent roles -- some agents could be designated as "explorers" that ignore environmental markers to prevent collective lock-in. The mathematical equivalence between stigmergy and RL means pheromone-style coordination over JetStream KV is not merely a metaphor but a formally grounded coordination mechanism.

### 1.2 Differentiation of Behaviors in Learning Pheromone-Based Communication
- **Authors**: Borghi, Mariani, Zambonelli (2025)
- **Citation Count**: 0
- **Journal**: DCOSS-IoT 2025, pp. 798-804
- **Key Finding**: Agents can **learn** to exploit pheromone-based communication rather than having it hard-coded. Different sub-populations learning concurrently to achieve distinct goals -- using the same pheromone signal -- can coexist with limited interference. This demonstrates that the same environmental marker infrastructure can support multiple concurrent coordination patterns.
- **Mister Smith Relevance**: **HIGH**. Mister Smith's 9 agent roles could each develop distinct interpretations of shared NATS KV markers. A single "task_pressure" key in JetStream KV could be read differently by Researcher agents (as exploration signal) vs. Coder agents (as implementation priority). This avoids the overhead of separate marker systems per role.

### 1.3 Stigmergic Multi-Agent Deep Reinforcement Learning (S-MADRL)
- **Authors**: Aina, Ha (2025)
- **Citation Count**: 0
- **Journal**: ArXiv abs/2510.03592
- **Key Finding**: Proposes virtual pheromones to model local and social interactions in multi-agent coordination, enabling **decentralized emergent coordination without explicit communication**. Uses curriculum learning to decompose complex tasks into progressively harder sub-problems. Agents self-organize into asymmetric workload distributions that reduce congestion.
- **Mister Smith Relevance**: **HIGH**. The asymmetric workload distribution pattern is directly applicable to Mister Smith's agent scheduler. Agents could observe virtual pheromone levels on NATS subjects to self-select tasks, with congestion naturally reducing as pheromone concentration (task claim count) increases on popular subjects.

### 1.4 Trace of Change: Stigmergy in Companion Modeling
- **Authors**: Vendel, Zaitsev, Bommel et al. (2025)
- **Citation Count**: 1
- **Journal**: Journal of Environmental Management 383:125292
- **Key Finding**: Digital traces (stigmergic imprints) can be used as a tool to monitor how learning and model development evolve during iterative workshops. Stigmergy fosters trust, knowledge sharing, and collaboration among diverse stakeholders.
- **Mister Smith Relevance**: **MEDIUM**. The concept of stigmergic traces as an audit and observability mechanism maps to Mister Smith's existing audit logging system. Agent decision traces stored in JetStream KV could serve dual purposes: coordination signals for other agents AND observability data for human operators.

---

## 2. Blackboard Architectures for LLM Multi-Agent Systems

### 2.1 Exploring Advanced LLM Multi-Agent Systems Based on Blackboard Architecture
- **Authors**: Han, Zhang (2025)
- **Citation Count**: 1
- **Journal**: ArXiv abs/2507.01701
- **Key Finding**: First implementation of blackboard architecture for LLM MAS where: (1) agents with various roles share all information during problem-solving, (2) agents taking actions are selected based on current blackboard content, and (3) selection/execution rounds repeat until consensus. **Achieves best average performance while spending fewer tokens** than static and dynamic MAS baselines.
- **Mister Smith Relevance**: **CRITICAL**. This maps almost directly to Mister Smith's NATS JetStream KV as a shared blackboard. The "select agent based on blackboard content" pattern aligns with Mister Smith's existing AgentRegistry + scheduler architecture. The token efficiency advantage is significant for cost-sensitive deployments.

### 2.2 LLM-based Multi-Agent Blackboard System for Information Discovery in Data Science
- **Authors**: Salemi, Parmar, Goyal, Song, Yoon, Zamani, Palangi, Pfister (2025)
- **Citation Count**: 3
- **Journal**: ArXiv abs/2510.01285
- **Key Finding**: Central agent posts requests to shared blackboard; autonomous subordinate agents **volunteer to respond based on their capabilities** -- eliminating the need for a central coordinator to have prior knowledge of sub-agents' expertise. Achieves 13-57% improvement over RAG and master-slave baselines in task success, and up to 9% F1 gain for data discovery.
- **Mister Smith Relevance**: **CRITICAL**. The "volunteer based on capability" model is a natural fit for NATS request-reply semantics. Mister Smith agents could subscribe to task-request subjects and self-select based on their own capability assessment. This eliminates the orchestrator bottleneck and leverages NATS's native pub/sub for capability advertisement.

### 2.3 Terrarium: Revisiting the Blackboard for Multi-Agent Safety, Privacy, and Security Studies
- **Authors**: Nakamura, Kumar, Mahmud, Abdelnabi, Zilberstein, Bagdasarian (2025)
- **Citation Count**: 1
- **Journal**: ArXiv abs/2510.14312
- **Key Finding**: Repurposes the blackboard design as a modular, configurable testbed for multi-agent collaboration with focus on safety. Identifies key attack vectors: misalignment, malicious agents, compromised communication, and data poisoning.
- **Mister Smith Relevance**: **MEDIUM-HIGH**. Relevant to Mister Smith's security architecture (Phase 5). The attack vectors identified (compromised communication, data poisoning on the shared blackboard/KV store) need to be addressed if stigmergic coordination via JetStream KV is adopted.

---

## 3. Swarm Intelligence Applied to LLM Orchestration

### 3.1 Multi-Agent Systems Powered by LLMs: Applications in Swarm Intelligence
- **Authors**: Jimenez-Romero, Yegenoglu, Blum (2025)
- **Citation Count**: 13
- **Journal**: Frontiers in Artificial Intelligence, vol. 8
- **Key Finding**: Replaces hard-coded agent programs with LLM-driven prompts in swarm intelligence simulations (ant colony foraging, bird flocking). Both structured rule-based prompts and autonomous knowledge-driven prompts successfully induce emergent behaviors. Demonstrates that LLMs can reproduce self-organizing processes.
- **Mister Smith Relevance**: **HIGH**. Validates the feasibility of using LLMs to implement swarm-like coordination in software agents. Mister Smith agents could receive environmental context (task board state, agent load metrics) as prompt context and generate coordination decisions naturally, rather than relying on hard-coded scheduling algorithms.

### 3.2 SwarmBench: Benchmarking LLMs' Swarm Intelligence
- **Authors**: Ruan, Huang, Wen, Sun (2025)
- **Citation Count**: 1
- **Journal**: ArXiv abs/2505.04364
- **Key Finding**: Introduces SwarmBench with 5 coordination tasks (Pursuit, Synchronization, Foraging, Flocking, Transport) under strict swarm constraints (limited local perception, local-only communication). **Current LLMs significantly struggle with robust long-range planning and adaptive strategy formation** under informational decentralization. Performance is highly task-dependent.
- **Mister Smith Relevance**: **HIGH** (as cautionary evidence). This reveals the limitations of relying purely on LLMs for swarm coordination -- they need structured environmental support. Mister Smith's architecture (with its supervision trees, event bus, and monitoring infrastructure) provides exactly the structured scaffolding that LLMs need to overcome their coordination weaknesses.

### 3.3 Swarm Intelligence Enhanced Reasoning (SIER)
- **Authors**: Zhu, Zhou, Su, Zhuang, Bai (2025)
- **Citation Count**: 1
- **Journal**: ArXiv abs/2505.17115
- **Key Finding**: Formulates LLM reasoning as an optimization problem and uses swarm intelligence to guide agents in collaboratively searching for optimal solutions. Uses **kernel density estimation and non-dominated sorting** to optimize both solution quality and diversity simultaneously. Step-level quality evaluation enables correction of low-quality intermediate steps. Quality thresholds dynamically control termination.
- **Mister Smith Relevance**: **HIGH**. The density-driven diversity maintenance maps to Mister Smith's agent team coordination. When multiple agents work on the same problem, tracking solution diversity (not just quality) prevents premature convergence. This could be implemented via NATS subjects for solution candidates with metadata tags for diversity tracking.

### 3.4 LLM-Powered Ant Colony Optimization
- **Authors**: Zhang, Wang, Mu (2025)
- **Citation Count**: 0
- **Journal**: AIEA 2025, pp. 440-443
- **Key Finding**: Uses MCTS + LLMs to automatically optimize ACO pheromone update functions for routing problems. LLMs generate novel pheromone update strategies that outperform hand-designed ones.
- **Mister Smith Relevance**: **MEDIUM**. Demonstrates that LLMs can design pheromone update rules. Mister Smith could use meta-agents to tune the parameters of its own coordination mechanisms (decay rates, signal weights) using LLM-generated suggestions.

### 3.5 Language Model Particle Swarm Optimization (LMPSO)
- **Authors**: Shinohara, Xu, Li, Iba (2025)
- **Citation Count**: 2
- **Journal**: IEEE CEC 2025, pp. 1-4
- **Key Finding**: Defines PSO velocity as prompt components, enabling LLM-driven search that respects PSO's foundational principles. Successfully applies to combinatorial optimization (TSP) and heuristic improvement.
- **Mister Smith Relevance**: **MEDIUM**. The idea of encoding optimization state as prompt context is applicable to any multi-agent search problem. Mister Smith's agent teams could use PSO-inspired position/velocity metaphors for exploring solution spaces collaboratively.

### 3.6 CoordField: Coordination Field for Agentic UAV Task Allocation
- **Authors**: Zhang, Tian, Lin, Huang, Suli, Qin, Wang (2025)
- **Citation Count**: 1
- **Journal**: ArXiv abs/2505.00091
- **Key Finding**: Introduces a **coordination field mechanism** -- a continuous pressure field that guides agent motion and task selection, enabling decentralized and adaptive allocation of emergent tasks. LLMs interpret high-level instructions; the coordination field handles low-level real-time allocation.
- **Mister Smith Relevance**: **HIGH**. This is the "pressure fields and environmental markers" concept in concrete form. Mister Smith could implement coordination fields as continuously updated numerical values in JetStream KV, representing task urgency, agent load, skill demand, etc. Agents read the field to determine their next action -- a more continuous and nuanced version of discrete task queues.

---

## 4. Decentralized Coordination and Dynamic Topology

### 4.1 AgentNet: Decentralized Evolutionary Coordination
- **Authors**: Yang, Chai, Shao, Song, Qi, Rui, Zhang (2025)
- **Citation Count**: 16
- **Journal**: ArXiv abs/2504.00587
- **Key Finding**: Agents specialize, evolve, and collaborate autonomously in a dynamically structured DAG. Three innovations: (1) fully decentralized coordination without central orchestrator, (2) dynamic graph topology that adapts in real-time, (3) retrieval-based memory for continual skill refinement. Achieves higher task accuracy than centralized baselines.
- **Mister Smith Relevance**: **HIGH**. AgentNet's dynamic DAG topology maps to dynamic NATS subscription patterns. Agents could adjust their subscription subjects based on evolving expertise, creating de facto dynamic topology. The retrieval-based memory maps to JetStream KV + persistence layer for agent skill profiles.

### 4.2 DynTaskMAS: Dynamic Task Graph-driven Framework
- **Authors**: Yu, Ding, Sato (2025)
- **Citation Count**: 1
- **Journal**: ArXiv abs/2503.07675
- **Key Finding**: Orchestrates asynchronous and parallel operations through dynamic task graphs. Four innovations: Dynamic Task Graph Generator, Asynchronous Parallel Execution Engine, Semantic-Aware Context Management, and Adaptive Workflow Manager. Achieves 21-33% reduction in execution time, 35.4% improvement in resource utilization, and near-linear throughput scaling up to 16 concurrent agents.
- **Mister Smith Relevance**: **CRITICAL**. The asynchronous parallel execution model maps directly to Mister Smith's Tokio runtime + NATS message passing. The semantic-aware context management maps to the existing transport layer's MessageEnvelope with priority and correlation IDs. Near-linear scaling to 16 agents validates the architectural approach.

### 4.3 HiVA: Self-Organized Hierarchical Variable Agent
- **Authors**: Tang, Zhang, Lv, Liu, Yang, Tang, Wang (2025)
- **Citation Count**: 2
- **Journal**: ArXiv abs/2509.00189
- **Key Finding**: Models agentic workflows as self-organized graphs with Semantic-Topological Evolution (STEV) algorithm. Uses **Multi-Armed Bandit-infused forward routing**, textual gradients as discrete-domain surrogates for backpropagation, and coordinated updates that co-evolve semantics and topology. Achieves 5-10% task accuracy improvements.
- **Mister Smith Relevance**: **MEDIUM-HIGH**. The MAB-infused routing could be implemented in Mister Smith's agent scheduler to balance exploration (trying new agent-task assignments) with exploitation (repeating proven assignments). The textual gradient concept suggests that agent performance feedback could be encoded as structured text in NATS messages.

### 4.4 Federation of Agents (FoA): Semantics-Aware Communication Fabric
- **Authors**: Giusti, Werner, Taiello et al. (2025)
- **Citation Count**: 0
- **Journal**: ArXiv abs/2509.20175
- **Key Finding**: Introduces Versioned Capability Vectors (VCVs) -- machine-readable agent capability profiles searchable via semantic embeddings. Three innovations: (1) semantic routing matching tasks to agents via sharded HNSW indices, (2) dynamic task decomposition through consensus-based merging, (3) smart clustering for collaborative refinement. **Built on MQTT publish-subscribe semantics** for scalable message passing. Achieves 13x improvement over single-model baselines.
- **Mister Smith Relevance**: **CRITICAL**. FoA is architecturally the closest match to Mister Smith. It uses MQTT pub/sub (analogous to NATS pub/sub), capability-based agent discovery, and dynamic task decomposition. The VCV concept could be directly implemented in Mister Smith's AgentRegistry, with capability vectors stored in JetStream KV and matched via NATS request-reply patterns.

### 4.5 AgentFlow: Resilient Adaptive Cloud-Edge Framework
- **Authors**: Chen, Shiu (2025)
- **Citation Count**: 0
- **Journal**: ArXiv abs/2505.07603
- **Key Finding**: Supports decentralized publish-subscribe messaging and many-to-many service elections for decision coordination without a central server. Features plug-and-play node discovery, flexible task reorganization, and fault tolerance/substitution mechanisms.
- **Mister Smith Relevance**: **HIGH**. The "many-to-many service elections" pattern maps to NATS queue groups for competitive agent selection. Plug-and-play node discovery maps to Mister Smith's NATS-based agent registration.

---

## 5. Shared Memory, Context Routing, and Indirect Communication

### 5.1 Collaborative Memory: Multi-User Memory Sharing with Dynamic Access Control
- **Authors**: Rezazadeh, Li, Lou, Zhao, Wei, Bao (2025)
- **Citation Count**: 3
- **Journal**: ArXiv abs/2505.18279
- **Key Finding**: Introduces two-tier memory: private fragments (per-originating user) and shared fragments (selectively shared). Each fragment carries immutable provenance attributes (contributing agents, accessed resources, timestamps). Granular read/write policies enforce constraints and support retrospective permission checks.
- **Mister Smith Relevance**: **HIGH**. This maps directly to Mister Smith's dual-store architecture (PostgreSQL + JetStream KV). Private agent memory in KV, shared coordination state in PostgreSQL with audit trails. The provenance attributes align with Mister Smith's existing AuditLogger pattern. The access control model could extend the RBAC system.

### 5.2 RCR-Router: Role-Aware Context Routing for Multi-Agent LLMs
- **Authors**: Liu, Kong, Yang et al. (2025)
- **Citation Count**: 3
- **Journal**: ArXiv abs/2508.04903
- **Key Finding**: First routing approach that dynamically selects semantically relevant memory subsets for each agent based on its role and task stage, under a strict token budget. Agent outputs iteratively integrated into shared memory store for progressive context refinement. **Reduces token usage by up to 30%** while maintaining quality.
- **Mister Smith Relevance**: **HIGH**. The role-aware context filtering could be applied to Mister Smith's 9 agent roles. Rather than broadcasting full context to all agents, the scheduler could use role metadata to select which JetStream KV entries are included in each agent's context window, dramatically reducing token costs.

### 5.3 Knowledge-Aware Iterative Retrieval for Multi-Agent Systems
- **Authors**: Song (2025)
- **Citation Count**: 1
- **Journal**: ArXiv abs/2503.13275
- **Key Finding**: Decouples external sources from an internal knowledge cache that is progressively updated. Mitigates bias-reinforcement loops and enables dynamic, trackable search exploration paths. Benefits of multi-agent configurations become especially prominent as task difficulty increases. **Number of convergence steps scales with task difficulty** -- suggesting cost-effective scalability.
- **Mister Smith Relevance**: **MEDIUM-HIGH**. The progressively updated knowledge cache maps to JetStream KV with watch semantics -- agents can observe real-time updates to shared knowledge. The scalability finding (more agents help more on harder tasks) validates Mister Smith's team-based approach.

---

## 6. Task Allocation, Routing, and Agent Selection

### 6.1 Decentralized Adaptive Task Allocation for Dynamic Multi-Agent Systems
- **Authors**: Tarasova, Erofeeva, Granichin, Chernikov (2025)
- **Citation Count**: 0
- **Journal**: Scientific Reports, vol. 15
- **Key Finding**: Two-layer architecture with adaptive controllers that predict task parameters via recursive regression with forgetting. Selectively broadcasts tasks to agents based on relevance and availability. Uses SPSA + consensus-based synchronization. **Directly evaluated on prompt-based tasks assigned to diverse LLMs**, demonstrating robustness across noise levels and task dynamics.
- **Mister Smith Relevance**: **CRITICAL**. This is the most directly applicable task allocation paper. The "regression with forgetting" is a temporal decay mechanism -- exactly the pheromone evaporation concept applied to task parameter estimation. The selective broadcasting maps to NATS subject-based filtering. The LLM evaluation validates applicability to Mister Smith's use case.

### 6.2 MasRouter: Learning to Route LLMs for Multi-Agent Systems
- **Authors**: Yue, Zhang, Liu et al. (2025)
- **Citation Count**: 17
- **Journal**: pp. 15549-15572
- **Key Finding**: First MASR solution integrating collaboration mode determination, role allocation, and LLM routing through a cascaded controller network. Achieves 1.8-8.2% accuracy improvement while reducing overhead by up to 52% compared to SOTA.
- **Mister Smith Relevance**: **HIGH**. MasRouter's cascaded routing (mode -> role -> LLM) maps to Mister Smith's layered decision architecture: Orchestrator -> Team -> Agent -> LLM. The overhead reduction validates selective routing over broadcast-all approaches.

### 6.3 Symbolic Mixture-of-Experts: Adaptive Skill-based Routing
- **Authors**: Chen, Yun, Stengel-Eskin, Chen, Bansal (2025)
- **Citation Count**: 20
- **Journal**: ArXiv abs/2503.05641
- **Key Finding**: Instance-level expert selection based on skills (not just task type). Skill-based recruiting dynamically selects relevant expert LLMs, with aggregation by a synthesizer. Batch strategy groups instances by assigned experts. **Outperforms GPT-4o-mini with 8.15% avg gain** using weaker models through intelligent routing.
- **Mister Smith Relevance**: **HIGH**. Validates Mister Smith's model-agnostic design. Weaker models routed to appropriate tasks can outperform a single strong model. Mister Smith's ModelProvider trait could support skill-tagged capability profiles for intelligent routing decisions.

### 6.4 MCP-Zero: Active Tool Discovery for Autonomous LLM Agents
- **Authors**: Fei, Zheng, Feng (2025)
- **Citation Count**: 13
- **Journal**: ArXiv abs/2506.01056
- **Key Finding**: Agents actively identify capability gaps and request specific tools on-demand (rather than having all tools injected). Hierarchical Semantic Routing matches requests to relevant MCP servers and tools. **98% reduction in token consumption** while maintaining accuracy. Scales with tool ecosystem growth.
- **Mister Smith Relevance**: **CRITICAL**. Mister Smith already has an MCP crate (`mister-smith-mcp`). MCP-Zero's active discovery pattern could be integrated directly -- agents request tools from the MCP registry based on identified needs rather than being loaded with all tool schemas. The 98% token reduction is transformative for cost control.

### 6.5 DRF: LLM-AGENT Dynamic Reputation Filtering Framework
- **Authors**: Lou, Hu, Ma, Zhang, Wang, Ge, Tao (2025)
- **Citation Count**: 1
- **Journal**: ArXiv abs/2509.05764
- **Key Finding**: Constructs interactive rating networks to quantify agent performance. Reputation scoring measures agent honesty and capability. Upper Confidence Bound-based strategy enhances agent selection efficiency. Significantly improves task quality and collaboration efficiency.
- **Mister Smith Relevance**: **HIGH**. Reputation scores could be stored in JetStream KV with the agent registry. The UCB-based selection strategy naturally balances exploring new agents with exploiting proven performers -- a direct application of the exploration/exploitation tradeoff from swarm intelligence.

### 6.6 NetMCP: Network-Aware Model Context Protocol Platform
- **Authors**: Li, Du, Huang (2025)
- **Citation Count**: 1
- **Journal**: ArXiv abs/2510.13467
- **Key Finding**: Enhances MCP tool routing with real-time network and server status awareness. SONAR algorithm jointly optimizes semantic similarity and network QoS for adaptive tool routing. Improves task success rate and reduces completion time vs. semantic-only baselines.
- **Mister Smith Relevance**: **HIGH**. Directly applicable to Mister Smith's MCP + NATS transport layers. Tool routing should consider not just semantic match but also latency, availability, and load -- all metrics already collected by Mister Smith's monitoring infrastructure (PhiAccrualFailureDetector, HealthMonitor).

---

## 7. Fault Tolerance, Byzantine Resilience, and Consensus

### 7.1 Byzantine Fault Tolerance in Multi-Agent LLM Systems
- **Authors**: Zheng, Chen, Yin, Zhang, Zeng, Tian (2025)
- **Citation Count**: 0
- **Key Finding**: LLM-based agents demonstrate **stronger skepticism** when processing erroneous message flows than traditional agents. Proposes CP-WBFT: confidence probe-based weighted BFT consensus. Achieves superior performance across diverse network topologies under extreme Byzantine conditions (85.7% fault rate).
- **Mister Smith Relevance**: **HIGH**. LLMs' inherent skepticism is a natural defense against cascading failures in Mister Smith's supervision trees. The confidence-weighted consensus could augment Mister Smith's existing OneForOne/OneForAll/RestForOne strategies with Byzantine-aware voting when agents disagree on task outcomes.

### 7.2 Byzantine-Robust Decentralized Coordination of LLM Agents (DecentLLMs)
- **Authors**: Jo, Park (2025)
- **Citation Count**: 0
- **Journal**: ArXiv abs/2507.14928
- **Key Finding**: Decentralized consensus where worker agents generate answers concurrently and evaluator agents independently score/rank to select the best. Eliminates leader vulnerability. Consistently selects higher-quality answers through Byzantine-robust aggregation.
- **Mister Smith Relevance**: **HIGH**. The worker/evaluator split maps to Mister Smith's existing role distinction between task-executing agents and monitoring/quality-assurance agents. Could be implemented as parallel NATS request-reply with aggregation at the orchestrator level.

### 7.3 Achieving Unanimous Consensus in Decision Making Using Multi-Agents
- **Authors**: Pokharel, Dantu, Zaman, Talapuru, Quach (2025)
- **Citation Count**: 1
- **Journal**: ArXiv abs/2504.02128
- **Key Finding**: LLMs as rational agents engaging in structured multi-round deliberation for unanimous consensus. Demonstrates that blockchain properties (consistency, agreement, liveness, determinism) are maintained. Addresses hallucination and degeneration of thoughts.
- **Mister Smith Relevance**: **MEDIUM**. The structured deliberation protocol could be useful for high-stakes decisions in Mister Smith where all agents must agree (e.g., architectural decisions, deployment approvals).

### 7.4 Beyond Majority Voting: LLM Aggregation by Leveraging Higher-Order Information
- **Authors**: Ai, Pan, Simchi-Levi, Tambe, Xu (2025)
- **Citation Count**: 0
- **Journal**: ArXiv abs/2510.01499
- **Key Finding**: Optimal Weight (OW) and Inverse Surprising Popularity (ISP) algorithms leverage both first-order and second-order information for better aggregation than majority voting. Provably mitigates voting limitations under mild assumptions.
- **Mister Smith Relevance**: **MEDIUM-HIGH**. When multiple agents produce candidate outputs, aggregation should not be simple majority vote. The ISP algorithm, which weights agents by how "surprisingly" their answers differ from predictions, could improve Mister Smith's team output synthesis.

---

## 8. Evolutionary and Self-Improving Agent Systems

### 8.1 Darwin Godel Machine: Open-Ended Evolution of Self-Improving Agents
- **Authors**: Zhang, Hu, Lu, Lange, Clune (2025)
- **Citation Count**: 30
- **Journal**: ArXiv abs/2505.22954
- **Key Finding**: Iteratively modifies its own code and empirically validates changes using benchmarks. Maintains an archive of generated agents, samples from it, and creates improved versions. **SWE-bench performance from 20.0% to 50.0%**. Significantly outperforms baselines without self-improvement or open-ended exploration. Open-ended exploration forms a growing tree of diverse, high-quality agents.
- **Mister Smith Relevance**: **HIGH** (forward-looking). The archive-based evolutionary approach could be applied to Mister Smith's prompt templates and agent configurations. Agents could store successful prompt/config combinations in JetStream KV, and a meta-agent could evolve new variations, validating them against task benchmarks.

### 8.2 EvoFlow: Evolving Diverse Agentic Workflows
- **Authors**: Zhang, Chen, Wan, Chang, Cheng, Wang, Hu, Bai (2025)
- **Citation Count**: 16
- **Journal**: ArXiv abs/2502.07373
- **Key Finding**: Niching evolutionary algorithm to search a population of heterogeneous, complexity-adaptive workflows (not a single homogeneous one). Outperforms handcrafted workflows by 1.23-29.86%. **Surpasses o1-preview at 12.4% of inference cost** using weaker open-source models.
- **Mister Smith Relevance**: **HIGH**. EvoFlow's workflow evolution could be integrated with Mister Smith's team orchestrator. Multiple team configurations could coexist and evolve, with the system maintaining a diverse population of effective workflows rather than converging on a single "best" one.

### 8.3 SE-Agent: Self-Evolution Trajectory Optimization
- **Authors**: Lin, Guo, Han et al. (2025)
- **Citation Count**: 9
- **Journal**: ArXiv abs/2508.02085
- **Key Finding**: Revisits and enhances former pilot trajectories through revision, recombination, and refinement (evolutionary operations). Expands search space beyond local optima by exploiting cross-trajectory inspiration. Up to 55% relative improvement on SWE-bench.
- **Mister Smith Relevance**: **MEDIUM-HIGH**. Task trajectory storage in JetStream + PostgreSQL could enable Mister Smith agents to learn from past execution traces, applying evolutionary operators to generate improved approaches for recurring task patterns.

### 8.4 SEW: Self-Evolving Agentic Workflows
- **Authors**: Liu, Fang, Zhou, Wang, Meng (2025)
- **Citation Count**: 5
- **Journal**: ArXiv abs/2505.18646
- **Key Finding**: Automatically generates and optimizes multi-agent workflows. Up to 33% improvement on LiveCodeBench. Investigates different workflow representation schemes for optimal text-based encoding.
- **Mister Smith Relevance**: **MEDIUM**. The workflow representation investigation is relevant to how Mister Smith serializes and communicates team configurations across NATS.

---

## 9. Multi-Agent Safety and Adversarial Robustness

### 9.1 MedSentry: Safety Risks in Medical LLM Multi-Agent Systems
- **Authors**: Chen, Zhen, Wang et al. (2025)
- **Citation Count**: 6
- **Journal**: ArXiv abs/2505.20824
- **Key Finding**: Compares four multi-agent topologies (Layers, SharedPool, Centralized, Decentralized) for adversarial resilience. **SharedPool (open information sharing) is highly susceptible; Decentralized architectures exhibit greater resilience** due to inherent redundancy and isolation. Personality-scale detection identifies and rehabilitates malicious agents.
- **Mister Smith Relevance**: **CRITICAL** (for security architecture). Mister Smith's NATS pub/sub is essentially a SharedPool topology for messages. This paper warns that open information sharing creates vulnerability. Mister Smith should implement message-level access controls and agent isolation mechanisms to gain the resilience benefits of decentralized topologies while keeping pub/sub convenience.

### 9.2 AdvEvo-MARL: Adversarial Co-Evolution for Internalized Safety
- **Authors**: Pan, Zhang, Liu et al. (2025)
- **Citation Count**: 0
- **Journal**: ArXiv abs/2510.01586
- **Key Finding**: Co-evolutionary framework that **internalizes safety into task agents** rather than relying on external guards. Jointly optimizes attackers and defenders. Keeps attack-success rate below 20% while preserving task accuracy. Shows safety and utility can be jointly improved.
- **Mister Smith Relevance**: **MEDIUM-HIGH**. Suggests Mister Smith's security shouldn't be solely in the SecurityLayer middleware but should be embedded in agent prompt design. Agents should be trained/prompted to recognize and resist unsafe instructions as part of their core behavior.

### 9.3 TAMAS: Benchmarking Adversarial Risks in Multi-Agent LLM Systems
- **Authors**: Kavathekar, Jain, Rathod, Kumaraguru, Ganu (2025)
- **Citation Count**: 0
- **Key Finding**: Introduces Effective Robustness Score (ERS) to assess tradeoff between safety and task effectiveness. Multi-agent systems are **highly vulnerable to adversarial attacks**, with significant failure modes in current deployments.
- **Mister Smith Relevance**: **MEDIUM**. The ERS metric could be adopted as a quality gate for Mister Smith deployments, ensuring that security measures don't degrade task performance below acceptable thresholds.

---

## 10. Hierarchical Coordination and Theoretical Foundations

### 10.1 A Taxonomy of Hierarchical Multi-Agent Systems
- **Authors**: Moore (2025)
- **Citation Count**: 2
- **Journal**: ArXiv abs/2508.12683
- **Key Finding**: First taxonomy unifying structural, temporal, and communication dimensions of hierarchical MAS. Five axes: control hierarchy, information flow, role/task delegation, temporal layering, communication structure. Bridges classical coordination (contract-net protocol) with modern RL and LLM agents. Identifies open challenges: explainability, scaling, and safe LLM integration into layered frameworks.
- **Mister Smith Relevance**: **HIGH**. Provides the theoretical framing for Mister Smith's supervision tree architecture. Mister Smith's design spans multiple axes of this taxonomy, and the framework could be used to evaluate design decisions (e.g., how much control hierarchy vs. how much autonomy at each level).

### 10.2 Coordination Requires Simplification: Thermodynamic Bounds (TCT)
- **Authors**: Anand (2025)
- **Citation Count**: 0
- **Journal**: ArXiv abs/2509.23144
- **Key Finding**: Derives information-theoretic minimum description length for coordination protocols: L(P) >= NK log2(K) + N^2 d^2 log(1/epsilon). Coordination forces progressive simplification. Defines "coordination temperature" to predict critical phenomena. Finds that coordination dynamics change the environment itself, creating metastable states.
- **Mister Smith Relevance**: **HIGH** (theoretical). This provides hard bounds on coordination complexity. As Mister Smith scales to more agents (N) with more conflicting objectives (d), the coordination protocol must simplify -- suggesting that stigmergic (environment-mediated) coordination scales better than explicit messaging because it naturally reduces protocol complexity.

### 10.3 Hierarchical Message-Passing Policies for Multi-Agent RL
- **Authors**: Marzi, Alippi, Cini (2025)
- **Citation Count**: 0
- **Journal**: ArXiv abs/2507.23604
- **Key Finding**: Hierarchical graph structure for planning and coordination. Agents at lower hierarchy levels receive goals from upper levels and exchange messages with same-level neighbors. Novel reward assignment trains lower-level policies to maximize upper-level advantage functions.
- **Mister Smith Relevance**: **MEDIUM-HIGH**. Maps to Mister Smith's supervision tree structure where supervisors set goals and workers execute. The reward propagation concept could inform how task completion feedback flows up through supervision hierarchies.

---

## 11. Comprehensive Surveys and Taxonomies

### 11.1 Multi-Agent Collaboration Mechanisms: A Survey of LLMs
- **Authors**: Tran, Dao, Nguyen, Pham, O'Sullivan, Nguyen (2025)
- **Citation Count**: 190
- **Journal**: ArXiv abs/2501.06322
- **Key Finding**: Characterizes collaboration along five dimensions: actors, types (cooperation/competition/coopetition), structures (peer-to-peer/centralized/distributed), strategies (role-based/model-based), and coordination protocols. Identifies path toward **artificial collective intelligence**.
- **Mister Smith Relevance**: **HIGH** (foundational survey). Provides the conceptual vocabulary for describing Mister Smith's coordination mechanisms. Mister Smith implements cooperation (team agents) with distributed structure (NATS) using role-based strategy (9 agent types) -- this paper validates that combination.

### 11.2 AI Agents vs. Agentic AI: A Conceptual Taxonomy
- **Authors**: Sapkota, Roumeliotis, Karkee (2025)
- **Citation Count**: 103
- **Journal**: Information Fusion 126:103599
- **Key Finding**: Distinguishes AI Agents (modular, task-specific) from Agentic AI (multi-agent collaboration, dynamic task decomposition, persistent memory, coordinated autonomy). Proposes targeted solutions including ReAct loops, RAG, automation coordination layers, and causal modeling.
- **Mister Smith Relevance**: **MEDIUM**. Mister Smith is firmly in the "Agentic AI" category per this taxonomy. The coordination failure solutions (ReAct, RAG, coordination layers) are relevant design patterns.

### 11.3 Advances and Challenges in Foundation Agents
- **Authors**: Liu, Li, Zhang et al. (2025)
- **Citation Count**: 77
- **Journal**: ArXiv abs/2504.01990
- **Key Finding**: Comprehensive book-length survey framing agents within brain-inspired modular architectures. Covers memory, world modeling, reward processing, self-enhancement, multi-agent collective intelligence, and safety. Identifies multi-agent collective intelligence emerging from interactions, cooperation, and societal structures.
- **Mister Smith Relevance**: **MEDIUM**. Provides the broadest context for Mister Smith's position in the agent landscape. The brain-inspired modular architecture framing aligns with Mister Smith's crate-per-concern design.

### 11.4 From LLM Reasoning to Autonomous AI Agents
- **Authors**: Ferrag, Tihanyi, Debbah (2025)
- **Citation Count**: 53
- **Journal**: ArXiv abs/2504.19678
- **Key Finding**: Reviews agent-to-agent collaboration protocols: ACP, MCP, and A2A. Identifies dynamic tool integration via RL, failure modes in multi-agent systems, and security vulnerabilities in agent protocols as key research directions.
- **Mister Smith Relevance**: **MEDIUM-HIGH**. MCP and A2A protocol analysis is directly relevant to Mister Smith's MCP crate and transport layer.

### 11.5 Agentic AI Frameworks: Architectures, Protocols, and Design Challenges
- **Authors**: Derouiche, Brahmi, Mazeni (2025)
- **Citation Count**: 3
- **Journal**: ArXiv abs/2508.10146
- **Key Finding**: Comparative analysis of CrewAI, LangGraph, AutoGen, Semantic Kernel, Agno, Google ADK, MetaGPT. In-depth analysis of Contract Net Protocol, A2A, Agent Network Protocol, and Agora communication protocols.
- **Mister Smith Relevance**: **MEDIUM**. Provides competitive landscape analysis and protocol comparison for positioning Mister Smith's architecture against existing frameworks.

---

## 12. Emerging Directions

The most novel and speculative findings from this research survey point to several frontier areas:

### 12.1 Stigmergy-RL Equivalence as a Design Principle
The Vellinger et al. (2025) proof that pheromone dynamics mathematically mirror RL cross-learning updates transforms stigmergy from biological metaphor into a formal coordination primitive. For distributed systems like Mister Smith, this means that simple environmental markers with temporal decay (KV entries with TTL) are not ad-hoc coordination hacks but theoretically grounded distributed learning mechanisms. **This is the most important theoretical result for Mister Smith's future coordination layer.**

### 12.2 Coordination Fields as Continuous Pressure Landscapes
CoordField's (Zhang et al. 2025) continuous coordination fields represent a departure from discrete task queues. Instead of binary "task available / task claimed" states, coordination fields provide gradient information -- how urgently a task needs attention, how saturated a capability area is, how much spare capacity exists. This continuous representation enables smoother, more responsive agent behavior and avoids the thundering-herd problem of discrete task announcements. **Implementation via NATS KV with numeric values and watch semantics is straightforward.**

### 12.3 Self-Evolving Agent Architectures
The Darwin Godel Machine (Zhang et al. 2025) and EvoFlow (Zhang et al. 2025) demonstrate that agent systems can autonomously discover better configurations -- including better coordination protocols. The implication for Mister Smith is that the coordination layer itself should be evolvable: agent prompt templates, scheduling heuristics, and even team compositions could be subject to evolutionary optimization, with JetStream providing the persistent archive of configurations and their performance histories.

### 12.4 Thermodynamic Limits on Coordination Complexity
Anand's (2025) Thermodynamic Coordination Theory provides hard scaling bounds. As N agents coordinate on d objectives, protocol complexity grows as N^2 * d^2. This has direct implications for Mister Smith: beyond a certain agent count, explicit coordination becomes infeasible and the system must shift toward simpler, environment-mediated (stigmergic) mechanisms. **This provides a principled criterion for when to switch from orchestrated to stigmergic coordination.**

### 12.5 Topology-Dependent Security
MedSentry's (Chen et al. 2025) finding that SharedPool topologies are most vulnerable while Decentralized architectures are most resilient suggests that Mister Smith's pub/sub backbone (inherently a shared pool) needs deliberate compartmentalization. Future work should explore namespace-based isolation in NATS (already supported via JetStream domains) to create topology-dependent security zones.

### 12.6 Active Tool Discovery via MCP
MCP-Zero's (Fei et al. 2025) 98% token reduction through active tool discovery is transformative. Rather than loading all tool schemas into agent context, agents should identify capability gaps and request specific tools. This is directly implementable in Mister Smith's existing MCP infrastructure.

---

## 13. Synthesis: Implications for Mister Smith

### Architecture Alignment

Mister Smith's existing architecture (NATS pub/sub, JetStream KV, supervision trees, 9 agent roles, EventBus) is remarkably well-positioned to implement the coordination patterns emerging from this research. Specific alignments:

| Research Concept | Mister Smith Primitive | Implementation Path |
|---|---|---|
| Digital pheromones | JetStream KV with TTL | Store coordination signals as KV entries with configurable expiry |
| Blackboard architecture | JetStream KV + NATS subjects | Shared problem state in KV, agent selection via subject-based pub/sub |
| Coordination fields | JetStream KV numeric values + watch | Continuous metrics (urgency, saturation, capacity) as watchable KV keys |
| Dynamic topology | NATS subscription management | Agents adjust subscriptions based on evolving capabilities |
| Capability vectors | AgentRegistry + KV | Store skill/capability profiles; match via NATS request-reply |
| Reputation scoring | JetStream KV + persistence | Track agent performance history; UCB-based selection |
| Byzantine resilience | Supervision trees + voting | Multi-agent validation with confidence-weighted consensus |
| Active tool discovery | MCP crate | On-demand tool request via MCP with semantic routing |

### Recommended Research-Informed Priorities

1. **Implement stigmergic coordination layer** using JetStream KV with temporal decay (TTL). This is theoretically grounded (Vellinger 2025), practically validated (Borghi 2025, Aina 2025), and maps directly to existing infrastructure.

2. **Add blackboard-style shared problem state** with volunteer-based agent selection (Salemi 2025). Agents subscribe to problem-type subjects and self-select based on capability match -- eliminating the orchestrator bottleneck.

3. **Integrate active tool discovery** (Fei 2025) into the MCP crate for dramatic token cost reduction.

4. **Implement coordination fields** (Zhang 2025) as continuously updated JetStream KV values representing task urgency, agent load, and capability demand.

5. **Add reputation-based agent selection** (Lou 2025) with UCB exploration strategy to the agent scheduler, balancing exploitation of proven agents with exploration of new assignments.

6. **Design topology-aware security zones** (Chen 2025) using NATS JetStream domains to compartmentalize agent communication and reduce SharedPool vulnerability.

### Cautionary Findings

- **LLMs struggle with pure swarm coordination** (Ruan 2025) -- they need structured environmental scaffolding, which Mister Smith's architecture provides.
- **Coordination complexity scales as N^2 * d^2** (Anand 2025) -- Mister Smith should plan for a transition from explicit to stigmergic coordination as agent count grows.
- **Open information sharing (SharedPool) is the most vulnerable topology** (Chen 2025) -- Mister Smith's pub/sub model needs deliberate access controls.
- **Persistent pheromone trails can lock systems into obsolete choices** (Vellinger 2025) -- temporal decay (TTL) and designated explorer agents are essential.

---

## Paper Index (Alphabetical by First Author)

| # | Authors | Title | Year | Citations | Section |
|---|---|---|---|---|---|
| 1 | Ai et al. | Beyond Majority Voting: LLM Aggregation | 2025 | 0 | 7.4 |
| 2 | Aina, Ha | S-MADRL: Deep RL for Multi-Agent Coordination | 2025 | 0 | 1.3 |
| 3 | Anand | Thermodynamic Coordination Theory | 2025 | 0 | 10.2 |
| 4 | Borghi, Mariani, Zambonelli | Differentiation of Behaviors in Pheromone Communication | 2025 | 0 | 1.2 |
| 5 | Chen, Shiu | AgentFlow: Resilient Adaptive Cloud-Edge Framework | 2025 | 0 | 4.5 |
| 6 | Chen, Zhen et al. | MedSentry: Safety Risks in Medical LLM MAS | 2025 | 6 | 9.1 |
| 7 | Derouiche, Brahmi, Mazeni | Agentic AI Frameworks: Architectures and Protocols | 2025 | 3 | 11.5 |
| 8 | Fei, Zheng, Feng | MCP-Zero: Active Tool Discovery | 2025 | 13 | 6.4 |
| 9 | Ferrag, Tihanyi, Debbah | From LLM Reasoning to Autonomous AI Agents | 2025 | 53 | 11.4 |
| 10 | Giusti et al. | Federation of Agents: Semantics-Aware Communication Fabric | 2025 | 0 | 4.4 |
| 11 | Han, Zhang | Blackboard Architecture for LLM MAS | 2025 | 1 | 2.1 |
| 12 | Jimenez-Romero et al. | Multi-Agent Systems Powered by LLMs: Swarm Intelligence | 2025 | 13 | 3.1 |
| 13 | Jo, Park | DecentLLMs: Byzantine-Robust Decentralized Coordination | 2025 | 0 | 7.2 |
| 14 | Kavathekar et al. | TAMAS: Benchmarking Adversarial Risks | 2025 | 0 | 9.3 |
| 15 | Li, Du, Huang | NetMCP: Network-Aware MCP Platform | 2025 | 1 | 6.6 |
| 16 | Li, Zhou | LLM-Flock: Decentralized Multi-Robot Flocking | 2025 | 2 | 3.6 (ref) |
| 17 | Lin, Guo et al. | SE-Agent: Self-Evolution Trajectory Optimization | 2025 | 9 | 8.3 |
| 18 | Liu, Fang et al. | SEW: Self-Evolving Agentic Workflows | 2025 | 5 | 8.4 |
| 19 | Liu, Li, Zhang et al. | Advances and Challenges in Foundation Agents | 2025 | 77 | 11.3 |
| 20 | Liu, Kong et al. | RCR-Router: Role-Aware Context Routing | 2025 | 3 | 5.2 |
| 21 | Lou et al. | DRF: Dynamic Reputation Filtering Framework | 2025 | 1 | 6.5 |
| 22 | Marzi, Alippi, Cini | Hierarchical Message-Passing Policies | 2025 | 0 | 10.3 |
| 23 | Moore | Taxonomy of Hierarchical Multi-Agent Systems | 2025 | 2 | 10.1 |
| 24 | Nakamura et al. | Terrarium: Blackboard for Multi-Agent Security | 2025 | 1 | 2.3 |
| 25 | Pan, Zhang et al. | AdvEvo-MARL: Adversarial Co-Evolution Safety | 2025 | 0 | 9.2 |
| 26 | Pokharel et al. | Achieving Unanimous Consensus | 2025 | 1 | 7.3 |
| 27 | Qi, Zhu et al. | Blockchain-Driven Decentralized LLM MAS | 2025 | 2 | 4.1 (ref) |
| 28 | Rezazadeh et al. | Collaborative Memory: Multi-User Sharing | 2025 | 3 | 5.1 |
| 29 | Rjoub et al. | Hybrid Swarm Intelligence for MLLM Deployment | 2025 | 6 | 3.1 (ref) |
| 30 | Ruan et al. | SwarmBench: Benchmarking LLMs' Swarm Intelligence | 2025 | 1 | 3.2 |
| 31 | Salemi et al. | LLM-based Multi-Agent Blackboard for Data Science | 2025 | 3 | 2.2 |
| 32 | Sapkota et al. | AI Agents vs. Agentic AI Taxonomy | 2025 | 103 | 11.2 |
| 33 | Shinohara et al. | LLMs as Particle Swarm Optimizers (LMPSO) | 2025 | 2 | 3.5 |
| 34 | Song | Knowledge-Aware Iterative Retrieval for MAS | 2025 | 1 | 5.3 |
| 35 | Tang et al. | HiVA: Self-Organized Hierarchical Variable Agent | 2025 | 2 | 4.3 |
| 36 | Tarasova et al. | Decentralized Adaptive Task Allocation | 2025 | 0 | 6.1 |
| 37 | Tran et al. | Multi-Agent Collaboration Mechanisms: Survey | 2025 | 190 | 11.1 |
| 38 | Vellinger et al. | From Pheromones to Policies: Stigmergy-RL Equivalence | 2025 | 1 | 1.1 |
| 39 | Vendel et al. | Trace of Change: Stigmergy in Companion Modeling | 2025 | 1 | 1.4 |
| 40 | Yang et al. | AgentNet: Decentralized Evolutionary Coordination | 2025 | 16 | 4.1 |
| 41 | Yang et al. | Multi-LLM Collaborative Search (MoSA) | 2025 | 7 | 8 (ref) |
| 42 | Yu, Ding, Sato | DynTaskMAS: Dynamic Task Graph Framework | 2025 | 1 | 4.2 |
| 43 | Yue et al. | MasRouter: Learning to Route LLMs for MAS | 2025 | 17 | 6.2 |
| 44 | Zeng et al. | S2-MAD: Token-Efficient Multi-Agent Debate | 2025 | 7 | 6 (ref) |
| 45 | Zhang, Chen et al. | EvoFlow: Evolving Diverse Agentic Workflows | 2025 | 16 | 8.2 |
| 46 | Zhang, Hu et al. | Darwin Godel Machine: Self-Improving Agents | 2025 | 30 | 8.1 |
| 47 | Zhang, Tian et al. | CoordField: Coordination Field for UAV Tasks | 2025 | 1 | 3.6 |
| 48 | Zhang, Wang, Mu | LLM-Powered Ant Colony Optimization | 2025 | 0 | 3.4 |
| 49 | Zheng et al. | BFT Reliability in Multi-Agent LLM Systems | 2025 | 0 | 7.1 |
| 50 | Zhu et al. | SIER: Swarm Intelligence Enhanced Reasoning | 2025 | 1 | 3.3 |
| 51 | Chen, Yun et al. | Symbolic-MoE: Skill-based Routing | 2025 | 20 | 6.3 |
| 52 | Guo et al. | MoMA: Generalized Model and Agent Routing | 2025 | 0 | 6.2 (ref) |

---

*Total unique papers catalogued: 52*
*Search queries executed: 22*
*Coverage: stigmergy, pheromone coordination, blackboard architectures, swarm intelligence, decentralized coordination, shared memory, task allocation, agent routing, fault tolerance, Byzantine resilience, consensus, evolutionary optimization, self-improving agents, multi-agent safety, hierarchical coordination, thermodynamic bounds, MCP integration*
