# Scaffold Checklist: First Real Coordinator-Subagent Runtime

**Purpose**: Validate the quality, clarity, and completeness of the packet `026` scaffold before
later implementation work
**Created**: 2026-04-01
**Feature**: `/Users/macmain/.local/share/symphony-workspaces/026-first-real-coordinator-subagent-runtime/specs/026-first-real-coordinator-subagent-runtime/spec.md`

**Note**: This checklist tests the packet requirements and scaffold boundaries, not the
implementation.

## Truth Separation

- [ ] CHK001 Does the scaffold clearly separate current live graph/runtime truth from the still
      missing grounded coordinator-subagent runtime? [Clarity, Spec §Current Truth & Scope]
- [ ] CHK002 Does the scaffold explicitly state that packets `022` through `025` are still in
      progress and must be reconciled later? [Completeness, Spec §Current Truth & Scope]

## Proof Standard

- [ ] CHK003 Does the scaffold define packet `026` success using delegation, state, grounded
      evidence, and coordinator decisions rather than graph completion alone? [Completeness,
      Spec §FR-003, Spec §FR-005, Spec §FR-008]
- [ ] CHK004 Is placeholder-only delegated completion clearly marked as insufficient for packet
      success? [Clarity, Spec §FR-006]

## Scope Boundaries

- [ ] CHK005 Are federation, capability discovery, and generic interoperability explicitly kept
      out of scope across `spec.md`, `plan.md`, and `tasks.md`? [Consistency]
- [ ] CHK006 Does the scaffold preserve the smallest-workflow rule and honest sequential collapse
      instead of forcing fan-out? [Coverage, Spec §FR-007]

## Upstream Ownership

- [ ] CHK007 Does packet `026` avoid redefining packet `022` through `025` ownership? [Consistency,
      Spec §FR-010]
- [ ] CHK008 Are the upstream dependency assumptions repeated consistently in `spec.md`,
      `plan.md`, `contracts/`, and `analyze.md`? [Consistency]

## Operator Surfaces

- [ ] CHK009 Are task result, autonomy status, and operator-console run detail the only required
      operator surfaces named for this packet? [Coverage, Spec §FR-009]
- [ ] CHK010 Does the scaffold avoid implying a broader UI or observability redesign?
      [Clarity, Plan §D5]

## Revision Gate

- [ ] CHK011 Is the pre-implementation revision gate present and visible in `spec.md`, `plan.md`,
      `quickstart.md`, and `tasks.md`? [Completeness]
- [ ] CHK012 Does the scaffold make it clear that no implementation or live-proof claim is valid
      before that gate is complete? [Coverage, Spec §FR-015]

## Session Follow-Up

- [ ] CHK013 Are session carry-forward limits defined in terms of identifiers and evidence
      references rather than transcript reuse? [Clarity, Spec §FR-013]

## Notes

- Keep this checklist open until the pre-implementation refresh pass is complete.
- If upstream packet wording changes materially, update this checklist before coding starts.
