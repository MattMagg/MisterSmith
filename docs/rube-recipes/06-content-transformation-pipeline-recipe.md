---
version: R1
created: 2026-03-16
updated: 2026-03-16
type: prompt-archetype
---

# Rube Recipe Archetype 06: Content Transformation Pipeline

## Goal

Create a reusable recipe that reads content from one source, transforms it according to explicit
rules, and then returns or publishes the transformed artifact.

Rube UI answers:

| UI question | Answer |
| --- | --- |
| What should the recipe do? | Transform content from one format or destination into another. |
| Which apps does it involve? | Rube MCP as the gateway, one source app, optional reference sources, and one destination app or markdown output. |
| What inputs should users provide? | Source location, transformation rules, target format, and destination. |
| What should it output? | Transformed content plus optional delivery confirmation. |

## Apps and MCP Routing

- Use Rube MCP as the gateway.
- Use a connected source app such as Google Docs, Notion, Slack, Gmail, CMS, or file storage.
- Use Context7 when the transformation depends on official formatting or API docs.
- Use Tavily only when public web references are required for grounding.
- Use a connected destination app when the transformed content should be published instead of
  returned directly.

## User Inputs

| Input | Required | Description |
| --- | --- | --- |
| `source_app` | yes | App containing the source content |
| `source_selector` | yes | Document, message, file, or page selector |
| `transformation_rules` | yes | Rewrite, summarize, reformat, classify, or extract rules |
| `target_format` | yes | Desired output format |
| `destination_app` | no | Optional place to publish the transformed content |

## Expected Outputs

- Transformed content in the target format.
- A concise summary of what was transformed.
- Optional delivery or publish confirmation when a destination is provided.

## Workflow Steps

1. Resolve the source app and source selector.
2. Fetch the source content through Rube MCP.
3. Apply the transformation rules and target format requirements.
4. If official formatting or product behavior matters, verify it through Context7.
5. Validate that the transformed content matches the requested format.
6. Return the transformed artifact or publish it to the destination app.

## Validation Rules

- Do not claim success unless the output matches the requested target format.
- If the source content cannot be fetched, stop instead of inventing inputs.
- If the destination publish step fails, return the transformed content and report the delivery
  failure separately.
- Preserve required source references when the transformation rules call for them.

## Fallbacks

- If the destination app is not provided, return the transformed content directly.
- If Context7 is unavailable for a docs-dependent transformation, continue only if the user accepts
  a best-effort result.
- If the source selector is ambiguous, stop and request clarification.

## Direct-Create Prompt

```text
Create a new reusable recipe for me.

What the recipe should do:
Read content from a connected source app, transform it according to explicit rules, and then return
or publish the transformed artifact.

Which apps and MCP tools it should use:
Use Rube MCP as the gateway. Use a connected source app such as Google Docs, Notion, Slack, Gmail,
CMS, or file storage. Use Context7 when the transformation depends on official formatting or API
docs. Use Tavily only for supplementary public web references. Use a connected destination app when
the transformed content should be published instead of returned directly.

What inputs users should provide:
- source_app
- source_selector
- transformation_rules
- target_format
- destination_app

What it should output:
- transformed content in the requested format
- concise transformation summary
- optional publish confirmation

Workflow logic to encode:
1. Resolve and fetch the source content.
2. Apply the requested transformation rules.
3. Verify docs-dependent formatting through Context7 when needed.
4. Validate the output format.
5. Return or publish the transformed artifact.

Validation rules:
Do not claim success unless the output matches the requested format and the source content was
actually retrieved. Report publish failures separately from transformation success.

Fallback behavior:
If the destination app is missing, return the content directly. If the source selector is ambiguous,
stop and ask for clarification.
```

## Execute-Then-Convert Prompt

```text
Execute a content transformation workflow with me once, then convert it into a reusable recipe.

Workflow to execute now:
Fetch source content, transform it according to my rules, and return or publish the result.

Apps and MCP routing:
Use Rube MCP as the gateway. Use the selected source app and destination app through Rube. Use
Context7 only when formatting or API behavior must match official docs.

Inputs to collect for this run:
- source_app
- source_selector
- transformation_rules
- target_format
- destination_app

Expected outputs from this run:
- transformed content
- concise summary of the transformation
- optional publish confirmation

Workflow steps to follow:
1. Resolve the source item.
2. Fetch the source content.
3. Apply the transformation rules.
4. Validate the result.
5. Publish if requested.
6. Convert the successful workflow into a reusable recipe.

Validation rules:
Do not convert the workflow into a recipe unless the transformed output matches the requested format
and the source content was retrieved successfully.

Fallback behavior:
Return the transformed content directly if no destination app is provided. Stop on ambiguous source
selectors.
```

## MCP-Authoring Prompt

```text
Author a reusable Rube recipe from this workflow specification.

Goal:
Read content from a connected source app, transform it according to explicit rules, and then return
or publish the result.

Routing:
Use Rube MCP as the gateway. Use the selected source app and optional destination app through Rube.
Use Context7 only when formatting or API behavior needs official docs.

Inputs:
- source_app
- source_selector
- transformation_rules
- target_format
- destination_app (optional)

Outputs:
- transformed content
- transformation summary
- optional publish confirmation

Workflow:
Resolve the source item, fetch the content, apply the requested transformation, validate the output
format, and then return or publish the result.

Validation:
Require successful source retrieval and explicit confirmation that the output matches the requested
format.

Fallbacks:
Return content directly when no destination app is provided. Stop on ambiguous selectors or missing
required source content.
```

## Worked Example

Example paste into the Rube UI:

```text
Create a recipe that transforms source content into a new format and optionally publishes it.

What the recipe should do:
Fetch a source document or message, transform it according to user-provided rules, and then return
or publish the transformed content.

Which apps does it involve:
Use Rube MCP as the gateway. Use a connected source app such as Google Docs, Notion, Slack, or
Gmail, and optionally publish to another connected destination app.

What inputs should users provide:
- source_app
- source_selector
- transformation_rules
- target_format
- destination_app

What should it output:
- transformed content
- summary of the transformation
- optional publish confirmation

Create it as a reusable recipe with neutral wording, explicit inputs, explicit outputs, and
self-contained workflow logic.
```
