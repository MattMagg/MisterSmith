# Smith-Managed Ralph Prompt

This file is intentionally ephemeral.

Before every `./scripts/ralph run`, rewrite `PROMPT.md` from the current issue, workpad, or plan
packet.
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
- If `PROMPT.md` has not been regenerated from the current task context, do not run Ralph.
