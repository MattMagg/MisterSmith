# Advanced streaming architecture for Mister Smith (Rust + Tokio + NATS JetStream)

Executive summary

- Recommended foundation: a composable Tokio Streams-based pipeline that emits a typed event stream (Stream<Item = ModelEvent>) bridged to supervised actors (one actor per logical stream or a bounded pool when necessary) and optionally persisted or routed via NATS JetStream for durability and distributed fan-out. This design combines incremental parsing, explicit stream finalizers, backpressure plumbing through bounded Tokio channels and JetStream consumer flow control, and structured incremental JSON handling using streaming/parsing crates with repair strategies. The approach minimizes repeated full-document parsing, enables deterministic finalization of structured outputs, and integrates supervision and observability for production reliability. (Supported by sources on streaming agents, structured finalizers, Tokio Streams, tokio mpsc backpressure, and NATS JetStream features.) [1], [2], [6], [9], [8]

Contents

- Overview and design goals
- Per-dimension findings and actionable guidance (1-8)
  - For each: (a) state of the art, (b) key techniques, (c) applicability to Tokio + NATS, (d) implementation complexity, (e) expected impact vs naive chunk-by-chunk streaming
- Cross-cutting synthesis: recommended end-to-end Mister Smith architecture, APIs, and rollout plan
- Evidence gaps and concrete experiments
- References

Overview and design goals

Mister Smith’s Phase 9 objective is to replace naive SSE-chunk parsing with a production-grade pipeline that supports incremental structured-output parsing, backpressure, multiplexing/fan-out, finalization into validated artifacts, actor supervision, and strong observability while remaining low-allocation and composable with Tokio async and NATS JetStream. The following sections analyze state-of-the-art approaches and provide concrete, implementable guidance for each research dimension.

1) Stream finalization & aggregation

(a) State of the art (primary sources)
- Microsoft Semantic Kernel / Agent Framework: explicit streaming APIs allow registering an intermediate callback (on_intermediate_message) and provide ResponseStream objects whose built-in finalizer assembles all streaming updates then parses structured output into a concrete class; callers can request a final parsed response via get_final_response(). This demonstrates a "stream finalizer" pattern that collects deltas and produces a validated final artifact only once assembly is complete. [1], [2]
- Practical LLM JSON repair/validation: community Rust work documents repair steps (fix trailing commas, quotes, unquoted keys), a "partial/streaming JSON" endpoint that minimally closes strings/brackets for UI display but not as the final source of truth, and an enforce API that can coerce types and report changes made. These emphasize that repair is often necessary before final validation and that the stream must be considered authoritative only after finalization. [3]

(b) Key techniques / algorithms
- Stream finalizer pattern: collect stream deltas (typed events or raw tokens) into a bounded accumulator keyed to the logical response; expose an API to (1) emit intermediate typed events for UI or tool execution, (2) perform incremental validation where possible, and (3) run a finalizing pass that repairs, validates, deserializes, and returns the final artifact. [1], [2], [3]
- Two-phase assembly: emit deltas for early consumption while keeping a canonical assembled buffer for final parse/validation on completion. [1], [2]
- Repair + enforce stage: deterministic, small-step repairs (trailing commas, quote fixes) and optional coercions with audit logs (changes_made) to produce a validated object. [3]

(c) Applicability to Tokio Streams + NATS
- Tokio Streams provide a natural async sequence to feed a local finalizer actor that incrementally receives ModelEvent items and appends to an in-memory assembler; once completion is signaled, the actor runs final validation/deserialization. [6]
- JetStream can persist intermediate messages or the final assembled result for durability; however, final parsing should remain a local operation (to avoid repeated remote parsing costs) while JetStream can store the final artifact or the stream of events for replay. JetStream’s persistent streams are suitable for storing the sequence of deltas, enabling consumers to reconstruct and finalize later. [8]

(d) Implementation complexity
- Medium. Implementing a robust finalizer requires: (1) typed upstream events (so finalizer knows which sequences to assemble), (2) a bounded accumulator with durable optional persistence, (3) deterministic repair rules and schema-aware validation hooks, and (4) well-tested edge-case handling (interleaved tool calls, truncated JSON). Using existing framework patterns (Microsoft’s ResponseStream) reduces design risk but adaptation to Rust+Tokio requires integrating repair libraries and supervision. [1], [2], [3]

(e) Expected impact vs naive chunk-by-chunk streaming
- High. Finalization avoids attempting to deserialize incomplete concatenated chunks on every delta, reduces repeated parsing work, provides a single validated artifact for downstream logic, and enables actionable repair reporting for operator/debugging workflows. It also separates transient UI rendering from authoritative output-improving correctness and debuggability. [1], [2], [3]

2) Reactive stream-processing patterns

(a) State of the art
- Reactive and streaming libraries and patterns inform operator sets and backpressure strategies (Rx-style observables in Rust via rxrust; high-level stream-processing crates such as RS2 that include built-in backpressure and stateful operators; Tokio’s Stream trait and combinators used as the base async primitive). [4], [5], [6]

(b) Key techniques / operators
- Operators that matter for LLM output pipelines: map (transform deltas), filter (drop heartbeats/pings), buffer/window (batching or coalescing tokens), merge/select/fan-in (multiplexing streams), zip/combine (coordinating multiple streams), flat_map (outer stream producing inner streams), and concurrency-limiting operators (buffered with a fixed concurrency). Backpressure strategies include bounded buffers with await-on-send semantics or explicit drop policies. [4], [5], [6]

(c) Applicability to Tokio Streams + NATS
- Tokio Streams and futures::Stream form a suitable foundation for reactive composition in Rust; operators such as buffered(n) (see tokio_stream::StreamExt::buffered) allow bounded concurrency while preserving order, and SelectAll can merge multiple streams into a single processing stream. Where more advanced stateful operators are required, crates like RS2 provide higher-level primitives. JetStream can be the durable source/sink while Tokio streams handle in-process reactive composition. [6], [5], [7], [8]

(d) Implementation complexity
- Low-Medium. Basic composition with tokio_stream combinators is low complexity (map, filter, buffer, buffered), but implementing advanced stateful operators (windowing, session tracking, circuit breakers) or porting RS2 semantics into the actor-supervised model raises complexity to medium. Using an existing high-level library (RS2) reduces custom work but requires vetting its production maturity. [5], [6]

(e) Expected impact vs naive chunk-by-chunk streaming
- High. Reactive combinators enable modular composition, predictable error propagation, ordered concurrency limits, and clear backpressure behavior-removing ad hoc parsing loops and enabling reuse of tested operators (e.g., buffered, SelectAll) for concurrency/ordering guarantees. [6], [7]

3) Stream multiplexing and fan-out (stream-of-streams)

(a) State of the art
- Patterns for stream-of-streams include outer streams yielding inner streams; typical building blocks are fan-out operators, SelectAll for merging, and buffered concurrency to limit parallel inner tasks. Tok io examples show SelectAll merging and buffered(n) preserving order for concurrent tasks. JetStream provides durable streams and features (rollup, sources/mirrors) useful for shaping message retention and subject-level purging. [7], [8]

(b) Key techniques / coordination strategies
- Preserve per-agent ordering: treat each sub-agent stream as an ordered sequence and tag events with (agent_id, sequence_number) so per-agent consumers can enforce local ordering even after global merge.
- Global ordering strategies: a) interleaved real-time merge (first-come-first-served), b) priority/weighting, c) causal or causal+vector timestamps (when causal ordering is required).
- Fan-out/fan-in: outer stream spawns inner streams; use SelectAll or custom merge with per-inner stream watermarks, plus bounded concurrency to avoid resource exhaustion. Use per-inner timeouts and cancellation to control lifecycle.

(c) Applicability to Tokio + NATS
- Tokio’s SelectAll and buffered combinators are directly applicable to merging and controlled concurrency of inner streams in-process. JetStream can host multiple subject streams and be configured to mirror or source streams; its retention and rollup semantics allow pruning prior messages when only the latest state matters (AllowRollup / Nats-Rollup headers). For distributed fan-out across nodes, JetStream subjects and durable consumers provide ordered delivery guarantees per subject. [7], [8]

(d) Implementation complexity
- Medium-High. Implementing multiplexing that preserves various ordering constraints and supports cancellation/timeouts requires careful per-stream metadata, efficient tagging, and robust supervision. Adding distributed guarantees (across multiple instances) increases complexity due to JetStream consumer configuration and potential message replay/flow-control interactions. [7], [8]

(e) Expected impact vs naive chunk-by-chunk streaming
- High. Multiplexing enables simultaneous sub-agent streaming and coordinated aggregation, supports planners that spawn parallel tool-using agents, and prevents head-of-line blocking that naive aggregation causes. JetStream-backed fan-out supports reliable cross-node distribution and recovery. [7], [8]

4) Backpressure and flow control

(a) State of the art
- Tokio bounded mpsc channels provide natural backpressure: send awaits when the buffer is full. JetStream implements flow control (server sends status when pending limits hit) and consumer-side settings (MaxAckPending, AckWait, rate limits) to throttle delivery. SSE and WebSocket have different backpressure semantics: SSE is server→client only (automatic reconnection, Last-Event-ID), but each SSE connection consumes resources and can be a scaling bottleneck; WebSockets are bidirectional but lack a standard receive-side backpressure mechanism (send buffers can fill). [9], [10], [11], [8]

(b) Key techniques / algorithms
- Bounded channels with await-on-send semantics (Tokio mpsc) to propagate backpressure locally.
- JetStream consumer flow control (maxAckPending, flow control statuses, rateLimit) to push backpressure across networked consumers.
- Application-level policies: DropNewest/DropOldest/Block (configurable per-subscriber) for SSE hubs, timeouts on send to avoid indefinite suspension, and slow-consumer callbacks for logging and adaptation. [9], [10], [11]

(c) Applicability to Tokio + NATS
- Use bounded tokio::sync::mpsc channels for in-process backpressure; when bridging to JetStream, configure consumer MaxAckPending and RateLimit to shape ingestion and enable server-to-client flow-control interaction. For SSE or WebSocket frontends, implement per-connection buffers with explicit drop policies or timeouts to avoid blocking the whole pipeline. [9], [10], [11], [8]

(d) Implementation complexity
- Low-Medium. Local backpressure via Tokio channels is straightforward. Coordinating JetStream consumer flow-control settings, implementing adaptive rate-limiting, and making reliable decisions across distributed components add medium complexity. SSE policies and send-timeout handling are modest but require careful testing. [9], [10], [11]

(e) Recommended backpressure heuristics (practical starting points)
- Local buffer sizes: start with small bounded channels (e.g., 32-256 events) and observe behavior per workload; use buffered concurrency to limit parallel tool executions.
- JetStream: set MaxAckPending to a value that reflects expected concurrency per consumer and enable flow control / rateLimit when consumers are slow.
- SSE/WebSocket frontends: apply send timeouts (to close stale connections), and per-client drop policy (DropOldest for realtime UIs; DropNewest when preserving earlier context matters).
Note: the findings include tokio channel behavior and JetStream flow-control primitives; exact numeric defaults must be benchmarked across workloads. [9], [10], [11], [8]

(f) Expected impact vs naive chunk-by-chunk streaming
- High. Explicit end-to-end backpressure prevents unbounded memory growth, avoids slow-consumer collapse, and ensures system stability under token bursts. Using JetStream flow control extends these guarantees across distributed deployments. [9], [10], [8]

5) Incremental structured-output parsing

(a) State of the art
- Several Rust crates and approaches address streaming/incremental JSON parsing: experimental json-stream-parser (incremental character-level updates), picojson-rs (zero-allocation slice and stream parsers suitable for constrained scenarios), simdjson’s on-demand API (forward-only, skip-unused-fields, streaming semantics), streaming_serde_json (incremental serde deserializers for large docs), json-event-parser, and actson (push-based feeder with NeedMoreInput events and async Tokio readers). Community patterns show deterministic repair steps and buffered NDJSON for streaming structured outputs. [12], [13], [14], [15], [16], [3]

(b) Key techniques
- Token-buffered incremental parser: feed bytes/tokens into a push-based parser (actson-like) that signals NeedMoreInput and emits partial events (start-object, key, value fragment).
- NDJSON/JSON-lines when feasible: avoids partial-object assembly problems by producing one object per line and using a buffered decoder to handle chunk-boundary splits.
- On-demand parsing (simdjson) for speed when skipping unused fields is acceptable.
- Deterministic repair + final schema validation: apply conservative repairs during finalization and record changes (audit trail). [12], [13], [14], [15], [16], [3]

(c) Applicability to Tokio + NATS
- Use a push-based incremental parser (actson or json-event-parser) in a Tokio task to accept bytes from provider streams and emit structured ModelEvent fragments; for persisted streams via JetStream, store raw deltas and allow replays to the parser for reassembly/finalization. For performance-sensitive paths, picojson-rs provides zero-allocation parsing if the input can be presented as slices. simdjson on-demand is applicable when forward-only parsing and skipping fields is acceptable. [16], [15], [13], [14], [12], [8]

(d) Implementation complexity
- Medium-High. Integrating a robust incremental parser with repair logic, schema-aware checkpoints, and recovery for truncated/invalid fragments requires careful design. Using established push-based parsers lowers risk; relying on experimental crates increases it (json-stream-parser is not production-ready). [12], [15], [16], [13]

(e) Expected impact vs naive chunk-by-chunk streaming
- High. Incremental parsing avoids repeated full-document parses (reducing quadratic parsing costs), allows earlier detection of structural problems, supports streaming tool-call extraction, and reduces latency for downstream consumers that can act on partial structured fragments. [14], [12], [3]

6) Stream-as-actor pattern

(a) State of the art / precedents
- Erlang/OTP supervision model (supervisors with one-for-one, one-for-all, rest-for-one restart strategies) is a canonical precedent for supervising ephemeral stream-processing workers. Rust actor frameworks (Ractor) and ractor-supervisor crates implement Erlang-like semantics and support supervisors, restart strategies, and backoff. Supertrees projects port experimental OTP-style supervision to Rust. These show feasibility of actor-per-stream supervision in Rust. [17], [18], [19], [60]

(b) Key techniques
- Actor-per-stream: spawn a supervised actor/task per logical streaming session; actor owns the assembly buffer, per-stream backpressure channel, and finalizer. Supervisor enforces lifecycle: restart on transient errors, escalate on repeated failures, and enforce timeouts.
- Pooled-worker hybrid: when per-stream actor overhead is high, use a pool that owns worker tasks and routes stream identifiers through bounded queues with per-stream metadata.
- Supervisor policies borrowed from OTP (one-for-one, rest-for-one) to control recovery semantics. [17], [18], [19]

(c) Applicability to Tokio + NATS
- Use Tokio tasks to implement actors; supervision semantics and restart strategies are available via ractor and ractor-supervisor crates to provide Erlang-style behavior in Rust. JetStream provides durable replay / message redelivery to allow restarted actors to recover unprocessed segments. [18], [19], [8]

(d) Implementation complexity
- Medium. Implementing supervisors and actor lifecycles with restart/backoff is feasible using existing crates, but careful attention is needed for resource limits (task counts) and restart semantics when streams are long-lived. Hybrid pooling policies add extra complexity to routing and per-stream isolation. [18], [19], [17]

(e) Expected impact vs naive chunk-by-chunk streaming
- High. Actor-per-stream provides clear isolation, fine-grained lifecycle control, and direct supervisory recovery for streaming failures (network blips, parse errors). It simplifies routing partial outputs to other agents (one actor forwards partial output messages to another actor). Using supervisors and durable JetStream storage improves resilience and observability. [17], [19], [8]

7) Event-typed streaming

(a) State of the art
- Modern AI SDKs use typed streaming events with start/delta/end patterns and discriminated event types (Vercel AI SDK: text-start/text-delta/text-end; AI SDK UI protocol enumerates tool-input-start, tool-input-delta, tool-input-available, tool-output-available, finish, abort, error). OpenAI streaming also emits delta events and explicit completion markers. These typed streams enable incremental UI rendering, tool execution orchestration, and finalization. [20], [21]

(b) Candidate ModelEvent shape and rationale
- A rich enum reduces ad hoc parsing and enables typed handlers and supervision. Candidate variants (non-exhaustive):
  - TextStart { id, metadata }
  - TextDelta { id, token_delta }
  - TextEnd { id, finish_reason }
  - ToolCallStart { call_id, tool_name }
  - ToolInputDelta { call_id, chunk }
  - ToolInputAvailable { call_id, input_object }  // when tool input assembled
  - ToolResultAvailable { call_id, result_object }
  - StructuredFragment { schema_id, fragment }     // partial JSON fragment
  - UsageDelta { tokens_in, tokens_out }
  - SpanStart/SpanEnd { span_id, metadata }
  - Error { code, message, recoverable }
  - Heartbeat/Ping
Rationale: aligns with Vercel/AISDK and OpenAI delta/start/end patterns; separates tool-call lifecycle from generic text deltas; enables observability events (UsageDelta, SpanStart/SpanEnd). [20], [21]

(c) Observability and billing events
- Emit usage deltas incrementally (many SDKs expose token counts in streaming events) to enable near-real-time billing/monitoring. Emit SpanStart/SpanEnd for tracing and to correlate events to distributed traces. Error events should be typed with a recoverable flag to let supervisors decide whether to terminate or resume. [21]

(d) API sketches (Rust types)
- Two compact sketches (design proposals - not factual claims) to implement ModelEvent:

Sketch A (enum + lightweight payloads)
  ```rust
  pub enum ModelEvent {
      TextStart { id: String, metadata: EventMeta },
      TextDelta { id: String, chunk: Bytes },
      TextEnd { id: String, finish_reason: Option<String> },
      ToolCallStart { call_id: String, tool: String },
      ToolInputDelta { call_id: String, chunk: Bytes },
      ToolInputAvailable { call_id: String, input: serde_json::Value },
      ToolResultAvailable { call_id: String, result: serde_json::Value },
      StructuredFragment { schema: String, fragment: serde_json::Value },
      UsageDelta { tokens_in: usize, tokens_out: usize },
      Error { code: u16, message: String, recoverable: bool },
      Heartbeat,
  }
  ```

Sketch B (event with typed envelope)
  ```rust
  pub struct EventEnvelope {
      pub session: SessionId,
      pub offset: u64, // per-session sequence
      pub body: EventBody,
      pub trace_ctx: Option<TraceContext>,
  }

  pub enum EventBody { Text(TextEvent), Tool(ToolEvent), Obs(ObservabilityEvent), Err(ErrorEvent) /* ... */ }
  ```

These shapes mirror typed streaming protocols from Vercel/AISDK and OpenAI that separate start/delta/end and tool-call semantics, enabling typed handlers and finalizers. [20], [21]

(e) Complexity & expected impact
- Low-Medium complexity to implement the enum and adapt providers’ SSE or chunk protocols into it. High impact: typed events make composition, supervision, observability, and finalization deterministic and testable, and they simplify incremental parsing and tool orchestration. [20], [21]

8) WebSocket vs SSE vs NATS native transports for long-running agentic loops

(a) State of the art
- SSE: simple server→client streaming with automatic reconnection and Last-Event-ID resume; suitable for unidirectional server pushes and works easily over proxies and HTTP ports. SSE consumes a TCP connection per client with scaling resource implications. [22]
- WebSocket: full-duplex low-latency persistent connection, supports binary frames and bidirectional messages, but needs explicit reconnection/heartbeat handling and can require more server complexity. WebSocket receive-side backpressure lacks a standard; developers implement application-level ack/drain handling. [22]
- NATS / JetStream: high-performance messaging with sub-millisecond latency, pub/sub, request/reply, and persistent JetStream for at-least-once and stronger delivery semantics; JetStream supports replication, mirroring, and features like rollup and consumer flow control for scalable distributed streaming. If JetStream is already used for distribution and persistence, the need for application-level WebSocket persistence depends on client topology and whether clients are co-located with JetStream consumers. [23], [8]

(b) Tradeoffs for agentic loops
- WebSocket preferred when interactive, bidirectional, low-latency exchanges and binary payloads are required; SSE preferred for simple server push with easier browser support and automatic reconnection. If JetStream is used for inter-service routing/persistence, WebSocket remains useful at the client boundary (browsers or external apps) while JetStream handles durable, distributed messaging between services. JetStream does not replace the need for WebSocket when direct browser or external client connectivity and bidirectional messaging are required. [22], [23], [8]

(c) Complexity & expected impact
- Medium. Choosing transport involves operational considerations (scaling TCP connections for SSE, heartbeat and reconnection logic for WebSocket, JetStream consumer tuning for distributed reliability). Using JetStream for server-to-server durability and WebSocket/SSE for client edges is a pragmatic hybrid. [23], [22], [8]

Cross-cutting synthesis - recommended end-to-end architecture for Mister Smith

Design principles (synthesizing the above)
- Base primitive: Tokio Streams + typed event envelopes (ModelEvent) for all in-process streaming. Use tokio_stream combinators (buffered, SelectAll) for composition and controlled concurrency. [6], [7]
- Typed events: normalize provider SSE/WebSocket chunk formats into ModelEvent (start/delta/end, tool call lifecycle, structured fragments, usage and trace events). This aligns with Vercel/AISDK and OpenAI patterns and simplifies finalizers and supervision. [20], [21]
- Finalizer actor per logical response: each session/response is handled by a supervised actor that receives ModelEvent, appends to the assembler buffer, issues intermediate callbacks, and on TextEnd/ToolInputAvailable runs repair/validation + schema enforcement to produce final artifacts; supervisor restarts on transient failures per OTP patterns. Use ractor or ractor-supervisor crates to implement supervisors. [17], [19], [18]
- Incremental parsing: implement a push-based incremental parser (actson or json-event-parser) inside the actor to accept structured fragments and emit partial structured events; fall back to deterministic repair and run jsonschema validation on final assembled value. For high-throughput cases, consider simdjson on-demand or picojson-rs for zero-allocation parsing. [16], [15], [14], [13]
- Backpressure plumbing: use bounded tokio::sync::mpsc channels between producers (provider stream readers) and finalizer actors to propagate local backpressure; configure JetStream consumer settings (MaxAckPending, RateLimit, flow-control) for distributed flow control. Apply per-connection drop policies or timeouts at SSE/WebSocket frontends to avoid head-of-line blocking. [9], [10], [11], [8]
- Multiplexing & fan-out: use SelectAll for merging inner streams, tag events with session/agent IDs, and preserve per-agent ordering in consumers; JetStream subjects and stream rollup/retention can be used to manage distributed fan-out and keep only the latest state where desired. [7], [8]
- Observability and billing: emit UsageDelta and SpanStart/SpanEnd events as part of ModelEvent stream to enable near-real-time metrics and billing attribution. Correlate with tracing context in envelopes. [21]
- Persistence and routing: use JetStream to persist sequences of ModelEvent for durability, distributed replay, and cross-node routing. Use rollup headers when only the latest per-subject state is wanted, and durable consumers for recovery. [8]

Concrete primitives and API sketches (minimal examples)

- Stream<Item = ModelEvent> (simplified)
```rust
pub type ModelStream = impl Stream<Item = Result<ModelEvent, StreamError>> + Send;

pub enum ModelEvent {
    TextStart { id: String, meta: Meta },
    TextDelta { id: String, chunk: Bytes },
    TextEnd { id: String, finish_reason: Option<String> },
    ToolCallStart { call_id: String, tool: String },
    ToolInputDelta { call_id: String, chunk: Bytes },
    ToolInputAvailable { call_id: String, input: serde_json::Value },
    ToolResultAvailable { call_id: String, result: serde_json::Value },
    UsageDelta { tokens_in: usize, tokens_out: usize },
    Error { code: u16, msg: String, recoverable: bool },
    Heartbeat,
}
```
(Design aligns with SDKs that use start/delta/end + tool-call lifecycle.) [20], [21]

- Stream finalizer API (sketch)
```rust
struct StreamFinalizer {
    rx: mpsc::Receiver<ModelEvent>,
    schema: Option<JsonSchema>,
}

impl StreamFinalizer {
    async fn run(mut self) -> Result<FinalArtifact, FinalizeError> {
        // accumulate, forward intermediate events, on end run repair + validate
    }

async fn get_final_response(&mut self) -> Result<FinalArtifact, FinalizeError> { /* ... */ }
}
```
This mirrors ResponseStream/get_final_response style found in Microsoft Agent Framework. [1], [2]

- Supervisor policy (sketch)
  - one-for-one restart for transient errors (parse failure, network blip)
  - escalate after N restarts with exponential backoff
  - for parsing errors mark artifact failed and emit Error event with recoverable=false only after repair attempts exhausted
These policies follow OTP style supervision documented in Erlang/OTP and implemented by ractor-supervisor patterns. [17], [19]

Zero-cost / low-allocation choices (evidence-based)
- Use zero-allocation parsers (picojson-rs) when inputs can be provided as slices and deterministic memory control is required. Use simdjson on-demand when skipping unused fields is acceptable to reduce memory and CPU overhead. Prefer bytes/buffer reuse across deltas rather than cloning strings when producing ModelEvent chunks. [13], [14]

Production reliability: timeouts, retries, metrics, tracing
- Per-actor timeouts and supervisor-driven restarts (OTP-style). Use JetStream durable consumers for resume on restart and MaxAckPending / flow control to prevent server overload. Incremental usage events + SpanStart/SpanEnd for tracing and billing. On SSE/WebSocket frontends, apply send timeouts and connection limits to avoid resource exhaustion. [17], [8], [21], [11]

Implementation complexity and rollout plan

Complexity summary (rough)
- Stream finalizer + typed events + tokio_stream composition: Medium (reusing design patterns from Microsoft and Vercel reduces design risk). [1], [20]
- Incremental JSON parsing with production-grade repair and schema-aware validation: Medium-High (some parsing crates are experimental; rigorous recovery logic required). [12], [15], [3]
- Actor-per-stream supervision and JetStream integration: Medium. Existing crates (ractor, ractor-supervisor) provide building blocks; distributed corner cases add complexity. [18], [19], [8]
- End-to-end distributed multiplexing with ordering guarantees: Medium-High depending on chosen ordering semantics. [7], [8]

Rollout plan (prototype → production)
1. Prototype (2-4 weeks)
   - Implement provider adapters that map SSE/WebSocket chunks into ModelEvent Stream using tokio_stream.
   - Implement a StreamFinalizer actor that accumulates events, emits intermediate typed deltas, and returns a final artifact on TextEnd.
   - Use in-memory bounded mpsc channels and local supervision via a simple task wrapper.

   Evidence: mapping SSE/OpenAI/Vercel events into start/delta/end patterns is standard in SDKs. [20], [21]

2. Integration testing / load testing (2-6 weeks)
   - Add incremental parser (actson/json-event-parser) into finalizer and run workloads: short-chatty, long multi-step agentic loops, token bursts.
   - Measure memory, latency, parsing CPU; test backpressure with bounded channels.

Evidence: tokio channels and buffered operators provide backpressure primitives; JetStream flow control can be simulated later. [9], [7]

3. JetStream persistence & distributed rollout (2-4 weeks)
   - Persist ModelEvent stream into JetStream subjects; configure durable consumers, MaxAckPending, and RateLimit; test consumer restarts and replay.
   - Replace local supervision restarts with supervisors capable of rehydrating state from JetStream replay when actor restarts.

Evidence: JetStream supports durable storage, consumer settings, rollup and mirrored streams for durability and distributed routing. [8], [10]

4. Production hardening (ongoing)
   - Implement observability (usage events, tracing), operational limits on connections (SSE/WebSocket), and robust repair heuristics with audit logging.
   - Scale worker pools, tune channel sizes, and instantiate per-region JetStream clusters as needed.

Test scenarios to validate design (minimum three realistic workloads)
- Short chatty streams with many concurrent agents: many short streams emitting small deltas, large concurrency; validate low per-stream latency, bounded memory, and fair scheduling using buffered(n) and bounded mpsc. (Use tokio_stream::buffered and mpsc to validate.) [7], [9]
- Long-lived multi-step agentic loops: streams that interleave tool calls and model responses over long durations; validate actor supervision, per-stream ordering, and finalizer correctness (including repair of structured outputs). (Use finalizer + incremental parser and supervisor restart to verify.) [1], [16], [17]
- High-throughput token bursts: single or few streams emitting tokens at very high rate; validate backpressure propagation from consumer to provider, JetStream flow control behavior, and system stability under full buffers. (Adjust JetStream MaxAckPending and RateLimit in tests.) [9], [10], [8]

Measurable expected improvements over naive chunk-by-chunk streaming
- Latency: earlier UI rendering retained for incremental tokens, but final artifact latency reduced for consumers that avoid repeated full-parses (incremental parser + finalizer) - avoids quadratic parse behavior from repeated full deserializations. [14], [1]
- Resource savings: bounded channels and JetStream flow control prevent unbounded memory growth; zero-allocation parser options reduce heap usage. [9], [13]
- Failure recovery: actor supervision + durable JetStream persistence reduces lost progress and enables replay on restart; typed ModelEvent reduces ambiguity and simplifies recovery logic. [17], [8], [20]
- Correctness: finalizer + repair + schema validation produce a validated final artifact, avoiding brittle downstream behavior caused by partial or malformed JSON. [2], [3]

Evidence gaps and required empirical experiments

- Precise numeric backpressure defaults (channel sizes, MaxAckPending) to use for representative workloads are not specified in the findings and require benchmarking across Mister Smith’s expected token rates and concurrency. The findings provide primitives but not tuned defaults. [9], [10]
- Production maturity and performance tradeoffs of some streaming-parsing crates (json-stream-parser, streaming_serde_json) are flagged as experimental or not production-ready; empirical validation on target workloads is needed before adoption. [12], [15]
- End-to-end producer-side throttling semantics for SSE/WebSocket in the context of LLM providers (whether the producer can be slowed by client backpressure) is not detailed in the findings; behavior varies by provider and transport, so experiments with real providers are required. [11], [21]
- Exact CPU/latency profiles for simdjson on-demand vs picojson-rs vs streaming_serde_json within Mister Smith workloads are not present in the findings; microbenchmarks are needed to decide parser selection. [14], [13], [15]

Suggested experiments / benchmarks
- Backpressure sweep: measure latency, memory, and throughput across channel buffer sizes (32, 128, 512) and JetStream MaxAckPending values under token-burst workloads.
- Parser correctness & latency: feed malformed and partial JSON tool-call fragments to candidate parsers (actson, json-event-parser, picojson-rs, simdjson) and measure recovery rate, latency, and CPU cost.
- Multiplexing & ordering: spawn planners that fan out to N sub-agents streaming concurrently (N=3,10,50) and verify per-agent ordering and end-to-end assembly correctness.
- Supervisor recovery: inject transient network failures and parser panics to verify actor restart, JetStream replays, and final artifact correctness.

Works Cited / References

[1] https://learn.microsoft.com/en-us/semantic-kernel/frameworks/agent/agent-streaming
[2] https://learn.microsoft.com/en-us/agent-framework/agents/structured-output
[3] https://dev.to/mtdevworks/i-built-an-api-for-llm-json-validation-in-rust-heres-what-i-learned-36nc
[4] https://sdpr.rantai.dev/docs/part-vi/chapter-41/
[5] https://crates.io/crates/rs2-stream
[6] https://www.qovery.com/blog/a-guided-tour-of-streams-in-rust
[7] https://oneuptime.com/blog/post/2026-01-25-infinite-data-streams-tokio-async-rust/view
[8] https://docs.nats.io/nats-concepts/jetstream/streams
[9] https://docs.rs/tokio/latest/tokio/sync/mpsc/fn.channel.html
[10] https://nats.io/blog/jetstream-java-client-03-consume/
[11] https://medium.com/@moali314/server-sent-events-a-comprehensive-guide-e4b15d147576
[12] https://crates.io/crates/json-stream-parser
[13] https://github.com/kaidokert/picojson-rs
[14] https://simdjson.org/api/0.6.0/md_doc_ondemand.html
[15] https://crates.io/crates/streaming_serde_json
[16] https://docs.rs/actson
[17] https://erlang.org/documentation/doc-5.6/pdf/design_principles.pdf
[18] https://www.reddit.com/r/rust/comments/113dp70/ractor_not_just_another_actor_framework/
[19] https://docs.rs/ractor-supervisor
[20] https://ably.com/docs/guides/ai-transport/vercel-ai-sdk/vercel-message-per-response
[21] https://developers.openai.com/api/docs/guides/streaming-responses/
[22] https://softwaremill.com/sse-vs-websockets-comparing-real-time-communication-protocols/
[23] https://github.com/nats-io/nats.rs