# Codex SpecKit Layout

This repository keeps both Codex layouts used by `spec-kit` and current Codex surfaces:

- `.codex/commands/` is the active slash-command source for Codex in this repository.
- `.codex/prompts/` mirrors the same command content for `spec-kit`'s Codex template compatibility.
- `.codex/agents/` contains the repo-scoped Codex subagent roster and its internal usage guide.

When updating a SpecKit Codex command, update `.codex/commands/` first, then sync the matching file in `.codex/prompts/` as `speckit.<name>.md`.
