# Walkthrough — Mister Smith Live Run Trace Evaluation Prompt

## Original Request Summary

Create a prompt for a new Codex session that goes through a live run of Mister Smith and evaluates
and traces the run thoroughly.

## Key Improvements Made

- anchored the prompt to the repo's current state rather than older recovery-era assumptions
- made current code and live runtime behavior the primary truth sources
- turned "trace the run thoroughly" into an explicit evidence checklist
- required a durable repo artifact rather than an ephemeral terminal-only conclusion
- kept provider/model claims honest and explicit
- kept Linear and Symphony out of the proof path except as optional development-state cross-checks

## Before / After Shape

### Before

- broad request for a live run and deep evaluation
- no explicit document start sequence
- no durable artifact requirement
- no concrete tracing markers

### After

- structured session brief with ordered phases
- current repo docs and code surfaces named up front
- explicit run evidence targets such as task result fields, autonomy status, lifecycle markers, and
  ToolBus boundaries
- durable evidence note requirement with clear contents
- explicit "do not claim" and stop-condition sections

## How To Use The Prompt

Start a new Codex session in `/Users/macmain/MisterSmith` and give it the final prompt from:

`docs/prompt-improver-spec/final-prompts/mister-smith-live-run-trace-evaluation.md`

The receiving session should then:

1. verify current repo truth
2. run Mister Smith live
3. collect and compare evidence
4. leave a durable evidence note
5. summarize what the run proves and what remains unproven

## Final Prompt Location

`docs/prompt-improver-spec/final-prompts/mister-smith-live-run-trace-evaluation.md`
