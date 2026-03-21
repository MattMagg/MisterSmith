# Smith-Managed Ralph Prompt

This file is intentionally ephemeral.

Before every `./scripts/ralph run`, generate the current issue, workpad, or plan packet and run
`./scripts/ralph prompt --packet <packet.json>` to regenerate `PROMPT.md`.
Each successful `./scripts/ralph run` consumes that prep marker, so rerun the prompt step before
every subsequent `run`.
Do not treat the checked-in contents of this file as the source of truth for any active workflow.

## Required Inputs

- current issue or task objective
- current repo-grounded plan or workpad
- relevant source-of-record docs and specs
- stop conditions
- validation expectations
- expected durable outputs to persist back into repo notes or Linear

## Required Structure

Use this structure when generating the active prompt:

1. Mode
2. Goal
3. Current context and source docs
4. Workflow requirements and sequencing
5. In-scope and out-of-scope work
6. Validation requirements
7. Stop conditions
8. Definition of done

## Repo Boundary Rules

- Ralph is only the loop runner.
- Smith owns task-type routing and decides when Ralph is used.
- SpecKit remains the upstream spec and task-pack scaffold.
- The active issue, workpad, or plan packet owns the live task context.
- If `PROMPT.md` has not been freshly regenerated with `./scripts/ralph prompt --packet <packet.json>`
  from the current task context, do not run Ralph.
