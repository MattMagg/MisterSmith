# Walkthrough — Mister Smith Pre-Packet Rube Tavily Research Handoff

## Original Prompt Summary

The user wanted a fresh-session handoff prompt for a **research-only** pass on the pre-spec
packet-prep docs under `docs/packet-prep/`.

The receiving session must:

- use **Rube MCP with the Tavily app** for the external research path
- thoroughly audit the packet-prep docs against official documentation and primary sources
- determine the most current stable versions or revisions of anything mentioned
- distinguish real protocols from vendor APIs, comparator docs, and product-specific feature pages
- refresh the packet-prep docs only if the evidence justifies it

This is explicitly **not** a spec-writing session, implementation session, or generic brainstorm.

## Final Prompt Location

`docs/prompt-improver-spec/final-prompts/mister-smith-pre-packet-rube-tavily-research-handoff.md`

## Key Improvements Made

- turned a broad "research everything" ask into a bounded **packet-prep external-doc audit**
- made the **Rube plus Tavily workflow explicit** instead of leaving research routing ambiguous
- required a **source inventory** of every external family the packet-prep docs rely on
- required **official-source version and revision verification** instead of hand-wavy "latest"
  claims
- added a mandatory **classification layer** so the receiving agent distinguishes:
  - open protocol
  - official vendor API or specification
  - comparator framework doc
  - product-specific feature or config model
- preserved repo authority order so external findings sharpen the packet-prep docs without
  overriding repo truth
- added anti-patterns that specifically block low-signal one-word searches and fake protocol
  categorization

## Before / After Highlights

### Before

- the user goal was clear, but the research method could still collapse into generic web search
- "actual protocols" was broad enough to blur protocol truth with vendor/product features
- "everything about the pre-packet docs" risked widening into architecture redesign or spec work

### After

- the final prompt requires **Rube MCP plus Tavily** and explicit tool-path validation
- the research loop is organized around **inventory, official extraction, version validation, and
  protocol classification**
- edit scope is tightly bound to `docs/packet-prep/README.md` plus dossiers `022-028`
- the receiving session is required to keep packet-prep work separate from spec authoring and code
  implementation

## How To Use The Final Prompt

1. Open a new Codex session in `/Users/macmain/MisterSmith`.
2. Provide the prompt as the kickoff instruction.
3. Fill or preserve these placeholders as needed:
   - `<repo_root>`
   - `<packet_prep_root>`
   - `<direction_doc>`
   - `<current_state_doc>`
4. Let the receiving agent:
   - read the local authority docs first
   - inventory the external source families
   - route external research through Rube MCP and Tavily
   - update the packet-prep docs only where evidence justifies it

## Cleanup Performed

- created the draft prompt under `docs/prompt-improver-spec/final-prompts/`
- created the production prompt in the same directory
- removed the draft file after finalization
