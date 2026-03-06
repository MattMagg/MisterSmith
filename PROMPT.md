# Ralph Task Prompt

Mode: prep

Goal:
[Describe the task or feature request here.]

Context:
- Use the repo-native Speckit flow.
- Use `./spec/` as canonical architecture guidance.
- Use `./specs/` as the active SpecKit artifact directory.
- Ralph is only the loop runner here; do not substitute a Ralph-defined workflow
  for the existing SpecKit command chain.
- Use `README.md`, `ROADMAP.md`, `VALIDATION_REPORT.md`, `AGENTS.md`, and `CLAUDE.md`
  as supporting repo context.

Workflow Requirements:
- Required Speckit order:
  `/speckit.constitution -> /speckit.specify -> /speckit.clarify -> /speckit.plan -> /speckit.tasks -> /speckit.analyze -> /speckit.implement`
- `/speckit.checklist` is optional support for requirements quality; it does not
  replace `/speckit.analyze`.
- Never skip `/speckit.analyze` before `/speckit.implement`.

Mode Semantics:
- `prep`: stop after `/speckit.analyze`
- `full`: continue through `/speckit.implement`
- `implement`: use existing SpecKit artifacts; validate/analyze first if needed

Definition of Done:
- `prep`: the active feature has current spec/plan/tasks artifacts and analyze has
  been run with blockers surfaced or cleared.
- `full`: `prep` is satisfied and implementation is completed with verification.
- `implement`: the target implementation task is completed without violating the
  current spec/plan/tasks/analyze state.
