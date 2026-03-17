# Mister Smith Codex Agent Roster

This repo ships a project-scoped Codex subagent system for Mister Smith work.

## Defaults

- Repo default fan-out: `max_threads = 24`, `max_depth = 4`
- Burst profile: `smith-burst` raises the cap to `32 / 6`
- CSV batch jobs default to `3600` seconds; `smith-burst` raises that to `5400`
- Parent thread owns durable control-plane mutations:
  - `save_linear_issue`
  - `save_issue_workpad`
  - `apply_queue_stage`
  - PR merge/push/land
  - final issue state transitions

Subagents may own repo file edits and local validation within bounded scopes.

## Agent Matrix

| Agent | Model | Sandbox | Primary job |
| --- | --- | --- | --- |
| `smith_repo_grounder` | `gpt-5.3-codex-spark` | `read-only` | Repo grounding, touched crates, source-of-record docs |
| `smith_control_plane_auditor` | `gpt-5.4` | `read-only` | Smith, Linear, PR, queue, and lifecycle evidence |
| `smith_docs_researcher` | `gpt-5.3-codex-spark` | `read-only` | OpenAI, Linear, framework, and library docs |
| `smith_slice_planner` | `gpt-5.4` | `read-only` | Bounded slices, blockers, acceptance, validation |
| `smith_frontier_guard` | `gpt-5.4` | `read-only` | Frontier legitimacy and operating-system leverage |
| `smith_speckit_router` | `gpt-5.4` | `read-only` | SpecKit entry and task-pack routing |
| `smith_crate_worker` | `gpt-5.4` | `workspace-write` | One crate or one disjoint write set |
| `smith_validator` | `gpt-5.3-codex-spark` | `workspace-write` | Build/test/proof execution and failure isolation |
| `smith_reviewer` | `gpt-5.4` | `read-only` | Findings-first review |
| `smith_ralph_packet_builder` | `gpt-5.4` | `workspace-write` | Fresh Ralph prompt/packet context |

## Orchestration Recipes

### Kickoff

Use:

- `smith_repo_grounder`
- `smith_control_plane_auditor`
- `smith_docs_researcher` when external docs matter

Goal: produce the grounded brief before planning or editing.

### Backlog Or Queue Planning

Use:

- `smith_frontier_guard`
- `smith_slice_planner`
- `smith_control_plane_auditor`

Goal: decide whether the work is legitimate, bounded, and stage-worthy. The parent thread still
decides whether to mutate queue or issue state.

### Implementation

Use:

- one `smith_crate_worker` per disjoint write scope
- `smith_validator`
- `smith_reviewer`

Rules:

- Do not overlap write scopes unless the parent explicitly coordinates it.
- Let the parent handle durable control-plane writes and finalization.

### Ralph

Use:

- `smith_ralph_packet_builder`
- `smith_validator`

Goal: regenerate fresh Ralph context from current issue/workpad state and verify the results.

### SpecKit

Use:

- `smith_speckit_router`
- `smith_slice_planner`

Goal: route into SpecKit or translate task packs without prematurely applying mutations.

## CSV Batch Recipe

Use this when you have many similar files, issues, or services to audit.

Example prompt:

```text
Create /tmp/smith-audit.csv with columns item_id,path,focus and one row per work item.

Then call spawn_agents_on_csv with:
- csv_path: /tmp/smith-audit.csv
- id_column: item_id
- instruction: "Review {path} with focus {focus}. Use smith_reviewer behavior. Return JSON with keys item_id, risk, summary, and follow_up via report_agent_job_result."
- output_csv_path: /tmp/smith-audit-results.csv
- output_schema: object with required string fields item_id, risk, summary, follow_up
```

Good uses:

- reviewing one crate or file cluster per row
- checking many backlog items for stage readiness
- verifying many external API surfaces or migration targets
