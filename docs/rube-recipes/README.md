---
version: R1
created: 2026-03-16
updated: 2026-03-16
type: prompt-pack
---

# Rube Recipe Prompt Workflow Pack

## Purpose

This directory contains paste-ready prompt workflows for Rube recipe authoring.

The pack is designed for two authoring surfaces:

- the Rube web UI "Create New Recipe" chat
- operator or agent flows that author recipes through Rube MCP

The documents are grounded in current Rube and Composio guidance verified through Context7 on
March 16, 2026.

## Source Basis

- [Rube README](https://github.com/composiohq/rube/blob/master/README.md)
- [Composio recipe and MCP docs](https://docs.composio.dev/toolkits/composio)
- Repo guidance that standardizes Rube as the external gateway:
  - [CLAUDE.md](/Users/macmain/MisterSmith/CLAUDE.md)
  - [WORKFLOW.md](/Users/macmain/MisterSmith/WORKFLOW.md)
  - [docs/linear/LINEAR.md](/Users/macmain/MisterSmith/docs/linear/LINEAR.md)

## Pack Rules

- Use Rube MCP as the gateway for external apps, APIs, MCP servers, and research.
- Keep recipe descriptions neutral and reusable.
- Ask for human-friendly inputs, not internal IDs unless IDs are the only stable selector.
- Make expected outputs explicit.
- Ask Rube to produce self-contained workflow logic rather than hidden operator assumptions.
- Keep prompts free of PII and workspace-specific defaults.

## When To Use Which Doc

| File | Use When |
| --- | --- |
| `recipe-prompt-contract.md` | You need the canonical section order and authoring rules |
| `01-research-brief-recipe.md` | You want a reusable research or briefing workflow |
| `02-docs-grounded-answer-recipe.md` | You want source-grounded documentation answers with citations |
| `03-linear-triage-update-recipe.md` | You want issue triage, status changes, or structured Linear updates |
| `04-status-digest-reporting-recipe.md` | You want periodic or on-demand summaries across tools |
| `05-cross-app-intake-to-linear-recipe.md` | You want inbound items routed into Linear with decision logic |
| `06-content-transformation-pipeline-recipe.md` | You want text or document transformation across source and destination apps |

## Standard Authoring Flow

1. Pick the closest archetype.
2. Fill in the placeholders with task-specific details.
3. Paste either the `Direct-Create Prompt` or the `Execute-Then-Convert Prompt` into the Rube UI.
4. If you are operating through tools instead of the UI, use the `MCP-Authoring Prompt`.
5. Keep the recipe generic, with explicit inputs, outputs, routing, and validation.

## Routing Defaults

- Context7: source-grounded docs and code examples
- Tavily: lighter search or targeted extraction when connected
- Parallel: deeper multi-source synthesis
- Linear: project management state and issue updates

If a preferred research connector is unavailable, the prompt should explicitly tell Rube to fall
back to the next best grounded source rather than improvising.

## Validation Checklist

Use this checklist before considering a prompt ready:

- The prompt answers the four Rube UI questions explicitly.
- The apps and toolkits are named.
- The user inputs are explicit and human-friendly.
- The outputs are explicit and reviewable.
- The workflow steps are concrete enough to become reusable logic.
- Validation rules name what success looks like.
- Fallback behavior is declared.
- The prompt tells Rube to keep the recipe description neutral and reusable.
