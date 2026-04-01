# Durability Checklist: Durable Workflow Core

**Purpose**: Validate the quality, clarity, and completeness of the packet requirements for
durable workflow semantics, lifecycle control, and effect boundaries
**Created**: 2026-04-01
**Feature**: `specs/022-durable-workflow-core/spec.md`

## Requirement Completeness

- [ ] CHK001 Are event-history requirements defined clearly enough to rebuild workflow state
  without inventing extra durable records during implementation? [Completeness, Spec §FR-001,
  Spec §FR-002]
- [ ] CHK002 Are effect-boundary requirements specific enough to separate state acceptance from
  external side effects? [Completeness, Spec §FR-004, Spec §FR-005]
- [ ] CHK003 Are lifecycle-verb requirements defined for pause, resume, cancel, terminate, and
  reset or rewind posture, including any explicit deferrals? [Completeness, Spec §FR-006]
- [ ] CHK004 Are compaction and replay-governance requirements present without widening into a
  full history platform redesign? [Completeness, Spec §FR-008, Spec §FR-009]

## Requirement Clarity

- [ ] CHK005 Is "canonical durable workflow history model" clear enough that implementers know
  which accepted changes belong in durable history and which do not? [Clarity, Spec §FR-001]
- [ ] CHK006 Is "completion unknown" defined clearly enough to avoid silent success assumptions at
  effect boundaries? [Clarity, Spec §FR-005]
- [ ] CHK007 Is the packet explicit about preserving current session continuity instead of
  redesigning session behavior? [Clarity, Spec §FR-003]
- [ ] CHK008 Are first-slice open questions separated clearly enough that open design points are
  not mistaken for frozen contracts? [Clarity]

## Requirement Consistency

- [ ] CHK009 Do the spec's lifecycle requirements align across user stories, functional
  requirements, and success criteria? [Consistency]
- [ ] CHK010 Do the packet's durable-workflow claims stay consistent with the repo's current
  truth split between live-default, landed-not-default, deterministic-only, and planned work?
  [Consistency, Spec §Current Truth & Scope, Spec §FR-014]
- [ ] CHK011 Do comparator references stay at the semantic level without drifting into product
  cloning language? [Consistency, Spec §Clarifications, Spec §FR-011]

## Acceptance Criteria Quality

- [ ] CHK012 Can replay-safety outcomes be objectively verified without inventing a new test
  target during implementation? [Measurability, Spec §SC-001]
- [ ] CHK013 Can effect-boundary correctness be verified without overstating exactly-once
  guarantees? [Measurability, Spec §SC-002, Spec §FR-010]
- [ ] CHK014 Can compaction success be measured in a bounded scenario without expanding the packet
  into storage-platform work? [Measurability, Spec §SC-004]

## Scenario Coverage

- [ ] CHK015 Are restart-resume continuity scenarios covered well enough to protect the existing
  proof surface? [Coverage, Spec §User Story 1, Spec §SC-005]
- [ ] CHK016 Are repeated lifecycle-command and repeated-effect scenarios covered explicitly?
  [Coverage, Spec §User Story 2, Spec §User Story 3, Spec §Edge Cases]
- [ ] CHK017 Are history-version and upstream-packet-drift scenarios called out honestly rather
  than hidden? [Coverage, Spec §Open Design Questions For The First Slice, Spec §Edge Cases]

## Dependencies And Assumptions

- [ ] CHK018 Is this packet clearly implementation-ready everywhere a later agent would look, with
  no false refresh gate left behind? [Dependency, Spec §Current Truth & Scope,
  Spec §Clarifications, Spec §Assumptions]
- [ ] CHK019 Are unfinished or still-open first-slice choices kept explicit and bounded instead of
  silently frozen? [Dependency, Spec §FR-013]

## Ambiguities And Conflicts

- [ ] CHK020 Is the first compaction mechanism intentionally left open but still bounded enough to
  keep the packet useful? [Ambiguity, Spec §Open Design Questions For The First Slice]
- [ ] CHK021 Is the initial placement of the intent and effect boundary explicit enough for plan
  work, or does it still need a pre-implementation narrowing note? [Ambiguity, Spec
  §Open Design Questions For The First Slice]

## Notes

- Check items off as completed: `[x]`
- If a current-main contradiction appears, fix the packet before implementation continues
- Use this checklist to keep packet `022` honest and implementation-ready during the first coding pass
