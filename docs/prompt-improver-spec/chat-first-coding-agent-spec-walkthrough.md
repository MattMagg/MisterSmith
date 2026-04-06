# Chat-First Coding-Agent Spec Walkthrough

## Original Prompt Summary

The source ask was to create a prompt for a new Codex session that would generate a spec for a
new frontier feature: making Mister Smith fully competitive with the best chat-first coding agents,
while using the frontier mandate and the full `speckit-specify` workflow.

## What Changed And Why

- I turned the vague “consider this skill” wording into an explicit first step:
  use Smith legitimacy tools before the spec flow.
- I added the repo reading order so the new session starts from current truth instead of jumping
  straight into speculative packet writing.
- I made the SpecKit flow explicit:
  read init options, run the create-new-feature script once, use the template, create the
  checklist, and iterate until the spec is clean.
- I added an honest scope rule:
  if the goal is too large for one spec, the next session must define the highest-leverage bounded
  slice instead of pretending one packet can solve everything.

## Highest-Impact Fixes

- Explicit skill order
- Explicit repo authority order
- Explicit SpecKit execution order
- Explicit non-goals to stop workflow and implementation drift

## Assumptions

- The next session should create a spec, not implementation work.
- The full ambition may be epic-sized, so the spec may need to define a first bounded frontier
  slice rather than the entire long-term product.

## Final Prompt Location

- [mister-smith-chat-first-coding-agent-spec.md](/Users/macmain/MisterSmith/docs/prompt-improver-spec/final-prompts/mister-smith-chat-first-coding-agent-spec.md)
