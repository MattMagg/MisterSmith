# Mister Smith Direction

Date: April 5, 2026
Status: Authoritative

## Purpose and Authority

This is Mister Smith's single authoritative direction source.

Use this file when you need one clear answer to:

- what Mister Smith is
- where Mister Smith is going
- what should be built next
- how current repo truth and research-backed direction fit together

Source split:

- `docs/current-state.md` is the authority for what is currently true on `main`
- the local-only research corpus under `docs/research-output/` provides the research rationale and
  strategic evidence when present
- `spec/` remains the authority for architecture and type contracts

Conflict rules:

- `docs/current-state.md` wins for shipped truth and live-vs-not-live status
- `docs/direction.md` wins for strategic priority and sequencing
- `spec/` wins for architecture, type, and interface contracts

## What Mister Smith Is

Mister Smith is a multi-agent orchestration operating system in Rust. Its product boundary is the
runtime substrate, supervision model, transport, persistence, routing, execution, task/session
state, and operator-facing runtime surfaces shipped from this repository.

Linear, Symphony, Ralph, and SpecKit are not part of the Mister Smith operating system. They are
external development workflow tools used to plan, stage, execute, review, and land work on the
repo.

## Current Position

Already landed or established foundations:

- the Rust workspace substrate through Phase 10
- supervised planner and executor lifecycles on the default runtime path
- ToolBus-backed execution on the default runtime path
- real local provider-backed runtime proof on the supported `openai_chatgpt` / `gpt-5.4` baseline
- bounded same-agent sessions and operator-visible provenance
- bounded runtime routing, budget, verifier-gated orchestration, and repair-lineage foundations
- deterministic packet-021 predictive-supervision evidence surfaces on `main`
- landed packet-022 durable workflow core ownership on `main`
- landed packet-023 runtime-truth and run-trace projections on `main`
- landed packet-024 agent-boundary security hardening on `main`
- landed packet-025 step-policy summaries on `main`
- landed packet-026 coordinator-runtime delegation and proof projections on `main`

Exists but is still opt-in, partial, or not the default runtime path:

- config-gated multi-provider runtime routing and budget enforcement
- additive external-agent interoperability surfaces
- broader orchestration, supervision, and coordination ideas that have repo foundations but are
  not yet the unqualified default path

Should not be described as fully live yet:

- any opt-in path without default-path proof
- any deterministic-only packet result without a fresh live rerun
- any research idea without repo-grounded implementation and validation
- any development workflow system as if it were part of the shipped operating system

## Frontier Mandate

Mister Smith is not being built to follow the current agent-framework market. It is being built to
define the standard that market later converges toward.

That means:

- do not copy the defaults of OpenAI Agents SDK, Google ADK, LangChain, CrewAI, AutoGen, Claude
  SDK, or similar systems
- benchmark existing systems, learn from them, then exceed them
- prefer architectures with long-term leverage in coordination, supervision, routing, memory,
  execution, reliability, observability, state, and distributed behavior
- reuse correct primitives when they are already strong, but do not normalize on popular weak
  patterns
- favor supervised autonomy, strong execution boundaries, and standard-setting design over
  incremental imitation

## Direction Priorities

### Already-Landed Foundations To Extend

The next work should extend the foundations already present on `main`, not restart them from
scratch. Dynamic orchestration foundations, session continuity, bounded runtime routing, verifier
gates, provenance, predictive-supervision evidence, runtime-truth projections, and agent-boundary
hardening already exist in bounded form and should be treated as base layers to harden and
generalize.

### Now

- **Coordinator-runtime follow-through and proof hardening.** This matters because the runtime has
  already moved past the first bounded coordinator-runtime landing and now needs follow-through
  that keeps delegation, child-state, and proof surfaces honest as the broader runtime hardens.
  This belongs now because packet `026` is already landed on top of packet `022` through `025`
  foundations, so the next work should extend and stabilize that surface rather than re-stage it
  as future work.
- **Streaming and routing hardening.** This matters because the runtime path already has bounded
  routing, budget, verifier, and provenance foundations that should become more robust and more
  honestly default over time. This belongs now because the system should finish and harden what is
  already partially real before opening new orchestration surfaces.
- **Benchmark and observability proof.** This matters because strategic claims need runtime proof,
  comparative evidence, and AI-native observability instead of theory alone. This belongs now
  because Mister Smith should prove its advantages while the substrate is still becoming the stable
  baseline.
- **Predictive supervision hardening and live proof.** This matters because the packet-021 through
  packet-026 surfaces are landed on the supported runtime path but still benefit from fresher
  whole-path live rerun. This belongs now because the honest gap is proof and hardening, not
  another greenfield supervision packet.

### Next

- **Dynamic orchestration and topology extension.** This matters because topology selection and
  adaptive team composition are among the strongest whole-system differentiators in the research
  corpus. This belongs next because Mister Smith already has orchestration foundations, so the
  honest move is extension and compiler-like control, not a greenfield orchestration rewrite.
- **Capability discovery and interoperability.** This matters because external federation and
  capability matching are important future surfaces for a real operating system rather than a
  single-runtime tool. This belongs next because it is more valuable once security boundaries,
  routing discipline, and supervision are stronger.

### Later

- **Hybrid CRDT coordination.** This matters because observation-driven shared-state coordination
  can become a major differentiator when used selectively and grounded in the existing JetStream
  substrate. This belongs later because it should sit on top of a more mature runtime contract,
  not replace the current coordination model prematurely.
- **MPST protocol safety.** This matters because compile-time choreography guarantees can remove
  classes of coordination bugs in critical agent interactions. This belongs later because the
  highest-value protocols should first stabilize enough to justify formal session-type encoding.
- **Persistent KV cache and neural paging.** This matters because resume economics and
  large-context recovery can improve dramatically with stronger memory persistence and paging
  control. This belongs later because workflow durability and orchestration shape should settle
  before heavy cache and paging investment.

### Not Yet

- **MAS^2-style architecture generation.** This matters as frontier R&D, but it is too early to
  place under the main product roadmap. This stays out of the near-term plan because the lower
  substrate still offers more reliable leverage.
- **RL puppeteer orchestration.** This matters as an experimental control strategy, but it is not
  yet an honest platform default. This stays out because it would add complexity before the runtime
  contract is mature enough to support it safely.
- **Auction and game-theoretic task allocation.** This matters for future large-scale coordination
  experiments, but it is not a near-term operating-system priority. This stays out because better
  execution boundaries, supervision, and orchestration structure come first.
- **Biomimetic swarm extensions and similar frontier experiments.** These matter as research
  probes, not current product commitments. They stay out because Mister Smith should first finish
  the strategically clearer substrate and differentiation layers above.

## What We Will Not Do

- We will not make packet-centric framing the main story of the system.
- We will not copy framework defaults just because they are popular or familiar.
- We will not describe opt-in or partially wired paths as default truth.
- We will not collapse Linear, Symphony, Ralph, SpecKit, or other development workflow tools into
  the product architecture.
- We will not restart already-landed foundations from zero when the honest move is to extend,
  harden, or generalize them.

## Source Map

- overall direction -> `docs/direction.md`
- current truth -> `docs/current-state.md`
- research basis -> local-only `docs/research-output/` corpus when present
- contracts -> `spec/`
- workflow control plane -> `WORKFLOW.md` and local-only `docs/linear/` notes when present
