---
version: R1
created: 2026-03-07
updated: 2026-03-07
type: prompt
tier: 1
---

# Deep Research Prompt: Step-Level Intelligence — Verification, Routing, and Budgeting Per Reasoning Step

## Context

Mister Smith is a Rust-based multi-agent orchestration framework built on NATS/JetStream messaging and OTP-style supervision trees. Its model routing layer selects which LLM to use for each task. Its supervision layer detects failures at the actor level.

Between these two layers is a gap: nobody verifies, routes, or budgets at the reasoning-step level. A Planner agent generating a 10-step plan uses the same model for every step, receives no verification until the plan is complete, and has no budget constraint per step. If step 3 is wrong, steps 4-10 are wasted.

## Frontier-First Mandate

Step-level intelligence is a new granularity that no competing framework operates at. OpenAI Agents SDK, LangGraph, CrewAI, AutoGen — they all route per task and verify per output. Mister Smith can route, verify, and budget per reasoning step within a task. This changes the cost model, the quality model, and the failure detection model simultaneously.

## Research Objective

Investigate how Process Reward Models (PRMs), Cognitive Load-Aware Inference (CLAI), and related techniques can be integrated into a multi-agent orchestration framework to provide step-level verification, per-step model routing, and per-step token budgeting. The goal is to design a "step intelligence" layer that sits between task-level orchestration and token-level streaming.

## What We Already Know (Do Not Rediscover)

- **R-PRM** (She et al. 2025, 19 citations): Bootstraps step-by-step evaluation from limited annotations, outperforms baselines by 11.9 F1
- **CRM** (Zhang et al. 2025): Temporal conditioning — conditions each step's reward on preceding steps AND final outcome, resolves credit assignment
- **RSD** (Liao et al. 2025, 63 citations): Combines PRMs with speculative decoding — PRM evaluates intermediate steps, dynamically invokes more powerful model. 4.4x FLOP reduction with better accuracy
- **Uncertainty-Aware Verification** (Ye et al. 2025): CoT Entropy for PRM uncertainty quantification
- **CLAI** (Zhang 2025): Three-load taxonomy (intrinsic/extraneous/germane), 45% token reduction
- **Computational economics in LLMs** (Reddy et al. 2025): LLMs reallocate attention under scarcity, ~40% FLOP reduction
- **Optimal CoT length** (Yang et al. 2025, 81 citations): Excessively long chain-of-thought impairs reasoning. Optimal length exists per domain.
- **Difficulty-aware routing** (DAAO, Su et al. 2025): Variational autoencoders for query difficulty prediction

We need: integration architecture for an orchestration framework, performance at inference time, composition with supervision and streaming, practical deployment patterns, and the feedback loop design.

## Research Dimensions

### 1. Process Reward Models — Deep Dive
- What is the architecture of a PRM? How large does it need to be to provide useful step-level signals? Can a small model (1-3B) serve as a PRM for a large model (70B+)?
- What is the inference latency of a PRM evaluation? Can it run in parallel with the next step's generation?
- How do you define "steps" for non-mathematical reasoning? PRM research focuses on math (ProcessBench, PRMBench). What about code generation, planning, research, writing?
- CRM's temporal conditioning — does conditioning on the final outcome require waiting for task completion, or can it work with partial trajectories?
- Training-free PRMs: Are there approaches that don't require training a reward model at all? (e.g., using LLM self-evaluation, entropy-based signals, embedding similarity)
- What is the relationship between PRMs and Monte Carlo Tree Search (MCTS)? MCTS already uses value estimates for tree node evaluation — can PRMs serve as the value function?

### 2. Per-Step Model Routing
- RSD dynamically invokes a more powerful model per step. What is the latency cost of model switching mid-task?
- Can routing decisions be made based on PRM confidence? (Low confidence on step N -> route step N+1 to a stronger model)
- What is the optimal routing strategy: always start cheap and escalate, or predict difficulty upfront?
- How does per-step routing interact with context management? If you switch models mid-task, how do you transfer context?
- Can guided decoding (constrained JSON output) from a small model replace unconstrained generation from a large model for structured steps?
- What does the speculative decoding literature say about draft-then-verify patterns? (Google's speculative decoding, Medusa, EAGLE)

### 3. Per-Step Token Budgeting (CLAI Integration)
- CLAI's three-load taxonomy: how do you measure intrinsic load (problem complexity), extraneous load (wasteful computation), and germane load (productive reasoning) in practice?
- Can intrinsic load be estimated before generation begins? What signals predict step complexity?
- How do you enforce a token budget per step? (max_tokens per API call? early stopping on low-value continuation? truncation?)
- What is the relationship between step budgeting and the "optimal CoT length" finding? Can we predict the right CoT length per step?
- How does per-step budgeting compose with streaming? If a step hits its budget mid-stream, how do you gracefully terminate?

### 4. Integration with Agentic Loops
- How does step-level verification interact with ReAct, LATS/MCTS, and Plan-then-Execute loop patterns?
- In a MCTS-style loop, PRM scores can serve as node values. How does this change the search efficiency?
- Can step-level verification trigger backtracking without failing the entire task? (Step failed verification -> retry step, not restart task)
- How does the supervision tree interact with step-level failures? Should the supervisor be aware of step-level events?
- What is the boundary between "the model is reasoning through steps" and "the framework is orchestrating steps"? Where should the step boundary be defined — by the model or by the framework?

### 5. Integration with Streaming Architecture
- PRM evaluation needs access to the completed step. In a streaming architecture, when is a "step" complete?
- Can PRM evaluation run in the streaming pipeline? (Step completes in stream -> PRM evaluates -> next step begins)
- How does step-level verification interact with the dual-stream design (lossless semantic stream + best-effort UI stream)?
- Can the streaming layer detect step boundaries automatically (e.g., by parsing structured output, detecting reasoning markers)?

### 6. Feedback Loops and Continuous Improvement
- PRM evaluations produce quality signals per step. Can these be aggregated to improve routing decisions over time?
- CLAI's load estimates could be refined empirically as the system observes actual step outcomes. How do you build this feedback loop?
- Can step-level signals be used to update the agent's prompt or system instructions dynamically? (If step verification fails repeatedly on a certain type of reasoning, adjust the prompt)
- How does this connect to the MaAS architecture search? Can step-level performance data inform team composition decisions?

### 7. Adjacent-Field Techniques
- What do compiler optimization literature (program analysis, intermediate representation optimization, speculative execution) offer for step-level intelligence?
- What does the educational assessment literature (item response theory, adaptive testing) say about step-level evaluation?
- What do game AI systems (evaluation functions in chess/Go engines) say about per-move evaluation that transfers to per-step reasoning evaluation?
- What does the control theory literature (model predictive control, receding horizon optimization) offer for step-by-step decision making under uncertainty?

## Output Structure

For each dimension:
1. **State of the art** — what exists, with citations
2. **Key techniques** — specific models, algorithms, or protocols
3. **Latency and cost profile** — can this run at inference time without unacceptable overhead?
4. **Integration with Mister Smith** — how it composes with actors, supervision, streaming, NATS
5. **Open problems** — what's unsolved, what fails, what's speculative

Conclude with:
- An architecture sketch for "step intelligence" in Mister Smith: where does it live, what does the API look like, how does it interact with the model router and supervision tree?
- A concrete implementation path: what ships first (simplest step verification), what comes next (per-step routing), what's longer-term (learned budgeting)?
- An honest assessment: is step-level intelligence a fundamental capability or an optimization? Does it change the framework's architecture or just improve its outputs?

## Research Methodology

1. Start with PRM/CRM/RSD papers and trace citation graphs
2. Search for "step-level verification", "process supervision", "reasoning verification", "intermediate reward models"
3. Deep dive into speculative decoding literature for draft-then-verify patterns
4. Look at Monte Carlo Tree Search literature for value function integration
5. Search for CLAI extensions, token budgeting, compute-optimal inference
6. Look at compiler optimization (speculative execution, branch prediction) for analogies
7. Search for production deployments of step-level techniques — who is actually using this?
8. Prioritize 2025-2026 papers; include foundational work on process supervision (Lightman et al. 2023) and speculative decoding (Leviathan et al. 2023)
