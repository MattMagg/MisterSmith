# Runtime Truth Proof Boundary Checklist: Runtime Truth And Run Trace

**Purpose**: Validate that packet `023` requirements clearly define truthful run-trace scope,
proof boundaries, placeholder-step limits, and revision gates before later implementation work
**Created**: 2026-04-01
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 Are all proof-bearing surfaces named explicitly for packet `023` coverage: task,
  session, autonomy, and operator run detail? [Completeness, Spec §Functional Requirements]
- [ ] CHK002 Does the spec define the full trace-taxonomy scope for graph, branch, node, tool,
  handoff, repair, retry, fan-out, join, and supervision relationships? [Completeness, Spec
  §FR-006]
- [ ] CHK003 Does the spec clearly define what packet `023` owns versus what packet `022` still
  owns? [Completeness, Spec §Current Truth & Scope; Spec §FR-009]
- [ ] CHK004 Are deferred revision points documented for upstream packet completion and repo-truth
  drift? [Completeness, Spec §Deferred Revision Points]

## Requirement Clarity

- [ ] CHK005 Is the difference between substrate completion and grounded task proof defined in
  direct, testable language rather than implied wording? [Clarity, Spec §FR-002]
- [ ] CHK006 Is the placeholder-step limit stated plainly enough that a future implementer cannot
  mistake `workflow.execute_step` completion for grounded task proof? [Clarity, Spec §FR-004]
- [ ] CHK007 Are the conservative phrases frozen exactly and presented as current truth wording
  rather than optional examples? [Clarity, Spec §FR-005]
- [ ] CHK008 Is the OpenTelemetry and W3C role stated as taxonomy guidance only, without blurred
  claims about existing emitted spans? [Clarity, Spec §FR-008]

## Requirement Consistency

- [ ] CHK009 Do the user stories, functional requirements, and success criteria all preserve the
  same packet `019` and `020` live-proof versus packet `021` deterministic-only split?
  [Consistency, Spec §User Story 1; Spec §FR-003; Spec §SC-003]
- [ ] CHK010 Do the scope and non-goal statements consistently exclude UI polish, generic
  observability-platform work, and coordinator-runtime scope? [Consistency, Spec §Current Truth &
  Scope; Spec §FR-011]
- [ ] CHK011 Do the key entities use one stable naming scheme for run trace, trace events, trace
  links, and proof-boundary views without synonym drift? [Consistency, Spec §Key Entities]

## Scenario Coverage

- [ ] CHK012 Does the spec define what should be said when a graph completes but no grounded work
  occurred below the placeholder step boundary? [Coverage, Spec §User Story 1]
- [ ] CHK013 Does the spec define how fan-out, join, repair, and retry relationships should be
  represented without claiming a full emitted span hierarchy? [Coverage, Spec §User Story 2]
- [ ] CHK014 Does the spec address cross-surface wording drift as a scenario instead of assuming
  all projections already agree? [Coverage, Spec §User Story 3; Spec §Edge Cases]

## Edge Case Coverage

- [ ] CHK015 Are mismatch cases between proof-boundary wording and actual runtime evidence
  explicitly called out? [Edge Case, Spec §Edge Cases]
- [ ] CHK016 Does the spec address the case where upstream packet work changes the preferred
  ownership or wording before implementation starts? [Edge Case, Spec §Deferred Revision Points]

## Revalidation Gate Quality

- [ ] CHK017 Is the before-implementation revalidation gate explicit, blocking, and tied to named
  source documents? [Acceptance Criteria, Spec §Before Implementation Revalidation Gate]
- [ ] CHK018 Does the revalidation gate require rerunning downstream SpecKit steps if repo truth
  moved? [Acceptance Criteria, Spec §Before Implementation Revalidation Gate]
- [ ] CHK019 Is the scaffold status clearly separated from an implementation-ready freeze?
  [Acceptance Criteria, Spec §Scaffold Status]

## Notes

- Check items off as later revision work confirms the scaffold is still accurate.
- This checklist validates requirement quality and readiness posture, not implementation behavior.
