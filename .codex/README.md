# Codex SpecKit Layout

This repository keeps both Codex layouts used by `spec-kit` and current Codex surfaces:

- `.codex/commands/` is the active slash-command source for Codex in this repository.
- `.codex/prompts/` mirrors the same command content for `spec-kit`'s Codex template compatibility.
- `.codex/skills/` is the canonical home for Mister Smith repo-owned workflow skills.

## Canonical Workflow Skills

The Mister Smith workflow system is MCP-first:

- start with `.codex/skills/mister-smith-control-plane-router/`
- bootstrap or repair the local MCP registration with `.codex/skills/mister-smith-control-plane-bootstrap/`
- use the repo-local specialized skills as wrappers around the constitutional control-plane MCP

The compatibility shims under `~/.codex/skills/` are transitional only and should point back into
this repo.

When updating a SpecKit Codex command, update `.codex/commands/` first, then sync the matching file in `.codex/prompts/` as `speckit.<name>.md`.
