---
version: R1
created: 2026-03-16
updated: 2026-03-16
type: prompt-archetype
---

# Rube Recipe Archetype 01: Research Brief

## Goal

Create a reusable recipe that researches a user-provided topic and returns a concise brief with
grounded sources.

Rube UI answers:

| UI question | Answer |
| --- | --- |
| What should the recipe do? | Research a topic, synthesize the findings, and produce a reusable brief. |
| Which apps does it involve? | Rube MCP as the gateway, with Tavily or Parallel for research and Context7 for official docs when relevant. |
| What inputs should users provide? | Topic, audience, depth, timeframe, and optional output destination. |
| What should it output? | A markdown brief, source list, and optional published document or message. |

## Apps and MCP Routing

- Use Rube MCP as the gateway for all external research tools.
- Prefer Tavily for quick search, quick verification, or targeted extraction when connected.
- Prefer Parallel for broader multi-source research or structured synthesis.
- Use Context7 when the topic depends on official technical documentation or versioned APIs.
- If the user wants the brief posted or stored, allow an optional destination app such as Notion,
  Google Docs, Slack, or email through Rube.

## User Inputs

| Input | Required | Description |
| --- | --- | --- |
| `research_topic` | yes | The topic or question to investigate |
| `audience` | yes | The intended audience, such as operator, executive, engineer, or customer |
| `depth` | yes | Brief depth: short, medium, or deep |
| `timeframe` | no | Date window or recency filter, such as last 30 days |
| `output_destination` | no | Where to send the brief, or `none` to return markdown only |

## Expected Outputs

- A markdown brief with clear headings and source links.
- A structured source list that records which tools or URLs supported the answer.
- Optional published output to the chosen destination app when one is provided.

## Workflow Steps

1. Resolve the research topic, audience, and requested depth.
2. Route through Rube-connected research tools:
   - Tavily for initial discovery when connected
   - Parallel for deeper synthesis when the scope is broad
   - Context7 for official technical docs when the topic needs them
3. Collect enough grounded source material to support the brief.
4. Synthesize the brief in the requested depth and audience tone.
5. Validate that every major claim is tied to a source.
6. Return the markdown brief and structured source list, or publish them to the optional
   destination.

## Validation Rules

- Do not return success unless the brief includes source links.
- Flag missing or weak sourcing instead of filling the gap with unsupported claims.
- If a destination app is provided, confirm that the publish step succeeded before marking the run
  complete.

## Fallbacks

- If Tavily is unavailable, use Parallel for discovery and synthesis.
- If the topic depends on official docs and Context7 is unavailable, say that the answer could not
  be fully source-grounded and continue only if the user accepts web-only sources.
- If no destination app is provided, return markdown only.

## Direct-Create Prompt

```text
Create a new reusable recipe for me.

What the recipe should do:
Research a user-provided topic, synthesize the findings into a brief, and return a grounded
markdown report that can optionally be posted or saved to another app.

Which apps and MCP tools it should use:
Use Rube MCP as the gateway for all external tools. Prefer Tavily for quick research and targeted
extraction when connected. Prefer Parallel for broader multi-source research or structured
synthesis. Use Context7 when the topic depends on official technical documentation or versioned
APIs. If an output destination is provided, use the chosen destination app through Rube.

What inputs users should provide:
- research_topic: the topic or question to investigate
- audience: who the brief is for
- depth: short, medium, or deep
- timeframe: optional time window
- output_destination: optional destination app or none

What it should output:
- markdown research brief with source links
- structured source list
- optional published copy in the destination app

Workflow logic to encode:
1. Resolve the topic, audience, and scope.
2. Gather sources using the preferred Rube-connected research tools.
3. Use Context7 for official docs when the topic requires documentation grounding.
4. Synthesize a brief that matches the requested depth.
5. Validate that every major claim is source-grounded.
6. Return the brief and source list, or publish them if a destination is provided.

Validation rules:
Do not claim success unless the brief includes citations or source links. Fail loudly when there is
not enough source material to support the requested brief.

Fallback behavior:
If Tavily is unavailable, use Parallel. If official docs are required and Context7 is unavailable,
stop and report that limitation unless the user accepts a web-only brief.

Author the recipe so that:
- Rube MCP is the gateway for external tools
- the recipe description is neutral and reusable
- the input schema uses human-friendly names
- the output schema is explicit
- the workflow logic is self-contained
- no PII or workspace-specific defaults are baked in
```

## Execute-Then-Convert Prompt

```text
I want to create a reusable research brief recipe, but first execute the workflow once with me and
then convert it into a recipe automatically.

Workflow to execute now:
Research a topic that I provide, produce a grounded brief, and return a source list.

Apps and MCP routing:
Use Rube MCP as the gateway. Prefer Tavily for quick research, Parallel for deeper synthesis, and
Context7 for official docs when needed.

Inputs to collect for this run:
- research_topic
- audience
- depth
- timeframe if relevant
- output_destination if I want the result posted somewhere

Expected outputs from this run:
- one markdown research brief with source links
- one structured source list
- optional published copy if I provide a destination

Workflow steps to follow:
1. Confirm the topic and audience.
2. Gather sources through the Rube-connected research tools.
3. Synthesize the brief.
4. Validate that citations are present.
5. Return the result and then convert the workflow into a reusable recipe.

Validation rules:
Do not convert the workflow into a recipe unless the live run succeeds and includes source-grounded
output.

Fallback behavior:
If Tavily is unavailable, use Parallel. If official docs are required and Context7 is unavailable,
report the limitation before converting the run into a recipe.
```

## MCP-Authoring Prompt

```text
Author a reusable Rube recipe from this workflow specification.

Goal:
Produce a source-grounded research brief on a user-provided topic.

Routing:
Use Rube MCP as the gateway. Prefer Tavily for quick discovery, Parallel for deeper synthesis, and
Context7 for official documentation.

Inputs:
- research_topic
- audience
- depth
- timeframe (optional)
- output_destination (optional)

Outputs:
- markdown brief
- structured source list
- optional published artifact

Workflow:
Resolve the topic, gather sources, synthesize a grounded brief, validate the citations, and then
return or publish the result.

Validation:
Require source links for major claims and fail loudly when sourcing is incomplete.

Fallbacks:
Tavily to Parallel for research fallback. Stop for missing official-doc grounding when Context7 is
required but unavailable.
```

## Worked Example

Example paste into the Rube UI:

```text
Create a new recipe that researches a market topic for an operator audience.

What the recipe should do:
Research the user-provided topic and produce a concise market brief for operations leaders.

Which apps does it involve:
Use Rube MCP as the gateway. Prefer Tavily for recent web research, Parallel for deeper synthesis,
and Context7 if the topic includes a technical product with official documentation.

What inputs should users provide:
- research_topic
- audience
- depth
- timeframe
- output_destination

What should it output:
- a markdown brief with sections for summary, key findings, risks, and recommendations
- a source list with links

Create it as a reusable recipe with neutral wording, explicit inputs, explicit outputs, and
self-contained workflow logic.
```
