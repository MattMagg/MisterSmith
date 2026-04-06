# Docs Guide

This directory now separates active tracked docs from local-only historical or private material.

## Active Tracked Docs

- `current-state.md`: repo-wide current truth and document router
- `direction.md`: strategic direction and sequencing
- `ms_recent_context.md`: recent repo context
- `RESEARCH_CHECKPOINT.md`: research corpus usage guidance

## Structured Active Subdirectories

- `automation-reports/`: dated automation output that is still repo-visible
- `audits/`: tracked audit and architectural review documents
- `code-review/`: review writeups and validation-oriented review notes
- `examples/`: example session/auth documents that still belong in the repo
- `plans/`: execution plans and plan artifacts
- `reports/`: standalone tracked reports that do not fit the plan/audit buckets

## Local-Only Trees

The following directories are intentionally kept local and ignored by git:

- `linear/`
- `prompt-improver-spec/`
- `pulse-tasks/`
- `research-output/`
- `research-prompts/`
- `/archive/` at the repo root for local historical spillover and uncertain legacy files

Those paths may still be useful locally, but they are no longer treated as tracked public repo
content.
