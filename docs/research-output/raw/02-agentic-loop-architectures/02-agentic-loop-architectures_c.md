# Agentic Loop Architectures for Multi-Agent Orchestration in Actor-Based Systems

Modern “agent loops” in production SDKs are still largely *linear*: call a model, parse either a final answer or tool calls, execute tools, append results, and repeat until the model returns a final output—or a hard stop triggers. This pattern is explicit in the OpenAI Agents SDK runner loop (final output vs handoff vs tool calls; enforced via `max_turns`). citeturn16search0turn2search0turn16search16 Similar “step-loop” control exists in Vercel’s AI SDK: multi-step tool calling continues until tool calls stop or a stop condition like `stepCountIs(n)` fires (default 20 steps in the agent helpers). citeturn18search10turn16search3turn18search2turn18search14 Google’s ADK exposes a comparable workflow primitive (`LoopAgent`) that repeatedly runs sub-agents until `max_iterations` or a sub-agent “escalates” via an event, preserving shared state via a persistent invocation context. citeturn18search4turn16search1turn18search12

Your objective for Mister Smith is to go beyond these counter-based loops, leveraging actor supervision, durable messaging, and multi-agent roles (Planner/Critic/Executor) to support: (a) search over alternative reasoning/action trajectories, (b) backtracking/rollback, (c) self-evaluation and stuck detection, (d) resource governance across distributed components, and (e) safe pause/resume boundaries for human approvals.

## Reasoning loop architectures

**Current state of the art.**  
Two foundational “loop families” still dominate.

First, **interleaved reasoning + acting**: *ReAct* formalized the idea of alternating “reason” tokens with explicit “act” steps (tool/environment calls), improving performance and interpretability by letting actions feed back into reasoning. citeturn0search0turn0search4 *Reflexion* then added **feedback-conditioned iteration**: rather than weight updates, agents store textual reflections in memory and reuse them on subsequent attempts (a “verbal RL” style loop). citeturn0search1turn0search17turn0search9

Second, **deliberative search over thought trajectories**: *Tree of Thoughts* (ToT) explicitly treats reasoning steps as a search space, exploring multiple candidates, self-evaluating them, and enabling lookahead/backtracking when needed. citeturn0search2turn0search6 *Graph of Thoughts* generalizes this by representing intermediate “thought units” as a dependency graph (not just a tree), enabling richer composition/aggregation patterns. citeturn12search3turn12search11turn1search9

By 2024–2025, the most notable “successor” direction is **search + value estimation + reflection**, where the loop becomes a planner that explores branches and uses learned or model-based scoring to guide exploration. *LATS (Language Agent Tree Search)* is representative: it unifies reasoning/acting/planning by integrating Monte Carlo Tree Search (MCTS) with LM-powered value functions and self-reflections. citeturn0search3turn0search11turn0search15 In software-agent settings, *SWE-Search* similarly injects MCTS plus multi-agent evaluation roles (value estimation + debate) to reduce repetitive ineffective actions and improve repository-level task performance. citeturn9search0turn9search8

In 2025–2026, **hierarchical and recursion-based control flow** becomes increasingly prominent for long-horizon tasks. *ReCAP* frames long-horizon coherence as a hierarchical process with shared context, committing to the head subtask while refining the remainder as new observations arrive, plus bounded “sliding-window” scaling and structured re-injection of higher-level context on backtracking. citeturn1search0turn1search12 *ReAcTree* similarly uses a dynamically built *agent tree* with explicit control-flow nodes coordinating execution; it also distinguishes working vs episodic memory for sharing observations vs retrieving examples. citeturn9search1turn9search13

**Key techniques.**  
The most practically relevant techniques for “beyond linear loops” (with evidence of effectiveness across papers) cluster into five patterns:

1. **Trajectory search with evaluation** (ToT, GoT): generate multiple candidate partial trajectories; score them (self-evaluation or external evaluator) to decide expansion/backtracking. citeturn0search6turn12search3  
2. **Tree-search planning with stochastic branching and value estimates** (LATS, SWE-Search): treat each model decision as a branching point; use value functions and exploration bonuses to allocate more calls to promising branches. citeturn0search3turn9search8  
3. **Hierarchical recursion with shared context** (ReCAP): keep global goals stable while allowing local replanning; explicitly manage what context persists across recursion levels. citeturn1search0turn1search12  
4. **Hierarchical agent-tree execution with control-flow nodes** (ReAcTree): make control flow explicit (sequence, fallback, loops) instead of emergent in a monolithic transcript. citeturn9search1turn9search13  
5. **Workflow-as-search-space compilation** (EnCompass): disentangle (a) workflow logic from (b) inference-time strategy by compiling “points of unreliability” (e.g., LLM calls) into a search space where different search policies can be swapped in. citeturn9search7turn9search11

**Applicability to Rust actors + supervision.**  
Actor systems are unusually well-aligned with these architectures because “candidate trajectories” can be realized as *isolated child processes* (one per branch), and search control can live in a supervisor (or coordinator) actor that allocates budget and terminates/forks branches. This mirrors the design intent in LATS and SWE-Search—explicit exploration governed by an outer control algorithm—while letting Mister Smith implement exploration via message-passing and restart semantics rather than nested call stacks. citeturn0search3turn9search8

Hierarchical designs like ReCAP/ReAcTree map naturally to *hierarchical actor trees*: each subgoal node can be an actor with its own local loop state, while parent actors own coordination/control-flow decisions. citeturn1search0turn9search13

**Implementation complexity.**  
A pragmatic complexity ladder:

- Low: ReAct-style single-path loop with tool calls. citeturn0search0turn14search2  
- Medium: Reflexion / Self-Refine style iterative critique + memory buffers. citeturn0search1turn7search1  
- High: LATS/SWE-Search class (tree search, value functions, branch management, exploration policy). citeturn0search3turn9search0  
- High+: EnCompass-style compilation and interchangeable inference-time strategies, plus efficient state capture across branches. citeturn9search7turn9search11

**Expected impact vs simple turn counters.**  
Search- and hierarchy-based loops primarily improve: (a) **recovery from local failures** (bad early tool call, wrong plan), (b) **reduced repetitive action loops** via explicit evaluation and exploration, and (c) **long-horizon coherence** by preventing “context drift” and enabling structured replanning. These are the explicit motivations and reported benefits in ReCAP, LATS, SWE-Search, and ReAcTree. citeturn1search0turn0search3turn9search8turn9search13

## Multi-turn tool calling patterns

**Current state of the art.**  
Production SDKs converge on a similar baseline: a runtime loop that continues model calls until the model returns final text with no tool calls, or until a termination condition fires.

- OpenAI Agents SDK: loop calls the model; if `final_output`, stop; else run tool calls and continue; or hand off to another agent; `max_turns` bounds the loop. citeturn16search0turn16search16turn2search4  
- Vercel AI SDK: “multi-step calls” are controlled by `stopWhen`; when tool calls appear, the SDK executes tools, appends results, and triggers another step until tool calls cease or `stopWhen` is satisfied; default stop is 20 steps. citeturn16search3turn18search10turn18search2turn16search7  
- Google ADK: `LoopAgent` repeats sub-agents; halts on `max_iterations` or an “escalate” event, sharing a persistent `InvocationContext` across iterations so state can accumulate deterministically across probabilistic model calls. citeturn18search4turn18search12turn16search1  
- Tool calling itself is still described as a multi-step conversation between application and model in OpenAI’s function/tool calling guidance. citeturn14search2

**Key techniques.**  
The most actionable patterns beyond “single tool then answer” are:

- **Multi-step tool chaining**: explicitly allow multiple consecutive tool steps (Vercel `stopWhen` / `stepCountIs`). citeturn16search7turn18search10  
- **Parallel tool calls**: some providers/models can emit multiple tool calls in one generation step; Vercel’s provider options expose `parallelToolCalls`, and OpenAI Agents SDK’s model settings explicitly control parallel tool calls. citeturn14search1turn15search0turn14search7  
- **Runtime tool authorization (approval gates)**: Vercel supports `needsApproval` for tools; the tool may be proposed by the model but not executed until approved. citeturn13search1turn13search7turn13search16 OpenAI Agents SDK similarly supports human-in-the-loop pauses where the run returns “interruptions” and can resume from serialized run state after approval/rejection. citeturn13search3turn15search13turn18search5  
- **Server-managed vs client-managed tool loops**: Anthropic documents “server tools” where their servers handle tool execution in a loop; this is a different operational model than pure client-side function calling. citeturn3search7  
- **Programmatic tool calling**: Anthropic’s “programmatic tool calling” lets the model write orchestration code in a code execution environment, reducing per-tool round-trips and enabling parallel tool execution in-code. citeturn2search7turn2search3

**Safety implications of parallel vs sequential tools.**  
Parallel tool calling improves latency when tools are independent, but introduces *dependency and authorization hazards*: a model may propose tool calls with implicit ordering dependencies (e.g., tool B assumes tool A’s result), so the runtime must either (a) constrain to sequential execution, or (b) topologically schedule based on declared dependencies and reject unsafe mixes. The existence of explicit “call tools in parallel” recipes and explicit parallel tool call toggles underscores that the runtime—not just the model—must manage this decision. citeturn14search4turn15search0turn14search7

Approval gating becomes more complex under parallelism: if any tool requires approval, you need a consistent “pause boundary” that freezes the *entire step* until approvals resolve, otherwise partial execution can create irreversible side effects and inconsistent state. OpenAI Agents SDK’s “interruptions + resume from RunState” model is a concrete example of designing around durable pause/resume at tool boundaries. citeturn13search3turn15search13turn18search5

**Applicability to Rust actors + supervision.**  
Actor systems can treat each tool invocation as a child actor or as a task scheduled onto a supervised worker pool. That makes:

- **Parallel tool fan-out** a first-class pattern: spawn N tool workers, gather results, then send an “observation” message back to the agent loop actor. (This aligns with the explicit guidance that tools can be executed in parallel when built async in ADK, and with SDK-level `parallelToolCalls` toggles.) citeturn14search7turn15search0  
- **Approval gates** cleanly representable as *state transitions*: the loop actor enters `AwaitingApproval` and persists a resumable state snapshot (see OpenAI RunState semantics as a prior art). citeturn15search13turn18search5

Supervision can also enforce tool safety: a supervisor can kill/restart a tool worker on timeout or abnormal exit, consistent with conventional supervision semantics in actor frameworks. citeturn4search2turn17search3

**Implementation complexity.**  
Sequential multi-step tool loops are straightforward (already implemented by existing SDKs). citeturn16search0turn18search14turn18search4 Complexity rises notably when adding:

- parallel tool scheduling + result aggregation, citeturn14search4turn14search7  
- durable pause/resume with serialized run snapshots, citeturn15search13turn18search5  
- programmatic orchestration sandboxes (code execution, security constraints). citeturn2search7turn2search3

**Expected impact vs simple turn-counter loops.**  
A richer tool loop architecture primarily improves: (a) **latency** (parallelism, programmatic orchestration), (b) **safety** (approval gating with durable pause boundaries), and (c) **operational reliability** (timeouts/retries per tool worker rather than failing whole runs). These are explicit motivations in the Vercel loop-control + approval design and Anthropic’s programmatic tool calling rationale (fewer round trips, lower token consumption). citeturn18search10turn13search1turn2search7turn2search3

## Self-evaluation and self-correction

**Current state of the art.**  
Self-correction is increasingly treated as a *first-class loop*, not just prompt seasoning:

- *Reflexion* stores language feedback as episodic memory used to improve later trials (no weight updates). citeturn0search1turn0search17  
- *Self-Refine* formalizes an iterative FEEDBACK → REFINE loop using the same model to critique and improve outputs, repeatedly, until a condition is met. citeturn7search1turn7search9turn7search17  
- *LATS* embeds self-reflections and LM-powered value estimation inside a tree search procedure, upgrading self-evaluation from a single “critique pass” to a branch-scoring component. citeturn0search3turn0search11  
- *SWE-Search* adds explicit evaluator roles (Value Agent and Discriminator Agent for debate), combining qualitative evaluations with numeric value estimation inside MCTS-style exploration. citeturn9search0turn9search8  
- “Constitutional AI” is a broader alignment approach (training-time), but its core idea—apply a rule/principle set to critique and revise outputs—directly informs runtime “constitutional critique” loops used in many agent designs. citeturn7search0turn7search4turn7search16

**Key techniques.**  
For a Planner/Critic/Executor trio, the most transferable techniques are:

- **Inline self-critique loops** (Self-Refine): cheap to integrate, but can be correlated with the original error modes. citeturn7search1turn7search9  
- **Separated Critic role** (multi-agent): reduces correlation by using different prompts/models/temperatures; SWE-Search’s distinct evaluation agents and multi-agent debate is an explicit example. citeturn9search0turn9search8  
- **LLM-as-a-judge** for structured evaluations: surveys show this has become a widespread paradigm, but also emphasize reliability pitfalls and mitigation strategies. citeturn7search2turn7search10  
- **Bias-aware evaluation**: empirical studies show both human and LLM judges exhibit multiple biases and perturbation vulnerabilities, motivating ensemble judges, calibration, and audit tooling. citeturn7search6turn7search2  
- **Confidence estimation + calibration**: surveys compile techniques for confidence estimation and calibration in LLMs; newer work explores “verbalized confidence” and methods to improve calibration. citeturn7search7turn7search3turn7search19

**Detecting “stuck loops” or low-quality iteration.**  
Production frameworks increasingly expose explicit termination conditions beyond turn counts, which can be repurposed as *stuck detection primitives*. AutoGen, for example, documents termination conditions including token-usage and timeout constraints, not just message counts. citeturn18search3turn18search7 ReCAP explicitly motivates hierarchical control by noting that sequential prompting can fall into recurrent failure cycles and context drift on long-horizon tasks. citeturn1search0turn1search12

A strong practice emerging from these sources is to treat “stuckness” as a *multi-signal diagnosis* (repeated tool calls, repeated argument patterns, low judge scores, lack of state progress), then trigger either (a) strategy change or (b) supervised restart/backtrack rather than continuing the same loop.

**Applicability to Rust actors + supervision.**  
Mister Smith’s existing Critic role can be elevated from “post-hoc reviewer” to a **value function service** used continuously: scoring partial trajectories, deciding whether to expand/stop/backtrack, and calibrating confidence levels before returning results. This exactly matches the role that value estimation and reflection play inside LATS and SWE-Search. citeturn0search3turn9search0

In an actor system, “self-evaluation” can be isolated into its own supervised subtree:

- executor actor emits state updates and tool results,  
- critic actor consumes them and produces scores/diagnostics,  
- supervisor actor decides restarts/backtracking/escalation based on the critic’s signals.

This separation reduces correlated failure and allows swapping evaluation models independently (including “judge ensembles”), consistent with the LLM-as-judge literature’s concern about bias and reliability. citeturn7search2turn7search6turn7search7

**Implementation complexity.**  
Implementing a Critic-as-judge loop is medium complexity (model calls + rubric). Making it robust (anti-bias ensembles, calibration, confidence tracking, persistent memory of failure patterns) is higher complexity, but guided by existing surveys and applied methods. citeturn7search2turn7search7turn0search1

**Expected impact vs simple turn-counter loops.**  
The measurable impact is usually: fewer silent failures, fewer repeated ineffective actions, and better “final answer” quality gates—especially when combined with search/backtracking. These are core claims of Reflexion/Self-Refine improvements and the search-with-evaluation framing in LATS/SWE-Search. citeturn0search1turn7search1turn0search3turn9search8

## Backtracking and rollback

**Current state of the art.**  
Backtracking in LLM agents now appears in three increasingly “industrializable” forms:

1. **Reasoning backtracking** (no external side effects): ToT explicitly supports looking ahead and backtracking among thought branches. citeturn0search6turn0search2  
2. **Search-based action backtracking** (limited side effects): LATS uses MCTS with reflections/value functions to explore action sequences and revise decisions based on feedback. citeturn0search3turn0search11 SWE-Search adapts this pattern to software engineering repositories, adding evaluator/discriminator agents to guide exploration and refinement. citeturn9search0turn9search8  
3. **Program/workflow backtracking with explicit nondeterminism**: EnCompass compiles workflows into a search space over “unreliable” operations like LLM calls; IterGen’s structured generation exposes forward/backward navigation during generation under grammatical constraints. citeturn9search7turn9search11turn9search14turn9search6

Separately, robotics has long addressed failure recovery under real-world side effects using explicit control architectures. Behavior Trees (BTs) are a standard approach for structuring reactive, modular task execution; they’re frequently used precisely because pure planning is brittle under uncertainty. citeturn6search2turn6search6 Robotics literature on reactive planning and plan repair further emphasizes time-bounded recovery procedures that “repair” plans at runtime rather than re-planning from scratch. citeturn6search11turn6search19

**Key techniques.**  
The most transferable rollback mechanisms for tool-using agents are:

- **Checkpoint/restore of agent state** at chosen boundaries (before tool calls; before committing plan segments). This is analogous to the durable pause/resume boundary explicitly documented in OpenAI’s run-state approach for approvals, but generalized from “approval interrupts” to “search forks.” citeturn15search13turn18search5  
- **Compensating actions** for tool side effects: rather than undoing time, define explicit rollback tools (“delete created record,” “revert commit”), mirroring transactional saga patterns rather than strict ACID transactions (the robotics “repair at runtime” framing is conceptually aligned). citeturn6search11turn6search19  
- **Treat execution as search**: model decisions are branching points; exploration uses MCTS/UCT-like allocation to promising branches, as done explicitly in LATS and discussed in MCTS references. citeturn0search3turn11search8turn11search1  
- **Pruning heuristics**: alpha–beta pruning is the canonical “don’t explore provably irrelevant branches” technique in adversarial search; while agent tasks are not strictly zero-sum games, the principle transfers: prune branches that violate constraints (budget, safety, consistency) or score below a threshold. citeturn11search2turn11search3  
- **Hierarchical rollback**: in ReCAP, context re-injection on backtracking preserves cross-level continuity while still allowing local replanning; in ReAcTree, control-flow nodes can reroute execution to alternative subtrees. citeturn1search0turn9search13

image_group{"layout":"carousel","aspect_ratio":"16:9","query":["Monte Carlo Tree Search UCT diagram","behavior tree selector sequence fallback diagram","software agent tree search diagram LATS"],"num_per_query":1}

**Applicability to Rust actors + supervision.**  
This is one of the strongest fits for Mister Smith specifically:

- **Forking** = spawn new child actors with copied checkpoints; each child explores an alternative plan/tool sequence.  
- **Backtracking** = terminate a subtree and resume from a prior checkpoint in a different branch (or restart with a different strategy).  
- **Rollback** = supervisors can enforce that side-effecting tool calls must be paired with compensators; if a child fails after a side effect, the supervisor triggers compensation before retrying.

This operationalizes the “agent as search process” architecture that LATS and SWE-Search demonstrate in research settings. citeturn0search3turn9search8

**Implementation complexity.**  
Implementing *pure reasoning backtracking* (tree-of-thought over internal messages) is moderate. Adding *tool rollback* is high complexity because you must classify tools by reversibility and define compensators or idempotency rules. The payoff is that this aligns with real-world failure recovery patterns in robotics and classical execution monitoring (repair rather than restart-everything). citeturn6search11turn6search2

**Expected impact vs simple turn-counter loops.**  
Backtracking/search architectures reduce “single-trajectory brittleness”: one bad tool call or wrong early assumption doesn’t doom the entire run. The explicit goal of ToT, LATS, and SWE-Search is to improve robustness by exploring alternatives guided by evaluation signals. citeturn0search6turn0search3turn9search8

## Budget and resource management

**Current state of the art.**  
Production systems increasingly expose *multiple* budget knobs, but they are not always unified into a single policy layer:

- OpenAI Agents SDK enforces a turn budget via `max_turns` and documents that exceeding it raises an exception; it also tracks token usage for runs so developers can monitor/enforce limits. citeturn16search0turn10search14turn2search4  
- Vercel AI SDK defaults agents to 20 steps and lets developers customize stop conditions (`stopWhen`). citeturn18search10turn18search14  
- LangChain’s AgentExecutor exposes both step and wall-clock budgets (`max_iterations`, `max_execution_time`) and supports explicit early-stopping modes. citeturn10search0turn10search12turn3search0  
- AutoGen documents termination conditions including token usage limits and timeouts, not only message counts. citeturn18search3turn18search7  
- Providers impose external constraints: Anthropic documents rate limits in token/minute dimensions and even discusses allocating limits across workspaces to prevent overuse. citeturn10search3 OpenAI documents managing billing limits/usage tiers as part of production best practices. citeturn10search6

**Key techniques.**  
To exceed “max turns” sophistication, the most robust patterns are:

- **Multi-dimensional budgets**: enforce *time*, *token*, and *cost* budgets simultaneously (AutoGen’s token/time termination types provide a concrete precedent). citeturn18search3turn10search3  
- **Hierarchical or cascading budgets**: allocate constraints across a team (root coordinator) → per-agent (Planner/Critic/Executor) → per-branch (search tree node). This is conceptually aligned with the need to manage resources under rate limits and quota tiers (provider docs show multi-axis constraints; the missing piece is orchestration-layer propagation). citeturn10search3turn10search6  
- **“Graceful degradation” behavior near budget exhaustion**: some frameworks explicitly describe “best answer when max iterations hit,” which can be interpreted as a policy knob for Mister Smith (best-effort summary vs hard failure). CrewAI documentation describes that once near the maximum iterations, the agent “will try its best to give a good answer.” citeturn3search5turn3search1  
- **Budget-aware strategy selection**: search and reflection strategies should be chosen based on remaining budget (e.g., tree search only after repeated failure signals; otherwise run a cheap linear loop first). This is consistent with the separation of workflow logic from inference-time strategy advocated by EnCompass. citeturn9search7turn9search11

**Applicability to Rust actors + supervision.**  
Budgets become enforceable *system invariants* when implemented as:

- a supervisor-owned “budget ledger” (tokens/time/cost),  
- per-agent allowances granted as messages (“budget grants”),  
- hard kill switches (timeouts; max tokens) enforced by supervisors.

This aligns with the existence of token/time termination primitives in AutoGen and wall-clock + step enforcement in LangChain. citeturn18search3turn10search12turn10search0

**Implementation complexity.**  
Medium if limited to counters/timeouts. High if implementing true cascading budgets across distributed branches and doing adaptive strategy selection (because it requires accurate per-provider usage accounting and consistent propagation). OpenAI’s run usage tracking and provider rate-limit documentation show the data you *must* wire into such a system. citeturn10search14turn10search3turn10search6

**Expected impact vs simple turn-counter loops.**  
Sophisticated budgets primarily improve **production safety** (prevent runaway spend/latency), **SLO predictability**, and **fairness across concurrent agent runs** under external rate limits and internal capacity. This follows directly from providers documenting multi-axis rate limits and from frameworks exposing token/time termination conditions. citeturn10search3turn18search3turn10search6

## Context window management

**Current state of the art.**  
Context overflow is now treated as a first-order engineering problem, with both SDK-level and research-level approaches:

- OpenAI Agents SDK offers “Sessions” as a persistent memory layer that maintains conversation history across runs. citeturn15search6turn15search18  
- Google ADK exposes explicit controls over whether prior conversation contents are sent to the model (`include_contents` can be set to `'none'` for stateless tasks), and it distinguishes context/state concepts in documentation. citeturn2search10turn16search5  
- Anthropic’s “effective context engineering” guidance frames context as a finite, critical resource and discusses strategies for curating and managing it. citeturn8search14turn3search15  
- RAG is the canonical method for injecting external knowledge via retrieval rather than stuffing full corpora into prompts; foundational RAG work formalized parametric + non-parametric memory for generation. citeturn8search0turn8search6 Surveys in 2024–2025 emphasize rapid evolution and system-level design considerations for retrieval-augmented LLMs. citeturn8search2turn8search5  
- Memory-tiered systems like MemGPT explicitly frame context management as an OS-like memory hierarchy with “interrupts” controlling interaction between agent and user, aiming to extend effective context under finite windows. citeturn8search1turn8search4  
- Multi-agent long-context decomposition (*Chain-of-Agents*) assigns segmented reading/reasoning across multiple agents and uses a manager to synthesize, explicitly targeting the difficulty of focusing in long contexts. citeturn12search5turn12search9turn12search13  
- Newer hierarchical agent loops explicitly incorporate bounded prompt growth: ReCAP claims “sliding-window scalability” where prompt size grows with depth, not total trajectory length. citeturn1search0turn1search12

**Key techniques.**  
A mature agentic loop typically combines:

- **Sliding windows** for recent turns,  
- **Summaries** for older dialogue/observations,  
- **Structured state** for durable facts (goals, constraints, tool outputs),  
- **Retrieval (RAG)** for external knowledge and prior run logs,  
- **Memory tiering** (working vs episodic vs archival), as highlighted in MemGPT and ReAcTree. citeturn8search1turn9search13turn8search0

Notably, ReCAP’s “structured injection” idea explicitly suggests that *what* you reinsert on backtracking matters (e.g., parent description, latest thoughts, remaining subtasks), which is highly relevant to preventing context drift during long loops. citeturn1search0turn1search12

**Applicability to Rust actors + supervision.**  
Context management is easiest to make reliable when it’s *not* an emergent property of a single growing message array:

- Put “working state” in a dedicated actor-owned state object (structured),  
- Put retrieval and summarization in dedicated memory actors,  
- Let the loop actor request “context packs” from memory actors each step (bounded by a prompt budget).

This corresponds to the explicit separation of “session memory” in SDKs (OpenAI Sessions; ADK controls) and OS-like memory tiering in MemGPT. citeturn15search6turn2search10turn8search4

**Implementation complexity.**  
Medium for sliding window + summarization. High for robust retrieval pipelines (chunking, embeddings, evaluation of retrieval quality) and for managing multiple memory tiers with explicit interrupts. The existence of RAG and MemGPT systems shows the design space, but implementing it cleanly in a distributed actor system requires careful state/versioning discipline. citeturn8search0turn8search4turn8search2

**Expected impact vs simple turn-counter loops.**  
Better context management reduces both failure rate and cost: it mitigates drift and hallucination from irrelevant prompt mass, improves long-horizon coherence (ReCAP), and allows scaling to long-context tasks via decomposition or retrieval rather than raw transcript growth. citeturn1search0turn12search5turn8search14

## Cognitive architecture and classical planning patterns

**Current state of the art.**  
Classical AI and cognitive architectures offer patterns for long-running goal pursuit and failure recovery that map cleanly to agent loops:

- **BDI (Belief–Desire–Intention)** agents separate world state (beliefs), goals (desires), and committed plans (intentions), emphasizing real-time performance and practical control of deliberation. citeturn5search0  
- **HTN (Hierarchical Task Network)** planning formalizes decomposition of tasks into subtasks and has deep results on expressivity/complexity; it’s a natural fit for “Planner produces hierarchy; Executor commits to leaf tasks; Critic validates.” citeturn5search3turn5search7  
- **STRIPS** is the canonical early model-based planner framing world models and operator sequences. citeturn6search0turn6search12  
- **PDDL** standardized representations for planning domains/problems, enabling planners to be benchmarked and interchanged. citeturn6search1turn6search9  
- **Behavior Trees** are widely used in robotics/game AI for modular, reactive control, providing predictable control flow under uncertainty. citeturn6search2turn6search6  
- **Reactive plan execution and repair** systems emphasize execution monitoring and runtime repair when plans fail in uncertain domains. citeturn6search11turn6search19

A key 2024–2025 planning-oriented critique is that autoregressive LLMs are unreliable as standalone planners and self-verifiers; the “LLM-modulo” view argues for tighter coupling between LLMs and external verifiers/planners instead of trusting pure prompting or self-verification. citeturn19search0turn19search4turn19search2

**Key techniques.**  
The most *implementable* transfers into LLM agent loops are:

- **Explicit belief state** updated by tool observations (BDI-style “belief revision” light), rather than letting beliefs live only in transcript text. citeturn5search0turn14search2  
- **Intention management / commitment**: commit to a subplan and execute it, but define explicit triggers for reconsideration (failure, low confidence, new constraints). This resembles ReCAP’s “commit to head item, refine remainder” and ReAcTree’s control-flow nodes. citeturn1search0turn9search13  
- **Hierarchical decomposition** (HTN): treat planning as building a tree of tasks, enabling local replanning rather than global rewrite. citeturn5search3turn9search13  
- **Verifier-in-the-loop** (“LLM-modulo”): LLM proposes; external system verifies constraints (PDDL planner, type checker, test runner), then feedback drives revision. citeturn19search0turn6search1  
- **Behavior Tree execution**: represent the Executor’s control policy as a behavior tree, with deterministic rules for fallback/retry/skip/human escalation. citeturn6search2turn6search3

**Applicability to Rust actors + supervision.**  
These architectures are structurally compatible with actor systems because they emphasize explicit state, modular control nodes, and recoverable execution. In particular, BTs and HTNs are effectively *graphs of control nodes*, which can map either to:

- a single agent actor running an explicit state machine, or  
- a tree of actors mirroring the plan hierarchy (each node supervised).

The “LLM-modulo” framing aligns tightly with introducing deterministic verifiers/tools as supervised, sandboxed actors that gate progress. citeturn19search0turn6search2turn5search3

**Implementation complexity.**  
- Medium: BDI-lite state + intention tracking + small plan library. citeturn5search0  
- High: HTN/PDDL integration and verifiers-in-the-loop across diverse domains. citeturn5search3turn6search1turn19search0  
- Medium–High: Behavior tree execution engine (but the structure is deterministic and testable). citeturn6search2turn6search6

**Expected impact vs simple turn-counter loops.**  
Cognitive/classical planning patterns mainly increase **predictability** and **testability**: control flow becomes explicit (BT/HTN) rather than emergent in transcripts; failures trigger defined repair/escalation paths; and external verifiers reduce the need to trust “self-verification” by the same stochastic model. This is exactly the motivation behind the LLM-modulo position and the robotics failure recovery literature. citeturn19search0turn6search11turn6search2

## Actor-model synthesis for Mister Smith

**Current state of the art.**  
Two threads of “durable agent loops” are particularly relevant:

- **Durable pause/resume with approvals**: OpenAI Agents SDK represents human-in-the-loop as an interruption + resumable run snapshot (`RunState`) that can be serialized and resumed after approval/rejection. citeturn15search13turn18search5turn13search3  
- **Deterministic workflow wrappers for probabilistic models**: Google ADK’s LoopAgent and shared invocation context explicitly frame loop control as deterministic structure around probabilistic inference. citeturn18search12turn16search1

For actor systems, OTP-style design offers concrete patterns for long-running processes:

- `gen_statem` describes a state machine behavior where an engine receives events and calls callback functions to compute new state + actions—an almost direct analog to an agent loop state machine. citeturn17search1turn4search1  
- Supervisor restart strategies (`one_for_one`, `one_for_all`, etc.) define how failure recovery should propagate. citeturn17search13turn4search4  
- Akka-style supervision semantics (resume/restart/stop/escalate) formalize the same conceptual space of recovery actions. citeturn17search3turn4search2

On the messaging layer, JetStream provides persistence and replay with at-least-once delivery semantics and message acknowledgements, enabling event-sourced loop traces and recovery after crashes. citeturn4search3turn4search7turn4search11

**Key techniques.**  
A synthesis that goes beyond simple loop counters is to treat “agent execution” as **(a) a state machine, (b) driven by events, (c) with durable checkpoints, and (d) capable of branching search**.

Concretely, the most defensible architecture—based on the cited best practices and research results—is:

1. **Agent Loop as an explicit state machine (gen_statem-like)**  
   Model each agent (Planner/Critic/Executor) as a long-running actor whose mailbox receives discrete events: `BeginTask`, `ModelResponse`, `ToolResult`, `ApprovalDecision`, `TimerExpired`, `BudgetUpdate`, `BranchScore`, `Backtrack`, etc. The callback returns next state and side effects, matching the gen_statem “event → actions → new state” framing. citeturn17search1turn4search5  

2. **Durable event log + checkpoint boundaries**  
   Persist every loop step (inputs, model outputs, tool calls, tool results, critic scores) to a JetStream stream so runs can be replayed and inspected; use message acknowledgements to ensure processing reliability. citeturn4search3turn4search11  
   Choose explicit checkpoint boundaries: *before side-effecting tools*, *after receiving tool results*, and *before returning final output*. This generalizes the “durable pause/resume boundary” idea of RunState beyond approvals to failure recovery and backtracking. citeturn15search13turn13search3  

3. **Supervision-driven strategy shifts and backtracking**  
   Instead of only restarting crashed actors, extend supervision policy to include **semantic restarts**: restart the agent loop with different inference settings or different loop policy when stuckness signals fire (timeouts, repeated tool calls, low critic scores). This mirrors the restart/escalate option set described in supervision documentation. citeturn17search13turn17search3  

4. **Search controller as a supervisor for branch exploration**  
   Implement a “search supervisor” actor that can spin up multiple child “trajectory actors” (each with its own loop state) using strategies inspired by LATS/SWE-Search/EnCompass: allocate budget to branches, prune low-scoring ones, and commit the best branch when confidence is sufficient. citeturn0search3turn9search8turn9search11  

5. **Tool execution as supervised workers with approval gates**  
   Tools should execute outside the model call path, supervised for timeouts/retries, and gated for human approval when required (analogous to `needsApproval` and RunState interruptions). citeturn13search1turn18search5turn13search3  
   For parallel tool calls, schedule tool workers concurrently only when tools are declared independent, otherwise force sequential execution (supported by explicit “parallel tool calls” toggles in existing SDKs). citeturn15search0turn14search1turn14search7  

6. **Policy-level budgets and termination conditions as composable “guards”**  
   Compose stop conditions like AutoGen (token usage, timeout) with per-agent and per-run budgets; when exhausted, trigger a best-effort summarization + safe termination rather than silent truncation. citeturn18search3turn3search5turn10search14

**Applicability to Rust actors + OTP supervision.**  
This architecture is exceptionally compatible with Mister Smith’s core primitives:

- Actor message passing already matches the “event → transition” model described in gen_statem behavior design. citeturn17search1turn4search1  
- OTP-style supervision concepts match the need to isolate failures, restart loops safely, and escalate systemic failures. citeturn4search4turn17search13  
- JetStream persistence and at-least-once delivery match the need for durable run traces, retryable tool work, and resumable long-horizon runs. citeturn4search3turn4search11  
- The human-in-the-loop “interruptions + resume” precedent shows that durable pause/resume semantics are already considered best practice in leading SDKs; Mister Smith can generalize it to backtracking and distributed search branches. citeturn15search13turn13search3turn18search5

**Implementation complexity.**  
High—but modular in exactly the way actor systems prefer:

- **Core loop state machine** (per agent): medium.  
- **Durable event log + checkpointing**: medium–high (serialization formats, schema evolution, replay tools). citeturn4search7turn4search11  
- **Search supervisor (MCTS / best-first)**: high (branch mgmt, value/judge integration). citeturn0search3turn11search8turn9search8  
- **Rollback/compensation semantics for tools**: high (requires tool taxonomy and compensators). citeturn6search11turn6search2  
- **Context/memory layer (summaries + retrieval + sessions)**: medium–high. citeturn8search14turn8search0turn15search6

**Expected impact vs simple loop-with-counter.**  
The expected improvements are structural:

- **Reliability under long horizons** via hierarchical control + bounded context injection (ReCAP), and explicit control flow (ReAcTree). citeturn1search0turn9search13  
- **Higher success rates on complex tasks** via search + evaluation (LATS, SWE-Search) rather than single-trajectory commitment. citeturn0search3turn9search8  
- **Production safety** via composable budgets, timeouts, approvals, and durable pause/resume boundaries. citeturn18search3turn13search3turn10search14  
- **Operational debuggability** because every step becomes an event you can replay and audit (a key practical advantage of durable execution patterns and persistent messaging). citeturn4search7turn4search11

**Synthesis recommendation: an “event-sourced, supervised, search-capable loop engine.”**  
The most “architecturally superior” loop design for Mister Smith—given the evidence across modern agent research and production SDK ergonomics—is:

- A **two-level loop**:  
  - **Inner loop (per trajectory):** ReAct-style reason/act/observe steps with strict tool schemas, approval gates, and context packs. citeturn0search0turn16search0turn13search1  
  - **Outer loop (supervisory controller):** branch-and-bound / MCTS-lite search that decides when to (a) continue the current trajectory, (b) fork alternatives, (c) backtrack to a checkpoint, or (d) terminate with best-effort output. citeturn0search3turn9search8turn9search11  

- **Critic as value function + safety judge:** continuously score partial trajectories, not just final outputs, borrowing directly from LATS/SWE-Search’s architecture. citeturn0search3turn9search0  

- **Durable boundaries everywhere side effects happen:** checkpoint before side-effecting tools; require compensators for irreversible actions; implement pause/resume exactly like human-in-the-loop run state, generalized to “search forks” and “rollback repairs.” citeturn15search13turn13search3  

- **Budget-aware strategy selection:** run cheap linear loops first; escalate to search/hierarchy only when critic signals low confidence or repeated failures, consistent with the need to control computational cost and the EnCompass separation of workflow logic vs inference-time strategy. citeturn9search7turn9search11turn10search3  

- **Context as a managed resource:** assemble bounded context packs each step using summaries + retrieval + structured state; adopt hierarchical context reinsertion on backtracking (ReCAP) to prevent drift. citeturn8search14turn1search0turn8search0  

This design uses the actor model and supervision trees not just as *implementation details*, but as *the architectural mechanism* for: isolation, restart, backtracking, exploration, and safe termination—capabilities that linear “max turns” loops cannot provide without substantial ad hoc complexity. citeturn17search13turn17search1turn4search3turn0search3