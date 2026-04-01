# Step Policy Checklist: Step-Level Intelligence v2

**Purpose**: Validate the quality, clarity, and completeness of the packet requirements for
bounded step-level intelligence policy
**Created**: 2026-04-01
**Feature**: `/Users/macmain/.local/share/symphony-workspaces/025-step-level-intelligence-v2/specs/025-step-level-intelligence-v2/spec.md`

**Note**: This checklist tests the packet requirements, not the implementation.

## Requirement Completeness

- [ ] CHK001 Are deterministic step-scoring requirements defined using current step, routing,
      supervision, and budget context? [Completeness, Spec §FR-001]
- [ ] CHK002 Are all bounded action outcomes defined for `keep`, `retry`, `clarify`, `downgrade`,
      and `escalate`? [Completeness, Spec §FR-002]
- [ ] CHK003 Are budget-aware policy requirements defined without creating a new trace schema?
      [Completeness, Spec §FR-005]
- [ ] CHK004 Are operator summary requirements defined for existing task inspect, autonomy, and
      packet-owned operator-facing summaries? [Completeness, Spec §FR-006]

## Requirement Clarity

- [ ] CHK005 Is "deterministic step scoring" specific enough to prevent later judge-heavy or
      training-heavy interpretation? [Clarity, Spec §FR-001, Spec §FR-010]
- [ ] CHK006 Is "placeholder orchestration proof" clearly distinguished from grounded task proof?
      [Clarity, Spec §FR-007, Spec §FR-011]
- [ ] CHK007 Is the budget-aware policy wording specific enough to choose between local correction
      and broader escalation without inventing new packet scope? [Clarity, Spec §User Story 2]

## Requirement Consistency

- [ ] CHK008 Do packet `020` repair-lineage requirements and packet `025` step-policy
      requirements avoid conflicting source-of-truth claims? [Consistency, Spec §FR-003]
- [ ] CHK009 Do packet `023` ownership requirements stay consistent across the spec, planned
      contract, and proof wording? [Consistency, Spec §FR-004]
- [ ] CHK010 Is packet-021 supervision evidence described consistently as deterministic-only unless
      fresher live proof is found? [Consistency, Spec §Current Truth & Scope, Spec §Clarifications]

## Acceptance Criteria Quality

- [ ] CHK011 Can each success criterion be verified through deterministic validation without
      inventing new runtime claims? [Measurability, Spec §Success Criteria]
- [ ] CHK012 Do the user stories define independently testable slices for scoring, policy action,
      and operator-visible summaries? [Acceptance Criteria, Spec §User Scenarios & Testing]

## Scenario Coverage

- [ ] CHK013 Are requirements defined for the no-graph-context case where packet `025` still must
      stay deterministic and bounded? [Coverage, Spec §Edge Cases]
- [ ] CHK014 Are conflicting budget-pressure and difficulty-signal cases covered by explicit
      policy wording? [Coverage, Spec §Edge Cases]
- [ ] CHK015 Are simultaneous packet `020` repair and packet `021` supervision cases addressed
      without scope drift? [Coverage, Spec §Edge Cases]

## Non-Functional Requirements

- [ ] CHK016 Are proof-honesty requirements explicit enough to prevent overstating
      `workflow.execute_step` completion? [Coverage, Spec §FR-007, Spec §FR-011]
- [ ] CHK017 Are scope-boundary requirements explicit enough to exclude benchmarks, training,
      coordinator runtime, and interoperability work? [Coverage, Spec §FR-012]

## Dependencies And Assumptions

- [ ] CHK018 Does the spec clearly say packet `023` owns proof-boundary schema and packet `025`
      only consumes it? [Dependency, Spec §FR-004]
- [ ] CHK019 Does the spec clearly say this scaffold will be revised before implementation after
      earlier packet work settles? [Assumption, Spec §Current Truth & Scope, Spec §Assumptions]

## Notes

- Check items off as completed: `[x]`
- Record findings inline when a requirement is incomplete or ambiguous
- Use this checklist to harden the packet before any later implementation freeze
