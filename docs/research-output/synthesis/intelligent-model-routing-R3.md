---
version: R3
created: 2026-03-07
updated: 2026-03-07
sources: Ultra2x (3 reports) → Synthesized
round: 3 (Triple Synthesis)
---

# Intelligent Model Routing for Mister Smith

## Executive Summary

This report synthesizes three independent research investigations into intelligent model routing for the Mister Smith framework (Rust actors + NATS/JetStream + OTP-style supervision trees). The convergence across all three reports is striking: each independently arrives at a **staged, operations-first routing architecture** that starts with deterministic health/capability/budget routing and progressively layers on learned routing, cascading, and multi-model composition. This convergence lends high confidence to the recommended approach.

**High-Confidence Conclusions (all three reports converge):**

- **NATS-native routing is a decisive architectural advantage.** NATS request-reply benchmarks demonstrate ~50 us latency, and Rust-based gateways like Bifrost achieve 11 us overhead at 5,000 RPS. This places Mister Smith orders of magnitude ahead of Python-based proxies like LiteLLM (<500 RPS). Building the ModelRouter on Core NATS request-reply avoids HTTP internal hops entirely.
- **Two-plane architecture is the correct design.** All reports converge on separating a microsecond-latency **data plane** (per-request routing with local computation + cached state) from a **control plane** (continuously updated telemetry, pricing, budgets, health, and learned parameters streamed via JetStream KV watches).
- **Health-aware routing and circuit breakers are foundational and must ship first.** Passive health checks (circuit breakers) combined with NATS queue groups provide automatic failover without active polling overhead. This is prerequisite infrastructure for all advanced routing.
- **Budget enforcement belongs in the router, not in application code.** Hierarchical budget tracking (org -> team -> user -> request) using JetStream KV atomic Compare-And-Swap (CAS) operations enables distributed, real-time budget limits entirely in-memory.
- **Learned routing (RouteLLM-style) delivers up to 85% cost savings** while maintaining 95% of GPT-4 quality, and can run in-process using Rust ML libraries (ONNX, candle) to keep routing decision latency under 1-50 ms.
- **Mixture-of-Agents is powerful but latency-prohibitive for interactive use.** MoA achieves state-of-the-art 65.1% on AlpacaEval 2.0 using only open-source models but suffers from high TTFT. It should be reserved for asynchronous, high-value workloads.
- **Market-based auction routing is intellectually promising but premature for production.** A "posted-price" or weighted load-balancing mechanism using JetStream KV is a safer, more stable alternative to full real-time bidding.

**Key Differentiating Insights (unique to individual reports):**

- **Speculative cascading** (combining FrugalGPT-style cascades with speculative decoding draft-then-verify) yields higher speed-ups and better cost-quality trade-offs than either method alone (Report A).
- **Confidence as a first-class provider capability**: confidence should be treated as a capability of the provider interface, not a bolt-on. Techniques like Self-REF confidence tokens outperform verbalized confidence and token-probability signals (Report B).
- **Local ONNX embeddings** (FastEmbed) achieve routing latency <50 ms and can route ~80% of traffic to Tier 1 models, providing a practical middle ground between heuristics and full learned routing (Report C).
- **Tiered classifier pipeline** with four stages (microsecond capability filters -> cached embedding lookup -> optional local ML inference -> optional LLM-assisted routing) provides a structured way to balance routing accuracy against latency (Report B).
- **Power-of-two-choices load balancing** dramatically improves load balance over purely random assignment and is explicitly connected to tail-latency reduction (Report B).
- **Hedged requests** can reduce tail latency but can backfire via congestion; they should be used selectively for high-priority traffic only when requests cross P95 expected latency (Report B).

---

## 1. Architectural Foundation: The Two-Plane Router

All three reports converge on a **two-plane router** as the correct architectural pattern for Mister Smith. This mirrors how global load balancing and high-scale gateways make decisions based on constantly refreshed measurements.

### 1.1 Data Plane (Fast Path)

The data plane handles per-request routing with bounded latency (microseconds to low milliseconds). It operates on purely local computation plus cached state.

**Core router state (in-memory, updated by control-plane events):**

| State Table | Contents | Update Source |
|:---|:---|:---|
| **Model Registry** | Capabilities (streaming, tool-calling, embeddings, multimodal), context limits, supported response formats | JetStream KV watch |
| **Cost Model** | Token pricing, provider tier mappings, model cost maps | JetStream KV watch |
| **Budgets & Quotas** | Hierarchical budgets (org/team/user/tag), reset windows, current spend | JetStream KV atomic CAS |
| **Health Snapshot** | Rolling error rates, latency percentiles (P50/P95/P99), 429/Retry-After cooldown timers, rate-limit proximity | Provider telemetry events |
| **Routing Policy** | Weights, allow/deny lists, cascade thresholds, quality mode flags | JetStream KV watch |

**Routing decision pipeline (ordered stages):**

1. **Hard constraints**: Required capabilities, policy allow/deny, max context, explicit user tier constraints. Capability mismatches (multimodal, tool-calling, context limits) are documented triggers for fallback in gateways like Vercel.
2. **Health filters**: Remove tripped circuits/outliers. Envoy and Kong provide archetypes for passive ejection and active probing.
3. **Budget filters**: Drop models that violate per-request or remaining budget constraints. Optionally choose cheaper models as budgets deplete (analogous to constrained online optimization and RTB primal-dual control).
4. **Utility maximization**: Pick the model (or cascade) maximizing expected utility: `value(query, model) - lambda_cost * expected_cost - lambda_latency * expected_latency - lambda_risk * health_risk`. This aligns with cascade-routing's emphasis on balancing cost and quality estimates.
5. **Optional escalation**: If confidence/quality estimation is low, cascade or switch to MoA/verification pipelines under explicit latency/quality modes.

### 1.2 Control Plane (Slow Path)

The control plane continuously updates telemetry, pricing, budgets, health, and learned parameters that are streamed into the data plane via JetStream KV watches and NATS subjects. This creates a "publish once, all routers update" mechanism without service restarts.

**Key control-plane mechanisms:**

- **JetStream KV watches** for routing policy, budgets, and model registry updates. KV is explicitly watchable (real-time updates) and can be treated as a message stream.
- **Instant policy rollbacks**: The ModelRouter subscribes to a JetStream KV bucket containing routing configurations. When a key is updated (e.g., `nats kv put config routing.policy v2`), the watcher receives the update in real-time and hot-swaps the policy in memory.
- **Kill switches**: If a new model exhibits severe hallucinations, an operator can update the KV store to instantly drain traffic from that provider.
- **Shadow testing**: Before deploying a new routing policy, run it in "shadow mode" using JetStream KV watchers. The router computes the decision but does not act on it, logging the intended route for offline comparison against the active policy.

### 1.3 NATS Subject Taxonomy

A well-designed subject hierarchy is critical for location transparency and dynamic discovery. Mister Smith should adopt a taxonomy encoding business intent:

```
llm.request.{team}.{capability}          -- ingress requests to Router actor
llm.route.{capability}.{priority}        -- capability-encoded routing
llm.route.choice.{team}.{capability}     -- Router publishes selected provider/model (observability)
llm.provider.{provider}.telemetry        -- provider-client publishes telemetry JSON
llm.health.{provider}                    -- health monitor publishes health state
kv.routing.policy                        -- JetStream KV key for dynamic routing policies
```

**Wildcard subscriptions**: `ModelProvider` actors subscribe to relevant wildcards (e.g., `llm.route.chat.*`), allowing the `ModelRouter` to publish requests without knowing the exact provider topology. NATS subject mapping and transforms can act as translation/filter layers in the broker.

### 1.4 Queue Groups for Load Balancing

NATS queue groups provide built-in, distributed load balancing. When multiple `ModelProvider` instances subscribe to the same subject and queue group name, NATS randomly selects only one subscriber to process each message. If a provider instance crashes, others in the queue group seamlessly absorb the load without duplicate message processing. This maps model-provider pool instances to queue subscribers for load balancing, making capacity scaling and backpressure explicit at the messaging layer.

### 1.5 OTP Supervision Integration

Mister Smith's actor-based architecture maps health-aware routing to explicit actor roles supervised by OTP-style supervisors:

- **Provider-client actors**: Translate `ModelProvider` trait calls to specific provider APIs, measure latency/tokens/errors, publish telemetry. Supervised and restartable.
- **Health monitor actors**: Compute latency percentiles/error rates, set provider health state, publish to JetStream/KV and NATS subjects.
- **Router actors**: Read the latest health snapshot from an in-memory cache updated through NATS events or JetStream KV watches.
- **Budget-enforcer actors**: Maintain per-scope spend counters, expose checks to Router actor, enforce caps/degradation policies.
- **Learned-router / embed-router actors**: Optional micro-model or local ONNX-based embed similarity lookup that returns candidate tier ranking.
- **Supervisor tree**: OTP-style supervision of all above actors; if a `ModelProvider` actor fails repeatedly, the supervisor applies a `OneForOne` restart strategy with exponential backoff. If the failure rate exceeds a threshold, the circuit breaker opens and the router temporarily removes the provider from the active pool.

---

## 2. Staged Implementation Roadmap

All three reports independently recommend a staged approach. The synthesized roadmap below merges their phasing into a unified plan with concrete timelines and success criteria.

### Stage 0: Minimal Safe MVP (1-4 weeks)

**Goal**: Resilience and baseline routing with sub-100 us overhead.

| Component | Details |
|:---|:---|
| **NATS request-reply routing** | Core NATS request-reply for synchronous LLM calls, targeting sub-100 us routing overhead |
| **Deterministic heuristic router** | Route based on token count, tool presence, system prompt cues, modality flags. Heuristics can capture 50-70% of traffic for lightweight routing |
| **Queue group load balancing** | NATS queue groups for automatic load distribution across `ModelProvider` instances |
| **Provider health monitor + circuit breaker** | Passive health checks monitoring proxied traffic for timeouts, 429s, 500s. Track latency percentiles (P50/P95/P99) and error rates. Integrate with supervisor to pause/restart provider actors |
| **NATS-native policy distribution** | JetStream KV or subjects for policy updates with watchers for hot-reload |
| **Request tracing** | OpenTelemetry instrumentation via `tracing` and `opentelemetry` crates; trace IDs propagated across NATS messages |

**KPIs**: p99 routing overhead < 100 us, automatic failover on provider failure, zero message loss on provider crash.

### Stage 1: Budgets and Intelligent Routing (2-6 weeks)

**Goal**: Cost optimization with sub-2 ms overhead for learned routing.

| Component | Details |
|:---|:---|
| **Hierarchical budget enforcement** | Org -> Team -> Key budget enforcement using JetStream KV atomic CAS operations. Reserve estimated tokens before sending, reconcile actual usage afterward |
| **Budget-conditioned routing** | As budgets deplete, route toward cheaper models or reduce quality mode features. Soft-degradation policies (prefer cheaper provider that meets capability) |
| **Local embedding-based semantic router** | Integrate small ONNX embedding runner (FastEmbed blueprint) for topic-based routing. Routing latency <50 ms, can route ~80% to Tier 1 |
| **Route telemetry/logging** | Structured logs: provider, model, duration_ms, tokens, cache_hit for observability. Emit metrics to Prometheus |
| **Tag-based accounting** | Cost center/project/customer attribution via request metadata tags with per-tag budgets and reset durations |

**KPIs**: Up to 85% cost reduction for simple queries via cheaper model routing, budget enforcement with < 1% overrun rate.

### Stage 2: Cascading and Learned Routing (6-12 weeks)

**Goal**: Latency/cost tradeoff optimization.

| Component | Details |
|:---|:---|
| **Synchronous fallthrough cascade** | Call small model, evaluate confidence, call larger model if needed. FrugalGPT-style with scoring function and thresholds |
| **Learned router pilot** | RouteLLM-style small routing classifier trained on collected labeled traffic, integrated as micro-model (ONNX/wasm) for sub-10-50 ms decisions. Calibrate thresholds on real traffic |
| **Speculative cascading** | Token-by-token deferral: small fast model drafts tokens, larger model verifies in parallel. Confidence metrics (margin on logits) for dynamic accept/defer decisions |
| **RouterBench evaluation** | Utilize RouterBench dataset (405k+ inference outcomes across 11 models) to assess routing algorithm efficacy |

**KPIs**: 2x+ cost reduction for classification-like tasks, 2-3x speedups for speculative settings with high draft acceptance.

### Stage 3: Multi-Model Composition (12+ weeks)

**Goal**: Maximum quality for high-value tasks.

| Component | Details |
|:---|:---|
| **Mixture-of-Agents** | Layered architecture with multiple proposer models and aggregator model. Reserve for async, high-value queries |
| **Chain-of-Verification (CoVe)** | Draft, generate verification questions, answer independently, produce verified response. Policy-driven for domains where correctness > latency |
| **Committee voting** | Weighted majority voting with calibration for classification accuracy improvement |
| **Posted-price market routing** | Providers publish current rates and capacities to JetStream KV. Router uses published metrics for weighted load balancing. Avoids latency and collusion risks of per-request auctions |

**KPIs**: Quality improvements on benchmark leaderboards vs single models, decreased hallucination rates for CoVe-enabled flows.

### Stage 4: Experimental / R&D

| Component | Details |
|:---|:---|
| **Market-based routing** | Simulated reverse-auction controller for research (requires provider cooperation and anti-manipulation measures) based on contextual MAB auction formulations |
| **Online bandit routing** | Contextual bandits (PILOT-style) with online cost policy modeled as multi-choice knapsack problem |
| **MoA Alignment (MoAA)** | Use MoA-generated data and reward-modeling pipelines to improve post-training of open-source models |

---

## 3. Cascading and Speculative Inference

### 3.1 Cascade Architectures

LLM cascades are formalized as sequentially querying models and stopping early when a response is deemed "reliable enough." The literature identifies two complementary paradigms:

**FrugalGPT-style cascades**: Combine (i) a generation scoring function that estimates reliability from the query and generated answer, plus (ii) a router that chooses the cascade order and thresholds. FrugalGPT reports up to 98% cost reductions in studied settings, with newer cascade-routing work quantifying further improvements when estimates are accurate.

**Cascade routing (unified framework)**: ETH's cascade routing paper argues that quality estimation after generation is the critical linchpin, and reports that cascade routing can outperform both pure routing and naive cascades on benchmarks. Gains are dependent on the accuracy of cost/quality estimates---when reliability scoring is credible, cascades yield large cost reductions; when inaccurate, gains diminish.

**Human-in-the-loop cascades**: A deferral policy escalates from a base model to a larger model and (if needed) abstains to a human, using confidence scores and online feedback. This maps naturally to Mister Smith's supervised actor workflows with explicit escalation states (cheap -> expensive -> human/tool).

### 3.2 Escalation Triggers and Confidence Signals

The literature clusters around these reliability signals for "escalate or stop" decisions:

| Signal Type | Technique | Source |
|:---|:---|:---|
| **Post-hoc quality estimation** | FrugalGPT scoring function with thresholding over scores | FrugalGPT |
| **Confidence tokens** | Self-REF adds confidence tokens trained with error-based feedback; outperforms verbalized confidence and token-probability signals for routing/rejection | Self-REF (NeurIPS) |
| **Logit-based confidence** | Per-token and sequence log-probability thresholds derived from logits | Multiple sources |
| **Calibrated confidence** | Temperature scaling, logit normalization, supervised uncertainty estimation (auxiliary models trained on activations) | Amazon Science, OpenReview |
| **Lightweight verifiers** | Small verifier models or heuristic checks (structured output validation, retrieval-similarity thresholds) | C-3PO, cascade routing |
| **Abstention/deferral policies** | Confidence score -> deferral to stronger model -> abstention to human experts, with online learning for drift adaptation | NeurIPS cascaded decision framework |

**High-confidence finding**: Confidence should be treated as a **first-class capability of the provider interface**, not a bolt-on. The research emphasis on quality estimation implies the `ModelProvider` trait should expose confidence/logit information as a standard capability.

### 3.3 Speculative Decoding

Speculative decoding is a token-level cascade-like acceleration strategy: a small draft model proposes multiple tokens and a larger target model verifies them in parallel, preserving the target distribution.

**Key findings across reports:**

- **Speedups of 2-3x** when draft acceptance rates are high, especially effective when the draft model is 1/10 to 1/50 the target model size.
- **Acceptance rate and domain determine speedups**: Draft model "capability" as a language model may not correlate with speculative performance; draft selection/design is itself an optimization problem.
- **Online speculative decoding** adapts draft models to evolving query distributions to improve acceptance rates in deployment.
- **Medusa** provides an alternative that adds multiple decoding heads to a backbone model to predict multiple tokens in parallel, aiming for lossless acceleration.
- **Tail latency and concurrency effects**: As concurrency increases, batching and compute-bound regimes can reduce speculative benefits.

**Speculative cascading** (unique insight from Report A): Combining FrugalGPT-style cascades with speculative decoding yields higher speed-ups and better cost-quality trade-offs than either method alone. The small model drafts tokens while the expensive model verifies in parallel, with flexible deferral rules based on confidence metrics (e.g., margin on logits).

### 3.4 Applicability to Rust + NATS

Cascading is a strong fit for an actor system because it is naturally modeled as a supervised workflow with explicit escalation states. Implementation patterns:

- **Synchronous fallthrough cascade**: Call small model, evaluate, then call larger model if needed. Maps to sequential actor calls with NATS request-reply. A ModelRouter actor awaits a small-model reply and conditionally issues a second request. This is the simplest pattern and suitable as an MVP.
- **Background speculative calls**: Perform an async draft (background actor) while streaming/returning early on accepted draft tokens. Requires background speculative worker supervision and KV/policy signaling for thresholds.
- **Streaming fallbacks**: Draft stream with target model verifying. Requires streaming orchestration and token-level acceptance logic.

For speculative decoding, NATS is less directly relevant unless you are hosting models yourself; speculative decoding happens inside the serving stack, but the router can still "route to speculative-enabled deployments" (a capability bit) if some backends support it.

### 3.5 Implementation Complexity

| Pattern | Complexity | Prerequisites |
|:---|:---|:---|
| Synchronous fallthrough cascade | Low-moderate | Confidence computation, sequential request orchestration, telemetry |
| Background speculative + streaming | Moderate-high | Background workers, token-level streaming coordination, supervision strategies |
| Speculative decoding (in-house) | High | Deep inference-stack control |
| Speculative decoding (as capability flag) | Low | Provider metadata in model registry |

### 3.6 Evidence Gaps

- No measured end-to-end cascaded pipeline latencies against sub-100 ms interactive SLOs across multi-provider API calls (Report C).
- No production-grade token-level streaming acceptance implementation examples in Rust + NATS (Report C).

---

## 4. Mixture-of-Agents and Multi-Model Composition

### 4.1 Mixture-of-Agents (MoA)

MoA is a layered architecture where multiple "proposer" agents generate candidate responses, and "aggregator" agents refine/synthesize responses across layers.

**Key findings:**

- Together AI's reference implementation reports 65.1% on AlpacaEval 2.0 using only open-source models, demonstrating a "collaborativeness" phenomenon: models often improve when given other models' outputs, even if those auxiliary outputs are individually lower quality (ICLR 2025).
- **Diversity-aware proposer selection**: MoA highlights output diversity as a selection criterion. Heterogeneous models contribute more than repeated identical models.
- **Explicit latency cost**: MoA explicitly increases time-to-first-token, making it unsuitable as a default for interactive traffic.
- **MoA Alignment (MoAA)**: Uses MoA-generated data and reward-modeling pipelines to improve post-training of open-source models, suggesting MoA can function not only at inference time but also as a self-improving data generator.

### 4.2 Verification-Style Compositions

**Chain-of-Verification (CoVe)**: Explicitly drafts a response, generates verification questions, answers them independently (isolating verification answers from bias of the original draft), and produces a final verified response. Reports decreased hallucinations across tasks.

**Draft + Refine**: Use a cheap model to produce a draft, then a stronger model refines or verifies. Cost-effective when drafts are often acceptable or can be cheaply filtered.

**Committee Voting / Weighted Majority**: Aggregating k model outputs with weights can improve classification accuracy. Calibration further improves ensemble performance (Amazon Science).

### 4.3 Composition Patterns Summary

| Pattern | Mechanism | Best For | Latency Impact |
|:---|:---|:---|:---|
| **MoA (layered)** | Proposers -> Aggregator -> Refine | Maximum quality, async tasks | High TTFT |
| **CoVe** | Draft -> Verify questions -> Independent answers -> Final | High-stakes factual generation | Multiple additional steps |
| **Draft + Refine** | Cheap draft -> Strong refine | Cost-effective quality improvement | 1 additional call |
| **Committee Voting** | k parallel calls -> Weighted majority | Classification accuracy | Parallel but k-fold cost |
| **Dynamic Ensemble Selection** | Run only a subset based on query | Cost-quality balance | Variable |

### 4.4 Applicability to Rust + NATS

A message-bus, actor-based framework is unusually well-suited to MoA because it can run proposers in parallel and treat aggregators as downstream actors. NATS subjects and request-reply patterns can carry draft/verify flows and enable routing policy mutations at runtime.

**Mitigation strategies for latency:**
- Restrict MoA to high-value queries identified by the routing classifier.
- Use partial streaming strategies: stream a cheap draft immediately, then refine asynchronously.
- Gate composition flows under explicit "latency budget" policies.
- Use the budget actor to restrict parallelism and control cost.

### 4.5 Implementation Complexity

High: multi-model orchestration, cancellation, streaming merge semantics, and prompt-security (preventing proposer outputs from poisoning aggregator behavior) are nontrivial. MoA selection also implies you need a "model portfolio" and continual evaluation to choose proposers/aggregators.

### 4.6 Evidence Gaps

- No specific production case studies showing MoA deployed inside an actor/NATS architecture (Report C).

---

## 5. Learned Routing and Query Complexity Classification

### 5.1 Routing Approaches Taxonomy

The research community identifies several families of learned routing:

**Preference-trained classifiers (RouteLLM)**: Trained on public Chatbot Arena preference comparisons (win/tie/loss). Provides multiple router architectures: similarity-weighted ranking via Elo, matrix factorization, BERT classifier, causal LLM classifier. Reports up to 85% cost reduction at near-strong-model quality on several benchmarks.

**Training-free routers**: NeurIPS "Eagle" combines global/local Elo modules with faster online updates. Positioned for environments where retraining is impractical.

**Online routing under budget constraints (PILOT)**: Treats routing as a contextual bandit with an online cost policy modeled as a multi-choice knapsack problem for diverse budgets.

**Training-free online routing via ANNS**: Approximate nearest neighbor search plus one-time optimization (~250 queries) to compute routing weights. Theoretical guarantees and throughput claims, suitable for high-volume serving.

**Semantic routing via embeddings**: Kong routes prompts by matching embeddings to model descriptions, a production-friendly method if embeddings can be computed cheaply or reused. AWS classifies this as a common dynamic routing approach.

### 5.2 Tiered Classifier Pipeline

Report B proposes a structured four-stage pipeline that all three reports implicitly support:

| Stage | Latency Budget | Mechanism | Coverage |
|:---|:---|:---|:---|
| **A** | Microseconds | Capability filters + rule-based features: image/file presence, tool call requirement, max context/response size, user tier, request deadline | Hard constraints, 50-70% of traffic |
| **B** | Microseconds-low ms | Embedding lookup or cached nearest-neighbor similarity (precomputed or computed locally). Resembles Kong's semantic load balancing without network-to-embedding-provider overhead | Semantic matching |
| **C** | Milliseconds | Optional ML inference (BERT-like router) executed locally via ONNX. RouteLLM shows BERT classifiers can be effective routers trained from preference data | High accuracy routing |
| **D** | Slow path | "LLM-assisted routing" (router LLM) as AWS describes, acknowledging inherent cost/latency | Complex/ambiguous queries |

### 5.3 Local ONNX Embedding Router (Practical Blueprint)

Report C identifies a concrete implementation pattern: the "Model Router Blueprint" using local ONNX embeddings (FastEmbed) that achieves:
- Routing latency <50 ms total (~20 ms per embed locally vs 50-150 ms for remote API)
- Routes ~80% of traffic to Tier 1 models
- Compatible with Rust actor patterns via async-nats and local ONNX inference

This provides an important middle ground between pure heuristics (low cost, moderate accuracy) and full learned routers (high cost, high accuracy).

### 5.4 Training-Free Heuristics

All three reports validate deterministic heuristics as effective and low-cost:
- **Token count**: Input/output token estimates predict complexity.
- **Tool presence**: Requests requiring tool calling need capable models.
- **System prompt cues**: Flags indicating task type (classification, generation, coding).
- **Modality**: Presence of images, files, or structured data.
- **Retrieval size**: Large retrieval contexts may need models with bigger context windows.

These heuristics can capture 50-70% of traffic for lightweight routing in practice and are suitable as the initial MVP.

### 5.5 Cost/Benefit Analysis

| Approach | Cost Reduction | Routing Latency | Data Requirements | Maintenance |
|:---|:---|:---|:---|:---|
| **Heuristics** | Moderate (50-70% traffic routed to cheap tier) | Microseconds | None | Low |
| **Local ONNX embeddings** | Good (~80% to Tier 1) | <50 ms | Embed model + model descriptions | Moderate |
| **RouteLLM classifier** | Up to 85% on benchmarks | <50 ms (ONNX) | Preference data, training pipeline | High |
| **Online bandits (PILOT)** | Adaptive | Variable | Online feedback signals | High |
| **Semantic (Kong-style)** | Good | Depends on embed source | Model descriptions | Low-moderate |

### 5.6 Evidence Gaps

- No microsecond-level learned router implementations in the provided findings; evidence supports low-millisecond local ONNX routing (~20-50 ms) but not microsecond decision budgets (Report C).

---

## 6. Market-Based and Auction Routing

### 6.1 Academic Foundations

A 2026 AAMAS paper formalizes multi-provider LLM selection as a **reverse auction** where providers submit costs. It combines mechanism design with contextual online learning to produce "truthful" and query-aware selection, explicitly treating routing as a sequential decision problem with competing providers. Additional work includes DSIC (dominant-strategy incentive-compatible) mechanisms and ad-segment auction formulations with regret bounds.

### 6.2 Adjacent Industry Patterns

| Industry | Mechanism | LLM Routing Analogue |
|:---|:---|:---|
| **Ad-tech RTB** | Constrained optimization with budget constraints, online bid adjustment tied to real-time constraint snapshots | Budget-constrained provider selection |
| **Search ads (GSP)** | Generalized second-price auctions with nontrivial equilibrium behavior | Provider pricing mechanisms |
| **CDN/GSLB** | Health checks, RTT measurements, proximity, dynamic policies | Health-aware provider selection |
| **Telecom** | Primal-dual online control treating quotas as dual variables | Budget depletion -> de-emphasis of scarce capacity |

### 6.3 Transferred Patterns

- **Reverse-auction allocation with truthfulness constraints**: Select the provider with best utility given bids, where mechanism design discourages strategic misreporting.
- **Primal-dual online control** (from RTB): Treat budgets/quotas as dual variables and update routing "prices" in real time, so the system naturally de-emphasizes scarce capacity as it depletes.
- **Two-layer market structures**: Ad-tech's evolution from pure second-price to more complex clearing mechanisms suggests naive pricing intuition can be brittle.

### 6.4 Risks and Mitigations

**All three reports converge on caution:**

- **Latency overhead**: True real-time bidding introduces per-request auction latency that is incompatible with microsecond routing targets.
- **Collusion risks**: Commit-reveal schemes are vulnerable to off-chain collusion and latency attacks. Studies show LLMs themselves can emulate complex economic behaviors in repeated games, indicating strategic risks.
- **Provider cooperation**: Most public LLM providers don't expose bidding APIs.

**Recommended approach (high-confidence convergence)**: Implement a **synthetic auction / posted-price mechanism** where each provider "bids" using internally measured metrics (latency percentiles, error rates, observed rate-limit headroom) and a configured price schedule. Providers publish their current rates and capacities to a JetStream KV bucket. The router uses these published metrics for weighted load balancing. This avoids the latency and security risks of per-request auctions while capturing the benefits of dynamic pricing.

### 6.5 Implementation Complexity

- High if pursuing formal mechanism design with provable truthfulness (explicit bid interfaces and strategic assumptions).
- Moderate if implemented as "auction-inspired scoring" (utility maximization) using only internal signals.
- Recommended: Start with posted-price / utility scoring; treat formal auctions as R&D.

### 6.6 Evidence Gaps

- No production deployments or OSS examples of ad-tech-style real-time bidding applied to multi-provider LLM selection (Reports B and C).
- No NATS-native auction implementation guidance (Report C).

---

## 7. Health-Aware Routing and Circuit Breakers

### 7.1 Health Check Taxonomy

Production gateways distinguish between two complementary approaches:

**Active health checks**: Probing targets periodically with synthetic requests. Advantage: can re-enable previously unhealthy targets without manual intervention. Disadvantage: polling overhead.

**Passive health checks / circuit breakers**: Inferring unhealthiness from proxied traffic (TCP errors, timeouts, HTTP status codes). Advantage: zero polling overhead, immediate response to failures. Disadvantage: cannot automatically re-enable targets without active checks or manual intervention.

**High-confidence recommendation**: Combine both. Passive checks quickly remove misbehaving targets; active checks can re-enable them. This is the approach recommended by Kong and validated across all three reports.

### 7.2 Health Signals

| Signal | Description | Response |
|:---|:---|:---|
| **Consecutive failures** | Sequential error count exceeds threshold | Circuit opens, provider ejected |
| **Error rate** | Rolling error rate exceeds threshold in time window | Progressive degradation |
| **Latency percentiles** | P50/P95/P99 exceed SLO targets | De-prioritize provider |
| **Rate-limit proximity** | 429 Too Many Requests, Retry-After headers | Honor Retry-After, break circuit |
| **Temporal success rate** | Success rate drops below threshold in time window | Envoy-style outlier detection |
| **Capability mismatch** | Context limits exceeded, unsupported inputs | Treat as health failure, trigger fallback (Vercel pattern) |

**Important note (Azure guidance)**: Predicting throttling in advance via consumption tracking is "fraught with edge cases." Instead, honor `Retry-After` on 429 responses and break the circuit rather than repeatedly hitting a throttled endpoint.

### 7.3 Circuit Breaker States

Following Resilience4j patterns:
- **Closed**: Normal operation, tracking failure/slow-call rates.
- **Open**: Provider is unavailable, all requests fail fast or route to fallback.
- **Half-Open**: Limited trial requests to test recovery.

Thresholds: failure-rate thresholds and slow-call-rate thresholds trigger open/close transitions.

### 7.4 Progressive Degradation and Failover

- Automatic failover to backup providers when primaries hit rate limits or errors (Bifrost pattern).
- Degradation to smaller models or tool-disabling as fallback options.
- Envoy's upstream outlier detection generalizes passive health checking by ejecting hosts based on consecutive failures, temporal success rate, and temporal latency.

### 7.5 Operational Challenges

The hard parts (convergent across reports):
- **Picking correct time windows/thresholds**: Too sensitive causes flapping; too lenient causes prolonged routing to degraded providers.
- **Avoiding oscillation ("flapping")**: Requires hysteresis in state transitions.
- **Distinguishing local-origin vs upstream-origin errors**: Network hiccups vs provider failures require different responses. Envoy explicitly supports this distinction in its outlier detection metrics.
- **Health state scope**: Must explicitly decide whether health state is local-only (each node determines target health separately, as Kong does) or globally synchronized. If synchronized, JetStream KV with watchers is the plausible mechanism.

### 7.6 Load Balancing Algorithm Selection

Kong upstreams support multiple algorithms, implying routing should consider not only which provider but which balancing scheme fits the workload:

| Algorithm | Best For |
|:---|:---|
| **Round-robin** | Uniform workloads |
| **Least-connections** | Variable request duration |
| **Consistent-hashing** | Cache affinity, session stickiness |
| **Lowest-latency** | Latency-sensitive workloads |
| **Power-of-two-choices** | Dramatic improvement over random assignment with minimal overhead; explicitly cited in tail-latency discussions |

### 7.7 Implementation Complexity

Moderate, but operationally unavoidable. Health-aware routing is a prerequisite for more advanced routing (bandits/auctions), because learning over unstable endpoints can produce noisy feedback and unstable policies.

---

## 8. Budget-Aware Routing

### 8.1 Budget Enforcement as Gateway Responsibility

All three reports converge: budget enforcement belongs in the gateway/router, not in application code. This prevents runaway spending and enables predictable service tiers as product primitives (per-agent budgets, per-team guardrails) that are difficult to retrofit later.

### 8.2 Hierarchical Budget Model

```
Organization
  |-- Team A (monthly budget: $10,000)
  |   |-- User 1 (daily budget: $100)
  |   |-- User 2 (daily budget: $50)
  |   |-- Tag: "project-alpha" (monthly budget: $3,000)
  |-- Team B (monthly budget: $5,000)
  |   |-- Virtual Key "prod-key-1" (no individual limit)
```

Budget enforcement uses JetStream KV atomic Compare-And-Swap (CAS) operations for distributed, real-time limits:
- **Atomic operations**: Use JetStream KV's atomic `create` and `update` (CAS) to enforce budgets safely under high concurrency.
- **In-memory validation**: Router caches the hierarchy and validates limits locally, only committing to JetStream KV asynchronously or via batched updates to maintain microsecond latency.

### 8.3 Budget Enforcement Patterns

| Pattern | Mechanism | When to Use |
|:---|:---|:---|
| **Hard caps** | Reject request when budget exhausted | Strict cost control |
| **Soft caps with degradation** | Downgrade to cheaper model when budget approaches threshold | Graceful cost management |
| **Provider/model budget tagging** | Per-provider USD amount over period (LiteLLM pattern, e.g., float USD over "1d" period string) | Provider cost management |
| **Tag-based accounting** | Cost center/project/customer attribution via request metadata tags | Chargeback and reporting |
| **Budget-conditioned routing** | As budgets deplete, route toward cheaper models or reduce quality mode features | Dynamic optimization |
| **Formal online policies** | PILOT's knapsack-based policy treats routing under budget constraints as structured online optimization | Research/advanced |

### 8.4 Thundering Herd Mitigation

In high-throughput environments, multiple concurrent requests might pass a budget check before the token usage is committed, leading to budget overruns. Mitigation requires:
1. **Reserve** estimated tokens *before* the request is sent.
2. **Reconcile** the actual usage afterward.
3. Use JetStream KV CAS to ensure atomic budget updates.

### 8.5 Cost Model Requirements

Accurate budget enforcement depends on a reliable cost model:
- **Price map**: Token pricing per model per provider, including provider-specific tier metadata.
- **Token accounting**: Accurate per-request input/output token counts.
- **Provider pricing sync**: LiteLLM highlights pricing data sync and tier metadata as an ongoing concern.
- **Time-windowed budgets**: Daily, monthly, or rolling duration reset windows.

### 8.6 NATS Integration

- Budget updates, pricing maps, and policy changes distributed via JetStream KV watchers without restarting services.
- NATS/JetStream can event-source "spend events" and "budget exceeded" events, making post-hoc reconciliation robust.
- Budget checks belong in the router fast path as constant-time lookups (in-memory state), refreshed via control-plane updates.

### 8.7 Evidence Gaps

- No detailed algorithm pseudocode (e.g., knapsack-style selection) tying budgets to routing decisions in any of the three reports.

---

## 9. NATS-Native Routing Patterns

### 9.1 Core NATS Primitives for Routing

| Primitive | Routing Application |
|:---|:---|
| **Queue groups** | Load balancing: one subscriber per queue group receives each message. Horizontal scale and fault tolerance |
| **Subject hierarchies + wildcards** | Topic-based routing: `*` matches one token; `>` matches multiple tokens at end of subject |
| **Request-reply** | Synchronous routing hop with unique reply subject and timeout. ~50.87 us average latency in benchmarks |
| **JetStream KV** | Control-plane config distribution with watch/watch-all. Real-time updates, treatable as message stream |
| **Subject mapping/transforms** | Translation/filter layers in the broker. Can act as routing transformation without client changes |
| **JetStream streams + durable consumers** | Reliable request handling and replay for retries |

### 9.2 Tail-Latency Reduction

The distributed-systems literature emphasizes that at large scale, tail latency can dominate user experience and available throughput:

**Hedged requests**: Send a backup request after a delay (often after the P95 latency). Can reduce tail latency with modest extra load. However, later work shows hedging can backfire via congestion and proposes safer scheduling policies. **Recommendation**: Use selectively for high-priority traffic only when a request crosses P95 expected latency.

**Power-of-two-choices**: Sampling two servers and choosing the less loaded dramatically improves load balance over purely random assignment. Low overhead, explicitly cited in "Tail at Scale" discussions. **Recommendation**: Use as the default load-balancing heuristic rather than scanning all instances.

### 9.3 Router Deployment Topology

NATS patterns enable both deployment models:

- **Router-in-process**: Routing colocated with agents. NATS is the distribution fabric for provider workers and policy updates. Lower latency, simpler deployment.
- **Router-as-a-service**: Centralized routing. NATS request-reply overhead remains small relative to LLM inference. Queue groups provide immediate load balancing and failover at the router's output boundary. Better for multi-tenant isolation.

### 9.4 Performance Benchmarks

| Gateway / Protocol | Architecture | Measured Overhead | Throughput (RPS) |
|:---|:---|:---|:---|
| **NATS Core (Request-Reply)** | TCP/Custom Protocol | ~50 us | 100,000+ |
| **Bifrost** | Go (Compiled) | 11 us | 5,000 |
| **Kong AI Gateway** | Lua/Go Hybrid | Moderate | 2,000-3,000 |
| **LiteLLM** | Python (FastAPI) | High | <500 |

Mister Smith's architecture aligns closely with Bifrost's performance profile while leveraging NATS for additional distributed-systems capabilities (queue groups, KV, streams).

### 9.5 Evidence Gaps

- No measured NATS vs HTTP/gRPC latency comparison in the provided findings (Report C). Proposed microbenchmarks should quantify this.

---

## 10. Evaluation Harness and Observability

### 10.1 Evaluation Infrastructure

**RouterBench**: Dataset of 405k+ inference outcomes across 11 models for systematically assessing routing algorithm efficacy. Should be integrated into CI/CD for regression detection.

**Shadow testing**: Run new routing policies in "shadow mode" via JetStream KV watchers. The router computes the decision but does not act on it, logging the intended route for offline comparison against the active policy. Critical for preventing router degradation from dataset drift and provider API changes.

### 10.2 Observability and Telemetry

Every request must be tracked for SLA compliance and accurate billing:

**Structured telemetry fields per request:**
- `provider` - which provider handled the request
- `model` - which model was used
- `duration_ms` - end-to-end latency
- `tokens` - input/output token counts
- `cache_hit` - whether response was cached
- `routing_decision` - which stage made the routing decision
- `confidence_score` - confidence of the routing decision
- `budget_remaining` - remaining budget after request
- `circuit_state` - circuit breaker state at time of routing

**OpenTelemetry**: Instrument Rust actors using `tracing` and `opentelemetry` crates. Automatically create spans for functions and propagate trace IDs across NATS messages.

**Metrics**: Track Requests Per Minute (RPM), Tokens Per Minute (TPM), and error rates per provider. Emit to Prometheus to trigger alerts when a provider nears its rate limit or budget cap.

---

## 11. Concrete Experiments and Benchmarks

### 11.1 NATS vs HTTP/gRPC Microbench

- **What to measure**: Per-request routing-decision latency including NATS request-reply roundtrip vs direct HTTP call to provider client, P50/P95/P99 under varied concurrency.
- **Expected outcome**: Quantify NATS overheads and tail behavior to decide on using NATS in hot path vs direct call.
- **Minimal harness**: Rust actors using async-nats and a simple HTTP mock provider; sweep concurrency and message sizes.

### 11.2 Cascading Latency/Cost vs Single-Model Baseline

- **What to measure**: End-to-end latency and cost for test queries using synchronous fallthrough cascade, speculative decoding (if draft + verify supported), and single-model baseline.
- **Expected outcome**: Verify 2x+ cost reductions for classification-like tasks and 2-3x speedups for speculative settings with high draft acceptance.
- **Minimal harness**: Simulate small/large model latencies and costs; vary confidence thresholds.

### 11.3 Router Classifier Accuracy vs Routing Cost-Savings

- **What to measure**: Cost reduction and accuracy retention when using a learned router (RouteLLM-style) vs heuristic and local-embedding routers.
- **Expected outcome**: Validate RouteLLM-style large cost reduction with calibration on in-distribution traffic.
- **Minimal harness**: Collect labeled dataset from production-like queries; train small router; measure routing decisions.

### 11.4 Failure-Injection for Circuit Breakers

- **What to measure**: Router behavior under provider latency spikes, error bursts, and rate-limit events; verify automatic failover and supervisor restarts.
- **Expected outcome**: Ensure health monitors and circuit-breakers switch traffic to backups and supervisors restart or backoff provider actors.
- **Minimal harness**: Inject HTTP error rates and latency into provider mock and observe routing decisions.

### 11.5 Budget Enforcement Under Concurrency

- **What to measure**: Budget overrun rate under high-concurrency request bursts with JetStream KV CAS vs naive check-then-spend.
- **Expected outcome**: CAS-based enforcement should demonstrate <1% overrun rate vs potentially unbounded overruns with naive approach.
- **Minimal harness**: Concurrent Tokio tasks competing for budget with simulated token costs.

---

## 12. Minimal Rust/NATS Architecture Sketch

### 12.1 Component Responsibilities

```
                    +-------------------+
                    |   JetStream KV    |
                    | (Control Plane)   |
                    | - routing.policy  |
                    | - budgets.*       |
                    | - models.registry |
                    +--------+----------+
                             | KV Watch
                             v
+----------+    NATS Req    +-------------------+    NATS Req    +---------------------+
|  Agent   | ------------> |   Router Actor    | ------------> | Provider-Client     |
|  Request |    (ingress)  | - capability gate | (queue group) | Actor (supervised)  |
+----------+               | - health filter   |               | - API translation   |
                           | - budget check    |               | - telemetry publish |
                           | - utility score   |               +---------------------+
                           +---+--------+------+
                               |        |
                    +----------+        +----------+
                    v                              v
            +---------------+            +------------------+
            | Health Monitor|            | Budget Enforcer  |
            | Actor         |            | Actor            |
            | - P50/P95/P99 |            | - spend counters |
            | - error rates |            | - CAS updates    |
            | - circuit     |            | - degradation    |
            |   breaker     |            |   policies       |
            +---------------+            +------------------+
```

### 12.2 Router Decision Path (Pseudo-Rust)

```rust
// Router receives request via NATS subject llm.request.{team}.{capability}
async fn route_request(&self, request: RoutingRequest) -> RoutingDecision {
    // Stage A: Hard constraints (microseconds)
    let candidates = self.model_registry
        .filter_by_capabilities(&request.required_capabilities)
        .filter_by_context_limit(request.estimated_tokens)
        .filter_by_policy(&request.team, &request.user_tier);

    if candidates.is_empty() {
        return RoutingDecision::Reject("No capable model available");
    }

    // Stage B: Health filters (microseconds, cached state)
    let healthy = candidates.iter()
        .filter(|m| self.health_snapshot.is_healthy(m.provider_id))
        .collect::<Vec<_>>();

    let healthy = if healthy.is_empty() {
        // Fallback: use least-unhealthy candidate
        candidates.iter()
            .min_by_key(|m| self.health_snapshot.failure_count(m.provider_id))
            .into_iter().collect()
    } else {
        healthy
    };

    // Stage C: Budget filters (microseconds, in-memory lookup)
    let affordable = healthy.iter()
        .filter(|m| self.budget_enforcer.can_afford(
            &request.team, &request.user, m.estimated_cost(request.estimated_tokens)
        ))
        .collect::<Vec<_>>();

    // Stage D: Utility maximization (microseconds-milliseconds)
    let selected = if self.learned_router_enabled {
        // Optional: local ONNX inference for routing score
        self.learned_router.rank(&request.prompt_features, &affordable).await
    } else {
        // Heuristic: utility = value - lambda_cost * cost - lambda_latency * latency
        affordable.iter()
            .max_by(|a, b| self.compute_utility(a, &request)
                .partial_cmp(&self.compute_utility(b, &request))
                .unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
    };

    // Reserve budget before sending
    self.budget_enforcer.reserve(&request.team, &request.user, selected.estimated_cost);

    RoutingDecision::Route {
        provider: selected.provider_id,
        model: selected.model_id,
        cascade_policy: self.determine_cascade_policy(&request, &selected),
    }
}
```

### 12.3 NATS Subject Examples

```
llm.request.{team}.{capability}           -- ingress requests to Router actor
llm.route.choice.{team}.{capability}      -- Router publishes routing decision (observability)
llm.provider.{provider}.request           -- provider-client queue group receives work
llm.provider.{provider}.telemetry         -- provider-client publishes telemetry JSON
llm.health.{provider}                     -- health monitor publishes health state
llm.budget.{team}.spend                   -- spend events for post-hoc reconciliation
llm.budget.{team}.exceeded                -- budget exceeded alerts
kv:routing.policy                         -- JetStream KV: routing policy
kv:budgets.{team}.{user}                  -- JetStream KV: budget state
kv:models.registry                        -- JetStream KV: model capabilities
```

---

## 13. Items to Verify in Production

These claims from the research should be validated against actual deployment conditions rather than assumed:

1. **End-to-end latency overhead** introduced by NATS request-reply in the specific deployment environment vs HTTP/gRPC.
2. **Real draft acceptance rates** and speculative decoding speedups on the actual workload.
3. **Learned router generalization** and cost/accuracy tradeoffs on production traffic distributions.
4. **Provider bid truthfulness** and feasibility before attempting any auction-based routing.
5. **Budget enforcement precision** under high-concurrency bursts with CAS operations.
6. **Health threshold calibration** for specific providers (429 rates, latency distributions).
7. **MoA quality improvements** vs latency cost for specific task types in the target domain.

---

## 14. Consolidated Evidence Gaps

| Gap | Reports Noting It | Impact |
|:---|:---|:---|
| No measured NATS vs HTTP/gRPC latency numbers | B, C | Cannot validate microsecond routing claim without benchmarking |
| No end-to-end cascaded pipeline latencies against sub-100 ms interactive SLOs | C | Cascade viability for interactive use unconfirmed |
| No NATS-native auction implementation examples | B, C | Market-based routing remains theoretical for NATS |
| No concrete budget-to-routing algorithm pseudocode (knapsack-style) | B, C | Budget-conditioned routing logic must be designed from principles |
| No production MoA deployments inside actor/NATS architectures | C | MoA integration patterns must be designed from first principles |
| No microsecond-level learned router implementations | C | Evidence supports low-ms (~20-50 ms) but not microsecond learned routing |
| No prescriptive health threshold values (numeric P95 cutoffs) | C | Thresholds must be calibrated per-deployment |

---

## References

Deduplicated union of all citations across the three research reports.

### NATS and Messaging
1. NATS Bench CLI Documentation. https://docs.nats.io/using-nats/nats-tools/nats_cli/natsbench
2. NATS Subject-Based Messaging. https://docs.nats.io/nats-concepts/subjects
3. NATS Queue Groups. https://docs.nats.io/nats-concepts/queue
4. NATS JetStream Key/Value Store. https://docs.nats.io/nats-concepts/jetstream/key-value-store
5. NATS JetStream KV Walkthrough. https://docs.nats.io/nats-concepts/jetstream/key-value-store/kv_walkthrough
6. NATS Subject Mapping. https://docs.nats.io/nats-concepts/subject_mapping
7. NATS Request-Reply Pattern (OneUptime). https://oneuptime.com/blog/post/2026-01-27-nats-request-reply-pattern/view
8. async-nats Rust Client Documentation. https://docs.rs/async-nats

### LLM Gateways and Production Systems
9. Bifrost AI Gateway - Getting Started / Benchmarking. https://docs.getbifrost.ai/benchmarking/getting-started
10. Bifrost: The LLM Gateway That's 40x Faster Than LiteLLM. https://dev.to/varshithvhegde/bifrost-the-llm-gateway-thats-40x-faster-than-litellm-1763
11. Top 5 LLM Gateways in 2026: A Deep-Dive Comparison. https://dev.to/varshithvhegde/top-5-llm-gateways-in-2026-a-deep-dive-comparison-for-production-teams-34d2
12. We Evaluated 13 LLM Gateways for Production. https://dev.to/debmckinney/we-evaluated-13-llm-gateways-for-production-heres-what-we-found-2dkm
13. LiteLLM Provider Budget Routing. https://docs.litellm.ai/docs/proxy/provider_budget_routing
14. Cloudflare AI Gateway Dynamic Routing (referenced in Report B).
15. Load Balancing in AI Gateway: A Comprehensive Guide. https://www.getmaxim.ai/articles/load-balancing-in-ai-gateway-a-comprehensive-guide/

### Health Checks and Circuit Breakers
16. Kong Gateway - Health Checks and Circuit Breakers. https://developer.konghq.com/gateway/traffic-control/health-checks-circuit-breakers/
17. Kong Gateway - Load Balancing. https://developer.konghq.com/gateway/load-balancing/
18. Envoy Upstream Outlier Detection (referenced in Report B).
19. Microsoft Azure - Gateway patterns for Azure OpenAI (referenced in Report B, guidance on Retry-After and 429 handling).

### Vercel and Cloud Provider Guidance
20. Vercel AI SDK - Model Fallbacks and Provider Routing (referenced in Report B).
21. AWS Multi-LLM Routing Strategies for Generative AI. https://aws.amazon.com/blogs/machine-learning/multi-llm-routing-strategies-for-generative-ai-applications-on-aws/

### Learned Routing
22. RouteLLM - GitHub Repository. https://github.com/lm-sys/RouteLLM
23. RouteLLM Blog Post (LMSYS). https://lmsys.org/blog/2024-07-01-routellm/
24. RouterBench: A Benchmark for Multi-LLM Routing System. https://arxiv.org/html/2403.12031v1
25. Model Routing for Cost Optimization. https://abhyashsuchi.in/model-routing-for-cost-optimization/
26. The Model Router Blueprint. https://arome.substack.com/p/the-model-router-blueprint-building
27. Semantic Routing with Qdrant + Rig (Rust). https://dev.to/joshmo_dev/semantic-routing-with-qdrant-rig-rust-mj4
28. Kong Semantic Load Balancing (referenced in Report B).
29. Eagle: Training-Free Router with Global/Local Elo Modules (NeurIPS, referenced in Report B).
30. PILOT: Online Routing Under Budget Constraints (contextual bandits + knapsack, referenced in Report B).

### Cascading and Cost Optimization
31. FrugalGPT (Nexos.ai Blog). https://nexos.ai/blog/frugal-gpt/
32. Implementing FrugalGPT (Portkey Blog). https://portkey.ai/blog/implementing-frugalgpt-smarter-llm-usage-for-lower-costs/
33. Cascade Routing - ETH Zurich. https://files.sri.inf.ethz.ch/website/papers/dekoninck2024cascaderouting.pdf
34. Label with Confidence: Effective Confidence Calibration and Ensembles in LLM-Powered Classification (Amazon Science). https://assets.amazon.science/9f/8f/5573088f450d840e7b4d4a9ffe3e/label-with-confidence-effective-confidence-calibration-and-ensembles-in-llm-powered-classification.pdf
35. LLM Cascading for E-Commerce (GenAI E-Commerce 2024). https://genai-ecommerce.github.io/assets/papers/GenAIECommerce2024/Genaiecom24_paper_17.pdf
36. Agreement-Based Cascading. https://github.com/stevenkolawole/Agreement-Based-Cascading
37. Self-REF: Confidence Tokens for Routing/Rejection (referenced in Report B).
38. NeurIPS Cascaded Decision Framework with Online Learning (referenced in Report B).

### Speculative Decoding
39. Speculative Decoding - Google Research Blog. https://research.google/blog/speculative-cascades-a-hybrid-approach-for-smarter-faster-llm-inference/
40. Speculative Decoding Guide (BentoML). https://bentoml.com/llm/inference-optimization/speculative-decoding
41. Speculative Decoding LLM Inference Speedup Guide (Introl). https://introl.com/blog/speculative-decoding-llm-inference-speedup-guide-2025
42. Low-Latency LLM Serving (CMU PhD Blog). https://www.cs.cmu.edu/~csd-phd-blog/2024/low-latency-llm-serving/
43. Optimizing LLM Inference with Speculative Decoding and Quantization. https://medium.com/@ns3888/optimizing-llm-inference-with-speculative-decoding-and-quantization-ccfb491e67f5
44. Online Speculative Decoding (arXiv). https://arxiv.org/html/2411.13157v2
45. Medusa: Multiple Decoding Heads for Parallel Token Prediction (referenced in Report B).

### Mixture-of-Agents
46. Together AI - Mixture-of-Agents Blog. https://www.together.ai/blog/together-moa
47. MoA ICLR 2025 Paper (referenced in Report B, collaborativeness phenomenon).
48. MoA Alignment (MoAA) - Using MoA-generated data for post-training (referenced in Report B).
49. C-3PO: Multi-Model Composition Framework. https://github.com/Chen-GX/C-3PO
50. Chain-of-Verification (CoVe) - Reducing Hallucinations (referenced in Report B).
51. Dynamic Multi-Model Selection (arXiv). https://arxiv.org/html/2503.15850v2

### Market-Based and Auction Routing
52. Reverse Auction for Multi-Provider LLM Selection (AAMAS 2026). https://arxiv.org/html/2602.14476v1
53. Ad-Segment Auctions (arXiv). https://arxiv.org/pdf/2406.09459
54. LLMs Emulating Economic Behavior in Repeated Games (arXiv). https://arxiv.org/html/2502.09053v2
55. x402: Internet-Native Payments for APIs and AI Agents (Allium). https://www.allium.so/blog/x402-explained-the-internet-native-payments-standard-for-apis-data-and-agent-commerce

### Tail Latency and Distributed Systems
56. The Tail at Scale (Dean & Barroso, referenced in Report B).
57. Hedged Requests and Congestion Effects (referenced in Report B).
58. Power-of-Two-Choices Load Balancing (referenced in Report B).

### Confidence and Calibration
59. Confidence Calibration in LLMs (arXiv). https://arxiv.org/html/2404.02655v1
60. Supervised Uncertainty Estimation (OpenReview). https://openreview.net/pdf/a88694613c585df89fa68ab535a073653f0b7f6e.pdf
61. Calibration and Verifiers for Cascade Decisions (arXiv). https://arxiv.org/html/2402.15991v1

### Supervision and Actor Frameworks
62. Ractor: Not Just Another Actor Framework (Reddit). https://www.reddit.com/r/rust/comments/113dp70/ractor_not_just_another_actor_framework/
63. ractor-supervisor Crate. https://crates.io/crates/ractor-supervisor
64. ractor-supervisor Documentation. https://docs.rs/ractor-supervisor

### Observability
65. How to Instrument Rust Applications with OpenTelemetry. https://oneuptime.com/blog/post/2026-01-07-rust-opentelemetry-instrumentation/view

### Budget and Cost Management
66. Building Hierarchical Budget Controls for Multi-Tenant LLM Gateways. https://dev.to/pranay_batta/building-hierarchical-budget-controls-for-multi-tenant-llm-gateways-ceo
