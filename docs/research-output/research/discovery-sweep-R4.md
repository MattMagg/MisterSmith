---
version: R4
created: 2026-03-07
updated: 2026-03-07
sources: Consensus (96 papers, 32 searches)
round: 4 (Academic Search — Discovery)
---

# Discovery Sweep: What We Don't Know We're Missing

**Date:** 2026-03-07
**Search Scope:** Academic papers 2025-2026 via Consensus API
**Methodology:** 32 searches across 8 dimensions -- agentic AI, distributed systems, inference optimization, cognitive architectures, reliability engineering, game theory, security, and wild cards.

---

## Theme 1: CRDTs for Multi-Agent Coordination -- Observation-Driven Instead of Message-Driven

### What Was Found

**CodeCRDT** (Pugachev, 2025, ArXiv) introduces a paradigm shift: instead of agents coordinating via explicit message passing, they coordinate by observing a shared CRDT state. Lock-free, conflict-free concurrent code generation achieves 100% convergence with zero merge failures across 600 trials. Semantic conflict rates are 5-10%, with up to 21.1% speedup on some task structures.

**Lattica** (Yang et al., 2025, ArXiv) builds a full decentralized communication framework for distributed AI using CRDTs as the consistency primitive, combined with DHTs for content discovery and NAT traversal for peer-to-peer mesh networking.

**CRDT-based knowledge synchronization** (Galeas et al., 2025, Computer Vision and Image Understanding, 1 citation) combines delta-CRDTs with pub/sub interaction protocols for distributed agent knowledge management, showing satisfactory latency and consistency.

### Why It Matters for Mister Smith

This is potentially the most impactful finding of this sweep. Mister Smith already uses NATS pub/sub for agent communication, and JetStream KV for distributed state. CRDTs represent a fundamentally different coordination model that could replace or augment explicit message passing for certain agent interactions. The CodeCRDT finding is directly relevant -- agents working on shared codebases could use observation-driven coordination instead of explicit message exchanges, potentially reducing communication overhead while maintaining consistency.

### Connection to Existing Research

Our existing research covers NATS patterns, stigmergy (indirect coordination through environment modification), and neural paging. CRDTs are the formal computer science analog to stigmergy -- agents modify shared state, and other agents observe the modifications. This bridges our stigmergy research with a well-established distributed systems primitive. The JetStream KV store we already use could potentially be enhanced with CRDT semantics.

### Implementation Ideas

- Implement CRDT-backed shared workspace for concurrent code editing agents
- Use delta-CRDTs over NATS JetStream for eventually-consistent agent state (smaller messages than full state sync)
- Hybrid model: CRDTs for "shared artifact" coordination, NATS pub/sub for "event notification" coordination
- Could leverage the `crdts` or `automerge` Rust crates

---

## Theme 2: DAG-Based Parallel Agent Execution

### What Was Found

**Beyond ReAct: Planner-Centric Framework** (Wei et al., 2025, ArXiv) proposes replacing ReAct's incremental decision-making with global DAG planning. The Planner model creates a Directed Acyclic Graph of tool operations, enabling optimized execution paths. Achieves state-of-the-art on StableToolBench.

**Flash-Searcher** (Qin et al., 2025, ArXiv, 2 citations) reimagines agent execution from sequential chains to DAGs. Independent reasoning paths execute concurrently while maintaining logical constraints. Reduces agent execution steps by up to 35%.

**AgentNet** (Yang et al., 2025, ArXiv, 16 citations) introduces a decentralized framework where agents self-organize into dynamic DAG topologies that adapt in real-time to task demands. Uses retrieval-based memory for continual skill refinement. Eliminates centralized orchestrators while achieving higher accuracy than both single-agent and centralized multi-agent baselines.

**Architecting Resilient LLM Agents** (Del Rosario et al., 2025, ArXiv, 2 citations) provides a comprehensive guide to Plan-then-Execute (P-t-E) architecture, arguing it is inherently more resilient to prompt injection than ReAct by establishing control-flow integrity. Discusses DAG-based parallel execution as an advanced pattern.

### Why It Matters for Mister Smith

Mister Smith's current orchestration model uses a centralized orchestrator with team-based coordination. DAG-based planning offers a more principled approach to parallel task execution. The Planner-Centric framework directly maps to our agent architecture: a planning agent creates a DAG, and executor agents process the nodes. AgentNet's decentralized DAG topology is particularly interesting because it aligns with our supervision tree model -- agents can dynamically reconfigure their collaboration graph.

### Connection to Existing Research

Our existing research on agentic loop architectures (LATS/MCTS) and NATS patterns covers sequential and tree-based execution. DAG execution is a natural extension that we haven't explicitly addressed. The P-t-E pattern's security analysis (control-flow integrity) connects to our capability security research.

### Implementation Ideas

- Add a `DagPlanner` trait to the agent system that generates execution DAGs from task descriptions
- Use NATS JetStream's consumer groups to parallelize independent DAG nodes
- Implement dependency tracking in the task scheduler for DAG edges
- Dynamic DAG re-planning on node failure (connect to supervision trees)

---

## Theme 3: Multi-Agent Architecture Search (MaAS) -- Self-Optimizing Agent Topologies

### What Was Found

**MaAS: Multi-agent Architecture Search via Agentic Supernet** (Zhang et al., 2025, ArXiv, 52 citations) is a high-impact paper introducing the concept of an "agentic supernet" -- a probabilistic distribution over possible multi-agent architectures. Instead of designing a fixed agent topology, MaAS samples query-dependent configurations that dynamically allocate resources (LLM calls, tool calls, token cost). Achieves 6-45% of inference costs of handcrafted systems while surpassing them by 0.5-12%.

**AutoMaAS** (Ma et al., 2025, ArXiv) extends this with self-evolving architecture search using automatic operator generation/fusion/elimination, cost-aware optimization, online feedback, and decision tracing. Achieves 1-7% performance improvement while reducing costs 3-5%.

### Why It Matters for Mister Smith

This is a meta-level innovation: instead of hard-coding which agents collaborate on which tasks, the system automatically discovers optimal agent configurations per query. With 9 agent roles defined in Mister Smith, the combinatorial space of possible team compositions is large. MaAS provides a principled way to navigate this space. The 52 citations indicate this is a rapidly influential paper.

### Connection to Existing Research

Our model routing research covers which LLM to use for a task. MaAS goes further: which *agents* with which *tools* using which *models* in which *topology*. This is a superset of model routing. Our existing agent orchestrator in Phase 7 uses static team definitions -- MaAS would make this dynamic.

### Implementation Ideas

- Implement an `AgentArchitectureSampler` that uses task features to select agent team composition
- Track performance/cost metrics per agent configuration to build the supernet empirically
- Start simple: learned routing between team sizes (single agent vs. pair vs. full team) based on task complexity
- Could use the existing monitoring/metrics infrastructure (Phase 2/8) to collect training data

---

## Theme 4: AgentOps -- Observability Purpose-Built for Non-Deterministic AI Systems

### What Was Found

**AgentOps Framework** (Moshkovich & Zeltyn, 2025, ArXiv, 4 citations) presents a comprehensive six-stage pipeline for observing, analyzing, optimizing, and automating operations of agentic AI systems. Identifies distinct needs across four roles: developers, testers, SREs, and business users. Emphasizes taming uncertainty rather than eliminating it.

**Beyond Black-Box Benchmarking** (Moshkovich et al., 2025, ArXiv, 7 citations) demonstrates that 79% of practitioners agree non-deterministic execution flow is a major challenge. Introduces taxonomies for analytics outcomes and extends standard observability frameworks. Proposes building benchmarks from runtime logs rather than static test suites.

**AgentSight** (Zheng et al., 2025, 1 citation) uses eBPF for system-level observability of AI agents. Bridges the "semantic gap" between high-level intent (LLM prompts) and low-level actions (system calls) by intercepting TLS-encrypted LLM traffic and correlating it with kernel events. Less than 3% performance overhead. Detects prompt injection attacks, reasoning loops, and multi-agent coordination bottlenecks. Open source.

**AIOpsLab** (Chen et al., 2025, ArXiv, 17 citations) presents a holistic framework for evaluating AI agents for autonomous cloud operations, coining the term "AgentOps" for the paradigm of AI agents managing their own operational lifecycle.

### Why It Matters for Mister Smith

Mister Smith has built traditional observability (OTel tracing, Prometheus metrics, structured logging) in Phase 8. But these papers show that traditional observability is insufficient for non-deterministic AI agent systems. The semantic gap (intent vs. action correlation) and the non-deterministic flow problem are specific challenges that standard tracing/metrics don't address. AgentSight's eBPF approach is particularly interesting for Mister Smith because it is framework-agnostic and instrumentation-free.

### Connection to Existing Research

Our Phase 8 observability infrastructure provides the foundation but doesn't address the AI-specific challenges identified here. The AgentOps six-stage pipeline (observe -> collect metrics -> detect issues -> root cause analysis -> recommend -> automate) maps to but extends our existing monitoring/events/supervision infrastructure.

### Implementation Ideas

- Extend the existing `MetricsCollector` with agent-specific metrics: reasoning steps per task, tool call patterns, token consumption per outcome
- Implement "trace quality" scoring that correlates LLM reasoning traces with task outcomes
- Build runtime anomaly detection on agent execution patterns (reasoning loops, excessive tool calls)
- Connect the existing PhiAccrualFailureDetector to detect "slow reasoning" anomalies (analogous to slow heartbeats)

---

## Theme 5: Hierarchical and Episodic Memory for Long-Term Agents

### What Was Found

**Episodic Memory is the Missing Piece** (Pink et al., 2025, ArXiv, 18 citations) argues that LLM agents need five properties of episodic memory for long-term operation: single-shot learning, temporal context, self-referentiality, constructive recall, and emotional association. Position paper with broad roadmap.

**H-MEM: Hierarchical Memory** (Sun & Zeng, 2025, ArXiv, 5 citations) proposes multi-level memory organized by semantic abstraction. Each memory vector has positional index encoding pointing to semantically related sub-memories. Index-based routing enables efficient retrieval without exhaustive similarity computation. Outperforms five baselines on LoCoMo dataset.

**MIRIX: Multi-Agent Memory System** (Wang & Chen, 2025, ArXiv, 16 citations) defines six memory types: Core, Episodic, Semantic, Procedural, Resource Memory, and Knowledge Vault. Multi-agent framework dynamically coordinates updates and retrieval. Achieves 35% higher accuracy than RAG with 99.9% storage reduction on multimodal benchmarks. State-of-the-art 85.4% on LOCOMO.

### Why It Matters for Mister Smith

Mister Smith's Phase 6 persistence layer provides PostgreSQL and JetStream KV storage, but it treats agent state as flat key-value pairs. The memory architectures described here suggest that agents should have typed, hierarchical memory systems that support different access patterns: fast procedural recall, slow semantic search, temporal episodic retrieval. The six-type MIRIX taxonomy directly maps to agent capabilities that Mister Smith's nine roles would need.

### Connection to Existing Research

Our existing research on neural paging addresses context window management but not persistent cross-session memory. The persistence layer (Phase 6) provides storage primitives but not memory abstraction. This theme bridges that gap -- memory is the semantic layer on top of storage.

### Implementation Ideas

- Define `MemoryStore` trait hierarchy: `EpisodicMemory`, `SemanticMemory`, `ProceduralMemory`
- Use JetStream KV for fast procedural memory (tool call patterns, recent actions)
- Use PostgreSQL with pgvector for semantic memory (embeddings + similarity search)
- Implement H-MEM's hierarchical index as a tree structure over JetStream KV keys
- Add memory compaction/consolidation as a background agent task (sleep/consolidation analogy)

---

## Theme 6: Process Reward Models (PRMs) for Step-by-Step Verification

### What Was Found

**R-PRM: Reasoning-Driven Process Reward Modeling** (She et al., 2025, ArXiv, 19 citations) uses stronger LLMs to generate training data from limited annotations, bootstrapping step-by-step evaluation. Outperforms baselines by 11.9 and 8.5 F1 points on ProcessBench and PRMBench. When guiding reasoning, achieves 8.5+ accuracy improvements across six datasets.

**Conditional Reward Modeling (CRM)** (Zhang et al., 2025, ArXiv) frames LLM reasoning as a temporal process, conditioning each step's reward on preceding steps AND the final outcome. Resolves credit assignment ambiguity. More robust to reward hacking than existing PRMs.

**Uncertainty-Aware Step-wise Verification** (Ye et al., 2025, ArXiv, 10 citations) introduces CoT Entropy for quantifying PRM uncertainty. Incorporating uncertainty estimates improves robustness of step-wise verification.

**Reward-Guided Speculative Decoding (RSD)** (Liao et al., 2025, ArXiv, 63 citations) combines process reward models with speculative decoding -- a PRM evaluates intermediate steps and dynamically decides whether to invoke a more powerful model. Up to 4.4x fewer FLOPs with better accuracy.

### Why It Matters for Mister Smith

Mister Smith orchestrates multi-step agent workflows where each step (code generation, review, testing) produces intermediate results. PRMs could verify the quality of each step before proceeding, catching errors early in the pipeline. The RSD finding is particularly relevant for model routing: a process reward model could decide whether a step needs a more capable (expensive) model or a lighter one suffices.

### Connection to Existing Research

Our model routing research focuses on task-level routing (which model for which task). PRMs enable step-level routing within a task execution. Our supervision tree research handles failure detection at the actor level but not at the reasoning-step level within an actor. PRMs bridge this gap.

### Implementation Ideas

- Add a `StepVerifier` trait that evaluates intermediate agent outputs before committing them
- Implement lightweight PRM as a "critic" agent role that scores each step in a workflow
- Use CRM's temporal conditioning: each verification step considers the full trajectory context
- Connect to supervision: verification failure triggers step retry (not full task restart)

---

## Theme 7: Cognitive Load-Aware Inference and Token Economics

### What Was Found

**Cognitive Load-Aware Inference (CLAI)** (Zhang, 2025, ArXiv, 2 citations) applies Cognitive Load Theory from neuroscience to LLM inference. Formalizes three load types for LLMs: Intrinsic (problem complexity), Extraneous (wasteful computation), and Germane (productive reasoning). CLAI-Prompt reduces token consumption up to 45% without sacrificing accuracy. CLAI-Tune spontaneously learns to decompose difficult problems -- an emergent ability.

**Computational Economics in LLMs** (Reddy et al., 2025, ArXiv) treats LLM internals as an economy of resource-constrained agents. Demonstrates that under resource scarcity, standard LLMs reallocate attention toward high-value tokens. An incentive-driven training paradigm achieves ~40% FLOP reduction while maintaining accuracy.

**Self-Resource Allocation in Multi-Agent LLM Systems** (Amayuelas et al., 2025, ArXiv, 8 citations) demonstrates that LLM planners outperform orchestrators for concurrent resource allocation. Explicit worker capability information enhances allocation strategies.

### Why It Matters for Mister Smith

Token consumption is a primary operational cost for LLM agent systems. CLAI provides a principled framework for managing this cost without sacrificing quality. The three-load-type taxonomy could be directly applied to Mister Smith's agent system: measure intrinsic task complexity, minimize wasted LLM calls, maximize productive reasoning per token. The self-resource allocation finding supports the architecture decision to use a planner agent for task distribution.

### Connection to Existing Research

Our streaming/backpressure research addresses throughput management at the message level. CLAI operates at a different level -- managing the LLM's "thinking budget" per task. These are complementary: backpressure controls message flow, CLAI controls reasoning depth. Neither was explored in our existing research.

### Implementation Ideas

- Implement task complexity estimation as a pre-processing step before model selection
- Add token budget allocation to the agent orchestrator (per-task token limits based on estimated complexity)
- Track "productive reasoning rate" (useful output tokens / total tokens consumed) as an operational metric
- Use CLAI's three-load framework as a prompt engineering pattern for agent system prompts

---

## Theme 8: Multi-Agent Security Threat Models and Defense

### What Was Found

**Multi-Agent Systems Execute Arbitrary Malicious Code** (Triedman et al., 2025, ArXiv, 15 citations) demonstrates that adversarial content can hijack multi-agent systems to execute arbitrary code in 58-100% of trials, even when individual agents resist prompt injection. The attack works by exploiting inter-agent communication -- one compromised agent influences others through the orchestration layer.

**End-to-End Threat Model for LLM-Agent Ecosystems** (Ferrag et al., 2025, ArXiv, 6 citations) catalogs 30+ attack techniques across four domains: Input Manipulation, Model Compromise, System/Privacy Attacks, and Protocol Vulnerabilities (including attacks on MCP, ACP, ANP, A2A protocols). Proposes cryptographic provenance tracking and dynamic trust management for MCP deployments.

**AgentSandbox** (Zhang et al., 2025, ArXiv, 7 citations) applies information security principles (defense-in-depth, least privilege, complete mediation) to LLM agent lifecycle. Maintains high utility while substantially mitigating privacy risks.

**AdvEvo-MARL** (Pan et al., 2025, ArXiv) internalizes safety into task agents via adversarial co-evolution, eliminating external guard models. Keeps attack success rate below 20% while preserving task accuracy.

### Why It Matters for Mister Smith

Mister Smith's Phase 5 security layer addresses JWT auth, RBAC, TLS, and audit logging -- but these are infrastructure-level security measures. The papers here reveal that multi-agent-specific attack vectors exist that bypass infrastructure security entirely. The inter-agent communication hijacking finding is particularly alarming because Mister Smith's NATS-based message passing could be exploited in exactly this way: a compromised agent publishes malicious messages that influence other agents' behavior.

### Connection to Existing Research

Our capability security research addresses WASM sandboxing for tool execution isolation. The findings here extend the threat model beyond tool execution to include the agent communication layer itself. The MCP protocol vulnerability analysis is directly relevant since Mister Smith includes an MCP crate (Phase 4).

### Implementation Ideas

- Add message content validation at the NATS transport layer (not just authentication)
- Implement "blast radius" limits on agent-to-agent influence (an agent's outputs should be sandboxed before affecting other agents)
- Use cryptographic provenance tracking for agent decision chains (extend the audit bridge from Phase 8)
- Apply least privilege to agent tool access: each agent role gets only the tools it needs, enforced by the supervision system
- Consider content-based firewall rules in the NATS subject namespace

---

## Theme 9: Provenance Tracking for Agentic Workflows

### What Was Found

**PROV-AGENT** (Souza et al., 2025, IEEE eScience, 4 citations) extends W3C PROV with MCP integration to capture agent-centric metadata (prompts, responses, decisions) alongside workflow context. Provides near real-time provenance capture across edge, cloud, and HPC environments.

**Auditable Agent Platform** (Unlu et al., 2025, ArXiv) demonstrates that provenance records capturing "molecular lineage" (agent decision chains) enable auditable reasoning trajectories and enable reuse of successful transformations via in-context learning.

### Why It Matters for Mister Smith

Mister Smith has audit logging (Phase 5) and event tracking (Phase 2), but these capture what happened, not why. Provenance tracking captures the causal chain: which agent made which decision based on which inputs and which other agents' outputs. This is critical for debugging non-deterministic multi-agent workflows and for regulatory compliance. The W3C PROV extension is particularly interesting because it provides a standardized provenance vocabulary.

### Connection to Existing Research

Our audit logging records security events but not reasoning provenance. Our event bus captures system events but not decision lineage. Provenance tracking is a semantic layer on top of both -- it captures the causal relationships between events and decisions.

### Implementation Ideas

- Extend `AuditEvent` with provenance fields: `parent_decision_id`, `input_sources`, `reasoning_summary`
- Implement W3C PROV-compatible provenance records alongside existing audit logs
- Use NATS message correlation IDs to build provenance graphs across agent interactions
- Store provenance in PostgreSQL with graph query capabilities (recursive CTEs or pgvector for embedding-based retrieval)

---

## Theme 10: Consensus-Free Multi-Agent Debate and Review Systems

### What Was Found

**Free-MAD: Consensus-Free Multi-Agent Debate** (Cui et al., 2025, ArXiv) eliminates the requirement for agents to reach consensus. Uses a score-based decision mechanism that evaluates entire debate trajectories rather than final-round majority voting. Introduces "anti-conformity" to prevent LLM agents from being swayed by incorrect majority opinions. Significantly improves reasoning with single-round debate while reducing token costs.

**MARS: Multi-Agent Review System** (Wang et al., 2025, ArXiv) implements a role-based review process: author generates, reviewers evaluate independently, meta-reviewer synthesizes. Matches debate accuracy with 50% less token usage and inference time by avoiding reviewer-to-reviewer interactions.

**Multi-Agent Debate with Adaptive Stability Detection** (Hu et al., 2025, ArXiv) formalizes debate mathematically, proving that debate amplifies correctness vs. static ensembles. Uses Beta-Binomial mixture models and Kolmogorov-Smirnov tests for adaptive stopping when consensus is detected.

### Why It Matters for Mister Smith

Mister Smith's nine agent roles include review-oriented roles that could benefit from structured debate/review patterns. The key insight from Free-MAD is counterintuitive: forcing consensus can actually degrade quality due to LLM conformity bias. MARS's review pattern (author -> independent reviewers -> meta-reviewer) maps perfectly to software development workflows (developer -> code reviewers -> tech lead).

### Connection to Existing Research

Our existing agent orchestration supports team-based coordination but doesn't implement structured debate/review patterns. The anti-conformity finding challenges the assumption that agent agreement signals correctness -- important for our supervision system.

### Implementation Ideas

- Implement `ReviewProtocol` trait: `Author`, `Reviewer`, `MetaReviewer` roles
- Use MARS-style independent review (no reviewer-to-reviewer communication) to reduce token costs
- Track "trajectory scores" across agent interactions for quality assessment
- Integrate adaptive stopping (K-S test on consensus) to avoid unnecessary debate rounds

---

## Theme 11: Small Language Models (SLMs) as Default for Agentic Tasks

### What Was Found

**SLMs for Agentic Systems** (Sharma & Mehta, 2025, ArXiv) provides comprehensive evidence that 1-12B parameter models are often sufficient and sometimes superior for schema-constrained agentic workloads. With guided decoding (XGrammar, Outlines) and JSON Schema enforcement, SLMs match or surpass larger models at 10-100x lower cost. Proposes SLM-default/LLM-fallback architecture with uncertainty-aware routing.

**Can 1B LLM Surpass 405B LLM?** (Liu et al., 2025, ArXiv, 106 citations) demonstrates that with compute-optimal test-time scaling, a 0.5B model outperforms GPT-4o and a 7B model beats o1 and DeepSeek-R1. The optimal strategy is highly dependent on policy model, Process Reward Model, and problem difficulty.

**Thinking-Optimal Scaling** (Yang et al., 2025, ArXiv, 81 citations) reveals that excessively long chain-of-thought can impair reasoning. There exists an optimal CoT length distribution per domain. Self-improved models match teacher models.

### Why It Matters for Mister Smith

These findings fundamentally change the economics of model routing in Phase 9. For many of Mister Smith's structured tasks (code formatting, JSON generation, test execution, schema validation), small local models with guided decoding could replace expensive API calls. The SLM-default/LLM-fallback pattern is directly implementable and could reduce costs by 10-100x for routine tasks.

### Connection to Existing Research

Our model routing research focuses on routing between different API-hosted models. The SLM research suggests that local inference with small models should be the primary tier, with API models as fallback. This represents a shift from "which cloud model" to "cloud vs. local."

### Implementation Ideas

- Add `LocalModelProvider` alongside `OpenAIProvider` and `ClaudeProvider` in Phase 9
- Implement guided decoding for structured outputs (JSON tool calls, code blocks)
- Build confidence-based routing: SLM attempts first, escalates to LLM on low confidence
- Track cost-per-successful-task (CPS) metric to empirically optimize routing thresholds

---

## Theme 12: Context Window Management via Summarization

### What Was Found

**SUPO: Summarization-augmented Policy Optimization** (Lu et al., 2025, ArXiv, 2 citations) trains LLM agents to compress tool-use history via summarization while optimizing both tool-use behavior and summarization strategy end-to-end via RL. Enables scaling beyond fixed context limits.

**ReSum** (Wu et al., 2025, ArXiv, 9 citations) enables indefinite agent exploration through periodic context summarization. Converts interaction histories into compact reasoning states. With 1K training samples, achieves state-of-the-art on BrowseComp.

**Event-centric Memory** (Zhou, 2025, ArXiv) represents conversation history as event-like propositions (neo-Davidsonian event semantics) organized in a heterogeneous graph. Non-compressive: preserves information while making it more accessible. Matches or surpasses baselines with shorter QA contexts.

### Why It Matters for Mister Smith

Long-running agent workflows in Mister Smith inevitably consume context windows. The current architecture doesn't address this -- agents simply run until context is exhausted. SUPO's key insight is that summarization strategy should be co-optimized with task behavior, not bolted on as an afterthought. The event-centric approach is particularly interesting because it preserves provenance (who did what when) while reducing context size.

### Connection to Existing Research

Our neural paging research addresses context management conceptually. These papers provide concrete, proven implementations. The event-centric approach connects to the provenance tracking theme (Theme 9) -- event propositions serve both as context compression and as provenance records.

### Implementation Ideas

- Implement a `ContextManager` trait that summarizes agent interaction history at configurable intervals
- Use event-centric decomposition for agent logs: each tool call / LLM response becomes an indexed event
- Store summarized context in JetStream KV for fast retrieval by resuming agents
- Track "context utilization efficiency" (task quality / context tokens consumed)

---

## Theme 13: Formal Verification of Agent-Generated Code

### What Was Found

**Astrogator** (Councilman et al., 2025, ArXiv, 2 citations) introduces a Formal Query Language for representing user intent in a verifiable manner, then verifies LLM-generated code against this specification. Verifies correct code in 83% of cases and identifies incorrect code in 92%.

**PREFACE** (Jha et al., 2025, GLSVLSI, 3 citations) couples LLMs with an RL agent that explores the prompt-code space to steer the LLM toward formally verifiable outputs. Model-agnostic framework that raises verification success by up to 21%.

**Neural Theorem Proving** (Rao et al., 2025, ArXiv, 3 citations) generates formal proofs in Isabelle using a two-stage training process (SFT + RL). Used to verify AWS S3 bucket access policy correctness.

### Why It Matters for Mister Smith

Mister Smith orchestrates code-generating agents. Current quality assurance relies on test execution and code review by other agents. Formal verification adds a mathematically rigorous check: does the generated code provably satisfy the specification? This is a higher bar than testing and could be integrated as an optional quality gate for critical code.

### Connection to Existing Research

Not covered in our existing research. This is a genuinely new capability dimension. The PREFACE approach is particularly relevant because it's model-agnostic (like Mister Smith) and uses RL to improve LLM outputs -- could be integrated into the tool-use training loop.

### Implementation Ideas

- Add a `FormalVerifier` tool that agents can invoke on generated code
- Start simple: property-based testing via proptest (Rust) as a lightweight formal verification step
- For critical paths, integrate with a Lean4 or Dafny verifier as an external tool
- Use PREFACE's approach: on verification failure, re-prompt the LLM with error metadata

---

## Theme 14: Agent Interoperability Protocols Beyond MCP

### What Was Found

**Survey of Agent Interoperability Protocols** (Ehtesham et al., 2025, ArXiv, 44 citations) compares four protocols:
- **MCP** (Anthropic): JSON-RPC client-server for tool invocation. Mister Smith already supports this.
- **ACP** (Agent Communication Protocol): RESTful HTTP with MIME-typed multipart messages, session management, DID-based identity.
- **A2A** (Google): Peer-to-peer task delegation with Agent Cards (capability descriptions).
- **ANP** (Agent Network Protocol): Open network discovery with W3C DIDs and JSON-LD graphs.

Proposes phased adoption: MCP -> ACP -> A2A -> ANP.

### Why It Matters for Mister Smith

Mister Smith's MCP crate (Phase 4) handles tool integration but doesn't address agent-to-agent interoperability. As the ecosystem matures, Mister Smith agents will need to communicate with agents from other frameworks. A2A's Agent Cards are particularly relevant -- they're analogous to service discovery in microservices but for AI agents, describing capabilities, input/output schemas, and trust requirements.

### Connection to Existing Research

We have deep research on MCP patterns via Phase 4. The other three protocols (ACP, A2A, ANP) represent gaps. A2A is most immediately relevant because Mister Smith's 9 agent roles could each publish an Agent Card describing their capabilities.

### Implementation Ideas

- Implement Agent Cards as a discovery mechanism over NATS (agents publish capability descriptions)
- Consider A2A support as a future transport option alongside MCP
- Use ANP's DID-based identity as a potential replacement for JWT-only auth in cross-organizational scenarios

---

## Theme 15: RL-Trained Agentic Workflows (Flow-GRPO)

### What Was Found

**AgentFlow with Flow-GRPO** (Li et al., 2025, ArXiv, 2 citations) trains a four-module agentic system (planner, executor, verifier, generator) in-the-flow -- directly optimizing the planner within the live multi-turn execution loop. Converts multi-turn RL optimization into tractable single-turn updates. 7B model outperforms GPT-4o on multiple benchmarks by 14.9% (search), 14.0% (agentic), 14.5% (math), 4.1% (science).

**JoyAgents-R1** (Han et al., 2025, ArXiv, 3 citations) applies GRPO to joint training of heterogeneous multi-agent systems. Introduces adaptive memory evolution that repurposes RL rewards as supervisory signals for memory management.

**ToolBrain** (Le et al., 2025, ArXiv) provides a framework for RL-training tool use in agents. Supports GRPO, DPO, and SFT strategies. Achieves 30% improvement in tool-use skills with knowledge distillation.

### Why It Matters for Mister Smith

While Mister Smith uses pre-trained models via API, understanding RL-trained agentic workflows matters for two reasons: (1) it reveals the optimal decomposition of agent workflows (planner/executor/verifier/generator), which Mister Smith should mirror in its role assignments, and (2) as open-weight RL-trained agent models become available, Mister Smith could host them locally alongside API models.

### Connection to Existing Research

Our agentic loop research covers ReAct, LATS, MCTS but not RL-trained workflows. The Flow-GRPO four-module architecture (planner, executor, verifier, generator) suggests a canonical decomposition that Mister Smith's agent roles should support.

### Implementation Ideas

- Align agent roles with the planner/executor/verifier/generator decomposition
- Implement configurable agentic loops that can switch between ReAct and P-t-E patterns
- Track trajectory-level outcomes to build reward signals for future optimization
- Consider LoRA-fine-tuned local models for specialized agent roles (e.g., a fine-tuned code review verifier)

---

## Top 10 Discoveries

Ranked by impact potential for Mister Smith, prioritizing findings that our existing research missed entirely:

### 1. CodeCRDT: Observation-Driven Coordination via CRDTs (Theme 1)
**Impact: Transformative.** A fundamentally different coordination model that could replace message passing for shared-artifact collaboration. Directly applicable to concurrent code editing agents. Maps perfectly to our NATS + JetStream KV infrastructure. Zero merge failures in evaluation. We had not considered CRDTs at all.

### 2. Multi-Agent Architecture Search / Agentic Supernet (Theme 3)
**Impact: High.** 52 citations in months. Automatic discovery of optimal agent team compositions per task, using 6-45% of costs while outperforming static designs. Converts our hardcoded team definitions into a dynamic, self-optimizing system. Entirely absent from our existing research.

### 3. AgentOps and eBPF-Based Agent Observability (Theme 4)
**Impact: High.** Purpose-built observability for non-deterministic AI systems, addressing the "semantic gap" between intent and action that standard tracing misses. AgentSight's eBPF approach is framework-agnostic and open source. We built traditional observability but missed the AI-specific challenges.

### 4. Cognitive Load-Aware Inference / Token Economics (Theme 7)
**Impact: High.** A principled framework for managing the primary operational cost (tokens). 45% token reduction without accuracy loss. The three-load-type taxonomy provides a practical engineering tool for optimizing every LLM call. Not addressed in any of our existing research.

### 5. Process Reward Models for Step-Level Verification (Theme 6)
**Impact: High.** Enables verification of each reasoning step in multi-step workflows, catching errors early. The RSD finding (4.4x FLOP reduction by dynamically escalating to stronger models per-step) directly enhances our model routing architecture. Not in our existing research.

### 6. SLM-Default/LLM-Fallback Architecture (Theme 11)
**Impact: High.** Empirical evidence that 1-12B models suffice for most agentic tasks with guided decoding. 10-100x cost reduction for structured outputs. Changes the economics of Phase 9 LLM provider integration fundamentally. Our model routing research assumed cloud-hosted models.

### 7. Multi-Agent Security: Inter-Agent Communication Hijacking (Theme 8)
**Impact: High (risk mitigation).** Demonstrated 58-100% attack success rate on multi-agent systems via content injection in inter-agent messages -- even when individual agents resist injection. Our security layer addresses infrastructure security but not this vector. Critical gap.

### 8. PROV-AGENT: W3C PROV-Based Agentic Provenance (Theme 9)
**Impact: Medium-High.** Standardized provenance tracking that captures the "why" behind agent decisions, not just the "what." Essential for debugging non-deterministic workflows and for trust/compliance. Not covered in our existing research.

### 9. DAG-Based Parallel Execution with Dynamic Re-Planning (Theme 2)
**Impact: Medium-High.** Principled approach to parallel task execution that reduces steps by 35%. AgentNet's decentralized DAG topology aligns with our supervision tree model. Partially covered by our agentic loop research but the DAG formalization and P-t-E security analysis add significant new insights.

### 10. Consensus-Free Multi-Agent Debate (Theme 10)
**Impact: Medium.** The anti-conformity finding is counterintuitive and important: LLM agents tend toward groupthink, so forcing consensus can degrade quality. The MARS review pattern (author -> independent reviewers -> meta-reviewer) maps directly to software development workflows. New coordination pattern not in our existing research.

---

## Honorable Mentions

- **Hierarchical/Episodic Memory** (Theme 5): Important for long-running agents but partially covered by our neural paging and persistence research.
- **Context Summarization** (Theme 12): Critical for long-horizon tasks. SUPO's co-optimization of summarization with task behavior is novel. Related to our neural paging research.
- **Formal Verification of Generated Code** (Theme 13): Exciting but longer-term. Could integrate via external tools.
- **Swarm Intelligence for Workload Balancing**: ACO-based load balancing for AI inference is interesting but our NATS-based load distribution is already well-designed.
- **Symbolic Mixture-of-Experts**: Skill-based routing across multiple pre-trained models is relevant to model routing but our existing research covers this dimension.

---

## Summary Statistics

| Metric | Count |
|--------|-------|
| Total searches executed | 32 |
| Total papers reviewed | ~96 |
| Papers from 2025 | ~96 (100%) |
| Papers with 10+ citations | ~15 |
| Papers with 50+ citations | 3 |
| Themes identified | 15 |
| Genuinely novel findings (not in existing research) | 10 |
| Findings that challenge existing assumptions | 3 |
| Directly implementable findings | 8 |

## Key Takeaway

The single biggest gap in our existing research is the absence of **CRDT-based coordination** and **agentic architecture search**. CRDTs provide a mathematically grounded alternative to explicit message passing that naturally maps to our NATS + JetStream infrastructure. Agentic architecture search (MaAS) transforms our hardcoded agent team definitions into a dynamic, self-optimizing system that allocates resources based on task characteristics. Together, these two innovations could fundamentally change how Mister Smith orchestrates agent collaboration.

The second biggest gap is **AI-specific observability** (AgentOps). Our Phase 8 observability is standard distributed systems monitoring. The papers here demonstrate that AI agent systems have fundamentally different observability needs: non-deterministic flows, semantic gap between intent and action, reasoning loop detection. Without AI-specific observability, we cannot effectively debug, optimize, or secure multi-agent workflows.
