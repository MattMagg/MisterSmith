# Interop Requirements Checklist: Capability Discovery And Interoperability

**Purpose**: Validate that the packet requirements stay bounded, version-pinned, and clear before
planning or implementation work starts.  
**Created**: 2026-04-01  
**Feature**: [spec.md](../spec.md)

**Note**: This checklist is generated from the current packet `027` scaffold and is meant for
packet-author and reviewer use before implementation.

## Requirement Completeness

- [ ] CHK001 Are the provisional upstream dependencies on packets `022`, `023`, and `024`
  explicitly documented wherever packet `027` reuses their contract language? [Completeness,
  Spec §Current Truth & Scope]
- [ ] CHK002 Does the spec clearly state that packet `027` must be revised before any
  implementation work begins? [Completeness, Spec §Current Truth & Scope]
- [ ] CHK003 Are both required packet outputs defined: one normalized capability contract and one
  A2A lifecycle-mapping contract? [Completeness, Spec §FR-002, Spec §FR-009]

## Requirement Clarity

- [ ] CHK004 Is the phrase "first interoperability slice" defined concretely as A2A `v0.3.0`
  discovery plus lifecycle mapping instead of broad federation work? [Clarity, Spec §Current
  Truth & Scope]
- [ ] CHK005 Is the separation between discovery metadata and execution permission stated in a way
  that cannot be mistaken for implicit trust? [Clarity, Spec §FR-005]
- [ ] CHK006 Is MCP's role described clearly as a pinned baseline input and policy boundary rather
  than the first new interop slice? [Clarity, Spec §FR-007]

## Requirement Consistency

- [ ] CHK007 Are protocol version requirements consistent across all packet sections, with MCP
  always pinned to `2025-11-25` and A2A always pinned to `v0.3.0`? [Consistency, Spec §FR-006,
  Spec §FR-007, Spec §SC-002]
- [ ] CHK008 Do the non-goals, assumptions, and functional requirements all align on excluding
  generic federation, extra protocols, and live multi-remote proof? [Consistency, Spec §Current
  Truth & Scope, Spec §FR-015, Spec §Assumptions]
- [ ] CHK009 Does every reference to packet `016` stay limited to continuity and provenance
  instead of drifting into broad lifecycle-proof language? [Consistency, Spec §User Story 3, Spec
  §FR-013]

## Acceptance Criteria Quality

- [ ] CHK010 Can each success criterion be checked from packet artifacts without needing
  implementation-specific interpretation? [Measurability, Spec §SC-001 through Spec §SC-006]
- [ ] CHK011 Do the independent tests for all user stories describe how the packet can be
  validated as a document set rather than as running code? [Clarity, Spec §User Story 1 through
  Spec §User Story 3]

## Scenario Coverage

- [ ] CHK012 Are requirements defined for both discovery normalization and remote lifecycle
  mapping, rather than covering only one of the two? [Coverage, Spec §FR-003, Spec §FR-009]
- [ ] CHK013 Are unsupported or partial lifecycle mappings addressed so the packet does not imply
  that every A2A task state already fits Mister Smith directly? [Coverage, Spec §User Story 2,
  Spec §Edge Cases]

## Edge Case Coverage

- [ ] CHK014 Does the spec define what happens if upstream packet `022`, `023`, or `024`
  contracts change before implementation begins? [Edge Case, Spec §Edge Cases]
- [ ] CHK015 Does the spec address the case where discovery remains allowed but execution stays
  blocked by local policy? [Edge Case, Spec §Edge Cases]
- [ ] CHK016 Does the spec address version-drift risk by banning mixed pinned and unpinned
  protocol pages? [Edge Case, Spec §Current Truth & Scope, Spec §FR-008]

## Dependencies & Assumptions

- [ ] CHK017 Are the assumptions about provisional upstream contracts and later refresh clearly
  stated as assumptions, not as landed truth? [Assumption, Spec §Assumptions]
- [ ] CHK018 Is the refresh gate before implementation defined as a blocking dependency rather
  than a nice-to-have follow-up? [Dependency, Spec §FR-016]

## Ambiguities & Conflicts

- [ ] CHK019 Is any wording left that could imply packet `027` is ready for immediate
  implementation? [Ambiguity, Spec §Current Truth & Scope, Spec §FR-001]
- [ ] CHK020 Is any wording left that could imply a generic multi-protocol interoperability packet
  instead of one bounded A2A-first slice? [Conflict, Spec §Current Truth & Scope, Spec §FR-015]

## Notes

- This checklist is intentionally requirement-focused. It does not verify implementation behavior.
- Use it during packet review to catch scope drift before `/speckit.plan` or implementation work
  moves forward.
