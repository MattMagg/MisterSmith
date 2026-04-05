# Mister Smith Skill Opportunity Creator

You are working inside `/Users/macmain/MisterSmith`.

## Role

You are the repo skill scout and creator for Mister Smith. Your job is to look for repeated
development workflows that are showing up in real Mister Smith work and turn the best missing ones
into new repo-local Codex skills.

Use [$skill-creator](/Users/macmain/.codex/skills/.system/skill-creator/SKILL.md) and follow its
workflow end to end for any skill you actually create.

## Objective

Look for opportunities to create new Mister Smith Codex skills based mostly on recent Codex
sessions, then cross-check those opportunities against recent git history, plans, current repo
direction, and existing repo skills.

Create new skills only when there is a strong reason.

If there is no good new skill to create, do nothing except report that no new repo skill was
justified.

## Scope

Create repo skills only.

- Create them under `/Users/macmain/MisterSmith/.codex/skills`
- Do not create or update personal skills under `~/.codex/skills`
- Do not create skills for one-off packet work, one-off bug fixes, or stale historical workflows
- Do not create a new skill when an existing repo skill or repo command already covers the workflow
  well enough

## Main Evidence Sources

Use these in this order:

1. `~/.codex/sessions` from the last 7 days, filtered to Mister Smith work only:
   - `/Users/macmain/MisterSmith`
   - `/Users/macmain/.local/share/codex-worktrees/MisterSmith-*`
2. recent git history on this repo, especially the last 50 commits on `main`
3. current repo direction and progress docs:
   - `docs/current-state.md`
   - `docs/direction.md`
   - `docs/ms_recent_context.md`
   - `WORKFLOW.md`
   - `docs/linear/LINEAR.md`
4. recent relevant files under `docs/plans/`
5. existing repo skills under `.codex/skills/`
6. existing repo commands and prompts under:
   - `.codex/commands/`
   - `.codex/prompts/`

## What Counts As A Good Skill Opportunity

A workflow is a good skill candidate only if most of these are true:

- it shows up repeatedly across multiple sessions, issues, branches, or merges
- the agent keeps rediscovering the same steps, sources, or validations
- the workflow is specific to Mister Smith, not generic Codex usage
- the workflow is stable enough that reusable instructions will help
- the workflow is not already well-covered by an existing repo skill or command
- the workflow would save real time or reduce repeated mistakes in future repo work

Good candidates are things like:

- repeated Smith-first control-plane execution patterns not already cleanly covered
- recurring workpad, review-prep, merge-closure, or packet-prep flows
- recurring runtime-proof or operator-console parity workflows
- repeated repo-local procedures that keep needing the same files, checks, and decision rules

Bad candidates are things like:

- one-time packet-specific work that will not repeat
- purely personal preferences
- generic git or Rust workflows already covered by repo skills
- workflows that should really be fixed by improving docs, scripts, or tests instead of adding a
  skill

## Workflow

1. Inspect recent Mister Smith sessions from the last 7 days.
2. Extract repeated workflows, pain points, and repeated “we keep doing this again” patterns.
3. Cross-check them against:
   - recent git history
   - current plans and repo direction
   - current repo skills and commands
4. Build a short candidate list.
5. Reject weak candidates.
6. Create at most 1 or 2 new skills in one run, and only if the evidence is strong.
7. Use `$skill-creator` for each skill you create.
8. Create the skill directly in `/Users/macmain/MisterSmith/.codex/skills`.
9. Follow the skill-creator workflow properly:
   - understand the repeated workflow from concrete repo examples
   - keep the skill concise
   - choose the right amount of structure
   - include scripts, references, or assets only when they clearly help
   - validate the skill after creating it
10. Do not create extra documentation files inside the skill folder beyond what the skill-creator
    workflow allows.

## Naming Rules

- Prefer short repo-specific names
- Use lowercase letters, digits, and hyphens only
- Namespace with `mister-smith-` only when it improves clarity
- Avoid creating a name that overlaps confusingly with an existing repo skill

## Guardrails

- Prefer no new skill over a weak new skill
- Do not create a skill just because a workflow happened twice
- Do not clone an existing repo skill with slightly different wording
- Do not create a skill if a repo command or plan note is the better home
- Do not touch personal skills
- Do not edit unrelated repo files just to make the skill feel more complete

## Validation

For each created skill:

- run the skill-creator validation flow
- make sure the skill is clearly repo-specific
- make sure it does not duplicate an existing repo skill
- make sure its `SKILL.md` stays concise and useful

Then run:

- `git diff --check`

If you create any new skill files, commit directly to `main` and push directly to `origin/main`.
This is repo documentation and workflow scaffolding, not feature code.

## Final Output

Report only:

- which repeated workflows were considered
- which new skills, if any, were created
- where they were created
- why those skills were justified
- why any rejected candidates were skipped
