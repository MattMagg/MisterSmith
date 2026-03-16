---
version: R1
created: 2026-03-16
updated: 2026-03-16
type: prompt-archetype
---

# Rube Recipe Archetype 02: Docs-Grounded Answer

## Goal

Create a reusable recipe that answers a user question using official documentation first, then
returns a cited answer.

Rube UI answers:

| UI question | Answer |
| --- | --- |
| What should the recipe do? | Answer a question using official docs and grounded sources. |
| Which apps does it involve? | Rube MCP, Context7 for official docs, and Tavily for fallback verification when needed. |
| What inputs should users provide? | Product or library name, question, version if relevant, and audience or output format. |
| What should it output? | A cited answer, source links, and a short note about evidence quality. |

## Apps and MCP Routing

- Use Rube MCP as the gateway.
- Prefer Context7 for official product, library, or framework documentation.
- Use Tavily only when the answer needs supplementary web verification or when official docs do not
  fully cover the question.
- If the answer should be posted to a destination app, do that through Rube after the grounded
  answer is built.

## User Inputs

| Input | Required | Description |
| --- | --- | --- |
| `product_or_library` | yes | Product, API, library, or framework name |
| `question` | yes | The user question to answer |
| `version` | no | Version or release line if the question is version-specific |
| `audience` | no | Intended audience or tone |
| `output_destination` | no | Optional destination app or `none` |

## Expected Outputs

- A concise answer grounded in official documentation.
- Source links or citations for the answer.
- An evidence note that distinguishes official docs from supplemental web sources.
- Optional posted or saved copy in a destination app.

## Workflow Steps

1. Resolve the product, library, and version if provided.
2. Query Context7 for official docs and examples first.
3. Use Tavily only for supplementary verification or to fill clearly identified gaps.
4. Build a concise answer that separates confirmed facts from inference.
5. Validate that the answer includes source links and an evidence note.
6. Return or publish the result.

## Validation Rules

- Do not present unsupported claims as confirmed.
- If official docs are missing or insufficient, say so explicitly.
- When version is provided, ensure the answer is version-aware.
- If a destination app is used, confirm the publish step succeeded.

## Fallbacks

- If Context7 is unavailable, stop and report that the docs-grounded path cannot be completed
  honestly, unless the user explicitly accepts a web-only answer.
- If Tavily is unavailable, continue with official docs only and label the answer accordingly.
- If version is missing and version matters, ask Rube to request it or clearly note the assumption.

## Direct-Create Prompt

```text
Create a new reusable recipe for me.

What the recipe should do:
Answer a user question using official documentation first, then return a concise cited answer.

Which apps and MCP tools it should use:
Use Rube MCP as the gateway. Prefer Context7 for official documentation and code examples. Use
Tavily only for supplementary verification or gap-filling when the official docs are insufficient.
If I provide an output destination, use the destination app through Rube after the answer is ready.

What inputs users should provide:
- product_or_library
- question
- version if relevant
- audience if relevant
- output_destination if they want the answer posted somewhere

What it should output:
- concise cited answer
- source links
- evidence note describing whether the answer came from official docs only or from docs plus web
  verification

Workflow logic to encode:
1. Resolve the product, version, and question.
2. Query official docs through Context7 first.
3. Use Tavily only when extra verification is needed.
4. Draft a concise answer grounded in the retrieved sources.
5. Validate that citations are present and unsupported claims are labeled.
6. Return or publish the result.

Validation rules:
Do not claim success unless the answer is grounded in retrieved documentation and includes source
links. Distinguish confirmed facts from inference.

Fallback behavior:
If Context7 is unavailable, stop unless the user explicitly allows a web-only answer. If Tavily is
unavailable, continue with official docs only and label the result.
```

## Execute-Then-Convert Prompt

```text
Execute a docs-grounded answer workflow with me once, then convert it into a reusable recipe.

Workflow to execute now:
Answer my documentation question using official docs first and return a cited answer.

Apps and MCP routing:
Use Rube MCP as the gateway. Prefer Context7 for official docs. Use Tavily only for supplementary
verification when necessary.

Inputs to collect for this run:
- product_or_library
- question
- version if relevant
- audience if relevant
- output_destination if I want the answer posted elsewhere

Expected outputs from this run:
- concise grounded answer
- source links
- evidence note

Workflow steps to follow:
1. Resolve the product and version.
2. Retrieve official documentation through Context7.
3. Supplement with Tavily only if needed.
4. Produce a cited answer.
5. Validate the evidence.
6. Convert the successful workflow into a reusable recipe.

Validation rules:
Do not convert the workflow into a recipe unless the live answer includes citations and makes the
evidence boundary explicit.

Fallback behavior:
Stop if Context7 is unavailable unless I approve a web-only answer.
```

## MCP-Authoring Prompt

```text
Author a reusable Rube recipe from this workflow specification.

Goal:
Answer a user question with official documentation first and return a concise cited answer.

Routing:
Use Rube MCP as the gateway. Prefer Context7. Use Tavily only as a supplement.

Inputs:
- product_or_library
- question
- version (optional)
- audience (optional)
- output_destination (optional)

Outputs:
- concise answer
- source links
- evidence note

Workflow:
Resolve the target product, query official docs through Context7, optionally verify with Tavily,
compose a cited answer, validate the evidence, and return or publish the result.

Validation:
Require citations and a clear statement about evidence quality.

Fallbacks:
Stop when Context7 is unavailable unless the user explicitly accepts a weaker web-only path.
```

## Worked Example

Example paste into the Rube UI:

```text
Create a recipe that answers technical questions using official documentation first.

What the recipe should do:
Take a user question about a product or library, use official docs to answer it, and return a
cited answer with a short evidence note.

Which apps does it involve:
Use Rube MCP as the gateway. Prefer Context7 for official docs and use Tavily only for
supplementary verification if needed.

What inputs should users provide:
- product_or_library
- question
- version
- audience
- output_destination

What should it output:
- concise cited answer
- source links
- evidence note

Create it as a reusable recipe with neutral wording, explicit inputs, explicit outputs, and
self-contained workflow logic.
```
