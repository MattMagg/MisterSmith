# Implementation Plans

Phased implementation plans for building Mister Smith from the specifications in `spec/`.

## Batch Status

| Batch | Domain | Agents | Status |
|-------|--------|--------|--------|
| 1 | Core Architecture | 7 (01-02, 04-08) | Complete |
| 2 | Data Management | 1 (16) | Partial |
| 3-5 | Security, Transport, Operations | — | Not started |

## SpecKit Feature Directories

Implementation specs are generated via SpecKit into `specs/`:
- `specs/001-phase1-foundation/` — Phase 1 spec, plan, tasks (all complete)
- `specs/002-phase2-runtime-async/` — Phase 2 spec, plan, tasks (all complete)

## Usage

1. Read the relevant spec domain first (`spec/core-architecture/`, etc.)
2. Then read the corresponding implementation plan
3. Plans reference specific spec files — follow those links for full context

## Key Files

- `IMPLEMENTATION_PLANNING_TRACKER.md` — Overall progress and batch status
- `roadmap-phases/` — Phase-by-phase roadmap deep dives (scope, dependencies, gates, validation)
- `roadmap-phases/phase-*-plan.md` — SpecKit implementation plans
- `roadmap-phases/phase-*-tasks.md` — SpecKit task breakdowns (checked off as completed)
- `batch1-core-architecture/` — 8 agent plans covering system architecture through core integration
- `batch2-data-management/` — Data flow integration (partial)
