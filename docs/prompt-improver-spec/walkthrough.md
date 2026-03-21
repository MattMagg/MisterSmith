# Walkthrough — Mister Smith Next SpecKit Epic Handoff Prompt

Status: Historical prompt-improver walkthrough for the packet-016 planning session. Do not use
this file as a current routing document; use
`docs/plans/2026-03-21-post-packet-016-development-checkpoint.md` for the current forward
checkpoint.

## Original Request Summary

Generate a handoff prompt for a new Codex session that starts the next phase of development by
creating and scoping the next bounded SpecKit epic for Mister Smith.

## Key Improvements Made

- anchored the prompt to the March 19 checkpoint instead of the older frontier-direction note
- forced the next session to ground on current runtime and stress-evaluation evidence before
  choosing scope
- kept the work bounded to one next SpecKit packet instead of implementation or general cleanup
- required a real `specs/` packet as output rather than an informal planning memo
- added an explicit decision point for whether any remaining post-`MS-77` external-agent work
  belongs in the same epic or a later one

## Before / After Shape

### Before

- broad request to start the next phase
- no explicit forward-authority document
- no requirement to use the March 19 evaluation notes
- no forced decision on external-agent follow-on scope

### After

- ordered read sequence through current-state, the March 19 checkpoint, runtime proof, stress
  evaluation, and `MS-77`
- bounded mission: create one next SpecKit packet only
- explicit packet deliverables under the next numbered `specs/` directory
- direct requirement to state what is in scope now and what is deferred

## How To Use The Prompt

Start a new Codex session in `/Users/macmain/MisterSmith` and give it the final prompt from:

`docs/prompt-improver-spec/final-prompts/mister-smith-next-speckit-epic-handoff.md`

The receiving session should then:

1. ground on current repo authority and March 19 evidence
2. decide the next bounded epic honestly
3. write the next full SpecKit packet under `specs/`
4. state whether any post-`MS-77` external-agent work stays inside that epic or moves to a later
   one
5. stop before implementation

## Final Prompt Location

`docs/prompt-improver-spec/final-prompts/mister-smith-next-speckit-epic-handoff.md`
