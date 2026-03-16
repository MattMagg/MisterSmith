---
version: R1
created: 2026-03-16
updated: 2026-03-16
type: prompt-archetype
---

# Rube Recipe Archetype 05: Cross-App Intake to Linear Routing

## Goal

Create a reusable recipe that reads intake from one app, evaluates it against routing rules, and
then creates, updates, or links the right Linear issue.

Rube UI answers:

| UI question | Answer |
| --- | --- |
| What should the recipe do? | Read inbound items from another app and route them into Linear. |
| Which apps does it involve? | Rube MCP as the gateway, one source app, and Linear as the destination. |
| What inputs should users provide? | Source app, intake selector, routing rules, and destination project or team rules. |
| What should it output? | Created or updated Linear issue details plus a routing summary. |

## Apps and MCP Routing

- Use Rube MCP as the gateway.
- Use one source app such as Slack, Gmail, Zendesk, Intercom, Typeform, or another intake source
  connected through Rube.
- Use Linear as the routing destination and system of record.
- Use Context7 or Tavily only if routing rules depend on verified docs or product behavior.

## User Inputs

| Input | Required | Description |
| --- | --- | --- |
| `source_app` | yes | The intake source app |
| `intake_selector` | yes | Message, thread, inbox query, form response, or similar selector |
| `routing_rules` | yes | Rules for whether to create, update, or link a Linear issue |
| `destination_rules` | yes | Target team, project, state, or labeling rules |
| `notify_destination` | no | Optional place to send the routing result |

## Expected Outputs

- Created, updated, or linked Linear issue data.
- A routing summary explaining what action was taken and why.
- Optional delivery of the routing result to another app.

## Workflow Steps

1. Resolve the source app and intake selector.
2. Fetch the intake item through Rube.
3. Apply the routing rules to determine whether to create, update, or link a Linear issue.
4. Apply the destination rules in Linear.
5. Validate that the issue action succeeded.
6. Return the issue details and routing summary.
7. Notify a downstream destination if requested.

## Validation Rules

- Do not create duplicate Linear issues when the routing rules call for linking or updating.
- Do not route an item if the selector does not resolve unambiguously.
- Return the resulting issue identifier and action taken.
- If notification delivery fails, report it separately from the Linear action result.

## Fallbacks

- If the source selector is ambiguous, stop and ask for clarification.
- If the routing rules do not match any valid destination behavior, return a blocked result instead
  of improvising.
- If documentation verification is needed and unavailable, report that before applying the route.

## Direct-Create Prompt

```text
Create a new reusable recipe for me.

What the recipe should do:
Read intake from a connected source app, evaluate routing rules, and then create, update, or link
the correct Linear issue.

Which apps and MCP tools it should use:
Use Rube MCP as the gateway. Use one source app such as Slack, Gmail, Zendesk, Intercom, or
Typeform for intake. Use Linear as the destination and source of truth for issue creation and
updates. Use Context7 or Tavily only if routing logic depends on verified documentation.

What inputs users should provide:
- source_app
- intake_selector
- routing_rules
- destination_rules
- notify_destination

What it should output:
- created or updated Linear issue details
- routing summary that explains the action taken
- optional notification result

Workflow logic to encode:
1. Resolve the intake source and item selector.
2. Fetch the intake item.
3. Apply the routing rules.
4. Create, update, or link the correct Linear issue.
5. Validate the result.
6. Return the issue identifier and routing summary.
7. Notify a downstream destination if requested.

Validation rules:
Do not create duplicates when the routing rules indicate that an existing issue should be updated or
linked. Do not claim success unless the resulting issue action succeeded.

Fallback behavior:
Stop on ambiguous selectors or invalid routing rules. Report unavailable required verification
sources before applying the route.
```

## Execute-Then-Convert Prompt

```text
Execute a cross-app intake to Linear routing workflow with me once, then convert it into a reusable
recipe.

Workflow to execute now:
Read an intake item from a connected source app, apply routing rules, and create, update, or link
the correct Linear issue.

Apps and MCP routing:
Use Rube MCP as the gateway. Use the selected source app for intake and Linear as the destination.
Use Context7 or Tavily only when the routing logic requires verified documentation.

Inputs to collect for this run:
- source_app
- intake_selector
- routing_rules
- destination_rules
- notify_destination

Expected outputs from this run:
- created or updated Linear issue details
- routing summary
- optional notification result

Workflow steps to follow:
1. Resolve the intake item.
2. Apply the routing rules.
3. Perform the Linear action.
4. Validate the result.
5. Convert the successful workflow into a reusable recipe.

Validation rules:
Do not convert the workflow into a recipe unless the intake item was resolved correctly and the
Linear action succeeded without duplication.

Fallback behavior:
Stop on ambiguous intake selectors or invalid routing rules.
```

## MCP-Authoring Prompt

```text
Author a reusable Rube recipe from this workflow specification.

Goal:
Read intake from a connected source app and route it into Linear using explicit create, update, or
link rules.

Routing:
Use Rube MCP as the gateway. Use the chosen source app for intake and Linear for issue actions.

Inputs:
- source_app
- intake_selector
- routing_rules
- destination_rules
- notify_destination (optional)

Outputs:
- issue action result
- routing summary
- optional notification result

Workflow:
Resolve the intake item, apply routing rules, perform the Linear action, validate the issue result,
and then return or notify.

Validation:
Require unambiguous intake resolution and explicit duplicate-avoidance behavior.

Fallbacks:
Stop on ambiguous selectors, invalid routing rules, or unavailable required verification sources.
```

## Worked Example

Example paste into the Rube UI:

```text
Create a recipe that routes inbound Slack or Gmail items into Linear.

What the recipe should do:
Read an inbound item from a connected source app, apply user-provided routing rules, and then
create, update, or link the correct Linear issue.

Which apps does it involve:
Use Rube MCP as the gateway. Use a source app such as Slack or Gmail for intake and Linear as the
destination for issue actions.

What inputs should users provide:
- source_app
- intake_selector
- routing_rules
- destination_rules
- notify_destination

What should it output:
- Linear issue details
- routing summary
- optional notification result

Create it as a reusable recipe with neutral wording, explicit inputs, explicit outputs, and
self-contained workflow logic.
```
