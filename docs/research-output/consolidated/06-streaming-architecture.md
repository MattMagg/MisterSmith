# Streaming Architecture & Real-Time Processing -- Consolidated State of Knowledge

**Topic**: Streaming architecture, backpressure, token-level processing, dual-stream design, real-time inference, disaggregated serving, KV cache management
**Consolidation date**: 2026-03-07
**Sources**: R3 synthesis (3 merged industry reports), R4 academic search (57 papers, 28 searches), R6 frontier deep dive (step-level intelligence, streaming monitors), R7d discovery sweep (PrefillShare, NVIDIA Dynamo, disaggregated serving)
**Framework context**: Mister Smith -- Rust + Tokio + NATS JetStream + OTP-style supervision trees + model-agnostic LLM orchestration

---

## Executive Summary

Streaming architecture in Mister Smith is not a transport concern -- it is the primary control plane through which the orchestrator manages LLM inference, applies backpressure, detects failures, and routes cognition in real time. Four independent research rounds converge on the same core conclusion: **streaming must be modeled as a typed event pipeline managed by supervised actors, not as raw text chunking over SSE**.

The highest-impact findings cluster around five axes:

1. **Actor-per-LLM-stream with OTP supervision** isolates failures, manages lifecycle, and enables per-stream backpressure. All sources converge independently on this pattern. [R3-A4, R3-B17, R3-C-turn19, R4-CRGC, R4-Gordon]

2. **Dual-stream design** (lossless semantic + best-effort UI) decouples correctness from presentation, enabling different backpressure policies per event class. Tool-call boundaries are lossless; text deltas are coalescible. [R3-C-turn13, R3-C-turn17]

3. **Pull-based JetStream consumers with `MaxAckPending`** provide distributed backpressure superior to push consumers for token-heavy pipelines. Token Throttling independently regulates prefill and decode rates, achieving 11-398% throughput improvement. [R3-A1, R3-B8, R4-gLLM]

4. **Streaming Content Monitors** achieve 95%+ detection accuracy at only 18% of generated tokens, enabling mid-stream abort of doomed reasoning paths. JetStream CAS provides lock-free state transfer for rollback. [R6-SCM, R6-JetStream-CAS]

5. **Disaggregated serving with shared KV cache** (PrefillShare, NVIDIA Dynamo) eliminates redundant prefill computation across sequential agent handoffs, the single largest latency bottleneck in multi-model orchestration. [R7d-PrefillShare, R7d-Dynamo]

Critical metrics from the research: messaging P95 target <50ms for intra-agent coordination; streaming verification overhead <10% latency; KV cache waste <4% with PagedAttention; SCM detection at 18% of tokens; CLAI/TALE token reduction of 45-67% with <3% accuracy loss; RSD FLOP reduction of 4.4x.

---

## High-Confidence Findings

These findings are independently corroborated across multiple research rounds and sources. Confidence is HIGH for all items in this section.

### 1. Typed ModelEvent Streams Over Raw SSE Chunking

All three R3 source reports, plus R4's structured output research, independently conclude that streaming must be modeled as a typed `ModelEvent` enum with start/delta/end lifecycle, tool-call semantics, and observability events. This is validated by production systems: OpenAI Responses API, Vercel AI SDK, Microsoft Agent Framework, and OpenAI Agents SDK. [R3-A10, R3-B20, R3-B21, R3-C6, R3-C13]

The enum must use `#[non_exhaustive]` for forward compatibility (providers silently add event types) and `#[serde(other)]` for graceful fallback to an `Unknown` variant. [R3-A49, R3-A50]

### 2. Stream Finalizers as First-Class Supervised Components

The Microsoft Agent Framework's `ResponseStream` / `get_final_response()` pattern is the canonical approach: iterate the stream for real-time UI, then call the finalizer to get a schema-validated artifact. The OpenAI Agents SDK reinforces this: stream completion happens when the iterator ends, not when an apparent stop token arrives. [R3-A12, R3-B1, R3-C-turn11, R3-C-turn12]

### 3. Pull-Based JetStream Consumers for Distributed Backpressure

Pull consumers let the agent fetch messages at its own pace, providing implicit one-to-one flow control. Push consumers rely on `MaxAckPending` and protocol-level `FlowControl` messages, which can stall if not handled. Pull consumers scale horizontally without rebalancing. [R3-A1, R3-A28, R3-A30, R3-C5]

### 4. Actor-Per-Stream with OTP Supervision

Each streamed LLM response is a short-lived actor that owns: (a) provider connection, (b) incremental parser, (c) finalizer state, (d) outbound routing. One-for-one restart handles transient errors; escalation after N restarts with exponential backoff. Restarted actors rehydrate from JetStream replay. [R3-A35, R3-B17, R3-C-turn19]

### 5. Bounded Tokio Channels for In-Process Backpressure

`tokio::sync::mpsc` bounded channels force the sender to `.await` at capacity. `OwnedPermit` prevents head-of-line blocking by reserving capacity before pulling the next provider chunk. Without this, a blocked `write()` inflates the receive socket buffer and pushes TCP backpressure to the remote peer. [R3-A22, R3-A23, R3-A24, R3-B9]

### 6. Streaming Verification Improves Accuracy AND Efficiency

Streaming-VR (Ko, Baek, Hwang, EMNLP 2025) demonstrates that once incorrect tokens are generated early, subsequent tokens are more likely incorrect. Real-time token subset checking enables early abort and improves factual accuracy. [R4-Streaming-VR]

### 7. Local Recovery Over Global Restart

Partial snapshots that localize recovery to failing operators (not the entire pipeline) achieve 50%+ recovery time improvement. This validates Mister Smith's RestForOne supervision strategy over global restart. [R4-Takdir]

---

## Key Techniques & Architectures

### Actor-Per-LLM-Stream (OTP Supervision, Failure Isolation)

**Mechanism**: Each streamed LLM response is modeled as a short-lived supervised actor. The actor reads provider events (SSE/WebSocket), decodes them into `ModelEvent`, publishes canonical events to NATS subjects (Core for ephemeral UI, JetStream for durable audit), runs the finalizer, and enforces backpressure policy.

**Evidence**: All three R3 reports converge independently. Erlang's `DynamicSupervisor` (formerly `simple_one_for_one`) is the model -- optimized to start children dynamically, holding millions of transient processes. Elixir's GenStage formalizes demand-driven pipelines: consumers request N events, producers emit at most N. [R3-A5, R3-A35, R3-B17, R3-C-turn9]

**Rust framework alignment**: Ractor is the most aligned with Mister Smith's OTP requirements -- pure Rust, full supervision trees, `SupervisionEvent` (startup, death, panic), separate state type. [R3-A4, R3-A39, R3-B18] However, Mister Smith already implements its own actor system (`mister-smith-actor`), so the pattern applies to the existing `ActorCell` / `ActorRef` / `SupervisedSystem` primitives.

**Supervisor policies**:
- One-for-one restart for transient errors (parse failure, network blip)
- Escalate after N restarts with exponential backoff
- Mark artifact failed and emit `Error { recoverable: false }` only after repair exhausted
- [R3-B17, R3-B19, R3-A35]

**Lifecycle management frontier**: CRGC (Plyukhin, Agha, Montesi, 2025) introduces fault-recovering cyclic actor garbage collection using CRDTs -- no locks, no message ordering assumptions, formalized in TLA+. This could automatically reclaim stream actors that become unreachable in the supervision tree. [R4-CRGC]

**Actor Capabilities** (Gordon, 2025): Protocol-restricted references that limit message types an actor can receive. A supervision actor's reference could restrict subordinates to only send health reports and completion notifications. [R4-Gordon]

**Mister Smith integration path**: The existing `ActorCell` spawns a child actor per LLM stream. The `SupervisedSystem` applies one-for-one restart. JetStream replay rehydrates state on restart. The `StreamFinalizer` runs inside the actor to guarantee finalization even if downstream consumers disconnect.

---

### Dual-Stream Design (Lossless Semantic + Best-Effort UI)

**Mechanism**: Emit two streams from the same canonical event log:
1. **Lossless semantic stream**: All events preserved (tool-call boundaries, step boundaries, errors, finalization). Used for correctness, replay, audit, persistence.
2. **Best-effort UI stream**: May coalesce text deltas into larger chunks, drop heartbeats, and downgrade under pressure. Used for real-time rendering.

**Evidence**: R3-C-turn13 proposes this directly, supported by Vercel's protocol already separating fine-grained deltas from "available/done" boundaries. GStreamer's queue elements provide the precedent -- configurable as blocking (lossless) or leaky (drop old/new). Aeron's `offer` returns `-2` under backpressure, requiring application-level retry. [R3-C-turn17]

**Backpressure policy matrix** (per-event-class, not binary):
| Event Class | Policy | Rationale |
|:---|:---|:---|
| Tool-call boundaries | Lossless | Losing a tool boundary breaks execution |
| Step boundaries | Lossless | Required for multi-step orchestration |
| Finalization / approvals | Lossless | Missing these corrupts workflow state |
| Error events | Lossless | Supervisors must see every error |
| Text deltas | Lossy/coalescible | UI can tolerate merged chunks |
| Heartbeats / typing indicators | Lossy/droppable | Purely cosmetic |
| Partial JSON deltas | Lossy (preserve final) | Only `ToolInputAvailable` matters |

**Numbers**: Starting buffer sizes of 32-256 events per bounded channel. `MaxAckPending` should reflect expected concurrency per consumer. SSE/WebSocket frontends: send timeouts to close stale connections, per-client drop policy (`DropOldest` for realtime UIs, `DropNewest` when preserving earlier context matters). [R3-B9, R3-B10, R3-B11]

**Mister Smith integration path**: NATS Core subjects for ephemeral UI events (`ms.<run_id>.ui.*`). JetStream subjects for durable semantic events (`ms.<run_id>.events.*`). A coalescing operator between the canonical event log and the UI stream merges text deltas when the downstream bounded channel is >75% full.

---

### Backpressure Mechanisms (NATS Pull Consumers, Tokio StreamMap, Token Throttling)

**Mechanism**: Three-layer backpressure architecture covering in-process, distributed, and inference-engine levels.

#### Layer 1: In-Process (Tokio Bounded Channels + OwnedPermit)

`tokio::sync::mpsc` bounded channels block the sender at capacity. `OwnedPermit` reserves capacity before pulling the next chunk from the provider, preventing head-of-line blocking in `select!` loops. Without `OwnedPermit`, a blocked `write()` stalls the entire event loop, inflating socket buffers and pushing TCP backpressure upstream. [R3-A23, R3-A24, R3-A25]

**Coalescing as backpressure strategy**: Under pressure, merge token deltas into larger chunks rather than blocking. This parallels Akka's windowed/batching strategies to amortize async-boundary costs. [R3-C3]

#### Layer 2: Distributed (JetStream Pull Consumers)

| Consumer Type | Flow Control | Best For |
|:---|:---|:---|
| Pull Consumer | Implicit demand-driven; client requests batches via `fetch()` | Token-heavy LLM pipelines (recommended) |
| Push Consumer | `MaxAckPending` + protocol `FlowControl` messages | Simple pub/sub |
| Ordered Consumer | Automatic; recreates on gap detection | Single-threaded replay |

Configure `MaxAckPending` to limit in-flight messages. Use `Backoff` sequence to override static `AckWait` with exponential delays, preventing redelivery storms when tools take long. [R3-A1, R3-A31]

JetStream flow control can stall if `FlowControl` reply messages are not handled -- "pressure bugs" look like mysterious hangs. [R3-C5]

#### Layer 3: Inference-Engine (Token Throttling)

**gLLM Token Throttling** (Guo et al., 2025): Independently regulates prefill and decode token quantities for balanced pipeline computation. Asynchronous execution with message passing architecture. **11-398% throughput improvement**. [R4-gLLM]

**Implication**: The LLM provider layer should expose separate throttling controls for request submission rate (prefill) and response consumption rate (decode).

#### Latency Shifting (Novel Backpressure Strategy)

**TaiChi** (Wang et al., 2025): When some agents meet latency SLOs, their resources shift to at-risk agents. Fine-grained reallocation maximizes SLO-satisfied requests globally. **77% goodput improvement** under balanced SLOs. [R4-TaiChi]

#### Adaptive Timeouts (Emergent Load Shedding)

**SLO-Aware Load-Adaptive Timeout** (Hanada & Ishibashi, 2025): Dynamic timeout adjustment naturally exhibits load shedding and circuit-breaking behavior during downstream overload. **40% average and 55% tail latency reduction**. Eliminates the need for a separate load-shedding layer. [R4-Hanada]

**Backpressure as observability**: Backlog depth and "send waited" time should be treated as metrics and trace events. [R3-C-turn19]

**Mister Smith integration path**: Bounded `mpsc` with `OwnedPermit` inside stream actors. JetStream pull consumers for distributed fan-out. Token Throttling policy in the `ModelProvider` trait. Adaptive timeouts in agent-to-provider communication. Prometheus metrics for channel backlog depth.

---

### Streaming Content Monitors (95% Detection at 18% of Tokens)

**Mechanism**: Streaming Content Monitors (SCM) run in parallel with autoregressive generation, fetching the latest token and providing timely judgment of failure or harmfulness. They do not wait for step completion.

**Evidence**: SCMs achieve **95%+ detection accuracy by observing only the first 18% of tokens** in a response. [R6-ref12, arxiv:2506.09996v2]

**Integration with step-level intelligence**: When a PRM or SCM detects failure, the orchestrator aborts the draft model's stream, rolls back JetStream KV state to the previous revision, and issues `NakWithDelay` to retry the step. [R6-ref25, R6-ref26]

**Dual-stream race condition prevention**: In a dual-stream design (generation + evaluation), JetStream KV Compare-And-Swap (CAS) with revision tracking prevents race conditions. When a step begins, a KV entry is created with a specific revision. If the evaluator triggers rollback, it performs a CAS update -- if the revision matches, rollback commits and the generator's stream is aborted. [R6-ref13]

**Mister Smith integration path**: Deploy a lightweight SCM actor per stream (or per step) that subscribes to the same token subject as the finalizer. SCM emits `smith.step.rejected` events. The supervision tree aborts the stream actor and applies micro-rollback via JetStream KV CAS. This enables abort of doomed reasoning paths within the first ~20% of token generation.

---

### Step Boundary Detection (Structured Output Parsing, Step Completion Signals)

**Mechanism**: Operating at the step level requires intercepting and evaluating tokens as they stream. Step boundaries vary by domain.

**Evidence and domain-specific boundaries**:
- **Mathematics**: PRM800K defines steps via newline characters (`\n\n`). [R6-ref5]
- **Code generation**: DreamPRM-Code treats entire functions as reasoning steps using "Chain-of-Function" prompting. [R6-ref16]
- **Tool use & planning**: ToolPRMBench evaluates PRMs on tool-using agents by converting interaction histories into step-level test cases. [R6-ref17]
- **Streaming constraints**: Boundary tokens inserted as supervisory signals guide the model to recognize when a reasoning unit should terminate. [R6-ref18]
- **Vercel AI SDK**: Explicit `start-step` / `finish-step` markers in the SSE protocol for multi-call agentic loops. [R3-C-turn13]
- **OpenAI Agents SDK**: Higher-level "run item" events (message generated, tool called, tool output, handoff requested) layered on raw token streaming. [R3-C-turn12]

**Incremental structured output parsing**: Tool arguments arrive as progressively longer JSON prefixes that are invalid until completion. Three failure classes must be handled: (1) incomplete (waiting for more bytes), (2) invalid/partial by provider semantics, (3) provider bugs and truncation. [R3-C-turn16, R3-C-turn2]

**Parser recommendations for Rust**:
| Parser | Best Use | Performance |
|:---|:---|:---|
| `actson` | Push-based incremental parsing inside finalizer actors | Minimal allocations, event-driven, `NeedMoreInput` signal |
| `simd-json` | High-throughput inner tool-call argument payloads | Multi-GB/s, SIMD-accelerated, two-stage model |
| `json-stream` | Outer SSE envelope at gateway edge | Deletes deserialized bytes from buffer |
| `json-event-parser` | SAX-like partial tool-call extraction | Low overhead, event-based |
| `deser-incomplete` | Tolerant parsing of streaming JSON | Designed for invalid-until-done patterns |

**Recommended hybrid**: `json-stream` for the outer SSE envelope, `simd-json` for inner tool-call payloads. Push-based parsers (`actson`, `json-event-parser`) avoid reparsing whole buffers and enable earlier structural problem detection. [R3-A-synthesis, R3-B16, R3-C-turn20]

**Constrained generation at the provider level** (Sejourne & Lata, 2025): Using xgrammar-style constrained generation achieves higher format compliance for function calls than post-parsing. Reduces error handling complexity in the ToolBus. [R4-Sejourne]

**Mister Smith integration path**: The `ModelEvent` enum includes `StepStarted` / `StepFinished` lifecycle events. The finalizer actor uses `actson` for push-based incremental parsing. Step boundaries detected by domain-specific heuristics (newline for math, AST for code, explicit markers for tool use). Structured output validated via `simd-json` on completion.

---

### Disaggregated Serving (PrefillShare, NVIDIA Dynamo, Shared KV Cache)

**Mechanism**: Physically decouple the compute-bound prefill phase from the memory-bound decode phase. Multiple task-specific agent models share a single frozen prefill module via prefill-only tuning that aligns latent spaces.

**Evidence**: PrefillShare (arxiv:2602.12029) demonstrates that heterogeneous, task-specific agent models can consume the exact same KV cache generated by a base model without redundant computation. In conventional architectures, when agents pass a growing context window, each subsequent model must redundantly recompute the KV cache for the identical prefix -- this is the dominant latency bottleneck in multi-step agent negotiations. [R7d-ref1, R7d-ref3]

| Dimension | Conventional | Disaggregated Shared-Prefill |
|:---|:---|:---|
| Workload Distribution | Prefill + Decode on same GPU | Isolated to dedicated GPU pools |
| Context Processing | Redundant prefill per agent handoff | Single prefill; KV cache shared across all agents |
| Network Payload | JSON text prompts | Distributed pointers to unified KV cache stores |
| TTFT Impact | Severe during multi-agent consensus | Substantial reduction + linear scalability |

**NVIDIA Dynamo** (open-source, 2026): Standardizes orchestration of disaggregated workloads. Implements local KV indexers and non-blocking radix snapshots for seamless state transfers between physically separate GPU nodes. Prevents prefill interference with autoregressive decoding, which traditionally degrades inter-token latency across concurrent agent streams. [R7d-ref5, R7d-ref7, R7d-ref8]

**SUN (Shared Use of Next-token Prediction)**: Further validates that multiple task-specific models can seamlessly ingest a universal KV cache. [R7d-ref2]

**Mister Smith integration path**: This requires evolving the data plane from routing JSON text payloads to streaming tensor embeddings and distributed KV cache pointers. The message broker becomes a distributed memory bus for latent context. This is a Phase 10+ capability -- the immediate integration point is the `ModelProvider` trait exposing cache hints and prefill/decode phase awareness in the streaming API.

---

### KV Cache Management (PagedAttention, Quantized Persistence, Cache Transfer)

**Mechanism**: The KV cache is the memory bottleneck in multi-agent LLM serving. Its size scales linearly with sequence length (LLaMA-2 70B at 4K context = ~10 GB). Management strategies determine whether multi-agent orchestration is feasible at scale.

**PagedAttention** (vLLM): Reduces KV cache waste to **under 4%** by managing cache in non-contiguous pages, analogous to virtual memory. [R6-ref21, R6-ref20]

**Tokencake** (Bian et al., 2025): Co-optimizes scheduling and memory for multi-agent workloads. Space Scheduler shields critical agents from KV cache contention via dynamic memory partitioning. Time Scheduler proactively offloads/uploads during tool call stalls. **47% latency reduction** vs vLLM. [R4-Tokencake]

**KVFlow** (Pan et al., 2025): Abstracts agent execution as an "Agent Step Graph" with steps-to-execution estimation for cache eviction. Fully overlapped KV prefetching from CPU to GPU in background threads. **1.83-2.19x speedup** over SGLang. [R4-KVFlow]

**Continuum** (Li et al., 2025): Tool calls break workflow continuity, causing KV cache eviction and scheduling bubbles. Solution: predict tool call durations, pin KV cache with TTL, program-level FCFS scheduling. [R4-Continuum]

**KV cache offloading**: NVIDIA Dynamo offloads cache blocks to CPU RAM or SSDs, instantly transferring them when switching models -- avoids expensive recomputation. [R6-ref22]

**FlashSVD** (Shao et al., 2025): Rank-aware streaming inference loads small tiles into on-chip SRAM, processes, and immediately evicts. **70.2% peak activation memory reduction**. The "tile-process-evict" pattern is a general design principle for token processing pipelines. [R4-FlashSVD]

**Persistent KV cache** (from R7c): Reduces Time-to-First-Token from **15.7s to 0.6s** for resumed sessions. [R7c cross-reference]

**Mister Smith integration path**: The `ModelProvider` trait should expose cache management hints: (1) TTL for pinning during tool calls, (2) priority hints for critical agents, (3) session resumption via cache identifiers. The AgentScheduler annotates the task DAG with steps-to-execution estimates to inform preemptive cache allocation.

---

### ModelEvent Enum (#[non_exhaustive], Forward-Compatible Streaming)

**Mechanism**: A canonical typed event enum that serves as the internal event log for all LLM streaming interactions. All downstream processing (finalization, persistence, UI rendering, observability) consumes this single type.

**Synthesized superset from all R3 proposals + R4 validation**:

```rust
/// Canonical streaming event type for all LLM interactions.
#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "kebab-case")]
#[non_exhaustive]
pub enum ModelEvent {
    // --- Lifecycle events ---
    RunStarted { run_id: String, metadata: EventMeta },
    StepStarted { step_id: String, metadata: EventMeta },
    StepFinished { step_id: String, finish_reason: Option<String> },
    RunCompleted { run_id: String, usage: UsageStats },
    RunFailed { run_id: String, error: ErrorInfo },

    // --- Text streaming ---
    TextStart { id: String, metadata: EventMeta },
    TextDelta { id: String, chunk: Bytes },
    TextEnd { id: String, finish_reason: Option<String> },

    // --- Tool call lifecycle ---
    ToolCallStart { call_id: String, tool_name: String },
    ToolInputDelta { call_id: String, input_text_delta: String },
    ToolInputAvailable { call_id: String, input: serde_json::Value },
    ToolResultAvailable { call_id: String, result: serde_json::Value },

    // --- Structured output fragments ---
    StructuredFragment { schema_id: String, fragment: serde_json::Value },

    // --- Observability ---
    UsageDelta { tokens_in: usize, tokens_out: usize },
    SpanStart { span_id: String, metadata: EventMeta },
    SpanEnd { span_id: String, metadata: EventMeta },

    // --- Error handling ---
    Error { code: u16, message: String, recoverable: bool },

    // --- Heartbeat / keep-alive ---
    Heartbeat,

    // --- Forward compatibility ---
    #[serde(other)]
    Unknown,
}
```

**Design rationale**:
- `#[non_exhaustive]`: Providers silently add event types; the framework must never break. [R3-A49]
- `#[serde(other)]` on `Unknown`: Graceful fallback for unrecognized events. [R3-A50]
- `ToolInputAvailable` separate from `ToolInputDelta`: Enables typed handlers -- finalizers wait for `Available`, UIs render `Delta`. [R3-B20]
- `Error { recoverable: bool }`: Lets supervisors decide whether to terminate or resume. [R3-C-turn21]
- `UsageDelta`: Streamable incrementally for near-real-time billing/monitoring. [R3-B21]
- Lifecycle events (`RunStarted`, `StepStarted`, etc.): Aligned with OpenAI and Vercel patterns. [R3-C6, R3-C-turn13]

**Event envelope for distributed routing** (R3-B alternative):

```rust
pub struct EventEnvelope {
    pub session: SessionId,
    pub offset: u64,        // per-session sequence number
    pub body: EventBody,    // wraps ModelEvent variants
    pub trace_ctx: Option<TraceContext>,
}
```

Separates routing metadata from event content -- advantageous for NATS subject-based routing and JetStream persistence. [R3-B-synthesis]

**Step-level telemetry extension** (R6):

| Field | Type | Purpose |
|:---|:---|:---|
| `trace_id` | UUID | Distributed tracing across agents |
| `step_id` | UUID | Deterministic hash of context + step index; idempotency |
| `actor_id` | String | Performance tracking per model |
| `intrinsic_load` | Float | Pre-step complexity estimate (0.0-1.0) |
| `prm_score_calibrated` | Float | Quantile-regressed success probability |
| `routing_action` | Enum | `ACCEPTED_DRAFT`, `ESCALATED_TARGET`, `ABORTED` |
| `tokens_generated` | Integer | Cost accounting and CLAI refinement |

Serialize with Protobuf or MessagePack for high-throughput JetStream ingestion. [R6-ref29]

**Mister Smith integration path**: The `ModelEvent` enum is the canonical type in `mister-smith-llm`. The `EventEnvelope` wraps it for NATS publishing. Step-level telemetry fields are optional extensions gated behind a `step-intelligence` feature flag.

---

## Transport Architecture

### Hybrid Transport Recommendation

All sources converge on a three-tier hybrid:

| Tier | Transport | Use Case |
|:---|:---|:---|
| Provider-facing | SSE (default), WebSocket (tool-heavy) | LLM API communication |
| Internal | NATS/JetStream | Server-to-server, durable messaging, fan-out, replay |
| Client-facing | SSE (simple push), WebSocket (bidirectional) | UI delivery |

**WebSocket for provider connections**: OpenAI's WebSocket mode claims **~40% faster end-to-end execution** for workflows with 20+ tool calls, by keeping persistent connections with `previous_response_id` continuation. Constraints: sequential (one response per connection), no multiplexing, 60-minute connection duration limit. [R3-C8]

**SSE proxy buffering problem**: NGINX and Cloudflare buffer SSE responses by default (Cloudflare up to ~100KB before flushing). Required mitigations: `X-Accel-Buffering: no` header, `proxy_buffering off`, `proxy_cache off`, `chunked_transfer_encoding off`. [R3-A6, R3-A7, R3-A41]

**NATS subject hierarchy for fan-out**: `ms.<run_id>.<agent_id>.events.*` enables per-subscriber consumers (UI, tracing, persistence, tool execution) to filter server-side. JetStream `AllowRollup` prunes prior messages when only latest state matters. [R3-C5, R3-B8]

**Zero-copy messaging** (R4): ROS 2 Agnocast achieves constant IPC overhead regardless of message size via true zero-copy shared memory. 16% average / 25% worst-case response time improvement. The InMemoryTransport could be enhanced with this for co-located agents. [R4-Agnocast]

---

## Concurrency & Fan-In/Fan-Out

**`StreamMap` over `SelectAll`**: `tokio_stream::StreamMap` provides O(1) insertion/removal with key-indexed streams, ideal for dynamic fan-in where sub-agents spin up and shut down. `SelectAll` requires uniform types, causes round-robin degradation, and closure type mismatches. [R3-A8, R3-A9, R3-A33]

**Ordering**: Core NATS ordering is per-publisher, not total across publishers. Options: (a) accept nondeterministic interleaving, (b) tag events with `(agent_id, sequence_number)` for per-agent ordering, (c) use Lamport clocks for causality, (d) watermark-based merging for bounded buffering. [R3-C4, R3-A34]

**Tail latency amplification**: A single slow tool call bottlenecks the entire aggregated response in multi-agent pipelines. `buffer_unordered` with strict timeouts prevents one slow stream from stalling the event loop. [R3-A11, citing Dean & Barroso, "The Tail at Scale"]

**Pluggable distribution strategies** (Skitter, Saey et al., 2025): Decouple data processing from distribution. Different strategies (round-robin, content-based, key-partitioned) composable with processing logic. Maps to Mister Smith's Transport trait. [R4-Skitter]

---

## Production Readiness

### Observability

- `SpanStart` / `SpanEnd` in `ModelEvent` for near-real-time distributed tracing
- `tokio-metrics` for scheduler starvation detection (`tokio_tasks_spawned`, `tokio_task_poll_duration`, `tokio_workers_idle`)
- Backlog depth and "send waited" time as Prometheus metrics
- W3C TraceContext `correlation_id` propagated through NATS JetStream headers
- **Critical-path extraction** (HybridRCA, Ekhlasi et al., 2025): Focus observability on spans that matter for latency, not all spans. 22.6% fewer spans analyzed. [R4-HybridRCA]

### Chaos Engineering Scenarios

| Scenario | Injection | Expected Behavior |
|:---|:---|:---|
| Broker failure | `kill -9` on NATS node | JetStream Raft failover; brief latency spike |
| Slow consumer | Artificial `sleep()` in tool execution | `MaxAckPending` halts ingestion; TCP backpressure to provider |
| Stream truncation | Drop TCP connection mid-SSE | Actor supervisor detects EOF without Stop; one-for-one restart |

### Benchmarking Targets (from R4)

| Metric | Target | Source |
|:---|:---|:---|
| Messaging throughput | >100K msgs/sec (NATS JetStream) | TBMQ achieves 3M+; Kafka 1.2M+ |
| Intra-agent P95 latency | <50ms | Event-driven architectures achieve 18-22ms |
| Agent restart recovery | <500ms | Local recovery = 50%+ improvement over global |
| Streaming verification overhead | <10% latency increase | Streaming-VR achieves this |

---

## Open Questions & Gaps

1. **Precise numeric backpressure defaults**: Optimal channel sizes (32, 128, 512?) and `MaxAckPending` values for Mister Smith's expected token rates require empirical benchmarking. [R3-B9, R3-B10]

2. **Provider-side backpressure behavior**: Do real LLM providers (OpenAI, Anthropic) actually slow token generation when client consumption slows, or do they buffer/drop? Behavior varies by provider and transport. [R3-B11, R3-B21]

3. **Streaming schema validation**: VPA-based approaches for validating streaming JSON against JSON Schema are academically promising but not production-ready. [R3-C-turn16]

4. **JetStream flow control stall characterization**: Specific configurations that cause "pressure bugs" (mysterious hangs from unhandled `FlowControl` messages) need empirical testing. [R3-C5]

5. **Parser performance under Mister Smith workloads**: Exact CPU/latency profiles for `simd-json` vs `actson` vs `picojson-rs` within streaming tool-call parsing are not profiled. Microbenchmarks needed. [R3-B13, R3-B14]

6. **No integrated supervision + LLM backpressure framework**: Research treats these as separate concerns. Mister Smith's integration is novel and unvalidated at scale. [R4-gap3]

7. **Model-agnostic provider abstraction with streaming backpressure**: Most papers assume a specific LLM backend. The `ModelProvider` trait working across providers while maintaining backpressure is an open area. [R4-gap4]

8. **Disaggregated serving integration timeline**: PrefillShare/Dynamo require GPU-level infrastructure changes. The path from NATS text payloads to KV cache pointer routing is multi-phase. [R7d]

9. **SCM calibration across domains**: The 95% at 18% figure is from specific benchmarks. Performance on diverse agentic workloads (code, planning, retrieval) needs validation. [R6-ref12]

10. **PRM overconfidence**: Off-the-shelf PRMs frequently overestimate success probabilities, breaking adaptive scaling. Quantile regression calibration required. [R6-ref9, R6-ref10]

---

## Implementation Priority for Mister Smith

### Tier 1: Phase 9 (LLM Provider Integration) -- Immediate

| Priority | Component | Rationale | Effort |
|:---|:---|:---|:---|
| 1 | `ModelEvent` enum with `#[non_exhaustive]` | Foundation for all streaming; blocks everything | 1 week |
| 2 | `StreamFinalizer` actor | Deterministic artifact production; correctness gate | 1-2 weeks |
| 3 | Actor-per-stream in supervision tree | Failure isolation; lifecycle management | 1-2 weeks |
| 4 | Bounded `mpsc` + `OwnedPermit` backpressure | Prevents unbounded memory; head-of-line blocking | 1 week |
| 5 | Incremental JSON parser (`actson` + `simd-json` hybrid) | Sub-millisecond partial validation; early tool execution | 1-2 weeks |
| 6 | JetStream pull consumer integration | Distributed backpressure; durable replay | 1-2 weeks |

### Tier 2: Phase 9+ -- Near-Term Enhancement

| Priority | Component | Rationale | Effort |
|:---|:---|:---|:---|
| 7 | Dual-stream design (lossless + UI) | Decouples correctness from presentation | 1-2 weeks |
| 8 | Backpressure policy matrix (per-event-class) | Different event types need different flow control | 1 week |
| 9 | `StreamMap` fan-in for sub-agent aggregation | Multi-agent streaming UX | 1 week |
| 10 | WebSocket transport option for providers | ~40% latency reduction in tool-heavy loops | 1-2 weeks |
| 11 | Step boundary detection (heuristic) | Foundation for step-level intelligence | 1 week |

### Tier 3: Phase 10+ -- Advanced Capabilities

| Priority | Component | Rationale | Effort |
|:---|:---|:---|:---|
| 12 | Streaming Content Monitors (SCM) | 95% detection at 18% tokens; mid-stream abort | 2-4 weeks |
| 13 | JetStream KV CAS for step-level rollback | Lock-free state transfer for micro-rollback | 1-2 weeks |
| 14 | Token Throttling (prefill/decode separation) | 11-398% throughput improvement | 2-3 weeks |
| 15 | Adaptive timeouts with emergent load shedding | 40% avg / 55% tail latency reduction | 1-2 weeks |
| 16 | KV cache management hints in ModelProvider | TTL pinning, priority hints, session resumption | 2-4 weeks |
| 17 | Disaggregated serving / shared KV cache | Eliminates redundant prefill; largest latency win | Research phase |

---

## Sources

### R3: Streaming Architecture Synthesis (Round 3)
- `/Users/matthewmaggio/Mister-Smith/docs/research-output/synthesis/streaming-architecture-R3.md`
- Merged from 3 independent industry research reports (Ultra2x). 50+ references spanning NATS docs, Tokio docs, AI SDK documentation (OpenAI, Vercel, Microsoft), actor framework docs (Ractor, Bastion, Actix), JSON parser crates, distributed systems theory (Lamport, Dean & Barroso).

### R4: Academic Research on Streaming, Backpressure, and Reactive Flow Control (Round 4)
- `/Users/matthewmaggio/Mister-Smith/docs/research-output/research/targeted-streaming-backpressure-reactive-R4.md`
- 57 peer-reviewed papers and ArXiv preprints (2025+), 28 searches via Consensus MCP. Key papers: Streaming-VR (EMNLP 2025), gLLM Token Throttling, Tokencake, KVFlow, TaiChi, CRGC, Skitter, StreamTune.

### R6: Step-Level Intelligence Deep Dive (Round 6)
- `/Users/matthewmaggio/Mister-Smith/docs/research-output/research/targeted-step-level-intelligence-R6.md`
- Ultra2x deep research on PRMs, RSD, CLAI/TALE, streaming monitors, step boundary detection, JetStream CAS. Key references: BiPRM (arxiv:2508.01682), RSD (arxiv:2501.19324), SCM (arxiv:2506.09996), TALE (arxiv:2412.18547).

### R7d: Discovery Sweep (Round 7)
- `/Users/matthewmaggio/Mister-Smith/docs/research-output/research/discovery-sweep-R7d.md`
- PrefillShare (arxiv:2602.12029), SUN (arxiv:2603.02599), NVIDIA Dynamo, disaggregated serving, multiparty session types, biomimetic fault tolerance, game-theoretic mechanism design, AdaptOrch topology routing.
