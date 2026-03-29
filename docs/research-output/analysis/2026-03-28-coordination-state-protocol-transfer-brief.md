# Post-Research Analysis Brief: Coordination, State, and Protocol Transfer

**Imported report:** `docs/research-output/inbox/deep-research-report (1).md`  
**Primary evidence base:** imported report above  
**Repo-local implementation files inspected:** `crates/mister-smith-app/src/execution.rs`,
`crates/mister-smith-transport/src/envelope.rs`,
`crates/mister-smith-persistence/src/kv/state.rs`,
`crates/mister-smith-core/src/autonomy.rs`,
`crates/mister-smith-core/src/supervision.rs`,
`crates/mister-smith-supervision/src/tree.rs`,
plus supporting reads in `crates/mister-smith-app/src/conversation.rs`,
`crates/mister-smith-agents/src/tool_bus.rs`,
`crates/mister-smith-nats/src/jetstream.rs`,
and `crates/mister-smith-events/src/autonomy.rs`  
**Router-only repo context:** `docs/current-state.md`,
`docs/plans/2026-03-16-frontier-direction.md`  
**Existing research context used only for novelty judgment:** `docs/research-prompts/R8/05-crdt-formal-verification.md`,
`docs/research-output/consolidated/05-coordination-and-state.md`,
`docs/research-output/CLAUDE.md`,
and `specs/021-profile-aware-predictive-runtime-supervision/research.md`  
**Analysis goal:** no explicit `<analysis_goal>` tag was present; this brief uses the user objective as the governing question  
**Decision horizon:** whole-system future direction first; bounded packet work second, only where a current packet is a legitimate consumer

## Executive Assessment

The imported report does **not** materially change Mister Smith's direction. It mostly **sharpens**
an already-accepted research direction while exposing a gap between current repo-level research
ambition and the supported runtime path.

The strongest transfer is not "adopt CRDTs and MPST now." It is narrower and more actionable:

1. treat protocol semantics as first-class runtime metadata rather than implicit conventions
2. make transport assumptions explicit when discussing protocol correctness
3. keep shared-state coordination selective and invariant-driven instead of pretending one
   consistency model fits everything
4. prefer governed semantic updates and replayable journals over silent semantic merge

Repo reality is decisive here. The landed runtime already has strong supervision, durable session
continuity, repair lineage, operator-visible provenance, JetStream-backed budget enforcement, and
a ToolBus-based execution boundary. What it does **not** have on the supported runtime path is a
protocol registry, role automata, transport-parametric implementability checks, CRDT-backed shared
state, coordination primitives as reusable replicated objects, or protocol/liveness monitors.

So the imported report should influence **how future coordination work is framed for Mister Smith
as a whole**, while current bounded packet work remains only one downstream consumer of that
direction. It does not justify widening the current packet or claiming that the coordination
substrate is already live.

## Current Implementation Reality

### Landed in code now

- **Supervised runtime execution exists on the supported path.**
  `crates/mister-smith-app/src/execution.rs` wires supervised planner and executor lifecycles,
  runtime routing metadata, ToolBus execution, and JetStream-backed budget reads into the live
  task path.
- **Transport envelopes already carry generic correlation and security metadata.**
  `crates/mister-smith-transport/src/envelope.rs` provides `message_id`, `correlation_id`,
  `trace_id`, `source_agent_id`, `target_agent_id`, arbitrary `headers`, `nonce`,
  `capability_token`, and plane/class markers.
- **Durable session continuity is already implemented.**
  `crates/mister-smith-app/src/conversation.rs` persists `session_id`,
  `coordinator_agent_id`, retained context, workflow linkage, and compensation paths for failed
  launch/setup steps.
- **Repair and provenance contracts are live operator surfaces.**
  `crates/mister-smith-core/src/autonomy.rs`,
  `crates/mister-smith-core/src/supervision.rs`,
  and `crates/mister-smith-events/src/autonomy.rs` define stable checkpoint, repair-directive,
  failure-context, topology, result-provenance, and orchestration-quality records.
- **OTP-style supervision is a landed primitive.**
  `crates/mister-smith-supervision/src/tree.rs` implements the tree structure, restart-scope
  handling, restart-budget checks, and escalation decisions.
- **Current shared state is KV/CAS oriented, not CRDT oriented.**
  `crates/mister-smith-persistence/src/kv/state.rs` offers JetStream KV persistence with
  `LastWriteWins`, `Timestamp`, `Reject`, and explicit CAS updates, plus branch checkpoint/resume
  keys.
- **Tool execution already has a capability-aware boundary.**
  `crates/mister-smith-agents/src/tool_bus.rs` exposes discoverable tool descriptors, policy
  enforcement, audit integration, delegation checks, and event publication.

### Not landed on the supported runtime path

- no protocol registry or compiled local role automata
- no typed protocol metadata contract such as `protocol_id`, `protocol_version`, `role`, or
  `seqnum` enforced at the envelope layer
- no inbox/outbox replay discipline or out-of-order buffering keyed by protocol/session state
- no transport-parametric implementability checker
- no CRDT/RDT shared-state substrate in runtime code
- no ARDT/PRDT-style coordination objects
- no local protocol safety or liveness monitors
- no decentralized hyperproperty monitoring
- no governed semantic-update pipeline for long-lived shared memory/spec state

### Planned or prior-research only

- The R8 coordination baseline and consolidated coordination corpus already accept a three-tier
  hybrid model: CRDTs for shared artifacts, JetStream KV CAS for invariants, and NATS/JetStream
  for routing and durable effects.
- `specs/021-profile-aware-predictive-runtime-supervision/` explicitly keeps CRDT coordination,
  MPST protocol verification, and event-triggered consensus **out of scope** for the current
  bounded packet.

### What the imported report actually collides with

The report collides most directly with the current envelope and state story:

- the envelope already has extension points, but not first-class protocol discipline
- the state layer is still CAS-centric and workflow-specific, not a reusable replicated-state plane
- observability is strong for repair/provenance, but not for protocol conformance or liveness

## Findings That Merit Consideration

### 1. Protocol metadata plus replay/buffering semantics should become a real runtime seam

- **What it is:** The report's Accompanist-style transfer says each sessioned interaction should
  carry explicit protocol/session metadata and tolerate duplicates, reordering, and delayed
  delivery through buffering and replay.
- **Why it matters:** This is the most natural next step from the current envelope contract.
  Mister Smith already has generic headers, replay-relevant provenance, and stable session/workflow
  identifiers; what is missing is protocol-aware structure instead of generic metadata buckets.
- **Evidence strength:** Strong. This is one of the report's best-supported and most transferable
  mechanisms.
- **Current implementation fit:** Good fit, but clearly absent today.
  `envelope.rs` can already carry the metadata, while `execution.rs` and `conversation.rs` already
  persist session/workflow continuity. The repo does **not** yet implement protocol IDs, role
  state, seqnums, out-of-order buffering, or inbox/outbox replay.
- **Evidence vs inference:** Imported evidence supports the mechanism. Code-level repo context
  shows the extension points exist. The judgment that this should become a future runtime seam is
  an architectural inference.
- **Decision:** `influence implementation now`
  This should shape the next coordination-oriented design packet, but it should not widen the
  current bounded packet.

### 2. Transport-aware protocol implementability is strategically important, but not a live-code commitment yet

- **What it is:** The report's Sprout(A)-style claim is that protocol correctness depends on the
  actual transport/buffer semantics, not just on abstract typing or projection.
- **Why it matters:** Mister Smith runs on NATS/JetStream, where ordering, replay, redelivery, and
  consumer behavior are part of the semantics. That makes transport-parametric checking more
  relevant here than in generic framework comparisons.
- **Evidence strength:** Strong as a design warning, moderate as a near-term implementation recipe.
- **Current implementation fit:** Low current fit in live code. The repo has no protocol registry,
  no MPST-generated automata, and no transport-parametric CI checker. Existing research already
  accepted MPST and hybrid coordination conceptually, and the current bounded supervision packet
  explicitly defers them.
- **Evidence vs inference:** Imported evidence strongly supports the risk. Repo docs and research
  context show this is already treated as later work. The judgment that it should remain a narrow
  prototype seam rather than a current runtime requirement is repo-local inference.
- **Decision:** `prototype next`
  Prototype a narrow critical-protocol checker later; do not treat this as supported runtime truth
  now.

### 3. Selective strong coordination is a better fit than broad CRDT-first expansion

- **What it is:** The report argues that shared state should be classified by invariants, with
  strong coordination used only where invariants require it and reusable coordination primitives
  kept separate from convergent state.
- **Why it matters:** This is the clearest way to keep Mister Smith standard-setting without
  overcommitting to fashionable CRDT rhetoric. It also matches what the current code already
  suggests: budgets and checkpoints need stricter semantics than generic shared observations.
- **Evidence strength:** Moderate-to-strong. The report's LoRe/CONLOC/ARDT/PRDT discussion is
  compelling directionally, but the automation-heavy parts are still research-grade.
- **Current implementation fit:** Partial fit. `state.rs` already reflects a coordination-first
  CAS surface for strict state, but there is no runtime state taxonomy, no CRDT layer, and no
  reusable coordination-object library.
- **Evidence vs inference:** Imported evidence supports invariant-driven selectivity. Code-level
  repo context shows the current system is already closer to explicit coordination than to
  CRDT-first replication. The recommendation to formalize a manual state taxonomy first is an
  inference.
- **Decision:** `prototype next`
  The near-term move is a hand-curated state taxonomy plus one reusable coordination primitive, not
  automated mixed-consistency inference.

### 4. Local protocol safety/liveness monitoring is more transferable than decentralized hyperproperty monitoring

- **What it is:** The report distinguishes lightweight local protocol monitors and liveness checks
  from heavier system-wide decentralized hyperproperty monitors.
- **Why it matters:** Mister Smith already values runtime proof and operator-visible evidence. A
  local protocol/liveness layer would strengthen that posture more directly than jumping to
  expensive global monitor synthesis.
- **Evidence strength:** Moderate. The local-monitoring case is stronger and more implementable
  than the hyperproperty-monitor case.
- **Current implementation fit:** Partial fit. The repo has rich autonomy/provenance surfaces and
  stream/supervision signals, but no protocol conformance monitor, no deadlock/liveness monitor,
  and no hyperproperty monitor.
- **Evidence vs inference:** Imported evidence supports both categories, but the prioritization is
  driven by current code and bounded-work scope. The repo-local inference is that local monitoring
  is the honest first step and decentralized hyperproperties are still too expensive to pull
  forward.
- **Decision:** local protocol/liveness monitoring -> `prototype next`  
  decentralized hyperproperty monitoring -> `monitor`

### 5. Governed semantic updates are strategically right, but not implementation-ready on the current runtime path

- **What it is:** The report treats semantic updates to long-lived memory/spec state as explicit,
  journaled, impact-analyzed changes rather than silent merges.
- **Why it matters:** This is architecturally important for a system that wants durable memory,
  replay, and trustworthy operator surfaces. It is also the cleanest answer to semantic-conflict
  optimism in the wider CRDT conversation.
- **Evidence strength:** Moderate. The report makes a persuasive governance case, but the actual
  mechanism space is still immature.
- **Current implementation fit:** Low fit on the supported path. The current runtime has retained
  context and result projections, but it does not expose a structured shared-memory graph or a
  semantic-change workflow.
- **Evidence vs inference:** Imported evidence supports the governance principle. Code-level repo
  context shows the memory substrate required to implement it is not yet present. The judgment that
  this is a later-stage governance seam is inference.
- **Decision:** `monitor`
  Keep it as a north-star rule for future memory work, not a near-term bounded implementation seam.

## Novelty Relative To Mister Smith

### Mostly already aligned

- hybrid coordination thinking that separates convergent state from invariant-critical state
- MPST and protocol verification as future coordination candidates
- CRDT/state-taxonomy interest as a later frontier stream
- JetStream-backed replay, provenance, and durable evidence as core strengths

### Genuinely new or materially sharper

- **Accompanist-style session metadata plus buffering/replay semantics**
  This is more concrete than the repo's current coordination baseline and maps well onto existing
  envelope/session surfaces.
- **Sprout(A)-style transport-parametric implementability framing**
  The repo already liked MPST in the abstract; this report sharpens that the transport model itself
  has to be part of the proof story.
- **Semantic-governance workflows**
  Prior repo research knew semantic conflicts existed; this report gives a clearer operating model
  for treating them as governed changes rather than merge side effects.

### Not new enough to change direction

- broad CRDT enthusiasm on its own
- MPST as a later coordination frontier
- selective coordination as a principle

Those were already present in repo-local research; the imported report mainly improves the
mechanism detail and ordering.

## Further Research Needed

### 1. JetStream-native dissemination choice for a future shared-state plane

The report is not enough to choose between op-based deltas, state-based sync, snapshot-plus-delta,
or a hybrid for Mister Smith's actual JetStream topology and recovery behavior.

### 2. Repo-specific state taxonomy and invariant inventory

The imported report supports selective coordination in the abstract, but Mister Smith still needs a
small, concrete inventory of which objects are convergent, which are invariant-critical, and which
need reusable coordination primitives.

### 3. Cost model for local versus decentralized monitoring

The report argues for monitors, but not with enough repo-specific evidence to choose which
workflow-level invariants justify lightweight local monitors versus audited global monitors.

### 4. Semantic-update correctness under concurrent agent memory mutation

The report is directionally useful, but it does not tell Mister Smith how to model concurrent
semantic updates once shared memory becomes structured and durable.

### 5. Minimal protocol runtime proof boundary

Before implementation, Mister Smith needs a narrower follow-up that defines:

- which protocol family is worth formalizing first
- what exact envelope metadata is required
- how buffering/replay would be bounded
- what the operator-visible proof surface would be

## Bottom Line

Mister Smith should take this imported research seriously as a **coordination-architecture
sharpening report**, not as a mandate to pull CRDTs, MPST, or semantic-merge machinery directly
into the current bounded packet.

The immediate architectural consequence is narrower and stronger: future coordination work should
promote **protocol metadata, replay discipline, explicit transport assumptions, and selective
coordination boundaries** to first-class runtime concepts. The imported report is most valuable
where it turns broad prior research agreement into more concrete transfer mechanisms.

What should not happen is just as important: Mister Smith should not describe the current runtime
as if it already has a live protocol plane or shared-state coordination substrate. Today it has the
supervision, provenance, session, envelope, and KV/CAS foundations those future layers would build
on.
