# ChatGPT Pulse — Daily Research Tasks for MisterSmith

## What Is This?

This directory contains 9 ChatGPT Pulse task documents. Each is a self-contained prompt you paste into ChatGPT to create a recurring daily research task that monitors a specific domain for developments relevant to Mister Smith's forward roadmap.

## What Is ChatGPT Pulse?

Pulse is OpenAI's proactive daily research agent (ChatGPT Pro, $200/mo). It runs scheduled research overnight and delivers personalized briefings as 5-10 visual cards. Tasks can be one-off or recurring, managed in the Pulse section of ChatGPT.

## How To Use

1. Open ChatGPT (Pro account required)
2. Copy the full contents of a Pulse document
3. Paste it into ChatGPT and say: "Create this as a daily Pulse task, scheduled for 6:00 AM"
4. ChatGPT creates the recurring task in Pulse
5. Each morning, check Pulse for your briefing cards
6. Save actionable findings or ask follow-up questions directly

## Task Inventory

| File | Domain | Feeds Phases | Schedule |
|------|--------|-------------|----------|
| `pulse-01-llm-routing-economics.md` | LLM routing, inference costs, speculative decoding | Phase 9, 10, 14 | Daily |
| `pulse-02-competitive-intelligence.md` | Agent framework releases, benchmarks, deployments | All | Daily |
| `pulse-03-security-and-trust.md` | Attack vectors, defenses, capability security | Phase 9.1 | Daily |
| `pulse-04-dynamic-orchestration.md` | Meta-orchestration, topology compilers, RL orchestration | Phase 11 | Daily |
| `pulse-05-crdt-formal-verification.md` | CRDTs, session types, formal protocol verification | Phase 13 | Daily |
| `pulse-06-predictive-supervision.md` | Agent profiling, cognitive coordination, predictive faults | Phase 12 | Daily |
| `pulse-07-rust-ai-ecosystem.md` | Rust crates, async-nats, Tokio, inference runtimes | All | Daily |
| `pulse-08-memory-context-engineering.md` | Neural paging, KV cache, context compression, tiered memory | Phase 14 | Daily |
| `pulse-09-cross-domain-paradigm-shifts.md` | Neuroscience, control theory, swarm robotics, game theory | Discovery | Daily |

## Feeding Results Back

When a Pulse card surfaces an actionable finding:
1. Save the finding to `docs/research-output/inbox/` as a markdown file
2. Name it: `YYYY-MM-DD-pulse-NN-brief-description.md`
3. Include the full card content plus any follow-up questions you asked
4. These get routed into the research corpus during the next research round

## Refresh Cadence

The "What Is Already Known" baseline sections in each document will become stale as the research corpus grows. Refresh them:
- **Quarterly**: Review all 9 baselines against consolidated synthesis docs
- **On new research round**: When a new round (R8+) lands, update the affected baselines
- **On phase completion**: When a roadmap phase ships, update the affected task to track the next frontier

## Relationship to Research Prompts

The `docs/research-prompts/` directory contains deep one-shot research prompts for comprehensive multi-hour investigations. These Pulse tasks serve a different purpose: **daily incremental monitoring** for evolving fields. They complement, not replace, deep research rounds.

## Template

The reusable template is at `docs/prompt-improver-spec/final-prompts/pulse-task-template.md`. Use it to create additional Pulse tasks for new domains as the roadmap expands.
