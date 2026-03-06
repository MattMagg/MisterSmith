# Ralph + Speckit

This repository uses Ralph as an orchestration layer over the existing Codex +
SpecKit workflow. Ralph does not replace the repo's Speckit commands here.

## Default Usage

Use plain Ralph loop mode for the full repo-native SpecKit workflow:

```bash
ralph run -c ralph.yml
```

The workflow source of truth remains the existing Codex command surface:

`/speckit.constitution -> /speckit.specify -> /speckit.clarify -> /speckit.plan -> /speckit.tasks -> /speckit.analyze -> /speckit.implement`

`PROMPT.md` should tell Ralph/Codex to use that exact sequence. No repo-local
Ralph hat collection is required for the primary workflow.

## Optional Ralph Presets

Ralph's built-in presets are separate workflows. They can still be useful, but
they are not the same thing as the repository's full SpecKit lifecycle.

### 1. Task implementation after SpecKit artifacts exist

Use Ralph's built-in implementation loop only when you intentionally want a
Ralph-native implementation workflow after specs/tasks already exist:

```bash
ralph run -c ralph.yml -H builtin:code-assist
```

### 2. Builder + reviewer loop

Use the lighter built-in review loop when you want a simpler build/review cycle:

```bash
ralph run -c ralph.yml -H builtin:feature
```

## Why Not `builtin:spec-driven`?

Ralph's built-in `spec-driven` preset is a generic contract-first loop, but this
repo already has a richer Speckit flow with explicit `clarify`, `plan`, `tasks`,
and `analyze` phases. Using plain `ralph run -c ralph.yml` keeps SpecKit itself
as the workflow authority instead of collapsing the repo into Ralph's generic
spec writer/critic flow.

## Validation Commands

```bash
ralph preflight -c ralph.yml --format json
ralph hats validate -c ralph.yml -H builtin:code-assist
```
