# Supervision, Fault Tolerance & Resilience -- Consolidated State of Knowledge

**Compiled:** 2026-03-07
**Sources:** 7 research files across rounds R3--R7 (3 industry synthesis reports, 49 academic papers via Consensus, 5 frontier deep dives, 4 discovery sweeps totaling ~2,000+ papers screened)
**Target:** Mister Smith framework -- Rust + NATS/JetStream + OTP-style supervision trees, model-agnostic multi-agent orchestration

---

## Executive Summary

Supervising LLM-based multi-agent systems is fundamentally harder than supervising deterministic processes. LLM APIs are long-running, non-deterministic external dependencies that fail semantically (hallucination loops, conformity bias, context drift) in addition to the standard transient/structural failure modes of distributed systems. The research corpus converges on a clear architectural thesis: Mister Smith must evolve from **reactive OTP restarts** to a **layered resilience architecture** combining (1) traditional supervision trees for hard crashes, (2) predictive Guard agents that anticipate failures via telemetry and agent profiling, (3) cognitive coordination models that enable agents to understand peer states, and (4) biomimetic immune-system patterns for detecting semantic corruption across the agent swarm.

Five findings achieved **high confidence** through independent convergence across all three R3 synthesis reports (each derived from separate research efforts):

1. **Isolate LLM I/O from supervisors** -- spawn short-lived async tasks with one-shot channels; never block actor mailboxes on 30-second LLM calls.
2. **Deploy stateful Gatekeeper actors** -- token-bucket rate limiting + supervised circuit breakers per provider; blind exponential backoff exacerbates structural failures.
3. **Classify failures structurally** -- transient (429, 5xx), structural (400, 401, content filter), streaming-specific (partial drops, stale SSE), and semantic (hallucination loops) each require distinct supervisory strategies.
4. **Checkpoint agent state durably** -- event-source to JetStream append-only logs with `Nats-Msg-Id` deduplication for exactly-once recovery.
5. **Use role-aware supervision** -- Executors are transient (`OneForOne`), Planners are permanent with escalation, Critics support quorum; Saga compensations enable graceful degradation without destroying the workflow.

Beyond these, the research identifies three frontier capabilities that differentiate Mister Smith from Python-based frameworks: predictive supervision via AWorld performance fingerprints (57.4% variance reduction), cognitive coordination via OSC Collaborator Knowledge Models (communication redundancy reduced to 12.6%), and biomimetic fault tolerance via consensus-based threat validation (sub-millisecond Byzantine-robust peer voting).

---

## High-Confidence Findings

These findings are supported by convergent evidence from multiple independent research efforts and peer-reviewed academic papers.

### 1. LLM Calls Must Be Isolated from the Supervision Tree

**Confidence: HIGH (all R3 reports converge; OTP documentation confirms)**

Blocking an actor on a 30-second LLM generation paralyzes the supervision tree. Late replies from timed-out calls become "poison pills" in the actor mailbox. The canonical solution is spawning unlinked, supervised background tasks (Erlang's `Task.Supervisor`, mapped to Tokio's `spawn` + `JoinHandle` in Rust) with one-shot channels (`tokio::sync::oneshot`) that can be safely dropped on timeout. The EEP-53 alias pattern ensures stale responses route to dead-letter queues rather than corrupting agent state.

**Mister Smith mapping:** Phase 3's `ActorCell` with bounded mailbox must never await LLM futures inline. Phase 2's `TaskExecutor` should manage LLM call lifecycles with timeout wrappers.

*Sources: R3 [A1][A2][A11][A12][B1][B2][B10][B38][B40][B85][C14]*

### 2. Structural Failure Classification Prevents Restart Storms

**Confidence: HIGH (all R3 reports produce near-identical taxonomies; MAST provides academic grounding)**

Not all LLM failures are retryable. The unified taxonomy:

| Category | Examples | Correct Response |
|:---|:---|:---|
| **Transient** | 429 (rate limit), 500/502/503, network timeout | Retry with backoff+jitter; circuit-breaker increments |
| **Structural** | 400 (invalid), 401/403 (auth), content filter, model deprecated | Fail-fast; escalate to planner; no retry |
| **Streaming** | Partial SSE drop, stale connection (no tokens) | Checkpoint received tokens; sever + resume or failover |
| **Semantic** | Hallucination loops, unbounded tool-call recursion, step repetition | Monitor token entropy/repetition; cap iterations; model switch |

Treating structural failures as transient causes billing spikes from futile retries and restart storms that overwhelm the supervisor.

*Sources: R3 [A3][A13--A18][B20--B31][C16][C26]; R4 MAST taxonomy (Huang et al. 2025, 1,642 traces)*

### 3. JetStream-Backed Checkpointing Enables Deterministic Recovery

**Confidence: HIGH (all R3 reports converge; LangGraph production validation)**

Agent state must be event-sourced to JetStream `FileStorage` streams at every super-step. Recommended JetStream configuration: `LimitsPolicy` retention (7-day age), `AckExplicit` acknowledgment, `MaxDeliver: 5` to cap poison-message loops. `Nats-Msg-Id` headers provide idempotent replay. Schema versioning with strict compatibility rules prevents silent corruption across code deployments.

**Checkpoint schema fields:** `workflow_id`, `checkpoint_id`, `schema_version`, conversation history, tool outputs, provider cursor (stream offset), model ID, tokens consumed, pending tasks, `resume_hint`, timestamp.

*Sources: R3 [A9][A10][A28--A36][B17][B66][B67][B69][B124--B138][C44][C46][C59]*

### 4. Role-Aware Supervision Outperforms Blanket Restarts

**Confidence: HIGH (all R3 reports converge independently)**

| Agent Role | Restart Policy | Strategy | Rationale |
|:---|:---|:---|:---|
| **Executor** | Transient | `OneForOne` | Independent tasks; restart only the failed worker; progressive model downgrade via Saga |
| **Planner** | Permanent | Escalation | Workflow-critical; diagnostic inspection before restart; dead-letter after threshold |
| **Critic** | Quorum/Replacement | Conditional | Can elect replacement via NATS; stricter diagnostics; quorum for safety-critical outputs |
| **Coordinator** | Permanent singleton | Supervised | Listens to JetStream health; executes topology changes |

Hybrid supervisor trees (inner supervisor for Executors with `OneForOne`, outer supervisor managing Planner + inner supervisor) give maximum flexibility during cascading failures.

*Sources: R3 [A39--A42][B11][B42--B45][B76][B104--B106][C3][C13]*

### 5. Circuit Breakers Must Be Active Supervised Processes

**Confidence: HIGH (all R3 reports converge; Erlang `fuse` validates)**

Circuit breakers modeled as passive middleware miss state-coordination opportunities. As supervised actors, they can broadcast Open/Half-Open/Closed state changes via NATS for cluster-wide coordination. Erlang's `fuse` library demonstrates this but loses counters on restart (ETS-backed). Mister Smith should optionally persist counters to JetStream KV for durability. Service-mesh studies show circuit-breaker + bulkhead patterns achieve **91.3% blast-radius reduction** [B116].

*Sources: R3 [A4][B4][B5][B12][B63][B68][B116]*

---

## Key Techniques & Architectures

### OTP-Style Supervision Trees (Reactive Restart)

**Mechanism:** Erlang/OTP's supervision model uses a tree of supervisor processes that monitor child workers. When a child crashes, the supervisor applies a restart strategy (`OneForOne`, `OneForAll`, `RestForOne`) bounded by intensity limits (max N restarts in T seconds). Mister Smith implements this in Phase 3 via `SupervisedSystem` with all three strategies.

**Evidence:** OTP supervision has 30+ years of production validation in telecom systems. Mister Smith's Phase 3 already implements `OneForOne`, `OneForAll`, and `RestForOne` with configurable restart budgets. Phase 8 adds `HeartbeatBridge` feeding `PhiAccrualFailureDetector`.

**Limitation for LLM agents:** Reactive restarts cannot fix the characteristic failure patterns of LLMs. Restarting an agent that hallucinated imports will reproduce the same hallucination because the underlying model behavior is unchanged. This is the core motivation for predictive supervision (see below).

**Mister Smith integration:** OTP supervision remains the foundation -- the "last resort" hard-restart layer. Predictive supervision layers above it as a pre-filter, consuming soft intervention budgets before escalating to hard restarts.

*Sources: R3 Section 7; R6 Section 1; R4 Section 9 (formal verification of actor systems)*

### MAST Failure Taxonomy (14 Failure Modes, 3 Categories)

**Mechanism:** The Multi-Agent System Failure Taxonomy (MAST), published by Huang et al. (2025), annotated **1,642 execution traces** to identify **14 fine-grained failure modes** across three categories:

| Category | Failure Modes | Example |
|:---|:---|:---|
| **System Design Issues** | FM-1.1 (Tool Selection Error), FM-1.2 (Tool Argument Error), FM-1.3 (Step Repetition -- **17.14% of failures**), FM-1.4 (Incorrect Planning) | Agent loops through identical tool calls without progress |
| **Inter-Agent Misalignment** | FM-2.1 (Instruction Non-compliance), FM-2.2 (Role Boundary Violation), FM-2.3 (Task Derailment), FM-2.4 (Information Withholding) | Agent fails to communicate API requirements to peers |
| **Task Verification** | FM-3.1 (Premature Termination), FM-3.2 (Verification Omission), FM-3.3 (Incorrect Verification) | Agent halts before fulfilling all user constraints |

**Evidence:** 1,642 annotated traces across multiple MAS benchmarks. Step Repetition (FM-1.3) is the single most common failure at 17.14%. The taxonomy provides a standardized vocabulary for Guard agent monitoring.

**Key metric:** MAS-FIRE (2026) found that iterative, closed-loop architectures (with Critic feedback) recover from **over 40%** of faults that break linear workflows.

**Mister Smith integration:** Guard agents should map their monitoring signals directly to MAST failure modes, enabling targeted interventions rather than generic restarts. The `smith_bus` telemetry stream should tag detected anomalies with MAST codes.

*Sources: R4 Section 1 (Huang et al. 2025, ArXiv 2503.13657); R6 Section 2; R3 [C35]*

### Predictive Supervision (AWorld Fingerprints, Profile-Aware Maneuvering)

**Mechanism:** AWorld's Profile-Aware Maneuvering (Xie et al. 2025) introduces an offline System Identification pipeline inspired by control theory:

1. **Benchmarking:** Subject each agent to 50--200 representative tasks (e.g., GAIA validation dataset).
2. **Fingerprint Generation:** A high-capacity analyzer LLM studies complete input-output logs to synthesize a structured "performance fingerprint" -- a human-readable profile of the agent's habitual failure modes.
3. **Online Execution:** The fingerprint is injected into a Guard Agent's prompt as "Context-Level Reinforcement," enabling the Guard to monitor for likely failure scenarios and offer targeted, preemptive advice.

**Evidence:** AWorld reports **57.4% reduction in performance variance standard deviation** when using profile-aware maneuvering vs. naive supervision. Fingerprints map directly to MAST failure modes for standardized intervention routing.

**Guard/Advisor Layer Architecture:**
1. **OTP Supervisor** (root) -- manages hard restart budgets; retains ultimate kill/restart authority
2. **Profile Manager** (background) -- syncs fingerprints and CKMs from NATS JetStream KV
3. **Guard/Predictive Advisor** (sidecar per agent) -- monitors streaming telemetry (token entropy, embedding drift, phi-accrual heartbeats); executes soft interventions from an intervention budget
4. **Execution Agent** (worker) -- performs actual LLM tasks

**Decision Framework:**
- **Fast-Path (Reactive):** Phi-accrual detector exceeds threshold or catastrophic self-report --> circuit breaker trip --> hard restart via OTP Supervisor
- **Predictive Path (Proactive):** Rising token entropy or MAST coordination failure detected --> consume intervention budget (prompt augmentation, context refresh, model switch) --> escalate to restart only if budget depletes

**Intervention budget:** Max 3 soft interventions per minute; max 5 hard restarts per hour. Anti-oscillation: `ModelSwitch` locked out for a defined TTL after application to prevent rapid toggling.

**Mister Smith integration:** Store fingerprints in JetStream KV (`KV_profiles.{agent_id}`, History Depth: 5, TTL-enabled). Guard agents consume from `agent.telemetry.{tenant}.{agent_id}` (LimitsPolicy, 3-day max age, ephemeral consumers). Use EWMA over cognitive heartbeats with <50ms latency budget.

*Sources: R6 Sections 1--4; R5 Section 3.3; Discovery R4 Theme 4*

### Cognitive Coordination (OSC CKMs, Collaborator Knowledge Models, Theory of Mind)

**Mechanism:** The Orchestrated Synergy in Collaboration (OSC) framework (Zhang et al. 2025) introduces Collaborator Knowledge Models (CKMs) to give agents a "Theory of Mind" -- the ability to dynamically perceive collaborators' cognitive states.

**Architecture:**
- **CKM:** Lightweight Transformer encoder (2 layers, 2 heads, **128-dimensional** model dimension)
- **Inputs:** Embeddings of the collaborator's recent utterances (last 5 turns), the query, and dialogue history
- **Outputs:** Dynamic latent cognitive state vector z in R^128 encoding the collaborator's understanding, confidence, and awareness of constraints

**RL-Based Communication Policy:** Using the CKM, agents perform "cognitive gap analysis" to identify discrepancies between their plan and their peers' understanding. An adaptive communication policy (pi_comm), trained via Proximal Policy Optimization (PPO), selects structured communication actions (objective, target, style) to bridge the gap.

**Evidence:**
- Communication redundancy reduced to **12.6%** (vs. uncoordinated baselines)
- Conflict resolution rate: **89.5% to 91.7%**
- Significant gains on complex reasoning tasks requiring deep collaboration

**Anti-Conformity:** To prevent LLM agents from "herding" (agreeing with dominant peers despite internal doubts), integrate Bayesian Truth Serum (BTS) and Peer Prediction. Score agents on their ability to accurately predict peer responses while providing surprisingly novel information, incentivizing truthful capability advertisement and breaking conformity bias. Free-MAD (Cui et al. 2025) demonstrates that forcing consensus degrades quality due to conformity bias -- score-based evaluation of entire debate trajectories outperforms majority voting.

**Mister Smith integration:** Store CKM state vectors in JetStream KV (`KV_profiles.{agent_id}`). Use Compare-And-Swap (CAS) semantics for atomic updates when multiple Guards update a CKM. Use `Nats-Msg-Id` for exactly-once processing of intervention commands. Start with periodic batched CKM updates (not per-token online learning) until lightweight on-device training becomes viable.

*Sources: R6 Section 5; R5 Section 3.3; Discovery R4 Theme 10*

### Guard/Advisor Layer (MetaOrch Fuzzy Evaluation, Targeted Interventions)

**Mechanism:** MetaOrch (Agrawal & Nargund 2025) introduces a fuzzy evaluation module scoring agent responses on three interpretable axes:

1. **Completeness:** Did the response fully address all task aspects?
2. **Relevance:** Was the response contextually appropriate?
3. **Confidence:** Was the response internally consistent?

These fuzzy scores drive intervention selection:
- Low Completeness --> **Prompt Augmentation** (remind agent of missing constraints)
- Low Relevance --> **Context Refresh** (scrub noisy tool outputs, truncate history)
- Low Confidence --> **Model Switching** (route to higher-capacity model)

**Evidence:** MetaOrch achieves **86.3% intervention selection accuracy** using fuzzy evaluation to dynamically route degraded agents to targeted interventions.

**Aviation CRM Escalation:** When automated interventions fail (3 consecutive failures), execute a "Sterile Cockpit" protocol: halt all external tool access, quarantine the agent, escalate to human-in-the-loop dashboard with a summarized decision trace. Adapted from FAA Crew Resource Management (AC 120-51C).

**Tokens-to-Derailment metric:** Adapted from industrial Predictive Maintenance (ISO 13381), track the degradation of MetaOrch fuzzy scores over a session to forecast when an agent's context window will become too noisy to function, triggering proactive context refresh before failure.

**L1 Adaptive Control analogy:** The Guard acts as the fast adaptation loop (high-frequency prompt augmentations), while the OTP Supervisor acts as the low-pass filter ensuring the system remains within proven stability bounds (the robustness loop).

*Sources: R6 Sections 4, 8; R5 Section 3.3*

### Biomimetic Fault Tolerance (Digital Immune System, Consensus-Based Threat Validation)

**Mechanism:** The Decentralized Multi-Agent Swarm architecture (2026) abandons centralized orchestrators for edge-based, peer-to-peer behavioral monitoring inspired by the adaptive immune system's Crossregulation model.

**Consensus-Based Threat Validation (CVT):**
1. An executing agent's output stream is broadcast to a localized quorum of lightweight evaluator actors (observer nodes)
2. Observer actors cast continuous, sub-millisecond votes on behavioral health using a weighted Byzantine fault-tolerant voting protocol
3. If the weighted consensus threshold is breached, the immune actors autonomously sever the agent's message bus connection and trigger context-aware rollback
4. The anomaly is quarantined before it corrupts the broader execution state

**Evidence:**
- **Sub-millisecond consensus** regarding node health and safety
- **Near-perfect detection accuracy** in large-scale simulations
- Prevents cascading control-plane failures even when a **significant percentage of agents are actively compromised** or exhibiting Byzantine faults
- Mirrors biological immune system: regulatory agents continuously limit excessive activation without centralized coordination

**Biomimetic memory formation:** Decay functions and pattern storage allow the immune system to recall previously verified threat states, reducing re-evaluation cost for known attack patterns.

**Comparison to traditional supervision:**

| Paradigm | Detection | Resolution | Blind Spot |
|:---|:---|:---|:---|
| Traditional OTP Supervision | Binary process crash | Clean-slate restart | Semantic degradation, hallucination loops |
| Centralized Reward Models | Step-wise LLM verification | MCTS backtracking | Single point of failure; high latency |
| Consensus-Based Immunity | Distributed sub-ms Byzantine voting | Autonomous quarantine + localized rollback | Requires redundant lightweight observer nodes |

**Mister Smith integration:** Use NATS pull consumers to broadcast agent output streams to observer quorums. Observer actors are lightweight (no LLM calls -- rule-based or small-model evaluation). The existing `EventBus` + `PhiAccrualFailureDetector` infrastructure provides the substrate. JetStream KV stores the "immune memory" of known threat signatures with TTL-based expiry.

*Sources: Discovery R7d (biomimetic section); ArXiv 2601.17303*

### Contextual Rollback (COCO, Failure Context Propagation)

**Mechanism:** When an agent fails, blind restart loses the context that caused the failure. Two complementary approaches address this:

1. **Checkpoint-and-resume:** Event-sourced state in JetStream allows replaying to the last known-good checkpoint, then re-executing with modified inputs (context truncation, prompt modification, model switch).

2. **AgentAsk clarification modules** (Li et al. 2025): Plug-and-play modules at every inter-agent message handoff that arrest error cascades by detecting ambiguity and requesting clarification before propagating potentially corrupted context. Acts as an edge-level error mitigation layer.

3. **Failure context propagation:** When a worker fails, the failure reason (MAST code, fuzzy evaluation scores, token entropy at failure) is attached to the checkpoint record and the failover directive. The replacement worker or Guard agent receives full diagnostic context rather than a bare "restart" signal.

**Evidence:** AgentAsk substantially improves reliability in long-horizon tasks with minimal latency/cost overhead (Evidence strength: 7/10). MAS-FIRE shows closed-loop architectures (which propagate failure context back to planners) recover **40%+ of faults** that break linear workflows.

**Mister Smith integration:** Extend the `ProcessStateTracker` (Phase 8) with failure context fields. Failover directives on NATS (`ms.failover.<workflow_id>`) should include `failure_context: { mast_code, fuzzy_scores, token_entropy, last_checkpoint_id }`. The Planner can use this context for informed replanning.

*Sources: R3 Section 5; Discovery R7b (AgentAsk); R3 [C35]*

### Saga Pattern for Multi-Agent Workflows (SagaLLM)

**Mechanism:** The Saga pattern decomposes a long-running distributed transaction into a sequence of local transactions, each with a compensating transaction that undoes its effects on failure.

**SagaLLM** (Li et al. 2025, ArXiv, 41 citations) directly applies saga patterns to LLM-based multi-agent orchestration:
- Maps each agent task step to a saga step with explicit compensating transactions
- Supports both choreography-based (agents react to events) and orchestration-based (central coordinator manages) execution
- Implements `AgentSagaCoordinator` with `undo()` for each step

**Evidence:** 41 citations in months indicates rapid adoption. SagaLLM demonstrates practical implementation of compensating transactions for LLM workflows -- e.g., if a code-generation agent produces invalid code, the compensating transaction deletes the draft and notifies the planner to try a simpler approach.

**Progressive degradation via Saga:**
1. **Model downgrade:** Executor fails with reasoning model (o1) --> Saga compensation triggers fallback to cheaper model (GPT-4o-mini)
2. **Plan simplification:** If task fundamentally cannot complete --> compensating transactions undo side-effects --> Planner generates simplified plan
3. **Human escalation:** After N Saga failures --> dead-letter to human-in-the-loop dashboard

**Capability degradation levels:**
- **Full mode:** All agents, best models, complete tool access
- **Degraded mode:** Fewer examples, smaller models, simplified planning
- **Minimal mode:** Templates, cached answers, direct human escalation

**Mister Smith integration:** Map `RestForOne` supervision to Saga compensations. Publish degraded-plan decisions to JetStream for traceability. Each tool invocation should register a compensating transaction in the checkpoint stream. The existing `ToolBus` (Phase 7) can be extended with compensation handlers.

*Sources: R4 Section 4 (SagaLLM -- Li et al. 2025, 41 citations); R3 Section 4.3*

---

## Open Questions & Gaps

### Empirically Validated

1. **Phi-accrual parameterization for LLM latency.** No source provides concrete parameters for adapting phi-accrual to high-variance LLM APIs. The detector works well for heartbeats with relatively stable distributions, but LLM inter-token latency varies by orders of magnitude depending on model, prompt length, and server load. Requires empirical calibration with real provider latency distributions from Phase 9 integration.

2. **Token entropy monitoring algorithms.** Multiple sources recommend monitoring token entropy to detect hallucination loops, but none provide specific algorithms or calibrated thresholds. Implementation requires experimental research during Phase 9.

3. **CKM online learning feasibility.** OSC's 128-dim CKM architecture is validated offline, but fully online, real-time PPO updates during live execution remain computationally expensive and prone to catastrophic forgetting. The R6 report honestly assesses this as "aspirational" requiring R&D. Start with periodic batched updates.

4. **Streaming resume protocol specifics.** Exact protocols for resuming partial LLM streams vary by provider and are not standardized. Implementation requires per-provider adapter logic in the `mister-smith-llm` crate.

5. **Durable circuit-breaker design.** Evidence notes Erlang `fuse`'s ETS counter loss on restart but provides no recommended durable design. Mister Smith must design JetStream KV backing for circuit-breaker state with acceptable write latency.

### Theoretically Promising but Unvalidated at Scale

6. **Biomimetic immune system in production.** Consensus-based threat validation shows near-perfect detection in simulation but lacks production deployment evidence. The redundant observer node cost is unquantified for real LLM workloads.

7. **Anti-gaming for predictive supervision.** If agents learn the Guard's entropy thresholds, RL-tuned models may artificially alter output distributions to avoid triggering interventions, masking true failures. Randomized adversarial "red team" audits are recommended but not validated.

8. **Trust calibration across heterogeneous agent modalities.** Trust-aware arbitration for integrating diverse LLMs/tools is nascent and primarily studied in vision-language settings. Generalization to Mister Smith's 9 agent roles is an open question.

9. **Adversarial robustness under decentralized coordination.** Few works rigorously address adversarial failure modes in decentralized agent DAGs. The "Agent Smith" infectious jailbreak vector (single poisoned input achieves system-wide compromise exponentially fast) remains an active threat without proven mitigations at scale.

10. **Formal verification of supervision policies.** Session types (pi-calculus mapped to Rust's type system via `rumpsteak`) can provide compile-time deadlock-freedom guarantees, but practical integration with OTP-style dynamic supervision trees is unexplored.

---

## Implementation Priority for Mister Smith

Ordered by impact-to-effort ratio, mapped to the existing crate architecture.

### Tier 1: MVP Foundation (Phase 9 Integration -- High Impact, Moderate Effort)

| Priority | Component | Crate | Description | Evidence |
|:---|:---|:---|:---|:---|
| **P1** | Isolated LLM Task Workers | `mister-smith-llm` | Spawn LLM calls as unlinked async tasks with `oneshot` channels + timeout wrappers. Never block actor mailboxes. | HIGH confidence; all R3 reports converge |
| **P2** | Structural Failure Classifier | `mister-smith-llm` | Classify provider errors into transient/structural/streaming/semantic categories; route to distinct recovery paths. | HIGH confidence; prevents restart storms |
| **P3** | Supervised Circuit Breaker | `mister-smith-llm` + `mister-smith-async` | Per-provider circuit breaker as supervised actor; broadcast state via NATS; consult before spawning Executors. | HIGH confidence; 91.3% blast-radius reduction |
| **P4** | Workflow Checkpoint Streams | `mister-smith-persistence` | Per-workflow JetStream `FileStorage` streams with checkpoint schema, resume capability, and `Nats-Msg-Id` dedup. | HIGH confidence; enables deterministic recovery |
| **P5** | Role-Aware Supervisor Logic | `mister-smith-supervision` | Extend `SupervisedSystem` with agent-role metadata; Executor=transient/OneForOne, Planner=permanent/escalation, Critic=quorum. | HIGH confidence; all reports converge |

### Tier 2: Resilience Hardening (Post-Phase 9 -- High Impact, Higher Effort)

| Priority | Component | Crate | Description | Evidence |
|:---|:---|:---|:---|:---|
| **P6** | Bulkhead Isolation | `mister-smith-supervision` | Separate supervisor pools for expensive vs. cheap models; per-pool concurrency + token budget limits. | HIGH confidence; standard pattern |
| **P7** | Saga Compensation Framework | `mister-smith-agents` | Register compensating transactions per tool invocation; progressive model downgrade; plan simplification. | 41-citation SagaLLM validates |
| **P8** | Provider Health Tracking | `mister-smith-llm` | `ProviderHealth` actor aggregating p50/p95/p99, error rates, token velocity, phi-accrual scores; publish to NATS + JetStream. | HIGH confidence; enables routing |
| **P9** | P2C+EWMA Routing | `mister-smith-llm` | Power-of-Two-Choices + EWMA latency routing across providers; outlier detection penalty boxes; NATS KV health sync. | Service-mesh validated |
| **P10** | Chaos Test Harness | `mister-smith-integration-tests` | Toxiproxy-style fault injection: 429 floods, SSE drops, stale connections, model deprecation, deadlock simulation. | All R3 reports require; essential for confidence |

### Tier 3: Predictive Supervision (Frontier Differentiation -- High Impact, High R&D)

| Priority | Component | Crate | Description | Evidence |
|:---|:---|:---|:---|:---|
| **P11** | AWorld Offline Profiling | `mister-smith-agents` | Benchmark agents against representative tasks; generate performance fingerprints; store in JetStream KV. | 57.4% variance reduction |
| **P12** | Rule-Based Guard Agents | `mister-smith-agents` | Sidecar Guard per Execution Agent; monitor telemetry (entropy, drift, tool-call anomalies); apply static-rule interventions. | AWorld + MetaOrch validate |
| **P13** | MetaOrch Fuzzy Evaluation | `mister-smith-agents` | Score agent outputs on Completeness/Relevance/Confidence; route to targeted interventions. | 86.3% selection accuracy |
| **P14** | Edge-Level Clarification | `mister-smith-agents` | AgentAsk-style modules at inter-agent handoffs to detect ambiguity and arrest error cascades. | Moderate evidence; minimal overhead |

### Tier 4: Aspirational (Requires Significant R&D)

| Priority | Component | Description | Evidence |
|:---|:---|:---|:---|
| **P15** | OSC Collaborator Knowledge Models | 128-dim CKMs with periodic batched updates; cognitive gap analysis for communication optimization. | 12.6% communication redundancy; needs training infra |
| **P16** | Biomimetic Observer Swarms | Lightweight evaluator quorums with Byzantine-robust voting on agent behavioral health. | Near-perfect sim accuracy; unvalidated in production |
| **P17** | Formal Protocol Verification | Multiparty Session Types via Rust type system for compile-time deadlock-freedom of agent choreographies. | Theoretically sound; practical integration unexplored |
| **P18** | RL-Trained Guard Policies | PPO-trained communication and intervention policies replacing static rules. | Aspirational; requires online training infrastructure |

---

## Sources

### Primary Synthesis (R3)
- `synthesis/supervision-llm-fault-tolerance-R3.md` -- Triple synthesis of 3 independent industry research reports covering OTP patterns, distributed resilience, service-mesh routing, chaos engineering, safety-critical systems, LLM failure taxonomies

### Academic Research (R4)
- `research/targeted-supervision-fault-tolerance-R4.md` -- 49 papers via Consensus Academic Search (Semantic Scholar, ArXiv, IEEE, ACM); covering MAST taxonomy, Byzantine fault tolerance, saga patterns, chaos engineering, observability, formal verification, tool-calling recovery

### Frontier Deep Dives (R6)
- `research/targeted-predictive-supervision-R6.md` -- AWorld performance fingerprints, OSC Collaborator Knowledge Models, MetaOrch fuzzy evaluation, Guard/Advisor hierarchy, NATS JetStream integration blueprint, L1 adaptive control, safety constraints

### Discovery Sweeps (R4, R5, R7b, R7d)
- `research/discovery-sweep-R4.md` -- CRDTs, DAGs, MaAS, PRMs, CLAI, AgentOps, inter-agent attacks, provenance tracking (96 papers screened)
- `research/discovery-sweep-R5.md` -- Decentralized DAGs, MAS-squared, OSC CKMs, AWorld profiling, KB-aware routing, event-triggered consensus (974 papers screened)
- `research/discovery-sweep-R7b.md` -- RL puppeteer orchestration, AgentAsk clarification, trust calibration, adversarial robustness gaps (948 papers screened)
- `research/discovery-sweep-R7d.md` -- PrefillShare, multiparty session types in Rust, biomimetic immunity, game-theoretic mechanism design, infectious jailbreaks (Agent Smith), AdaptOrch topology routing

### Key Individual Papers
- Huang et al. 2025 -- MAST failure taxonomy (ArXiv 2503.13657, 1,642 traces, 14 failure modes)
- Li et al. 2025 -- SagaLLM (ArXiv, 41 citations, saga patterns for LLM workflows)
- Xie et al. 2025 -- AWorld Profile-Aware Maneuvering (ArXiv 2508.09889, 57.4% variance reduction)
- Zhang et al. 2025 -- OSC Collaborator Knowledge Models (EMNLP Findings 2025, 12.6% redundancy)
- Agrawal & Nargund 2025 -- MetaOrch fuzzy evaluation (ArXiv 2505.02861, 86.3% accuracy)
- Hu et al. 2025 -- Randomized Smoothing for MAS robustness (ArXiv, probabilistic safety guarantees)
- Li et al. 2025 -- AgentAsk clarification modules (ArXiv 2510.07593, edge-level error mitigation)
- Decentralized Multi-Agent Swarms -- Consensus-Based Threat Validation (ArXiv 2601.17303)
- Hayashibara et al. 2004 -- Phi Accrual Failure Detector (foundational)
