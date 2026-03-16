---
version: R1
created: 2026-03-16
updated: 2026-03-16
type: prompt-archetype
---

# Rube Recipe Archetype 04: Status Digest and Reporting

## Goal

Create a reusable recipe that gathers updates from one or more systems and turns them into a clean
status digest for a chosen audience.

Rube UI answers:

| UI question | Answer |
| --- | --- |
| What should the recipe do? | Gather status data and produce a digest or report. |
| Which apps does it involve? | Rube MCP as the gateway, usually Linear plus optional supporting tools such as GitHub, Slack, or docs destinations. |
| What inputs should users provide? | Scope, period, audience, report format, and destination. |
| What should it output? | A structured digest with highlights, risks, and next steps, plus optional delivery to a destination app. |

## Apps and MCP Routing

- Use Rube MCP as the gateway.
- Prefer Linear for project and issue status.
- Add GitHub, Slack, or other source systems only if the report needs them.
- Use Context7 or Tavily only when the report needs grounded external verification.
- Use a destination app such as Slack, email, Notion, or Google Docs when delivery is required.

## User Inputs

| Input | Required | Description |
| --- | --- | --- |
| `report_scope` | yes | Team, project, initiative, label, or issue set |
| `report_period` | yes | Time window such as last 7 days |
| `audience` | yes | Intended audience |
| `report_format` | yes | Short summary, detailed digest, or executive brief |
| `delivery_destination` | no | Optional app or channel to publish into |

## Expected Outputs

- A structured digest with highlights, risks, blockers, and next steps.
- A summary of which systems were queried.
- Optional published copy in the chosen delivery destination.

## Workflow Steps

1. Resolve the report scope and time period.
2. Query the source systems through Rube MCP, usually Linear first.
3. Collect the relevant status signals and normalize them into one digest structure.
4. Synthesize the digest for the requested audience and format.
5. Validate that the summary reflects the gathered evidence.
6. Return or publish the result.

## Validation Rules

- Do not invent project status when source systems have no matching data.
- Include enough evidence in the digest to trace where the conclusions came from.
- If a delivery destination is provided, confirm that the publish step succeeded.
- Make blockers and missing data explicit.

## Fallbacks

- If one source system is unavailable, continue with the remaining systems and mark the gap.
- If the scope resolves to no data, return an empty-but-honest digest rather than a fabricated
  report.
- If delivery fails, return the digest and report the delivery error separately.

## Direct-Create Prompt

```text
Create a new reusable recipe for me.

What the recipe should do:
Gather updates from one or more systems and produce a clean status digest for a selected audience.

Which apps and MCP tools it should use:
Use Rube MCP as the gateway. Prefer Linear for project and issue status. Add GitHub, Slack, or
other source systems only when they are part of the requested report. Use Context7 or Tavily only
when external verification is needed. If delivery is requested, publish through the chosen
destination app after the digest is built.

What inputs users should provide:
- report_scope
- report_period
- audience
- report_format
- delivery_destination

What it should output:
- structured digest with highlights, risks, blockers, and next steps
- summary of which systems were queried
- optional published report

Workflow logic to encode:
1. Resolve the scope and time period.
2. Query the relevant systems through Rube.
3. Normalize the results into one digest structure.
4. Synthesize the digest for the selected audience.
5. Validate the evidence.
6. Return or publish the report.

Validation rules:
Do not claim success unless the digest is grounded in retrieved data and clearly marks missing data
or blockers.

Fallback behavior:
If one source system is unavailable, continue with the remaining systems and label the gap. If
delivery fails, return the digest and report the delivery error.
```

## Execute-Then-Convert Prompt

```text
Execute a status digest workflow with me once, then convert it into a reusable recipe.

Workflow to execute now:
Gather the requested status data and produce a digest for the chosen audience.

Apps and MCP routing:
Use Rube MCP as the gateway. Prefer Linear for project and issue state, then add other systems only
if the report requires them.

Inputs to collect for this run:
- report_scope
- report_period
- audience
- report_format
- delivery_destination

Expected outputs from this run:
- one status digest
- one list of source systems queried
- optional delivery result

Workflow steps to follow:
1. Resolve the scope.
2. Query the source systems.
3. Build the digest.
4. Validate the evidence.
5. Publish if requested.
6. Convert the successful workflow into a reusable recipe.

Validation rules:
Do not convert the workflow into a recipe unless the digest is grounded in retrieved status data and
the output is structured clearly.

Fallback behavior:
If a source system is unavailable, mark the gap and continue when the remaining data is still
sufficient.
```

## MCP-Authoring Prompt

```text
Author a reusable Rube recipe from this workflow specification.

Goal:
Gather project or workstream status from connected systems and produce a digest for a chosen
audience.

Routing:
Use Rube MCP as the gateway. Prefer Linear first. Add other systems only when required by the
report. Use Context7 or Tavily only for external verification.

Inputs:
- report_scope
- report_period
- audience
- report_format
- delivery_destination (optional)

Outputs:
- structured digest
- source-system summary
- optional delivery result

Workflow:
Resolve the scope, gather status signals, normalize them, synthesize the digest, validate the
evidence, and return or publish the result.

Validation:
Require grounded status data and explicit handling of blockers or missing data.

Fallbacks:
Continue with partial data only when the missing system is disclosed clearly.
```

## Worked Example

Example paste into the Rube UI:

```text
Create a recipe that builds a weekly status digest from project systems.

What the recipe should do:
Gather project and issue updates for a chosen time period and produce a structured digest for a
specific audience.

Which apps does it involve:
Use Rube MCP as the gateway. Prefer Linear for project status, add GitHub if code activity is part
of the digest, and post the final digest to a chosen destination app if requested.

What inputs should users provide:
- report_scope
- report_period
- audience
- report_format
- delivery_destination

What should it output:
- structured digest with highlights, blockers, and next steps
- optional delivered copy

Create it as a reusable recipe with neutral wording, explicit inputs, explicit outputs, and
self-contained workflow logic.
```
