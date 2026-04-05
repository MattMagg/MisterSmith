# Session Shell Checklist: Session-First User Shell

**Purpose**: Validate that the packet requirements for the session-first shell are complete, clear,
consistent, and ready for implementation review
**Created**: 2026-04-05
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 Are the exact startup-home requirements defined for recent sessions, start-new,
  resume-last, warnings, and config entry? [Completeness, Spec §FR-002, Spec §FR-003]
- [ ] CHK002 Are the required resume paths fully specified for both resume-last and
  resume-by-session behavior? [Completeness, Spec §FR-004, Spec §FR-005]
- [ ] CHK003 Are the shared-session requirements explicitly defined for identity, transcript
  continuity, and app-protocol reuse across CLI and GUI? [Completeness, Spec §FR-006, Spec
  §FR-007, Spec §FR-008]
- [ ] CHK004 Are the support-surface requirements documented for runtime, doctor, auth, proof,
  config, and MCP without leaving their placement ambiguous? [Completeness, Spec §FR-013]

## Requirement Clarity

- [ ] CHK005 Is "recent-first" defined clearly enough that reviewers can tell what must appear at
  startup and what is intentionally excluded? [Clarity, Spec §Assumptions & Defaults, Spec
  §FR-002, Spec §FR-003]
- [ ] CHK006 Is the difference between quick resume and broader session browsing stated without
  overlapping or contradictory behavior? [Clarity, Spec §FR-004, Spec §FR-005]
- [ ] CHK007 Is "shared session system" defined with enough specificity to avoid a second GUI-only
  session model? [Clarity, Spec §FR-006, Spec §FR-014]
- [ ] CHK008 Is "support surfaces stay secondary" defined with enough precision that implementers
  can tell what would count as product-drift? [Clarity, Spec §FR-013, Spec §FR-016]

## Requirement Consistency

- [ ] CHK009 Do the user stories, requirements, and contracts use `session` consistently as the
  primary user-facing noun without drifting back to `conversation` or `autonomy` as first-level
  navigation language? [Consistency, Spec §Assumptions & Defaults, Spec §FR-015]
- [ ] CHK010 Do the CLI and GUI parity requirements align across the spec, plan, and contracts for
  the same core control set? [Consistency, Spec §FR-009, Spec §FR-010]
- [ ] CHK011 Do the contracts and task list agree on the packet-bounded session-management surface
  without silently introducing extra command scope? [Consistency, Spec §FR-016, Plan §Design
  Decisions, Tasks §User Story 2]

## Scenario Coverage

- [ ] CHK012 Are primary startup, resume, browse, and cross-surface continuity journeys all
  represented with independent acceptance scenarios? [Coverage, Spec §User Story 1, Spec
  §User Story 2, Spec §User Story 3]
- [ ] CHK013 Are requirements defined for the no-prior-sessions scenario rather than assuming
  existing history? [Coverage, Spec §User Story 1, Spec §Edge Cases]
- [ ] CHK014 Are degraded runtime scenarios covered clearly enough to show what remains available
  and what becomes blocked? [Coverage, Spec §User Story 2, Spec §FR-011, Spec §FR-012]
- [ ] CHK015 Are simultaneous CLI-and-GUI session-open scenarios addressed so reviewers can judge
  whether one shared state model is preserved? [Coverage, Spec §User Story 3, Spec §Edge Cases]

## Acceptance Criteria Quality

- [ ] CHK016 Are the success criteria measurable enough to verify startup, resume, browse, and
  live-control behavior objectively? [Acceptance Criteria, Spec §SC-001, Spec §SC-006]
- [ ] CHK017 Do the success criteria avoid implementation-specific wording while still remaining
  testable? [Acceptance Criteria, Spec §SC-001, Spec §SC-007]

## Edge Case And Boundary Coverage

- [ ] CHK018 Are requirements explicit about warnings staying visible without burying the main
  product path? [Edge Case, Spec §Edge Cases, Spec §FR-011]
- [ ] CHK019 Is the packet boundary explicit enough to exclude repo-workflow tooling, generic
  admin-console expansion, and broad runtime redesign? [Boundary, Spec §Current Truth & Scope,
  Spec §FR-016]
- [ ] CHK020 Does the packet define whether support-surface commands remain reachable during shell
  use without turning into the default entry? [Boundary, Spec §FR-013, Contract
  `session-shell-contract.md`]

## Notes

- This checklist is for requirements quality review, not implementation testing.
- Focus areas for this run: session-shell UX requirements and shared-session consistency.
- Audience and timing defaults for this run: reviewer-facing, standard-depth packet review.
