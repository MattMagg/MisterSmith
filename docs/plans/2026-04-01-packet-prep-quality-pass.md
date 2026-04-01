# Packet-Prep Quality Pass

Date: April 1, 2026
Status: Completed

## Objective

Make `docs/packet-prep/README.md` and packet dossiers `022` through `028` harder to misread,
more exact about repo truth, and easier for a future cold-start SpecKit authoring session to use
without guessing.

## Scope

- `docs/packet-prep/README.md`
- `docs/packet-prep/022-durable-workflow-core.md`
- `docs/packet-prep/023-runtime-truth-and-run-trace.md`
- `docs/packet-prep/024-agent-boundary-security-hardening.md`
- `docs/packet-prep/025-step-level-intelligence-v2.md`
- `docs/packet-prep/026-first-real-coordinator-subagent-runtime.md`
- `docs/packet-prep/027-capability-discovery-and-interoperability.md`
- `docs/packet-prep/028-selective-strong-coordination.md`
- one small supporting doc under `docs/packet-prep/` only if it materially improves cold-start
  usability

## Authority And Constraints

- Authority order:
  1. `docs/direction.md`
  2. `docs/current-state.md`
  3. `docs/research-output/consolidated/`
  4. `docs/research-output/analysis/`
  5. official docs / primary sources
- `docs/current-state.md` wins for what is live or landed now.
- `docs/direction.md` wins for sequencing and strategic priority.
- This is a documentation pass only. Do not create packet specs, change `specs/`, or widen packet
  scope.

## Non-Goals

- writing or freezing new packet specs
- changing runtime or crate behavior
- broad rewriting of `docs/direction.md` or `docs/current-state.md` unless a real contradiction is
  discovered
- padding dossiers into long essays

## Milestones

### Milestone 1: Audit The Dossier Set

Validation:

- authority docs and supporting research reread
- packet dossiers reread
- packet-relevant repo seams inspected
- parallel audit notes gathered for truth, boundary, source-quality, consistency, and cold-start
  usability

### Milestone 2: Tighten The Dossiers

Validation:

- truth labels mean the same thing across all packets
- packet boundaries are sharper, not broader
- repo grounding points to concrete files, functions, tests, and proof notes
- source lists prefer the best official inputs and avoid mixed unstable baselines
- recommended inputs reduce future guesswork

### Milestone 3: Close Cleanly

Validation:

- `git diff --check`
- `npx markdownlint-cli2 "docs/packet-prep/**/*.md" "docs/current-state.md" --config .markdownlint.json`

## Stop Conditions

- packet dossiers are materially stronger for future pre-spec work
- live vs landed-not-default vs deterministic-only vs planned-only language is tighter
- README clearly frames these files as pre-spec dossiers and explains dependency order
- remaining unresolved points are explicit instead of blurred

## Completion Note

Completed work:

- tightened cold-start guidance in `docs/packet-prep/README.md`
- sharpened proof-boundary and exact-symbol grounding in packets `023`, `024`, `025`, and `026`
- tightened protocol/source posture in packet `025` and packet `027`
- fixed the truth-label edge case in packet `028`
- updated `docs/packet-prep/packet-authoring-checklist.md` so later packet authors preserve the
  pinned MCP and A2A baselines

Validation evidence:

- `git diff --check`
- `npx markdownlint-cli2 "docs/packet-prep/**/*.md" "docs/current-state.md" --config .markdownlint.json`
