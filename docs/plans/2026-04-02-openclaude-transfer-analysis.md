# 2026-04-02 OpenClaude Transfer Analysis

## Objective

Analyze `/Users/macmain/openclaude` for concrete product and implementation ideas that could
improve Mister Smith, then package the findings as repo-local markdown docs that clearly separate
current Mister Smith truth from transferable external ideas.

## Scope

- read-only analysis of `/Users/macmain/openclaude`
- repo-grounded comparison against Mister Smith current-state, direction, and research corpus
- one new documentation directory under `docs/research-output/analysis/`
- topic docs that explain each feature, why it matters, how real it looks in code, and how it
  could translate into Mister Smith

## Assumptions

- this is transfer analysis only, not a product-claim or implementation pass
- `docs/current-state.md` remains the authority for what is true on Mister Smith `main`
- external ideas should be filtered for Mister Smith's product boundary instead of copied whole
- features that belong only to repo-development workflow tools should not be described as Mister
  Smith OS runtime features

## Constraints

- no changes to Mister Smith runtime code, specs, or control-plane state
- no drift from the repo's product boundary or live-proof language
- no marketing-only feature list; every recommendation must point to concrete code evidence in
  `openclaude`

## Non-Goals

- implementing any of the findings
- benchmarking `openclaude` against Mister Smith
- rewriting existing direction docs or claiming new roadmap commitments

## Milestones

### Milestone 1: Map `openclaude`

Validation:

- inspect repo structure, README, and the key source modules behind the most important surfaces
- collect subagent findings for core execution, provider/tooling, and UX/operator flows

### Milestone 2: Judge Fit Against Mister Smith

Validation:

- compare candidate ideas against `docs/current-state.md`, `docs/direction.md`, and the research
  corpus layer rules
- reject features that do not fit Mister Smith's product boundary or current architecture posture

### Milestone 3: Write Transfer Package

Validation:

- create a new dated directory under `docs/research-output/analysis/`
- include an index plus topic markdown docs with feature, evidence, Mister Smith fit, translation
  path, and risks/non-fits

### Milestone 4: Close With Narrow Proof

Validation:

- `npx markdownlint-cli2 "docs/plans/2026-04-02-openclaude-transfer-analysis.md" "docs/research-output/analysis/2026-04-02-openclaude-transfer/**/*.md" --config .markdownlint.json`
- `git diff --check`

## Stop Conditions

- stop if the analysis cannot distinguish real code-backed features from README-only claims
- stop if the proposed translations would force Mister Smith to copy repo-workflow tooling into the
  OS product boundary
- stop if the output starts replacing current-state truth instead of serving as supporting transfer
  analysis
