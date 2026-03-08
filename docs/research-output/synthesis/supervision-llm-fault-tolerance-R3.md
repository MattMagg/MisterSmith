---
version: R3
created: 2026-03-07
updated: 2026-03-07
sources: Ultra2x (3 reports) → Synthesized
round: 3 (Triple Synthesis)
---

# Supervision & LLM Fault Tolerance: Comprehensive Research Synthesis

## Executive Summary

Integrating Large Language Models into multi-agent orchestration frameworks introduces a paradigm shift in fault tolerance. Unlike local processes that fail deterministically and instantly, LLM APIs are long-running, non-deterministic external dependencies prone to semantic failures, rate limits, silent streaming stalls, and degenerate output loops. For Mister Smith, achieving architectural superiority over Python-based frameworks requires extending its Erlang/OTP-style supervision trees, NATS/JetStream infrastructure, and Phase 2/3 primitives (PhiAccrualFailureDetector, CircuitBreaker, HealthMonitor) to handle these novel failure modes.

This synthesized report merges findings from three independent research efforts -- covering Erlang/OTP process patterns, distributed systems resilience, service-mesh routing, chaos engineering, safety-critical systems, and LLM-specific failure taxonomies -- into a single authoritative blueprint. The following five conclusions emerged with **high confidence** (independently converged upon by all three reports):

1. **Isolate LLM I/O from supervisors.** Blocking long-lived GenServer/actor processes on 30-second LLM generations will paralyze the supervision tree. All three reports converge on spawning short-lived, unlinked async tasks under a `Task.Supervisor`-equivalent, with one-shot channels (EEP-53 style aliases or `tokio::sync::oneshot`) to cleanly discard late replies without poisoning the agent's mailbox.

2. **Deploy stateful Gatekeeper actors over naive retries.** All reports agree that blind exponential backoff exacerbates structural LLM failures (token exhaustion, content filtering, auth errors). A supervised `Gatekeeper` actor per provider -- combining token-bucket rate limiting with a stateful circuit breaker (Erlang `fuse`-inspired) -- ensures fail-fast behavior during outages and prevents billing spikes from futile retries.

3. **Classify failures structurally.** All three reports independently produce nearly identical failure-mode taxonomies: transient errors (429, 5xx) warrant retries; structural errors (400, 401, 403, content filtering) require escalation or fallback; streaming-specific failures (partial drops, stale SSE) need checkpoint-and-resume; and semantic failures (hallucination loops) demand output monitoring and model switching. Treating all failures as transient network errors leads to restart storms.

4. **Checkpoint agent state durably.** All reports converge on event-sourcing agent state to NATS JetStream append-only logs, enabling LangGraph-style "time-travel" debugging, deterministic replay, and exactly-once execution via `Nats-Msg-Id` deduplication. This eliminates silent data loss from mid-workflow crashes.

5. **Use role-aware supervision, not blanket restarts.** All reports agree that `one_for_all` is too aggressive for agent teams. Executors should be transient (`one_for_one`), Planners should be permanent with escalation semantics, and Critics should support quorum or replacement strategies. Saga-pattern compensating transactions enable graceful model downgrades without destroying the entire workflow.

Beyond these convergent findings, the reports offer complementary depth on predictive health detection (Phi-accrual adaptation for Inter-Token Latency), service-mesh-style P2C+EWMA routing, bulkhead isolation, manager-worker patterns, ML/RL-based adaptive failover, and comprehensive chaos testing strategies.

---

## Table of Contents

1. [Supervising External Service Calls in OTP](#1-supervising-external-service-calls-in-otp)
2. [LLM-Specific Failure Modes](#2-llm-specific-failure-modes)
3. [Self-Healing Agent Topologies](#3-self-healing-agent-topologies)
4. [Graceful Degradation in Multi-Agent Systems](#4-graceful-degradation-in-multi-agent-systems)
5. [Checkpoint and Recovery for Long-Running Workflows](#5-checkpoint-and-recovery-for-long-running-workflows)
6. [Provider Health Tracking and Predictive Failure Detection](#6-provider-health-tracking-and-predictive-failure-detection)
7. [Supervision Strategies for Agent Teams](#7-supervision-strategies-for-agent-teams)
8. [Testing Fault Tolerance](#8-testing-fault-tolerance)
9. [Synthesis: The Optimal Fault Tolerance Architecture](#9-synthesis-the-optimal-fault-tolerance-architecture)
10. [Evidence Gaps](#10-evidence-gaps)
11. [References](#11-references)

---

## 1. Supervising External Service Calls in OTP

### 1.1 The Danger of Blocking GenServer Calls

**[HIGH CONFIDENCE -- all three reports converge]**

In traditional Erlang/OTP, a `gen_server` processes messages sequentially. If an actor makes a synchronous, blocking HTTP call to an LLM provider that takes 30 seconds, it cannot process other messages, including supervision signals [A1][B2][B10]. This leads to cascading timeouts and supervisor deadlocks. Furthermore, if a timeout occurs but the external service eventually responds, the late reply can sit in the actor's mailbox, acting as a "poison pill" that corrupts future state [A2].

OTP design guidance is explicit: long init work should be deferred using `handle_continue` or `self()` messaging to avoid blocking supervisor start [B1][B2][B10]. Processes relying on external services should *expect* failures and handle them gracefully -- returning `{error, Reason}` rather than crashing [C14].

### 1.2 Dynamic Task Supervision and EEP-53 Aliases

External LLM calls must be isolated into short-lived worker tasks. Three complementary patterns emerge from the research:

**Task.Supervisor pattern:** Spawn unlinked, supervised background tasks for each LLM call [A11][B38][B40][B85]. In Rust, frameworks like `ractor` provide similar dynamic supervision capabilities [A12]. Tasks should be configured as `temporary` or `transient` -- restarting a transient process whose context is gone makes no sense [C3].

**EEP-53 process aliases:** To handle the "poison pill" scenario, Mister Smith should adopt the pattern from Erlang's EEP-53 [A2]. By passing a temporary alias (or a one-shot `tokio::sync::oneshot` channel in Rust) to the LLM worker task, the supervisor can safely drop the alias upon timeout. If the LLM eventually responds, the message routes to a dead-letter queue rather than poisoning the agent's mailbox.

**Manager-worker pattern:** One report introduces the Adopting Erlang "manager-worker" pattern: a *manager* process sits above a temporary supervisor. The manager monitors failure counts and implements backoff or circuit-breaker logic. It checks if a subtree (making external calls) has failed too often; if so, it delays or stops restarting it and optionally triggers exponential backoff, circuit-breaker untrip, or admin override [C14]. This lets you treat the manager as a brain around the supervisor to implement non-trivial retry logic.

### 1.3 The Supervised Circuit Breaker Pattern

**[HIGH CONFIDENCE -- all three reports converge]**

Circuit breakers are often implemented as passive middleware, but all reports agree they are best modeled as active, supervised processes in an OTP-style system. Erlang's `fuse` library demonstrates this by running the circuit breaker as a standalone state machine that tracks failure intensities and manages persistent cooldowns [A4][B4][B5][B12][B63][B68].

By implementing the circuit breaker as a supervised Rust actor, Mister Smith can ensure that the breaker itself is fault-tolerant and can broadcast state changes (Open, Half-Open, Closed) across the cluster via NATS.

**Important caveat:** `fuse` stores counters in ETS and loses them on restart [B4][B18]. Mister Smith must design for this tradeoff -- either accept counter loss (simple, fast) or add durable backing to JetStream for counter persistence (more complex, survives restarts).

### 1.4 Applicability to Mister Smith

- Map `Task.Supervisor` to `ractor`-style dynamic supervisors or Tokio task spawning with `JoinHandle` monitoring
- Implement EEP-53 aliases as `tokio::sync::oneshot` channels with `tokio::time::timeout` wrappers
- Implement the `CircuitBreaker` as a dedicated actor publishing state to NATS
- Adopt the manager-worker pattern for subtrees that make external LLM calls
- Quantified impact: service-mesh studies show circuit-breaker + bulkhead patterns achieve 91.3% blast-radius reduction [B116]

**Implementation Complexity:** Moderate. Requires careful design of one-shot channels for LLM futures and correct cancellation semantics for in-flight streams.

**Expected Impact:** Eliminates supervisor blocking and mailbox poisoning, ensuring the control plane remains responsive during severe API degradation.

---

## 2. LLM-Specific Failure Modes

### 2.1 Failure Taxonomy

**[HIGH CONFIDENCE -- all three reports produce nearly identical taxonomies]**

LLM APIs fail in ways fundamentally different from traditional REST services. Treating all failures as transient network errors leads to restart storms and massive billing spikes. The following comprehensive taxonomy merges all unique failure modes identified across the three reports:

| Failure Mode | HTTP Code | Classification | Characteristics | Optimal Supervisory Strategy |
|:---|:---|:---|:---|:---|
| **Rate Limiting** | 429 | Transient (degrades to structural if persistent) | Provider throttling; includes `Retry-After` headers [A13][B23][B36][B37] | Suspend actor; retry strictly honoring `Retry-After` delay with jitter. Track Retry-After exposure as health signal. Circuit-breaker opens on repeated 429s. Do not use blind exponential backoff. |
| **Server Errors** | 500, 502, 503 | Transient | Provider internal errors [A3][B24][C16] | Retry with exponential backoff + jitter; circuit-breaker increments; on open, supervisor triggers failover. |
| **Gateway Timeout** | 504 | Transient (short) | Usually transient; single retry recommended [B25] | Supervise as server error. |
| **Token/Budget Exhaustion** | 402 / 429 | Structural | Hard quota hit; retries will continuously fail [A14][B31] | Trip circuit breaker immediately. Trigger structural failover to secondary provider. Mark provider unusable until quota resets. |
| **Context Window Overflow** | 400 | Semi-structural | Conversation history exceeds model limit [C26] | Catch error, truncate or summarize history, retry once. Not fixed by waiting, but recoverable with adaptation. |
| **Invalid Request** | 400 | Structural | Malformed inputs, invalid parameters [B26][B27] | Worker returns deterministic failure to planner. No automatic restart. Escalate to planner to recompose context. |
| **Authentication Failure** | 401 / 403 | Structural | Invalid API key or expired credentials [B28] | Rotate API keys or alert on-call. Do not retry blindly. Log and crash so operator notices. |
| **Content Filtering** | 400 / 403 | Structural | Prompt or output violates safety policies [A15][B29] | Escalate to Planner to rewrite prompt, or escalate to human-in-the-loop. Retries are futile. Dead-letter the request. |
| **Model Deprecation** | 404 / specific error | Structural | Model removed or unavailable [B8][B20] | Detect via specific errors; failover to alternate model/provider. Update provider capability registry. Escalate operator action. |
| **Partial Stream Drop** | 200 (Truncated) | Streaming-specific | SSE connection drops mid-generation [A16][B21][B22][B6][B7] | Checkpoint received tokens. Supervise streaming client with recv_timeout. Resume generation by passing partial response back to LLM if supported, or restart from checkpoint. |
| **Stale SSE Connection** | 200 (No data) | Streaming-specific | Connection established but no tokens arrive | Monitor with adapted Phi-accrual on inter-token latency. Sever connection proactively and failover. |
| **Hallucination Loop** | 200 (Degenerate) | Semantic/Algorithmic | Model repeats phrases infinitely; syntax valid but semantics fail [A7][B30][B104][B20] | Monitor token entropy / repetition. Cap iterations (e.g., 10). Terminate generation and fallback to different model family. |
| **Unbounded Tool-Call Loops** | N/A | Semantic/Algorithmic | Recursive tool calls without convergence [B30] | Enforce iteration caps by supervisor or planner. Treat repeated failures as structural; escalate. |
| **Timeout / Network Error** | N/A | Transient | Connect/read timeout, DNS failure [C26] | Retry with backoff. Use Phi-accrual on response latencies to flag degradation. |

### 2.2 Proactive Rate Limit Management

Rate limits must be managed proactively rather than reactively. Implementing a token-bucket rate limiter (like FluxNinja Aperture) within the `Gatekeeper` actor ensures outbound requests stay synchronized with the provider's TPM/RPM limits, queuing requests rather than hitting 429s [A17]. The `Gatekeeper` can parse OpenAI/Anthropic headers (e.g., `x-ratelimit-reset-tokens`, `Retry-After`) to dynamically adjust its internal token bucket [A13][A18].

### 2.3 Failure-Mode to Supervision/Recovery Policy Matrix

Synthesized from all three reports into a compact decision matrix:

- **Transient (retryable):** 5xx, 504, brief network/TLS faults -- transient worker retry with exponential backoff + jitter; circuit-breaker increments; on open, supervisor triggers failover or degradation
- **Rate-limit sensitive:** 429 -- obey Retry-After; treat repeated 429 as transient-to-structural if persists; circuit-breaker + provider health marking and failover
- **Structural (no retry):** 400 invalid/context-too-long, 401/403 auth, content-filter -- worker returns deterministic failure to planner; no automatic restart; escalate or apply corrective transformation (truncate, rotate key)
- **Streaming-interrupt:** Partial streaming/stale SSE -- supervise streaming client with recv_timeout and dedicated stream restarts; resume using stream offsets/checkpoints if provider supports
- **Semantic/algorithmic:** Hallucination or degenerate loops -- bounded retries, run Critic checks, degrade capability (cheaper model or simpler plan) and escalate

**Implementation Complexity:** Medium-high. Correctly classifying errors and encoding structured failure policies requires careful mapping of provider error semantics and streaming/cancellation design.

**Expected Impact:** Prevents infinite retry loops on 400-level errors, saving significant token costs and reducing latency. Structured classification + supervised circuit-breakers + failover reduces wasted retries and prevents cascading retries during provider outages.

---

## 3. Self-Healing Agent Topologies

### 3.1 Dynamic Failover and Topology Reconfiguration

When a primary LLM provider degrades, the supervision tree must dynamically restructure the agent graph to route traffic to healthy alternatives without manual intervention.

**[HIGH CONFIDENCE]** All three reports agree on the core principle: supervisors should be able to terminate failing provider children and spawn new children bound to alternative APIs at runtime. OTP supervisors support dynamic `start_child`/`delete_child`/`terminate_child` operations [B11][B42][B45].

### 3.2 P2C + EWMA Load Balancing

One report contributes a sophisticated routing algorithm drawn from service-mesh architecture. Mister Smith should borrow from Envoy's "Power of Two Choices" (P2C) combined with an Exponentially Weighted Moving Average (EWMA) of latency [A19][A20]:

1. Randomly select two providers from the healthy pool
2. Compare their EWMA latency and error rates
3. Route to the better one

This naturally shifts traffic away from degrading models before they fully fail, providing sub-millisecond failover decisions.

### 3.3 Outlier Detection and Penalty Boxes

Drawing from Istio's `DestinationRule` outlier detection [A21], implement a "penalty box" mechanism [A22]:

- If a provider returns a configurable number of consecutive 5xx or gateway errors (e.g., 5 errors), eject it from the routing pool for a base ejection time (e.g., 30 seconds)
- Subsequent ejections multiply the cooldown exponentially, preventing flapping
- Re-admit providers via half-open probing after the cooldown expires

### 3.4 Distributed Policy Sync via NATS

**[HIGH CONFIDENCE -- all three reports converge on NATS for health coordination]**

All agents in the cluster must share the same view of provider health. The research converges on a two-tier approach:

- **NATS subjects (ephemeral):** Fast local health broadcasts for immediate supervisor decisions. Local supervisors subscribe and make fast-path decisions.
- **JetStream durable subjects:** Record longitudinal health events and provider state changes (open/close circuit, quota exhaustion) for global supervisors, auditing, and replay [B17][B66][B67][B69].

The `Router` actor subscribes to NATS KV watches to update its local P2C+EWMA routing tables in real-time [A23].

### 3.5 Adaptive Failover: Heuristics vs ML/RL

Two reports discuss the spectrum of adaptive strategies:

- **Heuristics (recommended starting point):** Thresholds on failure counts, p95/p99 latency, Retry-After exposure, and token-usage velocity. Low complexity, effective in practice [B30][B31][B32].
- **ML/RL augmentation (advanced):** Research experiments show RL can improve repair success and MTTR, but requires training infrastructure and has nontrivial complexity [B30][B31][B32]. Consider only after heuristic-based system is proven.

### 3.6 Closed-Loop Architecture Benefits

One report cites MAS-FIRE (2026) research finding that *iterative, closed-loop architectures* (with feedback through critic agents and repeated planning) recover from over 40% of faults that break linear workflows [C35]. This implies agent teams designed with feedback loops are inherently more fault-tolerant than one-shot pipelines -- a strong argument for Mister Smith's Critic agent role.

**Implementation Complexity:** Medium-high. P2C+EWMA math is straightforward, but distributed state synchronization via NATS KV watchers and dynamic supervisor restructuring requires careful handling of race conditions and idempotent operations.

**Expected Impact:** Sub-millisecond failover and optimal cost/latency routing across multiple LLM providers. Automated topology changes reduce time-to-recovery and lower human intervention.

---

## 4. Graceful Degradation in Multi-Agent Systems

### 4.1 Fail-Operational Design

**[HIGH CONFIDENCE -- all three reports converge]**

In safety-critical domains like aviation, systems are designed to be "fail-operational" -- if a primary component fails, the system continues operating at reduced capacity rather than crashing completely [A24][B114]. All reports agree this principle must be applied to LLM agent systems.

### 4.2 Supervisor-Enforced Bulkheads

**[HIGH CONFIDENCE]**

All reports converge on the Bulkhead pattern [A25][B86][B93][C58]:

- Partition the agent system so failures are contained
- Run expensive model calls (e.g., GPT-4) in separate actor pools from cheaper calls (e.g., GPT-4o-mini)
- Supervisors enforce strict concurrency limits on the `DynamicSupervisor` managing LLM tasks
- If Provider A's bulkhead is full, requests are immediately rejected (fail-fast) or routed to Provider B
- Use per-agent token budgets and quotas to cap cost and prevent runaway execution [B81]

This ensures that one slow or failing LLM provider cannot exhaust all system threads, protecting the rest of the agent swarm [A26].

### 4.3 Progressive Simplification and Sagas

When an agent fails, restarting the entire workflow is expensive. The Saga pattern [A27] provides a structured alternative:

1. **Model downgrade:** If an Executor fails to complete a complex task using a reasoning model (e.g., o1), the supervisor triggers a fallback to a cheaper, faster model (e.g., GPT-4o-mini) [A7]
2. **Compensating transactions:** If the task fundamentally cannot be completed, the Saga executes compensating transactions to undo external side-effects (e.g., deleting a drafted email) and escalates back to the Planner to generate a simplified plan [A8]
3. **Multi-tier plans:** The Planner produces alternate plans for lower-cost models, partial results, or shorter horizons. The Coordinator supports dynamic replanning when Executors fail [B76][B99][B106]

Map Erlang's `rest_for_one` strategy to Saga compensations. Publish degraded-plan decisions to JetStream for traceability and possible human review.

### 4.4 Capability Degradation Levels

One report introduces explicit service-level tiers:

- **Full mode:** All agents, best models, complete tool access
- **Degraded mode:** Fewer examples, smaller models, simplified planning
- **Minimal mode:** Templates, cached answers, or direct human escalation

Each agent should know its "reduced mode": Planner can skip optional branches, Executor can use cached completions, Critic can simplify validation.

### 4.5 Resource Shedding

When resource limits are hit (token budget, latency targets), the system can shed non-critical tasks:

- Skip optional Critic steps
- Delay less urgent queries
- Limit concurrency so a flood of requests to one agent cannot deplete the whole system
- Communicate degraded status downstream so agents expect simpler inputs

**Implementation Complexity:** High. Requires developers to write explicit compensation logic for every tool/action and design multi-tier plan generation.

**Expected Impact:** Transforms catastrophic workflow failures into minor UX degradations, maintaining service continuity. Bulkheads plus degraded planning preserve core functionality when a provider becomes partially or fully unavailable.

---

## 5. Checkpoint and Recovery for Long-Running Workflows

### 5.1 Event Sourcing the Agent State

**[HIGH CONFIDENCE -- all three reports converge]**

LLM agents are non-deterministic and can run for minutes or hours. If a node crashes mid-execution, losing the context window is unacceptable. All reports converge on implementing "durable execution" by saving structured state at every super-step, modeled as an event-sourced append-only log [A10][A28][A29][B124][B125][C44][C46].

The checkpoint schema must contain:

- `workflow_id` and `checkpoint_id` / `step_id`
- `schema_version` (for compatibility enforcement)
- Current state values (conversation history, tool outputs)
- Provider cursor (stream offset or chunk ID)
- Model ID and tokens consumed so far
- Pending tasks and deterministic seeds
- Step-level success criteria and side-effects
- `resume_hint` for recovery guidance
- Timestamp

### 5.2 JetStream as Checkpoint Backend

**[HIGH CONFIDENCE]**

All reports agree NATS JetStream is the ideal backend for checkpoint storage:

| JetStream Parameter | Recommended Setting | Rationale |
|:---|:---|:---|
| **Storage** | `FileStorage` | Ensures checkpoints survive complete cluster restarts [A33] |
| **Retention** | `LimitsPolicy` | Retain by age (e.g., 7 days) to allow time-travel debugging [A34] |
| **AckPolicy** | `AckExplicit` | Ensures the agent fully commits the next step before acknowledging [A35] |
| **MaxDeliver** | `5` | Caps infinite retry loops on poisoned tasks [A36] |
| **Subject pattern** | `js.checkpoint.<workflow_id>` | Per-workflow streams for isolation [B17][B66] |

**Replay Semantics:** If an agent crashes, a new worker can pull the last checkpoint and resume execution exactly where it left off [A28][B17][B66][B67][B69]. The current state is the sequence of events so far; on failure, rehydrate state by replaying the log up to the last good event.

**Idempotency:** To prevent duplicate external API calls during a replay, use JetStream's `Nats-Msg-Id` header for deduplication (e.g., a 2-minute sliding window) [A9][A32].

### 5.3 Atomic Multi-Agent Checkpointing

For multi-agent atomic transitions (e.g., when multiple Executors need to commit a coordinated state), use a prepare/commit pattern recorded to JetStream WAL. This is analogous to two-phase commit from streaming frameworks like Flink and Spark [B130][B136]:

1. **Prepare phase:** All participants write their pending state
2. **Commit phase:** Coordinator writes commit record after all prepare records are present
3. **Recovery:** On crash, replay WAL and either complete or abort based on commit record presence

### 5.4 Schema Versioning

One report emphasizes a critical constraint: changing state schema between restarts is disallowed in stream processing systems [B126]. Checkpoint schemas must be versioned with strict compatibility rules enforced at deserialization time. Breaking schema changes require migration paths.

### 5.5 Savepoints and Branching (Time-Travel)

LangGraph demonstrates production-ready "time-travel" debugging [C44]:

- Mark savepoints at key milestones (after each agent turn)
- Allow branching: resume from a savepoint and alter inputs (exploiting non-determinism)
- Record metadata (thread ID, checkpoint ID) to enable forking execution
- Download failure traces and replay deterministically in CI

### 5.6 Checkpoint API (Concrete)

```
checkpoint_write(workflow_id, step_id, checkpoint_payload) -> seqno
checkpoint_read(workflow_id, since_seqno) -> stream cursor / resume_hint
prepare_commit(workflow_id, participants) -> prepare_id
commit(prepare_id) -> commit_seqno
```

**Implementation Complexity:** High. Requires strict separation of pure logic from side-effects to ensure safe replays. Designing compact, versioned checkpoint formats and ensuring efficient checkpoint frequency (tradeoff between performance and recovery time) is complex; frequent checkpoints degrade runtime performance [B138].

**Expected Impact:** Zero silent data loss, deterministic debugging, and the ability to pause/resume workflows for human-in-the-loop approvals. JetStream consumer offsets provide proven resume capabilities.

---

## 6. Provider Health Tracking and Predictive Failure Detection

### 6.1 The Case for Predictive Detection

**[HIGH CONFIDENCE]**

Reactive circuit breakers trip *after* errors occur. For LLMs, where a single request can take 30 seconds to fail, predictive detection is required. All reports agree that combining proactive health signals with reactive breakers is essential.

### 6.2 Adapting Phi-Accrual to LLM Latency

Mister Smith's existing `PhiAccrualFailureDetector` (based on Hayashibara 2004) calculates suspicion based on the normal distribution of heartbeat inter-arrival times [A5][A37][C49]. This must be adapted for LLMs by tracking two key streaming metrics:

1. **Time to First Token (TTFT):** The initial connection latency
2. **Inter-Token Latency (ITL):** The speed of the streaming response [A6]

By feeding ITL samples into the Phi-accrual math, the detector can identify when a stream is silently stalling. If the Phi value crosses a threshold (e.g., Phi > 8), the system can proactively sever the connection and failover to a backup model before the standard HTTP timeout is reached [A37].

**Note (evidence gap):** The provided evidence does not specify concrete phi-accrual parameter adaptation methods for LLM latency/streaming signals. Parameterization guidance requires empirical testing with real provider latency distributions [B-gap].

### 6.3 Composite Health Scoring

Health should be a composite metric combining multiple signals. The union of all tracked metrics from the three reports:

| Metric | Description | Source |
|:---|:---|:---|
| Latency p50/p95/p99 | Response time percentiles | [B102][B103][B23] |
| Error rate | Fraction of requests returning errors (5xx/429/4xx) | [B23][B36] |
| Retry-After exposure | Count and duration of Retry-After headers received | [B23][B36][B37] |
| Token consumption velocity | Rate of token usage vs budget | [B23] |
| Streaming stalls | Count of stale connections / partial chunk failures | [B23] |
| TTFT distribution | Time to first token statistical properties | [A6] |
| ITL distribution | Inter-token latency statistical properties | [A6] |
| Phi suspicion score | Adapted phi-accrual value for response times | [A5][A37] |
| EWMA latency | Exponentially weighted moving average of response time | [A19][A20] |

Advanced implementations can use algorithms like Isolation Forests to detect anomalies in telemetry time-series data, identifying subtle degradation patterns [A38].

### 6.4 Health Data Flow Architecture

```
Provider Response Metrics
    |
    v
ProviderHealth Actor (per provider, supervised)
    |-- Aggregates p50/p95/p99, error rates, Retry-After, token velocity
    |-- Feeds adapted Phi-accrual detector
    |-- Computes EWMA latency
    |
    +-- Publishes to NATS (ephemeral): ms.health.<provider>
    |       JSON: { provider_id, p50_ms, p95_ms, p99_ms, error_rate,
    |               retry_after_count, token_velocity, phi_score, timestamp }
    |
    +-- Persists to JetStream (durable): js.health.<provider>
            For historical records, auditing, and global supervisor decisions
```

### 6.5 External Health Signals

One report uniquely suggests subscribing to provider status pages or webhooks. When a provider issues warnings (e.g., "Scheduled maintenance at 3pm"), supervisors can proactively shift to alternatives before any failures occur [C-external].

### 6.6 Cross-Provider Correlation

If multiple customers see growing errors, escalate as a provider-wide issue. If only this system sees issues, investigate internal causes (network, credentials) [C-cross].

**Implementation Complexity:** Medium. Extending the existing Phase 2 `PhiAccrualFailureDetector` to accept ITL floats instead of just boolean heartbeats is moderate. Tuning the Phi threshold for high-variance LLM APIs requires empirical testing.

**Expected Impact:** Slashes tail latency by abandoning doomed requests early. Predictive detection turns occasional outages into smoother degradations. Preemptive strategy significantly reduces user-visible errors.

---

## 7. Supervision Strategies for Agent Teams

### 7.1 Strategy Selection

**[HIGH CONFIDENCE -- all three reports converge on role-aware supervision]**

When supervising a team of agents (Planner, Executor, Critic), blanket restart strategies are destructive. All reports agree on the following principles:

| Strategy | Use Case | Agent Team Application |
|:---|:---|:---|
| **OneForOne** | Independent agents | Executors handling separate tasks. If one Executor fails to call a tool, only that Executor is restarted or downgraded [A40] |
| **RestForOne** | Linear dependency chains | Sequential pipelines (Data Gatherer -> Analyzer -> Summarizer). If Analyzer crashes, Analyzer and Summarizer restart but Data Gatherer's expensive output is preserved [A40] |
| **OneForAll** | Strong coupling with shared state | Rare for agents. Only when partial state would be inconsistent. Too aggressive for most agent teams [A39] |

### 7.2 Role-Aware Restart Semantics

**[HIGH CONFIDENCE]**

All three reports independently converge on the same role-based taxonomy:

- **Executors (Transient):** Supervised under `one_for_one`. Spawn as transient workers in bulkhead compartments (resource-limited supervisors or thread pools). Can be restarted or replaced without planner restart. Do not restart on normal exit [A40][B76][B104][B105].

- **Critics (Quorum):** If a Critic fails, the system can use NATS to elect a new Critic or require a quorum of multiple smaller models to validate an output [A41]. Restartable but with stricter diagnostics before restart.

- **Planners (Permanent):** If the Planner fails, the workflow is fundamentally broken. Use permanent restart but with diagnostic inspection before auto-restart. If repeated restarts exceed threshold, gather last checkpoints, suspend the thread, record failure in JetStream, send alert, and escalate to a human operator via dead-letter queue [A7][B124][B125].

### 7.3 Hybrid Supervisor Trees

One report proposes a multi-level supervisor architecture:

1. **Inner supervisor:** Manages all Executors with `one_for_one` or `rest_for_one`
2. **Outer supervisor:** Manages the Planner and the inner Executor supervisor as siblings
3. If an Executor fails, the inner supervisor restarts it
4. If the inner supervisor's restart intensity is exceeded, the outer supervisor decides whether to restart the entire subtree (including Planner) or escalate

This gives maximum flexibility in handling cascading failures while preserving Planner state as long as possible.

### 7.4 Child Spec Metadata

Implement child spec metadata including:
- Role (Planner / Executor / Critic / Coordinator)
- Restart policy (permanent / transient / temporary)
- Resource quotas (token budget, concurrency limit)
- Provider affinity (preferred model/provider)
- Shutdown timeout (allow graceful completion vs immediate kill)

Supervisors use this metadata for role-aware restart decisions and dynamic `add_child`/`delete_child`/`restart_child` operations [B11][B42][B44][B45].

### 7.5 Quorum vs Eager Restart Tradeoffs

**[DIFFERENT PERSPECTIVES]**

Two reports offer contrasting views on quorum-based recovery:

- **Report B:** Quorum-based recovery (waiting for a set of healthy agents) increases safety for coordinated tasks but adds latency and complexity. Eager restart prioritizes availability but may hide systemic faults. Decentralized agent frameworks avoid single points of failure by local negotiation [B55][B22][B16].
- **Report C:** Quorum is more relevant when agents are parallel workers (like Kubernetes deployments with PodAntiAffinity). For most agent teams, simple restart strategies suffice.

**Recommendation:** Start with simple role-aware restarts. Add quorum semantics only for safety-critical coordinated outputs where consensus matters.

### 7.6 Deadlock Prevention

One report specifically calls out agent deadlock risks: inter-agent waits can cause deadlocks that were fixed in related systems by adding timeouts and orchestrator-owned leases [B120][B121]. Tests must verify that timeouts and leases hold under concurrent operation.

**Implementation Complexity:** Medium. Implementing role-aware supervisors is manageable but requires disciplined metadata and testing. Rust crates like `ractor-supervisor` natively support `OneForOne`, `OneForAll`, and `RestForOne` [A42].

**Expected Impact:** Maximizes stability and minimizes token waste by preserving healthy agent states during partial failures. Properly tuned restart limits prevent supervisor crashes and lower MTTR.

---

## 8. Testing Fault Tolerance

### 8.1 The Testing Imperative

**[HIGH CONFIDENCE]**

All three reports agree: fault tolerance mechanisms that are not continuously tested will fail in production. Three complementary testing approaches emerge.

### 8.2 Chaos Engineering and Failure Injection

Following Netflix Chaos Monkey and Gremlin principles [A43][A44][B33], Mister Smith must validate its steady-state hypothesis under stress. This requires building a Toxiproxy-style failure-injection adapter [A45] that sits between the framework and the LLM provider.

**Specific fault injection scenarios (union of all three reports):**

| Scenario | Purpose |
|:---|:---|
| HTTP 429 flood with varying `Retry-After` headers | Validate circuit-breaker opens and failover occurs |
| SSE streams that abruptly close mid-JSON payload | Validate checkpoint-and-resume for partial streams |
| Connections that establish but never return a first token (silent stalls) | Validate Phi-accrual ITL detection and proactive severing |
| Token/quota exhaustion responses | Validate ProviderHealth marks provider unusable and fails over |
| Model deprecation (404 / model-not-found) | Validate Planner receives structural error and switches model |
| Slow/poison responses (valid but extremely delayed) | Validate timeout enforcement and manager-worker backoff |
| Malformed content-filter rejections | Validate dead-lettering without retry |
| Inter-agent deadlock simulation | Validate timeouts and orchestrator-owned lease recovery [B120][B121] |
| Random child process kills | Validate supervisor restarts correctly (Supertester-style) [C67][C68] |

### 8.3 Property-Based Testing for Supervisors

Using Rust's `proptest` [A46], write property-based tests to verify supervisor invariants:

- A supervisor configured with `intensity = 5` and `period = 10` will *always* shut down if 6 failures occur within 10 seconds, preventing infinite hallucination loops [A47]
- "Planner restarts <= N in 5 minutes" [B11][B126]
- "Checkpoint stream always contains final step before worker exit" [B124]
- "Every request either succeeds or hits dead-letter queue, and never loops forever"
- "No matter which child crashes, the application eventually stabilizes"

### 8.4 Deterministic Record-and-Replay

Because LLMs are non-deterministic, debugging failures is notoriously difficult. By leveraging JetStream's append-only logs, Mister Smith can capture the exact sequence of events that led to a failure. This trace can be downloaded and replayed deterministically in CI to verify that the supervision tree correctly handles the edge case [A29][A48].

### 8.5 Supervised-Failure Tests

One report introduces specific OTP testing patterns from the Supertester library:

- `wait_for_process_restart` -- verify that after failures, the system arrives in a healthy state [C67]
- `assert_supervision_tree_structure` -- verify tree integrity post-failure [C67]
- `chaos_kill_children` -- randomly terminate children under a supervisor, checking the supervisor stays alive and children restart [C68]

### 8.6 Checkpoint/Resume Validation

Validate that:
- Checkpoint schemas are backward-compatible across versions [B126]
- Resume from checkpoint produces identical results to uninterrupted execution (for deterministic operations)
- Partial checkpoint writes do not corrupt recovery
- Schema version mismatches produce clear errors, not silent corruption

### 8.7 Staged Deployment

- **Unit/integration tests:** Verify handle_continue/deferred-init patterns, supervision behaviors, termination semantics
- **Fault injection (staging):** Full chaos test suite against mock providers
- **Production canary:** Deploy small percentage of traffic to new supervisors and circuit-breakers with Chaos Monkey-style injections to validate real-world resilience before wider rollout [B33][B13]

**Implementation Complexity:** High. Building a robust chaos harness requires significant engineering effort, but is essential for confidence.

**Expected Impact:** Catches regressions in recovery logic and builds immense operator confidence. Without testing, subtle failures in supervision logic can hide until production.

---

## 9. Synthesis: The Optimal Fault Tolerance Architecture

### 9.1 Architectural Layers

To achieve architectural superiority over Python-based frameworks, Mister Smith must synthesize OTP supervision, JetStream persistence, and service-mesh routing into a cohesive resilience layer organized in five tiers:

**Layer 1 -- The Execution Layer:**
LLM calls are executed as isolated, unlinked async tasks under a `DynamicSupervisor`. They communicate with the main agent actor via EEP-53 style one-shot channels, ensuring that timeouts cleanly sever the connection without poisoning the agent's mailbox. Each call enforces per-call timeouts and spawns streaming-aware async handlers that publish chunk events to the parent.

**Layer 2 -- The Gateway Layer:**
Every LLM provider is fronted by a supervised `Gatekeeper` actor. This actor maintains a token bucket to respect 429 `Retry-After` headers and houses a stateful Circuit Breaker. The circuit breaker emits state-change events to NATS for cluster-wide coordination. Consider durable backing for circuit-breaker counters to survive restarts.

**Layer 3 -- The Routing Layer:**
A `Router` actor uses P2C+EWMA to dynamically select the fastest healthy provider. It relies on a modified `PhiAccrualFailureDetector` that monitors Inter-Token Latency to proactively eject stalling providers into an exponential penalty box. Health state is synchronized via NATS KV watches.

**Layer 4 -- The Persistence Layer:**
At every super-step, the agent's state is checkpointed to a NATS JetStream `FileStorage` stream. If an agent crashes, a new worker claims the durable consumer, utilizing `Nats-Msg-Id` to ensure idempotent recovery without duplicating external side-effects. Schemas are versioned with strict compatibility rules.

**Layer 5 -- The Supervision Layer:**
Agent teams are supervised based on their roles. Executors use `OneForOne` restarts with progressive model downgrades (Saga compensations). Planners use permanent restart with escalation semantics. Critics support quorum or replacement. Bulkheads enforce resource isolation between teams.

### 9.2 Component Responsibilities

| Component | Lifecycle | Responsibilities |
|:---|:---|:---|
| **Planner** | Supervised, permanent | Generates multi-tier plans (full/degraded/minimal). Owns workflow-level checkpoint writing to JetStream. Coordinates retries, context truncation, and escalation. |
| **Executor** | Transient child tasks under TaskSupervisor | Spawns one worker per LLM call (HTTP/stream). Enforces per-call timeouts. Collects streaming chunks. Reports success/failure. Writes step-level checkpoints. |
| **CircuitBreaker** | Per provider, supervised | Maintains failure counts. Emits events when opened/closed/half-open. Offers ask/install/reset APIs. Optional durable backing for counters. |
| **ProviderHealth** | Per provider, supervised | Aggregates latency percentiles, error rates, Retry-After exposure, token velocity, Phi scores. Publishes transient health broadcasts to NATS and durable events to JetStream. |
| **Gatekeeper** | Per provider, supervised | Token-bucket rate limiting synchronized with provider TPM/RPM limits. Parses rate-limit headers dynamically. |
| **Router** | Singleton, supervised | P2C+EWMA routing decisions. Subscribes to NATS KV watches. Manages outlier detection penalty boxes. |
| **Coordinator / GlobalSupervisor** | Singleton, supervised | Listens to JetStream health events and NATS broadcasts. Executes topology changes (start/stop provider children, update routing). Records actions to JetStream for audit. |
| **Checkpoint Streams** | JetStream infrastructure | Per-workflow and per-provider durable append-only streams. Versioned schemas. Support prepare/commit for multi-participant atomic changes. |

### 9.3 NATS/JetStream Message Schema

| Subject | Type | Schema | Purpose |
|:---|:---|:---|:---|
| `ms.health.<provider>` | Ephemeral NATS | `{ provider_id, p50_ms, p95_ms, p99_ms, error_rate, retry_after_count, token_velocity, phi_score, timestamp }` | Fast local health decisions |
| `js.health.<provider>` | JetStream durable | Same as above | Historical records, auditing |
| `ms.cb.event.<provider>` | NATS + JetStream | `{ provider_id, state: open|closed|half_open, reason, fail_count, ts }` | Circuit-breaker coordination |
| `js.checkpoint.<workflow_id>` | JetStream append-only | `{ workflow_id, step_id, step_state, provider_cursor, model_id, token_usage, schema_version, ts, resume_hint }` | Durable workflow checkpointing |
| `ms.failover.<workflow_id>` | NATS request-reply | `{ workflow_id, failed_provider, suggested_provider, reason, ts }` | Failover coordination |
| `ms.failover.result.<workflow_id>` | NATS pub/sub | `{ workflow_id, decision, new_provider, ts }` | Failover acknowledgment |

### 9.4 Concrete Restart and Failover Algorithms

**Executor failure:**
1. `one_for_one` restart as transient child (up to configured `max_restarts` within window)
2. On repeated failures, circuit-breaker increments
3. If circuit-breaker opens, mark provider degraded
4. Invoke failover: Coordinator instructs Supervisors to start Executor children configured for alternate provider
5. Publish failover directive to NATS and record to JetStream

**Planner failure:**
1. Permanent restart with diagnostic inspection before auto-restart
2. Gather last checkpoints
3. If repeated restarts exceed threshold, suspend workflow
4. Record failure in JetStream and send alert
5. Optionally wait for operator/automated repair

**Provider degradation:**
1. ProviderHealth reports sustained degradation or circuit-breaker opens
2. Coordinator instructs Supervisors to start Executor children configured for alternate provider or model
3. If failover unavailable, Planner produces degraded plan tier
4. Write checkpoint noting degradation reason

### 9.5 Prioritized Implementation Roadmap

**MVP (low-medium effort, high impact):**

1. **Transient Executor workers** under a TaskSupervisor equivalent. Enforce per-call timeouts and spawn streaming-aware async handlers. *[Low complexity]*
2. **Supervised CircuitBreaker actor** per provider (Fuse-style). Simple in-memory counters plus NATS events for open/close. Supervisors consult circuit-breaker before starting Executors. *[Medium complexity]*
3. **ProviderHealth actor** aggregating p50/p95/p99 and error rates. Publish to NATS. Implement simple failover to alternate provider when circuit-breaker opens. Persist health events to JetStream. *[Medium complexity]*
4. **Per-workflow JetStream checkpoint stream** with basic checkpoint schema and resume capability for Executors and Planners. *[Medium complexity]*

**Phase 2 (higher complexity):**

5. **Role-aware supervisor logic** (Planner vs Executor vs Critic) with restart policies and restart-rate limiting. Dynamic add/delete child support for provider replacement. *[Medium-high complexity]*
6. **Bulkhead enforcement** via separate Executor supervisors with per-supervisor quotas and token budgets enforced by ProviderHealth/CircuitBreaker. *[Medium-high complexity]*
7. **Streaming resume semantics** (cursor handling) and robust streaming client wrappers with supervised cancellation and restart. *[High complexity]*
8. **P2C+EWMA routing** with Phi-accrual ITL adaptation and outlier detection penalty boxes. *[Medium-high complexity]*
9. **Saga-pattern compensation** for multi-step workflow failures with model downgrades. *[High complexity]*

**Advanced (research/operational cost):**

10. **Durable circuit-breaker state** (persist counters to JetStream) to avoid counter loss across restarts. *[Medium-high complexity]*
11. **ML/RL-based predictive failure detection** and adaptive failover policies. Requires training infrastructure. Evidence shows potential MTTR improvements but higher cost [B30][B31][B32]. *[High complexity]*
12. **Isolation Forest anomaly detection** on telemetry time-series for subtle degradation pattern identification [A38]. *[High complexity]*

---

## 10. Evidence Gaps

The following gaps were identified across the three research efforts:

1. **Phi-accrual parameterization for LLM:** The evidence does not specify concrete phi-accrual parameter adaptation methods for LLM latency/streaming signals. Parameterization guidance for adapting phi-accrual to high-variance LLM APIs requires empirical testing with real provider latency distributions.

2. **Canonical checkpoint schema:** No evidence source defines an exact canonical schema for JetStream checkpoint records. Recommended JSON field names are derived from synthesis across reports.

3. **Durable circuit-breaker design:** Detailed implementation patterns for durable circuit-breaker state (beyond noting the ETS limitation) are not specified in any source. Evidence notes Fuse/ETS loss but provides no recommended durable design [B4][B18].

4. **Quantitative LLM failover comparisons:** Empirical quantitative comparisons specific to LLM provider failover strategies (availability or MTTR numbers) are not present except for general service-mesh and multi-region availability figures (91.3% blast-radius reduction) [B116].

5. **Streaming resume protocol specifics:** Exact protocol for resuming a partial LLM stream (passing partial response back to provider) varies by provider and is not standardized. Implementation requires per-provider adapter logic.

6. **Token entropy monitoring algorithms:** Specific algorithms for detecting hallucination loops via token entropy or repetition scoring are not detailed in the evidence. This requires implementation research.

---

## 11. References

### Report A References

- [A1] *Type of calls allowed inside Erlang Process (sync/async, blocking/nonblocking) - Erlang Forums*. https://erlangforums.com/t/type-of-calls-allowed-inside-erlang-process-sync-async-blocking-nonblocking/1951
- [A2] *Erlang/OTP 24 Highlights - Erlang/OTP*. https://www.erlang.org/blog/my-otp-24-highlights/
- [A3] *Retries, Fallbacks, and Circuit Breakers in LLM Apps: A Production Guide*. https://www.getmaxim.ai/articles/retries-fallbacks-and-circuit-breakers-in-llm-apps-a-production-guide/
- [A4] *Fuse - circuit breaker library for Erlang*. https://github.com/jlouis/fuse
- [A5] *The Phi Accrual Failure Detector (Hayashibara 2004)*. https://dspace.jaist.ac.jp/dspace/bitstream/10119/4784/1/IS-RR-2004-010.pdf
- [A6] *How to Create Latency Monitoring (LLMOps)*. https://oneuptime.com/blog/post/2026-01-30-llmops-latency-monitoring/view
- [A7] *Error Recovery and Graceful Degradation in AI Agents*. https://notes.muthu.co/2026/02/error-recovery-and-graceful-degradation-in-ai-agents/
- [A8] *AI Agent Failures Are Distributed Systems Failures - DEV Community*. https://dev.to/arif/ai-agent-failures-are-distributed-systems-failures-heres-the-complete-mapping-216k
- [A9] *NATS JetStream Playbook: Exactly-Once, Minus the Bloat*. https://medium.com/@hadiyolworld007/nats-jetstream-playbook-exactly-once-minus-the-bloat-02fd9d5a051c
- [A10] *Persistence - LangGraph Docs*. https://docs.langchain.com/oss/python/langgraph/persistence
- [A11] *Task.Supervisor - Elixir Docs*. https://hexdocs.pm/elixir/Task.Supervisor.html
- [A12] *Ractor: not just another actor framework - r/rust*. https://www.reddit.com/r/rust/comments/113dp70/ractor_not_just_another_actor_framework/
- [A13] *Rate limits - OpenAI API*. https://developers.openai.com/api/docs/guides/rate-limits/
- [A14] *Error Codes - OpenAI Docs*. https://platform.openai.com/docs/guides/error-codes
- [A15] *Moderation - OpenAI Docs*. https://platform.openai.com/docs/guides/moderation
- [A16] *Responses API streaming - OpenAI Community*. https://community.openai.com/t/responses-api-streaming-the-simple-guide-to-events/1363122
- [A17] *Managing OpenAI API Rate Limits - FluxNinja Aperture*. https://docs.fluxninja.com/guides/openai
- [A18] *Rate limits - Claude API Docs*. https://platform.claude.com/docs/en/api/rate-limits
- [A19] *Supported load balancers - Envoy documentation*. https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/upstream/load_balancing/load_balancers
- [A20] *Adaptive Load Balancing Algorithm and Implementation - FAUN*. https://faun.pub/adaptive-load-balancing-algorithm-and-implementation-6f13ccb61bea
- [A21] *Istio Destination Rule*. https://istio.io/latest/docs/reference/config/networking/destination-rule/
- [A22] *Outlier detection - Envoy documentation*. https://www.envoyproxy.io/docs/envoy/latest/intro/arch_overview/upstream/outlier
- [A23] *JetStream KV Overview - NATS Docs*. https://docs.nats.io/jetstream/kv/overview
- [A24] *Designing for the Inevitable: Fail-Silent and Fail-Operational Patterns*. https://medium.com/@jusuftopic/designing-for-the-inevitable-fail-silent-and-fail-operational-patterns-explained-621db0232270
- [A25] *Isolate to Survive: Applying the Bulkhead Pattern in Microservices*. https://medium.com/@jusuftopic/isolate-to-survive-applying-the-bulkhead-pattern-in-microservices-a7f47f51249a
- [A26] *Best Practices for Designing Resilient Distributed Cloud Applications*. https://www.ijsat.org/papers/2025/1/2440.pdf
- [A27] *Saga Pattern - Microservices.io*. https://microservices.io/patterns/data/saga.html
- [A28] *Durable execution - LangGraph Docs*. https://docs.langchain.com/oss/python/langgraph/durable-execution
- [A29] *Event Sourcing - Martin Fowler*. https://martinfowler.com/eaaDev/EventSourcing.html
- [A30] *Consumers - NATS Docs*. https://docs.nats.io/nats-concepts/jetstream/consumers
- [A31] *How to Use NATS JetStream for Persistence*. https://oneuptime.com/blog/post/2026-01-26-nats-jetstream-persistence/view
- [A32] *Building a Durable Telemetry Ingestion Pipeline with Rust and NATS JetStream*. https://ricofritzsche.me/building-a-durable-telemetry-ingestion-pipeline-with-rust-and-nats-jetstream/
- [A33] *Streams - NATS Docs*. https://docs.nats.io/nats-concepts/jetstream/streams
- [A34] *JetStream Concepts: Streams - NATS Docs*. https://docs.nats.io/jetstream/concepts/streams
- [A35] *JetStream Model Deep Dive - NATS Docs*. https://docs.nats.io/using-nats/developer/develop_jetstream/model_deep_dive
- [A36] *Consumer Details - NATS Docs*. https://docs.nats.io/using-nats/developer/develop_jetstream/consumers
- [A37] *Phi Accrual Failure Detector - Akka Documentation*. https://doc.akka.io/libraries/akka-core/current/typed/failure-detector.html
- [A38] *Isolation Forest (Liu et al.)*. https://www.researchgate.net/publication/224384174_Isolation_Forest
- [A39] *Supervision Principles - Erlang Docs*. https://erlang.org/documentation/doc-4.9.1/doc/design_principles/sup_princ.html
- [A40] *Erlang Supervisor Behaviour*. https://www.erlang.org/docs/24/design_principles/sup_princ
- [A41] *Raft Consensus Algorithm*. https://raft.github.io/raft.pdf
- [A42] *ractor-supervisor - crates.io*. https://crates.io/crates/ractor-supervisor
- [A43] *Netflix Chaos Monkey*. https://netflix.github.io/chaosmonkey/
- [A44] *Chaos Engineering - Gremlin*. https://www.gremlin.com/chaos-engineering
- [A45] *Toxiproxy - Simulate network and system conditions*. https://news.ycombinator.com/item?id=37842301
- [A46] *An Introduction to Property-Based Testing in Rust - Luca Palmieri*. https://lpalmieri.com/posts/an-introduction-to-property-based-testing-in-rust/
- [A47] *Supervisor Behaviour - Erlang System Documentation v28.4*. https://www.erlang.org/doc/system/sup_princ.html
- [A48] *JetStream - NATS Docs*. https://docs.nats.io/nats-concepts/jetstream

### Report B References

- [B1] https://stackoverflow.com/questions/26809391/erlang-how-to-deal-with-long-running-init-callback
- [B2] https://www.erlang.org/doc/apps/stdlib/gen_server.html
- [B3] https://www.erlang.org/doc/apps/stdlib/supervisor.html
- [B4] https://github.com/jlouis/fuse
- [B5] https://rokkincat.com/blog/2015/09/24/circuit-breakers-in-elixir/
- [B6] https://ninenines.eu/docs/en/gun/2.2/manual/
- [B7] https://stackoverflow.com/questions/67739157/elixir-how-to-consume-a-stream-of-server-sent-events-as-a-client
- [B8] https://backendbytes.com/articles/llm-api-integration-patterns/
- [B9] https://portkey.ai/blog/retries-fallbacks-and-circuit-breakers-in-llm-apps
- [B10] https://hexdocs.pm/elixir/GenServer.html
- [B11] https://hexdocs.pm/elixir/1.12/Supervisor.html
- [B12] https://www.mojotech.com/blog/safeguard-web-service-failures-in-elixir-with-fuse/
- [B13] https://www.theimpactinstitute.org/Publications/Noorian-Hosseini-Ulieru-Autonomous.pdf
- [B14] https://thenewstack.io/three-stages-of-building-self-healing-it-systems-with-multiagent-ai/
- [B15] https://www.netbraintech.com/blog/self-healing-networks/
- [B16] https://hal.science/hal-01068045/file/POSTER_Self-Healing_Mechanisms_for_Software-Defined_Networks.pdf
- [B17] https://docs.nats.io/nats-concepts/jetstream
- [B18] https://rokkincat.com/blog/2015/09/24/circuit-breakers-in-elixir/ (circuit-breaker ETS loss on restart)
- [B19] https://medium.com/@hadiyolworld007/nats-jetstream-playbook-exactly-once-minus-the-bloat-02fd9d5a051c
- [B20] https://arxiv.org/abs/2511.19933 (LLM failure-modes taxonomy)
- [B21] https://aws.amazon.com/blogs/machine-learning/detect-hallucinations-for-rag-based-systems/
- [B22] https://online.stevens.edu/blog/building-self-healing-ai-orchestrator-reflexion-patterns/
- [B23] https://sarcouncil.com/download-article/SJMD-259-2025-333-339.pdf
- [B24] https://pmc.ncbi.nlm.nih.gov/articles/PMC12603247/
- [B25] https://agentpatterns.tech/en/failures/deadlocks
- [B26] https://www.kunalganglani.com/blog/multi-agent-ai-systems-production
- [B27] https://www.comet.com/site/blog/multi-agent-systems/
- [B28] https://esy.com/agents/patterns/planner-executor
- [B29] https://docs.databricks.com/aws/en/structured-streaming/checkpoints
- [B30] https://propelius.tech/blogs/checkpointing-in-stream-processing-best-practices
- [B31] https://15445.courses.cs.cmu.edu/fall2018/notes/20-logging.pdf
- [B32] https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html
- [B33] https://www.architecture-weekly.com/p/the-write-ahead-log-a-foundation
- [B36] https://oneuptime.com/blog/post/2026-02-16-how-to-handle-rate-limiting-and-throttling-in-azure-openai-api-calls/view
- [B37] https://developers.openai.com/cookbook/examples/how_to_handle_rate_limits/
- [B38] https://elixirforum.com/t/task-supervisor-with-max-restart-and-max-seconds/34108
- [B42] https://www.netbraintech.com/blog/self-healing-networks/
- [B44] https://docs.nats.io/nats-concepts/jetstream
- [B45] https://zilliz.com/glossary/nats
- [B49] https://online.stevens.edu/blog/building-self-healing-ai-orchestrator-reflexion-patterns/
- [B55] https://esy.com/agents/patterns/planner-executor
- [B63] https://oneuptime.com/blog/post/2026-02-16-how-to-handle-rate-limiting-and-throttling-in-azure-openai-api-calls/view
- [B66] https://stackoverflow.com/questions/52423061/how-to-use-gunopen-in-a-gen-server-module
- [B67] https://hexdocs.pm/hackney/news.html
- [B68] https://www.mojotech.com/blog/safeguard-web-service-failures-in-elixir-with-fuse/
- [B69] https://arxiv.org/pdf/2504.20093
- [B76] https://www.comet.com/site/blog/multi-agent-systems/ (planner-executor patterns)
- [B80] https://docs.databricks.com/aws/en/structured-streaming/checkpoints
- [B81] https://propelius.tech/blogs/checkpointing-in-stream-processing-best-practices
- [B85] https://elixirforum.com/t/task-supervisor-with-max-restart-and-max-seconds/34108
- [B86] https://www.kunalganglani.com/blog/multi-agent-ai-systems-production (bulkhead)
- [B88] https://docs.databricks.com/aws/en/structured-streaming/checkpoints
- [B91] https://hexdocs.pm/elixir/1.12/Supervisor.html (restart intensity)
- [B93] https://www.comet.com/site/blog/multi-agent-systems/ (isolation)
- [B95] https://esy.com/agents/patterns/planner-executor
- [B96] https://docs.databricks.com/aws/en/structured-streaming/checkpoints
- [B97] https://propelius.tech/blogs/checkpointing-in-stream-processing-best-practices
- [B99] https://www.comet.com/site/blog/multi-agent-systems/ (replanning)
- [B102] https://15445.courses.cs.cmu.edu/fall2018/notes/20-logging.pdf
- [B103] https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html
- [B104] https://propelius.tech/blogs/checkpointing-in-stream-processing-best-practices (role-aware)
- [B105] https://15445.courses.cs.cmu.edu/fall2018/notes/20-logging.pdf (role-aware restart)
- [B106] https://www.comet.com/site/blog/multi-agent-systems/ (coordinator)
- [B114] https://eajournals.org/bjms/wp-content/uploads/sites/21/2025/06/Cloud-Orchestration.pdf (aviation analogy)
- [B116] https://sarcouncil.com/download-article/SJMD-259-2025-333-339.pdf (91.3% blast-radius reduction)
- [B120] https://agentpatterns.tech/en/failures/deadlocks
- [B121] https://www.kunalganglani.com/blog/multi-agent-ai-systems-production (deadlock fixes)
- [B124] https://docs.databricks.com/aws/en/structured-streaming/checkpoints (checkpointing)
- [B125] https://propelius.tech/blogs/checkpointing-in-stream-processing-best-practices
- [B126] https://docs.databricks.com/aws/en/structured-streaming/checkpoints (schema versioning)
- [B127] https://propelius.tech/blogs/checkpointing-in-stream-processing-best-practices
- [B128] https://15445.courses.cs.cmu.edu/fall2018/notes/20-logging.pdf
- [B130] https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html
- [B132] https://www.architecture-weekly.com/p/the-write-ahead-log-a-foundation
- [B134] https://faculty.cc.gatech.edu/~jarulraj/courses/8803-s22/slides/06-logging-2.pdf
- [B136] https://codelabs.solace.dev/codelabs/solace-agent-mesh/?index=..%2F..index
- [B137] https://oneuptime.com/blog/post/2026-02-16-how-to-handle-rate-limiting-and-throttling-in-azure-openai-api-calls/view
- [B138] https://developers.openai.com/cookbook/examples/how_to_handle_rate_limits/ (checkpoint frequency tradeoff)

### Report C References

- [C3] OTP Supervisor design -- `restart: temporary` rationale
- [C9] GenServer trap_exit and worker supervision
- [C13] Erlang Supervisor strategies (one_for_one, rest_for_one, one_for_all)
- [C14] Adopting Erlang -- manager-worker pattern, graceful failure handling
- [C16] *LLM Error Handling Guides* -- rate limits, 5xx handling, backoff with jitter
- [C26] *LLM API Error Codes* -- context_length_exceeded, error classification
- [C28] *Fallback chains for LLM providers*
- [C35] *MAS-FIRE (2026)* -- iterative closed-loop architectures recover 40%+ of faults breaking linear workflows
- [C39] *PraisonAI* -- graceful degradation testing guidance
- [C44] *LangGraph time-travel debugging* -- checkpoint state at each node, resume from any prior point
- [C46] *LangGraph checkpointing and audit trail*
- [C49] *Phi Accrual Failure Detector* -- heartbeat-based detection in distributed systems
- [C58] *Bulkhead Pattern* -- isolate components so one failure doesn't sink the ship
- [C59] *JetStream replay semantics* -- durable log for agent workflow state
- [C67] *Supertester OTPHelpers* -- `wait_for_process_restart`, `assert_supervision_tree_structure`
- [C68] *Supertester chaos testing* -- `chaos_kill_children`, fault injection helpers
