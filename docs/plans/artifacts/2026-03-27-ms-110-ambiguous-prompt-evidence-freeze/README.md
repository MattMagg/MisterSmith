# MS-110 Ambiguous Prompt Evidence Freeze Artifacts

This lane captures Milestone 1 from `MS-110`: three bounded live runs that test whether ambiguous
non-explicit prompts still drift into unnecessary branching on current `main`.

## Cases

- `trust-first/20260327T184924Z/`
  - prompt asks which operator surface to trust first when task and autonomy differ in emphasis
  - observed result: `Sequential`, `parallelism_width = 1`, `branch_count = 1`, `node_count = 3`
- `three-axis-compare/20260327T185027Z/`
  - prompt asks for one answer comparing provider/model, graph shape, and result provenance
  - observed result: `Sequential`, `parallelism_width = 1`, `branch_count = 1`, `node_count = 2`
- `readiness-vs-result/20260327T185140Z/`
  - prompt asks whether readiness and terminal-result evidence can be summarized in one bounded
    answer without splitting the work
  - observed result: `Sequential`, `parallelism_width = 1`, `branch_count = 1`, `node_count = 2`

## Runtime Path

- provider: `openai_chatgpt`
- model: `gpt-5.4`
- ingress: `POST /api/v1/tasks`
- autonomy route: `GET /api/v1/autonomy/status/{workflow_id}`
- harness: `python3 scripts/live_runtime_proof_smoke.py --profile baseline --scenario non_memo --task-description ...`

## Related Note

- `docs/plans/2026-03-27-ms-110-ambiguous-prompt-evidence-freeze.md`
