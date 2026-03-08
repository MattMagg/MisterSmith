# Deep Research Prompt: Supervision Trees and LLM Fault Tolerance

## Directive Context

Mister Smith is a Rust-based multi-agent orchestration framework with NATS/JetStream messaging, OTP-style supervision trees, and actor-based architecture. It must become architecturally superior to all competing agent frameworks.

Mister Smith has a unique advantage no competitor shares: Erlang/OTP-style supervision trees (Phase 3) with restart strategies (OneForOne, OneForAll, RestForOne), phi accrual failure detection (Phase 2), circuit breakers (Phase 2), and health monitoring. No Python agent framework has anything comparable.

Phase 9 adds LLM provider integration. The open question is: how do supervision trees compose with LLM provider lifecycle management? LLM calls are long-running external HTTP requests to third-party APIs that can fail in novel ways (rate limiting, token exhaustion, model deprecation, content filtering, partial streaming failure). This is fundamentally different from supervising local processes.

## Research Objective

Discover the most effective patterns for applying fault tolerance, supervision, and self-healing to LLM-powered agent systems. This is genuinely novel territory — no existing agent framework has Erlang-style supervision. Look at Erlang/OTP literature, distributed systems research, chaos engineering, and resilience patterns from critical infrastructure (aviation, nuclear, medical systems) for transferable patterns.

## Research Dimensions

### 1. Supervising External Service Calls in OTP
- What does the Erlang/OTP literature say about supervising processes that make external HTTP calls?
- How do Erlang applications handle gen_server processes that are blocked waiting for an external response?
- What patterns exist for timeout management when the supervised process is waiting on a third-party API?
- How does the Erlang community handle "poison pill" scenarios where restarting a process just hits the same external failure?
- Are there OTP patterns for "circuit breaker as a supervised process" rather than a library?

### 2. LLM-Specific Failure Modes
- What are the unique failure modes of LLM API calls that differ from typical HTTP services?
  - Rate limiting with retry-after headers
  - Token/budget exhaustion (402-style failures)
  - Model deprecation (model no longer available)
  - Content filtering (request or response blocked)
  - Partial streaming failure (stream starts but dies mid-response)
  - Stale connections (SSE connection appears alive but no data flows)
  - Model "hallucination loops" (model produces repetitive/degenerate output)
  - Context window overflow (accumulated conversation exceeds model limits)
- How should supervision strategies differ for each failure mode?
- Which failures are transient (retry helps) vs structural (retry makes it worse)?

### 3. Self-Healing Agent Topologies
- Can the supervision tree dynamically restructure the agent graph based on observed failure patterns?
- If Provider A consistently fails for a particular type of request, can the supervisor reroute to Provider B without external configuration?
- How do self-healing networks (SDN, mesh networking) handle similar dynamic topology changes?
- Are there adaptive supervision strategies that learn from failure history?
- What does the chaos engineering literature (Netflix Chaos Monkey, Gremlin) say about building resilient multi-service architectures?

### 4. Graceful Degradation in Multi-Agent Systems
- When one agent in a multi-agent workflow fails, what strategies exist for completing the workflow with reduced capability?
- Can a Planner agent produce a simpler plan when its preferred model is unavailable?
- Can an Executor agent fall back to a cheaper model when the expensive one is rate-limited?
- How do aviation systems handle degraded operations (fly-by-wire fallbacks, simplified displays)?
- What is "bulkhead isolation" and how does it apply to agent systems?

### 5. Checkpoint and Recovery for Long-Running Agent Workflows
- When a multi-turn agentic loop fails mid-execution, how do you resume from the last good state?
- What checkpoint formats exist for conversation state, tool execution state, and intermediate results?
- How do database transaction patterns (WAL, savepoints, two-phase commit) translate to agent execution?
- LangGraph has checkpointing and "time-travel debugging" — how is this implemented and what can we learn?
- Can JetStream provide durable checkpointing for agent state via append-only streams?

### 6. Provider Health Tracking and Predictive Failure Detection
- Beyond reactive circuit breakers — are there predictive approaches that detect provider degradation before failure?
- The phi accrual failure detector (which Mister Smith already has) uses heartbeat inter-arrival times. Can this be adapted for LLM response latency?
- What health signals should be tracked per-provider? (p50/p95/p99 latency, error rate, rate limit proximity, token usage velocity)
- How do cloud providers (AWS, GCP) implement predictive health checking for downstream services?
- Are there machine learning approaches to failure prediction in distributed systems?

### 7. Supervision Strategies for Agent Teams
- When a team of agents (Planner + 3 Executors + Critic) is supervised as a group, what restart strategy is optimal?
- OneForAll (restart all when one fails) is expensive for agent teams. Are there more nuanced strategies?
- How does Erlang's `rest_for_one` strategy apply when agents have sequential dependencies?
- Can supervision be aware of the agent's role — restart a failed Executor but escalate a failed Planner?
- What does the distributed systems literature say about quorum-based recovery for multi-node workflows?

### 8. Testing Fault Tolerance
- How do you test that supervision and failover actually work correctly?
- What chaos engineering patterns are applicable to agent systems?
- How do you simulate provider failures, rate limiting, and partial streaming errors in tests?
- Are there property-based testing approaches for verifying supervision tree behavior?
- How do Erlang applications test their supervision trees?

## Output Requirements

For each dimension, provide:
1. **Current state of the art** — what exists today, with specific citations
2. **Key techniques** — specific patterns, algorithms, or architectures discovered
3. **Applicability to Rust + OTP supervision + NATS** — how well does this transfer?
4. **Implementation complexity** — rough assessment
5. **Expected impact** — what improvement over naive "retry and hope"?

Conclude with a **synthesis section** recommending the optimal fault tolerance architecture for a Rust agent framework with OTP supervision trees, considering:
- Existing Phase 2-3 infrastructure (CircuitBreaker, HealthMonitor, SupervisedSystem, PhiAccrualFailureDetector)
- NATS for distributed health state and coordination
- The unique failure modes of LLM APIs vs traditional services
- Production reliability requirements (no silent data loss, no infinite loops, graceful degradation)

## Research Methodology

1. Start with Erlang/OTP literature on external service supervision
2. Survey chaos engineering and resilience engineering literature
3. Study how critical infrastructure systems (aviation, nuclear, medical) handle degraded operations
4. Look at what distributed systems research says about self-healing topologies
5. Examine LangGraph checkpointing as a concrete production implementation
6. Focus on patterns that compose with existing Mister Smith infrastructure (Phases 2-3)
7. Be practical — distinguish patterns that work in production from academic novelties
