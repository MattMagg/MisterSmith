---
version: R1
created: 2026-03-07
updated: 2026-03-07
type: prompt
tier: 2
---

# Deep Research Prompt: Predictive Supervision & Cognitive Agent Coordination

## Context

Mister Smith is a Rust-based multi-agent orchestration framework built on NATS/JetStream messaging and OTP-style supervision trees. Its supervision system (Phase 3) implements Erlang/OTP-style restart strategies (OneForOne, OneForAll, RestForOne), phi accrual failure detection, circuit breakers, and health monitoring. No competing Python agent framework has anything comparable.

But OTP supervision is inherently reactive: detect failure, restart. For LLM agents, this is insufficient. Agents have characteristic failure patterns — a code-generating agent might hallucinate imports, a planning agent might produce circular dependencies, a critic agent might exhibit conformity bias. These patterns are predictable. Restart doesn't fix them.

## Frontier-First Mandate

OTP supervision is one of Mister Smith's strongest advantages. The goal is not to replace it but to leapfrog it — extending reactive supervision into predictive supervision that anticipates and prevents failures before they occur, and enriching agent coordination with cognitive models of collaborator capabilities. No framework, agent or otherwise, does this today. Erlang/OTP never had to supervise non-deterministic processes.

## Research Objective

Investigate two related frontiers:
1. **Predictive supervision**: Using agent performance profiles, historical failure patterns, and real-time signals to predict and prevent failures before they occur — extending OTP beyond restart into anticipation.
2. **Cognitive coordination**: Agents that build models of each other's capabilities, knowledge states, and failure modes to enable deep collaboration beyond simple task delegation.

These are related because both involve agents understanding each other's behavior at a deeper level than message passing allows.

## What We Already Know (Do Not Rediscover)

- **AWorld profile-aware maneuvering** (Xie et al. 2025): Offline profiling creates "performance fingerprints" per agent, enabling guard agents to deliver targeted interventions based on known failure patterns
- **OSC Collaborator Knowledge Models** (Zhang et al. 2025): Agents dynamically perceive collaborators' cognitive states, real-time gap analysis for adaptive communication. Significant gains on complex reasoning.
- **MetaOrch** (Agrawal & Nargund 2025): Neural selection with fuzzy evaluation for agent selection in multi-domain environments
- **MAST failure taxonomy** (134 citations): 14 failure modes in 3 categories map to supervision tree levels
- **Phi accrual failure detection**: Already implemented in Mister Smith, uses heartbeat inter-arrival times
- **Circuit breakers**: Already implemented, stateful (Closed/Open/HalfOpen)
- **Consensus-free debate / anti-conformity**: LLMs exhibit groupthink; forcing consensus degrades quality

We need: implementation details for performance fingerprinting, the cognitive model architecture, integration with OTP supervision, and adjacent-field techniques for predictive failure management.

## Research Dimensions

### 1. Agent Performance Profiling
- How does AWorld create "performance fingerprints"? What metrics compose a profile? How large is the profiling dataset needed?
- Can performance profiles be built online (during deployment) or do they require offline profiling?
- What agent behaviors are predictable vs. truly non-deterministic? Is there research on LLM behavioral stability?
- How do you profile an agent's interaction with specific models? (An agent might perform well with GPT-4 but poorly with Claude — is this captured?)
- What does the software testing literature say about characterization testing and behavior profiling?
- How do you update profiles as models are updated (model drift)?

### 2. Predictive Failure Detection
- Beyond phi accrual failure detection — what signals predict agent failure before it occurs?
- Can reasoning quality be estimated from partial outputs? (e.g., entropy of token predictions, embedding drift from expected trajectory, structural markers in generated text)
- What does the anomaly detection literature say about predicting failures in non-deterministic systems?
- Are there approaches from predictive maintenance (industrial IoT, aircraft engines, nuclear systems) that transfer to agent supervision?
- What about leading indicators? (e.g., increasing tool call frequency predicts task confusion, decreasing output length predicts context exhaustion)
- How does predictive failure detection compose with the existing circuit breaker? Can you trip a circuit breaker based on predicted failure rather than observed failure?

### 3. Targeted Intervention Strategies
- When a failure is predicted, what interventions are available beyond restart?
- Prompt augmentation: inject corrective context based on known failure pattern
- Model switching: route the next step to a different model known to handle this failure mode
- Context refresh: summarize and restart with compressed context
- Peer assistance: delegate the problematic step to a different agent with a stronger profile for this task type
- Human escalation: when confidence in all automated interventions is low
- How do you choose between interventions? Is there a decision tree, a learned policy, or a rule-based system?
- What does the aviation literature (CRM — Crew Resource Management) say about intervention strategies when a team member is degraded?

### 4. Cognitive Models of Collaborators
- OSC's Collaborator Knowledge Models (CKMs): what representation do they use? How large are they? How do they update?
- What does the Theory of Mind (ToM) literature say about agents modeling each other's knowledge states?
- Can agents maintain lightweight models of collaborators' strengths and weaknesses? What is the overhead?
- How does this differ from capability advertisement (A2A Agent Cards describe what an agent CAN do; cognitive models describe what it IS LIKELY TO DO in specific contexts)?
- What does the organizational behavior literature say about team mental models and transactive memory systems?
- Can cognitive models detect conformity bias and compensate? (If the cognitive model predicts Agent B will agree with Agent A regardless of quality, weight Agent B's agreement less)

### 5. Integration with OTP Supervision Trees
- How does predictive supervision extend the existing restart strategy interface?
- Can supervision strategies be parameterized by agent profiles? (e.g., "for this agent with this profile, on this type of predicted failure, use this intervention")
- What is the supervision hierarchy? Profile Manager -> Guard Agent -> Supervised Agent? Or does the supervisor itself maintain profiles?
- How do predictive interventions interact with restart intensity limits? Does a predicted failure count against the restart budget?
- Can the supervision tree learn which intervention strategies are effective per agent type?

### 6. Integration with NATS Infrastructure
- Agent performance metrics can be published to NATS telemetry subjects. How do you aggregate and analyze these for predictive signals in real-time?
- JetStream KV could store agent profiles. How do profiles compose with the existing KV-based config system?
- Can NATS advisory events ($JS.EVENT.ADVISORY.>) provide signals for predictive supervision?
- Profile updates should be eventually consistent across the cluster. Does this create issues for coordinated prediction?

### 7. Adjacent-Field Techniques
- **Predictive maintenance**: Industrial IoT uses vibration analysis, temperature trends, and degradation curves to predict equipment failure. What transfers to agent reasoning degradation?
- **Crew Resource Management (aviation)**: How do flight crews monitor and compensate for degraded team members?
- **Adaptive control theory**: Model Reference Adaptive Control (MRAC) adjusts controller parameters based on observed system behavior. Can this be applied to supervision parameters?
- **Game theory / mechanism design**: Can supervision be modeled as a mechanism that incentivizes agents to self-report degradation?
- **Cognitive science**: Joint attention, shared mental models, distributed cognition — what transfers to multi-agent LLM systems?
- **Sports analytics**: Player performance profiling, fatigue prediction, dynamic substitution — what transfers to agent team management?

## Output Structure

For each dimension:
1. **State of the art** — what exists, with citations
2. **Key techniques** — specific models, algorithms, or data structures
3. **Applicability to Mister Smith** — integration with existing PhiAccrualFailureDetector, CircuitBreaker, SupervisedSystem
4. **Overhead assessment** — latency, compute, memory cost of predictive supervision vs. reactive supervision
5. **Open problems** — what's unsolved, speculative, or potentially counterproductive

Conclude with:
- Architecture sketch: how predictive supervision extends Mister Smith's existing supervision infrastructure without replacing it
- Decision framework: when to use reactive restart vs. predictive intervention vs. human escalation
- Cognitive coordination design: what does the agent-to-agent knowledge model look like, where does it live, how does it compose with supervision?
- Honest assessment: is predictive supervision practically achievable with current LLM capabilities, or is it aspirational?

## Research Methodology

1. Start with AWorld and OSC papers — trace citation graphs
2. Search for "predictive supervision", "agent performance profiling", "failure prediction multi-agent"
3. Deep dive into Theory of Mind for AI agents literature
4. Study predictive maintenance literature (ISO 13381, condition monitoring, remaining useful life estimation)
5. Study Crew Resource Management literature from aviation
6. Look at adaptive control theory (MRAC, L1 adaptive control) for supervision parameter adaptation
7. Search for "team mental models", "transactive memory systems", "shared situation awareness"
8. Prioritize 2025-2026 papers; include foundational work on ToM for LLMs and adaptive supervision
