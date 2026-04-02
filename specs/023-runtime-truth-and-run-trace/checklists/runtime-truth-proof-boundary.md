# Runtime Truth Proof Boundary Checklist: Runtime Truth And Run Trace

**Purpose**: Validate that packet `023` requirements clearly define truthful runtime-truth scope,
proof boundaries, placeholder-step limits, and bounded run-trace taxonomy
**Created**: 2026-04-01
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [x] CHK001 All proof-bearing surfaces are named explicitly: task, session, autonomy, and operator
  run detail
- [x] CHK002 The packet defines the full run-trace taxonomy scope for graph, branch, node, tool,
  handoff, repair, retry, fan-out, join, and supervision relationships
- [x] CHK003 The spec clearly defines what packet `023` owns versus what packet `022` still owns
- [x] CHK004 Dependencies and proof-status split are documented for packet `019`, `020`, `021`,
  `022`, and `023`

## Requirement Clarity

- [x] CHK005 The difference between substrate completion and grounded task proof is defined in
  direct, testable language
- [x] CHK006 The placeholder-step limit is stated plainly enough that a future implementer cannot
  mistake `workflow.execute_step` completion for grounded task proof
- [x] CHK007 The conservative phrases are frozen exactly and presented as packet-owned wording
- [x] CHK008 The OpenTelemetry and W3C role is stated as taxonomy guidance only, without blurred
  claims about existing emitted spans

## Requirement Consistency

- [x] CHK009 User stories, functional requirements, and success criteria preserve the same packet
  `019` and `020` live-proof versus packet `021` and `022` deterministic-only split
- [x] CHK010 Scope and non-goal statements consistently exclude UI polish, generic
  observability-platform work, interoperability, and coordinator-runtime scope
- [x] CHK011 Key entities use one stable naming scheme for runtime truth, proof boundaries,
  run-trace summaries, and grounded evidence references

## Scenario Coverage

- [x] CHK012 The spec defines what should be said when a graph completes but no grounded work
  occurred below the placeholder step boundary
- [x] CHK013 The spec defines how fan-out, join, repair, retry, and supervision relationships
  should be represented without claiming a full emitted span hierarchy
- [x] CHK014 The spec addresses cross-surface wording drift as a concrete scenario instead of
  assuming all projections already agree

## Edge Case Coverage

- [x] CHK015 Mismatch cases between proof-boundary wording and actual runtime evidence are called
  out explicitly
- [x] CHK016 The spec addresses the case where packet `021` or packet `022` wording could drift
  from packet `023` surface truth

## Packet Readiness

- [x] CHK017 The packet is implementation-ready on current `main`
- [x] CHK018 The task pack is executable without a scaffold-only revalidation stop
- [x] CHK019 Packet `021` supervision evidence stays separate from packet `023` runtime truth

## Notes

- This checklist validates requirement quality and implementation-readiness posture.
- Deterministic projection proof stays separate from fresh live runtime proof unless a real rerun
  is executed.
