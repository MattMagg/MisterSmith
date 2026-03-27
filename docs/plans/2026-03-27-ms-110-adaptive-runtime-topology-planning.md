# 2026-03-27 MS-110 Adaptive Runtime Topology Planning

## Summary

`MS-110` is now a planning-only follow-up. The March 27 correctness work already landed:

- fixed two-worker shaping is removed from the live path
- merge handling is legal and structural
- simple prompts stay sequential by default
- explicit parallel prompts still complete with a legal coordinator merge
- packet-020-style repair telemetry is now projected from an explicit runtime-owned record

The remaining question is narrower: how much further adaptive topology behavior should be added
beyond the current smallest-workflow baseline, and in which layer should that policy live.

## Current Repo Truth

The relevant surfaces are:

- planner contract: `crates/mister-smith-agents/src/roles/planner.rs`
- live planning context and runtime normalization:
  `crates/mister-smith-app/src/execution.rs`
- graph-shape classification and topology selection:
  `crates/mister-smith-agents/src/topology.rs`
- proof baseline and live matrix:
  `docs/plans/2026-03-27-runtime-planning-simplification.md`

What is already true on `main`:

- the planner is instructed to prefer the smallest workflow, stay sequential by default, only
  branch for clearly independent work, and only merge when branch outputs need consolidation
- runtime normalization coerces malformed merge steps into legal coordinator semantics
- once a planner emits a real explicit graph, runtime normalization preserves that graph shape
  rather than second-guessing it
- topology selection is primarily dependency-shape driven, with operational signals layered on top

## Remaining Problem

The remaining open problem is not a correctness failure. It is policy placement and over-shaping
control:

1. **Planner-source shaping**
   - the planner prompt is now softer, but it is still a global textual contract
   - if the planner emits unnecessary branches, the runtime currently preserves them
2. **Normalization boundaries**
   - `normalize_runtime_plan` repairs malformed graph semantics, but it does not yet collapse an
     explicit graph back to a smaller workflow when the graph is syntactically valid but
     unnecessary
3. **Topology-selection boundaries**
   - `TopologyCompiler` selects topology from graph shape and runtime signals
   - it does not decide whether the graph itself should have been simpler in the first place

## Objective

Define a bounded next slice that improves adaptive topology selection without undoing the March 27
stability wins or reintroducing hidden hard-coded workflow shapes.

## Scope

- decide what evidence is still missing before additional runtime changes
- decide whether the next slice belongs in:
  - planner prompt/policy
  - runtime normalization
  - topology selection
- define one bounded follow-up slice with explicit validation criteria if implementation is
  justified

## Non-Goals

- do not reopen the March 27 correctness fixes
- do not widen the HTTP task ingress
- do not introduce a new frozen packet until the placement decision is evidence-backed
- do not treat `MS-110` as blocking the next development stage unless a new live regression appears

## Key Design Decision

Use this decision rule before writing more code:

- if the problem is that the planner still emits too many branches for ambiguous prompts, fix the
  planner contract first
- if the problem is that syntactically valid but obviously unnecessary explicit graphs survive into
  execution, add a narrow normalization-collapse rule
- if the problem is only how an already-valid graph should execute under runtime conditions, refine
  topology selection

Do not mix all three in one slice.

## Proposed Milestones

### Milestone 1: Evidence Freeze

Goal:

- extend the March 27 live matrix with an explicit set of ambiguous prompts that could drift
  between sequential and parallel forms

Deliverable:

- one dated note or appendix that records prompt, generated plan shape, normalized graph shape,
  selected topology, and whether the result feels over-shaped

Validation:

- run the existing harness on the supported `openai_chatgpt` / `gpt-5.4` path
- preserve artifacts under a dated `docs/plans/artifacts/` lane

Stop condition:

- if ambiguous prompts already stay sequential consistently, stop and keep `MS-110` as a dormant
  planning issue

### Milestone 2: Placement Decision

Goal:

- choose exactly one layer for the next implementation slice

Decision outputs:

- `planner-policy` if the planner remains the main source of unnecessary graph structure
- `normalization-collapse` if explicit but semantically unnecessary graphs survive into runtime
- `topology-selection` if the graph is appropriate but execution mode still needs refinement

Validation:

- the decision must cite concrete prompt and artifact examples from Milestone 1

### Milestone 3: Bounded Follow-Up Slice

Only execute this milestone if Milestone 2 shows a real remaining problem.

Preferred order:

1. planner-policy refinement
2. narrow normalization-collapse heuristic
3. topology-selection refinement

Acceptance criteria for any implementation slice:

- baseline prompt still stays sequential
- explicit parallel prompt still produces a legal coordinator merge
- non-memo prompt still preserves requested output shape
- packet-020-style repair probe still exposes runtime-owned repair telemetry
- no reintroduction of unsupported-role or forced-shape behavior

## Recommended Next Action

Start with Milestone 1 only. There is not enough current evidence to justify more code right now,
because the March 27 live matrix proved the current baseline is stable and the remaining issue is
about policy quality, not an active correctness defect.

## Validation Checklist

For the planning-only pass:

- update `MS-110` with this note
- keep repo state clean
- do not claim a new phase freeze

For any later implementation slice:

- `cargo test -p mister-smith-app`
- `cargo test -p mister-smith-agents`
- `cargo clippy -p mister-smith-app -- -D warnings`
- `cargo clippy -p mister-smith-agents -- -D warnings`
- repeat the bounded live matrix on the supported runtime path

## Stop Conditions

- stop if no live evidence shows remaining over-shaping after Milestone 1
- stop if the only proposed improvement requires mixing planner, normalization, and topology
  selection changes in one pass
- stop if a new request supersedes this planning lane with a different bounded packet decision
