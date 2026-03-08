---
version: R1
created: 2026-03-07
updated: 2026-03-07
type: prompt
tier: 1
---

# Deep Research Prompt: Dynamic Self-Organization & Meta-Orchestration

## Context

Mister Smith is a Rust-based multi-agent orchestration framework built on NATS/JetStream messaging and OTP-style supervision trees. It currently has 9 predefined agent roles (Planner, Executor, Critic, Researcher, etc.) with static team-based orchestration — a Planner creates a plan, the Orchestrator assigns agents to roles, and the team executes.

This is how every competing framework works. It is also a ceiling.

## Frontier-First Mandate

Mister Smith must be engineered to become the architectural standard for agent orchestration. Incremental imitation of existing frameworks is failure. For this research dimension specifically: the question is not "how to optimize static agent teams" but "how to build a system that discovers, generates, and evolves its own agent configurations."

## Research Objective

Investigate dynamic self-organization: systems where agent topologies, team compositions, role assignments, and coordination protocols are not designed by humans but are discovered, generated, or evolved by the system itself. This is meta-orchestration — the orchestration of orchestration.

The key papers identified so far are MaAS (Multi-agent Architecture Search via Agentic Supernet, Zhang et al. 2025, 52 citations), AutoMaAS (self-evolving architecture search), and MAS^2 (Self-Generative, Self-Configuring, Self-Rectifying Multi-Agent Systems, Wang et al. 2025). These establish that dynamic architecture search outperforms static designs at 6-45% of inference costs.

Go deeper. Go wider. Find what these papers don't cover.

## Research Dimensions

### 1. Architecture Search for Agent Teams
- How does MaAS's "agentic supernet" work in detail? What is the search space? How are configurations sampled and evaluated?
- What optimization methods work for agent architecture search — RL, evolutionary algorithms, Bayesian optimization, neural architecture search transfer?
- AutoMaAS extends MaAS with automatic operator generation/fusion/elimination. What are these operators and how do they compose?
- What is the training data and evaluation protocol? Can architecture search be done online (during deployment) or only offline?
- What constraints matter — latency budgets, cost ceilings, capability requirements, safety constraints?
- How does this relate to neural architecture search (NAS) from deep learning? What lessons transfer?

### 2. Recursive Self-Generation
- MAS^2 proposes a tri-agent meta-system (generator-implementer-rectifier) that creates bespoke MAS architectures per problem. How does the generator decide what to create? What representation does it use for agent architectures?
- What prevents degenerate or adversarial self-generation? What safety constraints are needed?
- Is there research on self-modifying agent systems from the classical AI literature (SOAR, ACT-R, meta-reasoning)?
- Can a meta-agent system learn to generate architectures it has never seen before, or does it only recombine known patterns?
- What is the relationship between MAS^2 and program synthesis / code generation? Is the generator essentially writing orchestration code?

### 3. Decentralized Self-Organization
- AgentNet (Yang et al. 2025) and FoA (Giusti et al. 2025) demonstrate decentralized coordination without central orchestrators. How do agents discover each other's capabilities and form dynamic teams?
- FoA uses Versioned Capability Vectors (VCVs) indexed in sharded HNSW structures for semantic matching. What are the latency and accuracy characteristics?
- AgentNet uses retrieval-based memory for continual skill refinement. How does this work? What memory architecture?
- DynTaskMAS (Yu et al. 2025) shows near-linear scaling to 16 agents with dynamic task graphs. What is the scaling limit? What breaks at 50, 100, 1000 agents?
- How do decentralized topologies handle failure and recovery? If there's no central orchestrator, who restarts failed agents?
- What does the swarm intelligence literature (ant colony optimization, particle swarm, boids) say about emergent organization at scale?

### 4. Evolving Orchestration
- Dang et al. (2025) use a RL-learned "puppeteer" orchestrator that adaptively sequences agents as task states evolve. What RL algorithm? What reward signal? How does it generalize?
- Can orchestration strategies evolve during deployment based on observed performance?
- What is the relationship between evolving orchestration and online reinforcement learning? Can we use bandit algorithms (UCB, Thompson sampling) for orchestration decisions?
- How do you prevent catastrophic forgetting when the orchestration policy evolves?

### 5. Integration with Supervision Trees
- This is Mister Smith's unique angle: how does dynamic self-organization compose with OTP-style supervision? If agents self-organize into a DAG, who supervises the DAG?
- Can supervision strategies themselves be dynamically selected based on the agent configuration?
- What happens when a self-organized topology encounters a fault? Does it self-heal into a different topology?
- Are there formal models (process algebras, category theory) for supervised self-organizing systems?

### 6. Integration with NATS Infrastructure
- NATS subject-based routing and queue groups provide natural infrastructure for dynamic topologies. Can agent self-organization be expressed purely through NATS subscription patterns?
- JetStream KV could store agent capability descriptions and team configurations. How does this interact with architecture search?
- NATS service discovery ($SRV.PING/$SRV.INFO) already supports dynamic service registration. Can this be extended for agent capability advertisement?

## What We Already Know (Do Not Rediscover)
- MaAS basic concept and results (52 citations, 6-45% cost reduction)
- MAS^2 basic concept (19.6% improvement over static MAS)
- AgentNet decentralized DAGs (16 citations)
- FoA and VCVs (basic concept)
- DynTaskMAS near-linear scaling
- AutoMaAS existence and self-evolving extension

We need: implementation details, failure modes, scaling limits, composition with supervision, formal foundations, and adjacent-field patterns that transfer.

## Output Structure

For each dimension:
1. **State of the art** — what exists, with specific citations
2. **Key techniques** — specific algorithms, representations, or architectures
3. **Applicability to Rust + NATS + OTP** — how well does this transfer to Mister Smith's infrastructure?
4. **Open problems** — what doesn't work yet, what's unsolved, what's speculative
5. **Implementation path** — concrete sketch of how this could be built in Mister Smith

Conclude with a synthesis: what is the most viable path to making Mister Smith's orchestration layer self-organizing? What ships first? What requires more research?

## Research Methodology

1. Start with the papers listed above and trace their citation graphs — who cites them, who they cite
2. Search for "multi-agent architecture search", "self-organizing multi-agent systems", "meta-orchestration", "automatic team composition"
3. Look beyond LLM agents: classical multi-agent systems (FIPA, JADE), swarm robotics, distributed computing self-organization
4. Look at neural architecture search (NAS) literature for transferable search strategies
5. Look at evolutionary computation and genetic programming for architecture evolution
6. Prioritize papers from 2025-2026 but include foundational work if directly relevant
7. Focus on what fails, what scales, and what composes — not just what works in isolation
