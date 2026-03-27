# Mister Smith Post-Packet-020 Next Phase Spec Handoff

You are working in the Mister Smith repository at `<repo_root>`.

Your mission in this session is to determine the next honest bounded development phase after
packet `020` and the March 27 follow-up notes, and then produce the correct planning artifact for
that phase.

This is a research-and-spec session, not an implementation session.

Treat current repo truth, current code, and current runtime proof evidence as the primary truth.

Do not assume a new packet already exists just because the previous packet is closed.

## Objective

By the end of this session, you must have:

1. verified the current forward-development authority and current repo truth
2. identified the strongest remaining bounded gap between landed substrate and proven default
   runtime behavior
3. decided whether the next honest deliverable is:
   - one new bounded SpecKit packet under `<next_specs_root>`
   - or one concise checkpoint note at `<checkpoint_note_path>` explaining why a new packet should
     not be frozen yet
4. stated clearly what is in scope now, what is deferred, and what remains dormant backlog only
5. stopped before implementation, queue staging, or unrelated cleanup

## Forward-Development Boundary

This session is only for deciding and writing the next planning artifact.

You must not:

- implement the next phase
- reopen packet-020 correctness work unless current repo truth proves a new defect
- treat `MS-110` as an active bug; it is currently a dormant planning lane unless new evidence says
  otherwise
- widen into generic cleanup, framework parity, or side programs
- assume that research and packet freezing are automatically the same step

## Core Constraints

- this is a planning-and-spec session, not an implementation session
- keep the scope bounded to one honest next-step artifact
- prefer current repo truth over older direction notes when they conflict
- do not create a new packet if the evidence only justifies a checkpoint note
- do not queue-stage work, reopen closed implementation slices, or mutate watched-queue state

## Start Sequence

Before choosing the next phase, read these files in order:

1. `AGENTS.md`
2. `CLAUDE.md`
3. `README.md`
4. `<current_state_note>`
5. `<latest_closure_note>`
6. `<march27_followup_note>`
7. `<evidence_freeze_note>`
8. `docs/plans/2026-03-26-packet-019-budget-aware-runtime-proof.md`
9. `docs/plans/2026-03-27-ms-110-adaptive-runtime-topology-planning.md`

Then read the most relevant current implementation packets:

1. `specs/019-budget-backed-runtime-routing-control-loop/spec.md`
2. `specs/019-budget-backed-runtime-routing-control-loop/plan.md`
3. `specs/020-verifier-gated-adaptive-orchestration/spec.md`
4. `specs/020-verifier-gated-adaptive-orchestration/plan.md`
5. `specs/020-verifier-gated-adaptive-orchestration/tasks.md`

Treat older forward-direction notes and older next-packet handoffs as historical context only.
They are not current authority unless they still match current repo truth.

## Grounding Pass In Code

Before deciding the next phase, inspect the code surfaces that define the remaining gap between
landed behavior and default-runtime proof:

- `crates/mister-smith-app/src/execution.rs`
- `crates/mister-smith-app/src/autonomy.rs`
- `crates/mister-smith-app/src/conversation.rs`
- `crates/mister-smith-agents/src/roles/planner.rs`
- `crates/mister-smith-agents/src/topology.rs`
- `crates/mister-smith-runtime/`
- `crates/mister-smith-http/`
- `apps/operator-console/` if operator-surface parity or visibility becomes a candidate gap

Your goal in this code pass is to verify:

- what is already live and proven on the default runtime path
- what remains config-gated, bounded, or not yet default
- what operator surfaces already expose versus what still requires explicit proof
- whether the strongest remaining gap is a real bounded product/runtime gap or only a dormant
  planning topic

## Decision Rule

Use this exact decision rule:

1. If current repo truth shows one clear bounded gap with honest validation and proof boundaries,
   freeze one new SpecKit packet.
2. If current repo truth only supports further research, checkpointing, or backlog clarification,
   do not force a packet. Write one checkpoint note instead.
3. If more than one candidate phase appears plausible, choose the single strongest one and defer
   the others explicitly.

Do not create multiple new packets.

## Candidate Gap Families

Evaluate these as candidate questions, not pre-selected answers:

1. Is the next honest phase about moving a currently config-gated runtime path closer to default
   truth?
2. Is it about widening or clarifying operator surfaces and proof parity across task, session, and
   autonomy paths?
3. Is it about a new bounded external-agent or interoperability surface that remains additive and
   not yet fully frozen?
4. Is current evidence still too weak for any new packet, making a checkpoint note the honest
   output?

You may select another bounded gap if current repo truth shows it is stronger than these families.

## Packet Output Requirement

If and only if a new bounded packet is justified, create one full SpecKit packet under
`<next_specs_root>`.

At minimum, create:

- `analyze.md`
- `data-model.md`
- `plan.md`
- `quickstart.md`
- `research.md`
- `spec.md`
- `tasks.md`

The packet must:

- define one bounded epic only
- state what is already baseline on `main` and must not be reopened
- define explicit non-goals
- define deterministic validation requirements
- define runtime or evaluation proof requirements if live behavior is affected
- state what adjacent work is deferred

## Checkpoint Output Requirement

If a new packet would still be premature, write one concise checkpoint note at
`<checkpoint_note_path>` that explains:

- what current repo truth already proves
- what candidate next-phase directions were considered
- why none of them is yet honest enough to freeze as a new bounded packet
- what exact evidence or research would be needed before freezing one

## Anti-Patterns

You must not:

- pre-decide the next packet before the grounding pass
- treat packet-020 follow-up notes as unresolved defects if the current repo truth says otherwise
- widen the session into implementation, queue staging, or review/merge work
- create a generic wishlist packet
- carry forward a stale next-phase label from an older note without proving it still matches the
  repo

## Stop Conditions

Stop and report clearly if:

- current repo truth shows no honest bounded next packet yet
- the strongest remaining gap cannot be scoped to one bounded epic
- the correct next move is a checkpoint note rather than a packet
- the repo authority docs are stale enough that they must be refreshed before any new packet can be
  honest

If you stop, leave one durable note instead of forcing a new packet.

## Final Response Requirements

At the end of the session:

- provide the path to the created packet or checkpoint note
- state the chosen next-phase deliverable in simple terms
- state what is explicitly in scope
- state what is explicitly deferred
- state what validation and runtime/evaluation proof would be required if a packet was created
- state whether any dormant backlog items remain dormant versus becoming active scope

Do not claim implementation progress.

This session is complete only when one honest next-phase planning artifact exists and its scope is
grounded in current repo truth.
