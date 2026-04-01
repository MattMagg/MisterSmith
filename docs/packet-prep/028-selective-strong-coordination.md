# Packet 028: Selective Strong Coordination

## Packet Name

Selective strong coordination

## Why This Packet Exists

Not every shared state needs strong coordination. Not every shared state is safe under eventual
convergence either. Mister Smith needs a later packet that makes those distinctions explicit, so
future coordination work is driven by invariants rather than hype.

## Why This Stage Is Correct

`docs/direction.md` puts hybrid CRDT coordination and MPST protocol safety in the `Later` band.
That is the right posture. This packet should stay later in the sequence and should only become a
real spec once earlier packets clarify:

- durable workflow semantics
- trace and protocol metadata
- safe boundary enforcement
- what "real coordinator runtime" actually needs to coordinate

## Repo Truth Status

- Packet outcome today: `planned-only`
- Foundation truth status: `landed-not-default`
- Extra caution: the case for freezing strong-coordination semantics is still `deterministic-only`
- Live-default today:
  - the default supported runtime does not require stronger coordination beyond the current
    workflow, transport, and repair surfaces
- Landed but not yet a strong-coordination packet:
  - JetStream KV CAS and durable transport subjects are real repo primitives
  - append-only event, provenance, and replay-oriented result fragments already exist in the repo,
    but not yet as one reusable strong-coordination layer
- Deterministic-only today:
  - the current case for freezing selective coordination rests on deterministic proof notes and the
    repo's existing strict-state and transport primitives, not on a dedicated live runtime feature
    slice
- Missing for this packet:
  - one frozen state taxonomy
  - one proof rule for when to choose CAS/serialized state instead of convergent state
  - one decision on whether protocol safety belongs in the first slice or a follow-up child slice

## Current Repo Grounding

### Landed in repo but not the default live path

- JetStream KV CAS for strict serialized state transitions
- append-only event and provenance surfaces that a later coordination layer could reuse
- transport metadata that could carry stronger protocol identity later

### Deterministically grounded but not yet frozen as one packet

- the repo already distinguishes some strict state from loosely shared state in practice, but not
  yet as one explicit runtime taxonomy
- capability discovery and external boundary work creates pressure for stronger protocol semantics
- the coordination transfer brief and related proof notes support the packet direction, but that
  directional case is still not the same thing as a live runtime feature slice

### Missing pieces

- explicit runtime state taxonomy: convergent, coordinated, and effectful state
- reusable strong-coordination primitives
- first-class protocol metadata and liveness monitors
- proof criteria for when stronger coordination is worth its cost

### High-Signal Repo Anchors

- `crates/mister-smith-persistence/src/kv/state.rs`
  - `StateManager::update`
  - `ConflictStrategy`
  - This is the current strict serialized-state primitive.
- `crates/mister-smith-persistence/src/hybrid/manager.rs`
  - branch checkpoint and resume-history reads/writes
  - This is the current SQL-plus-KV coordination seam.
- `crates/mister-smith-persistence/src/hybrid/router.rs`
  - `DataRouter::select_storage`
  - `uses_kv`
  - `agent_state_routes_to_kv_primary`
  - This is the strongest current local state-routing seam for deciding when stricter coordination
    already exists in practice.
- `crates/mister-smith-transport/src/durable.rs`
  - `DurableTransport::durable_publish`
  - `DurableTransport::durable_subscribe`
  - This is the current effect-carrying durable message seam.
- `crates/mister-smith-transport/src/subject.rs`
  - `SubjectTaxonomy`
  - `workflow_subjects`
  - `wildcard_subjects`
  - This is the current protocol-identity naming seam.
- `crates/mister-smith-transport/src/envelope.rs`
  - trace, correlation, and capability-token metadata
  - This is the current transport metadata seam that stronger coordination would likely build on.
- `crates/mister-smith-persistence/tests/kv_tests.rs`
  - `ConflictStrategy::Reject` coverage
  - CAS-update expectations
  - This is the strongest deterministic proof anchor for strict KV coordination behavior.
- `crates/mister-smith-persistence/tests/hybrid_tests.rs`
  - hybrid read/write routing tests
  - This is the strongest deterministic proof anchor for the current SQL-plus-KV split.
- `crates/mister-smith-integration-tests/tests/transport_e2e.rs`
  - workflow and task subject assertions
  - This is the strongest end-to-end transport proof anchor for durable subject usage today.
- `docs/research-output/analysis/2026-03-28-coordination-state-protocol-transfer-brief.md`
  - This is the core research note for the three-tier coordination model.
- `docs/research-output/consolidated/05-coordination-and-state.md`
  - This is the clearest consolidated research authority for selective coordination choices.

### Scope Cut For Later Spec Writing

- Core packet scope:
  - freeze a state taxonomy
  - freeze when CAS-style strict coordination is required
  - define one or two reusable strong-coordination primitives
- Explicitly not core packet scope:
  - repo-wide CRDT rollout
  - generic distributed-systems experimentation
  - full MPST adoption everywhere
- MPST posture:
  - treat protocol safety as a follow-on extension inside the packet only if packet `027` proves a
    stable protocol seam worth freezing
  - do not make MPST the gating reason the packet stays vague

## Official Docs / Primary Sources

- [NATS JetStream model deep dive](https://docs.nats.io/using-nats/developer/develop_jetstream/model_deep_dive)  
  Why it matters: official reference for the strict-state substrate Mister Smith already uses.
- [NATS JetStream Key-Value](https://docs.nats.io/using-nats/developer/develop_jetstream/kv)  
  Why it matters: official CAS and optimistic-concurrency reference for the first likely strong-only
  coordination primitive.

## Conditional Follow-On Source

- [rumpsteak crate docs](https://docs.rs/rumpsteak/latest/rumpsteak/)  
  Why it matters: only pull this in if packet `027` actually freezes a protocol seam and the first
  packet `028` slice explicitly includes an MPST follow-on. It is a library-level follow-on
  reference, not a core packet source by default.

## Research / Directional Papers

- [LoRe paper](https://arxiv.org/abs/2304.07133)  
  Why it matters: research source for selective coordination based on verified safety invariants.
- [CodeCRDT paper](https://arxiv.org/abs/2510.18893)  
  Why it matters: research source for observation-driven CRDT coordination in LLM systems,
  including its limits.
- [Local-first software paper](https://martin.kleppmann.com/papers/local-first.pdf)  
  Why it matters: research source for collaboration and local-first convergence tradeoffs that
  apply to later strong-coordination work.

## Research Findings That Matter

- The coordination transfer brief says protocol metadata, replay, and buffering should come before
  stronger correctness claims.
- The consolidated coordination corpus recommends a three-tier model:
  CRDTs for shared artifacts, JetStream KV CAS for invariants, and streams for durable effects.
- CodeCRDT shows strong syntactic convergence but still semantic conflicts. That matters because
  Mister Smith should not romanticize CRDTs.
- LoRe-style selective coordination is the right direction: coordinate strongly where invariants
  require it, not everywhere.

## Best-Practice Guidance

- Start with state taxonomy, not with implementation zeal.
- Use strong coordination only where correctness invariants justify the latency and complexity.
- Keep CRDT surfaces away from external side effects.
- Treat protocol safety and liveness as separate concerns from replicated state convergence.
- Keep protocol metadata and interoperability ownership in packet `027`; packet `028` should only
  consume a stable protocol seam if one was already frozen.
- Keep the packet later until earlier packets prove which invariants truly need more than the
  current KV/CAS and durable-effect substrate.
- Require measurable proof criteria before introducing stronger coordination into the live path.

## Likely Architecture Shape

- documented state classes: convergent shared artifact, coordinated invariant state, effectful state
- one or two reusable strong-coordination primitives for invariant-critical cases
- optional protocol-safety follow-on only if packet `027` freezes a seam worth protecting
- operator-visible proof of when and why stronger coordination was required

## Risks / Constraints / Non-Goals

- Do not turn this into a repo-wide CRDT rewrite.
- Do not introduce protocol formalisms before the runtime seams they are meant to protect are stable.
- Do not force MPST or CRDTs onto flows that stay simpler and safer with existing primitives.
- Do not claim stronger coordination is a near-term default runtime requirement.

## Open Questions Before Spec Writing

- What exact state taxonomy should Mister Smith freeze first?
- Which live runtime invariants actually need strong coordination?
- Is the first usable strong-coordination primitive strict shared-state coordination only, with
  protocol safety deferred until a later child slice?
- What should be proven in a benchmark or runtime artifact before this packet is declared worth building?
- What is the smallest runtime proof that would justify treating the packet case as more than a
  `deterministic-only` design argument?

## Fixed Constraints Before Spec Writing

- Keep packet `028` about state taxonomy, invariant-driven coordination choices, and one or two
  reusable strong-coordination primitives. Do not turn it into a repo-wide CRDT rollout.
- Treat protocol safety and MPST as a follow-on only if packet `027` freezes a protocol seam worth
  protecting. They are not the default first slice.
- Keep CRDT-style convergent state away from external side effects and durable effect execution.
- Do not move packet `028` out of its later posture until earlier packets prove which invariants
  actually need more than the current KV/CAS and durable-effect substrate.

## Recommended Inputs For Future SpecKit Packet

Read these in order: repo routers -> earlier packet constraints -> packet `027` seam check ->
coordination research notes -> strict-state and durable-effect seams -> official NATS docs ->
optional MPST follow-on only if the packet scope really needs it.

- `docs/direction.md`
- `docs/current-state.md`
- `docs/packet-prep/022-durable-workflow-core.md`
  - use to confirm lifecycle and effect-boundary assumptions before deciding where stronger
    coordination is actually needed
- `docs/packet-prep/023-runtime-truth-and-run-trace.md`
  - use to keep proof claims and operator-visible coordination evidence honest
- `docs/packet-prep/024-agent-boundary-security-hardening.md`
  - use to confirm that any later coordination seam does not bypass the current least-privilege
    and quarantine posture
- `docs/packet-prep/027-capability-discovery-and-interoperability.md`
  - use to confirm whether a stable protocol seam was actually frozen before moving protocol
    safety or richer coordination metadata into packet `028`
- `docs/plans/2026-03-19-ms-77-bounded-external-agent-surface.md`
  - use as the current bounded discovery surface proof note before assuming any richer protocol
    seam exists to consume
- `docs/research-output/analysis/2026-03-28-coordination-state-protocol-transfer-brief.md`
- `docs/research-output/consolidated/05-coordination-and-state.md`
- `crates/mister-smith-persistence/src/kv/state.rs`
  - start from `StateManager::update` and `ConflictStrategy`
- `crates/mister-smith-persistence/src/hybrid/manager.rs`
  - start from branch checkpoint and resume-history read/write helpers
- `crates/mister-smith-persistence/src/hybrid/router.rs`
  - start from `DataRouter::select_storage`, `uses_kv`, and the state-routing tests
- `crates/mister-smith-transport/src/durable.rs`
  - start from `DurableTransport::durable_publish` and `DurableTransport::durable_subscribe`
- `crates/mister-smith-transport/src/subject.rs`
  - start from `SubjectTaxonomy` and the subject taxonomy tests
- `crates/mister-smith-transport/src/envelope.rs`
  - start from trace/correlation/capability metadata
- `crates/mister-smith-persistence/tests/kv_tests.rs`
  - use to keep strict KV coordination claims tied to existing CAS and reject-mode coverage
- `crates/mister-smith-persistence/tests/hybrid_tests.rs`
  - use to keep SQL-plus-KV routing claims tied to deterministic proof
- `crates/mister-smith-integration-tests/tests/transport_e2e.rs`
  - use to confirm which durable transport and subject guarantees are already exercised today
- use the official docs above for current primitives first
- use the directional papers above only after the repo anchors and packet `027` protocol seam are
  understood
- if the future packet deliberately adds an MPST follow-on, then pull in the conditional
  `rumpsteak` source above; otherwise keep it out of the first slice
