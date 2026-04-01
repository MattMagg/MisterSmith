# SpecKit Refresh From Source

Date: April 1, 2026
Status: Completed

## Objective

Refresh the repo-local SpecKit scaffolding from the current upstream source release and make sure
the local constitution stays aligned instead of being silently replaced by the generic template.

## Scope

- `.specify/`
- `.codex/commands/`
- `.codex/prompts/`
- `docs/plans/2026-04-01-speckit-refresh-from-source.md`

## Assumptions

- Current repo state uses SpecKit `0.4.3` in `.specify/init-options.json`.
- Upstream `github/spec-kit` release `v0.4.4` is the latest release as of April 1, 2026.
- The repo-local `.specify/memory/constitution.md` is customized and must be reviewed, not blindly
  overwritten.

## Constraints

- Do not touch `specs/`.
- Keep the refresh scoped to SpecKit scaffolding and constitution alignment.
- Preserve adjacent timestamp backups before running the refresh.
- Do not revert unrelated dirty files in the primary checkout.

## Validation

- inspect touched-file diff after refresh
- `git diff --check`
- targeted markdown lint if constitution or markdown prompt files change

## Stop Conditions

- local SpecKit scaffolding reflects upstream `v0.4.4`
- local constitution is still repo-specific and not replaced by a generic upstream default
- touched-file set is bounded and intelligible

## Completion Note

Completed work:

- confirmed the repo was on SpecKit `0.4.3` and upstream `github/spec-kit` had moved to `v0.4.4`
- refreshed the repo with
  `uvx --from git+https://github.com/github/spec-kit.git@v0.4.4 specify init --here --force --ai codex --ai-skills --no-git`
- captured the upstream behavior change for Codex: prompt-based init is deprecated and the
  supported path now requires `--ai-skills`
- kept the refresh bounded to `.specify/` because `.codex/` prompt files were not touched by the
  upstream refresh path in this repo
- verified `.specify/memory/constitution.md` was preserved unchanged and that
  `.specify/templates/constitution-template.md` also did not change in this upstream release
- normalized upstream template markdown so the repo diff and lint checks pass cleanly

Touched files:

- `.specify/init-options.json`
- `.specify/scripts/bash/create-new-feature.sh`
- `.specify/scripts/bash/update-agent-context.sh`
- `.specify/templates/spec-template.md`
- `.specify/templates/plan-template.md`
- `.specify/templates/tasks-template.md`

Validation evidence:

- `git diff --check`
- `npx markdownlint-cli2 "docs/plans/2026-04-01-speckit-refresh-from-source.md" ".specify/memory/constitution.md" ".specify/templates/*.md" --config .markdownlint.json`
