---
description: Perform a read-only Mister Smith packet analysis across spec.md, plan.md, and tasks.md after task generation.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Goal

Identify high-signal inconsistencies, gaps, and scope drift across `spec.md`, `plan.md`, and
`tasks.md` before implementation. This command MUST run only after `/speckit.tasks` has produced a
complete `tasks.md`.

## Operating Constraints

**STRICTLY READ-ONLY**: Do **not** modify files. Output a structured analysis report and, if
helpful, offer a remediation plan that can be applied later.

**Constitution Authority**: `.specify/memory/constitution.md` is non-negotiable within this
analysis scope.

## Execution Steps

1. Run `.specify/scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks` and
   derive:
   - `SPEC = FEATURE_DIR/spec.md`
   - `PLAN = FEATURE_DIR/plan.md`
   - `TASKS = FEATURE_DIR/tasks.md`

2. Load only the minimal necessary context from:
   - `spec.md`: Current Truth & Scope, user stories, requirements, edge cases, success criteria
   - `plan.md`: summary, technical context, constitution check, design decisions, milestones,
     explicit deferrals
   - `tasks.md`: status reconciliation, blocking freeze, task IDs, `[P]` markers, choke points,
     final validation
   - `.specify/memory/constitution.md`

3. Build internal models for:
   - requirements and success criteria
   - user stories and independent tests
   - task coverage mapping
   - bounded scope and explicit non-goals
   - validation and proof boundaries

4. Detection passes:
   - **Bounded scope**: does the packet reopen landed work or leak into unrelated programs?
   - **Coverage**: do requirements and stories have corresponding tasks?
   - **Validation**: do tasks cover the promised deterministic checks and live-proof boundaries?
   - **Parallel safety**: are `[P]` tasks actually compatible with the plan's choke points?
   - **Constitution**: any MUST-level violation is CRITICAL
   - **Terminology and truth**: does the packet describe current repo truth consistently?

5. Severity assignment:
   - **CRITICAL**: constitution violation, reopened closed scope, missing core validation or zero
     coverage for a baseline requirement
   - **HIGH**: contradictory requirements, fake parallelism, missing explicit deferral or proof boundary
   - **MEDIUM**: terminology drift, underspecified edge case, incomplete evidence mapping
   - **LOW**: wording or structure improvements that do not affect execution

6. Produce a compact report:

## Packet Analysis Report

| ID | Category | Severity | Location(s) | Summary | Recommendation |
| -- | -------- | -------- | ----------- | ------- | -------------- |
| A1 | Coverage | HIGH | tasks.md | Requirement X has no task coverage | Add one bounded validation or implementation task |

Also include:

- **Coverage Summary**
- **Constitution Alignment Issues**
- **Unmapped Tasks**
- **Metrics**:
  - total requirements
  - total tasks
  - requirements with coverage
  - ambiguity count
  - critical issues count

1. End with next actions:
   - if CRITICAL issues exist, recommend fixing before `/speckit.implement`
   - otherwise note whether the packet is ready for implementation
   - ask whether the user wants concrete remediation edits for the top issues

## Analysis Guidelines

- Never hallucinate missing sections
- Prioritize reopened scope and fake-proof claims above style issues
- Treat missing closure tasks or missing proof-boundary tasks as real findings
- Report a clean pass explicitly if no issues are found

## Context

$ARGUMENTS
