---
version: R1
created: 2026-03-16
updated: 2026-03-16
type: plan
---

# Rube Recipe Prompt Pack

Date: March 16, 2026

## Objective

Create a docs-only prompt pack for authoring reusable Rube recipes through the web UI and through
Rube MCP operator flows. The pack must be grounded in current Rube and Composio recipe guidance,
not in repo-local assumptions.

## Source Basis

- Rube README via Context7 on March 16, 2026:
  [composiohq/rube README](https://github.com/composiohq/rube/blob/master/README.md)
- Composio recipe and MCP docs via Context7 on March 16, 2026:
  [docs.composio.dev/toolkits/composio](https://docs.composio.dev/toolkits/composio)
- Mister Smith repo guidance:
  - `WORKFLOW.md`
  - `docs/linear/LINEAR.md`
  - `CLAUDE.md`

## Constraints

- Use Rube MCP as the gateway for external apps, APIs, MCP servers, and research.
- Keep prompts generic and reusable.
- Do not include PII, workspace-specific IDs, or user-specific defaults.
- Treat recipe authoring as a contract:
  - neutral description
  - human-friendly input schema
  - explicit output schema
  - self-contained reusable workflow logic
- Target the Rube "Create New Recipe" chat flow first.

## Deliverables

Create the following docs:

1. `docs/rube-recipes/README.md`
2. `docs/rube-recipes/recipe-prompt-contract.md`
3. `docs/rube-recipes/01-research-brief-recipe.md`
4. `docs/rube-recipes/02-docs-grounded-answer-recipe.md`
5. `docs/rube-recipes/03-linear-triage-update-recipe.md`
6. `docs/rube-recipes/04-status-digest-reporting-recipe.md`
7. `docs/rube-recipes/05-cross-app-intake-to-linear-recipe.md`
8. `docs/rube-recipes/06-content-transformation-pipeline-recipe.md`

## Shared Contract

Every archetype doc must use this section order:

1. `Goal`
2. `Apps and MCP Routing`
3. `User Inputs`
4. `Expected Outputs`
5. `Workflow Steps`
6. `Validation Rules`
7. `Fallbacks`
8. `Direct-Create Prompt`
9. `Execute-Then-Convert Prompt`
10. `MCP-Authoring Prompt`
11. `Worked Example`

## Routing Defaults

- Use Rube MCP as the gateway.
- Prefer Context7 for source-grounded documentation and code examples.
- Prefer Tavily for quick search, quick verification, or targeted extraction when connected.
- Prefer Parallel for deeper, broader multi-source synthesis.
- Prefer Linear for project state, issue state, and structured project updates.
- If a preferred research connector is unavailable, fall back to the next tool that still preserves
  source grounding rather than silently changing the task.

## Validation

- `npx markdownlint-cli2 "docs/rube-recipes/**/*.md" --config .markdownlint.json`
- Dry-read each archetype prompt to confirm the Rube UI questions are answered explicitly:
  - what the recipe does
  - which apps it uses
  - what inputs the user provides
  - what it outputs
- Cross-check all archetypes for section order, tone, and routing rules.

## Stop Conditions

- Stop and tighten an archetype if it becomes a meta-framework instead of a reusable recipe prompt.
- Split an archetype into narrower examples only if that is required to preserve clarity.
- Do not create or run live recipes in this phase.
