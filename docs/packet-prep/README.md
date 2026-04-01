# Pre-Spec Packet Dossiers (022-028)

Date: April 1, 2026  
Status: Working pre-spec packet-prep layer; not a frozen packet-spec set

## Purpose

This directory is the pre-spec knowledge layer for the next future packets after packet `021`.

It exists so a later SpecKit session can start from:

- current repo truth
- packet-shaped scope boundaries
- official-doc and primary-source guidance
- explicit risks, non-goals, and open questions

These files are not specs. They do not freeze implementation. They are prep inputs for later
packet authoring.
They are dossier inputs, not packet commitments.

Cold-start rule:

- do not treat dossier existence as approval to write a packet spec
- re-check `docs/current-state.md` before turning any dossier into a packet
- preserve the dossier boundary unless newer repo truth now forces a scope change

## Authority

Use these sources in this order:

1. `docs/direction.md`
2. `docs/current-state.md`
3. `docs/research-output/consolidated/`
4. `docs/research-output/analysis/`
5. official docs and primary sources linked inside each dossier

If these dossiers conflict with later repo-truth updates, repo-truth wins.
If a later session changes packet order, scope, or proof status, update this README first so the
directory stays usable cold.

## Shared Start-Here

Before a future session writes any packet spec:

1. read `docs/direction.md` for sequencing and non-goals
2. read `docs/current-state.md` for live-vs-landed truth
3. read this README's boundary, truth-status, readiness, and dependency sections
4. read the specific packet dossier you are authoring next plus any prerequisite dossiers named in
   the dependency map below
5. follow that dossier's "Recommended Inputs For Future SpecKit Packet" section in order
6. use `packet-authoring-checklist.md` before freezing anything into a real spec
Use `docs/current-state.md` for `live-default`, `landed-not-default`, `deterministic-only`, and
`planned-only` truth.
Use `docs/direction.md` for packet ordering and strategic sequencing.

## Working Packet Baseline

These dossiers intentionally keep the user-requested packet sequence as the working prep baseline:

1. `022` Durable workflow core
2. `023` Runtime truth and run trace
3. `024` Agent-boundary security hardening
4. `025` Step-level intelligence v2
5. `026` First real coordinator-subagent runtime
6. `027` Capability discovery and interoperability
7. `028` Selective strong coordination

Important caveat:

- `docs/direction.md` still groups future work under broader `Now`, `Next`, and `Later` bands rather than these packet numbers.
- These packet numbers and names should be treated as pre-spec dossier labels, not yet as frozen repo packet commitments.
- The dossiers below call out where the user-requested sequence diverges from the repo's broader direction language.
- A later spec author should still ask whether the packet is actually ready to freeze now before
  writing in `specs/`.

## Boundary Adjustments

- `022` is sharpened to durable workflow semantics, effect boundaries, and lifecycle control. It is not a packet-021 follow-on cleanup lane.
- `023` is treated as the truth-and-trace packet for honest run semantics, operator proof
  boundaries, and execution provenance. It overlaps the repo's broader benchmark and observability
  direction, but stays narrower here.
- `024` stays focused on agent-boundary hardening, quarantine, delegation, identity, and deterministic enforcement.
- `025` extends the landed packet-020 and packet-021 control surfaces. It should not pretend the current `workflow.execute_step` placeholder is already grounded step execution.
- `026` is the first packet that should make the runtime feel like a real coordinator-subagent system, not just a graph compiler plus placeholder execution boundary.
- `027` stays behind stronger security and runtime-truth surfaces. It should not become open-ended federation work.
- `028` is intentionally later and selective. It should stay centered on invariant-driven strong
  coordination and state taxonomy. MPST-style protocol safety is a later sub-slice only if the
  protocol seam from `027` proves worth freezing.

## Truth Status Legend

Each dossier now uses the repo's sharper truth split from `docs/current-state.md`:

- `live-default`: exercised on the current supported runtime path
- `landed-not-default`: present in code or accepted repo artifacts, but not the default live path
- `deterministic-only`: backed by deterministic validation or closure notes, but not yet by a fresh
  default-path live proof
- `planned-only`: directionally accepted, but not landed

In each dossier, this is the **foundation truth status**, not the packet implementation state.
Every packet in this directory is still pre-spec. Do not read `landed-not-default` as "the packet
is partly done." Read it as "the substrate this future packet would build on is already landed."

Each dossier should name one primary foundation truth status. If a packet also depends on a
`deterministic-only` seam, call that out separately as a caution. Do not collapse mixed truth into
one vague label.

This is the minimum truth split a later SpecKit author should preserve. Do not collapse these back
into a generic "partial" bucket.

If a dossier has live-default code paths but only deterministic proof for the newer claim surface,
it should say both explicitly.

## How To Read One Dossier

Every dossier below has two different status ideas:

1. `Packet outcome today`
   - this should stay `planned-only` in this directory until a real packet spec is frozen and
     landed
2. `Foundation truth status`
   - this is only the status of the current repo substrate the future packet would build on

Do not read a foundation status like `landed-not-default` as if the packet itself is partly done.

If a packet builds on landed substrate but also depends on deterministic-only proof surfaces, call
that mix out explicitly. Do not force everything into one misleading single-status sentence.

## Protocol Posture

- `027` owns version-pinned interoperability baselines. Other packets can consume those
  decisions, but they should not silently redefine them.
- `027` should keep the dossier's current A2A `v0.3.0` pin. If a later author wants to move off
  that baseline, that change needs an explicit re-audit instead of a quiet drift to newer pages.
- `027` should start from the MCP versioning page, then keep lifecycle, tools, and authorization
  references on one pinned revision. Do not mix `latest` and versioned MCP pages inside the same
  packet.
- `024` can use MCP security best-practices material as an operational hardening guide, but the
  protocol baseline should still stay on the same pinned MCP revision the dossier set uses
  elsewhere.
- `025` should treat the OpenAI Responses streaming-events reference as the canonical event-schema
  input, with the streaming guide, `responses.create` reference, and function-calling guide as
  companion behavior docs.
- `026` should use external framework docs only as comparator guidance after the repo constraints
  from packets `022` through `025` are already clear.

## Pre-Spec Readiness Summary

All packet outcomes in this directory are still `planned-only`. The labels below describe
foundation truth and dossier readiness, not packet implementation status.

`Ready` here means "ready to begin bounded SpecKit packet authoring without first rediscovering
repo truth." It does not mean "ready to implement on the default runtime path," and it does not
change the packet outcome from `planned-only`.

- `022`
  Foundation truth status: `landed-not-default`
  Dossier ready for future spec authoring: yes, with open design choices
  Main open gate: the seam is clear, the repo already has branch-checkpoint and
  session-durability pieces, and the missing durable event-history and effect-boundary contract is
  now precisely scoped.
  Dependency gate: none beyond current repo truth
- `023`
  Primary foundation truth status: `landed-not-default`
  Extra caution: packet-021 proof-boundary and supervision surfaces used here are still
  `deterministic-only`
  Dossier ready for future spec authoring: yes, with one caution
  Main open gate: the repo already exposes truth and provenance projections, but the packet must
  keep packet-021 deterministic-only evidence separate from the older live-proof baseline.
  Dependency gate: packet `022` should freeze durable identifiers and lifecycle language first
- `024`
  Foundation truth status: `landed-not-default`
  Dossier ready for future spec authoring: yes
  Main open gate: the boundary is clean, and the repo already has real policy, delegation,
  quarantine, and MCP boundary code, but the hardening posture still needs one coherent frozen
  runtime contract.
  Dependency gate: can start from current repo truth, but should preserve packet `016` boundary
  continuity and not wait for `026`
- `025`
  Primary foundation truth status: `landed-not-default`
  Extra caution: packet-021 supervision surfaces used here are still `deterministic-only`
  Dossier ready for future spec authoring: yes, if the first slice stays narrow
  Main open gate: the right first slice is verifier escalation and smarter step control on top of
  existing step records, not a full PRM or training stack.
  Dependency gate: packet `023` should own proof-boundary language before packet `025` freezes
  step-policy terms
- `026`
  Foundation truth status: `landed-not-default`
  Dossier ready for future spec authoring: partly
  Main open gate: the runtime already compiles graphs and carries coordinator IDs, but it still
  lacks honest visible subagent delegation and grounded branch execution.
  Dependency gate: wait for packets `022` through `025` to freeze their reused contracts first
- `027`
  Foundation truth status: `landed-not-default`
  Dossier ready for future spec authoring: partly
  Main open gate: the repo has bounded capability-discovery and MCP compatibility seams, but the
  first real interop contract still needs a stricter version-pinned baseline and a clearer split
  between deterministic discovery proof and live runtime use.
  Dependency gate: wait for packets `022`, `023`, and `024` to freeze lifecycle, truth, and
  boundary contracts first
- `028`
  Primary foundation truth status: `landed-not-default`
  Extra caution: the packet case rests on landed state and transport primitives, but the actual
  argument for freezing stronger coordination is still `deterministic-only`
  Dossier ready for future spec authoring: not yet
  Main open gate: the repo already has strict coordination primitives, but the packet still
  depends on earlier packets proving which invariants truly need stronger coordination and whether
  packet `027` froze a stable protocol seam worth protecting.
  Dependency gate: do not freeze until packet `027` proves whether there is a stable protocol seam
  worth consuming

## Packet Dependency Map

- `022` freezes durable workflow semantics and lifecycle control.
- `023` freezes run-trace and proof-boundary language on top of those durable identifiers.
- `024` hardens trust boundaries before later packets widen delegation or interoperability.
- `025` consumes `023` truth surfaces to make step policy smarter without redefining trace truth.
- `026` should consume `022` through `025`; it is the first packet that should make real
  coordinator-subagent behavior visible.
- `027` should consume `022`, `023`, and `024`; it owns interop and capability mapping, not local
  runtime truth or strong coordination policy.
- `028` should stay later and should only freeze once earlier packets prove which invariants
  actually need strong coordination.

## Cleanest Boundaries

- Cleanest now: `022`, `024`, `025`
- Most dependent on earlier packet clarification: `026`, `027`, `028`
- Needs the strongest "do not overclaim" guardrail: `023`, `026`, `028`

## Cold-Start Packet Entry Map

- `022`: start from `docs/direction.md`, `docs/current-state.md`, the durable-workflow transfer
  brief, the session restart/resume proof note, and the branch-checkpoint persistence seams. Do
  not start from Temporal docs first.
- `023`: start from `docs/current-state.md`, the packet-021 proof-boundary note, the packet-021
  live-evaluation note, the March 28 session-context report, and `crates/mister-smith-app/src/execution.rs`.
  Keep packet `019` and `020` live proof separate from packet `021` deterministic-only proof.
- `024`: start from `docs/current-state.md`, packet `016` closure evidence, the Phase 9.1 security
  contracts, `MS-77` bounded external-agent surface proof, delegation/auth-callout/quarantine
  code, and the ToolBus and MCP enforcement seams. Preserve discover-vs-execute separation.
- `025`: start from packet `023`, then packet `020` and `021` closure notes, then the
  verifier/repair seam in `crates/mister-smith-app/src/execution.rs`. Do not treat placeholder
  step completion as grounded task proof.
- `026`: start from packets `022` through `025`, then the March 28 session-context report and the
  March 27 runtime-planning note. Preserve the smallest-workflow rule and prove real delegation,
  not just visible branching.
- `027`: start from packets `022`, `023`, and `024`, then `MS-77` and packet `016` proof notes,
  then the pinned MCP and A2A baseline docs. Choose one first interop slice explicitly instead of
  drafting a generic federation packet.
- `028`: start only after packet `027` shows there is a stable protocol seam worth reusing. Begin
  from state taxonomy and existing KV/CAS seams, not MPST-first or CRDT-everywhere design.

## Supporting Helper

- [packet-authoring-checklist.md](./packet-authoring-checklist.md): small cold-start checklist for
  deciding whether one of these dossiers is actually ready to become a real SpecKit packet

## What This Dossier Set Already Gives You

- explicit truth-status language instead of one coarse "partial" bucket
- a clear split between `Packet outcome today` and `Foundation truth status`
- concrete repo seams: structs, functions, tests, proof notes, and artifact lanes
- packet dependency gates instead of only packet ordering
- tighter protocol posture where version drift would otherwise produce a bad starting point
- a cold-start entry map so a later session does not have to infer where to begin

## Document Map

- [022-durable-workflow-core.md](./022-durable-workflow-core.md)
- [023-runtime-truth-and-run-trace.md](./023-runtime-truth-and-run-trace.md)
- [024-agent-boundary-security-hardening.md](./024-agent-boundary-security-hardening.md)
- [025-step-level-intelligence-v2.md](./025-step-level-intelligence-v2.md)
- [026-first-real-coordinator-subagent-runtime.md](./026-first-real-coordinator-subagent-runtime.md)
- [027-capability-discovery-and-interoperability.md](./027-capability-discovery-and-interoperability.md)
- [028-selective-strong-coordination.md](./028-selective-strong-coordination.md)

## Cold-Start Use

### Packet Authoring Preflight

Before freezing a future SpecKit packet from this directory:

1. re-read `docs/direction.md` and `docs/current-state.md`
2. confirm the dossier truth labels still match current `main`
3. confirm any OpenAI, MCP, and A2A version pins still match the intended freeze choice
4. preserve unresolved proof gaps as explicit spec constraints or non-goals instead of guessing
5. keep the packet inside the dependency story in this README; do not silently absorb a neighbor
   packet's scope

When a later Codex session starts cold:

1. read `docs/direction.md`
2. read `docs/current-state.md`
3. read the relevant packet dossier here plus the packet entry in the map above
4. read any prerequisite packet dossiers named there before drafting a spec
5. treat that dossier's "Fixed Constraints Before Spec Writing" section as already decided unless
   repo truth has changed
6. follow that dossier's "Recommended Inputs For Future SpecKit Packet" section in order
7. only then freeze packet scope or draft a SpecKit packet

## Freeze-Time Checks For Any Future Packet Authoring

- reread `docs/direction.md` and `docs/current-state.md` first so the packet does not freeze stale
  truth
- keep `live-default`, `landed-not-default`, `deterministic-only`, and `planned-only` separate in
  the packet spec
- re-check protocol versions before freezing any packet that touches MCP or A2A
- do not turn comparator docs or research notes into runtime-truth claims unless the repo now
  proves them
