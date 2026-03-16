---
version: R1
created: 2026-03-16
updated: 2026-03-16
type: prompt-contract
---

# Rube Recipe Prompt Contract

This file defines the stable section order and authoring rules for every prompt workflow doc in
this directory.

## Goal

State the recipe outcome in one or two sentences and answer the Rube UI's first question directly:
what should the recipe do.

Required content:

- the business or operator outcome
- the primary action or workflow
- a plain-language summary that a human can paste into the UI without extra context

Preferred shape:

```text
Create a reusable recipe that [primary action] for [target user or artifact].
The recipe should [main outcome] and stay generic enough for repeated use.
```

## Apps and MCP Routing

State which apps and MCP-backed tools the recipe should use, and in what order.

Required content:

- named toolkits or apps
- the rule that Rube MCP is the gateway
- preferred routing path and fallback path

Required defaults:

- Context7 for source-grounded docs and code examples
- Tavily for quick search or targeted extraction when connected
- Parallel for broader research or structured synthesis
- Linear for project state and issue actions

## User Inputs

Answer the Rube UI's third question directly: what should the user provide when running the recipe.

Required content:

- human-friendly parameter names
- what each parameter means
- required versus optional inputs
- format guidance

Avoid:

- raw internal IDs unless they are the only stable selector
- hidden assumptions that depend on one workspace or one user

## Expected Outputs

Answer the Rube UI's fourth question directly: what should the recipe output.

Required content:

- structured outputs or side effects
- success signal
- artifact destination if relevant

Preferred shape:

- one sentence describing the human-visible output
- one sentence describing machine-usable structured output

## Workflow Steps

Describe the reusable logic Rube should encode.

Required content:

1. gather or resolve inputs
2. route through the right Rube-connected tools
3. perform the core workflow
4. validate the result
5. return or publish the output

When possible, tell Rube to infer:

- a neutral description
- a human-friendly input schema
- an explicit output schema
- self-contained workflow logic

## Validation Rules

Describe how the recipe should prove that it succeeded.

Required content:

- what must be checked before returning success
- what should fail loudly
- what should be included in the final output for traceability

Examples:

- citations or source links are present
- the target issue or document was updated successfully
- the summary is grounded in fetched source material

## Fallbacks

Describe the next-best path when a preferred connector or input is unavailable.

Required content:

- connector fallback
- missing-data behavior
- no-silent-success rule

Preferred shape:

```text
If [preferred tool] is unavailable, use [fallback tool].
If required inputs are missing, stop and request them instead of inventing values.
```

## Direct-Create Prompt

Use this mode when the user already knows the workflow they want and wants Rube to author the
recipe immediately.

Template:

```text
Create a new reusable recipe for me.

What the recipe should do:
[goal]

Which apps and MCP tools it should use:
[apps_and_mcp_routing]

What inputs users should provide:
[user_inputs]

What it should output:
[expected_outputs]

Workflow logic to encode:
[workflow_steps]

Validation rules:
[validation_rules]

Fallback behavior:
[fallbacks]

Author the recipe so that:
- Rube MCP is the gateway for all external tools and APIs
- the description is neutral and reusable
- the input schema uses human-friendly names and format guidance
- the output schema is explicit
- the workflow logic is self-contained
- no PII or workspace-specific defaults are baked into the recipe
```

## Execute-Then-Convert Prompt

Use this mode when the workflow is easier to demonstrate first and then convert into a reusable
recipe.

Template:

```text
I want to create a reusable recipe, but first execute the workflow with me once and then convert
it into a recipe automatically.

Workflow to execute now:
[goal]

Apps and MCP routing:
[apps_and_mcp_routing]

Inputs to collect for this run:
[user_inputs]

Expected outputs from this run:
[expected_outputs]

Workflow steps to follow:
[workflow_steps]

Validation rules:
[validation_rules]

Fallback behavior:
[fallbacks]

After the live run succeeds, convert it into a reusable recipe with:
- a neutral description
- human-friendly input schema
- explicit output schema
- self-contained workflow logic
- no user-specific defaults except placeholder examples
```

## MCP-Authoring Prompt

Use this mode when an operator or agent is directing Rube from an MCP-aware environment instead of
the Rube web UI.

Template:

```text
Author a reusable Rube recipe from this workflow specification.

Goal:
[goal]

Routing:
[apps_and_mcp_routing]

Inputs:
[user_inputs]

Outputs:
[expected_outputs]

Workflow:
[workflow_steps]

Validation:
[validation_rules]

Fallbacks:
[fallbacks]

Recipe authoring rules:
- use Rube MCP as the gateway to external apps, APIs, and MCP servers
- keep the description neutral and generic
- use human-friendly input fields
- define explicit outputs
- make the workflow reusable and self-contained
- fail loudly when source data is missing or invalid
```

## Worked Example

Minimal example:

```text
Goal:
Create a reusable recipe that produces a cited research brief on a user-provided topic.

Apps and MCP routing:
Use Rube MCP as the gateway. Prefer Tavily for quick discovery, Parallel for deeper synthesis, and
Context7 when the topic depends on official technical documentation.

User inputs:
- research_topic: topic to investigate
- audience: who the brief is for
- depth: short, medium, or deep

Expected outputs:
- markdown brief with citations
- structured metadata listing sources used

Workflow steps:
1. Resolve the topic and audience.
2. Gather initial sources through Rube-connected research tools.
3. Synthesize a grounded brief.
4. Validate that citations are present.
5. Return the brief and source list.

Validation rules:
Do not claim success unless the brief includes citations and every major claim is source-grounded.

Fallbacks:
If Tavily is unavailable, use Parallel for discovery. If neither is available, stop and report the
missing connector.
```
