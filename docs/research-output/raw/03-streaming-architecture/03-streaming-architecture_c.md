# Advanced Streaming Architecture for LLM Agent Systems

## Stream finalization and aggregation

**Current state of the art**  
Several production-grade agent frameworks have converged on a “two-phase” streaming model: (a) emit incremental updates for UI/progress, and (b) run a **finalizer** that consumes the whole update stream to produce a validated “final artifact.” A clear, explicit implementation of this pattern appears in Microsoft Agent Framework’s `ResponseStream`, which wraps an async stream of updates plus a `finalizer` function (e.g., `AgentResponse.from_updates`) to compute the final response after streaming finishes. citeturn11view0turn2search0  
Microsoft Agent Framework extends this into structured outputs: its streaming API returns a `ResponseStream` you can iterate for real-time text, then call `get_final_response()`; the docs explicitly state the stream’s **built-in finalizer** handles structured output parsing so the final response yields a parsed typed value. citeturn10view0turn2search3  
In the OpenAI Agents SDK, streaming is treated as a first-class run lifecycle: you must consume the event iterator until it ends; only then is the run considered complete, and post-processing (session persistence, approvals bookkeeping, history compaction) may still occur after the last visible token. citeturn12view0turn2search1  
At the model-API layer, OpenAI’s Responses streaming is explicitly “semantic event” streaming: a typed sequence of lifecycle/content/tool events (e.g., `response.output_text.delta`, `response.completed`, and dedicated “arguments delta/done” events for function calls) rather than raw token bytes; this makes implementing deterministic finalizers much easier than ad‑hoc SSE chunk concatenation. citeturn6view0turn0search9  
On the UI protocol side, Vercel’s AI SDK defines an SSE protocol with explicit start/delta/end parts (including tool-input start/delta/available, tool-output available) and explicit message/step termination markers, including a `[DONE]` end-of-stream marker. This is effectively a wire-level finalization contract for “stitched” multi-step/tool-driven assistant runs. citeturn13view0turn0search3  

**Key techniques**  
The highest-leverage “finalization” techniques in today’s frameworks are:

- **Explicit finalizer hook**: wrap `Stream<Update>` with a finalizer that reduces updates into a final response object (and optionally a typed structured value), as in Microsoft Agent Framework’s `ResponseStream(..., finalizer=...)`. citeturn11view0turn2search0  
- **Lifecycle correctness over UI tokens**: declare that “stream completion” happens when the iterator ends (not when you receive an apparent stop token), enabling post-run work to continue after last visible output (OpenAI Agents SDK). citeturn12view0turn2search1  
- **Semantic event models**: represent streaming output as a typed event log (text deltas + tool-call deltas + done events + errors), as in OpenAI Responses streaming. citeturn6view0turn0search9  
- **Step demarcation** for multi-call “agentic loops”: emit explicit step boundaries so clients can reason about multiple backend LLM calls/tools stitched into one conversation turn (Vercel’s `start-step` / `finish-step`). citeturn13view0turn0search3  

**Applicability to Rust async + NATS**  
This pattern maps cleanly onto a Rust signature like `Stream<Item = Result<StreamChunk, LlmError>>`, but only if you treat “final response” as a **separate product** from “streamed deltas.” Microsoft Agent Framework’s pattern is directly portable as a Rust abstraction: `ResponseStream<Update, Final> { stream, finalizer }`, where `finalizer` is a state machine implementing `fold`/`try_fold` across updates. citeturn11view0turn2search0  
The key is to define a finalizer contract that (1) can be driven to completion even if a consumer stops listening to deltas, and (2) cleanly publishes final artifacts into JetStream (e.g., “final answer,” “final tool calls,” “usage/cost,” “trace summary”) as separate messages. This mirrors the “consume stream or call `get_final_response()`” option described in Microsoft Agent Framework structured output streaming. citeturn10view0turn2search3  

**Implementation complexity**  
Moderate to high. A robust finalizer must handle: cancellations, early consumer disconnects, partial tool JSON, provider-specific event ordering, and “run not complete until iterator ends” semantics. citeturn12view0turn2search1  

**Expected impact**  
Big improvement over naive “parse SSE and yield text” because it creates (a) deterministic final artifacts, (b) stronger correctness guarantees about run completion, and (c) a foundation for backpressure-aware buffering and schema validation. This is particularly important because tool arguments often arrive as *incomplete JSON fragments* during streaming. citeturn16search1turn13view0  

## Reactive stream processing patterns

**Current state of the art**  
The mainstream “reactive” model for high-performance streaming is shaped by the Reactive Streams contract: downstream consumers signal **demand**, and upstream publishers must not emit beyond that demand, preventing unbounded buffering. citeturn3search1  
Akka Streams explicitly frames backpressure as a demand protocol (“Subscriber demand”) and guarantees it will not emit more than signaled demand. citeturn3search2  
In the Erlang/OTP ecosystem (highly relevant to “supervision tree” thinking), Elixir’s GenStage formalizes demand-driven pipelines between processes: consumers request N events; producers emit at most N, providing built-in backpressure. citeturn9search1turn9search23  
In JavaScript, RxJS largely centers on rich composition operators (map/filter/merge/window/buffer), while “backpressure” is more commonly expressed as buffering/windowing/throttling operators, since the browser Observable model is not inherently demand-driven like Reactive Streams. citeturn9search0turn9search22  

**Key techniques**  
For LLM output processing, the reactive techniques that matter most are:

- **Stateful transforms (`scan`/reducers)**: accumulate text, build tool-call state, track JSON brace depth, compute partial structured outputs. (This is the “finalizer as fold” approach.) citeturn11view0turn12view0  
- **Windowing/buffering**: batch tokens/deltas into time or size windows to smooth bursty producers (RxJS `bufferTime` illustrates the idea; Akka discusses batching/windowed strategies to amortize async-boundary costs). citeturn9search22turn3search18  
- **Flattening “stream of streams”**: a common multi-agent pattern is an outer stream that yields per-agent inner streams. Rust’s `StreamExt::flatten_unordered` directly encodes this. citeturn4search2  
- **Bounded concurrency operators**: run N tool executions concurrently while keeping streaming responsiveness (Rust has both `StreamExt` combinators and dedicated concurrency crates). citeturn18search2turn4search2  

**Applicability to Rust async + NATS**  
Rust’s `Stream` is pull-based (“does nothing unless polled”), which naturally supports backpressure internally if your producer is implemented idiomatically; the hard parts arise when you bridge from push-based external IO (SSE/WebSocket) into Rust streams, and when you fan-out events to multiple consumers. citeturn4search2turn15search0  
Tokio’s ecosystem supports “stream-first” composition via wrappers like `ReceiverStream` and libraries like `tokio-stream`; they explicitly call out that a common pattern is converting channels into streams, and also reference `async-stream` as an alternative for authoring complex stream logic with `yield`. citeturn18search1turn4search9turn4search1  

**Implementation complexity**  
Moderate. Implementing core operator chains is straightforward with `StreamExt`, but building a **debuggable, allocation-conscious, provider-agnostic** stream graph (with correct cancellation and finalization) is materially harder—exactly the kind of “framework-level” engineering where explicit contracts (typed events, finalizers, backpressure policies) pay off. citeturn12view0turn3search0  

**Expected impact**  
High. Reactive patterns turn “token drips” into **composable pipelines**: you gain testability (each operator is a unit), better observability (operator-level timing/backlogs), and backpressure-aware semantics rather than ad-hoc buffering. citeturn3search1turn19view0  

## Stream multiplexing and fan-out

**Current state of the art**  
Modern agent runtimes increasingly treat “streaming output” as a multiplexed event log rather than a single text stream. OpenAI Responses streaming is already a typed event stream with explicit content and tool-call events, and OpenAI Agents SDK layers higher-level “run item” events (message generated, tool called, tool output, handoff requested) on top of raw token streaming. citeturn6view0turn12view0  
Vercel’s UI stream protocol similarly assigns IDs to blocks (text-start/delta/end) and includes tool-input/tool-output parts with explicit boundaries; this is a practical multiplexing design to support rich front-ends without forcing the UI to infer structure from raw text. citeturn13view0turn0search3  

**Key techniques**  
Multiplexing patterns that transfer well from high-performance streaming domains:

- **Attach ordering metadata**: sequence numbers, timestamps, or step IDs allow deterministic reconstruction even when multiple sources interleave. In real-time media transport, sequence numbers and timestamps exist precisely to reorder and synchronize streams; the RTP specification formalizes this family of approaches. citeturn9search2turn9search21  
- **Watermark-based merging**: in distributed stream processing, watermarks declare “we believe all events up to time T have arrived,” enabling bounded buffering and ordered emission from out-of-order sources. citeturn9search3turn9search11  
- **Stream-of-streams flattening**: represent a planner spawning sub-agents as `Stream<Stream<ModelEvent>>` and flatten with concurrency bounds (`flatten_unordered(limit)`), which exists directly in Rust `futures` as a standard operator. citeturn4search2  
- **Explicit demarcation**: treat each model call/tool round-trip as a “step,” and carry `step_id` in events. Vercel explicitly requires step boundaries to correctly handle “multiple stitched assistant calls.” citeturn13view0turn0search3  

**Applicability to Rust async + NATS**  
On one machine, Rust can multiplex by selecting across multiple streams and emitting a unified `ModelEvent` enum. For distributed multiplexing (across agent processes), JetStream already provides a native multiplexing dimension: **subjects**. JetStream streams persist messages published on configured NATS subjects, and consumers are “views” that can filter by subject. citeturn5search5turn5search0  
This suggests a clean distributed design: publish each agent-run’s events to a subject like `ms.<run_id>.<agent_id>.events.*`, and allow per-subscriber consumers (UI, tracing, persistence, tool execution) to filter server-side. Filter-subject semantics are explicitly supported for consumers. citeturn5search0turn5search30  
Ordering caveat: Core NATS ordering is **per publisher**, not total ordering across publishers. If multiple agents publish concurrently, you must either (a) accept nondeterministic interleaving, or (b) introduce an ordering key/sequence at the application layer. citeturn4search31  

image_group{"layout":"carousel","aspect_ratio":"16:9","query":["NATS JetStream stream consumer diagram","Akka Streams backpressure diagram","GStreamer queue element pipeline diagram","RTP jitter buffer sequence number diagram"],"num_per_query":1}

**Implementation complexity**  
Moderate to high depending on ordering guarantees. “Merge as events arrive” is easy; “merge to a stable causal timeline across distributed publishers” requires explicit metadata (sequence numbers, step IDs, or watermark-like logic) and clear semantics about what “ordering” even means in the UI. citeturn4search31turn9search3  

**Expected impact**  
High. Good multiplexing enables true multi-agent streaming UX (planner + subagents + tools + observations) without sacrificing correctness or debuggability, and it makes NATS/JetStream an architectural advantage rather than just a transport. citeturn5search5turn12view0  

## Backpressure and flow control

**Current state of the art**  
Backpressure is the difference between a streaming demo and a production streaming system. In Rust/Tokio, the foundational primitive is the bounded `mpsc` channel: it is explicitly documented as providing backpressure by buffering up to N messages; once full, `send().await` waits until capacity is available. citeturn3search0turn3search4  
In distributed messaging, JetStream consumers add explicit delivery/ack state, and JetStream even defines a protocol-level **flow control** mechanism: the wire protocol specifies `100 FlowControl Request` messages that must be replied to, otherwise the consumer may stall. citeturn5search23turn4search0  
In high-performance feed systems, backpressure is also “first-class API.” For example, Aeron’s `offer` returns a negative code when backpressured (e.g., `-2`), requiring the application to retry/back off. citeturn17search2  
In media pipelines, buffering/backpressure choices are explicit: entity["organization","GStreamer","multimedia framework"]’s queue elements can block upstream when full, or be configured as “leaky” to drop old/new buffers instead of blocking. citeturn17search1turn17search5  

**Key techniques**  
The most effective backpressure patterns for agent streaming are:

- **Bounded buffers + await-based pressure propagation**: use bounded channels between stages so slow consumers naturally slow producers (Tokio `mpsc::channel`). citeturn3search0turn3search4  
- **Coalescing instead of infinite buffering**: when text arrives faster than downstream can process, merge token deltas into larger chunks (windowing) rather than enqueueing every token. This parallels Akka’s discussion of windowed/batching strategies to amortize async-boundary cost. citeturn3search18  
- **Lossy modes for non-critical streams**: adopt “leaky queue” semantics for UI typing indicators or intermediate deltas, while preserving lossless delivery for semantic boundaries (tool-call start/done, step end, final response). citeturn17search5turn13view0  
- **Transport-level flow control hooks**: in JetStream push consumers, flow control and heartbeat are established patterns; flow control can suspend delivery until the client responds, and idle heartbeats help detect stalled connections. citeturn5search2turn5search23  
- **Backpressure as observability**: treat backlog depth and “send waited” time as metrics and trace events. This is a proven pattern in actor pipelines; entity["company","Quickwit","open-source search engine"] explicitly describes bounded queues to prevent backlog blowups and discusses measuring/reporting backpressure as part of an actor framework. citeturn19view0  

**Applicability to Rust async + NATS**  
Tokio bounded channels are ideal for **in-process** actor mailboxes and per-stream operator boundaries. citeturn3search0turn19view0  
Across processes, JetStream provides several “pressure points”: consumers track ack state (so `max_ack_pending` effectively caps in-flight messages), and flow-control messages can force the client to prove it is keeping up. citeturn5search9turn5search23  
This is a major architectural advantage: if Mister Smith publishes streaming events into JetStream, downstream services can apply backpressure by controlling consumption rate (pull consumers fetch on demand; push consumers can use flow control/ack pending constraints). citeturn5search20turn5search23  

**Implementation complexity**  
High if you want graceful degradation rather than “stall or crash.” You must decide per event class whether to (a) block, (b) coalesce, (c) drop, or (d) divert to disk/JetStream for later replay. The fact that JetStream flow control can stall if not handled correctly is an example of how “pressure bugs” can look like mysterious hangs unless they’re modeled explicitly. citeturn5search23turn3search0  

**Expected impact**  
Very high. Backpressure-aware design prevents tail-latency explosions, unbounded memory growth, and “random stream stops” under multi-tool workloads—failure modes that are common once you move beyond single-call chat. citeturn19view0turn13view0  

## Incremental structured output parsing

**Current state of the art**  
Incremental structured output parsing is currently the “sharp edge” of LLM streaming. Tool/function arguments are usually JSON, but during streaming they are *almost always invalid until completion* (“incomplete JSON problem”). entity["company","Aha!","product management software"] provides a clear illustration: tool arguments arrive as progressively longer prefixes (`{"user`, `{"user":"ali`, …) that cannot be parsed until enough bytes arrive. citeturn16search1  
Providers are starting to acknowledge this explicitly. For example, Amazon Bedrock’s documentation for fine-grained tool streaming warns you may receive invalid or partial JSON tool inputs and should handle those edge cases. citeturn2search14  
Some APIs mitigate the problem by emitting **semantic “done” events** that delimit tool arguments. OpenAI Responses streaming includes function-call-argument delta and done events in its streaming event model. citeturn6view0turn0search9  
At the framework layer, Microsoft Agent Framework leans on finalizers: its streaming `ResponseStream` can parse structured output in the finalizer so callers can stream updates but obtain a validated typed result at the end. citeturn10view0turn2search3  
On the validation side, some “LLM guardrail” tooling has adapted streaming itself: entity["company","Guardrails AI","llm validation tools"] describes rewriting streaming so validators can declare how much context they need (sentence/paragraph/whole output); the runtime accumulates chunks until validators can run, rather than validating per-token. citeturn16search29  

**Key techniques**  
Incremental structured output in production systems generally uses layered techniques:

- **Per-tool-call incremental assembly buffers** keyed by `toolCallId`/`call_id`, plus explicit “arguments done” or “tool-input-available” boundaries when providers supply them. citeturn13view0turn6view0  
- **Incremental parsers / event parsers (SAX-like)** to avoid reparsing whole buffers: Rust crates such as `json-event-parser` provide streaming JSON parsing into events, and crates like `picojson` advertise SAX-style push parsing. citeturn20search4turn20search11  
- **Tolerant parsing for incomplete input**: Rust’s `deser-incomplete` explicitly targets “parse incomplete or broken data with Serde,” calling out streaming JSON as a motivating case because the stream is invalid until done. citeturn20search13  
- **Schema validation after parse**: Rust’s `jsonschema` crate is a high-performance JSON Schema validator, but (like most validators) fundamentally validates a JSON **instance** against a schema, which typically presumes a complete parse tree. citeturn16search3  
- **True streaming schema validation research**: academic work exists on validating streaming JSON against JSON Schema using visibly pushdown automata (VPA)–based approaches, demonstrating that “schema validation while bytes are in flight” is possible but nontrivial. entity["organization","arXiv","preprint repository"] hosts one such approach. citeturn16search12  

**Applicability to Rust async + NATS**  
Rust is well-suited to incremental parsing because you can model parsing as a finite-state machine that consumes `Bytes` chunks and emits typed parse events without allocating intermediate strings unnecessarily (i.e., “SAX-like JSON event streams” rather than “DOM parse every time”). The existence of dedicated streaming/event parser crates and incomplete-tolerant Serde adapters makes this feasible today. citeturn20search4turn20search13  
JetStream can act as a “safety valve” for large structured outputs: persist the raw tool-argument delta stream (for replay/debug) while a local parser actor attempts to assemble/validate it; if validation fails, you still have the exact event log that caused failure for deterministic reproduction. (This aligns with JetStream’s role as a durable message store and with consumer state tracking.) citeturn5search5turn4search0  

**Implementation complexity**  
High. There are three distinct failure classes you must handle explicitly:

- **Incomplete** (waiting for more bytes): common during streaming. citeturn16search1turn20search13  
- **Invalid/partial by provider semantics** (fine-grained tool streaming may emit invalid JSON): a documented edge case. citeturn2search14  
- **Provider/tooling bugs and truncation** (invalid JSON tool args can break sessions, as reported in Agents SDK issue trackers): these show up in real deployments. citeturn16search33turn20search22  

**Expected impact**  
Very high. Incremental structured parsing is what enables “tool execution starts before the model finishes typing the last brace,” which can remove seconds of latency in tool-heavy workflows. It also enables early, user-visible progress (e.g., show which tool is being called and with what partial arguments), which is a major UX differentiator. citeturn13view0turn8view1  

## Stream-as-actor and event-typed streaming

**Current state of the art**  
“Stream as actor” is strongly supported by precedent in actor runtimes:

- In Erlang/OTP-like systems, GenStage is explicitly about exchanging events with backpressure between processes, matching the intuition of “stream = process with a mailbox” and demand-driven flow. citeturn9search1turn9search23  
- In Rust, entity["company","Quickwit","open-source search engine"] describes building a high-throughput ingestion/indexing pipeline as actors connected by bounded queues; it explicitly calls bounded `mpsc` a “natural solution” and frames it as an actor pattern, with supervision/monitoring and message scheduling as framework responsibilities. citeturn19view0  

Event-typed streaming is likewise converging across frameworks: OpenAI’s Responses API streams typed semantic events with a predefined schema; OpenAI Agents SDK exposes both raw response events and higher-level run-item events; and Vercel’s UI protocol defines explicit typed parts for text, tool inputs, tool outputs, steps, errors, and termination. citeturn6view0turn12view0turn13view0  

**Key techniques**  
For a Rust multi-agent orchestration framework, the most transferable techniques are:

- **Per-run stream actor**: model each LLM streaming response as a short-lived actor owning (a) provider connection, (b) incremental parsers, (c) finalizer state, (d) outbound routing. This matches the “bounded queue prevents backlog blowups” argument in actor pipelines. citeturn19view0turn3search0  
- **Supervision-managed recovery**: if the provider stream errors, the supervisor can decide to restart (idempotency permitting), switch transports, or fail the run with structured error events. (This mirrors the “management chores” described in actor frameworks: restart on failure, retries, scheduling, observability.) citeturn19view0  
- **Rich `ModelEvent` enum** that mirrors proven event taxonomies:
  - “raw response” token deltas and lifecycle (`response.created`, `response.completed`, `error`) from OpenAI Responses. citeturn6view0turn21view0  
  - tool-call argument delta/done events and step boundaries for stitched runs (OpenAI Responses + Vercel step parts). citeturn6view0turn13view0  
  - higher-level “run item” events (tool called, tool output, message output created) from OpenAI Agents SDK. citeturn12view0  
- **Dual-stream design**: emit (1) a lossless semantic event stream for correctness/replay, and (2) a best-effort UI stream that may coalesce/downgrade under pressure. Vercel’s protocol already separates fine-grained deltas from “available/done” boundaries, which is the right conceptual split. citeturn13view0turn3search18  

**Applicability to Rust async + NATS**  
This is an excellent fit for Mister Smith’s architecture:

- Actor-per-stream aligns with your OTP-style supervision trees. citeturn19view0turn9search23  
- Mailboxes can be bounded `mpsc` to enforce local backpressure. citeturn3search0turn19view0  
- Cross-actor routing can be modeled as publishing `ModelEvent`s to NATS subjects, with JetStream persistence where reliability/debuggability is required. citeturn5search5turn4search19  

**Implementation complexity**  
Moderate to high. The complexity is not in “spawning an actor,” but in getting lifecycle semantics right: cancellation, consumer disconnect, timeouts, partial JSON parse failures, and ensuring final artifacts are produced even if UI consumers stop reading deltas. OpenAI Agents SDK’s explicit warning that a run isn’t complete until the iterator ends highlights exactly the kind of lifecycle edge that stream-as-actor makes easier to reason about (the actor owns completion). citeturn12view0turn2search1  

**Expected impact**  
High. Stream-as-actor (with typed events) enables:

- deterministic replay/debug (persist event logs),  
- supervision-based recovery,  
- safer fan-out (multiple consumers subscribe to the same canonical event log),  
- and the ability to route one model’s streaming output into another model/tool pipeline in real time. citeturn5search5turn12view0turn13view0  

## WebSocket vs SSE for long-running agent streams

**Current state of the art**  
SSE remains a common default for “stream tokens to the UI” because it is simple, HTTP-native, and unidirectional server→client. entity["organization","Mozilla Developer Network","web platform docs"] explicitly notes SSE is unidirectional and uses the `text/event-stream` MIME type; this makes it suitable when the client does not need to send application messages back over the same channel. citeturn15search4turn15search0  
The SSE ecosystem also includes standardized reconnection behavior (e.g., connection reestablishment and `Last-Event-ID` support are part of the HTML server-sent events model, standardized by bodies like entity["organization","WHATWG","html standard body"]; reconnection semantics also appear in earlier entity["organization","W3C","web standards body"] drafts). citeturn15search6turn15search2  
WebSockets are the standard choice for persistent, bidirectional, interactive sessions; MDN describes the WebSocket API as enabling a two-way interactive communication session between browser and server. citeturn15search1  

**Key techniques**  
The most relevant new development for agent systems is OpenAI’s explicit WebSocket mode for the Responses API: it targets long-running, tool-call-heavy workflows by keeping a persistent connection and continuing each turn with incremental inputs plus `previous_response_id`. citeturn8view1turn8view0  
OpenAI’s documentation claims this reduces per-turn continuation overhead and can improve end-to-end latency substantially in “many round trips” workflows (they cite up to ~40% faster end-to-end execution for rollouts with 20+ tool calls). citeturn8view1  
The same doc also clarifies operational constraints: WebSocket mode runs sequentially (one in-flight response per connection), has no multiplexing, and limits connection duration (e.g., 60 minutes) requiring reconnect/recover strategies. citeturn8view1turn8view0  
On the SDK side, OpenAI Agents JS explicitly states its streaming APIs also work with the Responses WebSocket transport and provides configuration knobs like `setOpenAIResponsesTransport('websocket')`. citeturn15search3turn15search25  

**Applicability to Rust async + NATS**  
For provider integration:

- If your agent loop does many model↔tool round trips, WebSocket mode can reduce overhead by sending only incremental inputs and reusing connection-local state (per OpenAI’s design). citeturn8view1turn8view0  
- If your orchestration layer already uses NATS for persistence and routing, you still need one transport to the LLM provider. WebSocket mode can be advantageous for *provider-side* continuity/latency, while NATS handles *internal* continuity and fan-out.

For client/UI integration:

- SSE is attractive for “just render events,” especially when you leverage standardized reconnection, but it remains one-way. citeturn15search4turn15search6  
- WebSocket is attractive when the UI needs bidirectional interactivity (cancellation, tool approvals, live adjustments), aligning with the general WebSocket contract. citeturn15search1  

**Implementation complexity**  
Moderate. SSE is simpler operationally, but WebSocket mode introduces connection lifecycle management (reconnects, limits, “no multiplexing”) and sequencing for continuation (`previous_response_id`). citeturn8view1turn8view0  

**Expected impact**  
In long tool-call chains, WebSocket mode can materially reduce end-to-end latency (per OpenAI’s own reported measurements) and simplify “incremental input” turn continuation, which directly maps to agentic orchestration loops. citeturn8view1turn6view0  

## Synthesis: recommended architecture for a Rust multi-agent framework

A streaming architecture that is “architecturally superior” to naive SSE chunk streaming should combine four ideas that are independently validated across today’s leading AI SDKs, reactive systems, and high-performance streaming domains:

### Canonical event log first, presentation stream second  
Adopt the OpenAI Responses / Vercel / OpenAI Agents SDK lesson: define streaming as a **typed event stream** rather than “text tokens.” OpenAI explicitly frames Responses streaming as typed semantic events; Vercel defines typed UI stream parts including tool boundaries/steps; OpenAI Agents SDK splits raw deltas from higher-level run item events. citeturn6view0turn13view0turn12view0  

**Recommendation**  
Define `ModelEvent` (rich enum) as the internal canonical log. It should be expressive enough to represent:

- lifecycle (`RunStarted`, `StepStarted`, `StepFinished`, `RunCompleted`, `RunFailed`) aligned with existing step/lifecycle patterns, citeturn13view0turn21view0turn6view0  
- text deltas and “text done,” citeturn6view0turn21view0  
- tool-call start / argument delta / argument done / tool-output available, citeturn6view0turn13view0  
- usage/cost availability at completion (common pattern: usage is reliably available at completion), citeturn21view0turn6view0  
- error events that can be terminal or recoverable depending on scope (provider IO vs tool-arg parse vs tool execution).

Then derive a **presentation stream** (UI deltas, progress indicators) from `ModelEvent` via coalescing/windowing operators so backpressure policies can differ by event importance. This matches the “validators need context” insight from Guardrails AI and the “step boundaries” approach in Vercel. citeturn16search29turn13view0  

### Stream finalizers as first-class, supervised components  
Microsoft Agent Framework shows a concrete “finalizer” hook encapsulated in `ResponseStream(..., finalizer=...)`, and its structured output docs explicitly say streaming plus `get_final_response()` yields a parsed typed result via the stream’s built-in finalizer. citeturn11view0turn10view0  
OpenAI Agents SDK similarly insists a run is not complete until the streaming iterator ends, because post-processing can happen after the last visible token. citeturn12view0turn2search1  

**Recommendation**  
Implement a Rust `StreamFinalizer` trait as a deterministic reducer over `ModelEvent`. It should output:

- `FinalText` (or multi-part content),  
- `FinalToolCalls` (validated JSON),  
- `FinalUsage`,  
- `FinalTraceSummary` (optional),  
- and a “final run state” (`Succeeded`, `Failed`, `Cancelled`, `InterruptedForApproval`). citeturn12view0turn21view0  

Run this finalizer inside a **stream actor** so you can guarantee finalization happens even if downstream consumers stop reading (mirroring “skip iteration entirely; `get_final_response()` consumes the stream” behavior). citeturn10view0turn2search3  

### Backpressure as a policy matrix, not a boolean  
Tokio bounded channels provide backpressure, and JetStream has explicit flow control at the protocol level; both are strong primitives, but neither answers the policy question “what do we do when the UI can’t keep up?” citeturn3search0turn5search23  
Media pipelines solve this by allowing both blocking queues and leaky queues (drop policies), and high-performance transports like Aeron expose explicit backpressure signals. citeturn17search5turn17search2  

**Recommendation**  
Define a backpressure policy per event class:

- **Lossless**: tool-call boundaries, step boundaries, finalization events, approvals, errors.  
- **Lossy/coalescible**: text deltas, “typing” indicators, intermediate partial JSON deltas (while preserving the final “arguments done”). citeturn13view0turn6view0turn17search5  

Implement this with bounded in-process channels plus coalescing operators (window/time/size), and use JetStream as a durable buffer where it makes sense (debug/replay, distributed fan-out). citeturn3search0turn5search5turn3search18  

### Treat each stream as an actor with lifecycle, routing, and recovery  
Actor frameworks in Rust ingestion pipelines explicitly use bounded queues to prevent backlog crash, and they emphasize supervision, scheduling, retries, and observability as core framework responsibilities. citeturn19view0  
GenStage provides the OTP-world precedent for demand-driven stream processing between processes. citeturn9search1turn9search23  

**Recommendation**  
Model each streamed LLM response as a short-lived actor (supervised) that:

- reads provider events (SSE or WebSocket),  
- decodes them into `ModelEvent`,  
- publishes canonical events to NATS subjects (Core for ephemeral UI, JetStream for durable audit), citeturn5search5turn4search19  
- runs the finalizer to produce final artifacts, citeturn11view0turn10view0  
- and enforces backpressure policy on downstream consumers. citeturn3search0turn17search5  

This architecture directly aligns with the “OTP-style supervision trees + actor mailboxes + streaming pipelines” goal and turns streaming into a **managed lifecycle** rather than just a parser loop. citeturn19view0turn12view0  

### Transport choice: SSE for simple egress, provider WebSocket for tool-heavy loops  
For the provider layer, OpenAI’s WebSocket mode is explicitly optimized for long agentic chains with many tool calls and incremental inputs, with documented latency improvements and constraints (no multiplexing per connection, 60-minute limit). citeturn8view1turn8view0  
For UI egress, SSE remains a reasonable default when you only need server→client delivery with standardized semantics; MDN highlights the unidirectional nature and `text/event-stream` usage. citeturn15search4turn15search0  

**Recommendation**  
In a Rust + NATS agent framework, treat transport as swappable:

- Provider transport: SSE by default, optional WebSocket for high round-trip workflows (especially when providers support incremental-input continuation semantics). citeturn6view0turn8view1  
- Client transport: SSE for simple consumption, WebSocket when you need bidirectional control (cancel, approve tools, interactive debugging). citeturn15search1turn13view0  

### Net effect versus naive chunk-by-chunk streaming  
Relative to “parse SSE, yield text” streaming, this architecture delivers:

- **Correctness**: explicit completion semantics (iterator end + finalizer) rather than guessing when output is “done.” citeturn12view0turn11view0  
- **Reliability under load**: bounded queues and explicit flow-control behavior rather than unbounded buffering or silent stalls. citeturn3search0turn5search23  
- **Tool latency wins**: incremental tool argument assembly and early readiness (tool-input available / arguments done) rather than waiting for full responses. citeturn13view0turn6view0turn16search1  
- **Distributed superiority**: JetStream subject-based multiplexing, filtered consumers, durable replay for debugging—turning NATS into a first-class streaming substrate rather than “just transport.” citeturn5search5turn5search0turn4search31  
- **Observability**: event-level tracing and backpressure metrics become natural because the system is already an event log + actor lifecycles (mirroring production actor pipeline practice). citeturn19view0turn12view0