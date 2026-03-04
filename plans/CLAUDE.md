# Implementation Plans

Phased implementation plans for building Mister Smith from the specifications in `spec/`.

## Batch Status

| Batch | Domain | Agents | Status |
|-------|--------|--------|--------|
| 1 | Core Architecture | 7 (01-02, 04-08) | Complete (agent03 consolidated into async-patterns) |
| 2 | Data Management | 1 (16) | Partial — 1 of ~8 agents |
| 3-5 | Security, Transport, Operations | — | Not started |

## Usage

1. Read the relevant spec domain first (`spec/core-architecture/`, etc.)
2. Then read the corresponding implementation plan
3. Plans reference specific spec files — follow those links for full context

## Key Files

- `IMPLEMENTATION_PLANNING_TRACKER.md` — Overall progress and batch status
- `roadmap-phases/` — Phase-by-phase roadmap deep dives (scope, dependencies, gates, validation)
- `batch1-core-architecture/` — 8 agent plans covering system architecture through core integration
- `batch2-data-management/` — Data flow integration (partial)
