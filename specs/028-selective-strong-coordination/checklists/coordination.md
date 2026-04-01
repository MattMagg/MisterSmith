# Coordination Checklist: Selective Strong Coordination

**Purpose**: Validate the quality, clarity, and completeness of the packet requirements for
selective strong coordination
**Created**: 2026-04-01
**Feature**: `/Users/macmain/MisterSmith/specs/028-selective-strong-coordination/spec.md`

**Note**: This checklist tests the packet requirements, not the implementation.

## Truth-Status Honesty

- [ ] CHK001 Are landed substrate, deterministic-only justification, live-default truth, and
      planned packet work kept distinct throughout the packet? [Consistency, Spec §Current Truth &
      Scaffolding Posture]
- [ ] CHK002 Does the packet avoid implying that strong coordination is already part of the
      default live runtime? [Clarity, Spec §FR-009, Spec §SC-005]
- [ ] CHK003 Are non-live claims tied to concrete existing seams rather than narrative hype?
      [Completeness, Spec §Current Truth & Scaffolding Posture]

## Dependency-Gate Clarity

- [ ] CHK004 Are upstream dependency gates on packets `022`, `023`, `024`, and `027` explicit and
      non-optional? [Completeness, Spec §FR-014]
- [ ] CHK005 Is the distinction between authoring gate and implementation gate stated clearly?
      [Clarity, Spec §FR-011]
- [ ] CHK006 Does the packet define what must be revalidated before implementation starts? [Gap,
      Spec §Before Implementation Revalidation Gate]

## Taxonomy Clarity

- [ ] CHK007 Are the three state classes defined in terms of correctness needs rather than named as
      vague categories? [Clarity, Spec §FR-001, Spec §FR-002]
- [ ] CHK008 Can representative examples be classified into exactly one state class without
      overlap? [Measurability, Spec §SC-001]
- [ ] CHK009 Are effectful state rules clearly separated from convergent state rules? [Consistency,
      Spec §FR-006]

## Invariant-Rule Clarity

- [ ] CHK010 Is the coordination choice rule explicit about when invariants force stronger
      coordination? [Clarity, Spec §FR-003, Spec §FR-005]
- [ ] CHK011 Are the decision rule and the taxonomy aligned without contradiction? [Consistency,
      Spec §User Story 2]
- [ ] CHK012 Are the packet examples sufficient to explain when strong coordination is *not*
      needed? [Coverage, Spec §User Story 2]

## Primitive Boundaries

- [ ] CHK013 Is `InvariantCell` the only frozen reusable primitive in the first slice? [Clarity,
      Spec §FR-007, Spec §SC-003]
- [ ] CHK014 Is `InvariantCell` grounded in existing CAS and reject-on-conflict behavior rather
      than in a new coordination subsystem? [Completeness, Spec §FR-008]
- [ ] CHK015 Is the protocol seam gate defined clearly enough to keep MPST and protocol safety
      deferred unless packet `027` later freezes that seam? [Clarity, Spec §FR-012]

## Non-Goal Discipline

- [ ] CHK016 Are CRDT rollout, generic distributed-systems experimentation, and additional
      primitives explicitly excluded from the first slice? [Completeness, Spec §FR-015]
- [ ] CHK017 Does the packet avoid turning protocol safety into the hidden real scope? [Conflict,
      Spec §Current Truth & Scaffolding Posture]

## Revision-Before-Implementation Note

- [ ] CHK018 Does every major packet artifact repeat that this scaffold must be revised before
      implementation starts? [Coverage, Spec §FR-010]
- [ ] CHK019 Is the pre-implementation revalidation phase required before any future code task?
      [Coverage, Spec §FR-016]

## Notes

- Check items off as completed: `[x]`
- Add findings inline if wording, scope, or dependency gates drift in later revisions.
