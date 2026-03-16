---
version: R1
created: 2026-03-16
updated: 2026-03-16
type: prompt-archetype
---

# Rube Recipe Archetype 03: Linear Triage and Update

## Goal

Create a reusable recipe that inspects a Linear issue or intake item, decides on the right next
update, and then applies or drafts the update in a structured way.

Rube UI answers:

| UI question | Answer |
| --- | --- |
| What should the recipe do? | Triage a Linear issue or candidate issue and make a structured update. |
| Which apps does it involve? | Rube MCP as the gateway, Linear as the source of truth, and optional research tools if the issue needs verification. |
| What inputs should users provide? | Issue identifier or search query, intended action, triage notes, and optional policy constraints. |
| What should it output? | Updated issue state or drafted change set, plus a summary of what changed. |

## Apps and MCP Routing

- Use Rube MCP as the gateway.
- Use Linear for issue state, project state, labels, comments, and assignments.
- Use Context7 or Tavily only when the issue depends on product or technical documentation that
  must be verified before making the update.
- Allow an optional destination app for status notifications after the Linear update.

## User Inputs

| Input | Required | Description |
| --- | --- | --- |
| `issue_selector` | yes | Linear issue identifier or search query |
| `triage_action` | yes | Intended action, such as classify, summarize, comment, update state, or relabel |
| `triage_notes` | no | Extra context or notes to include in the update |
| `policy_constraints` | no | Rules such as allowed states, label constraints, or assignee rules |
| `notify_destination` | no | Optional place to send a status notification |

## Expected Outputs

- Updated issue fields, drafted issue updates, or a draft comment.
- A concise summary of the decision and the change applied.
- Optional notification to a downstream app.

## Workflow Steps

1. Resolve the target issue or issues in Linear.
2. Read the current state, labels, description, and recent comments.
3. Determine the requested triage action and any policy constraints.
4. If documentation verification is needed, route through Context7 or Tavily before making the
   update.
5. Apply or draft the Linear update.
6. Validate that the resulting state matches the intended action.
7. Return a concise summary and optionally notify a downstream app.

## Validation Rules

- Do not update the wrong issue because of ambiguous selectors.
- Do not claim success unless the issue update or comment was actually created.
- If the action is blocked by policy constraints, return a blocked result instead of improvising.
- Include the issue identifier in the final result.

## Fallbacks

- If the selector is ambiguous, stop and ask for clarification.
- If documentation is required and Context7 is unavailable, continue only if the user allows a
  non-docs-based triage path.
- If notification delivery fails, still return the Linear result and report the notification
  failure separately.

## Direct-Create Prompt

```text
Create a new reusable recipe for me.

What the recipe should do:
Inspect a Linear issue or candidate issue, perform a structured triage action, and return a clear
summary of what changed.

Which apps and MCP tools it should use:
Use Rube MCP as the gateway. Use Linear as the source of truth for issues, labels, comments,
states, and assignees. Use Context7 or Tavily only when the update depends on verified product or
technical documentation. Optionally send a follow-up notification through another app after the
Linear action succeeds.

What inputs users should provide:
- issue_selector
- triage_action
- triage_notes
- policy_constraints
- notify_destination

What it should output:
- the updated issue state, comment, or draft update
- a concise summary of the decision and change
- optional notification result

Workflow logic to encode:
1. Resolve the issue in Linear.
2. Read its current state and recent context.
3. Determine the requested triage action.
4. Verify any documentation-dependent claims if needed.
5. Apply or draft the Linear update.
6. Validate the result.
7. Return the issue identifier and a summary.

Validation rules:
Do not claim success unless the intended Linear change was applied or drafted successfully. Include
the issue identifier and the resulting state in the output.

Fallback behavior:
If the issue selector is ambiguous, stop and ask for clarification. If documentation verification is
needed and unavailable, report the limitation instead of guessing.
```

## Execute-Then-Convert Prompt

```text
Execute a Linear triage workflow with me once, then convert it into a reusable recipe.

Workflow to execute now:
Find the target Linear issue, apply the requested triage action, and return a concise summary.

Apps and MCP routing:
Use Rube MCP as the gateway. Use Linear as the source of truth. Use Context7 or Tavily only if the
triage action depends on verified documentation.

Inputs to collect for this run:
- issue_selector
- triage_action
- triage_notes
- policy_constraints
- notify_destination

Expected outputs from this run:
- updated or drafted Linear change
- concise decision summary
- optional notification result

Workflow steps to follow:
1. Resolve the issue.
2. Read the current issue context.
3. Perform the requested triage action.
4. Validate the result.
5. Convert the successful workflow into a reusable recipe.

Validation rules:
Do not convert the run into a recipe unless the issue was resolved correctly and the change was
applied or drafted successfully.

Fallback behavior:
If the issue selector is ambiguous, stop. If documentation verification is needed and unavailable,
report the limitation before any conversion.
```

## MCP-Authoring Prompt

```text
Author a reusable Rube recipe from this workflow specification.

Goal:
Triage a Linear issue or candidate issue and apply or draft a structured update.

Routing:
Use Rube MCP as the gateway. Use Linear as the source of truth. Use Context7 or Tavily only when
the update requires verified documentation.

Inputs:
- issue_selector
- triage_action
- triage_notes (optional)
- policy_constraints (optional)
- notify_destination (optional)

Outputs:
- updated issue state, comment, or draft
- concise summary with issue identifier
- optional notification result

Workflow:
Resolve the issue, inspect its context, apply the requested triage action, validate the change, and
return a structured summary.

Validation:
Require unambiguous issue resolution and confirmation that the Linear action succeeded.

Fallbacks:
Stop on ambiguous selectors or unavailable required verification sources.
```

## Worked Example

Example paste into the Rube UI:

```text
Create a recipe that triages a Linear issue and applies a structured update.

What the recipe should do:
Take a Linear issue identifier or search query, inspect the issue, and then update its state,
labels, or comments based on a user-provided triage action.

Which apps does it involve:
Use Rube MCP as the gateway. Use Linear as the source of truth, and use Context7 or Tavily only if
the update depends on verified documentation.

What inputs should users provide:
- issue_selector
- triage_action
- triage_notes
- policy_constraints
- notify_destination

What should it output:
- updated issue details or drafted update
- concise summary with the issue identifier

Create it as a reusable recipe with neutral wording, explicit inputs, explicit outputs, and
self-contained workflow logic.
```
