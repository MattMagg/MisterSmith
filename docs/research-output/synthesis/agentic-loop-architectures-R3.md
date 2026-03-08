---
version: R3
created: 2026-03-07
updated: 2026-03-07
sources: Ultra2x (3 reports) → Synthesized
round: 3 (Triple Synthesis)
---

# Agentic Loop Architectures for Mister Smith: A Comprehensive Research Synthesis

## Executive Summary

This report synthesizes three independent research analyses on agentic loop architectures, converging on a unified design blueprint for Mister Smith -- a Rust + NATS + OTP-supervised multi-agent orchestration framework. All three reports independently arrive at the same high-confidence conclusion: **production-grade agentic loops must move beyond linear ReAct/turn-counter patterns toward event-sourced, supervised, search-capable loop engines built on explicit actor decomposition.** The convergence across independently conducted research lends strong confidence to this architectural direction.

The core findings, distilled:

1. **Reasoning architectures must support search, not just sequence.** Linear ReAct loops top out at ~85% success rates on benchmark tasks; tree-search methods (LATS/MCTS) achieve up to 94.4% pass@1 on HumanEval. All three reports converge on LATS as the current state-of-the-art reasoning architecture, and all three independently recommend mapping MCTS to actor supervision trees. Hierarchical designs (ReCAP, ReAcTree) address long-horizon coherence by preventing context drift through structured replanning.

2. **The actor model is an exceptionally strong fit.** Candidate reasoning trajectories map naturally to isolated child actors; search control maps to supervisor actors; tool execution maps to supervised worker pools. This is not merely an implementation convenience -- it is the architectural mechanism for isolation, restart, backtracking, exploration, and safe termination. Rust actor frameworks (e.g., Agentor) demonstrate 38ms cold starts, 42MB memory per agent, and 200+ concurrent agents on a 4-core machine.

3. **State management must be event-sourced.** JetStream's append-only log with replay capabilities natively solves LLM backtracking, checkpoint/restore, and audit trail requirements without complex in-memory state cloning. All three reports converge on JetStream as the durable state backbone, with exactly-once semantics via `Nats-Msg-Id` deduplication.

4. **Self-evaluation must be continuous, not post-hoc.** The Critic role should function as a value function service -- continuously scoring partial trajectories, detecting stuck loops, and triggering backtracking -- not merely reviewing final outputs. Reflexion-style episodic memory and Constitutional AI-style critique loops demonstrably improve outcomes.

5. **Budgets, context, and safety are first-class architectural concerns.** Cascading multi-dimensional budgets (token/cost/time), active context compression (22.7% token reduction with identical accuracy), approval gates with durable pause/resume, and Wasmtime sandboxing for untrusted code execution are all required for enterprise deployment.

6. **Classical AI planning patterns transfer directly.** BDI (Belief-Desire-Intention), HTN (Hierarchical Task Networks), Behavior Trees, and the "LLM-modulo" framing (LLM proposes, external verifier validates) all provide deterministic structure around probabilistic inference. These patterns increase predictability and testability by making control flow explicit rather than emergent.

---

## Table of Contents

1. [Competitive Landscape and the Mister Smith Opportunity](#1-competitive-landscape-and-the-mister-smith-opportunity)
2. [Reasoning Loop Architectures](#2-reasoning-loop-architectures)
3. [Multi-Turn Tool Calling Patterns](#3-multi-turn-tool-calling-patterns)
4. [Self-Evaluation and Self-Correction](#4-self-evaluation-and-self-correction)
5. [Backtracking and Rollback](#5-backtracking-and-rollback)
6. [State Management and Event Sourcing](#6-state-management-and-event-sourcing)
7. [Budget and Resource Management](#7-budget-and-resource-management)
8. [Context Window and Memory Management](#8-context-window-and-memory-management)
9. [Cognitive Architecture and Classical Planning Patterns](#9-cognitive-architecture-and-classical-planning-patterns)
10. [Actor-Model Integration and OTP Supervision](#10-actor-model-integration-and-otp-supervision)
11. [Production Safety and Governance](#11-production-safety-and-governance)
12. [Synthesis: The Mister Smith Architecture Blueprint](#12-synthesis-the-mister-smith-architecture-blueprint)
13. [Evaluation Rubric for Candidate Architectures](#13-evaluation-rubric-for-candidate-architectures)
14. [Prioritized Prototypes and Experiments](#14-prioritized-prototypes-and-experiments)
15. [Evidence Gaps](#15-evidence-gaps)
16. [References](#16-references)

---

## 1. Competitive Landscape and the Mister Smith Opportunity

### Python's Concurrency and State Limits Create an Enterprise Void

The 2026 agentic framework market is dominated by Python-based orchestration, which introduces severe production limitations. Python's Global Interpreter Lock means that multi-agent orchestration is fundamentally single-threaded [R1-3]. Cold start times range from 2-5 seconds depending on the dependency tree, and memory consumption for a single agent with a modest tool set sits around 200-400MB [R1-3]. Frameworks like AutoGen treat workflows as conversations between agents, while LangGraph represents them as graphs with nodes and edges [R1-4]. Both approaches typically manage state through in-memory data structures or database snapshots, lacking the immutable audit trail and deterministic replay guarantees essential for production software engineering workflows [R1-2].

### Feature-Depth Comparison of 2026 Agentic Frameworks

| Framework | Architecture Model | State Management | Concurrency & Scaling | Key Production Limitation |
|:---|:---|:---|:---|:---|
| **LangGraph** | Stateful directed graphs | Checkpointing at super-steps | Highly scalable with Pragal and Apache Beam [R1-5] | Steepest learning curve; requires designing agentic architecture [R1-5] |
| **AutoGen** | Event-driven conversations / actor model (v0.4) | In-memory / Custom | Supported with agent networks [R1-5]; token/timeout termination conditions [R3-C18] | Least flexible base; actor model in v0.4 is a step forward [R1-5, R2-34] |
| **CrewAI** | Role-based sequential/hierarchical | Short-term and long-term memory | Supported with task management [R1-5] | Workflows may occasionally produce truncated outputs [R1-4]; "best answer when max iterations hit" [R3-C3] |
| **OpenAI Agents SDK** | Lightweight Python primitives | Persistent session memory; durable RunState for pause/resume [R3-C15, R3-C18] | Limited built-in parallel support [R1-5] | Lacks built-in parallel execution [R1-5]; loop bounded by `max_turns` [R3-C16] |
| **Google ADK** | Sequential, Parallel, Loop agents | Artifact management; `InvocationContext` for shared state [R3-C18] | Scales with Vertex AI Agent Engine [R1-5] | Heavyweight and enterprise-focused [R1-5] |
| **Vercel AI SDK** | Multi-step tool calling with `stopWhen` | Session-based | `parallelToolCalls` toggle per provider [R3-C14, R3-C15] | Default 20-step limit; step-counter semantics [R3-C18] |
| **Mister Smith (Target)** | Rust Actors + OTP Supervision + MCTS | JetStream Event Sourcing | True OS-level multi-threading | Requires Rust expertise |

**High-confidence finding (all 3 reports converge):** Every major production framework relies on simple turn counters (`max_turns`, `max_iterations`, `stepCountIs(n)`) as the primary loop control mechanism. None implements search-based reasoning, durable backtracking, or multi-dimensional budget enforcement as first-class loop constructs. This is Mister Smith's primary competitive gap to exploit.

### The Rust + OTP Differentiator

Rust actor frameworks fundamentally solve the resource bloat of Python agents. The Agentor Rust framework demonstrated cold start times of 38ms and a memory footprint of just 42MB per agent, allowing 200+ agents to run simultaneously on a 4-core machine with linear throughput scaling [R1-3]. By utilizing libraries like `tokio-actors`, Mister Smith can ensure every actor has a bounded mailbox (default: 64), meaning when full, senders wait automatically -- no OOM crashes from runaway queues [R1-6]. Actix actors process one message at a time and provide supervision/failure handling and mailbox policies; `tokio-actors` provides production-ready features like timer drift handling and miss policies [R2-42, R2-43, R2-47].

---

## 2. Reasoning Loop Architectures

### State of the Art

Two foundational "loop families" still dominate, with a clear evolutionary trajectory toward search-based methods.

**Interleaved reasoning + acting.** ReAct formalized the idea of alternating "reason" tokens with explicit "act" steps (tool/environment calls), improving performance and interpretability by letting actions feed back into reasoning [R2-1, R3-C0]. Reflexion then added feedback-conditioned iteration: rather than weight updates, agents store textual reflections in memory and reuse them on subsequent attempts (a "verbal RL" style loop) [R3-C0, R2-13]. REBACT extends ReAct by inserting a reflect step before acting, achieving 98.5% success rate on ALFWorld vs ReAct's 85.1% [R1-7].

**Deliberative search over thought trajectories.** Tree of Thoughts (ToT) explicitly treats reasoning steps as a search space, exploring multiple candidates via BFS/DFS, self-evaluating partial solutions, and enabling lookahead/backtracking when needed (74% success on Game of 24) [R1-8, R3-C0, R2-6]. Graph of Thoughts (GoT) generalizes this by representing intermediate "thought units" as a dependency graph (not just a tree), enabling richer composition/aggregation patterns and using graph attention to capture cross-step dependencies and prune inconsistent paths [R3-C12, R2-25, R2-26, R2-27].

**Search + value estimation + reflection.** The most notable successor direction is Language Agent Tree Search (LATS), which unifies reasoning/acting/planning by integrating Monte Carlo Tree Search (MCTS) with LM-powered value functions and self-reflections [R1-1, R2-10, R3-C0]. LATS achieves 94.4% pass@1 on HumanEval [R1-9]. In software-agent settings, SWE-Search similarly injects MCTS plus multi-agent evaluation roles (value estimation + debate) to reduce repetitive ineffective actions and improve repository-level task performance [R3-C9]. RATT (Retrieval-Augmented Thinking on Trees) layers factual retrieval and stepwise evaluation into reasoning trees to improve factual coherence [R2-21].

**Hierarchical and recursion-based control flow (2025-2026).** ReCAP frames long-horizon coherence as a hierarchical process with shared context, committing to the head subtask while refining the remainder as new observations arrive, plus bounded "sliding-window" scaling and structured re-injection of higher-level context on backtracking [R3-C1]. ReAcTree uses a dynamically built agent tree with explicit control-flow nodes (sequence, fallback, loops) coordinating execution; it also distinguishes working vs episodic memory for sharing observations vs retrieving examples [R3-C9]. EnCompass disentangles workflow logic from inference-time strategy by compiling "points of unreliability" (e.g., LLM calls) into a search space where different search policies can be swapped in [R3-C9].

### Comparison of Execution Strategies

| Strategy | Search Topology | Self-Correction | Performance Benchmark | Implementation Complexity |
|:---|:---|:---|:---|:---|
| **ReAct** | Linear | Next-step only | ALFWorld SR: 85.1% [R1-7] | Low (Simple prompt loop) |
| **REBACT** | Linear with pre-check | Inserts reflect step before act | ALFWorld SR: 98.5% [R1-7] | Medium (Requires state rollback) |
| **Reflexion** | Linear with episodic memory | Verbal reflections stored for future trials | Strong coding benchmark gains [R2-13, R3-C0] | Medium (Requires memory store) |
| **Self-Refine** | Iterative FEEDBACK/REFINE | Same model critiques and revises | Measurable quality improvement [R3-C7] | Medium (Critique + revision loop) |
| **Tree of Thought** | BFS / DFS | Evaluates partial solutions | Game of 24: 74% success [R1-8] | High (Requires state branching) |
| **Graph of Thoughts** | DAG with graph attention | Graph-level predictors + pruning | Reasoning efficiency gains [R2-25, R3-C12] | High (Graph construction + attention) |
| **LATS** | Monte Carlo Tree Search | LM-powered value functions + reflections | HumanEval pass@1: 94.4% [R1-9] | Very High (Requires MCTS engine) |
| **SWE-Search** | MCTS + multi-agent debate | Value Agent + Discriminator Agent | Improved repo-level SWE tasks [R3-C9] | Very High (MCTS + evaluator roles) |
| **ReCAP** | Hierarchical recursion | Structured context re-injection on backtrack | Sliding-window scalability [R3-C1] | High (Hierarchical actor tree) |
| **ReAcTree** | Dynamic agent tree with control-flow nodes | Working + episodic memory separation | Long-horizon coherence [R3-C9] | High (Control-flow compilation) |
| **EnCompass** | Compiled search space | Swappable inference-time strategies | Strategy-independent workflow logic [R3-C9] | High+ (Workflow compilation) |

### Five Key Technique Patterns (High-Confidence Convergence)

All three reports independently identify these as the most practically relevant techniques for "beyond linear loops":

1. **Trajectory search with evaluation** (ToT, GoT): generate multiple candidate partial trajectories; score them (self-evaluation or external evaluator) to decide expansion/backtracking [R2-6, R3-C0, R3-C12].
2. **Tree-search planning with stochastic branching and value estimates** (LATS, SWE-Search): treat each model decision as a branching point; use value functions and exploration bonuses to allocate more calls to promising branches [R1-1, R2-10, R3-C0, R3-C9].
3. **Hierarchical recursion with shared context** (ReCAP): keep global goals stable while allowing local replanning; explicitly manage what context persists across recursion levels [R3-C1].
4. **Hierarchical agent-tree execution with control-flow nodes** (ReAcTree): make control flow explicit (sequence, fallback, loops) instead of emergent in a monolithic transcript [R3-C9].
5. **Workflow-as-search-space compilation** (EnCompass): disentangle workflow logic from inference-time strategy by compiling "points of unreliability" into a search space where different search policies can be swapped in [R3-C9].

### Applicability to Rust Actors + Supervision

**High-confidence finding (all 3 reports converge):** Actor systems are unusually well-aligned with search-based reasoning architectures because "candidate trajectories" can be realized as isolated child actors (one per branch), and search control can live in a supervisor (or coordinator) actor that allocates budget and terminates/forks branches. This mirrors the design intent in LATS and SWE-Search while letting Mister Smith implement exploration via message-passing and restart semantics rather than nested call stacks.

Hierarchical designs like ReCAP/ReAcTree map naturally to hierarchical actor trees: each subgoal node can be an actor with its own local loop state, while parent actors own coordination/control-flow decisions.

A "turn" becomes a message (e.g., `GenerateThought`, `EvaluateNode`, `ExecuteAction`, `ObservationReceived`) routed between these actors; each actor appends observations or evaluation scores to the node's durable record in JetStream. This permits interruption, checkpointing, and supervisor-led restarts [R2].

### Implementation Complexity Ladder

- **Low:** ReAct-style single-path loop with tool calls.
- **Medium:** Reflexion / Self-Refine style iterative critique + memory buffers.
- **High:** LATS/SWE-Search class (tree search, value functions, branch management, exploration policy).
- **High+:** EnCompass-style compilation and interchangeable inference-time strategies, plus efficient state capture across branches.

### Neurosymbolic Planning: HTN and PDDL in the Loop

Relying solely on LLMs for long-horizon planning leads to compounding errors. A novel neuro-symbolic task planner decomposes complex tasks into subgoals using LLM and carries out task planning for each subgoal using either symbolic or MCTS-based LLM planners, depending on subgoal complexity [R1-10]. This decomposition reduces planning time and improves success rates by narrowing the search space and enabling LLMs to focus on more manageable tasks [R1-10]. Mister Smith can map Belief-Desire-Intention (BDI) architectures to actor state, where beliefs represent the agent's information about its environment, goals (desires) are states of affairs to achieve, and intentions are commitments to achieving particular goals [R1-11].

---

## 3. Multi-Turn Tool Calling Patterns

### State of the Art

Production SDKs converge on a similar baseline: a runtime loop that continues model calls until the model returns final text with no tool calls, or until a termination condition fires.

- **OpenAI Agents SDK:** Loop calls the model; if `final_output`, stop; else run tool calls and continue; or hand off to another agent; `max_turns` bounds the loop. Supports human-in-the-loop pauses where the run returns "interruptions" and can resume from serialized run state after approval/rejection [R3-C16, R3-C15, R3-C13].
- **Vercel AI SDK:** "Multi-step calls" controlled by `stopWhen`; tool calls trigger SDK execution, result appending, and another step until tool calls cease or `stopWhen` is satisfied; default stop is 20 steps. `parallelToolCalls` exposed per provider. `needsApproval` supported for tools [R3-C16, R3-C18, R3-C13, R3-C14].
- **Google ADK:** `LoopAgent` repeats sub-agents; halts on `max_iterations` or an "escalate" event, sharing a persistent `InvocationContext` across iterations so state can accumulate deterministically across probabilistic model calls [R3-C18, R3-C16].
- **Anthropic:** Documents "server tools" where their servers handle tool execution in a loop; "programmatic tool calling" lets the model write orchestration code in a code execution environment, reducing per-tool round-trips and enabling parallel tool execution in-code [R3-C2, R3-C3].
- **LangChain:** Supports parallel router/subagent patterns and evaluates tradeoffs in token usage across subagents, skills, handoffs, and routers [R2-32, R2-33, R2-27].

### Key Techniques

**Sequential tool calls:** Model proposes an ordered action list; each tool result becomes next input -- simple, safe for dependent operations.

**Parallel tool calls:** Issue several independent tool calls concurrently (for latency gains); total latency equals the slowest call, and token usage may rise due to duplicated context across parallel calls [R2-35, R2-36]. Parallel tool calling improves latency when tools are independent, but introduces dependency and authorization hazards: a model may propose tool calls with implicit ordering dependencies, so the runtime must either constrain to sequential execution or topologically schedule based on declared dependencies and reject unsafe mixes [R3-C14, R3-C15].

**Router / Subagents:** A router model routes subrequests to specialized agents (parallel) or subagents maintain isolated contexts to reduce repeated tokens at the cost of some duplication; LangChain's analysis shows subagents can reduce tokens vs skills in certain patterns [R2-32, R2-29].

**Programmatic tool calling:** Anthropic's approach lets the model write orchestration code in a code execution environment, reducing per-tool round-trips and enabling parallel tool execution in-code [R3-C2].

**Runtime tool authorization (approval gates):** SDK-level step limits and hooks to approve tool invocations before execution. Vercel supports `needsApproval` for tools; OpenAI Agents SDK supports human-in-the-loop pauses with resumable `RunState` [R3-C13, R3-C15, R3-C18]. Approval gating becomes more complex under parallelism: if any tool requires approval, you need a consistent "pause boundary" that freezes the entire step until approvals resolve, otherwise partial execution can create irreversible side effects and inconsistent state.

### Parallel Fan-Out and Gather via Actors

When an LLM requests multiple tools simultaneously, Mister Smith's executor actor should spawn ephemeral worker actors for each tool call. This prevents the main agent loop from blocking [R1]. The Vercel AI SDK unifies `generateObject` and `generateText` to enable multi-step tool calling loops with structured output generation at the end [R1-19]. Mister Smith can replicate this by defining stopping conditions using a `stopWhen` equivalent, allowing the loop to pause and yield control back to the orchestrator [R1-20].

### Applicability to Rust Actors + Supervision

Actor systems can treat each tool invocation as a child actor or as a task scheduled onto a supervised worker pool. This makes:

- **Parallel tool fan-out** a first-class pattern: spawn N tool workers, gather results, then send an "observation" message back to the agent loop actor.
- **Approval gates** cleanly representable as state transitions: the loop actor enters `AwaitingApproval` and persists a resumable state snapshot.
- **Supervision** can enforce tool safety: a supervisor can kill/restart a tool worker on timeout or abnormal exit.

Safety considerations demand that ToolWorker actors be sandboxed and supervised (restart isolation) and that authorization/approval be enforced at the dispatcher (not left to model proposals) [R2-44, R2-21].

### Expected Impact vs Turn-Counter Loops

A richer tool loop architecture primarily improves: (a) **latency** (parallelism, programmatic orchestration), (b) **safety** (approval gating with durable pause boundaries), and (c) **operational reliability** (timeouts/retries per tool worker rather than failing whole runs).

---

## 4. Self-Evaluation and Self-Correction

### State of the Art

Self-correction is increasingly treated as a first-class loop, not just prompt seasoning:

- **Constitutional AI:** Uses an explicit constitution and AI feedback to critique and revise outputs during training and inference; informs runtime "constitutional critique" loops used in many agent designs [R2-61, R3-C7].
- **Reflexion:** Augments ReAct by storing verbal reflections in episodic memory and iteratively improving decision-making across trials; achieved strong gains on coding benchmarks [R2-13, R3-C0].
- **Self-Refine:** Formalizes an iterative FEEDBACK then REFINE loop using the same model to critique and improve outputs, repeatedly, until a condition is met [R3-C7].
- **LATS:** Embeds self-reflections and LM-powered value estimation inside a tree search procedure, upgrading self-evaluation from a single "critique pass" to a branch-scoring component [R1-1, R3-C0].
- **SWE-Search:** Adds explicit evaluator roles (Value Agent and Discriminator Agent for debate), combining qualitative evaluations with numeric value estimation inside MCTS-style exploration [R3-C9].
- **Two-LLM patterns (Writer/Critic):** Use one LM to generate and another (or the same with different prompt) to critique and decide whether revisions are needed [R2-61].
- **Confidence estimation:** Structural Confidence extracts lightweight features (hidden-state patterns) plus a small classifier to predict uncertainty; enables efficient stuck detection and selective escalation [R2-55]. Surveys compile techniques for confidence estimation and calibration in LLMs; newer work explores "verbalized confidence" and methods to improve calibration [R3-C7].

### Key Techniques for a Planner/Critic/Executor Trio

- **Inline self-critique loops** (Self-Refine): Cheap to integrate, but can be correlated with the original error modes [R3-C7].
- **Separated Critic role** (multi-agent): Reduces correlation by using different prompts/models/temperatures; SWE-Search's distinct evaluation agents and multi-agent debate is an explicit example [R3-C9].
- **LLM-as-a-judge** for structured evaluations: Has become a widespread paradigm, but surveys emphasize reliability pitfalls and mitigation strategies including bias-aware evaluation (both human and LLM judges exhibit biases and perturbation vulnerabilities), motivating ensemble judges, calibration, and audit tooling [R3-C7].
- **Episodic reflections:** History of verbal reflections stored in memory to inform subsequent runs (Reflexion approach) [R2-13, R2-14].

### Detecting "Stuck Loops" or Low-Quality Iteration

**High-confidence finding (all 3 reports converge):** A strong practice emerging from multiple sources is to treat "stuckness" as a multi-signal diagnosis -- repeated tool calls, repeated argument patterns, low judge scores, lack of state progress -- then trigger either strategy change or supervised restart/backtrack rather than continuing the same loop.

Production frameworks increasingly expose explicit termination conditions beyond turn counts. AutoGen documents termination conditions including token-usage and timeout constraints, not just message counts [R3-C18]. ReCAP explicitly motivates hierarchical control by noting that sequential prompting can fall into recurrent failure cycles and context drift on long-horizon tasks [R3-C1]. Report A proposes semantic circuit breakers: traditional circuit breakers catch timeouts and 500 errors, but they cannot catch an LLM confidently hallucinating sources that do not exist, or an agent stuck in a reasoning loop burning tokens without progress [R1-28]. Mister Smith must implement a DEGRADED state for partial capability: if the Critic actor detects repetitive tool calls or semantic failures, the circuit breaker trips to DEGRADED, disabling risky tools, adding human review, or switching to a conservative model [R1-28].

### Applicability to Rust Actors + Supervision

Mister Smith's existing Critic role can be elevated from "post-hoc reviewer" to a **value function service** used continuously: scoring partial trajectories, deciding whether to expand/stop/backtrack, and calibrating confidence levels before returning results. This exactly matches the role that value estimation and reflection play inside LATS and SWE-Search.

In an actor system, "self-evaluation" can be isolated into its own supervised subtree:
- Executor actor emits state updates and tool results
- Critic actor consumes them and produces scores/diagnostics
- Supervisor actor decides restarts/backtracking/escalation based on the critic's signals

This separation reduces correlated failure and allows swapping evaluation models independently (including "judge ensembles").

---

## 5. Backtracking and Rollback

### State of the Art

Backtracking in LLM agents now appears in three increasingly "industrializable" forms:

1. **Reasoning backtracking** (no external side effects): ToT explicitly supports looking ahead and backtracking among thought branches [R3-C0].
2. **Search-based action backtracking** (limited side effects): LATS uses MCTS with reflections/value functions to explore action sequences and revise decisions based on feedback [R1-1, R3-C0]. SWE-Search adapts this to software engineering repositories, adding evaluator/discriminator agents to guide exploration [R3-C9].
3. **Program/workflow backtracking with explicit nondeterminism:** EnCompass compiles workflows into a search space over "unreliable" operations like LLM calls; IterGen's structured generation exposes forward/backward navigation during generation under grammatical constraints [R3-C9].

Separately, robotics has long addressed failure recovery under real-world side effects using explicit control architectures. Behavior Trees (BTs) are a standard approach for structuring reactive, modular task execution; they are frequently used precisely because pure planning is brittle under uncertainty [R3-C6]. Robotics literature on reactive planning and plan repair further emphasizes time-bounded recovery procedures that "repair" plans at runtime rather than re-planning from scratch [R3-C6].

### Backtracking Strategies

| Backtracking Strategy | Mechanism | Best Used For |
|:---|:---|:---|
| **Checkpoint / Restore** | Replay JetStream log to a specific sequence number | Recovering from fatal tool errors or crashes [R1] |
| **Compensating Actions (Sagas)** | Execute reverse operations (e.g., delete created file) | Reverting side-effects in external APIs [R1]; mirrors transactional saga patterns [R3-C6] |
| **Branching (MCTS)** | Spawn new actor from cloned state | Exploring alternative reasoning paths in parallel [R1] |
| **Search pruning** | Alpha-beta / threshold-based pruning | Don't explore provably irrelevant branches; prune branches that violate constraints (budget, safety, consistency) [R3-C11] |
| **Hierarchical rollback** | ReCAP context re-injection on backtrack | Preserving cross-level continuity while allowing local replanning [R3-C1, R3-C9] |
| **Supervisor-triggered backtracking** | Supervisors detect failures or low-confidence outcomes | Request rollback to prior checkpoint; switch strategies on repeated failures [R2] |

### Key Techniques

- **Versioned checkpoints:** Snapshot plan/partial state and persist it to durable storage so alternative branches can be explored without losing baseline state. This generalizes the "durable pause/resume boundary" from approval interrupts to search forks [R3-C15, R2-86].
- **Compensating actions for tool side effects:** Define explicit rollback tools ("delete created record," "revert commit"), mirroring transactional saga patterns rather than strict ACID transactions [R3-C6].
- **Treat execution as search:** Model decisions are branching points; exploration uses MCTS/UCT-like allocation to promising branches [R1-1, R3-C11].

### Applicability to Rust Actors + Supervision

**High-confidence finding (all 3 reports converge):** This is one of the strongest fits for Mister Smith specifically:

- **Forking** = spawn new child actors with copied checkpoints; each child explores an alternative plan/tool sequence.
- **Backtracking** = terminate a subtree and resume from a prior checkpoint in a different branch (or restart with a different strategy).
- **Rollback** = supervisors can enforce that side-effecting tool calls must be paired with compensators; if a child fails after a side effect, the supervisor triggers compensation before retrying.

JetStream's replay capabilities natively solve the LLM backtracking problem without requiring complex in-memory state cloning [R1]. Supervisors can implement policies: on low confidence or tool failure, either (a) trigger automatic backtrack and replan, (b) escalate to Critic/Human, or (c) switch strategies (e.g., from parallel exploration to sequential focused planning) [R2].

### Implementation Complexity

Implementing pure reasoning backtracking (tree-of-thought over internal messages) is moderate. Adding tool rollback is high complexity because you must classify tools by reversibility and define compensators or idempotency rules. The payoff is that this aligns with real-world failure recovery patterns in robotics and classical execution monitoring (repair rather than restart-everything) [R3-C6].

---

## 6. State Management and Event Sourcing

### Event Sourcing the Agent's Mind

**High-confidence finding (all 3 reports converge):** Traditional frameworks overwrite state, destroying the agent's reasoning history. The ESAA (Event Sourcing for Autonomous Agents) architecture separates the agent's cognitive intention from the project's state mutation [R1-2]. Event Sourcing records every state change as an immutable event in an append-only log; the current state is a projection of these events [R1-2].

Mister Smith will use NATS JetStream for this. JetStream provides both the ability to consume messages as they are published (i.e. 'queueing') as well as the ability to replay messages on demand (i.e. 'streaming') [R1-16]. Persist every loop step (inputs, model outputs, tool calls, tool results, critic scores) to a JetStream stream so runs can be replayed and inspected; use message acknowledgements to ensure processing reliability [R3-C4].

### Idempotency and Exactly-Once Semantics

Because actors may restart and replay messages, tool execution must be idempotent. JetStream supports idempotent message writes by ignoring duplicate messages as indicated by the `Nats-Msg-Id` header [R1-17]. "Exactly-once" in practice means no duplicate effects -- even if a message is delivered twice [R1-18]. JetStream's persistence and exactly-once semantics must be handled when storing/acknowledging node state [R2-38, R2-75].

### Checkpoint Boundaries

Choose explicit checkpoint boundaries: *before side-effecting tools*, *after receiving tool results*, and *before returning final output*. This generalizes the "durable pause/resume boundary" idea of `RunState` beyond approvals to failure recovery and backtracking [R3-C15, R3-C13].

### Cryptographic Audit Trails

For SOC2 and SLSA compliance, every agent decision must be auditable. JetStream's file-based streams persist messages to disk [R1-16]. Setting `sync_interval: always` will make sure servers `fsync` after every message before it is acknowledged [R1-16]. This setting, combined with replication in different data centers or availability zones, provides the strongest durability guarantees [R1-16], creating an immutable, tamper-evident audit log of the agent's entire lifecycle.

---

## 7. Budget and Resource Management

### State of the Art

Multi-dimensional budget techniques include token budgets, cost budgets, and time budgets; query-aware budget-tier routing dynamically assigns queries to models based on cost/quality constraints for large savings [R2-25, R2-53]. BudgetThinker introduces special control tokens during inference to inform the model of remaining token budget, enabling budget-aware reasoning [R2-61, R2-20]. The INTENT framework leverages a learned language world model to simulate tool outcomes and performs calibrated Monte Carlo lookahead to estimate future costs [R1-25].

Production systems increasingly expose multiple budget knobs:
- **OpenAI Agents SDK:** Enforces turn budget via `max_turns`; tracks token usage for runs [R3-C16, R3-C10].
- **Vercel AI SDK:** Defaults agents to 20 steps; customizable `stopWhen` [R3-C18].
- **LangChain AgentExecutor:** Exposes both step and wall-clock budgets (`max_iterations`, `max_execution_time`) and supports explicit early-stopping modes [R3-C10].
- **AutoGen:** Termination conditions including token usage limits and timeouts [R3-C18].
- **Providers:** Anthropic documents rate limits in token/minute dimensions; OpenAI documents managing billing limits/usage tiers as part of production best practices [R3-C10].

### Key Techniques

**Multi-dimensional budgets:** Enforce time, token, and cost budgets simultaneously (AutoGen's token/time termination types provide a concrete precedent) [R3-C18, R3-C10].

**Hierarchical / cascading budgets:** Allocate constraints across team (root coordinator) to per-agent (Planner/Critic/Executor) to per-branch (search tree node). If the cumulative cost exceeds the allocated budget at any point, execution is immediately terminated [R1-26]. Report A recommends a dedicated Ledger Actor to enforce budgets.

**Model cascading:** Light models for classification/intent routing, mid-tier for retrieval, heavyweight for final reasoning only when confidence is low; this reduces costs while preserving quality when needed [R2-61, R2-25].

**Budget-aware planning:** Inject remaining budget into prompts (BudgetThinker) or use model-informed strategy selection (prefer short reasoning or retrieval when budget is tight) [R2-61, R2-20]. Budget-aware strategy selection is consistent with EnCompass's separation of workflow logic from inference-time strategy [R3-C9].

**"Graceful degradation" near budget exhaustion:** Some frameworks explicitly describe "best answer when max iterations hit," which can be interpreted as a policy knob. CrewAI documentation describes that once near the maximum iterations, the agent "will try its best to give a good answer" [R3-C3].

### Taming Tail Latency with Hedged Requests

LLM APIs suffer from severe tail latency. Hedging functions as an "insurance policy" that kicks in automatically when an issued request starts to slow down [R1-27]. In one BigTable benchmark, sending a hedged request after a 10ms delay reduced the 99.9th percentile latency for retrieving 1,000 keys from 1,800ms to just 74ms while incurring only a 2% increase in total requests [R1-27]. Mister Smith's LLM provider trait should automatically fire a hedged request to a fallback model if the primary model exceeds the P90 Time-To-First-Token (TTFT) threshold.

### Applicability to Rust Actors + Supervision

Budgets become enforceable system invariants when implemented as:
- A supervisor-owned Budget actor storing cascading budgets
- Per-agent allowances granted as messages ("budget grants" / `BudgetReservation` messages)
- Hard kill switches (timeouts; max tokens) enforced by supervisors
- `BudgetLow` triggers the Planner and SearchCoordinator to switch to cheaper strategies (shorter beam, retrieval-only, or model cascade)
- JetStream/Traces can record token usage per stream for postmortem billing [R2-50, R2-51]
- Observability via Traceloop/Portkey-like telemetry records usage for alerts and hard throttling

### Expected Impact vs Turn-Counter Loops

Budgets implemented as runtime-enforced resources enable graceful degradation and cost predictability. BudgetThinker-like methods can reduce token usage by informing the model of constraints. Sophisticated budgets primarily improve production safety (prevent runaway spend/latency), SLO predictability, and fairness across concurrent agent runs under external rate limits and internal capacity.

---

## 8. Context Window and Memory Management

### State of the Art

Context overflow is now treated as a first-order engineering problem. Large token windows reduce some fragmentation, but summarization, sliding windows, and RAG remain necessary to manage long-running interactions.

- **Active context compression (Focus agent):** Autonomously decides when to consolidate key learnings into a persistent "Knowledge" block and actively prunes raw interaction history. With aggressive prompting that encourages frequent compression, Focus achieves 22.7% token reduction (14.9M to 11.5M tokens) while maintaining identical accuracy [R1-22].
- **RAG:** The canonical method for injecting external knowledge via retrieval rather than stuffing full corpora into prompts; foundational RAG work formalized parametric + non-parametric memory for generation [R3-C8].
- **MemGPT:** Explicitly frames context management as an OS-like memory hierarchy with "interrupts" controlling interaction between agent and user, aiming to extend effective context under finite windows [R3-C8].
- **Chain-of-Agents:** Multi-agent long-context decomposition assigns segmented reading/reasoning across multiple agents and uses a manager to synthesize, explicitly targeting the difficulty of focusing in long contexts [R3-C12].
- **Reasoning-graph techniques:** Convert CoT streams into graphs to decide what to preserve and what to compress [R2-27].
- **ReCAP:** Claims "sliding-window scalability" where prompt size grows with depth, not total trajectory length; its "structured injection" idea explicitly suggests that *what* you reinsert on backtracking matters (parent description, latest thoughts, remaining subtasks), preventing context drift [R3-C1].
- **OpenAI Agents SDK Sessions:** Persistent memory layer maintaining conversation history across runs [R3-C15].
- **Google ADK:** `include_contents` can be set to `'none'` for stateless tasks [R3-C2].
- **Anthropic:** "Effective context engineering" guidance frames context as a finite, critical resource [R3-C8].

### Memory Tier Architecture

| Memory Tier | Technology | Access Pattern | TTL / Eviction |
|:---|:---|:---|:---|
| **Working Memory** | Actor State (RAM) | Instant, per-turn | Ephemeral (cleared on restart) [R1] |
| **Semantic Cache** | Redis / Valkey | Vector similarity search | 1 hour TTL [R1-23] |
| **Episodic Memory** | JetStream KV Store | Key-value lookup | Retained per session [R1] |
| **Reflexion Memory** | JetStream Stream | Append-only verbal reflections | Retained across trials [R2-13] |
| **Semantic Knowledge** | Weaviate / Qdrant | RAG embedding search | Persistent [R1] |
| **Archival Memory** | PostgreSQL / Object Store | Historical run logs | Persistent [R3-C8] |

Semantic caching uses vector embeddings to match queries by their meaning, not their exact text [R1-23]. If a semantically similar query is found, the corresponding response can be provided immediately, bypassing the need for an additional API call to the LLM [R1-24]. This can reduce costs by 40-80% and speed up responses by 250x [R1-23].

### Key Techniques

A mature agentic loop typically combines:
- **Sliding windows** for recent turns
- **Summaries** for older dialogue/observations
- **Structured state** for durable facts (goals, constraints, tool outputs)
- **Retrieval (RAG)** for external knowledge and prior run logs
- **Memory tiering** (working vs episodic vs archival), as highlighted in MemGPT and ReAcTree [R3-C8, R3-C9]

### Applicability to Rust Actors + Supervision

Context management is easiest to make reliable when it is not an emergent property of a single growing message array:

- Put "working state" in a dedicated actor-owned state object (structured)
- Put retrieval and summarization in dedicated Memory actors
- Let the loop actor request "context packs" from Memory actors each step (bounded by a prompt budget)
- Memory actor can publish summaries to JetStream for durability
- ModelProvider may be used to produce summaries or to score salience
- Reasoning-graph tooling can be implemented as an Evaluator actor that transforms CoT traces into graph structures [R2-27, R2-13, R2-14, R2-21]

### Expected Impact vs Turn-Counter Loops

Better context management reduces both failure rate and cost: it mitigates drift and hallucination from irrelevant prompt mass, improves long-horizon coherence (ReCAP), and allows scaling to long-context tasks via decomposition or retrieval rather than raw transcript growth.

---

## 9. Cognitive Architecture and Classical Planning Patterns

### State of the Art

Classical AI and cognitive architectures offer patterns for long-running goal pursuit and failure recovery that map cleanly to agent loops:

- **BDI (Belief-Desire-Intention):** Separates world state (beliefs), goals (desires), and committed plans (intentions), emphasizing real-time performance and practical control of deliberation [R3-C5, R1-11].
- **HTN (Hierarchical Task Network):** Formalizes decomposition of tasks into subtasks; natural fit for "Planner produces hierarchy; Executor commits to leaf tasks; Critic validates" [R3-C5, R2-37].
- **STRIPS:** Canonical early model-based planner framing world models and operator sequences [R3-C6].
- **PDDL:** Standardized representations for planning domains/problems, enabling planners to be benchmarked and interchanged [R3-C6].
- **Behavior Trees:** Widely used in robotics/game AI for modular, reactive control, providing predictable control flow under uncertainty [R3-C6].
- **Reactive plan execution and repair:** Execution monitoring and runtime repair when plans fail in uncertain domains [R3-C6].
- **ACT-R and SOAR:** Provide hierarchical planning and planning+execution monitoring patterns applicable to LLM agents [R2].

### The LLM-Modulo Position

A key 2024-2025 planning-oriented critique is that autoregressive LLMs are unreliable as standalone planners and self-verifiers; the "LLM-modulo" view argues for tighter coupling between LLMs and external verifiers/planners instead of trusting pure prompting or self-verification [R3-C19]. This framing aligns tightly with introducing deterministic verifiers/tools as supervised, sandboxed actors that gate progress: LLM proposes; external system verifies constraints (PDDL planner, type checker, test runner); feedback drives revision.

### Hybrid Symbolic-LLM Systems

A novel neuro-symbolic task planner decomposes complex tasks into subgoals using LLM and carries out task planning for each subgoal using either symbolic or MCTS-based LLM planners, depending on subgoal complexity [R1-10]. Mister Smith can use LLMs for flexible generation and symbolic planners for rigid constraint enforcement and recovery.

### Key Implementable Transfers to LLM Agent Loops

- **Explicit belief state** updated by tool observations (BDI-style "belief revision" light), rather than letting beliefs live only in transcript text [R3-C5, R3-C14].
- **Intention management / commitment:** Commit to a subplan and execute it, but define explicit triggers for reconsideration (failure, low confidence, new constraints). Resembles ReCAP's "commit to head item, refine remainder" [R3-C1, R3-C9].
- **Hierarchical decomposition** (HTN): Treat planning as building a tree of tasks, enabling local replanning rather than global rewrite [R3-C5, R3-C9].
- **Verifier-in-the-loop** ("LLM-modulo"): LLM proposes; external system verifies constraints [R3-C19, R3-C6].
- **Behavior Tree execution:** Represent the Executor's control policy as a behavior tree, with deterministic rules for fallback/retry/skip/human escalation [R3-C6].

### Applicability to Rust Actors + Supervision

These architectures are structurally compatible with actor systems because they emphasize explicit state, modular control nodes, and recoverable execution. BTs and HTNs are effectively graphs of control nodes, which can map to:

- A single agent actor running an explicit state machine, or
- A tree of actors mirroring the plan hierarchy (each node supervised)

Implement BDI/HTN roles as typed actor roles: BeliefStore (Memory actor), Planner (decomposition and plan issuance), IntentionManager (active plan tracking), Executor (tool invocation). Supervisors enforce plan contracts, backtracking policies, and can statically verify plan safety pre-execution [R2-37, R2-32].

### Expected Impact vs Turn-Counter Loops

Cognitive/classical planning patterns mainly increase **predictability** and **testability**: control flow becomes explicit (BT/HTN) rather than emergent in transcripts; failures trigger defined repair/escalation paths; and external verifiers reduce the need to trust "self-verification" by the same stochastic model.

---

## 10. Actor-Model Integration and OTP Supervision

### Mapping the Agentic Loop to `gen_statem`

**High-confidence finding (all 3 reports converge):** The agent loop should be modeled as an explicit state machine (gen_statem-like). Erlang's `gen_statem` provides a generic state machine behaviour that since Erlang/OTP 20.0 replaces its predecessor `gen_fsm` [R1-12]. It separates the concept of the FSM's "State" (typically an atom representing a mode of operation) from its "Data" (an arbitrary Erlang term holding process-specific information) [R1-13]. The engine receives events and calls callback functions to compute new state + actions -- an almost direct analog to an agent loop state machine [R3-C17, R3-C4].

```rust
// Conceptual Mister Smith Agent State Machine
enum AgentState {
    Thinking,
    WaitingForTool(ToolCallId),
    Evaluating(ThoughtNode),
    AwaitingApproval(RunStateSnapshot),
    Degraded(FallbackStrategy),
}
```

Each agent (Planner/Critic/Executor) becomes a long-running actor whose mailbox receives discrete events: `BeginTask`, `ModelResponse`, `ToolResult`, `ApprovalDecision`, `TimerExpired`, `BudgetUpdate`, `BranchScore`, `Backtrack`, etc. The callback returns next state and side effects [R3-C17, R3-C4].

This fusion of a formal FSM for control flow with an Erlang process for data management effectively creates a Turing-complete actor, giving developers the structural benefits of an FSM without its computational limitations [R1-13].

### Supervision Trees for Hallucination and Crash Recovery

A basic concept in Erlang/OTP is the supervision tree, a hierarchical arrangement of code into supervisors and workers, which makes it possible to design and program fault-tolerant software [R1-14]. All three reports converge on standard OTP restart strategies:

- **OneForOne:** If a child process terminates, only that process is restarted [R1-15].
- **OneForAll:** If a child process terminates, all other child processes are terminated, and then all child processes, including the terminated one, are restarted [R1-15].
- **RestForOne:** If a child process terminates, the rest of the child processes (in start order after the terminated one) are terminated and then all are restarted [R1-15].

Akka-style supervision semantics (resume/restart/stop/escalate) formalize the same conceptual space of recovery actions [R3-C17, R3-C4].

**Semantic restarts (novel contribution):** Beyond only restarting crashed actors, extend supervision policy to include semantic restarts: restart the agent loop with different inference settings or different loop policy when stuckness signals fire (timeouts, repeated tool calls, low critic scores). If an agent enters an infinite reasoning loop, the supervisor can detect the timeout, terminate the actor, and restart it with a higher temperature or a fallback prompt [R1, R3-C17].

### Actor Decomposition

**High-confidence finding (all 3 reports converge on this decomposition):**

| Actor | Responsibility |
|:---|:---|
| **Orchestrator** | Top-level gen_server; receives task request and spawns a Session actor under supervision |
| **Session** | Owns per-task metadata (goals, agent roster, budgets); spawns role actors |
| **Planner** | Produces candidate plans / partial thoughts; emits Candidate messages to SearchCoordinator |
| **SearchCoordinator** | Runs chosen search strategy (beam/MCTS/ToT); enqueues NodeExpansion messages; persists nodes to JetStream; requests Evaluator/Critic scores |
| **Evaluator / Critic** | Scores nodes; requests model-based critique if needed; returns Score messages; persists critiques to Reflection stream for episodic memory |
| **Executor** | Receives approved Action messages; requests BudgetReservation; dispatches ToolCall messages to ToolWorker actors via NATS subjects |
| **ToolWorker** | Subscribes to tool-specific NATS subjects; executes tool logic in sandboxed processes; publishes ToolResult messages back to JetStream or direct reply subjects |
| **Memory** | Maintains summaries, vector indices, retrieval for RAG; persists summaries/checkpoints to JetStream |
| **Budget / Ledger** | Global/session/turn budgets with atomic Reserve/Commit/Refund semantics; enforces hard/soft limits; emits BudgetLow or BudgetExceeded events to Supervisor |
| **Router** | Forwards requests to appropriate model via ModelProvider trait (cheap model on edge vs heavyweight cloud model); implements model cascading |
| **Supervisor** | Enforces restart/backoff policies; triggers Backtrack messages to SearchCoordinator; escalates to HumanApproval actor when needed |

### Message Representation

All three reports converge on representing loop constructs as typed messages:

- **Turn:** `TurnTick(session_id, turn_number)` message to SessionActor, which records token consumption and enforces per-turn policies.
- **Tool call:** `ToolCall(tool_id, args, authorized_by)` message dispatched by Executor to ToolRouter, published to NATS subject for ToolWorkers. ToolResult published back to JetStream with deterministic identifier for idempotence.
- **Checkpoint:** `Checkpoint(node_id, serialized_context, metadata)` persisted to JetStream; `CheckpointAck(message_seq)` confirms durability.
- **Backtrack:** `Backtrack(to_checkpoint_id)` message to SearchCoordinator; SearchCoordinator rehydrates checkpoint and spawns alternate expansions as new nodes. Supervisor may escalate or switch search algorithms.

### Distributed Coordination via NATS

Use NATS subjects for request/reply tool calls and JetStream for durable traces and checkpoints; workers subscribe to tool subjects and publish results to result subjects or JetStream streams [R2-37, R2-36, R2-38]. Parallel ToolWorker actors can subscribe to tool subjects and run concurrently for independent calls; Executor must mark tool calls as independent before dispatch to allow parallelism and must manage semaphores for shared resource tools.

### ModelProvider Integration

ModelProvider must support: synchronous text generation/score APIs, streaming partial outputs, and hooks for adding control tokens or budget hints when available. Planner uses ModelProvider.generate for thought and plan generation; Critic uses ModelProvider.score/critique calls; Evaluator may use ModelProvider to compute structural/confidence features or accept an external confidence service. Model cascading is implemented by the Router actor invoking different ModelProvider instances based on Budget actor guidance [R2-61, R2-20, R2-25].

**Migration consideration:** If a ModelProvider supports streaming and partial observations, wire them into the Executor/Evaluator so observations can be consumed incrementally. If hidden-state access is unavailable, Structural Confidence methods that need hidden states may be unavailable or must be simulated via surrogate lightweight classifiers. Control-token budget techniques (BudgetThinker) require the provider to accept injected tokens or a protocol to convey remaining budget [R2-61, R2-20, R2-55].

---

## 11. Production Safety and Governance

### Wasmtime Sandboxing for Untrusted Execution

Executing LLM-generated code directly on the host is a critical vulnerability. When an agent invokes a tool -- whether it is reading a file, making an HTTP request, or executing generated code -- that tool runs inside an isolated WASM instance [R1-3]. The instance has no access to the host filesystem, no network capabilities, and no shared memory with other tools or the host process [R1-3]. Access to any resource must be explicitly granted through a capability system defined at deployment time [R1-3].

### Approval Gates and Human-in-the-Loop

**High-confidence finding (all 3 reports converge):** For high-risk actions, Mister Smith must separate "intent" from "execution." When using MCP tools, always enable tool approvals so end users can review and confirm every operation [R1-21]. The agent emits an intention to the JetStream log, which is picked up by a Human-in-the-Loop (HITL) actor. The execution actor remains suspended until the HITL actor publishes an approval message.

OpenAI Agents SDK's "interruptions + resume from RunState" model is a concrete example of designing around durable pause/resume at tool boundaries [R3-C13, R3-C15, R3-C18]. Mister Smith can generalize this from "approval interrupts" to "search forks" and "rollback repairs."

### Policy-as-Code Enforcement

To meet enterprise compliance, Mister Smith must decouple authorization logic from the agent. The Open Policy Agent (OPA) is an open-source, general-purpose policy engine [R1-29]. It uses a high-level declarative language called Rego to draft policies and rules [R1-29]. Every tool execution intent emitted by an agent must be validated by an OPA middleware actor before reaching the executor.

### PII Redaction and Secrets Management

Data flow through the agent pipeline must be tracked at the type level. PII fields are marked with Rust's type system, and any attempt to log, serialize, or transmit PII without explicit redaction is caught at compile time [R1-3]. Tools like Microsoft Presidio can be integrated into the NATS pipeline to detect and mask PII before it is sent to external LLM providers [R1-30].

---

## 12. Synthesis: The Mister Smith Architecture Blueprint

### High-Level Design Goals (All 3 Reports Converge)

- Make planning, evaluation, and execution explicit and modular (Planner, SearchCoordinator, Critic, Executor, Memory, Budget, ToolWorkers).
- Use durable JetStream streams to store checkpoints, search nodes, traces, and token usage for recoverability and auditability.
- Enforce budgets and authorization in runtime (Supervisor/Budget actors), not in unchecked model proposals.
- Support multiple search strategies (beam, MCTS/LATS, ToT, GoT) selectable per task and switchable by Supervisor on failures.
- Provide safe parallelism for independent tool calls and sequential execution for dependent operations; use router actors for multi-domain task decomposition.
- Integrate Critic/Constitutional checks as a separate actor pipeline with episodic reflections persisted for Reflexion-style improvement.

### The Two-Level Loop Architecture

The most "architecturally superior" loop design for Mister Smith -- based on convergent evidence across modern agent research and production SDK ergonomics -- is:

**Inner loop (per trajectory):** ReAct-style reason/act/observe steps with strict tool schemas, approval gates, and context packs.

**Outer loop (supervisory controller):** Branch-and-bound / MCTS-lite search that decides when to (a) continue the current trajectory, (b) fork alternatives, (c) backtrack to a checkpoint, or (d) terminate with best-effort output.

**Critic as value function + safety judge:** Continuously score partial trajectories, not just final outputs, borrowing directly from LATS/SWE-Search's architecture.

**Durable boundaries everywhere side effects happen:** Checkpoint before side-effecting tools; require compensators for irreversible actions; implement pause/resume exactly like human-in-the-loop run state, generalized to "search forks" and "rollback repairs."

**Budget-aware strategy selection:** Run cheap linear loops first; escalate to search/hierarchy only when critic signals low confidence or repeated failures, consistent with the need to control computational cost and the EnCompass separation of workflow logic vs inference-time strategy.

**Context as a managed resource:** Assemble bounded context packs each step using summaries + retrieval + structured state; adopt hierarchical context reinsertion on backtracking (ReCAP) to prevent drift.

### The Lifecycle of a Mister Smith Agent

1. **Ingestion:** A user prompt arrives via NATS JetStream.
2. **Session Creation:** Orchestrator spawns a Session actor with Budget, Memory, Planner, SearchCoordinator, Executor, and Critic actors under supervision.
3. **Planning (cheap first):** Planner produces a ReAct-style reasoning trace. If task is simple, proceed directly to execution.
4. **Reasoning Escalation (if needed):** On low confidence or failure, SearchCoordinator escalates to MCTS/LATS, spawning child actors to explore parallel reasoning branches via Monte Carlo Tree Search.
5. **Intent Emission:** Planner/SearchCoordinator emits an immutable "tool execution intent" to the JetStream event log.
6. **Governance Gate:** OPA middleware actor intercepts the intent, checking budgets via the Ledger Actor, enforcing safety policies, and routing to Human-in-the-Loop actor for high-risk actions.
7. **Budget Reservation:** Executor requests BudgetReservation from Budget actor. Budget actor responds OK or Deny. If Deny, Supervisor triggers graceful degradation.
8. **Execution:** Executor dispatches ToolCall messages to ToolWorker actors via NATS subjects. ToolWorkers execute inside secure Wasmtime/WASI sandboxes. ToolResult published to JetStream with deterministic identifier for idempotence.
9. **Evaluation:** Critic actor evaluates the result as a continuous value function. If semantic failure detected (repetitive tool calls, low confidence, hallucination), the Critic signals the Supervisor.
10. **Supervisor Decision:** On failure signal, Supervisor can: (a) trigger backtrack by replaying JetStream log to prior checkpoint, (b) switch search strategy, (c) trip semantic circuit breaker to DEGRADED mode, or (d) escalate to human.
11. **Context Compression:** Memory actor asynchronously compresses the context window (Focus agent pattern), stores episodic memories in JetStream KV, and persists reflections to Reflection stream for Reflexion-style improvement.
12. **Result Delivery:** Best trajectory's final output returned to client with full audit trail persisted in JetStream.

### Example Message Flow (Pseudocode)

```
1. Client -> Orchestrator: NewTask(goal, constraints)
2. Orchestrator -> Session (spawn): create SessionActor(goal).
   SessionActor initializes Budget, Memory, Planner, SearchCoordinator, Executor, Critic.
3. Planner -> SearchCoordinator: CandidateRoot(node).
   SearchCoordinator persists node to JetStream (NodeStream). [checkpoint]
4. SearchCoordinator -> Evaluator: Evaluate(node_id).
   Evaluator calls ModelProvider.score(...) and responds Score(node_id, score).
   Critic optionally produces Critique(node_id, issues).
   Critique persisted to Reflection stream.
5. SearchCoordinator: expand/pick nodes per search strategy (beam/MCTS).
   If node selected for execution: send PlanApproved(node_id) to Executor.
6. Executor -> Budget: Reserve(session_id, tokens_estimate).
   Budget responds OK or Deny.
   If OK: Executor -> ToolRouter: Dispatch Action (ToolCall spec).
   ToolRouter publishes to NATS subject e.g., tool.weather.request.
   ToolWorker(s) pick up and execute.
   ToolResult -> JetStream result stream and Observation -> Executor.
   Executor publishes ObservationReceived to SearchCoordinator.
7. SearchCoordinator: append observation to node context, persist updated node.
   Continue search or finalize result.
   On low confidence (Evaluator/StructuralConfidence):
   Supervisor may send Backtrack(to_node_id) to explore alternative branches.
8. If human approval required: Supervisor emits HumanApprovalRequest with snapshot;
   human responds via Approval/Rejection, which Supervisor enforces.
```

### Supervisor-Managed Policies

- **Budget enforcement:** Budget actor implements Reserve/Commit/Refund semantics; expensive operations must acquire reservations; BudgetLow triggers Planner and SearchCoordinator to switch to cheaper strategies. Observability via Traceloop/Portkey-like telemetry records usage for alerts.
- **Stuck detection:** Evaluator/Critic signals low structural confidence or repeated low-value expansions. Supervisor then: (a) triggers backtrack to previous checkpoint, (b) switches to Critic+HumanApproval, or (c) falls back to summary/abort path.
- **Graceful degradation:** On BudgetExceeded, system returns best-effort summary or escalates to human with concise explanation and latest checkpoint; on tool failure, Supervisor retries per policy or routes to alternative tool workers.

---

## 13. Evaluation Rubric for Candidate Architectures

| Dimension | Metric | Measurement Method |
|:---|:---|:---|
| **Latency** | End-to-end wall time per task (ms/s) | Compare parallel tool calls vs sequential |
| **Cost** | Tokens and model cost per task (USD/token or abstract units) | Per-session telemetry via observability hooks |
| **Success rate** | Task-completion correctness (%) | Deterministic planning tasks + open-ended tasks |
| **Fault-recovery rate** | Fraction of failures recovered via backtracking/restart without human intervention (%) | Inject tool failures, model errors, timeouts |
| **Developer complexity** | Lines/components, cognitive load, test coverage required | Estimate per architecture variant |
| **Operational safety** | Unsafe tool invocations prevented, approvals invoked, budget override incidents | Audit log analysis |

---

## 14. Prioritized Prototypes and Experiments

### Architecture Prototypes (Recommended Priority Order)

**1. Planner + LATS-style MCTS SearchCoordinator (core).**
Durable node store in JetStream; search coordinator actor implementing MCTS; Evaluator actor for node scoring. Test tasks: HumanEval-like programming tasks (deterministic) and multi-step WebShop-like browse-and-purchase flows (multi-step, tool-dependent). Failure modes: token explosion, search noise, supervisor backtrack recovery. Evidence: LATS reported strong gains in benchmarks [R1-1, R1-9, R2-29, R2-30].

**2. ReAct + Critic + Reflexion hybrid (writer/critic + episodic reflection).**
Planner produces ReAct-style thought/action traces; Critic actor evaluates and appends reflections to episodic memory; Reflexion improves subsequent trials. Test tasks: coding generation with repeated trials and postmortem improvements; measure pass@1 gains. Evidence: Reflexion improved coding benchmarks and stores reflections for episodic improvement [R2-13, R2-15, R2-16].

**3. Two-level loop (cheap-first with escalation).**
Run cheap linear ReAct loops first; escalate to MCTS/hierarchy only when critic signals low confidence or repeated failures. Test tasks: mixed-difficulty task batches. Evidence: Consistent with EnCompass's separation of workflow logic from inference-time strategy and budget-aware strategy selection [R3-C9].

**4. Budget-aware cascading model router.**
Router actor implements model-cascading (edge small models to mid-tier to large LLM) with Budget actor enforcing reservations and BudgetThinker hints where supported. Test tasks: mixed-cost workloads; measure cost savings and quality loss. Evidence: model cascading and BudgetThinker techniques provide cost/quality tradeoffs [R2-61, R2-25, R2-20].

**5. Graph-of-Thoughts evaluator + summary memory.**
Store CoT streams as graphs (reasoning-graph toolkit) to compute graph-level predictors and prune branches; Memory actor applies context compression. Test tasks: multi-step reasoning QA (ScienceQA/AQUA-RAT) where graph structure aids pruning. Evidence: GoT and reasoning-graph tools correlate graph predictors with performance [R2-25, R2-27, R2-28].

**6. Parallel router/subagent tool orchestration (LangChain Router pattern).**
Implement Router actor that routes subrequests to subagent actors or tool workers with safe sandboxing and per-tool authorization. Test tasks: multi-domain information aggregation tasks; measure latency and token usage tradeoffs. Evidence: LangChain shows parallel router and subagent token/latency tradeoffs [R2-32, R2-33, R2-29].

### Recommended Experiments and Failure-Mode Tests

- **Deterministic planning tasks:** Program synthesis, algorithmic puzzles, or WebShop-like scripted workflows to evaluate correctness, backtracking efficiency, and search pruning (LATS/ToT comparators).
- **Open-ended creative tasks:** Document drafting and multi-document summarization to measure Reflexion improvements and the Critic actor's ability to reduce harmful outputs.
- **Failure-mode tests:**
  - (a) Tool worker crash / network partition -- verify Supervisor restart and checkpoint restore
  - (b) Budget exhaustion mid-search -- verify graceful degradation and human escalation
  - (c) Model misalignment or low-confidence outputs -- trigger Critic/HumanApproval flows
  - (d) Token-cost blowup from parallel search -- measure mitigation via Budget actor
  - (e) Agent infinite reasoning loop -- verify semantic circuit breaker trips to DEGRADED
- **Observability tests:** Ensure per-session, per-agent token accounting is recorded and visible via telemetry hooks.

### Expected Engineering Effort Summary

| Component | Effort | Notes |
|:---|:---|:---|
| Core actor scaffolding + ModelProvider adapters + NATS/JetStream plumbing | Medium | Several sprints |
| MCTS/LATS and durable node management + backtracking | Medium-High | Complex correctness requirements |
| Critic/Episodic memory + budget/reservation | Medium | Architecturally straightforward |
| Parallel tool sandboxing and authorization | Medium-High | Depends on tool complexity and side effects |
| Context compression + RAG memory | Medium-High | Vector indices, retrieval evaluation |

### Implementation Priorities for Phase 9

1. Implement the `gen_statem` actor loop with typed Rust enums for state transitions.
2. Wire the actor mailboxes to JetStream durable consumers for event sourcing.
3. Integrate Wasmtime for secure, capability-based tool execution.
4. Deploy the Ledger/Budget Actor for cascading token and cost budget enforcement.
5. Implement the Critic as a continuous value function service, not post-hoc reviewer.
6. Build the two-level loop: cheap ReAct inner loop, MCTS outer loop on escalation.

---

## 15. Evidence Gaps

The following gaps are identified across all three research reports:

1. **No OTP supervision policy catalog for LLM search:** No findings provide an explicit OTP-style supervision policy catalog or canonical restart strategies tailored to LLM search; Actix/Tokio actor features are documented, but detailed OTP-style supervisor mappings for these exact agent patterns are not specified [R2-47, R2-42, R2-43].

2. **ModelProvider trait surface for advanced techniques:** The research does not include concrete ModelProvider trait definitions or provider-specific APIs (e.g., hidden state access, explicit control-token support) for production LLM providers; BudgetThinker and Structural Confidence require model features that may not be universally available [R2-61, R2-20, R2-55].

3. **JetStream snapshotting scheme specifics:** Limited direct evidence tying specific implementation patterns (exact message schemas, serialization formats) to production JetStream deployments; JetStream durability semantics are documented but mapping to particular snapshotting schemes is left to design [R2-38, R2-75].

4. **Head-to-head benchmarks in Rust + NATS + actors:** Detailed empirical benchmarks comparing all candidate hybrid architectures in the same environment (Rust + NATS + actor supervision) are not present; evidence includes individual method results (LATS, Reflexion, ReAct) but not head-to-head trials in a distributed actor runtime [R2-29, R2-30, R2-13, R2-1].

5. **Compensation/saga patterns for LLM tools:** While compensating actions are discussed conceptually, no research provides a taxonomy of LLM tool reversibility or concrete compensation patterns for common tool types (file operations, API calls, database mutations).

6. **EnCompass and IterGen maturity:** Workflow-as-search-space compilation (EnCompass) and structured generation with forward/backward navigation (IterGen) are cutting-edge and lack production validation in actor systems.

---

## 16. References

### Report A References (R1-*)

[R1-1] LATS: Language Agent Tree Search. https://arxiv.org/abs/2310.04406
[R1-2] ESAA: Event Sourcing for Autonomous Agents. https://arxiv.org/pdf/2602.23193
[R1-3] From OpenClaw to Agentor: Building Secure AI Agents in Rust. https://www.xcapit.com/en/blog/from-openclaw-to-agentor-building-secure-ai-agents-in-rust
[R1-4] A Detailed Comparison of Top 6 AI Agent Frameworks in 2026. https://www.turing.com/resources/ai-agent-frameworks
[R1-5] A Developer's Guide to Agentic Frameworks in 2026. https://pub.towardsai.net/a-developers-guide-to-agentic-frameworks-in-2026-3f22a492dc3d
[R1-6] tokio-actors crate. https://crates.io/crates/tokio-actors
[R1-7] ReAct Architectures for LLM Agents. https://www.emergentmind.com/topics/reason-act-reflect-react-architectures
[R1-8] Tree of Thoughts: Deliberate Problem Solving with Large Language Models. https://arxiv.org/abs/2305.10601
[R1-9] Language Agent Tree Search Unifies Reasoning Acting and Planning in Language Models. https://openreview.net/forum?id=6LNTSrJjBe
[R1-10] Fast and Accurate Task Planning using Neuro-Symbolic Language Models and Multi-level Goal Decomposition. https://arxiv.org/abs/2409.19250
[R1-11] BDI Agent Architectures: A Survey. https://www.ijcai.org/proceedings/2020/0684.pdf
[R1-12] Erlang gen_statem documentation. https://erlang.org/doc/man/gen_statem.html
[R1-13] The Absolute Guide to State Machines in Erlang. https://medium.com/@matheuscamarques/the-absolute-guide-to-state-machines-in-erlang-implementation-complexity-and-testing-1b7ae3a3f5dd
[R1-14] OTP Design Principles. https://github.com/erlang/otp/blob/master/system/doc/design_principles/design_principles.md
[R1-15] Erlang Supervisor Behaviour. https://www.erlang.org/docs/24/design_principles/sup_princ
[R1-16] NATS JetStream Documentation. https://docs.nats.io/jetstream
[R1-17] JetStream Model Deep Dive. https://docs.nats.io/using-nats/developer/develop_jetstream/model_deep_dive
[R1-18] NATS JetStream Playbook: Exactly-Once. https://medium.com/@hadiyolworld007/nats-jetstream-playbook-exactly-once-minus-the-bloat-02fd9d5a051c
[R1-19] AI SDK 6 (Vercel). https://vercel.com/blog/ai-sdk-6
[R1-20] Node: Call Tools in Multiple Steps. https://ai-sdk.dev/cookbook/node/call-tools-multiple-steps
[R1-21] Safety in building agents (OpenAI). https://developers.openai.com/api/docs/guides/agent-builder-safety/
[R1-22] Active Context Compression: Autonomous Memory Management in LLM Agents. https://arxiv.org/abs/2601.07190
[R1-23] Semantic Caching for LLM Apps: Reduce Costs by 40-80% and Speed up by 250x. https://www.percona.com/blog/semantic-caching-for-llm-apps-reduce-costs-by-40-80-and-speed-up-by-250x/
[R1-24] GPT Semantic Cache: Reducing LLM Costs and Latency via Semantic Embedding Caching. https://arxiv.org/html/2411.05276v2
[R1-25] Budget-Constrained Agentic Large Language Models. https://arxiv.org/pdf/2602.11541
[R1-26] BAMAS: Structuring Budget-Aware Multi-Agent Systems. https://arxiv.org/html/2511.21572v1
[R1-27] Hedging: A 'Simple' Tactic to Tame Tail Latency in Distributed Systems. https://blog.alexoglou.com/posts/hedging/
[R1-28] Resilience Circuit Breakers for Agentic AI. https://medium.com/@michael.hannecke/resilience-circuit-breakers-for-agentic-ai-cc7075101486
[R1-29] Implementing a PDP by using OPA (AWS Prescriptive Guidance). https://docs.aws.amazon.com/prescriptive-guidance/latest/saas-multitenant-api-access-authorization/opa.html
[R1-30] Presidio by Microsoft: A Practical Guide to Detecting and Masking PII at Scale. https://medium.com/@nkbvikram/presidio-by-microsoft-a-practical-guide-to-detecting-and-masking-pii-at-scale-c3b39ce4f52c

### Report B References (R2-*)

[R2-1] ReAct Reproduction. https://astrocvijo.github.io/react_reproduction/react_reproduction.pdf
[R2-2] ReAct Framework: Reasoning + Acting. https://apxml.com/courses/agentic-llm-memory-architectures/chapter-2-advanced-agent-architectures-reasoning/react-framework-reasoning-acting
[R2-3] ReAct Prompting Guide. https://www.promptingguide.ai/techniques/react
[R2-4] ReAct (OpenReview). https://openreview.net/forum?id=vAElhFcKW6
[R2-5] Reflexion. https://arxiv.org/abs/2303.11366
[R2-6] Tree of Thoughts. https://arxiv.org/pdf/2305.10601
[R2-7] NAACL 2024 Findings. https://aclanthology.org/2024.findings-naacl.78.pdf
[R2-8] JHU NAACL 2024. https://www.cs.jhu.edu/~kevinduh/t/naacl24/final_pdf/paper690.pdf
[R2-9] EMNLP 2025. https://aclanthology.org/2025.emnlp-main.896.pdf
[R2-10] LATS. https://arxiv.org/abs/2310.04406
[R2-11] LATS (OpenReview). https://openreview.net/forum?id=6LNTSrJjBe
[R2-12] OpenAI Agents SDK Guide. https://developers.openai.com/api/docs/guides/agents-sdk/
[R2-13] Reflexion. https://arxiv.org/abs/2303.11366
[R2-14] Chain of Thought Prompting. https://www.promptingguide.ai/techniques/cot
[R2-17] Constitutional AI. https://arxiv.org/pdf/2406.02746
[R2-18] CoT Variants. https://arxiv.org/html/2302.12246v5
[R2-19] Dynamic Least-to-Most. https://openreview.net/pdf?id=_VjQlMeSB_J
[R2-20] BudgetThinker. https://arxiv.org/html/2508.17196v1
[R2-21] RATT. https://arxiv.org/pdf/2406.02746
[R2-22] OpenAI Agents Python. https://openai.github.io/openai-agents-python/
[R2-25] LangChain Docs. https://docs.langchain.com/oss/python/langchain/overview
[R2-27] Choosing the Right Multi-Agent Architecture (LangChain Blog). https://blog.langchain.com/choosing-the-right-multi-agent-architecture/
[R2-28] Orchestrating LangChain Agents with Orkes Conductor. https://orkes.io/blog/how-to-orchestrate-langchain-agents-for-production-with-orkes-conductor/
[R2-29] AI SDK Tools and Tool Calling. https://ai-sdk.dev/docs/ai-sdk-core/tools-and-tool-calling
[R2-30] AI SDK: Call Tools in Multiple Steps. https://ai-sdk.dev/cookbook/next/call-tools-multiple-steps
[R2-32] Parallel Tool Calling (CodeAnt). https://www.codeant.ai/blogs/parallel-tool-calling
[R2-33] Multi-AI Agent Systems with CrewAI. https://github.com/ksm26/Multi-AI-Agent-Systems-with-crewAI
[R2-34] AutoGen. https://www.emergentmind.com/topics/autogen
[R2-36] AutoGen Migration Guide (Microsoft). https://learn.microsoft.com/en-us/agent-framework/migration-guide/from-autogen/
[R2-37] NATS.io. https://nats.io/
[R2-38] NATS JetStream Concepts. https://docs.nats.io/nats-concepts/jetstream
[R2-39] NATS Rust Client. https://github.com/nats-io/nats.rs
[R2-42] OpenAI Response API and Agents SDK Guide. https://github.com/Dicklesworthstone/guide_to_openai_response_api_and_agents_sdk
[R2-43] LLMs AI Orchestration Toolkits Comparison. https://www.cudocompute.com/blog/llms-ai-orchestration-toolkits-comparison
[R2-44] Human-in-the-Loop and LLM-as-a-Judge. https://kili-technology.com/blog/human-in-the-loop-human-on-the-loop-and-llm-as-a-judge-for-validating-ai-outputs
[R2-45] AI Agents and Security. https://zinatullin.com/2026/01/13/ai-agents-and-security/
[R2-47] Checkpoint/Restore. https://eunomia.dev/zh/blog/posts/check-restore/
[R2-50] Tracking LLM Token Usage (Portkey). https://portkey.ai/blog/tracking-llm-token-usage-across-providers-teams-and-workloads
[R2-51] LLM Optimization Techniques (Mirantis). https://www.mirantis.com/blog/llm-optimization-techniques/
[R2-53] Query-Aware Budget-Tier Routing. https://www.emergentmind.com/topics/query-aware-budget-tier-routing
[R2-55] Structural Confidence. https://arxiv.org/html/2508.17627v1
[R2-56] Overthinking Detection. https://arxiv.org/html/2601.11038v1
[R2-61] tokio-actors Discussion (Reddit). https://www.reddit.com/r/rust/comments/1p3iqmv/tokioactors_010_productionready_actors_built_for/
[R2-75] Traceloop: Token Usage and Cost per User. https://www.traceloop.com/blog/from-bills-to-budgets-how-to-track-llm-token-usage-and-cost-per-user

### Report C References (R3-C*)

[R3-C0] ReAct, Reflexion, ToT, GoT, LATS citations in Report C (inline turn-based citations referencing the same foundational papers as R1/R2).
[R3-C1] ReCAP: Hierarchical recursion with shared context for long-horizon coherence (inline citations: turn1search0, turn1search12).
[R3-C2] Anthropic programmatic tool calling and server tools documentation (inline citations: turn2search7, turn2search3, turn3search7).
[R3-C3] CrewAI max iterations behavior; LangChain AgentExecutor early stopping (inline citations: turn3search5, turn3search1, turn3search0).
[R3-C4] OTP gen_statem and supervision (inline citations: turn4search1, turn4search2, turn4search3, turn4search5, turn4search7, turn4search11).
[R3-C5] BDI and HTN planning (inline citations: turn5search0, turn5search3, turn5search7).
[R3-C6] Behavior Trees, STRIPS, PDDL, reactive planning (inline citations: turn6search0, turn6search1, turn6search2, turn6search6, turn6search9, turn6search11, turn6search12, turn6search19).
[R3-C7] Self-Refine, Constitutional AI, LLM-as-judge, confidence estimation (inline citations: turn7search0-19).
[R3-C8] RAG, MemGPT, Anthropic context engineering (inline citations: turn8search0-14).
[R3-C9] SWE-Search, ReAcTree, EnCompass, IterGen (inline citations: turn9search0-14).
[R3-C10] Budget/resource management in LangChain, OpenAI, Anthropic (inline citations: turn10search0-14).
[R3-C11] MCTS, alpha-beta pruning references (inline citations: turn11search1-8).
[R3-C12] Graph of Thoughts, Chain-of-Agents (inline citations: turn12search3-13).
[R3-C13] Approval gates: Vercel needsApproval, OpenAI interruptions (inline citations: turn13search1-16).
[R3-C14] Parallel tool calling patterns (inline citations: turn14search1-7).
[R3-C15] Durable pause/resume, OpenAI RunState, Vercel parallelToolCalls (inline citations: turn15search0-18).
[R3-C16] OpenAI Agents SDK runner loop, Google ADK LoopAgent (inline citations: turn16search0-16).
[R3-C17] OTP gen_statem behavior, Akka supervision semantics (inline citations: turn17search1-13).
[R3-C18] Vercel AI SDK multi-step, AutoGen termination conditions, Google ADK InvocationContext (inline citations: turn18search2-14).
[R3-C19] LLM-modulo: LLMs as unreliable planners/self-verifiers (inline citations: turn19search0-4).

### Additional Cross-Report References

- Actix actor framework: https://github.com/actix/actix
- OpenAI Realtime Agents: https://github.com/openai/openai-realtime-agents
- LangChain Agent Reference: https://reference.langchain.com/v0.3/python/core/agents.html
- Flow Engineers Toolkit: https://maccelerator.la/en/blog/entrepreneurship/flow-engineers-toolkit-n8n-langchain-ai-agent-architectures/
- CrewAI Tools: https://github.com/crewAIInc/crewAI-tools
- AutoGen Discussion on Actor Model: https://github.com/microsoft/autogen/discussions/6347
- Zalando AI Postmortem Analysis: https://engineering.zalando.com/posts/2025/09/dead-ends-or-data-goldmines-ai-powered-postmortem-analysis.html
- Using LLM as Reverse Engineering Sidekick: https://blog.talosintelligence.com/using-llm-as-a-reverse-engineering-sidekick/
- Harvard Data Science Review: https://hdsr.mitpress.mit.edu/pub/jaqt0vpb
- USENIX ATC 2025: https://www.usenix.org/system/files/atc25-tian.pdf
- EMNLP 2024: https://aclanthology.org/2024.emnlp-main.1112.pdf
- PMC Article on LLM Optimization: https://pmc.ncbi.nlm.nih.gov/articles/PMC12846292/
- RCP Detection: https://arxiv.org/html/2602.00977v1
