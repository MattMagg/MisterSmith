# Walkthrough — Mister Smith MS-114 Packet-021 Contract-Freeze Handoff Prompt

## Original Prompt Summary

The source request asked for two things:

1. mirror packet `021` into Linear as issues/tasks/spec context
2. create a new-session prompt, using the prompt-improver workflow, that initializes
   implementation of the new packet

The main prompt-design challenge was deciding what the receiving session should actually start on.
Once the Linear packet structure existed, the honest answer was not the packet parent. It was the
first runnable child issue: `MS-114`.

## Key Improvements Made

- converted a broad "initialize this spec implementation" ask into a direct fresh-session handoff
  for `MS-114`
- grounded the prompt on the new tracker structure:
  - parent packet `MS-113`
  - first runnable slice `MS-114`
  - attached Linear doc `Packet 021 spec packet`
- made the contract artifact a first-class read instead of assuming the receiving agent would
  infer it from `plan.md` or `tasks.md`
- added explicit boundaries that keep the session out of `MS-115`, `MS-116`, `MS-117`, and
  `MS-118`
- kept the repo's Smith-first lifecycle, validation, and clean-closure requirements intact

## Before / After Comparison

### Session target

- Before: ambiguous between packet-parent initialization and first-slice implementation
- After: explicitly targets `MS-114`, the first runnable contract-freeze slice

### Contract grounding

- Before: could have treated the contract artifact as just one more packet file
- After: the prompt requires the receiving agent to read and honor the published supervision
  contract before editing code

### Tracker integration

- Before: "put it in Linear" did not yet give the next session actionable issue context
- After: the prompt is anchored on the concrete Linear parent, child issue, attached packet doc,
  and suggested branch name

## How To Use The Improved Prompt

1. Start a fresh Codex session in `/Users/macmain/MisterSmith`.
2. Paste the final prompt from the file below.
3. Let the receiving agent execute `MS-114` end to end before moving to later packet-021 slices.
4. Expect the receiving agent to keep repo and Linear state aligned and to stop at the contract
   freeze boundary rather than widening into later packet work.

## Final Prompt Location

- `docs/prompt-improver-spec/final-prompts/mister-smith-ms-114-packet-021-contract-freeze-handoff.md`
