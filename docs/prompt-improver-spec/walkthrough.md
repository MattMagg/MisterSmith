# Walkthrough — Mister Smith MS-106 Orchestration Provenance Handoff Prompt

## Original Prompt Summary

Produce a prompt to continue with the next task after the just-landed packet-020 slice, using the
`prompt-improver` workflow.

## Key Improvements Made

- identified the real next task as `MS-106` instead of assuming the next packet heading alone was
  sufficient
- grounded the prompt on current repo, packet, and Linear issue truth
- preserved Smith-first staging requirements because `MS-106` is still in backlog
- kept the prompt bounded to provenance projection and inspection surfaces rather than solving the
  implementation itself
- made validation and clean-closure expectations explicit

## Before / After Comparison

### Before

- generic request for a continuation prompt
- no explicit next issue id
- no control-plane staging instructions
- no explicit scope guardrails or validation requirements

### After

- concrete fresh-session prompt for `MS-106`
- direct reading order through repo authority and packet-020 docs
- Smith-first issue staging and workpad reconciliation instructions
- bounded scope, non-goals, validation, and closure sections

## How To Use The Improved Prompt

Start a new Codex session in `/Users/macmain/MisterSmith` and give it the final prompt from:

`docs/prompt-improver-spec/final-prompts/mister-smith-ms-106-orchestration-provenance-handoff.md`

The receiving session should then:

1. verify current repo and issue state
2. stage `MS-106` into the watched queue if needed
3. execute the bounded provenance-projection slice
4. validate honestly
5. finish the full git/PR/Linear closure lane

## Final Prompt Location

`docs/prompt-improver-spec/final-prompts/mister-smith-ms-106-orchestration-provenance-handoff.md`
