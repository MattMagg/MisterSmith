# Step Policy Checklist: Step-Level Intelligence v2

**Purpose**: Validate the quality, clarity, and completeness of the packet requirements for
bounded step-level intelligence policy
**Created**: 2026-04-01
**Feature**: `/Users/macmain/MisterSmith/specs/025-step-level-intelligence-v2/spec.md`

**Note**: This checklist tests packet requirements, not the implementation.

## Requirement Completeness

- [X] CHK001 Are deterministic step-scoring requirements defined using current step, routing,
      supervision, runtime-truth, and budget context? [Completeness, Spec §FR-001 through §FR-004]
- [X] CHK002 Are all bounded action outcomes defined for `keep`, `retry`, `clarify`,
      `downgrade`, and `escalate`? [Completeness, Spec §FR-003]
- [X] CHK003 Are budget-aware policy requirements defined without creating a new trace or proof
      schema? [Completeness, Spec §FR-002, Spec §FR-006]
- [X] CHK004 Are operator summary requirements defined for existing task inspect, autonomy status,
      and selected-run operator detail surfaces? [Completeness, Spec §FR-008, Spec §FR-009]

## Requirement Clarity

- [X] CHK005 Is "deterministic step policy" specific enough to prevent later judge-heavy or
      training-heavy interpretation? [Clarity, Spec §FR-012]
- [X] CHK006 Is "placeholder orchestration proof" clearly distinguished from grounded task proof?
      [Clarity, Spec §FR-010]
- [X] CHK007 Is the budget-aware policy wording specific enough to choose between local correction
      and broader `downgrade` or `escalate` action without inventing new packet scope? [Clarity,
      Spec §User Story 2]

## Requirement Consistency

- [X] CHK008 Do packet `020` repair-lineage requirements and packet `025` step-policy
      requirements avoid conflicting source-of-truth claims? [Consistency, Spec §FR-005]
- [X] CHK009 Do packet `023` ownership requirements stay consistent across the spec, plan,
      contract, and proof wording? [Consistency, Spec §FR-006]
- [X] CHK010 Is packet-021 supervision evidence described consistently as deterministic-only
      unless fresher live proof is found? [Consistency, Spec §Current Truth And Scope]

## Acceptance Criteria Quality

- [X] CHK011 Can each success criterion be verified through deterministic validation without
      inventing new runtime claims? [Measurability, Spec §Success Criteria]
- [X] CHK012 Do the user stories define independently testable slices for scoring, policy action,
      and operator-visible summaries? [Acceptance Criteria, Spec §User Scenarios And Testing]

## Scenario Coverage

- [X] CHK013 Are requirements defined for the no-graph-context case where packet `025` still must
      stay deterministic and bounded? [Coverage, Spec §Edge Cases]
- [X] CHK014 Are conflicting budget-pressure and difficulty-signal cases covered by explicit
      policy wording? [Coverage, Spec §Edge Cases]
- [X] CHK015 Are simultaneous packet `020` repair and packet `021` supervision cases addressed
      without scope drift? [Coverage, Spec §Edge Cases]

## Non-Functional Requirements

- [X] CHK016 Are proof-honesty requirements explicit enough to prevent overstating
      `workflow.execute_step` completion? [Coverage, Spec §FR-010]
- [X] CHK017 Are scope-boundary requirements explicit enough to exclude benchmarks, training,
      coordinator runtime, subagent runtime, and interoperability work? [Coverage, Spec §FR-012,
      Spec §FR-013]

## Dependencies And Assumptions

- [X] CHK018 Does the spec clearly say packet `023` owns proof-boundary schema and packet `025`
      only consumes it? [Dependency, Spec §FR-006]
- [X] CHK019 Does the spec clearly say packet `025` is now the active implementation packet on top
      of landed packet `019` through `024` seams? [Dependency, Spec §Current Truth And Scope]

## Notes

- Checklist status: `19/19` complete.
- Use this checklist to keep packet `025` implementation-ready before `/speckit.implement`.
