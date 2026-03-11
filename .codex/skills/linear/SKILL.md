---
name: linear
description: |
  Use when raw Linear GraphQL is required during Symphony-managed sessions and
  the Mister Smith constitutional control-plane MCP does not expose the needed
  operation.
---

# Linear GraphQL

Use this skill as an escape hatch for direct Linear GraphQL work inside Symphony-managed Codex
sessions.

Default routing order:

1. use `mistersmith_control_plane` MCP first
2. use `$mister-smith-control-plane-router` if the request is ambiguous
3. use this skill only when the control-plane MCP does not expose the required Linear operation

## Primary tool

Use the `linear_graphql` client tool exposed by Symphony. It reuses the session's configured Linear auth.

Tool input:

```json
{
  "query": "query or mutation document",
  "variables": {
    "optional": "graphql variables object"
  }
}
```

Rules:

- prefer the control-plane MCP for queue staging, runtime reconciliation, legitimacy, review-dispatch, and phase planning
- Send one GraphQL operation per tool call.
- Treat a top-level `errors` array as failure even if the tool call itself succeeds.
- Keep reads and mutations narrow.

## Common queries

### Read an issue by key

```text
query IssueByKey($key: String!) {
  issue(id: $key) {
    id
    identifier
    title
    description
    url
    branchName
    state {
      id
      name
      type
    }
    project {
      id
      name
    }
    attachments {
      nodes {
        id
        title
        url
        sourceType
      }
    }
  }
}
```

### Read team workflow states

```text
query IssueTeamStates($id: String!) {
  issue(id: $id) {
    id
    team {
      id
      key
      name
      states {
        nodes {
          id
          name
          type
        }
      }
    }
  }
}
```

### Create or update the workpad comment

```text
mutation CreateComment($issueId: String!, $body: String!) {
  commentCreate(input: { issueId: $issueId, body: $body }) {
    success
    comment {
      id
      url
    }
  }
}
```

```text
mutation UpdateComment($id: String!, $body: String!) {
  commentUpdate(id: $id, input: { body: $body }) {
    success
    comment {
      id
      body
    }
  }
}
```

### Move an issue to another state

```text
mutation MoveIssueToState($id: String!, $stateId: String!) {
  issueUpdate(id: $id, input: { stateId: $stateId }) {
    success
    issue {
      id
      identifier
      state {
        id
        name
      }
    }
  }
}
```

### Attach a GitHub PR

```text
mutation AttachGitHubPR($issueId: String!, $url: String!, $title: String) {
  attachmentLinkGitHubPR(
    issueId: $issueId
    url: $url
    title: $title
    linkKind: links
  ) {
    success
    attachment {
      id
      title
      url
    }
  }
}
```

## Discovery

When you do not know an input type or mutation shape, use targeted schema introspection:

```text
query ListMutations {
  __type(name: "Mutation") {
    fields {
      name
    }
  }
}
```
