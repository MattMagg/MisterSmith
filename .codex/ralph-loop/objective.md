Audit Phase 1 and Phase 2 spec/planning artifacts for implementation readiness and task sequencing quality.

Scope:
- /Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/spec.md
- /Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/plan.md
- /Users/matthewmaggio/Mister-Smith/specs/001-phase1-foundation/tasks.md
- /Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/spec.md
- /Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/plan.md
- /Users/matthewmaggio/Mister-Smith/specs/002-phase2-runtime-async/tasks.md

Objective:
1) Determine whether task sequences are complete and actionable.
2) Identify obvious missing sequencing steps and dependencies.
3) Ensure each major implementation-step/task has references to the exact implementation-detail location where needed information lives.
4) Flag ambiguity, duplication, or weak traceability in a way that can be acted on immediately.

Deliverable:
- Write a markdown report to /Users/matthewmaggio/Mister-Smith/docs/phase1-phase2-spec-audit.md
- Include sections:
  - Executive Summary
  - Phase 1 Findings (gaps, strengths, sequencing issues)
  - Phase 2 Findings (gaps, strengths, sequencing issues)
  - Cross-Phase Sequencing Risks
  - Recommended Task Sequence Adjustments
  - Reference Map (task -> source doc + section)
  - Immediate Next Actions

Constraints:
- Do not run /speckit.implement.
- You may update spec/plan/tasks files only if needed to resolve critical audit findings discovered during this run.
- Keep recommendations concrete and file/section anchored.
