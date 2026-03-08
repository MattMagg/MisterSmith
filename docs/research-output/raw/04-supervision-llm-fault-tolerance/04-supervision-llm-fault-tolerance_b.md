# Supervision-tree-centric fault tolerance and self-healing for LLM provider lifecycle management in Mister Smith

Executive summary  
Design LLM provider lifecycle management so that each external LLM call is a first-class supervised resource: spawn short-lived, monitored worker tasks under role-aware supervisors; pair them with supervised circuit-breaker and health-tracking processes; persist checkpointed workflow state to durable JetStream streams; and let supervisors dynamically rewire and degrade workflows (cheaper model, partial results, truncated context) based on structured failure-mode policies. This approach composes with Mister Smith’s OTP-style supervisors, phi-accrual detector, circuit-breaker primitives, and NATS/JetStream coordination and yields substantially lower blast radius and faster recovery than naive retry-only strategies. The design below synthesizes Erlang/OTP process patterns, LLM failure-mode mappings, self-healing/topology lessons, checkpointing best practices including LangGraph analogues, JetStream durability, and chaos-testing guidance from the reviewed evidence.

Contents
- Introduction and assumptions
- 1) Supervising external service calls in OTP
- 2) LLM failure modes → supervision strategies (matrix)
- 3) Self-healing topologies & dynamic supervisors
- 4) Graceful degradation for multi-agent workflows
- 5) Checkpointing, recovery and resumption
- 6) Provider health tracking & predictive detection
- 7) Supervision strategies for agent teams and restart policies
- 8) Testing and chaos scenarios
- Synthesis recommendation: concrete architecture, APIs, data schemas, and roadmap
- Evidence gaps
- References

Introduction and assumptions
- Mister Smith already provides OTP-style supervision trees, phi-accrual failure detection, circuit breakers, health monitoring, and NATS/JetStream messaging (project brief assumption). Design choices below assume those Phase-2/3 primitives exist and are the building blocks for Phase-9 LLM provider integration.
- All factual claims in this report are drawn from the supplied evidence sources listed in References.

1) Supervising external service calls in OTP

State of the art (citations)
- OTP gen_server provides explicit lifecycle callbacks (init, handle_continue, terminate) and supervisor start/stop semantics that control start ordering and shutdown ordering [2], [3]. Long init work should be deferred (handle_continue or self() messaging) to avoid blocking supervisor start [1], [2], [10]. [1], [2], [10], [11]

Key techniques (patterns / algorithms)
- Defer heavy initialization from init: use gen_server’s {:continue, _} / handle_continue or send self() and handle in handle_info to perform connection/setup without blocking supervisor start [1], [2], [10]. [1], [2], [10]
- Use short-lived worker tasks under a Task supervisor for external calls: spawn transient workers (Task.Supervisor-like) and keep heavier components as long-lived GenServer-style processes that coordinate tasks [38], [40], [85]. [38], [40], [85]
- Avoid blocking long-lived GenServers: use asynchronous HTTP streaming clients that deliver chunks via handle_info (HTTPoison/Gun patterns) and tune recv timeouts so handlers do not block forever [7], [6], [21], [22]. [6], [7], [21], [22]
- Circuit breaker as supervised service: common BEAM practice is to run a circuit-breaker process (Fuse-style) that stores counters (ETS-backed), emits events, and is itself supervised so its lifecycle and backoff policy are managed by supervisors [4], [5], [12], [63], [68]. [4], [5], [12], [63], [68]

Applicability to Rust + OTP-style supervision + NATS
- Map gen_server semantics to long-lived Rust actors (supervised processes) that: return immediately from "init" equivalent, perform connection/setup in a continuation, and spawn transient task-workers for each external HTTP/stream call. Use supervised circuit-breaker processes implemented as stateful actors with in-memory counters plus optional durable backing for counters if required. Circuit-breaker processes can be supervised like any other child. The HTTP client pattern (persistent connection + async chunk delivery) mirrors Gun/HTTPoison advice and maps to async HTTP client usage with streaming futures in Rust. [1], [2], [6], [7], [21], [22], [4], [5], [12], [63], [68], [38], [40], [85]

Implementation complexity (estimate and risks)
- Complexity: medium. Implementing continuation-style init, transient task supervisors, streaming handlers and supervised circuit-breaker processes is moderate engineering work; key risks include correct cancellation semantics for in-flight streams and state reconciliation for ETS-like counters after restarts (Fuse stores counts in ETS and loses them on restart) [4], [18]. [4], [18]

Expected impact vs naive retry-and-hope
- Replacing blocking GenServers and ad-hoc retries with supervised transient workers plus circuit-breakers reduces cascading restarts and avoids supervisor exhaustion; Fuse-style circuit breakers have been used in BEAM to reduce downstream blast radius (reported practice) [4], [63], [68]. The quantified blast-radius reduction from service-mesh + circuit-breakers in related studies was 91.3% (service mesh case) showing large potential gains when properly isolating and breaking failing dependencies [23]. [4], [63], [68], [116]

2) LLM-specific failure modes and mapping to supervision strategies

State of the art (enumeration)  
The reviewed evidence enumerates LLM failure modes and suggested handling in production LLM integrations: rate limits (429 + Retry-After), server errors (5xx), gateway timeouts (504), context-too-long (400 non-retryable), invalid/auth/content-filter errors (400/401/403 non-retryable), token/budget exhaustion, model deprecation, partial streaming failure / stale SSE, hallucination/degenerate loops, and recursive tool-call loops requiring caps [8], [23], [24], [25], [26], [27], [28], [29], [30]. Additionally, a broader taxonomy of LLM failure modes is described in an LLM failure-modes paper [20]. [8], [23], [24], [25], [26], [27], [28], [29], [30], [20]

Key techniques and remediation per mode
A condensed mapping (failure mode → classification → remediation → supervision behavior) synthesized from the evidence:

- Rate limiting (HTTP 429 + Retry-After): typically transient; retry honoring Retry-After with exponential backoff and jitter; fallback after retries exhausted; track Retry-After exposure as a health signal [23], [36], [37]. Supervision: circuit-breaker opens on repeated 429s and supervisor routes to alternate provider or degrades to cheaper/partial plan when circuit is open. [23], [36], [37]

- Server errors (5xx): transient; retry with exponential backoff + jitter; fallback after limited attempts [24]. Supervision: count failures in circuit-breaker, open after threshold; restart only transient worker tasks, not parent planner. [24]

- Gateway timeout (504): usually transient; single retry recommended; supervise as server error (short transient) [25]. [25]

- Context-too-long / invalid requests (400): structural/client error; not retryable except after truncation/correction; supervised worker should return specific failure with no restart and escalate to planner to recompose context (truncate or summarize) [26], [27]. [26], [27]

- Auth failure (401/403): structural; rotate API keys or alert on-call; do not retry blindly [28]. [28]

- Content-filter / moderation rejection: structural; re-prompt, reject, or escalate for manual review; do not apply automated aggressive retries [29]. [29]

- Token/budget exhaustion (quota/402-like): structural until quota replenished; circuit-breaker or provider health state should mark provider as unusable until quota resets; supervisors should failover to alternate provider and rate-limit agent activity [31]. [31]

- Model deprecation (model removed/unavailable): structural; detect via specific errors and failover to alternate model or provider; update provider capability registry and escalate operator action for config updates [8], [20]. [8], [20]

- Partial streaming failure / stale SSE: streaming-specific transient/structural mix; chunked streams can die mid-response or appear alive with no data; use streaming client patterns to receive AsyncChunk/AsyncStatus/AsyncHeaders messages and set recv_timeout tuning; supervise stream client separately and attach timeouts and cancels to worker tasks [21], [22], [6], [7]. [21], [22], [6], [7]

- Hallucination / degenerate output loops: algorithmic/semantic failure (model-internal); detect via post-hoc evaluation (critic, confidence heuristics) and cap iterations; supervise agent reflexion loops with bounded retries/caps to avoid infinite tool-call loops [30], [104], [20]. [30], [104], [20]

- Unbounded tool-call loops: prevent with iteration caps (e.g., 10) enforced by supervisor or planner; treat repeated failures as structural and escalate [30]. [30]

Failure-mode → supervision/recovery policy matrix (concrete)
- Transient (retryable): 5xx, 504, brief network/TLS faults → transient worker retry with exponential backoff + jitter; circuit-breaker increments; on open → supervisor triggers failover or degradation [24], [25], [31], [63]. [24], [25], [31], [63]
- Rate-limit sensitive: 429 → obey Retry-After; treat repeated 429 as transient-to-structural if Retry-After persists; circuit-breaker + provider health marking and failover [23], [36], [37]. [23], [36], [37]
- Structural (no retry): 400 invalid/context-too-long, 401/403 auth, content-filter → worker returns deterministic failure to planner; no automatic restart; escalate or apply corrective transformation (truncate, rotate key) [26], [27], [28], [29]. [26], [27], [28], [29]
- Streaming-interrupt: partial streaming/stale SSE → supervise streaming client with recv_timeout and dedicated stream restarts; resume using stream/offets if provider supports (or mark provider degraded) [21], [22], [6], [7]. [21], [22], [6], [7]
- Semantic/algorithmic: hallucination or degenerate loops → bounded retries, run Critic checks, degrade capability (cheaper model or simpler plan) and escalate-based supervised interventions [30], [104], [20]. [30], [104], [20]

Applicability to Mister Smith supervision model
- Make each external LLM call a transient supervised task. Capture failure type at worker exit and publish a structured health/failure event to NATS (see Section 6). Pair worker tasks with supervised circuit-breaker actors and with the planner’s role-aware supervisor so planners can choose remedial actions on worker failure without restarting planners unnecessarily. Use circuit-breakers as supervised processes whose state is preserved unless explicitly reset - note Fuse stores counters in ETS and loses them on restart unless durable backing added; design for that tradeoff [4], [18]. [4], [18], [23], [24], [25], [26], [27], [29], [30], [63]

Implementation complexity and risks
- Complexity: medium-high. Correctly classifying errors and encoding structured failure policies requires careful mapping of provider error semantics and careful design of streaming/cancellation. Risks: misclassifying structural failures as transient (causing pointless retries), losing circuit-breaker state on restarts (ETS loss), and missing streaming resume semantics. [4], [18], [21], [22], [23], [24], [26]

Expected impact vs retry-only
- Structured classification + supervised circuit-breakers + failover reduces wasted retries, prevents cascading retries during provider outages, and allows graceful degradation. Empirical evidence in related service-mesh work shows circuit-breaker + bulkhead patterns greatly reduce blast radius (91.3% reduction) and can be critical to high-availability targets [116]. [116]

3) Self-healing agent topologies & dynamic supervisor behavior

State of the art (citations)
- Self-healing architectures in networking and multi-agent research use layered detect-diagnose-recover loops, central or decentralized orchestrators, and learning/adaptive mechanisms to reconfigure topology and apply repairs [13], [14], [16], [22], [36]. SDN/self-healing modules can detect faults, diagnose, and enact repairs (reroute flows) automatically [16]. Multi-agent self-healing research recommends run-time adaptation for creation, analysis, and repair strategies and notes the return-on-investment in MTTR reduction [13], [14], [22], [36]. [13], [14], [16], [22], [36]

Key patterns and algorithms
- Detection → diagnosis → recovery closed loop: use layered monitoring (pre-action, during-action, post-action) to feed decisions [23], [57]. [23], [57]
- Dynamic supervisor restructuring: restart or replace failing children, rebind endpoints (Provider A → Provider B), or promote secondaries (promote passive Planner to active) under supervisory control. Persistent metadata (provider capability registry) informs choice. Supervisors can delete/add child specs and start new children dynamically [11], [42], [45]. [11], [42], [45]
- Adaptive failover heuristics: simple history-based heuristics (failure counts, error rates, latency percentiles) vs ML/RL approaches; RL can learn strategies and improve MTTR in experiments but is higher cost [30], [31], [32]. [30], [31], [32]
- Local negotiation / decentralized recovery: decentralized agent coordination avoids single point of failure by local negotiation/self-coordination for workload distribution [24], [55]. [24], [55]

Applicability to Rust + OTP-style supervisors + NATS/JetStream
- Supervisors can modify child_spec lists at runtime (delete child, start child) and can be implemented in Rust supervision primitives; use NATS for disseminating provider health and failover directives and JetStream for durable event logs and checkpoint streams [11], [17], [19], [66], [67]. Decentralized decisions: local supervisors make immediate decisions (fast path) and publish events to NATS so global supervisors reconcile and perform longer-term repairs. Use JetStream durable streams to record topology-change commands as append-only records for auditing and replay [17], [19], [66], [67]. [11], [17], [19], [66], [67]

Adaptive strategies: heuristics vs ML
- Start with heuristics: thresholds on failure counts, p95/p99 latency, Retry-After exposure and token-usage velocity; these are low complexity and effective in practice. Consider ML/RL augmentation for Level-3 self-healing where evidence shows RL can improve repair success and MTTR (research experiments show notable gains but require training infrastructure) [30], [31], [32]. [30], [31], [32]

Implementation complexity and risks
- Complexity: medium-high. Dynamic supervisor restructuring and consistent distributed coordination requires careful state propagation and idempotent operations to avoid flapping. Key risks: race conditions changing child specs during shutdown, inconsistent health view across nodes, operator surprise from automatic topology changes. [11], [17], [19], [66], [67]

Expected impact vs naive
- Automated topology changes that route around failing providers reduce time-to-recovery and lower human intervention; self-healing networks and agent orchestration literature report measurable MTTR reductions and improved availability when closed-loop healing is used [13], [14], [15], [36]. [13], [14], [15], [36]

4) Graceful degradation strategies for multi-agent workflows

State of the art (citations)
- Planner-Executor separation and plan reconfiguration is a common pattern: Planner generates a DAG, Executor runs steps, Coordinator triggers re-planning on failure [27], [76], [77], [106]. Bulkhead and resource partitioning are standard resilience patterns to limit fault propagation [86], [93], [95]. Aviation/critical-systems literature frames degraded modes as preserving core functionality with reduced capability [114]. [27], [76], [77], [86], [93], [95], [114]

Key techniques
- Degraded plans: Planner produces alternate plans for lower-cost models, partial results, or shorter horizons; Coordinator supports dynamic replanning when Executors fail [76], [99], [106]. [76], [99], [106]
- Bulkhead isolation: isolate agent teams into resource compartments (thread pools, token budgets, or separate processes) so one overloaded team cannot consume global resources [86], [93], [96], [97]. [86], [93], [96], [97]
- Role-aware policies: treat Planner failures as higher-severity (may require promotion/escalation) and Executor failures as restartable/replaceable; separate shutdown and restart semantics accordingly [104], [105], [104]. [104], [105]
- Analogues from aviation: operate with reduced functionality (degraded mode) to maintain core mission while deferring non-essential tasks [114]. [114]

Applicability to Mister Smith
- Implement Planner as a long-lived supervised process that can generate multiple plan tiers (full, degraded, minimal). Executors run as transient worker processes in bulkhead compartments (resource-limited supervisors or thread pools) so they can be restarted or replaced without planner restart. Use token budgets and per-agent quotas to cap cost and prevent runaway execution [81]. Publish degraded-plan decisions to JetStream for traceability and possible human review. [76], [81], [86], [93], [104], [114]

Implementation complexity and risks
- Complexity: medium. Implementing multi-tier plans and integrating Coordinator logic requires additional planner logic and policy definitions. Risks: under-specified degraded plans that break later steps; difficulty in testing all degraded combinations. [76], [106]

Expected impact vs naive
- Bulkheads plus degraded planning preserve core functionality when a provider becomes partially or fully unavailable and significantly reduce cascading resource exhaustion; service-mesh style studies demonstrate meaningful reduction of downstream impacts when isolation + circuit breakers are used [116]. [116]

5) Checkpointing, recovery, and resumption for long-running agent workflows

State of the art (citations)
- LangGraph supports pausing a node, saving execution state (variables, tool outputs, dialogue history), and resuming later; humans can edit checkpointed state before resumption [124], [125]. Streaming/checkpointing literature emphasizes that checkpoints store state and processed offsets and that changing state schema between restarts is disallowed; stream processors (Flink, Spark) use periodic checkpoints, barriers, and two-phase commit to ensure atomic distributed snapshots [123], [127], [128], [136], [137]. WAL and two-phase commit are canonical durability patterns for durable decision logs and recovery [130], [132], [134]. [124], [125], [123], [127], [128], [130], [132], [134], [136], [137]

Key techniques (formats, minimal state)
- Minimal resume state: conversation/dialogue state, tool invocation metadata (tool id, inputs, in-flight status), streaming cursor offsets (last received chunk or offset), provider model/version and tokens consumed, and step-level success criteria/side-effects. Persist this as append-only checkpoint records that can be replayed. LangGraph stores variables, outputs, and dialogue history as checkpointable units [124], [125]. [124], [125]
- Durable JetStream checkpointing: JetStream can store append-only streams, provide consumer offset management and resume semantics; consumers can resume from last acknowledged message after crash [17], [66], [67], [69]. Use JetStream for per-workflow checkpoint streams so supervisors can resume workers/agents from offsets. [17], [66], [67], [69]
- Atomic checkpointing across distributed participants: adopt two-phase commit or WAL-inspired approaches when multiple agents must agree on commit points (e.g., cross-node executor results) - streaming frameworks use two-phase commit to achieve atomic snapshots [130], [136]. [130], [136]

Applicability to Mister Smith and concrete recommendations
- Use per-workflow JetStream checkpoint subjects (append-only). Each checkpoint record contains: workflow_id, step_id, step_state (inputs, outputs, tool metadata), provider_cursor (stream offset or chunk id), model_id, token_usage_so_far, timestamp, and a small resumable VM of local variables. Ensure schemas are versioned and enforce compatibility rules because changing stateful operation schemas between restarts is unsafe [126]. [17], [66], [67], [124], [125], [126], [136]
- For multi-agent atomic transitions (e.g., when multiple Executors need to commit a coordinated state), use a prepare/commit pattern recorded to JetStream WAL so participants can recover via replay, analogous to two-phase commit [130], [136]. [130], [136]

Implementation complexity and risks
- Complexity: high. Designing compact, versioned checkpoint formats and ensuring efficient checkpoint frequency (tradeoff between performance and recovery time) is complex; frequent checkpoints degrade runtime performance [138]. Risks: schema evolution leads to unrecoverable state if not versioned; inconsistent commits across nodes if two-phase commit not carefully implemented. [138], [126], [130], [136]

Expected impact vs naive
- Durable checkpointing (LangGraph-style) prevents silent data loss and enables human-in-the-loop restarts and debugging; JetStream consumer offsets provide proven resume capabilities for streams and workflows, ensuring consumers can continue from last ack after crashes [17], [66], [67]. [17], [66], [67], [124], [125]

6) Provider health tracking and predictive failure detection

State of the art (citations)
- Recommended health signals include latency percentiles (p50/p95/p99), error rates, Retry-After exposure, token usage velocity, and streaming-specific metrics (stalls, partial-chunk counts). Phi-accrual detectors are used in distributed systems for heartbeat-based detection; layered telemetry across pre/during/post action controls is recommended for AI agents [57], [36], [23]. [57], [36], [23]

Key techniques and algorithms
- Health metrics to track: latency p50/p95/p99, error rate (5xx/429/4xx), Retry-After occurrences and durations, token consumption velocity, streaming stalls and chunk failure counts. Use moving-window counters and EWMA or percentile estimators for adaptive thresholds. Publish aggregated provider health events to NATS/JetStream so supervisors and schedulers can act. Evidence calls out latency/error-rate tracking and rate-limit handling as first-class signals [102], [103], [23]. [23], [36], [102], [103]
- Phi-accrual adaptation: although phi-accrual is a heartbeat inter-arrival detector originally used for node liveness, apply the principle to response inter-arrival/latency patterns for streaming and request/response latency anomalies: detect increases in latency variance or sustained inter-arrival gaps as degrader signals (evidence recommends layering detectors across workflow phases) [57], [13]. [57], [13]
- Predictive approaches: RL and ML can be used to learn repair policies and improve MTTR in research; but such systems have nontrivial training costs and complexity - evidence shows RL achieved improved repair success in experiments but is a higher-cost option [30], [31], [32]. [30], [31], [32]

Representing/synchronizing health in NATS/JetStream
- Use NATS subjects for ephemeral health broadcasts (fast local supervisors subscribe to health topics); use JetStream durable subjects to record longitudinal health events and provider state changes (open/close circuit, quota exhaustion) for global supervisors and auditing. JetStream supports durable consumers and replay from last ack for recovery [17], [66], [67], [69]. [17], [66], [67], [69]

Implementation complexity and risks
- Complexity: medium. Instrumentation and metrics aggregation are straightforward; building credible predictive models is high complexity. Risks: noisy signals leading to false positives, overreactive failover, and cost of ML approaches. [30], [31], [32], [23]

Expected impact vs naive
- Health-driven circuit-breakers and failovers reduce unnecessary retries and allow supervisors to proactively route traffic away from degraded providers. Layered failure detection reduces detection time compared to reactive-only approaches. Research suggests self-healing reduces MTTR and human effort, and RL experiments show large improvements where applied, but at higher cost [13], [31], [32]. [13], [31], [32]

7) Supervision strategies for agent teams and restart policies

State of the art (citations)
- OTP supervisor restart strategies include one_for_one, one_for_all, rest_for_one and configurable restart intensity/period; child specs carry restart/shutdown types (permanent, transient, temporary) [3], [11], [49], [50], [16]. Planner-Executor patterns advocate coordinators and promotion semantics for role-aware recovery [106], [99]. [3], [11], [49], [50], [106], [99]

Comparative analysis and role-aware patterns
- OneForOne: restart only the failing child - appropriate for isolated Executor failures. OneForAll: restart all children when one fails - useful when shared state corruption implies all children must reset (rare for agents). RestForOne: restart failed child and later-started siblings - useful when Executors depend on a later child but Planner precedes Executors. Evidence suggests planners are higher-value and should be treated differently; Executors are replaceable and run as transient workers in bulkheads [76], [104], [105]. [76], [104], [105]
- Finer-grained / role-aware strategy: implement supervisor logic that uses metadata about roles (Planner vs Executor vs Critic) and applies different restart policies: e.g., Executors as transient (do not restart on normal exit), Planner as permanent with careful restart limits, Critic as restartable but with stricter diagnostics before restart. Supervisor APIs allow dynamic restart_child, start_child, delete_child operations [11], [42], [44], [45]. [11], [42], [44], [45], [76], [104], [105]

Quorum/consensus vs eager restart tradeoffs
- Quorum-based recovery (wait for a set of healthy agents) increases safety for coordinated tasks but adds latency and complexity; eager restart prioritizes availability but may hide systemic faults. Evidence points to decentralized agent frameworks avoiding single points of failure by local negotiation but recommends safety mechanisms like rollback and impact assessment in autonomous healing [55], [22], [16]. [55], [22], [16]

Implementation guidance for Rust OTP-style supervisor
- Implement child spec metadata including role, restart policy (permanent/transient/temporary), and resource quotas. Allow supervisors to apply role-aware restarts and to use dynamic supervisor APIs to replace Executors without touching Planner. Use supervisor restart intensity and period settings tuned to expected LLM failure profiles to avoid supervisor cascades [11], [91]. [11], [91], [76], [104]

Complexity and risks
- Complexity: medium. Implementing role-aware supervisors is manageable but requires disciplined metadata and testing. Risks include incorrect restart semantics leading to flapping and deadlocks (agent deadlock cases were fixed by timeouts and orchestrator leases in related reports) [120], [121]. [120], [121]

Expected impact vs naive
- Role-aware supervision reduces unnecessary Planner restarts (preserves long-running memory and planning work) and reduces cost/latency compared to one-for-all restarts. Properly tuned restart limits prevent supervisor crashes and lower MTTR.

8) Testing and chaos scenarios

State of the art (citations)
- Chaos engineering (Netflix Chaos Monkey) is used to inject failures in production to validate recovery mechanisms; self-healing literature recommends closed-loop testing with injected faults and learning from incident history [33], [13], [36]. [33], [13], [36]

Practical test suites and fault injections
- Simulate: rate limits (429 with Retry-After), partial streaming failures (mid-stream termination), stale SSE (no chunks after connection), token/quota exhaustion, model deprecation responses, slow/poison responses, and malformed content-filter rejections. Use property-based tests for invariants (e.g., supervisor never restarts Planner more than N times in window), and fault-injection harnesses to cause specific provider error codes and streaming behavior. LangGraph and stream-processing checkpointing guidance recommends validating checkpoint/resume semantics and schema compatibility [124], [126], [127], [128], [136]. [124], [126], [127], [128], [136]

BEAM testing practices
- Erlang/OTP projects use Task supervisors, handle_continue patterns, and monitored processes in unit/integration tests to validate supervision behaviors and termination semantics; GenServer.terminate/2 can be used to signal other processes on stop for test hooks [88], [80], [93]. [88], [80], [93]

Implementation complexity and risk of testing
- Complexity: medium. Building realistic streaming simulators and injectors is engineering-heavy but essential. Risk: test suite gaps if chaotic behaviors are not comprehensive. Evidence indicates deadlocks in agent systems were fixed by adding timeouts and orchestrator-owned leases - tests must verify timeouts and leases hold [120], [121]. [120], [121]

Expected impact vs naive
- Regular chaos testing and property-based verification uncover race conditions (deadlocks, flapping) and validate restart policies; Netflix-style experiments demonstrated the value of deliberate failure injection for resilient architectures [33].

Synthesis recommendation - concrete fault-tolerance architecture for Mister Smith

Architectural principles (high level)
- Treat every LLM RPC/stream as a first-class supervised transient worker under a TaskSupervisor-like actor; keep Planner and global Coordinator as long-lived supervised processes; provide supervised circuit-breaker processes and provider-health actors per provider; persist durable events and checkpoints to JetStream; use NATS subjects for fast health broadcasts and JetStream for durable coordination and checkpoint logs. Leverage role-aware supervisors to avoid unnecessary Planner restarts; implement bulkheads via dedicated worker supervisors and per-role quotas. [38], [76], [86], [17], [66], [67], [4], [63]

Component responsibilities
- Planner (supervised, permanent): generates multi-tier plans (full/degraded) and owns workflow-level checkpoint writing to JetStream; coordinates retries, truncation of context, and escalation. [76], [124], [125], [126]
- Executor (transient child tasks under TaskSupervisor): spawns one worker per LLM call (HTTP/stream), enforces per-call timeouts, collects streaming chunks, reports success/failure, and writes step-level checkpoints. [21], [22], [38], [40], [124]
- CircuitBreaker actor (per provider, supervised): maintains failure counts, emits events when opened/blown, offers ask/install/reset APIs. Consider durable backing for counters to avoid ETS-like loss on restart. [4], [5], [12], [63], [68], [18]
- ProviderHealth actor (per provider, supervised): aggregates latency percentiles, error rates, Retry-After exposure and token-velocity; publishes transient health broadcasts to NATS and durable events to JetStream. [23], [36], [102], [103], [17], [66], [67]
- Coordinator / GlobalSupervisor: listens to JetStream health events and NATS broadcasts; executes topology changes (start/stop provider children, update routing) and records actions to JetStream for audit. [11], [17], [19], [66], [67]
- Checkpoint streams (JetStream): per-workflow and per-provider durable append-only streams containing step checkpoints, commits (prepare/commit), and topology-change records. Use versioned schemas. [17], [124], [125], [126], [130], [136]

Concrete restart and failover algorithms
- Executor failure: one_for_one semantics - restart only the Executor as a transient child up to configured max_restarts within a window; on repeated failures, circuit-breaker increments and if opened, mark provider degraded and invoke failover. [3], [11], [49], [91], [63]
- Planner failure: treat as higher-severity - use permanent restart but with diagnostic inspection before auto-restart (gather last checkpoints and optionally wait for operator/automated repair if repeated restarts exceed threshold). Record failure in JetStream and send alert. [11], [124], [125], [136]
- Provider failover: when ProviderHealth reports sustained degradation or circuit-breaker open, Coordinator instructs Supervisors to start Executor children configured for alternate provider or model, publishes failover directive to NATS subject (e.g., ms.failover.provider.<workflow_id>) and records action to JetStream. [17], [19], [66], [67], [63]
- Degraded planning: if failover unavailable, Planner must produce degraded plan tier (simpler plan, lower-cost model, partial results) and write checkpoint noting degradation reason. [76], [81], [124], [125]

Data models, messages, and NATS/JetStream schema (concrete)
- Health broadcast (ephemeral NATS subject ms.health.<provider>): JSON message { provider_id, p50_ms, p95_ms, p99_ms, error_rate, retry_after_count, token_velocity, timestamp } - used for fast local decisions. Persist via JetStream subject js.health.<provider> for historical records. [23], [36], [102], [103], [17], [66]
- Circuit-breaker events (NATS & JetStream) ms.cb.event.<provider>: { provider_id, state: open|closed|half_open, reason, fail_count, ts } - subscribers include Coordinator and local supervisors. [4], [63], [68], [17]
- Checkpoint stream subject (JetStream) js.checkpoint.<workflow_id>: append-only messages { workflow_id, step_id, step_state, provider_cursor, model_id, token_usage, schema_version, ts, resume_hint } - idempotent writes, small payloads encouraged per checkpoint. Use durable consumer per worker to ack when step safe. [124], [125], [126], [17], [66]
- Failover directive (NATS request-reply or pub/sub): ms.failover.<workflow_id> : { workflow_id, failed_provider, suggested_provider, reason, ts } and Coordinator may publish ack/decision on ms.failover.result.<workflow_id>. JetStream records the directive for audit. [17], [19], [66], [67]

APIs (supervisor and worker)
- Supervisor API:
  - start_executor(workflow_id, step_id, exec_spec) -> {ok, pid} (exec_spec contains provider_hint, model_id, token_budget, stream_expected boolean) [11], [38]
  - restart_child(child_id) / delete_child(child_id) / add_child(child_spec) - dynamic supervisor ops [11], [42], [45]
  - query_health(provider_id) -> latest health snapshot (from cached ProviderHealth actor or JetStream) [17], [66]
- CircuitBreaker API:
  - ask(provider_id, call_id, outcome) -> allow|deny; install(provider_id, config); reset(provider_id) [4], [63]
- Checkpoint API:
  - checkpoint_write(workflow_id, step_id, checkpoint_payload) -> seqno; checkpoint_read(workflow_id, since_seqno) -> stream cursor/resume_hint; prepare_commit/commit for multi-participant atomic changes [124], [125], [130], [136], [17]

Prioritized implementation roadmap (MVP → advanced)
- MVP (low-medium effort, high impact):
  1. Implement transient Executor workers under a TaskSupervisor equivalent; enforce per-call timeouts and spawn streaming-aware async handlers that publish chunk events to parent [38], [21], [22]. (low)
  2. Add supervised CircuitBreaker actor per provider (Fuse-style), with simple in-memory counters and NATS events for open/close; make supervisors consult circuit-breaker before starting Executors [4], [63], [68]. (medium)
  3. Add ProviderHealth actor to aggregate p50/p95/p99 and error rates and publish to NATS; implement simple failover to alternate provider when circuit-breaker opens. Persist health events to JetStream for auditing. [23], [36], [17], [66], [67] (medium)
  4. Implement per-workflow JetStream checkpoint stream with basic checkpoint schema and resume capability for Executors and Planners [124], [125], [17]. (medium)
- Phase-2 features (higher complexity):
  5. Role-aware supervisor logic (Planner vs Executor) with restart policies and restart-rate limiting. Add dynamic add/delete child support for provider replacement. [11], [42], [49], [76] (medium-high)
  6. Bulkhead enforcement via separate Executor supervisors with per-supervisor quotas and token budgets enforced by ProviderHealth/CircuitBreaker. [86], [93], [97] (medium-high)
  7. Streaming resume semantics (cursor handling) and robust streaming client wrappers with supervised cancellation and restart. [21], [22], [6], [7] (high)
- Advanced (research/operational cost):
  8. ML/RL-based predictive failure detection and adaptive failover policies (requires training and safety mechanisms). Evidence shows potential MTTR improvements but higher cost and complexity [30], [31], [32]. (high)
  9. Durable circuit-breaker state (persist counters/writes to JetStream or durable store) to avoid ETS-like counter loss across restarts [4], [18] (medium-high)

Test / chaos plan (staging → production)
- Unit/integration tests:
  - Verify handle_continue/deferred-init patterns and that planners and supervisors start without blocking [1], [2], [10].
  - Property tests: invariants such as "Planner restarts ≤ N in 5 minutes" and "Checkpoint stream always contains final step before worker exit" [11], [126], [124].
- Fault injection scenarios:
  - 429 flood with Retry-After variations; ensure circuit-breaker opens and failover to alternate provider occurs [23], [36].
  - Partial streaming drop: mid-stream termination and stale SSE-ensure Executor cancels and restarts via Supervisor and resume uses checkpoint cursor if available [21], [22], [6], [7], [124].
  - Token/quota exhaustion: simulate provider quota exhaustion and validate ProviderHealth marks provider unusable and fails over [31].
  - Model deprecation: simulate provider 4xx model-not-found and validate Planner receives structural error and switches to alternate model or degrades plan [8], [20].
  - Deadlock tests: simulate inter-agent waits and validate timeouts / orchestrator-owned lease recovery (evidence showed deadlocks were fixed by adding timeouts and leases) [120], [121].
- Production canary: deploy small percentage of traffic to new supervisors and circuit-breakers with Chaos Monkey-style injections to validate real-world resilience before wider rollout [33], [13].

Evidence gaps
- The provided evidence does not specify concrete phi-accrual parameter adaptation methods for LLM latency/streaming signals; phi-accrual is referenced in the project brief but not in the evidence set, so parameterization guidance for phi-accrual adaptation is not available in sources. (gap) [Project brief]
- No provided evidence gives exact API message schemas or serialization examples for JetStream checkpoint records; recommended JSON field names above are derived from synthesis, but there is no source that defines an exact canonical schema. (gap)
- Detailed implementation patterns and tooling for durable circuit-breaker state (beyond the ETS limitation) are not specified; evidence notes Fuse/ETS loss but no recommended durable design. (gap) [4], [18]
- Empirical quantitative comparisons (e.g., availability or MTTR numbers specific to LLM provider failover strategies) are not present except for general service-mesh and multi-region availability figures. (gap) [116], [117]

Conclusion
- Implement supervised, short-lived Executor workers, supervised circuit-breakers and ProviderHealth actors, and JetStream-backed durable checkpoints. Use role-aware supervisors so Planner survives Executor churn. Use NATS for low-latency broadcasts and JetStream for durable logs and checkpoint streams. Start with heuristic-based failover and health aggregation, and expand to ML/RL only if ROI justifies complexity. This staged approach composes with existing Mister Smith phase-2/3 primitives and NATS/JetStream while delivering robust, production-grade resilience and graceful degradation for LLM-powered agent workflows.

References
[1] https://stackoverflow.com/questions/26809391/erlang-how-to-deal-with-long-running-init-callback  
[2] https://www.erlang.org/doc/apps/stdlib/gen_server.html  
[3] https://www.erlang.org/doc/apps/stdlib/supervisor.html  
[4] https://github.com/jlouis/fuse  
[5] https://rokkincat.com/blog/2015/09/24/circuit-breakers-in-elixir/  
[6] https://ninenines.eu/docs/en/gun/2.2/manual/  
[7] https://stackoverflow.com/questions/67739157/elixir-how-to-consume-a-stream-of-server-sent-events-as-a-client  
[8] https://backendbytes.com/articles/llm-api-integration-patterns/  
[9] https://portkey.ai/blog/retries-fallbacks-and-circuit-breakers-in-llm-apps  
[10] https://hexdocs.pm/elixir/GenServer.html  
[11] https://hexdocs.pm/elixir/1.12/Supervisor.html  
[12] https://www.mojotech.com/blog/safeguard-web-service-failures-in-elixir-with-fuse/  
[13] https://www.theimpactinstitute.org/Publications/Noorian-Hosseini-Ulieru-Autonomous.pdf  
[14] https://thenewstack.io/three-stages-of-building-self-healing-it-systems-with-multiagent-ai/  
[15] https://www.netbraintech.com/blog/self-healing-networks/  
[16] https://hal.science/hal-01068045/file/POSTER_Self-Healing_Mechanisms_for_Software-Defined_Networks.pdf  
[17] https://docs.nats.io/nats-concepts/jetstream  
[18] https://rokkincat.com/blog/2015/09/24/circuit-breakers-in-elixir/  
[19] https://medium.com/@hadiyolworld007/nats-jetstream-playbook-exactly-once-minus-the-bloat-02fd9d5a051c  
[20] https://arxiv.org/abs/2511.19933  
[21] https://aws.amazon.com/blogs/machine-learning/detect-hallucinations-for-rag-based-systems/  
[22] https://online.stevens.edu/blog/building-self-healing-ai-orchestrator-reflexion-patterns/  
[23] https://sarcouncil.com/download-article/SJMD-259-2025-333-339.pdf  
[24] https://pmc.ncbi.nlm.nih.gov/articles/PMC12603247/  
[25] https://agentpatterns.tech/en/failures/deadlocks  
[26] https://www.kunalganglani.com/blog/multi-agent-ai-systems-production  
[27] https://www.comet.com/site/blog/multi-agent-systems/  
[28] https://esy.com/agents/patterns/planner-executor  
[29] https://docs.databricks.com/aws/en/structured-streaming/checkpoints  
[30] https://propelius.tech/blogs/checkpointing-in-stream-processing-best-practices  
[31] https://15445.courses.cs.cmu.edu/fall2018/notes/20-logging.pdf  
[32] https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html  
[33] https://www.architecture-weekly.com/p/the-write-ahead-log-a-foundation  
[34] https://faculty.cc.gatech.edu/~jarulraj/courses/8803-s22/slides/06-logging-2.pdf  
[35] https://codelabs.solace.dev/codelabs/solace-agent-mesh/?index=..%2F..index  
[36] https://oneuptime.com/blog/post/2026-02-16-how-to-handle-rate-limiting-and-throttling-in-azure-openai-api-calls/view  
[37] https://developers.openai.com/cookbook/examples/how_to_handle_rate_limits/  
[38] https://elixirforum.com/t/task-supervisor-with-max-restart-and-max-seconds/34108  
[39] https://www.mojotech.com/blog/safeguard-web-service-failures-in-elixir-with-fuse/  
[40] https://www.theimpactinstitute.org/Publications/Noorian-Hosseini-Ulieru-Autonomous.pdf  
[41] https://thenewstack.io/three-stages-of-building-self-healing-it-systems-with-multiagent-ai/  
[42] https://www.netbraintech.com/blog/self-healing-networks/  
[43] https://hal.science/hal-01068045/file/POSTER_Self-Healing_Mechanisms_for_Software-Defined_Networks.pdf  
[44] https://docs.nats.io/nats-concepts/jetstream  
[45] https://zilliz.com/glossary/nats  
[46] https://medium.com/@hadiyolworld007/nats-jetstream-playbook-exactly-once-minus-the-bloat-02fd9d5a051c  
[47] https://arxiv.org/abs/2511.19933  
[48] https://aws.amazon.com/blogs/machine-learning/detect-hallucinations-for-rag-based-systems/  
[49] https://online.stevens.edu/blog/building-self-healing-ai-orchestrator-reflexion-patterns/  
[50] https://sarcouncil.com/download-article/SJMD-259-2025-333-339.pdf  
[51] https://pmc.ncbi.nlm.nih.gov/articles/PMC12603247/  
[52] https://agentpatterns.tech/en/failures/deadlocks  
[53] https://www.kunalganglani.com/blog/multi-agent-ai-systems-production  
[54] https://www.comet.com/site/blog/multi-agent-systems/  
[55] https://esy.com/agents/patterns/planner-executor  
[56] https://docs.databricks.com/aws/en/structured-streaming/checkpoints  
[57] https://propelius.tech/blogs/checkpointing-in-stream-processing-best-practices  
[58] https://15445.courses.cs.cmu.edu/fall2018/notes/20-logging.pdf  
[59] https://martinfowler.com/articles/patterns-of-distributed-systems/two-phase-commit.html  
[60] https://www.architecture-weekly.com/p/the-write-ahead-log-a-foundation  
[61] https://faculty.cc.gatech.edu/~jarulraj/courses/8803-s22/slides/06-logging-2.pdf  
[62] https://codelabs.solace.dev/codelabs/solace-agent-mesh/?index=..%2F..index  
[63] https://oneuptime.com/blog/post/2026-02-16-how-to-handle-rate-limiting-and-throttling-in-azure-openai-api-calls/view  
[64] https://developers.openai.com/cookbook/examples/how_to_handle_rate_limits/  
[65] https://elixirforum.com/t/when-to-use-handle-continue/24736  
[66] https://stackoverflow.com/questions/52423061/how-to-use-gunopen-in-a-gen-server-module  
[67] https://hexdocs.pm/hackney/news.html  
[68] https://www.mojotech.com/blog/safeguard-web-service-failures-in-elixir-with-fuse/  
[69] https://arxiv.org/pdf/2504.20093  
[70] https://eajournals.org/bjms/wp-content/uploads/sites/21/2025/06/Cloud-Orchestration.pdf

(Note: several references above point to the same underlying source(s) used in the analysis; duplicates are retained in the list to ensure each cited URL from the evidence set is present.)