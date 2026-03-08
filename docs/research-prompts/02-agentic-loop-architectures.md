# Deep Research Prompt: Agentic Loop Architectures for Multi-Agent Orchestration

## Directive Context

Mister Smith is a Rust-based multi-agent orchestration framework with NATS/JetStream messaging, OTP-style supervision trees, and actor-based architecture. It must become architecturally superior to OpenAI Agents SDK, Google ADK, LangChain, CrewAI, AutoGen, and Claude SDK.

Phase 9 adds LLM provider integration. Agents (Planner, Critic, Executor) will call LLMs via a `ModelProvider` trait. The framework needs to define the agentic execution loop — how an agent interacts with a model across multiple turns, handles tool calls, evaluates its own output, backtracks on failure, and terminates safely.

Current competing frameworks use simple loop-with-counter patterns (OpenAI `max_turns`, Vercel `maxSteps`). The goal is to discover more sophisticated execution architectures that leverage Mister Smith's actor model and supervision trees.

## Research Objective

Discover the most advanced, innovative, and effective architectures for agentic execution loops — the pattern by which an LLM-powered agent reasons, acts, observes, and iterates toward a goal. Go beyond simple tool-call-and-return loops. Investigate reasoning architectures, search strategies, self-evaluation patterns, and backtracking mechanisms from both the AI agent literature and adjacent fields (game AI, planning systems, robotics).

## Research Dimensions

### 1. Reasoning Loop Architectures
- What is the current state of ReAct (Reasoning + Acting), Reflexion, and their successors?
- What newer reasoning loop patterns have emerged in 2025-2026?
- How do chain-of-thought, tree-of-thought, and graph-of-thought compare as execution strategies?
- Are there hybrid approaches that combine structured reasoning with freeform generation?
- What does LATS (Language Agent Tree Search) do differently from linear loops?

### 2. Multi-Turn Tool Calling Patterns
- Beyond simple "call tool, return result" — what patterns exist for multi-step tool orchestration?
- How do frameworks handle parallel tool execution (model requests 3 tools simultaneously)?
- What are the safety implications of parallel vs sequential tool execution?
- How do approval gates / human-in-the-loop checkpoints integrate into tool calling loops?
- What is Vercel AI SDK's `stopWhen` pattern and how does it separate "model proposes" from "runtime authorizes"?

### 3. Self-Evaluation and Self-Correction
- How do current frameworks implement agent self-evaluation (the agent critiques its own output before returning)?
- What is the state of Constitutional AI-style self-critique in agent loops?
- How does the Critic agent role (which Mister Smith already has) compare to inline self-evaluation?
- Are there techniques for detecting when an agent is stuck in a loop or producing low-quality output?
- What confidence estimation techniques exist for LLM outputs?

### 4. Backtracking and Rollback
- When a tool call produces a bad result, can an agent undo and try a different approach?
- How does this compose with supervision trees — can a supervisor trigger backtracking?
- Are there checkpoint/restore patterns for agent state that enable exploration of multiple solution paths?
- How do game AI systems (Monte Carlo Tree Search, minimax with alpha-beta pruning) approach similar problems?
- Can agent execution be modeled as a search problem with pruning?

### 5. Budget and Resource Management
- Beyond simple turn counters — what sophisticated budget management patterns exist?
- Token budgets, cost budgets, time budgets — how do frameworks enforce multiple simultaneous constraints?
- How do cascading budgets work (team budget -> agent budget -> turn budget)?
- What happens when a budget is nearly exhausted — does the agent produce a "best effort" response or fail gracefully?
- Are there patterns for budget-aware reasoning (agent adjusts its strategy based on remaining budget)?

### 6. Context Window Management
- As the agentic loop accumulates messages, how do frameworks handle context overflow?
- What are the current best practices for conversation summarization, sliding windows, and context compression?
- How do frameworks decide what to keep vs what to summarize vs what to drop?
- Are there techniques that leverage the model itself to manage its own context?
- How does RAG (retrieval-augmented generation) interact with the agentic loop?

### 7. Cognitive Architecture Patterns
- ACT-R, SOAR, and their modern successors — are there patterns from cognitive science that inform agent loop design?
- What can we learn from BDI (Belief-Desire-Intention) agent architectures?
- Are there hierarchical planning patterns (HTN — Hierarchical Task Networks) applicable to LLM agents?
- How do robotic planning systems (STRIPS, PDDL) handle plan execution with failure recovery?
- Is there research on combining classical AI planning with LLM-based reasoning?

### 8. Actor-Model Integration
- How does the agentic loop map onto actor-based execution?
- Can each turn of the loop be modeled as a message in the actor's mailbox?
- How do Erlang/OTP patterns for long-running processes (gen_server, gen_statem) inform agentic loop design?
- Can supervision trees provide automatic restart of stuck agents with different strategies?
- How does Akka (JVM actor framework) handle similar long-running, multi-step workflows?

## Output Requirements

For each dimension, provide:
1. **Current state of the art** — what exists today, with specific citations
2. **Key techniques** — the specific algorithms, architectures, or patterns discovered
3. **Applicability to Rust actors + supervision** — how well does this transfer?
4. **Implementation complexity** — rough assessment of effort
5. **Expected impact** — what improvement over simple turn-counter loops?

Conclude with a **synthesis section** recommending the optimal agentic loop architecture for a Rust actor system with OTP supervision, considering:
- Actor message-passing as the execution primitive
- Supervision-managed failure recovery and backtracking
- NATS messaging for distributed agent coordination
- Production safety (budget enforcement, stuck detection, graceful degradation)

## Research Methodology

1. Start with the academic literature on agent reasoning architectures
2. Survey what production agent frameworks actually implement (not just what they claim)
3. Look into adjacent fields: game AI, robotics, classical planning, cognitive science
4. Prioritize techniques that compose naturally with actor models and supervision trees
5. Be skeptical of approaches that work in demos but not at scale
6. For each technique, assess: does this work with real LLMs, or only in simulated environments?
