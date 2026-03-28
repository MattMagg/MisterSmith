---
version: R9
created: 2026-03-28
type: prompt
tier: 1
timeline: March 7, 2026 — present
---

# Deep Research Prompt: Real-Time Inter-Agent Communication and Transport

## Context

Mister Smith is a first-class multi-agent orchestration operating system in Rust, built on
NATS/JetStream and Erlang OTP-inspired supervision trees. It is model-agnostic and designed to
define the standard that the agent framework market will converge toward.

The current research corpus already concludes that streaming is not a presentation detail but a
control-plane surface for coordination, failure detection, and routing. The repo already has strong
baseline work on typed stream events, actor-per-stream supervision, JetStream pull consumers,
backpressure, dual-stream designs, and external protocol awareness. This prompt asks what has
changed since the March 7, 2026 baseline, especially for real-time inter-agent communication.

**Fixed inputs**

- `<baseline_boundary>`: March 7, 2026
- `<search_window>`: March 7, 2026 to present
- `<repo_state_router>`: `docs/current-state.md`
- `<routing_manifest>`: `docs/research-output/ROUTING_MANIFEST.md`
- `<baseline_docs>`:
  - `docs/research-output/consolidated/00-MASTER-FINDINGS.md`
  - `docs/research-output/consolidated/05-coordination-and-state.md`
  - `docs/research-output/consolidated/06-streaming-architecture.md`
  - `docs/research-output/consolidated/07-memory-and-context.md`
  - `docs/research-output/consolidated/08-competitive-landscape-and-ecosystem.md`

Use those documents as the authoritative baseline. `R8` is a structure reference only.

## Frontier-First Mandate

Do not choose an approach because it is popular, familiar, or already normalized by OpenAI Agents
SDK, Google ADK, LangChain/LangGraph, CrewAI, AutoGen, Claude SDK, or similar systems. Benchmark
them. Learn from them. Then exceed them.

Pull from real-time messaging, telecom, high-frequency trading, RTC systems, multiplayer game
networking, QUIC/HTTP3 transports, and distributed stream-processing systems when those fields
offer stronger communication patterns.

Reuse what is already correct. Do not reinvent primitives without benefit. But wherever the choice
affects Mister Smith's streaming, routing, latency control, coordination, or distributed behavior,
prefer the architecture with the highest long-term leverage rather than the most conventional
framework transport.

Incremental imitation is failure. Favor designs that create real advantage.

## Research Objective

Survey everything published from March 7, 2026 to the present on real-time inter-agent transport,
bidirectional streaming, backpressure, stream multiplexing, QoS, session resumption, transport
protocol evolution, and transport adapters bridging A2A, MCP, WebMCP, QUIC/HTTP3, WebSocket/SSE,
and NATS-style messaging.

The goal is to discover what has changed since the repo's streaming and protocol baseline and
identify techniques that should influence Mister Smith's real-time communication architecture.

This is an open-ended research task. Go beyond the dimensions below if you discover strong leads.

**Older-source rule**: include older sources only if they are absent from the current repo
baseline and materially change the direction.

## What Has Already Been Researched (Baseline — Do Not Rediscover)

The current research corpus already says Mister Smith should model streaming as typed event
pipelines managed by supervised actors, not raw text chunking. It already covers actor-per-stream,
stream finalizers, bounded in-process backpressure, JetStream pull consumers for distributed
backpressure, dual-stream designs, fan-in via `StreamMap`, and the broad tradeoffs of SSE versus
WebSocket. It also already treats A2A and MCP as important protocol standards and recognizes
WebMCP, QUIC/HTTP3-adjacent surfaces, and shared KV cache routing as meaningful future areas.

The baseline open questions are transport-specific and operational:

- provider-side backpressure behavior remains unclear
- numeric backpressure defaults are uncalibrated
- JetStream flow-control stall modes still need empirical characterization
- model-agnostic provider abstractions with streaming backpressure remain open
- real-time communication semantics across heterogeneous protocols remain thinner than the stream
  pipeline design itself

Treat the following as already known:

- typed supervised streaming is the correct baseline architecture
- backpressure is a first-class systems problem
- real-time agent communication is broader than parsing SSE chunks
- protocol evolution matters because Mister Smith should speak external standards without copying
  their internal architecture

Do not rediscover those findings. Only surface work that materially contradicts, sharpens, or
extends them.

## Research Dimensions

### 1. Bidirectional and Multiplexed Agent Communication

- What new transport patterns exist for agent-to-agent bidirectional streaming and stream
  multiplexing?
- Are there newer approaches for merging or routing many live sub-agent streams with correctness
  guarantees?

### 2. Backpressure, Flow Control, and QoS

- What new work exists on backpressure for long-running AI or agent streams?
- Are there stronger QoS, fairness, or prioritization models for mixed-priority agent traffic?
- Has anything changed on provider-side or transport-side handling when consumers slow down?

### 3. Session Resumption, Recovery, and Transport-Level Resume

- What new techniques exist for resuming interrupted real-time streams or live sessions?
- Are there stronger transport-level recovery models for partially completed streams or tool calls?

### 4. Protocol Evolution and Adapter Design

- What has changed in A2A, MCP, WebMCP, QUIC/HTTP3, WebSocket, SSE, or related protocol work
  that materially affects agent communication architecture?
- Are there new adapter patterns for bridging heterogeneous protocols into one internal event
  model?

### 5. Adjacent-Field Transfer

- What should Mister Smith borrow from telecom signaling, exchange routing, RTC, and multiplayer
  networking for real-time agent communication?
- Which proven communication patterns remain absent from mainstream agent frameworks?

### 6. Production Evidence and Failure Modes

- What new production reports or postmortems exist for real-time multi-agent communication?
- What transport-level failure modes, congestion patterns, or coordination bugs have surfaced in
  real deployments?

## Per-Dimension Output Structure

For each research dimension, provide:

1. **Current state of the art** — what exists today, with specific citations
2. **Key techniques** — the concrete transport or communication mechanisms discovered
3. **Applicability to Rust + NATS + OTP** — how well the pattern transfers to Mister Smith
4. **Delta from baseline** — what is genuinely new versus the repo's current research corpus
5. **Frontier classification** — classify the finding as `EXTEND`, `TRANSFORM`, or `NEW`, and
   also as `FRONTIER` or `INCREMENTAL`
6. **Mister Smith implementation vector** — name the likely crates, runtime surfaces, or spec
   areas affected; prefer concrete repo surfaces such as `mister-smith-nats`,
   `mister-smith-llm`, `mister-smith-core`, `mister-smith-events`, `mister-smith-app`,
   `mister-smith-mcp`, or `apps/operator-console/`
7. **Evidence status** — classify the finding as `production-validated` or `research-only`
8. **Implementation complexity** — rough effort, prerequisites, and risk

## Synthesis

After completing all dimensions, provide a synthesis that:

- ranks the top 5 findings by strategic value for Mister Smith's transport and communication layer
- identifies **contradictions to current assumptions** in the repo baseline
- separates findings into:
  - `production-validated`
  - `research-only`
  - `thin-results`
- recommends which findings should be prototyped, benchmarked, adopted, or only monitored
- names the likely implementation vectors for the strongest findings
- clearly states where the literature remains weak instead of padding with speculation

## Research Methodology

1. Read the baseline docs named above before searching.
2. Search broadly across March 7, 2026 to present. Include papers, standards updates, releases,
   postmortems, and engineering reports.
3. Follow promising leads into telecom, RTC, trading, and real-time distributed systems.
4. Distinguish protocol headlines from evidence that actually changes transport architecture.
5. If a topic yields thin results, say so directly rather than padding.
