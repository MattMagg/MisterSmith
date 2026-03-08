---
version: R4
created: 2026-03-07
updated: 2026-03-07
sources: Consensus (49 papers, 20+ searches)
round: 4 (Academic Search)
---

# Research Digest: Supervision and Fault Tolerance for Non-Deterministic Multi-Agent Systems

**Search Period:** 2025 -- present (mid-2025 to early 2026)
**Search Tool:** Consensus Academic Search (200M+ papers: Semantic Scholar, PubMed, ArXiv, IEEE, ACM, etc.)
**Date Compiled:** 2026-03-07
**Relevance Target:** Mister Smith framework -- Rust + NATS/JetStream + OTP-style supervision trees, model-agnostic multi-agent orchestration

---

## Table of Contents

1. [MAS Failure Taxonomies and Root Cause Analysis](#1-mas-failure-taxonomies-and-root-cause-analysis)
2. [Byzantine Fault Tolerance for LLM-Based MAS](#2-byzantine-fault-tolerance-for-llm-based-mas)
3. [Error Cascade Prevention and Communication Topology](#3-error-cascade-prevention-and-communication-topology)
4. [Saga Patterns and Transactional Guarantees for Agent Workflows](#4-saga-patterns-and-transactional-guarantees-for-agent-workflows)
5. [Tool-Calling Failure Recovery](#5-tool-calling-failure-recovery)
6. [Chaos Engineering for LLM Multi-Agent Systems](#6-chaos-engineering-for-llm-multi-agent-systems)
7. [Self-Healing and Continuous Oversight Architectures](#7-self-healing-and-continuous-oversight-architectures)
8. [Agent Observability and Telemetry](#8-agent-observability-and-telemetry)
9. [Formal Verification of Actor and Multi-Agent Systems](#9-formal-verification-of-actor-and-multi-agent-systems)
10. [Fault-Recovering Actor Garbage Collection](#10-fault-recovering-actor-garbage-collection)
11. [LLM Provider Resilience: Routing, Rate Limiting, and Serving](#11-llm-provider-resilience-routing-rate-limiting-and-serving)
12. [Resilience Patterns for Distributed Systems (Circuit Breakers, Bulkheads, DLQ)](#12-resilience-patterns-for-distributed-systems)
13. [Runtime Governance and Graduated Containment](#13-runtime-governance-and-graduated-containment)
14. [Agent Memory and Long-Horizon State Management](#14-agent-memory-and-long-horizon-state-management)
15. [Testing Non-Deterministic Agent Systems](#15-testing-non-deterministic-agent-systems)
16. [Messaging Infrastructure Benchmarks](#16-messaging-infrastructure-benchmarks)
17. [Emerging Directions](#17-emerging-directions)

---

## 1. MAS Failure Taxonomies and Root Cause Analysis

### Why Do Multi-Agent LLM Systems Fail? (MAST)
- **Authors:** Cemri, Pan, Yang, Agrawal, Chopra, Tiwari, Keutzer, Parameswaran, Klein, Ramchandran, Zaharia, Gonzalez, Stoica
- **Year:** 2025 | **Citations:** 134 | **Venue:** ArXiv
- **Key Finding:** First comprehensive failure taxonomy for multi-agent LLM systems. Analyzed 1600+ annotated execution traces across 7 popular MAS frameworks (AutoGen, CrewAI, LangGraph, etc.). Identified **14 unique failure modes** clustered into 3 categories: (i) system design issues, (ii) inter-agent misalignment, and (iii) task verification failures. High inter-annotator agreement (kappa = 0.88).
- **Mister Smith Relevance:** **Critical.** The MAST taxonomy should directly inform Mister Smith's supervision strategies. The 3-category failure model maps to different supervision tree levels: system design issues -> orchestrator-level supervision; inter-agent misalignment -> team-level supervision (OneForAll restart); task verification -> individual agent supervision (OneForOne restart). The finding that failures require "more sophisticated solutions" than simple retry validates Mister Smith's multi-strategy supervision approach.

### Diagnosing Failure Root Causes in Platform-Orchestrated Agentic Systems (AgentFail)
- **Authors:** Ma, Xie, Wang, Wang, Wu, Li, Wang
- **Year:** 2025 | **Citations:** 0 | **Venue:** ArXiv
- **Key Finding:** Constructed AgentFail dataset with 307 failure logs from 10 agentic systems with fine-grained root cause annotations. Used counterfactual reasoning-based repair strategy for annotation reliability. Root cause identification by LLMs reaches only 33.6% accuracy, indicating this remains a hard problem. Provides actionable guidelines for building more reliable agentic systems.
- **Mister Smith Relevance:** The low accuracy of automated root cause identification validates the need for Mister Smith's structured supervision approach (pre-defined strategies) rather than relying on agents to self-diagnose. The failure taxonomy can inform the AuditLogger's event categories.

### MedAgentAudit: Collaborative Failure Modes in Medical Multi-Agent Systems
- **Authors:** Gu, Zhu, Sang, Wang, Sui, Tang, Harrison, Gao, Yu, Ma
- **Year:** 2025 | **Citations:** 0 | **Venue:** ArXiv
- **Key Finding:** Large-scale study of 3,600 cases across 6 medical datasets and 6 MAS frameworks. Identified 4 dominant failure patterns: (1) flawed consensus from shared model deficiencies, (2) suppression of correct minority opinions, (3) ineffective discussion dynamics, (4) critical information loss during synthesis. Demonstrates that high accuracy alone is insufficient -- auditable reasoning processes are essential.
- **Mister Smith Relevance:** The "suppression of correct minority opinions" failure mode is relevant to Mister Smith's team-based agent orchestration. Supervision strategies should not simply restart failing agents but consider whether the "failure" was actually correct minority output being suppressed. The audit trail requirement aligns with Mister Smith's Phase 5 audit logging infrastructure.

---

## 2. Byzantine Fault Tolerance for LLM-Based MAS

### Rethinking MAS Reliability: A Perspective from Byzantine Fault Tolerance (CP-WBFT)
- **Authors:** Zheng, Chen, Yin, Zhang, Zeng, Tian
- **Year:** 2025 | **Citations:** 0 | **Venue:** Unknown Journal
- **Key Finding:** First work to quantify LLM-based agent reliability through Byzantine fault tolerance lens. Key insight: LLM-based agents demonstrate **stronger skepticism** when processing erroneous message flows compared to traditional agents. Proposed CP-WBFT (confidence probe-based weighted BFT) consensus mechanism. Achieves superior performance under **extreme 85.7% fault rate** across diverse network topologies.
- **Mister Smith Relevance:** **High.** The confidence-probe approach could be adapted for Mister Smith's health monitoring. When agents produce outputs, a lightweight confidence probe could feed into the PhiAccrualFailureDetector to distinguish between genuine failures and valid minority opinions. The 85.7% fault tolerance rate provides a theoretical upper bound for system design.

### Weighted BFT Consensus for Trusted Multi-LLM Networks (WBFT)
- **Authors:** Luo, Sun, Liu, Zhao, Niyato, Yu, Dustdar
- **Year:** 2025 | **Citations:** 7 | **Venue:** ArXiv
- **Key Finding:** Proposes a blockchain-inspired WBFT consensus mechanism for multi-LLM collaboration. Voting weights are adaptively assigned based on response quality and trustworthiness. Addresses single points of failure in centralized coordination. Demonstrated improved consensus security and efficiency, especially under wireless network conditions.
- **Mister Smith Relevance:** The adaptive weight assignment based on trustworthiness maps to Mister Smith's agent role system. High-performing agents (e.g., Architect, Lead Developer) could accumulate higher trust weights over time, informing the orchestrator's decision about which agent outputs to prioritize during team coordination.

### MARTA: Fault-Tolerant Multi-Agent Learning with Adversarial Budget Constraints
- **Authors:** Mguni, Sun, Chen, Darabi, Orimoloye, Yang
- **Year:** 2025 | **Citations:** 0 | **Venue:** ArXiv
- **Key Finding:** Plug-and-play framework for training MARL agents to be resilient to severe faults. Uses adversarial Markov game with Markov switching controls to model agent disabling. Enforces a "malfunction budget" to constrain the adversary. Provides theoretical convergence guarantees to Markov perfect equilibrium.
- **Mister Smith Relevance:** The "malfunction budget" concept could inform Mister Smith's supervision configuration -- defining maximum acceptable failure rates before escalating restart strategies (e.g., switching from OneForOne to OneForAll when the budget is exceeded).

---

## 3. Error Cascade Prevention and Communication Topology

### AgentAsk: Multi-Agent Systems Need to Ask
- **Authors:** Li, Yang, Lai, Zhang, Zhang, Zhang, Yu, Yu, Wang, Wang
- **Year:** 2025 | **Citations:** 0 | **Venue:** ArXiv
- **Key Finding:** Demonstrates that MAS frequently underperform single-agent baselines due to **edge-level error cascades** -- minor inaccuracies at message handoffs propagate across the chain. Proposes a lightweight clarification module that treats every inter-agent message as a potential failure point. Architecture-agnostic, with <5% overhead. Provides a principled taxonomy of edge-level errors.
- **Mister Smith Relevance:** **High.** Mister Smith's NATS-based message passing between agents should incorporate validation at message boundaries. The AgentAsk approach of "link-local intervention" maps directly to message envelope validation in the Transport layer -- each MessageEnvelope could carry a confidence score, and the receiving agent could request clarification before processing.

### Understanding Information Propagation Effects of Communication Topologies (EIB-learner)
- **Authors:** Shen, Liu, Dai, Wang, Miao, Tan, Pan, Wang
- **Year:** 2025 | **Citations:** 4 | **Venue:** ArXiv
- **Key Finding:** Presents a causal framework analyzing how correct and erroneous outputs propagate under topologies with varying sparsity. Key insight: **moderately sparse topologies** optimally suppress error propagation while preserving beneficial information diffusion. Proposes EIB-learner that balances error suppression and beneficial information propagation.
- **Mister Smith Relevance:** Informs the design of Mister Smith's team communication patterns. Rather than fully-connected agent teams, the orchestrator should design moderately sparse communication graphs. This validates the existing architecture where agents communicate through the orchestrator rather than directly peer-to-peer.

### GUARDIAN: Safeguarding LLM Multi-Agent Collaborations with Temporal Graph Modeling
- **Authors:** Zhou, Wang, Yang
- **Year:** 2025 | **Citations:** 4 | **Venue:** ArXiv
- **Key Finding:** Models multi-agent collaboration as a discrete-time temporal attributed graph to capture hallucination and error propagation dynamics. Uses unsupervised encoder-decoder with incremental training to detect anomalous nodes and edges. Introduces graph abstraction via Information Bottleneck Theory for efficiency.
- **Mister Smith Relevance:** The temporal graph modeling approach could enhance Mister Smith's monitoring layer. Agent interactions over NATS form a natural graph, and the EventBus already captures these interactions. A lightweight anomaly detector on the event stream could identify propagating errors before they cascade.

### Randomized Smoothing for LLM-Driven MAS Robustness
- **Authors:** Hu, Dong, Ding, Huang
- **Year:** 2025 | **Citations:** 5 | **Venue:** ArXiv
- **Key Finding:** Applies randomized smoothing (a statistical robustness certification technique) to MAS consensus, enabling probabilistic guarantees on agent decisions under adversarial influence. Works in black-box settings with a two-stage adaptive sampling mechanism. Effectively prevents propagation of adversarial behaviors and hallucinations.
- **Mister Smith Relevance:** Provides a theoretical foundation for adding probabilistic safety guarantees to Mister Smith's agent output validation. The black-box nature is important since Mister Smith is model-agnostic.

---

## 4. Saga Patterns and Transactional Guarantees for Agent Workflows

### SagaLLM: Context Management, Validation, and Transaction Guarantees for Multi-Agent LLM Planning
- **Authors:** Chang, Geng
- **Year:** 2025 | **Citations:** 11 | **Venue:** PVLDB (top database venue)
- **Key Finding:** Integrates the Saga transactional pattern with persistent memory, automated compensation, and independent validation agents for multi-agent LLM planning. Addresses: unreliable self-validation, context loss, lack of transactional safeguards, and insufficient inter-agent coordination. Uses modular checkpointing and compensable execution. Relaxes ACID but ensures workflow-wide consistency.
- **Mister Smith Relevance:** **Critical.** Directly validates the Saga pattern approach for Mister Smith's agent task orchestration. The compensation mechanism maps to supervision tree rollback strategies. SagaLLM's "independent validation agents" pattern could be implemented as a dedicated Validator agent role. The modular checkpointing approach aligns with Mister Smith's JetStream KV for state persistence.

### Saga Pattern Review for Distributed Transactions in Microservices
- **Authors:** Neelan
- **Year:** 2025 | **Citations:** 0 | **Venue:** IJMR
- **Key Finding:** Comprehensive review comparing orchestration-based and choreography-based Saga implementations. Identifies key challenges: compensation logic complexity, idempotency requirements, observability needs, and fault tolerance. Highlights emerging research in formal verification of Saga workflows.
- **Mister Smith Relevance:** Mister Smith's orchestrator agent naturally implements the orchestration-based Saga variant. The emphasis on idempotency is critical for NATS-based message delivery (at-least-once semantics require idempotent handlers).

### Hybrid Saga/2PC for Banking APIs
- **Authors:** Hebbar
- **Year:** 2025 | **Citations:** 0 | **Venue:** American J. Engineering and Technology
- **Key Finding:** Proposes hybrid strategy: 2PC for core operations requiring strict consistency, Saga for auxiliary services needing availability. Includes chaos engineering simulation benchmarking both patterns. Saga outperforms in availability and fault recovery; 2PC superior for immediate consistency.
- **Mister Smith Relevance:** Validates using different consistency models for different agent operations. Critical agent state (role assignments, task ownership) could use stronger consistency, while agent output propagation uses eventual consistency through JetStream.

---

## 5. Tool-Calling Failure Recovery

### PALADIN: Self-Correcting Language Model Agents to Cure Tool-Failure Cases
- **Authors:** Vuddanti, Shah, Chittiprolu, Song, Dev, Zhu, Chaudhary
- **Year:** 2025 | **Citations:** 0 | **Venue:** ArXiv
- **Key Finding:** Tool-augmented agents fail frequently due to tool malfunctions -- timeouts, API exceptions, inconsistent outputs -- triggering cascading reasoning errors. PALADIN trains on 50,000+ recovery-annotated trajectories via systematic failure injection. At inference, detects execution-time errors and retrieves similar cases from a curated bank of 55+ failure exemplars. Improves Recovery Rate from 32.76% to **89.68%** (+57% relative). Generalizes to novel failures with 95.2% recovery on unseen APIs.
- **Mister Smith Relevance:** **High.** Directly relevant to Mister Smith's ToolBus <-> LLM function calling bridge (Phase 9). The failure exemplar bank concept could be implemented as a persistent failure pattern catalog in JetStream KV, informing retry strategies and circuit breaker thresholds per tool.

### Structured Reflection for Reliable Tool Interactions
- **Authors:** Su, Wan, Yang, Shi, Han, Luo, Qiu
- **Year:** 2025 | **Citations:** 1 | **Venue:** ArXiv
- **Key Finding:** Turns error-to-repair path into an explicit, controllable, trainable action: "Reflect, then Call, then Final." Introduces Tool-Reflection-Bench benchmark. Shows large gains in multi-turn tool-call success and reduction of redundant calls. Key insight: making reflection explicit and optimizing it directly improves tool interaction reliability.
- **Mister Smith Relevance:** The "Reflect, then Call, then Final" pattern could be encoded in Mister Smith's agent task lifecycle. After a tool failure, rather than immediate retry, the agent enters a reflection phase (captured in the agent state machine) before attempting the corrected call.

### Conditional Multi-Stage Failure Recovery for Embodied Agents
- **Authors:** Farag, Stoyanchev, Li, Keizer, Doddipatla
- **Year:** 2025 | **Citations:** 1 | **Venue:** ArXiv
- **Key Finding:** Four-stage error-handling framework: three stages during execution, one post-execution reflection. Uses zero-shot chain prompting. Achieves SOTA performance, outperforming baselines without error recovery by 11.5% and strongest existing model by 19%.
- **Mister Smith Relevance:** The multi-stage approach aligns with Mister Smith's supervision hierarchy. Stage 1-3 (execution-time) map to OneForOne restarts and circuit breaker trips. Stage 4 (reflection) maps to the supervision tree's escalation to parent supervisor for strategy reassessment.

---

## 6. Chaos Engineering for LLM Multi-Agent Systems

### Assessing and Enhancing the Robustness of LLM-Based Multi-Agent Systems Through Chaos Engineering
- **Authors:** Owotogbe
- **Year:** 2025 | **Citations:** 4 | **Venue:** IEEE/ACM CAIN 2025
- **Key Finding:** First chaos engineering framework specifically for LLM-MAS. Targets three vulnerability classes: hallucinations, agent failures, and agent communication failures. Proposes proactive identification of vulnerabilities in production-like environments.
- **Mister Smith Relevance:** **High.** Provides a testing methodology for Mister Smith's supervision infrastructure. The three vulnerability classes map directly to: hallucinations -> output validation in agent pipeline; agent failures -> supervision tree restart strategies; communication failures -> NATS transport resilience and circuit breakers.

### Chaos Engineering 2.0: AI-Driven, Policy-Guided Resilience for Multi-Cloud Systems
- **Authors:** Opara, Akatakpo, Ironuru, Anyaene, Enobakhare
- **Year:** 2025 | **Citations:** 1 | **Venue:** Journal of Computer, Software, and Program
- **Key Finding:** Extends chaos engineering with AI-guided experiment orchestration, service-mesh fault injection, and chaos-as-code guarded by policy-as-code. Synthesizes resilience patterns (circuit breakers, bulkheads, adaptive retries, progressive delivery) mapped to modern toolchain. Discusses autonomous chaos agents as a future direction.
- **Mister Smith Relevance:** The chaos-as-code concept could be integrated into Mister Smith's integration test suite. Policy-as-code for guardrailing chaos experiments maps to Mister Smith's RBAC system for controlling who can trigger fault injection.

### MAD-Spear: Conformity-Driven Prompt Injection Attack on Multi-Agent Debate
- **Authors:** Cui, Du
- **Year:** 2025 | **Citations:** 0 | **Venue:** ArXiv
- **Key Finding:** Compromising a small subset of agents can significantly disrupt the entire MAS process by exploiting LLMs' conformity tendencies. Proposes formal definition of MAD fault-tolerance. Key finding: **agent diversity substantially improves MAS performance in mathematical reasoning**, challenging prior work suggesting diversity has minimal impact.
- **Mister Smith Relevance:** Validates Mister Smith's model-agnostic design. Using diverse LLM providers across agent roles (not just different prompts on the same model) provides genuine fault tolerance. The formal fault-tolerance definition could inform Mister Smith's supervision parameter tuning.

---

## 7. Self-Healing and Continuous Oversight Architectures

### COCO: Cognitive Operating System with Continuous Oversight for Multi-Agent Workflow Reliability
- **Authors:** Liang, Gan, Hong, Tian, Wu, Li
- **Year:** 2025 | **Citations:** 0 | **Venue:** ArXiv
- **Key Finding:** Decoupled architecture separating error detection from the critical execution path, achieving **O(1) monitoring overhead**. Three algorithmic innovations: (1) **Contextual Rollback Mechanism** -- stateful restart preserving execution history and error diagnostics; (2) **Bidirectional Reflection Protocol** -- mutual validation preventing oscillatory behavior; (3) **Heterogeneous Cross-Validation** -- leveraging model diversity via ensemble disagreement metrics. Achieves 6.5% average performance improvement.
- **Mister Smith Relevance:** **Critical.** The Contextual Rollback Mechanism is essentially a sophisticated version of OTP's "let it crash + restart with state" philosophy. The key insight of preserving execution history during restart should be incorporated into Mister Smith's supervision restart logic. Instead of a clean restart, agents should receive a "failure context" enabling informed re-computation. The O(1) monitoring overhead validates the approach of using a separate monitoring actor rather than inline checks.

### Autonomous AI Agents for Fault Detection and Self-Healing in Smart Manufacturing
- **Authors:** Ogunmolu, Olaniyi, Popoola, Olisa, Bamigbade
- **Year:** 2025 | **Citations:** 1 | **Venue:** J. Energy Research and Reviews
- **Key Finding:** Hybrid framework integrating spiking neural networks, symbolic reasoning, and Isolation Forest for fault detection and self-healing. Achieves 97.3% fault detection accuracy and 89.4% self-healing recovery rate. Reduces mean-time-to-repair by 31.7%.
- **Mister Smith Relevance:** The symbolic reasoning component for self-healing is relevant. Mister Smith's supervision rules are essentially symbolic reasoning over failure patterns. The 89.4% self-healing rate provides a realistic benchmark for what automated recovery can achieve.

### BISNet: Bio-Inspired Self-Healing Network
- **Authors:** Husain, Kumar, Narayana, Shunmugapriya, Annapurna, Srihari
- **Year:** 2025 | **Citations:** 0 | **Venue:** ICMCTC 2025
- **Key Finding:** Uses neuromorphic learning for dynamic adaptation, swarm intelligence for fault mitigation, and artificial immune system for anomaly detection. Self-reconfiguration based on failure patterns using spiking neural networks.
- **Mister Smith Relevance:** The "artificial immune system" metaphor for anomaly detection is interesting but more speculative. The self-reconfiguration concept aligns with Mister Smith's dynamic agent topology changes during failure recovery.

---

## 8. Agent Observability and Telemetry

### AgentSight: System-Level Observability for AI Agents Using eBPF
- **Authors:** Zheng, Hu, Yu, Quinn
- **Year:** 2025 | **Citations:** 1 | **Venue:** 4th Workshop on Practical Adoption Challenges of ML for Systems
- **Key Finding:** Identifies the "semantic gap" in agent monitoring: existing tools observe either high-level intent (LLM prompts) or low-level actions (system calls) but cannot correlate the two. AgentSight uses eBPF-based "boundary tracing" to monitor agents at stable system interfaces. Intercepts TLS-encrypted LLM traffic, monitors kernel events, and causally correlates both streams. Framework-agnostic with <3% performance overhead. Detects prompt injection attacks, resource-wasting reasoning loops, and hidden coordination bottlenecks.
- **Mister Smith Relevance:** **High.** The semantic gap concept is directly relevant. Mister Smith's observability layer (OpenTelemetry + Prometheus) captures infrastructure metrics but may miss the correlation between agent intent and actual behavior. The "boundary tracing" approach could be adapted: instrument the ToolBus and NATS message boundaries to capture both semantic intent and execution outcomes.

### Log2Graph: LLM-Powered Dynamic Knowledge Graphs for Real-Time Cloud Observability
- **Authors:** Amanmadov, Abdullayev
- **Year:** 2025 | **Citations:** 0 | **Venue:** IJACSA
- **Key Finding:** Unifies unstructured logs, distributed traces, and configuration data into a living graph representation. Enables causal chain analysis, dependency mapping, and natural language queries ("what services will be impacted if this database fails?").
- **Mister Smith Relevance:** The dynamic dependency graph concept could be applied to Mister Smith's agent topology. As agents are added/removed by the orchestrator, a live dependency graph would enable proactive failure impact analysis before supervision decisions.

---

## 9. Formal Verification of Actor and Multi-Agent Systems

### Modelling and Model-Checking ROS2 Multi-Robot Systems using Timed Rebeca
- **Authors:** Trinh, Sirjani, Ciccozzi, Masud, Sjodin
- **Year:** 2025 | **Citations:** 0 | **Venue:** Unknown Journal
- **Key Finding:** Uses Timed Rebeca, an actor-based modeling language with reactive, concurrent, and time semantics, for formal verification of multi-agent systems via model checking. Addresses the challenge of bridging discrete models with continuous systems. Demonstrates round-trip engineering between model and implementation.
- **Mister Smith Relevance:** Timed Rebeca's actor-based semantics closely match Mister Smith's ActorCell/ActorRef model. Formal verification of supervision restart strategies (OneForOne, OneForAll, RestForOne) could use similar actor-based model checking to prove properties like "all agents eventually reach a consistent state after restart."

### Temporal Consistency Verification for Intelligent Unmanned Systems
- **Authors:** Lu, Li, Liu, Wu, Huang
- **Year:** 2025 | **Citations:** 0 | **Venue:** Scientific Reports
- **Key Finding:** Model checking-based verification of temporal consistency during dynamic agent addition, deletion, and replacement. Uses global time automaton network model with global and local clock variables. Depth-first search algorithm for temporal consistency checks at each state.
- **Mister Smith Relevance:** Directly relevant to Mister Smith's dynamic agent lifecycle. When the orchestrator adds, removes, or replaces agents in a team, temporal consistency must be maintained. This paper's approach could verify that Mister Smith's supervision tree correctly handles agent hot-swapping.

### Testing Message-Passing Concurrency
- **Authors:** Shi, Moldrup, Mathur, Pavlogiannis
- **Year:** 2025 | **Citations:** 0 | **Venue:** ArXiv
- **Key Finding:** Studies the consistency question for channel-based (message-passing) concurrency in Go, Rust, and Kotlin. Draws a rich complexity landscape with tractability/intractability boundaries. Provides novel algorithms for verifying channel consistency in automated verification tools.
- **Mister Smith Relevance:** Directly applicable to Mister Smith's Rust + Tokio + NATS architecture. The complexity results help understand the theoretical limits of testing Mister Smith's message-passing correctness. The tractability boundaries inform where automated testing can be effective vs. where manual analysis is needed.

---

## 10. Fault-Recovering Actor Garbage Collection

### CRGC: Fault-Recovering Actor Garbage Collection in Pekko
- **Authors:** Plyukhin, Agha, Montesi
- **Year:** 2025 | **Citations:** 0 | **Venue:** Proceedings of the ACM on Programming Languages (PACMPL)
- **Key Finding:** First fault-recovering cyclic actor garbage collector. Addresses the problem that in all four major actor frameworks (Pekko, Akka, Erlang, Elixir), programmers must explicitly kill actors. CRGC uses conflict-free replicated data structures -- actors record information locally and broadcast to GC on each node. Formalized in **TLA+** with proved soundness (non-garbage never killed) and completeness (all garbage eventually killed). Competitive performance with simpler approaches like weighted reference counting.
- **Mister Smith Relevance:** **Critical.** This paper addresses a real challenge in Mister Smith's actor system. When supervision trees restart actors, orphaned actor references can leak. CRGC's approach -- local recording with CRDT-based broadcast -- maps directly to Mister Smith's architecture where NATS provides the broadcast mechanism and JetStream KV provides the local recording. The TLA+ formalization could serve as a model for formally verifying Mister Smith's own actor lifecycle management.

---

## 11. LLM Provider Resilience: Routing, Rate Limiting, and Serving

### Enhancing Reliability in AI Inference Services: Production Incident Analysis
- **Authors:** Ranganathan, Zhang, Wu
- **Year:** 2025 | **Citations:** 0 | **Venue:** Unknown Journal
- **Key Finding:** One of the first provider-internal analyses of LLM inference incidents. Analyzed 156 high-severity incidents with a taxonomy validated at Cohen's K ~0.89. **60% are inference engine failures, with 40% of those being timeouts.** 74% auto-detected; 28% required hotfix. Mitigation levers include traffic routing, node rebalancing, and capacity increase. Contributes a practitioner-oriented adoption checklist.
- **Mister Smith Relevance:** **Critical for Phase 9.** The finding that timeouts dominate failure modes directly informs Mister Smith's LLM provider circuit breaker configuration. The 60/40 split (engine failures / timeouts) should be reflected in distinct error handling paths: timeout -> retry with backoff; engine failure -> provider failover.

### Towards Efficient Multi-LLM Inference: Routing and Hierarchical Techniques
- **Authors:** Behera, Champati, Morabito, Tarkoma, Gross
- **Year:** 2025 | **Citations:** 4 | **Venue:** ArXiv
- **Key Finding:** Comprehensive survey of two strategies: (i) routing (selecting the most suitable model per query), and (ii) cascading/hierarchical inference (escalating through model sequence until confident response). Both reduce computation by using lightweight models for simpler tasks.
- **Mister Smith Relevance:** Directly relevant to Mister Smith's ModelProvider trait design. The cascading approach maps to a fallback chain: try fast/cheap provider first, escalate to more capable provider only when needed. This is a resilience pattern (graceful degradation) and a cost optimization simultaneously.

### Niyama: QoS-Driven LLM Inference Serving
- **Authors:** Goel, Mohan, Kwatra, Anupindi, Ramjee
- **Year:** 2025 | **Citations:** 4 | **Venue:** ArXiv
- **Key Finding:** Fine-grained QoS classification allowing applications to specify precise latency requirements. Key innovation: **selective request relegation** for graceful service degradation during overload. Increases serving capacity by 32% while maintaining QoS guarantees. Reduces SLO violations by an order of magnitude under extreme load.
- **Mister Smith Relevance:** The selective request relegation concept maps to Mister Smith's agent priority system. During LLM provider overload, lower-priority agent requests could be deferred or routed to fallback providers, while critical agents maintain their SLOs.

### AnchorTP: Resilient LLM Inference with State-Preserving Elastic Tensor Parallelism
- **Authors:** Xu, Chen, Li, Xiong, Zhou, Tao, Bai, Yu, Wong
- **Year:** 2025 | **Citations:** 0 | **Venue:** Unknown Journal
- **Key Finding:** State-preserving recovery framework that decouples model parameters and KV caches from the inference process via a daemon. Reduces Time to First Success by up to 11x and Time to Peak by up to 59% versus restart-and-reload.
- **Mister Smith Relevance:** The daemon-based state preservation concept is relevant to Mister Smith's agent state management. Agent context (conversation history, task state) should be persisted independently of the agent process, enabling fast recovery without re-prompting the LLM from scratch.

### TurboBatch: Rate-Safe Asynchronous Batch Processing for Cloud LLM APIs
- **Authors:** Syed, Robitshek
- **Year:** 2025 | **Citations:** 0 | **Venue:** IEEE IC2E 2025
- **Key Finding:** Combines accurate token usage prediction with adaptive rate control for automated batch processing under strict API rate limits. Addresses the unaddressed gap in client-side orchestration for quota-constrained LLM APIs.
- **Mister Smith Relevance:** Relevant to Mister Smith's LLM provider rate limiting. The token usage prediction concept should be incorporated into the ResourcePool for LLM providers, enabling proactive rate limiting rather than reactive circuit breaking.

### Performance-Aware LLM Load Balancer for Mixed Workloads
- **Authors:** Jain, Parayil, Mallick, Choukse, Qin, Zhang, Goiri, Wang, Bansal, Ruhle, Kulkarni, Kofsky, Rajmohan
- **Year:** 2025 | **Citations:** 4 | **Venue:** 5th Workshop on ML and Systems
- **Key Finding:** LLM workloads have distinct prefill and decode phases with different compute/memory requirements. Proposes RL-based router with response-length predictor. Achieves 11% lower end-to-end latency than existing methods.
- **Mister Smith Relevance:** When Mister Smith uses multiple LLM instances, workload-aware routing based on request characteristics (embedding vs. completion vs. streaming) should be implemented in the provider abstraction layer.

---

## 12. Resilience Patterns for Distributed Systems

### Fault-Tolerant Systems: Design and Implementation Principles
- **Authors:** Kuznetsov
- **Year:** 2025 | **Citations:** 0 | **Venue:** Programmnie Sistemy i Vychislitelnye Metody
- **Key Finding:** Coherent classification of four strategic approaches: redundancy, recovery, error masking, and proactive methods. Links them to canonical patterns (active-passive, active-active, circuit breaker, bulkhead). Traces evolution from monoliths through microservices to CPS. Key thesis: fault tolerance is an **emergent property** from coordinated application of redundancy, isolation, and recovery at all levels. Single measures have limited effect; best outcomes from multi-layered combination.
- **Mister Smith Relevance:** Validates Mister Smith's multi-layered approach: Circuit breakers (mister-smith-async) + supervision trees (mister-smith-supervision) + health monitoring (mister-smith-monitoring) + persistence (mister-smith-persistence). The "emergent property" framing is important for documentation.

### Building Resilient Systems: Error Handling, Retry Mechanisms, and Predictive Analytics in EDA
- **Authors:** Dhanaraj
- **Year:** 2025 | **Citations:** 1 | **Venue:** JCSTS
- **Key Finding:** Categorizes failures into delivery errors, processing errors, and infrastructure failures. DLQs serve as critical safety nets. Key insight: some apparent failures should be reconceptualized as **alternative business flows** rather than errors. Advocates shift from reactive to proactive operations via predictive analytics.
- **Mister Smith Relevance:** The DLQ concept is already present in Mister Smith's EventBus (dead letter queue). The reframing of "alternative flows" is relevant: when an LLM agent produces an unexpected but valid output, the supervision tree should not treat it as a failure requiring restart.

### Designing Hybrid Execution Strategies: Full Restarts vs. Partial Recovery
- **Authors:** Raveendran
- **Year:** 2025 | **Citations:** 0 | **Venue:** IJCESE
- **Key Finding:** Adaptive threshold mechanisms for choosing between full restart and partial recovery in DAG-based computation. Evaluates task count, data volume, execution parallelism, and failure patterns. Demonstrates significant improvements in recovery time and resource utilization.
- **Mister Smith Relevance:** Directly relevant to Mister Smith's supervision strategy selection. The multi-dimensional threshold approach (not just failure count but also task complexity, data volume, parallelism) could enhance the SupervisedSystem's strategy selection beyond simple failure counting.

### Ensuring Resilience in Microservices with Cloud-Native API Gateways
- **Authors:** Pasunoori
- **Year:** 2025 | **Citations:** 0 | **Venue:** IJSRCSEIT
- **Key Finding:** Empirical data from 178 organizations: circuit breaking patterns **reduce cascading failures by 83.5%**. API gateways achieve 99.95% service availability. Intelligent load balancing improves resource utilization by 88%.
- **Mister Smith Relevance:** The 83.5% cascading failure reduction from circuit breakers is a concrete metric validating Mister Smith's CircuitBreaker implementation in mister-smith-async. The HTTP gateway in mister-smith-http serves the same role as the API gateways studied.

---

## 13. Runtime Governance and Graduated Containment

### MI9: An Integrated Runtime Governance Framework for Agentic AI
- **Authors:** Wang, Singhal, Kelkar, Tuo
- **Year:** 2025 | **Citations:** 0 | **Venue:** Unknown Journal
- **Key Finding:** First fully integrated runtime governance framework for agentic AI. Six components: (1) agency-risk index, (2) agent-semantic telemetry capture, (3) continuous authorization monitoring, (4) **Finite-State-Machine (FSM)-based conformance engines**, (5) goal-conditioned drift detection, (6) **graduated containment strategies**. Operates transparently across heterogeneous agent architectures.
- **Mister Smith Relevance:** **Critical.** The FSM-based conformance engine maps directly to Mister Smith's agent state machine (AgentState). The graduated containment strategy (from monitoring -> warning -> throttling -> isolation -> termination) provides a more nuanced alternative to binary "let it crash" supervision. This could be implemented as an enhancement to the SupervisedSystem, adding intermediate containment levels before full restart.

### Osprey: Scalable Framework for Orchestration of Agentic Systems
- **Authors:** Hellert, Montenegro, Sulc
- **Year:** 2025 | **Citations:** 1 | **Venue:** Unknown Journal
- **Key Finding:** Production-ready architecture with: dynamic capability classification, plan-first orchestration with explicit dependencies and optional human approval, context-aware task extraction, and **production-ready execution with checkpointing and artifact management**. Deployed at Advanced Light Source particle accelerator.
- **Mister Smith Relevance:** The checkpointing and artifact management pattern validates Mister Smith's JetStream KV state persistence approach. The human approval gate concept could be added to Mister Smith's orchestrator for high-risk operations.

---

## 14. Agent Memory and Long-Horizon State Management

### MEM1: Learning to Synergize Memory and Reasoning for Long-Horizon Agents
- **Authors:** Zhou, Qu, Wu, Kim, Kim, Prakash, Rus, Zhao, Low, Liang
- **Year:** 2025 | **Citations:** 29 | **Venue:** ArXiv
- **Key Finding:** End-to-end RL framework enabling agents to operate with **constant memory** across long multi-turn tasks. Compact shared internal state jointly supports memory consolidation and reasoning. Improves performance by 3.5x while reducing memory usage by 3.7x. Generalizes beyond training horizon.
- **Mister Smith Relevance:** The constant-memory approach is relevant to Mister Smith's agent state management. Rather than unbounded context growth, agents should maintain a compact, consolidated state that is persisted to JetStream KV and survives supervision restarts.

### Git Context Controller (GCC): Manage LLM Agent Context like Git
- **Authors:** Wu
- **Year:** 2025 | **Citations:** 0 | **Venue:** ArXiv
- **Key Finding:** Structures agent memory as a persistent file system with COMMIT, BRANCH, MERGE, and CONTEXT operations for milestone-based checkpointing, exploration of alternative plans, and structured reflection. Achieves SOTA on SWE-Bench-Lite (48.00% resolution).
- **Mister Smith Relevance:** The git-like branching and checkpointing model could enhance Mister Smith's state persistence. When an agent explores alternative approaches, rather than losing the original state on restart, the supervision tree could "branch" the agent state and merge results.

---

## 15. Testing Non-Deterministic Agent Systems

### An Approach to Checking Correctness for Agentic Systems (Temporal Expression Language)
- **Authors:** Sheffler
- **Year:** 2025 | **Citations:** 0 | **Venue:** ArXiv
- **Key Finding:** Temporal expression language for monitoring AI agent behavior, inspired by hardware verification temporal logic. Focuses on **sequence of agent actions** (tool invocations, inter-agent communications) rather than textual outputs. Serves dual purpose: validating prompt engineering during development and regression testing when agents are updated with new LLMs. Successfully flagged behavioral regressions when smaller models were substituted.
- **Mister Smith Relevance:** **High.** This approach directly addresses testing Mister Smith's supervision behavior. Temporal assertions like "after agent failure, supervisor restarts within T milliseconds" or "agent never transitions directly from Running to Terminated without passing through Stopping" can be expressed and checked against execution traces captured by the EventBus.

### Challenges in Testing LLM-Based Software: A Faceted Taxonomy
- **Authors:** Dobslaw, Feldt, Yoon, Yoo
- **Year:** 2025 | **Citations:** 1 | **Venue:** ArXiv
- **Key Finding:** LLMs introduce non-determinism unlike traditional software, requiring new verification approaches. Defines four facets of LLM test case design. Key insight: **correctness should be viewed as a distribution of outcomes rather than a binary property**. Current tools treat test executions as isolated events and lack variability-aware testing methodologies.
- **Mister Smith Relevance:** Fundamental to how Mister Smith should test its LLM-integrated agents. The supervision system should define "healthy" as a distribution of acceptable behaviors rather than a single expected output. This directly impacts the PhiAccrualFailureDetector's threshold tuning.

---

## 16. Messaging Infrastructure Benchmarks

### Next-Generation Event-Driven Architectures: AIEO Benchmarking Framework
- **Authors:** Arafat, Tasmin, Poudel
- **Year:** 2025 | **Citations:** 3 | **Venue:** ArXiv
- **Key Finding:** First comprehensive benchmarking of 12 messaging systems including **NATS JetStream** across 3 workloads: e-commerce, IoT, and **AI inference pipelines**. AIEO (AI-Enhanced Event Orchestration) with ML-driven predictive scaling achieves 34% latency reduction, 28% resource utilization improvement, 42% cost optimization. Kafka peak: 1.2M msg/sec, 18ms p95. Pulsar: 950K msg/sec, 22ms p95.
- **Mister Smith Relevance:** **Directly relevant.** Provides empirical data for NATS JetStream performance in AI inference pipelines -- exactly Mister Smith's use case. The AIEO orchestration patterns (predictive scaling, RL-based resource allocation) could enhance Mister Smith's dynamic scaling decisions.

---

## 17. Emerging Directions

### 17.1. Confidence-Weighted Byzantine Fault Tolerance
The convergence of BFT protocols with LLM confidence probing (CP-WBFT, WBFT) represents a genuinely new direction. Traditional BFT assumes binary correct/faulty behavior; LLM agents exhibit a spectrum of reliability. Mister Smith could pioneer "confidence-weighted supervision" where restart decisions are modulated by the agent's self-assessed confidence rather than binary success/failure.

### 17.2. Temporal Graph-Based Safety Monitoring
GUARDIAN's approach of modeling agent interactions as temporal attributed graphs is nascent but promising. Combined with Mister Smith's existing EventBus event stream and NATS message topology, this could enable predictive failure detection -- identifying patterns of interaction that historically precede cascading failures, and pre-emptively restructuring the supervision tree.

### 17.3. Contextual Rollback (Informed Restarts)
COCO's Contextual Rollback Mechanism represents an evolution beyond simple "let it crash" restarts. Rather than restarting agents with clean state, agents receive a failure context (what went wrong, what was tried, what the environment state was). This is the most impactful near-term enhancement for Mister Smith's supervision system.

### 17.4. Graduated Containment Strategies
MI9's graduated containment (monitor -> warn -> throttle -> isolate -> terminate) is more nuanced than OTP's binary restart-or-escalate model. For LLM agents that may produce degraded but not catastrophically wrong output, intermediate containment (rate limiting, output filtering) may be preferable to full restart.

### 17.5. Chaos Engineering as a First-Class Testing Discipline for Agent Systems
The Owotogbe (CAIN 2025) paper establishes chaos engineering for LLM-MAS as a legitimate research area. Combined with the Chaos Engineering 2.0 review's "chaos-as-code" concept, this suggests Mister Smith should include a fault injection framework as part of its test infrastructure, not just as an afterthought.

### 17.6. Edge-Level Error Intervention
AgentAsk's concept of treating every inter-agent message as a potential failure point and inserting lightweight validation represents a shift from node-level to edge-level fault tolerance. For Mister Smith, this means the NATS transport layer itself could become an active participant in fault detection, rather than passively delivering messages.

### 17.7. Formal Verification of Actor Lifecycle
CRGC's TLA+ formalization of actor garbage collection demonstrates that formal verification of actor lifecycle properties is tractable. Extending this to Mister Smith's supervision semantics (proving that restart strategies preserve safety and liveness invariants) is a natural next step.

### 17.8. Semantic Observability Gap
AgentSight identifies a fundamental gap: correlating agent intent with system behavior. Traditional APM sees syscalls; LLM monitoring sees prompts; nobody sees both. For production Mister Smith deployments, bridging this gap through ToolBus and NATS boundary instrumentation would be a differentiating capability.

---

## Synthesis: Implications for Mister Smith Architecture

### Immediate Architecture Validations
The research strongly validates several existing Mister Smith design decisions:
1. **Multi-layered fault tolerance** (Kuznetsov 2025): Circuit breakers + supervision trees + health monitoring = emergent resilience
2. **Message-bus architecture** (AIEO 2025): NATS JetStream benchmarked specifically for AI inference pipelines
3. **Model-agnostic design** (MAD-Spear 2025): Agent diversity across LLM providers provides genuine fault tolerance
4. **Saga-like orchestration** (SagaLLM 2025, PVLDB): Compensation and rollback for multi-agent workflows
5. **Dead letter queue** (Dhanaraj 2025): Already implemented in Mister Smith's EventBus

### Recommended Enhancements (Priority-Ordered)
1. **Contextual Rollback** (from COCO): Modify supervision restart to pass failure context to restarted agents
2. **Graduated Containment** (from MI9): Add intermediate supervision states between "healthy" and "restarting"
3. **Edge-Level Validation** (from AgentAsk): Add confidence scoring to MessageEnvelope in the Transport layer
4. **Temporal Assertions** (from Sheffler): Build temporal logic monitoring on the EventBus for regression testing
5. **Confidence-Weighted Health** (from CP-WBFT): Enhance PhiAccrualFailureDetector with LLM confidence probes
6. **Failure Pattern Catalog** (from PALADIN): Persist tool failure patterns in JetStream KV for informed retry
7. **Provider Timeout Handling** (from Ranganathan et al.): Distinct error paths for timeouts vs. engine failures in Phase 9

### Key Metrics from the Literature
| Metric | Source | Value |
|--------|--------|-------|
| Circuit breaker cascading failure reduction | Pasunoori 2025 | 83.5% |
| Tool failure recovery rate (PALADIN) | Vuddanti et al. 2025 | 89.68% |
| Byzantine fault tolerance threshold | Zheng et al. 2025 | 85.7% fault rate |
| LLM inference timeout dominance | Ranganathan et al. 2025 | 60% engine failures, 40% timeouts |
| Monitoring overhead achievable | Liang et al. 2025 | O(1) with decoupled architecture |
| Self-healing recovery rate | Ogunmolu et al. 2025 | 89.4% |
| NATS JetStream AI pipeline suitability | Arafat et al. 2025 | Benchmarked across 2400+ configs |

---

## Full Paper Index (Alphabetical by First Author)

1. Arafat et al. (2025) -- "Next-Generation Event-Driven Architectures" -- ArXiv, 3 citations
2. Bansal (2025) -- "Building Resilient Gen-AI Systems" -- IJARSCT, 0 citations
3. Behera et al. (2025) -- "Efficient Multi-LLM Inference: Routing and Hierarchical" -- ArXiv, 4 citations
4. Cemri et al. (2025) -- "Why Do Multi-Agent LLM Systems Fail?" -- ArXiv, 134 citations
5. Cui & Du (2025) -- "MAD-Spear: Conformity-Driven Prompt Injection" -- ArXiv, 0 citations
6. Dhanaraj (2025) -- "Resilient Systems: Error Handling, Retry, Predictive Analytics in EDA" -- JCSTS, 1 citation
7. Dobslaw et al. (2025) -- "Challenges in Testing LLM-Based Software" -- ArXiv, 1 citation
8. Dusad (2025) -- "Taming Asynchrony in Distributed Payment Systems" -- JCSTS, 0 citations
9. Farag et al. (2025) -- "Conditional Multi-Stage Failure Recovery for Embodied Agents" -- ArXiv, 1 citation
10. Ferrag et al. (2025) -- "From LLM Reasoning to Autonomous AI Agents: Comprehensive Review" -- ArXiv, 53 citations
11. Goel et al. (2025) -- "Niyama: QoS-Driven LLM Inference Serving" -- ArXiv, 4 citations
12. Gu et al. (2025) -- "MedAgentAudit: Collaborative Failure Modes in Medical MAS" -- ArXiv, 0 citations
13. Hebbar (2025) -- "Saga Pattern vs. 2PC in Banking APIs" -- Am. J. Eng. Tech., 0 citations
14. Hellert et al. (2025) -- "Osprey: Scalable Agentic System Orchestration" -- Unknown, 1 citation
15. Hu et al. (2025) -- "Randomized Smoothing for LLM-Driven MAS Robustness" -- ArXiv, 5 citations
16. Jain et al. (2025) -- "Performance-Aware LLM Load Balancer" -- 5th Workshop on ML and Systems, 4 citations
17. Kuznetsov (2025) -- "Fault-Tolerant Systems: Design and Implementation Principles" -- PSVM, 0 citations
18. Li et al. (2025) -- "AgentAsk: Multi-Agent Systems Need to Ask" -- ArXiv, 0 citations
19. Liang et al. (2025) -- "COCO: Cognitive OS with Continuous Oversight" -- ArXiv, 0 citations
20. Luo et al. (2025) -- "WBFT Consensus for Trusted Multi-LLM Networks" -- ArXiv, 7 citations
21. Ma et al. (2025) -- "AgentFail: Diagnosing Failure Root Causes" -- ArXiv, 0 citations
22. Mguni et al. (2025) -- "MARTA: Fault-Tolerant Multi-Agent Learning" -- ArXiv, 0 citations
23. Neelan (2025) -- "Saga Pattern Review for Microservices" -- IJMR, 0 citations
24. Ogunmolu et al. (2025) -- "Autonomous AI Agents for Self-Healing Manufacturing" -- J. Energy Research, 1 citation
25. Opara et al. (2025) -- "Chaos Engineering 2.0: AI-Driven Resilience" -- JCSP, 1 citation
26. Owotogbe (2025) -- "Chaos Engineering for LLM-Based MAS" -- IEEE/ACM CAIN, 4 citations
27. Pasunoori (2025) -- "Resilience in Microservices with API Gateways" -- IJSRCSEIT, 0 citations
28. Plyukhin et al. (2025) -- "CRGC: Fault-Recovering Actor GC in Pekko" -- PACMPL, 0 citations
29. Ranganathan et al. (2025) -- "Reliability in AI Inference Services: Production Incidents" -- Unknown, 0 citations
30. Raveendran (2025) -- "Hybrid Execution: Full Restarts vs. Partial Recovery" -- IJCESE, 0 citations
31. Sapkota et al. (2025) -- "AI Agents vs. Agentic AI: Taxonomy" -- Inf. Fusion, 103 citations
32. Sheffler (2025) -- "Checking Correctness for Agentic Systems" -- ArXiv, 0 citations
33. Shen et al. (2025) -- "Communication Topologies in LLM-Based MAS" -- ArXiv, 4 citations
34. Shi et al. (2025) -- "Testing Message-Passing Concurrency" -- ArXiv, 0 citations
35. Su et al. (2025) -- "Structured Reflection for Reliable Tool Interactions" -- ArXiv, 1 citation
36. Syed & Robitshek (2025) -- "TurboBatch: Rate-Safe LLM API Batch Processing" -- IEEE IC2E, 0 citations
37. Trinh et al. (2025) -- "Model-Checking ROS2 Multi-Robot with Timed Rebeca" -- Unknown, 0 citations
38. Vankayalapati & Pandugula (2025) -- "Self-Healing Cloud Infrastructures" -- SSRN, 10 citations
39. Vuddanti et al. (2025) -- "PALADIN: Self-Correcting Agents for Tool Failures" -- ArXiv, 0 citations
40. Wang et al. (2025) -- "MI9: Runtime Governance for Agentic AI" -- Unknown, 0 citations
41. Wu (2025) -- "Git Context Controller for LLM Agents" -- ArXiv, 0 citations
42. Xu et al. (2025) -- "AnchorTP: State-Preserving Elastic Tensor Parallelism" -- Unknown, 0 citations
43. Yang et al. (2025) -- "AgentNet: Decentralized Evolutionary Coordination" -- ArXiv, 16 citations
44. Yang et al. (2025) -- "AutoHMA-LLM: Heterogeneous MAS Coordination" -- IEEE TCCN, 11 citations
45. Zhang et al. (2025) -- "Graph Neural AI for Microservice Anomaly Detection" -- Unknown, 0 citations
46. Zheng et al. (2025) -- "Rethinking MAS Reliability via Byzantine Fault Tolerance" -- Unknown, 0 citations
47. Zheng et al. (2025) -- "AgentSight: System-Level Observability via eBPF" -- ML for Systems, 1 citation
48. Zhou et al. (2025) -- "GUARDIAN: Temporal Graph Modeling for MAS Safety" -- ArXiv, 4 citations
49. Zhou et al. (2025) -- "MEM1: Memory and Reasoning for Long-Horizon Agents" -- ArXiv, 29 citations

---

*Search methodology: 20+ iterative queries across Consensus Academic Search API. Each query returned top-3 results from a pool of 20 ranked papers. Queries were dynamically refined based on emerging themes from earlier results. Only papers from 2025 onward were included. Papers were selected for relevance to Mister Smith's specific architecture: Rust actor model, NATS/JetStream messaging, OTP-style supervision, model-agnostic LLM integration.*
