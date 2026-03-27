# Walkthrough — Mister Smith MS-103 Packet-020 Live Evaluation Handoff Prompt

## Original Prompt Summary

The source request asked for a handoff prompt, written through the prompt-improver workflow, for a
new session to perform live evaluations of the completed packet-020 parent issue `MS-103` after
the closure of `MS-103` through `MS-107`.

## Key Improvements Made

- converted the high-level ask into a repo-grounded fresh-session handoff prompt tied to current
  packet-020 closure truth
- combined the strongest parts of the existing live-run evaluation prompt and the issue-specific
  implementation handoff prompt
- made the live-proof boundary explicit:
  - baseline live proof is not automatically packet-020 proof
  - packet-020 claims must come from the actual transcript and operator surfaces
- added an evaluation-only boundary so the receiving agent does not reopen closed work or patch
  code just to force a result
- added a run-selection rule so the receiving agent chooses the narrowest honest live method
  without turning the session into an open-ended exploration exercise
- added explicit durable artifact expectations and final reporting requirements for observed versus
  inferred packet-020 fields

## Before / After Comparison

### Scope framing

- Before: "write a handoff prompt for evaluations"
- After: a prompt that names the exact parent issue, current closure state, read order, current
  code surfaces, and proof boundaries

### Live-proof boundary

- Before: implied live evaluations, but no explicit separation between packet-019 baseline proof
  and packet-020 proof
- After: the prompt explicitly forbids treating a baseline-only run as packet-020 proof

### Session boundary

- Before: no explicit guard against reopening completed work or slipping into implementation
- After: the prompt states that the session is evaluation-only and forbids reopening or patching
  unless a later follow-up is explicitly requested

## How To Use The Improved Prompt

1. Start a fresh Codex session in `/Users/macmain/MisterSmith`.
2. Paste the final prompt from the file below.
3. Let the receiving agent ground on current repo truth, run the live evaluations, and write the
   durable evaluation note plus artifacts.

## Final Prompt Location

- `docs/prompt-improver-spec/final-prompts/mister-smith-ms-103-packet-020-live-evaluation-handoff.md`
